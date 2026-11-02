pub mod auth;
pub mod cache;
pub mod config;
pub mod contributions;
pub mod error;
pub mod openapi;
pub mod reads;

use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, MatchedPath, Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use config::Config;
use error::ApiError;
use observability::Metrics;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{collections::BTreeMap, sync::Arc, time::Instant};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub metrics: Metrics,
}

pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(12)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SET statement_timeout='3s'")
                    .execute(&mut *connection)
                    .await?;
                sqlx::query("SET timezone='UTC'")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/v1/games", get(reads::catalogue))
        .route("/v1/games/{game}", get(reads::catalogue))
        .route("/v1/games/{game}/series", get(reads::catalogue))
        .route("/v1/games/{game}/series/{id}", get(reads::catalogue))
        .route("/v1/games/{game}/sets", get(reads::catalogue))
        .route("/v1/games/{game}/sets/{id}", get(reads::catalogue))
        .route("/v1/games/{game}/cards", get(reads::catalogue))
        .route("/v1/games/{game}/cards/{id}", get(reads::catalogue))
        .route("/v1/games/{game}/printings", get(reads::catalogue))
        .route("/v1/games/{game}/printings/{id}", get(reads::catalogue))
        .route("/v1/search", get(reads::catalogue))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            canonical_query,
        ));
    Router::new()
        .merge(public)
        .route("/v1/stats", get(reads::stats))
        .route("/v1/datasets", get(reads::datasets))
        .route("/datasets/{filename}", get(reads::download))
        .route("/v1/status", get(reads::status))
        .route("/v1/changelog", get(reads::changelog))
        .route("/healthz", get(reads::health))
        .route("/readyz", get(reads::ready))
        .route("/openapi.json", get(|| async { Json(openapi::document()) }))
        .route("/v1/contributions", post(contributions::submit))
        .route("/v1/contributions/{id}", get(contributions::contribution))
        .route(
            "/v1/contributions/{id}/scans/{side}",
            post(contributions::upload).layer(DefaultBodyLimit::max(8 * 1024 * 1024)),
        )
        .route("/v1/moderation/contributions", get(contributions::queue))
        .route(
            "/v1/moderation/contributions/{id}/review",
            post(contributions::review),
        )
        .route("/v1/moderation/scans/{id}", get(contributions::scan))
        .route("/v1/moderation/outbox", get(contributions::outbox))
        .route(
            "/v1/moderation/outbox/{id}/replay",
            post(contributions::replay),
        )
        .route("/metrics", get(metrics))
        .fallback(|| async { ApiError::not_found() })
        .method_not_allowed_fallback(|| async {
            ApiError {
                status: StatusCode::METHOD_NOT_ALLOWED,
                code: "method_not_allowed",
                message: "Method not supported for this resource".into(),
            }
        })
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), instrument))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::HEAD, Method::OPTIONS])
                .allow_headers([
                    axum::http::header::IF_NONE_MATCH,
                    axum::http::header::IF_MODIFIED_SINCE,
                ])
                .expose_headers([
                    axum::http::header::ETAG,
                    axum::http::header::LAST_MODIFIED,
                    axum::http::header::CACHE_CONTROL,
                ]),
        )
        .with_state(state)
}

async fn canonical_query(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let uri = request.uri().clone();
    if let Some(raw) = uri.query() {
        if raw.len() > 4096 {
            return ApiError::bad("Query exceeds 4096 bytes").into_response();
        }
        let mut parameters = BTreeMap::new();
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            if ![
                "limit", "cursor", "q", "language", "sort", "setId", "cardId", "rarity", "game",
            ]
            .contains(&key.as_ref())
                || parameters.contains_key(key.as_ref())
            {
                return ApiError::bad("Unknown or repeated query parameter").into_response();
            }
            let value = if key == "q" {
                catalogue::normalize(&value)
            } else {
                value.into_owned()
            };
            parameters.insert(key.into_owned(), value);
        }
        parameters.retain(|key, value| {
            !((key == "limit" && value == "20")
                || (key == "sort" && value == "name")
                || (key == "q" && value.is_empty()))
        });
        let encoded = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(parameters.iter())
            .finish();
        if encoded != raw {
            let location = if encoded.is_empty() {
                uri.path().to_string()
            } else {
                format!("{}?{encoded}", uri.path())
            };
            return (
                StatusCode::PERMANENT_REDIRECT,
                [("location", location), ("cache-control", "no-store".into())],
            )
                .into_response();
        }
    }
    if uri.path() == "/v1/search" || uri.query().is_some_and(|q| q.contains("q=")) {
        let peer = request
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|c| c.0.ip().to_string())
            .unwrap_or_else(|| "local".into());
        if let Err(error) = auth::rate_limit(
            &state,
            &format!("search:{}", auth::hash(peer.as_bytes())),
            120,
        )
        .await
        {
            return error.into_response();
        }
    }
    next.run(request).await
}

async fn instrument(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().as_str().to_string();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str())
        .unwrap_or("unmatched")
        .to_owned();
    let private = route.contains("contributions")
        || route.contains("moderation")
        || route == "/metrics"
        || route == "/v1/status"
        || route.ends_with("healthz")
        || route.ends_with("readyz")
        || route == "/v1/changelog";
    let mut response = next.run(request).await;
    // Normalize extractor failures into the same public error contract.
    if response.status().is_client_error()
        && !response.headers().get("content-type").is_some_and(|v| {
            v.to_str()
                .unwrap_or_default()
                .starts_with("application/json")
        })
    {
        let status = response.status();
        response = ApiError {
            status,
            code: "invalid_request",
            message: status
                .canonical_reason()
                .unwrap_or("Invalid request")
                .into(),
        }
        .into_response();
    }
    let status = response.status().as_u16().to_string();
    state
        .metrics
        .requests
        .with_label_values(&[&route, &method, &status])
        .inc();
    state
        .metrics
        .latency
        .with_label_values(&[&route])
        .observe(start.elapsed().as_secs_f64());
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response
        .headers_mut()
        .insert("x-cardlattice-origin", "axum".parse().unwrap());
    if private || !response.status().is_success() && response.status() != StatusCode::NOT_MODIFIED {
        response
            .headers_mut()
            .insert("cache-control", "no-store".parse().unwrap());
    }
    if !response.headers().contains_key("cache-control") {
        response
            .headers_mut()
            .insert("cache-control", "no-store".parse().unwrap());
    }
    response
}

async fn metrics(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, ApiError> {
    auth::authorize(&headers, &state.config.metrics_token)?;
    let mut output = state.metrics.render();
    let (pending,failed):(i64,i64)=sqlx::query_as("SELECT count(*) FILTER(WHERE status='pending'),count(*) FILTER(WHERE status='failed') FROM cache_invalidation_outbox").fetch_one(&state.pool).await?;
    output.push_str(&format!("# TYPE cardlattice_invalidation_pending gauge\ncardlattice_invalidation_pending {pending}\n# TYPE cardlattice_invalidation_failed gauge\ncardlattice_invalidation_failed {failed}\n"));
    for (kind, count) in sqlx::query_as::<_, (String, i64)>(
        "SELECT kind,count(*) FROM catalogue_records WHERE deleted_at IS NULL GROUP BY kind",
    )
    .fetch_all(&state.pool)
    .await?
    {
        output.push_str(&format!(
            "cardlattice_catalogue_records{{kind=\"{kind}\"}} {count}\n"
        ));
    }
    let (latest_revision,published_revision):(i64,i64)=sqlx::query_as("SELECT (SELECT COALESCE(MAX(revision),0) FROM catalogue_revisions WHERE after_data->>'synthetic'='false' OR before_data->>'synthetic'='false'),(SELECT COALESCE(MAX(revision),0) FROM publication_manifests WHERE withdrawn_at IS NULL)").fetch_one(&state.pool).await?;
    output.push_str(&format!(
        "cardlattice_publication_revision_lag {}\n",
        (latest_revision - published_revision).max(0)
    ));
    Ok((
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        Body::from(output),
    )
        .into_response())
}

pub async fn dataset_background(state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if let Err(error) =
            publishing::publish_snapshot(&state.pool, &state.config.publishing).await
        {
            tracing::error!(%error, "daily dataset publication failed; retrying next minute");
        }
    }
}

pub async fn background(state: AppState) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
    let mut tick = 0_u64;
    loop {
        interval.tick().await;
        if let Err(error) = publishing::process_one(&state.pool, &state.config.publishing).await {
            tracing::error!(%error,"cache invalidation worker failure");
        }
        if tick.is_multiple_of(6) {
            let start = Instant::now();
            let healthy = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
            let latency = start.elapsed().as_secs_f64() * 1000.0;
            if let Err(error) = sqlx::query(
                "INSERT INTO health_measurements(healthy,database_latency_ms) VALUES($1,$2)",
            )
            .bind(healthy)
            .bind(latency)
            .execute(&state.pool)
            .await
            {
                tracing::warn!(%error,"probe storage unavailable");
            }
        }
        if tick.is_multiple_of(720) {
            for query in [
                "DELETE FROM health_measurements WHERE measured_at<now()-interval '7 days'",
                "DELETE FROM rate_limits WHERE bucket<now()-interval '1 hour'",
            ] {
                let _ = sqlx::query(query).execute(&state.pool).await;
            }
        }
        tick = tick.wrapping_add(1);
    }
}
