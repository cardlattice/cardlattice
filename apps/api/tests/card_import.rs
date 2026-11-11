use chrono::{NaiveDate, Utc};
use ingestion::{
    official_pokemon,
    pokemon_cards::{self, CardBatch, CardCandidate, Checklist},
};
use serde_json::Value;
use sqlx::PgPool;

const SET_URL: &str = "https://press.pokemon.com/en/products/Original-checklist-test";
fn batch() -> CardBatch {
    CardBatch {
        schema_version: 1, reviewed_by: Some("Original synthetic test".into()), terms_url: pokemon_cards::TERMS_URL.into(),
        sets: vec![Checklist {
            set_source_url: SET_URL.into(), source_url: "https://d1i787aglh9bmb.cloudfront.net/assets/img/me-expansions/test/gallery/_pdfs/Original.pdf".into(),
            retrieved_at: Utc::now(), content_sha256: "b".repeat(64), scope: "main-numbered-checklist".into(), expected_count: 2,
            cards: vec![CardCandidate { collector_number: "001".into(), name: "Original Flabébé ♀".into() },CardCandidate { collector_number: "002".into(), name: "Original Flabébé ♀".into() }],
        }],
    }
}
async fn count(pool: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}
#[sqlx::test(migrations = "../../migrations")]
async fn checklist_import_preserves_identity_is_atomic_and_idempotent(pool: PgPool) {
    let input = batch();
    assert!(pokemon_cards::import(&pool, &input, true).await.is_err());
    official_pokemon::import(
        &pool,
        &official_pokemon::OfficialBatch {
            schema_version: 1,
            reviewed_by: Some("Original synthetic test".into()),
            terms_url: official_pokemon::TERMS_URL.into(),
            sets: vec![official_pokemon::SetCandidate {
                name: "Original checklist test".into(),
                release_date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                source_url: SET_URL.into(),
                retrieved_at: Utc::now(),
                content_sha256: "a".repeat(64),
            }],
        },
        true,
        false,
    )
    .await
    .unwrap();
    let prior = count(&pool, "SELECT count(*) FROM cache_invalidation_outbox").await;
    let dry = pokemon_cards::import(&pool, &input, false).await.unwrap();
    assert_eq!(dry.inserted_printings, 2);
    assert_eq!(count(&pool, "SELECT count(*) FROM printings").await, 0);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM catalogue_imports").await,
        1
    );
    let applied = pokemon_cards::import(&pool, &input, true).await.unwrap();
    assert_eq!(applied.inserted_cards, 2);
    assert_eq!(applied.inserted_printings, 2);
    assert_eq!(
        count(&pool, "SELECT count(DISTINCT card_id) FROM printings").await,
        2
    );
    assert_eq!(
        count(&pool, "SELECT count(*) FROM cache_invalidation_outbox").await,
        prior + 1
    );
    assert_eq!(
        count(&pool, "SELECT count(*) FROM catalogue_import_revisions").await,
        6
    );
    let records: Vec<(String, String)> =
        sqlx::query_as("SELECT id,card_id FROM printings ORDER BY collector_number")
            .fetch_all(&pool)
            .await
            .unwrap();
    let paths: Value = sqlx::query_scalar(
        "SELECT paths FROM cache_invalidation_outbox ORDER BY revision DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for (printing, card) in &records {
        assert!(
            paths
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p.as_str() == Some(&format!("/v1/games/pokemon/printings/{printing}")))
        );
        let scope: String = sqlx::query_scalar(
            "SELECT attributes->>'identityScope' FROM card_identities WHERE id=$1",
        )
        .bind(card)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(scope, printing.to_string());
    }
    let repeat = pokemon_cards::import(&pool, &input, true).await.unwrap();
    assert_eq!(repeat.unchanged_printings, 2);
    assert_eq!(repeat.inserted_cards, 0);
    assert_eq!(count(&pool, "SELECT count(*) FROM sources").await, 6);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM catalogue_imports").await,
        2
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM images").await, 0);
    let mut changed = input.clone();
    changed.sets[0].cards[1].name = "A conflicting correction".into();
    assert!(pokemon_cards::import(&pool, &changed, true).await.is_err());
    assert_eq!(
        count(&pool, "SELECT count(*) FROM card_identities").await,
        2
    );
    assert_eq!(
        count(&pool, "SELECT count(*) FROM catalogue_revisions").await,
        6
    );
    // Missing external identifiers must not bypass printing-level duplicate detection.
    sqlx::query(
        "DELETE FROM external_identifiers WHERE namespace='pokemon-official-numbered-checklist'",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(pokemon_cards::import(&pool, &input, true).await.is_err());
    assert_eq!(
        count(&pool, "SELECT count(*) FROM card_identities").await,
        2
    );
}
#[test]
fn checklist_validation_fails_closed() {
    let input = batch();
    assert!(input.validate().is_ok());
    let mut bad = input.clone();
    bad.reviewed_by = None;
    assert!(bad.validate().is_err());
    let mut bad = input.clone();
    bad.sets[0].source_url = "https://example.org/cards.pdf".into();
    assert!(bad.validate().is_err());
    let mut bad = input.clone();
    bad.sets[0].cards[1].collector_number = "001".into();
    assert!(bad.validate().is_err());
    let mut bad = input.clone();
    bad.sets[0].cards.pop();
    assert!(bad.validate().is_err());
    let mut bad = input.clone();
    bad.sets[0].content_sha256 = "z".repeat(64);
    assert!(bad.validate().is_err());
    let mut bad = input;
    bad.sets[0].scope = "classic-collection".into();
    assert!(bad.validate().is_err());
}
