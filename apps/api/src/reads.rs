use crate::{AppState, cache::cached_json, error::ApiError};
use axum::{
    Json,
    extract::{OriginalUri, Path, Query, State, rejection::QueryRejection},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use catalogue::{ListQuery, Page, Record, encode_cursor, normalize};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Postgres, QueryBuilder};
use utoipa::ToSchema;
use uuid::Uuid;

pub async fn modified(state: &AppState) -> Result<DateTime<Utc>, ApiError> {
    Ok(sqlx::query_scalar(
        "SELECT COALESCE(MAX(created_at),'1970-01-01'::timestamptz) FROM catalogue_revisions",
    )
    .fetch_one(&state.pool)
    .await?)
}

pub async fn catalogue(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    query: Result<Query<ListQuery>, QueryRejection>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let Query(query) = query.map_err(|error| ApiError::bad(error.body_text()))?;
    query.validate()?;
    let path = uri.path();
    let segments: Vec<_> = path.trim_matches('/').split('/').collect();
    let is_search = path == "/v1/search";
    let kind = if is_search {
        None
    } else {
        match segments.get(3).copied().unwrap_or("games") {
            "games" => Some("game"),
            "series" => Some("series"),
            "sets" => Some("set"),
            "cards" => Some("card"),
            "printings" => Some("printing"),
            _ => return Err(ApiError::not_found()),
        }
    };
    let game_slug = if is_search {
        query.game.as_deref()
    } else {
        segments.get(2).copied()
    };
    let game_id: Option<Uuid> = if let Some(slug) = game_slug {
        Some(
            sqlx::query_scalar("SELECT id FROM games WHERE slug=$1 OR id::text=$1")
                .bind(slug)
                .fetch_optional(&state.pool)
                .await?
                .ok_or_else(ApiError::not_found)?,
        )
    } else {
        None
    };
    if segments.len() == 3 && !is_search {
        let record = sqlx::query_as::<_, Record>("SELECT * FROM games WHERE id=$1")
            .bind(game_id)
            .fetch_one(&state.pool)
            .await?;
        return Ok(cached_json(
            &json!({"data":record}),
            &headers,
            modified(&state).await?,
            86400,
        ));
    }
    if let Some(id) = segments.get(4) {
        let id = Uuid::parse_str(id).map_err(|_| ApiError::bad("Record ID must be a UUID"))?;
        let mut record=sqlx::query_as::<_,Record>("SELECT * FROM catalogue_records WHERE id=$1 AND kind=$2 AND game_id=$3 AND deleted_at IS NULL").bind(id).bind(kind).bind(game_id).fetch_optional(&state.pool).await?.ok_or_else(ApiError::not_found)?;
        if let Some(language) = &query.language
            && let Some(name) = sqlx::query_scalar::<_, String>(
                "SELECT name FROM localisations WHERE record_id=$1 AND language=$2",
            )
            .bind(id)
            .bind(language)
            .fetch_optional(&state.pool)
            .await?
        {
            record.name = name;
        }
        let related=sqlx::query_as::<_,Record>("SELECT * FROM printings WHERE card_id=$1 OR set_id=$1 ORDER BY normalized_name,id LIMIT 20").bind(id).fetch_all(&state.pool).await?;
        let sources: Vec<SourceView> = typed_rows(sqlx::query_scalar("SELECT jsonb_build_object('id',s.id,'description',s.description,'referenceUrl',s.reference_url,'method',s.method,'license',s.license,'fields',jsonb_agg(p.field_path ORDER BY p.field_path)) FROM sources s JOIN provenance p ON p.source_id=s.id WHERE p.record_id=$1 GROUP BY s.id ORDER BY s.recorded_at LIMIT 100").bind(id).fetch_all(&state.pool).await?)?;
        let images: Vec<ImageView> = typed_rows(sqlx::query_scalar("SELECT jsonb_build_object('id',i.id,'url',i.public_url,'side',i.side,'license',r.license,'attribution',r.attribution) FROM images i JOIN image_rights r ON r.id=i.rights_id WHERE printing_id=$1 ORDER BY i.id LIMIT 20").bind(id).fetch_all(&state.pool).await?)?;
        let variants: Vec<VariantView> = typed_rows(sqlx::query_scalar("SELECT to_jsonb(v) FROM printing_variants v WHERE printing_id=$1 ORDER BY id LIMIT 100").bind(id).fetch_all(&state.pool).await?)?;
        let localisations: Vec<LocalisationView> = typed_rows(sqlx::query_scalar("SELECT jsonb_build_object('language',language,'name',name,'textContent',text_content) FROM localisations WHERE record_id=$1 ORDER BY language LIMIT 100").bind(id).fetch_all(&state.pool).await?)?;
        let printings_url = match kind {
            Some("card") => Some(format!(
                "/v1/games/{}/printings?cardId={id}",
                game_slug.unwrap_or_default()
            )),
            Some("set") => Some(format!(
                "/v1/games/{}/printings?setId={id}",
                game_slug.unwrap_or_default()
            )),
            _ => None,
        };
        return Ok(cached_json(
            &RecordDetail {
                data: record,
                printings: related,
                printings_url,
                sources,
                images,
                variants,
                localisations,
            },
            &headers,
            modified(&state).await?,
            if query.language.is_some() { 60 } else { 86400 },
        ));
    }
    let cursor = query.decode_cursor(path)?;
    let newest = query.sort.as_deref() == Some("newest");
    let key = if newest {
        "to_char(r.created_at AT TIME ZONE 'UTC','YYYY-MM-DD HH24:MI:SS.US')"
    } else {
        "r.normalized_name"
    };
    let mut sql = QueryBuilder::<Postgres>::new(
        "SELECT r.id,r.kind,r.game_id,r.slug,COALESCE(l.name,r.name) AS name,r.series_id,r.set_id,r.card_id,r.collector_number,r.language,r.region,r.rarity,r.release_date,r.attributes,r.synthetic,r.created_at,r.updated_at FROM catalogue_records r LEFT JOIN localisations l ON l.record_id=r.id AND l.language=",
    );
    sql.push_bind(query.language.as_deref().unwrap_or("en"));
    sql.push(" WHERE r.deleted_at IS NULL");
    if let Some(kind) = kind {
        sql.push(" AND r.kind=").push_bind(kind);
    }
    if let Some(game_id) = game_id {
        sql.push(" AND r.game_id=").push_bind(game_id);
    }
    if let Some(id) = query.set_id {
        sql.push(" AND r.set_id=").push_bind(id);
    }
    if let Some(id) = query.card_id {
        sql.push(" AND r.card_id=").push_bind(id);
    }
    if let Some(rarity) = &query.rarity {
        sql.push(" AND r.rarity=").push_bind(rarity);
    }
    if let Some(language) = &query.language {
        sql.push(" AND (r.kind <> 'printing' OR r.language=")
            .push_bind(language)
            .push(")");
    }
    if let Some(search) = query
        .q
        .as_ref()
        .map(|q| normalize(q))
        .filter(|q| !q.is_empty())
    {
        let like = format!(
            "%{}%",
            search
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        sql.push(" AND (r.search_document @@ plainto_tsquery('simple',").push_bind(search.clone()).push(") OR r.normalized_name LIKE ").push_bind(like.clone())
            .push(" OR EXISTS(SELECT 1 FROM localisations sl WHERE sl.record_id=r.id AND (sl.search_document @@ plainto_tsquery('simple',").push_bind(search).push(") OR sl.normalized_name LIKE ").push_bind(like).push(")))");
    }
    if let Some(cursor) = cursor {
        sql.push(" AND (")
            .push(key)
            .push(" COLLATE \"C\",r.id) ")
            .push(if newest { "<" } else { ">" })
            .push(" (")
            .push_bind(cursor.key)
            .push(" COLLATE \"C\",")
            .push_bind(cursor.id)
            .push(")");
    }
    sql.push(" ORDER BY ")
        .push(key)
        .push(" COLLATE \"C\" ")
        .push(if newest { "DESC" } else { "ASC" })
        .push(",r.id ")
        .push(if newest { "DESC" } else { "ASC" })
        .push(" LIMIT ")
        .push_bind(i64::from(query.limit.unwrap_or(20)) + 1);
    let timer = state
        .metrics
        .database_latency
        .with_label_values(&["catalogue_list"])
        .start_timer();
    let mut records = sql
        .build_query_as::<Record>()
        .fetch_all(&state.pool)
        .await?;
    timer.observe_duration();
    let limit = query.limit.unwrap_or(20);
    let has_more = records.len() > usize::from(limit);
    records.truncate(usize::from(limit));
    let next_cursor = if has_more {
        let last = records.last().expect("positive page limit");
        // Sort by canonical name even when the displayed name is localized.
        let key = if newest {
            last.created_at.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
        } else {
            sqlx::query_scalar("SELECT normalized_name FROM catalogue_records WHERE id=$1")
                .bind(last.id)
                .fetch_one(&state.pool)
                .await?
        };
        Some(encode_cursor(key, last.id, query.scope(path)))
    } else {
        None
    };
    Ok(cached_json(
        &Page {
            data: records,
            next_cursor,
            limit,
        },
        &headers,
        modified(&state).await?,
        60,
    ))
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordDetail {
    pub data: Record,
    pub printings: Vec<Record>,
    pub printings_url: Option<String>,
    pub sources: Vec<SourceView>,
    pub images: Vec<ImageView>,
    pub variants: Vec<VariantView>,
    pub localisations: Vec<LocalisationView>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceView {
    pub id: Uuid,
    pub description: String,
    pub reference_url: Option<String>,
    pub method: String,
    pub license: String,
    pub fields: Vec<String>,
}
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImageView {
    pub id: Uuid,
    pub url: String,
    pub side: String,
    pub license: String,
    pub attribution: Option<String>,
}
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VariantView {
    pub id: Uuid,
    pub finish: String,
    pub stamp: String,
    pub attributes: Value,
}
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LocalisationView {
    pub language: String,
    pub name: String,
    pub text_content: Option<String>,
}

fn typed_rows<T: serde::de::DeserializeOwned>(values: Vec<Value>) -> Result<Vec<T>, ApiError> {
    values
        .into_iter()
        .map(|value| {
            serde_json::from_value(value).map_err(|error| {
                tracing::error!(%error,"database projection did not match response schema");
                ApiError {
                    status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    code: "response_schema_error",
                    message: "Stored record cannot be represented".into(),
                }
            })
        })
        .collect()
}
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub games: i64,
    pub series: i64,
    pub sets: i64,
    pub cards: i64,
    pub printings: i64,
    pub synthetic_records: i64,
    pub missing_rarity: i64,
    pub printings_with_images: i64,
    pub revision: i64,
    pub completeness: Option<f64>,
    pub completeness_note: String,
}

pub async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let counts:(i64,i64,i64,i64,i64,i64,i64,i64)=sqlx::query_as("SELECT count(*) FILTER(WHERE kind='game'),count(*) FILTER(WHERE kind='series'),count(*) FILTER(WHERE kind='set'),count(*) FILTER(WHERE kind='card'),count(*) FILTER(WHERE kind='printing'),count(*) FILTER(WHERE synthetic),count(*) FILTER(WHERE kind='printing' AND rarity IS NULL),count(*) FILTER(WHERE kind='printing' AND EXISTS(SELECT 1 FROM images WHERE printing_id=r.id)) FROM catalogue_records r WHERE deleted_at IS NULL").fetch_one(&state.pool).await?;
    let revision: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(revision),0) FROM catalogue_revisions")
            .fetch_one(&state.pool)
            .await?;
    Ok(cached_json(&Stats{games:counts.0,series:counts.1,sets:counts.2,cards:counts.3,printings:counts.4,synthetic_records:counts.5,missing_rarity:counts.6,printings_with_images:counts.7,revision,completeness:None,completeness_note:"No independently verified reference total exists. Counts describe this catalogue only.".into()},&headers,modified(&state).await?,60))
}

pub async fn datasets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let manifests = sqlx::query_as::<_, publishing::Manifest>(
        "SELECT * FROM publication_manifests ORDER BY revision DESC LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await?;
    let modified = manifests
        .first()
        .map(|m| m.created_at)
        .unwrap_or(DateTime::UNIX_EPOCH);
    Ok(cached_json(
        &json!({"data":manifests,"license":"CC0-1.0","includesArtwork":false,"limit":50}),
        &headers,
        modified,
        60,
    ))
}
pub async fn download(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let manifest = sqlx::query_as::<_, publishing::Manifest>(
        "SELECT * FROM publication_manifests WHERE filename=$1",
    )
    .bind(&filename)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(ApiError::not_found)?;
    let etag = format!("\"{}\"", manifest.sha256);
    let mut response = if headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .any(|s| s.trim().trim_start_matches("W/") == etag || s.trim() == "*")
        }) {
        axum::http::StatusCode::NOT_MODIFIED.into_response()
    } else {
        let file =
            tokio::fs::File::open(state.config.publishing.directory.join(&manifest.filename))
                .await
                .map_err(|_| ApiError::not_found())?;
        Response::new(axum::body::Body::from_stream(
            tokio_util::io::ReaderStream::new(file),
        ))
    };
    response.headers_mut().insert("etag", etag.parse().unwrap());
    response
        .headers_mut()
        .insert("content-type", "application/x-ndjson".parse().unwrap());
    response.headers_mut().insert(
        "content-disposition",
        format!("attachment; filename=\"{}\"", manifest.filename)
            .parse()
            .unwrap(),
    );
    response.headers_mut().insert(
        "cache-control",
        "public, max-age=31536000, immutable".parse().unwrap(),
    );
    Ok(response)
}

pub async fn health() -> Json<Value> {
    Json(json!({"status":"alive"}))
}
pub async fn ready(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    sqlx::query("SELECT 1 FROM catalogue_records LIMIT 1")
        .execute(&state.pool)
        .await?;
    Ok(Json(json!({"status":"ready"})))
}

pub async fn status(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let measurements:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('healthy',healthy,'databaseLatencyMs',database_latency_ms,'measuredAt',measured_at) FROM health_measurements ORDER BY measured_at DESC LIMIT 120").fetch_all(&state.pool).await?;
    let recent = measurements
        .first()
        .and_then(|v| v["measuredAt"].as_str())
        .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
        .is_some_and(|v| (Utc::now() - v.with_timezone(&Utc)).num_seconds() < 90);
    let (pending,failed):(i64,i64)=sqlx::query_as("SELECT count(*) FILTER(WHERE status='pending'),count(*) FILTER(WHERE status='failed') FROM cache_invalidation_outbox").fetch_one(&state.pool).await?;
    let latest: Option<DateTime<Utc>> =
        sqlx::query_scalar("SELECT MAX(created_at) FROM publication_manifests")
            .fetch_one(&state.pool)
            .await?;
    Ok(Json(
        json!({"status":if !recent {"unknown"}else if measurements.first().is_some_and(|v|v["healthy"]==true)&&failed==0 {"operational"}else{"degraded"},"measurements":measurements,"pendingInvalidations":pending,"failedInvalidations":failed,"latestPublicationAt":latest,"probeIntervalSeconds":30,"note":"Origin database probes; not global uptime. Probe storage is unavailable during a complete database outage; stale measurements are unknown."}),
    ))
}

pub async fn changelog(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let rows:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('revision',revision,'recordId',record_id,'name',after_data->>'name','kind',after_data->>'kind','createdAt',created_at,'operation',CASE WHEN before_data IS NULL THEN 'create' WHEN after_data->>'deleted_at' IS NOT NULL THEN 'delete' ELSE 'update' END) FROM catalogue_revisions ORDER BY revision DESC LIMIT 50").fetch_all(&state.pool).await?;
    Ok(Json(json!({"data":rows,"limit":50})))
}
