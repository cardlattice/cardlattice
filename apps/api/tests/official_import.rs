use catalogue::Draft;
use chrono::{NaiveDate, Utc};
use ingestion::official_pokemon::{OfficialBatch, SetCandidate, TERMS_URL, import};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const FIXTURES: &str = include_str!("../../../data/development.json");
const GAME: &str = "01900000-0000-7000-8000-000000000001";
const DEMO_SET: &str = "01900000-0000-7000-8000-000000000003";
const DEMO_CARD: &str = "01900000-0000-7000-8000-000000000004";

// Original parser/import fixtures only; these are not claims about real products.
fn batch() -> OfficialBatch {
    OfficialBatch {
        schema_version: 1,
        reviewed_by: Some("original integration fixture".into()),
        terms_url: TERMS_URL.into(),
        sets: vec![SetCandidate {
            name: "Original import test set".into(),
            release_date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            source_url: "https://press.pokemon.com/en/products/Original-import-test-set".into(),
            retrieved_at: Utc::now(),
            content_sha256: "a".repeat(64),
        }],
    }
}

async fn scalar(pool: &PgPool, query: &str) -> i64 {
    sqlx::query_scalar(query).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn official_import_is_atomic_audited_and_idempotent(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let reviewed = batch();
    let preview = import(&pool, &reviewed, false, true).await.unwrap();
    assert!(!preview.applied);
    assert!(preview.game_promoted);
    assert_eq!(preview.inserted_sets, 1);
    assert_eq!(preview.retired_synthetic_records, 6);
    assert_eq!(
        scalar(
            &pool,
            "SELECT count(*) FROM catalogue_records WHERE deleted_at IS NULL"
        )
        .await,
        7
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_imports").await,
        0
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_revisions").await,
        7
    );

    let result = import(&pool, &reviewed, true, true).await.unwrap();
    assert!(result.applied);
    assert_eq!(result.retired_synthetic_records, 6);
    assert_eq!(
        scalar(
            &pool,
            "SELECT count(*) FROM catalogue_records WHERE deleted_at IS NULL AND synthetic"
        )
        .await,
        0
    );
    assert_eq!(
        scalar(
            &pool,
            "SELECT count(*) FROM catalogue_records WHERE deleted_at IS NULL"
        )
        .await,
        2
    );
    let set_id: Uuid = sqlx::query_scalar("SELECT id FROM sets")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_ne!(set_id.to_string(), DEMO_SET);
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_imports").await,
        1
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_import_revisions").await,
        8
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_revisions").await,
        15
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM cache_invalidation_outbox").await,
        15
    );
    assert_eq!(scalar(&pool, "SELECT count(*) FROM provenance p JOIN sources s ON s.id=p.source_id WHERE s.method='permitted_official_reference' AND p.field_path='$.releaseDate'").await, 2);
    let before: Value = sqlx::query_scalar("SELECT before_data FROM catalogue_revisions WHERE record_id=$1 AND before_data IS NOT NULL").bind(Uuid::parse_str(DEMO_SET).unwrap()).fetch_one(&pool).await.unwrap();
    assert_eq!(before["synthetic"], true);

    let repeated = import(&pool, &reviewed, true, true).await.unwrap();
    assert_eq!(repeated.unchanged_sets, 1);
    assert_eq!(repeated.retired_synthetic_records, 0);
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_revisions").await,
        15
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_imports").await,
        1
    );
    // Explicit seeding cannot bring retired fixtures back on a later restart.
    assert_eq!(ingestion::seed(&pool, FIXTURES).await.unwrap(), 0);

    sqlx::query("UPDATE catalogue_records SET attributes=$1 WHERE id=$2")
        .bind(json!({"independentlyRecorded":true}))
        .bind(set_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut corrected = reviewed;
    corrected.sets[0].release_date = NaiveDate::from_ymd_opt(2026, 1, 3).unwrap();
    assert_eq!(
        import(&pool, &corrected, true, false)
            .await
            .unwrap()
            .updated_sets,
        1
    );
    let actual: (Uuid, Value) = sqlx::query_as("SELECT id,attributes FROM sets")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(actual.0, set_id);
    assert_eq!(actual.1, json!({"independentlyRecorded":true}));

    let directory = tempfile::tempdir().unwrap();
    let config = publishing::PublishingConfig {
        directory: directory.path().into(),
        public_origin: "http://localhost".into(),
        cloudflare: None,
    };
    let manifest = publishing::publish_snapshot(&pool, &config).await.unwrap();
    assert_eq!(manifest.record_count, 2);
    let snapshot = tokio::fs::read_to_string(directory.path().join(manifest.filename))
        .await
        .unwrap();
    assert!(
        snapshot
            .lines()
            .all(|line| serde_json::from_str::<Value>(line).unwrap()["synthetic"] == false)
    );
    assert!(publishing::process_one(&pool, &config).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn invalid_or_unreviewed_official_imports_leave_no_partial_data(pool: PgPool) {
    let mut candidate = batch();
    candidate.reviewed_by = None;
    assert!(import(&pool, &candidate, true, false).await.is_err());
    candidate.reviewed_by = Some("original integration fixture".into());
    let mut invalid = candidate.sets[0].clone();
    invalid.source_url = "https://press.pokemon.com/en/products/Another-original-test".into();
    invalid.name = "".into();
    candidate.sets.push(invalid);
    assert!(import(&pool, &candidate, true, false).await.is_err());
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_records").await,
        0
    );
    assert_eq!(scalar(&pool, "SELECT count(*) FROM sources").await, 0);
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM cache_invalidation_outbox").await,
        0
    );
    candidate.sets.pop();
    candidate.sets[0].source_url = "https://example.com/copied-catalogue".into();
    assert!(import(&pool, &candidate, true, false).await.is_err());
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_records").await,
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn demo_retirement_preserves_unrelated_records_and_rejects_pending_demo_drafts(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let payload = json!({"kind":"card","name":"Original independent test observation","gameId":GAME,"attributes":{},"synthetic":false,"source":{"description":"Independently authored integration test observation","method":"original_contribution","license":"CC0-1.0"}});
    let original: Draft = serde_json::from_value(payload.clone()).unwrap();
    let id = Uuid::now_v7();
    let mut connection = pool.acquire().await.unwrap();
    ingestion::write_record(&mut connection, id, &original)
        .await
        .unwrap();
    drop(connection);
    let mut pending = payload;
    pending["synthetic"] = json!(true);
    let contribution = Uuid::now_v7();
    sqlx::query("INSERT INTO contributions(id,owner_hash,idempotency_key,request_hash,operation,payload,duplicate_key) VALUES($1,'test-owner',$2,'test-request','create',$3,'test-pending-demo')")
        .bind(contribution).bind(Uuid::now_v7()).bind(pending).execute(&pool).await.unwrap();
    let result = import(&pool, &batch(), true, true).await.unwrap();
    assert_eq!(result.rejected_synthetic_contributions, 1);
    let retained: bool =
        sqlx::query_scalar("SELECT deleted_at IS NULL FROM catalogue_records WHERE id=$1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(retained);
    let status: String = sqlx::query_scalar("SELECT status FROM contributions WHERE id=$1")
        .bind(contribution)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "rejected");
    assert_eq!(
        scalar(
            &pool,
            "SELECT count(*) FROM review_decisions WHERE decision='reject'"
        )
        .await,
        1
    );
    assert_eq!(
        import(&pool, &batch(), true, true)
            .await
            .unwrap()
            .rejected_synthetic_contributions,
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_real_dependent_prevents_demo_retirement_and_rolls_back_import(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let id = Uuid::now_v7();
    let printing: Draft = serde_json::from_value(json!({"kind":"printing","name":"Independent original test printing","gameId":GAME,"setId":DEMO_SET,"cardId":DEMO_CARD,"collectorNumber":"TEST-REAL","language":"en","synthetic":false,"attributes":{},"source":{"description":"Independently authored fixture for dependency protection","method":"original_contribution","license":"CC0-1.0"}})).unwrap();
    let mut connection = pool.acquire().await.unwrap();
    ingestion::write_record(&mut connection, id, &printing)
        .await
        .unwrap();
    drop(connection);
    assert!(import(&pool, &batch(), true, true).await.is_err());
    assert_eq!(
        scalar(
            &pool,
            "SELECT count(*) FROM catalogue_records WHERE deleted_at IS NOT NULL"
        )
        .await,
        0
    );
    assert_eq!(
        scalar(&pool, "SELECT count(*) FROM catalogue_imports").await,
        0
    );
    assert_eq!(
        scalar(
            &pool,
            "SELECT count(*) FROM catalogue_records WHERE deleted_at IS NULL"
        )
        .await,
        8
    );
}
