use ingestion::bundle::{self, Bundle, Entry};
use sqlx::PgPool;

fn input() -> Bundle {
    let fixtures: Vec<ingestion::Fixture> =
        serde_json::from_str(include_str!("../../../data/development.json")).unwrap();
    Bundle {
        schema_version: 1,
        source_commit: "a".repeat(40),
        records: fixtures
            .into_iter()
            .map(|f| {
                let mut draft = f.draft;
                // Original synthetic test content is marked publishable only inside this test.
                draft.synthetic = false;
                Entry {
                    id: f.id,
                    draft,
                    external_ids: vec![],
                }
            })
            .collect(),
    }
}

#[test]
fn reproducible_build_checks_manifest_integrity_and_relations() {
    let source = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let mut bundle = input();
    std::fs::write(
        source.path().join("records.json"),
        serde_json::to_vec(&bundle.records).unwrap(),
    )
    .unwrap();
    let manifest = bundle::build(source.path(), first.path(), &bundle.source_commit).unwrap();
    bundle.records.reverse();
    std::fs::write(
        source.path().join("records.json"),
        serde_json::to_vec_pretty(&bundle.records).unwrap(),
    )
    .unwrap();
    let again = bundle::build(source.path(), second.path(), &bundle.source_commit).unwrap();
    assert_eq!(manifest.sha256, again.sha256);
    let loaded = bundle::load(&first.path().join("manifest.json")).unwrap();
    assert_eq!(loaded.records.len(), bundle.records.len());
    std::fs::write(first.path().join(manifest.filename), b"{}").unwrap();
    assert!(bundle::load(&first.path().join("manifest.json")).is_err());
    bundle.records[0].draft.synthetic = true;
    assert!(bundle.validate().is_err());
    bundle = input();
    bundle.records.push(bundle.records[0].clone());
    assert!(bundle.validate().is_err());
    bundle = input();
    bundle
        .records
        .retain(|e| e.draft.kind != catalogue::Kind::Game);
    assert!(bundle.validate().is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn indexing_is_atomic_idempotent_and_detects_local_conflicts(pool: PgPool) {
    let mut bundle = input();
    let dry = bundle::index(&pool, &bundle, false).await.unwrap();
    assert_eq!(dry.inserted, bundle.records.len());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM catalogue_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let applied = bundle::index(&pool, &bundle, true).await.unwrap();
    assert_eq!(applied.inserted, bundle.records.len());
    let repeat = bundle::index(&pool, &bundle, true).await.unwrap();
    assert_eq!(repeat.unchanged, bundle.records.len());
    let events: i64 = sqlx::query_scalar("SELECT count(*) FROM cache_invalidation_outbox")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(events, 1);
    let card = bundle
        .records
        .iter_mut()
        .find(|r| r.draft.kind == catalogue::Kind::Card)
        .unwrap();
    let id = card.id.clone();
    card.draft.name = "Original Flabébé ♀ correction".into();
    bundle.source_commit = "b".repeat(40);
    assert_eq!(
        bundle::index(&pool, &bundle, true).await.unwrap().updated,
        1
    );
    sqlx::query("UPDATE catalogue_records SET updated_at=clock_timestamp() WHERE id=$1")
        .bind(&id)
        .execute(&pool)
        .await
        .unwrap();
    assert!(bundle::index(&pool, &bundle, true).await.is_err());
    let events: i64 = sqlx::query_scalar("SELECT count(*) FROM cache_invalidation_outbox")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(events, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn retirement_removes_only_managed_records_and_first_adoption_is_safe(pool: PgPool) {
    let bundle = input();
    let mut tx = pool.begin().await.unwrap();
    for entry in &bundle.records {
        ingestion::write_record(&mut tx, &entry.id, &entry.draft)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    let mut conflict = bundle.clone();
    conflict.records[0].draft.name = "Different preexisting record".into();
    assert!(bundle::index(&pool, &conflict, true).await.is_err());
    assert_eq!(
        bundle::index(&pool, &bundle, true).await.unwrap().unchanged,
        bundle.records.len()
    );
    let empty = Bundle {
        records: vec![],
        ..bundle
    };
    assert!(bundle::index(&pool, &empty, false).await.unwrap().retired > 0);
    assert!(bundle::index(&pool, &empty, true).await.unwrap().retired > 0);
    assert_eq!(bundle::index(&pool, &empty, true).await.unwrap().retired, 0);
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM catalogue_records WHERE deleted_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn indexed_download_is_the_exact_ci_artifact(pool: PgPool) {
    let source = tempfile::tempdir().unwrap();
    let built = tempfile::tempdir().unwrap();
    let downloads = tempfile::tempdir().unwrap();
    let bundle = input();
    std::fs::write(
        source.path().join("records.json"),
        serde_json::to_vec(&bundle.records).unwrap(),
    )
    .unwrap();
    let manifest = bundle::build(source.path(), built.path(), &bundle.source_commit).unwrap();
    let path = built.path().join("manifest.json");
    cardlattice_api::catalogue_index::import(&pool, &path, downloads.path(), false)
        .await
        .unwrap();
    assert!(!downloads.path().join(&manifest.filename).exists());
    cardlattice_api::catalogue_index::import(&pool, &path, downloads.path(), true)
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(built.path().join(&manifest.filename)).unwrap(),
        std::fs::read(downloads.path().join(&manifest.filename)).unwrap()
    );
    assert_eq!(
        cardlattice_api::catalogue_index::import(&pool, &path, downloads.path(), true)
            .await
            .unwrap()
            .unchanged,
        bundle.records.len()
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM publication_manifests")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
