use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use cardlattice_api::{AppState, config::Config, router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const CONTRIBUTOR: &str = "integration-contributor-token-not-for-production";
const MODERATOR: &str = "integration-moderator-token-not-for-production";
const METRICS: &str = "integration-metrics-token-not-for-production";
const GAME: &str = "01900000-0000-7000-8000-000000000001";
const CARD: &str = "01900000-0000-7000-8000-000000000004";
const SET: &str = "01900000-0000-7000-8000-000000000003";
const FIXTURES: &str = include_str!("../../../data/development.json");

fn state(pool: PgPool) -> (AppState, tempfile::TempDir) {
    let directory = tempfile::tempdir().unwrap();
    let config = Config {
        database_url: String::new(),
        bind: "127.0.0.1:0".into(),
        contributor_token: CONTRIBUTOR.into(),
        moderator_token: MODERATOR.into(),
        metrics_token: METRICS.into(),
        publishing: publishing::PublishingConfig {
            directory: directory.path().to_owned(),
            public_origin: "http://localhost".into(),
            cloudflare: None,
        },
    };
    (
        AppState {
            pool,
            config: Arc::new(config),
            metrics: observability::Metrics::default(),
        },
        directory,
    )
}
async fn call(
    state: &AppState,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
    key: Option<Uuid>,
) -> (StatusCode, Value, HeaderMap) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key.to_string());
    }
    let request = request
        .header("content-type", "application/json")
        .body(
            body.map(|v| Body::from(v.to_string()))
                .unwrap_or_else(Body::empty),
        )
        .unwrap();
    let response = router(state.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value =
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes)));
    (status, value, headers)
}
fn submission(name: &str) -> Value {
    json!({"operation":"create","termsAccepted":true,"draft":{"kind":"card","name":name,"gameId":GAME,"attributes":{"category":"pokemon","hp":40,"types":["grass"]},"synthetic":true,"source":{"description":"Original synthetic integration test contribution; not a real card.","method":"original_contribution","license":"CC0-1.0"}}})
}
async fn submit(state: &AppState, name: &str) -> Uuid {
    let (status, value, _) = call(
        state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(submission(name)),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    Uuid::parse_str(value["id"].as_str().unwrap()).unwrap()
}
async fn approve(state: &AppState, id: Uuid) -> Value {
    let (status,value,_)=call(state,"POST",&format!("/v1/moderation/contributions/{id}/review"),Some(MODERATOR),Some(json!({"decision":"approve","reason":"Verified original synthetic metadata and provenance"})),None).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    value
}

#[sqlx::test(migrations = "../../migrations")]
async fn empty_database_and_consistent_errors(pool: PgPool) {
    let (state, _directory) = state(pool);
    let (status, page, _) = call(&state, "GET", "/v1/games", None, None, None).await;
    assert_eq!(status, 200);
    assert_eq!(page["data"], json!([]));
    assert!(page["nextCursor"].is_null());
    let (_, stats, _) = call(&state, "GET", "/v1/stats", None, None, None).await;
    assert_eq!(stats["cards"], 0);
    assert!(stats["completeness"].is_null());
    let (_, status, _) = call(&state, "GET", "/v1/status", None, None, None).await;
    assert_eq!(status["status"], "unknown");
    for (path, expected) in [
        ("/v1/games/no-game", 404),
        ("/v1/games?limit=101", 400),
        ("/v1/games?limit=0", 400),
        ("/v1/games?unexpected=1", 400),
        ("/v1/games?limit=1&limit=2", 400),
        ("/v1/games?cursor=bad", 400),
        ("/missing", 404),
    ] {
        let (status, error, headers) = call(&state, "GET", path, None, None, None).await;
        assert_eq!(status.as_u16(), expected, "{path}: {error}");
        assert!(error["error"]["code"].is_string());
        assert_eq!(headers["cache-control"], "no-store");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn idempotent_seed_pagination_unicode_and_filters(pool: PgPool) {
    assert_eq!(ingestion::seed(&pool, FIXTURES).await.unwrap(), 7);
    assert_eq!(ingestion::seed(&pool, FIXTURES).await.unwrap(), 0);
    let (state, _directory) = state(pool.clone());
    let (_, first, _) = call(
        &state,
        "GET",
        "/v1/games/pokemon/cards?limit=1",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(first["data"].as_array().unwrap().len(), 1);
    assert!(
        first["data"][0]["name"]
            .as_str()
            .unwrap()
            .starts_with("Café")
    );
    let cursor = first["nextCursor"].as_str().unwrap();
    let (status, second, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards?cursor={cursor}&limit=1"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 200, "{second}");
    assert!(second["nextCursor"].is_null());
    assert_ne!(first["data"][0]["id"], second["data"][0]["id"]);
    let (status, _, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/sets?cursor={cursor}&limit=1"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 400);
    let (_, search, _) = call(&state, "GET", "/v1/search?q=caf%C3%A9", None, None, None).await;
    assert_eq!(search["data"].as_array().unwrap().len(), 2);
    let (status, _, headers) = call(
        &state,
        "GET",
        "/v1/search?sort=name&q=%EF%BC%A3%EF%BC%A1%EF%BC%A6%EF%BC%A5%CC%81",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 308);
    assert!(headers["location"].to_str().unwrap().contains("caf%C3%A9"));
    let (_, printings, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/printings?language=ja&setId={SET}"),
        None,
        None,
        None,
    )
    .await;
    assert!(printings["data"].as_array().unwrap().is_empty());
    sqlx::query("INSERT INTO localisations(record_id,language,name,normalized_name) VALUES($1,'ja','プリズムの種','プリズムの種')").bind(Uuid::parse_str(CARD).unwrap()).execute(&pool).await.unwrap();
    let (_, localized, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards/{CARD}?language=ja"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(localized["data"]["name"], "プリズムの種");
    let (_, japanese, _) = call(
        &state,
        "GET",
        "/v1/search?q=%E3%83%97%E3%83%AA%E3%82%BA%E3%83%A0",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(japanese["data"].as_array().unwrap().len(), 1);
    let (_, newest, _) = call(
        &state,
        "GET",
        "/v1/games/pokemon/cards?limit=1&sort=newest",
        None,
        None,
        None,
    )
    .await;
    let cursor = newest["nextCursor"].as_str().unwrap();
    let (status, page, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards?cursor={cursor}&limit=1&sort=newest"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 200, "{page}");
    assert_eq!(page["data"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn anonymous_get_etag_head_and_contract(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool);
    let spec = cardlattice_api::openapi::document();
    assert_eq!(spec["openapi"], "3.1.0");
    for (path, schema) in [
        ("/v1/games", "Page"),
        ("/v1/games/pokemon/sets", "Page"),
        ("/v1/stats", "Stats"),
    ] {
        let (status, value, headers) = call(&state, "GET", path, None, None, None).await;
        assert_eq!(status, 200);
        let mut schema_document = spec["components"]["schemas"][schema].clone();
        schema_document["components"] = spec["components"].clone();
        let validator = jsonschema::validator_for(&schema_document).unwrap();
        assert!(validator.is_valid(&value), "{schema}: {value}");
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("if-none-match", &headers["etag"])
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 304);
        assert!(response.headers().contains_key("etag"));
        assert!(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
        let response = router(state.clone())
            .oneshot(
                Request::builder()
                    .method("HEAD")
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["etag"], headers["etag"]);
        assert!(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
    }
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/v1/games")
                .header("origin", "https://example.org")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.headers()["access-control-allow-origin"], "*");
    let (status, _, _) = call(&state, "GET", "/metrics", None, None, None).await;
    assert_eq!(status, 401);
    let (status, body, _) = call(&state, "GET", "/metrics", Some(METRICS), None, None).await;
    assert_eq!(status, 200);
    assert!(
        body.as_str()
            .unwrap()
            .contains("cardlattice_http_requests_total")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn submission_review_publication_and_download(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool.clone());
    let body = submission("Integration prism — synthetic");
    let key = Uuid::now_v7();
    let (status, _, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        None,
        Some(body.clone()),
        Some(key),
    )
    .await;
    assert_eq!(status, 401);
    let (status, result, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(body.clone()),
        Some(key),
    )
    .await;
    assert_eq!(status, 200, "{result}");
    let id = Uuid::parse_str(result["id"].as_str().unwrap()).unwrap();
    let (_, replayed, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(body.clone()),
        Some(key),
    )
    .await;
    assert_eq!(replayed["replayed"], true);
    assert_eq!(replayed["id"], result["id"]);
    let (status, _, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(submission("Different — synthetic")),
        Some(key),
    )
    .await;
    assert_eq!(status, 409);
    let (status, _, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(body),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 409);
    let (status, _, _) = call(
        &state,
        "GET",
        "/v1/moderation/contributions",
        Some(CONTRIBUTOR),
        None,
        None,
    )
    .await;
    assert_eq!(status, 401);
    let (_, before, _) = call(&state, "GET", "/v1/stats", None, None, None).await;
    assert_eq!(before["cards"], 2);
    let reviewed = approve(&state, id).await;
    assert_eq!(reviewed["revision"], 8);
    let (_, record, _) = call(
        &state,
        "GET",
        &format!(
            "/v1/games/pokemon/cards/{}",
            reviewed["recordId"].as_str().unwrap()
        ),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(record["data"]["name"], "Integration prism — synthetic");
    assert!(!record["sources"].as_array().unwrap().is_empty());
    assert!(record["images"].as_array().unwrap().is_empty());
    let (_, audit, _) = call(
        &state,
        "GET",
        &format!("/v1/contributions/{id}"),
        Some(CONTRIBUTOR),
        None,
        None,
    )
    .await;
    assert_eq!(audit["decisions"][0]["decision"], "approve");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM cache_invalidation_outbox")
            .fetch_one(&pool)
            .await
            .unwrap(),
        8
    );
    while publishing::process_one(&pool, &state.config.publishing)
        .await
        .unwrap()
    {}
    let manifest = publishing::publish_snapshot(&pool, &state.config.publishing)
        .await
        .unwrap();
    assert_eq!(manifest.revision, 8);
    assert_eq!(manifest.record_count, 8);
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!("/datasets/{}", manifest.filename))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(cardlattice_api::auth::hash(&bytes), manifest.sha256);
    assert_eq!(bytes.len() as i64, manifest.byte_size);
    assert_eq!(
        String::from_utf8(bytes.to_vec()).unwrap().lines().count(),
        8
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM cache_invalidation_outbox WHERE status='completed'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        8
    );
}

struct FailingPurger;
impl publishing::CachePurger for FailingPurger {
    async fn purge(&self, _: &[String]) -> Result<(), publishing::PublishError> {
        Err(publishing::PublishError::Purge)
    }
}
#[sqlx::test(migrations = "../../migrations")]
async fn invalidation_failure_backoff_and_manual_replay(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool.clone());
    publishing::process_one_using(&pool, &state.config.publishing, &FailingPurger)
        .await
        .unwrap();
    let(id,status,attempts,delayed):(Uuid,String,i32,bool)=sqlx::query_as("SELECT id,status,attempts,next_attempt_at>now() FROM cache_invalidation_outbox ORDER BY revision LIMIT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "pending");
    assert_eq!(attempts, 1);
    assert!(delayed);
    sqlx::query(
        "UPDATE cache_invalidation_outbox SET attempts=9,next_attempt_at=now() WHERE id=$1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    publishing::process_one_using(&pool, &state.config.publishing, &FailingPurger)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT status FROM cache_invalidation_outbox WHERE id=$1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        "failed"
    );
    let (status, _, _) = call(
        &state,
        "POST",
        &format!("/v1/moderation/outbox/{id}/replay"),
        Some(CONTRIBUTOR),
        None,
        None,
    )
    .await;
    assert_eq!(status, 401);
    let (status, _, _) = call(
        &state,
        "POST",
        &format!("/v1/moderation/outbox/{id}/replay"),
        Some(MODERATOR),
        None,
        None,
    )
    .await;
    assert_eq!(status, 200);
    publishing::process_one(&pool, &state.config.publishing)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT status FROM cache_invalidation_outbox WHERE id=$1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap(),
        "completed"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn stale_correction_and_atomic_validation(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool.clone());
    let (_, record, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards/{CARD}"),
        None,
        None,
        None,
    )
    .await;
    let mut body = submission("Corrected prism — synthetic");
    body["operation"] = json!("update");
    body["targetId"] = json!(CARD);
    body["baseUpdatedAt"] = record["data"]["updatedAt"].clone();
    let (status, contribution, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(body),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 200, "{contribution}");
    sqlx::query("UPDATE catalogue_records SET updated_at=clock_timestamp() WHERE id=$1")
        .bind(Uuid::parse_str(CARD).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let (status, _, _) = call(
        &state,
        "POST",
        &format!(
            "/v1/moderation/contributions/{}/review",
            contribution["id"].as_str().unwrap()
        ),
        Some(MODERATOR),
        Some(json!({"decision":"approve","reason":"Attempt to approve stale version"})),
        None,
    )
    .await;
    assert_eq!(status, 409);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM catalogue_revisions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        7
    );
    let invalid_printing = json!({"operation":"create","termsAccepted":true,"draft":{"kind":"printing","gameId":GAME,"name":"Invalid relationship — synthetic","setId":CARD,"cardId":CARD,"collectorNumber":"BAD-001","language":"en","synthetic":true,"source":{"description":"Original synthetic validation test record","method":"original_contribution","license":"CC0-1.0"}}});
    let (status, pending, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(invalid_printing),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 200, "{pending}");
    let (status, _, _) = call(
        &state,
        "POST",
        &format!(
            "/v1/moderation/contributions/{}/review",
            pending["id"].as_str().unwrap()
        ),
        Some(MODERATOR),
        Some(json!({"decision":"approve","reason":"Attempt to publish invalid relationship"})),
        None,
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM catalogue_revisions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        7
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM review_decisions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn uploads_are_private_and_validated(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool.clone());
    let id = submit(&state, "Scan fixture — synthetic").await;
    // A tiny valid RGB PNG, generated solely for upload testing.
    let mut png_buffer = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut png_buffer, image::ImageFormat::Png)
        .unwrap();
    let png = png_buffer.into_inner();
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/contributions/{id}/scans/front"))
                .header("authorization", format!("Bearer {CONTRIBUTOR}"))
                .header("content-type", "image/png")
                .body(Body::from(png))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let value: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(status, 200, "{value}");
    let scan_id = value["id"].as_str().unwrap();
    let (status, _, headers) = call(
        &state,
        "GET",
        &format!("/v1/moderation/scans/{scan_id}"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 401);
    assert_eq!(headers["cache-control"], "no-store");
    let (status, _, headers) = call(
        &state,
        "GET",
        &format!("/v1/moderation/scans/{scan_id}"),
        Some(MODERATOR),
        None,
        None,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(headers["content-type"], "image/png");
    assert_eq!(headers["cache-control"], "no-store");
    let response = router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/contributions/{id}/scans/back"))
                .header("authorization", format!("Bearer {CONTRIBUTOR}"))
                .body(Body::from("<svg>unsafe</svg>"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 400);
    approve(&state, id).await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM images")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn rejection_permissions_and_budget(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool.clone());
    let id = submit(&state, "Rejected fixture — synthetic").await;
    let (status, _, _) = call(
        &state,
        "POST",
        &format!("/v1/moderation/contributions/{id}/review"),
        Some(MODERATOR),
        Some(json!({"decision":"reject","reason":"Source evidence is not sufficient"})),
        None,
    )
    .await;
    assert_eq!(status, 200);
    let (_, contribution, _) = call(
        &state,
        "GET",
        &format!("/v1/contributions/{id}"),
        Some(CONTRIBUTOR),
        None,
        None,
    )
    .await;
    assert_eq!(contribution["data"]["status"], "rejected");
    assert_eq!(contribution["decisions"].as_array().unwrap().len(), 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM catalogue_revisions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        7
    );
    for _ in 0..10 {
        let _ = call(
            &state,
            "POST",
            "/v1/contributions",
            Some(CONTRIBUTOR),
            Some(submission("Budget test")),
            Some(Uuid::now_v7()),
        )
        .await;
    }
    let (status, body, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(submission("Beyond budget")),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 429);
    assert_eq!(body["error"]["code"], "rate_limited");
}

#[sqlx::test(migrations = "../../migrations")]
async fn reviewed_updates_deletion_and_other_games(pool: PgPool) {
    ingestion::seed(&pool, FIXTURES).await.unwrap();
    let (state, _directory) = state(pool.clone());
    let id = submit(&state, "Editable original — synthetic").await;
    let published = approve(&state, id).await;
    let record_id = published["recordId"].as_str().unwrap();
    let (_, record, headers) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards/{record_id}"),
        None,
        None,
        None,
    )
    .await;
    let mut correction = submission("Edited original — synthetic");
    correction["operation"] = json!("update");
    correction["targetId"] = json!(record_id);
    correction["baseUpdatedAt"] = record["data"]["updatedAt"].clone();
    let (status, pending, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(correction.clone()),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 200, "{pending}");
    approve(
        &state,
        Uuid::parse_str(pending["id"].as_str().unwrap()).unwrap(),
    )
    .await;
    let (_, updated, new_headers) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards/{record_id}"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(updated["data"]["name"], "Edited original — synthetic");
    assert_ne!(headers["etag"], new_headers["etag"]);
    correction["operation"] = json!("delete");
    correction["baseUpdatedAt"] = updated["data"]["updatedAt"].clone();
    let (status, pending, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(correction),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 200, "{pending}");
    approve(
        &state,
        Uuid::parse_str(pending["id"].as_str().unwrap()).unwrap(),
    )
    .await;
    let (status, _, _) = call(
        &state,
        "GET",
        &format!("/v1/games/pokemon/cards/{record_id}"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 404);
    let mut game = submission("Original second game — synthetic");
    game["draft"]["kind"] = json!("game");
    game["draft"]["slug"] = json!("synthetic-second-game");
    game["draft"]["gameId"] = Value::Null;
    game["draft"]["attributes"] = json!({});
    let (status, pending, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(game),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 200, "{pending}");
    let published = approve(
        &state,
        Uuid::parse_str(pending["id"].as_str().unwrap()).unwrap(),
    )
    .await;
    let game_id = published["recordId"].as_str().unwrap();
    let mut other_card = submission("Other game card — synthetic");
    other_card["draft"]["gameId"] = json!(game_id);
    other_card["draft"]["attributes"] = json!({"originalMechanic":{"cost":3}});
    let (status, pending, _) = call(
        &state,
        "POST",
        "/v1/contributions",
        Some(CONTRIBUTOR),
        Some(other_card),
        Some(Uuid::now_v7()),
    )
    .await;
    assert_eq!(status, 200, "{pending}");
    approve(
        &state,
        Uuid::parse_str(pending["id"].as_str().unwrap()).unwrap(),
    )
    .await;
    let (status, records, _) = call(
        &state,
        "GET",
        &format!("/v1/games/{game_id}/cards"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(records["data"].as_array().unwrap().len(), 1);
    let (status, _, _) = call(
        &state,
        "GET",
        &format!("/v1/games/{game_id}/cards/{CARD}"),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, 404);
}
