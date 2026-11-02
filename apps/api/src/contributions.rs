use crate::{
    AppState,
    auth::{authorize, hash, rate_limit},
    error::ApiError,
};
use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use catalogue::{Draft, Record};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Create,
    Update,
    Delete,
}
impl Operation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
        }
    }
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Submission {
    pub operation: Operation,
    pub target_id: Option<Uuid>,
    pub base_updated_at: Option<DateTime<Utc>>,
    pub draft: Draft,
    pub terms_accepted: bool,
    pub website: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub id: Uuid,
    pub operation: String,
    pub target_id: Option<Uuid>,
    pub base_updated_at: Option<DateTime<Utc>>,
    pub payload: Value,
    pub status: String,
    pub record_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Review {
    pub decision: String,
    pub reason: String,
}

pub async fn submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<Submission>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let owner = authorize(&headers, &state.config.contributor_token)?;
    rate_limit(&state, &format!("submit:{owner}"), 10).await?;
    let Json(submission) = body.map_err(|error| ApiError::bad(error.body_text()))?;
    if !submission.terms_accepted || submission.website.as_ref().is_some_and(|s| !s.is_empty()) {
        return Err(ApiError::bad(
            "Accept the contributor terms and leave the website field empty",
        ));
    }
    let idempotency = headers
        .get("idempotency-key")
        .and_then(|h| h.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| ApiError::bad("Idempotency-Key must be a UUID"))?;
    let request_hash =
        hash(&serde_json::to_vec(&submission).map_err(|_| ApiError::bad("Invalid submission"))?);
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(71001)")
        .execute(&mut *tx)
        .await?;
    if let Some((id, old_hash)) = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id,request_hash FROM contributions WHERE owner_hash=$1 AND idempotency_key=$2",
    )
    .bind(&owner)
    .bind(idempotency)
    .fetch_optional(&mut *tx)
    .await?
    {
        if old_hash != request_hash {
            return Err(ApiError::conflict(
                "Idempotency key was already used for a different request",
            ));
        }
        return Ok(Json(json!({"id":id,"replayed":true})));
    }
    let slug: Option<String> = sqlx::query_scalar("SELECT slug FROM games WHERE id=$1")
        .bind(submission.draft.game_id)
        .fetch_optional(&mut *tx)
        .await?;
    submission.draft.validate(slug.as_deref())?;
    if submission.draft.kind != catalogue::Kind::Game && slug.is_none() {
        return Err(ApiError::bad("Game does not exist"));
    }
    if matches!(submission.operation, Operation::Create) {
        if submission.target_id.is_some() || submission.base_updated_at.is_some() {
            return Err(ApiError::bad(
                "New records cannot specify targetId or baseUpdatedAt",
            ));
        }
    } else {
        let target = submission
            .target_id
            .ok_or_else(|| ApiError::bad("Corrections require targetId"))?;
        let current = sqlx::query_as::<_, Record>(
            "SELECT * FROM catalogue_records WHERE id=$1 AND deleted_at IS NULL",
        )
        .bind(target)
        .fetch_one(&mut *tx)
        .await?;
        if Some(current.updated_at) != submission.base_updated_at {
            return Err(ApiError::conflict(
                "The record changed; load the current revision before submitting",
            ));
        }
        if current.kind != submission.draft.kind.as_str()
            || current.game_id != submission.draft.game_id
            || current.slug != submission.draft.slug
        {
            return Err(ApiError::bad("Kind, game and slug are immutable"));
        }
    }
    let duplicate:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM catalogue_records WHERE duplicate_key=$1 AND deleted_at IS NULL AND ($2::uuid IS NULL OR id<>$2))").bind(submission.draft.duplicate_key()).bind(submission.target_id).fetch_one(&mut *tx).await?;
    if duplicate {
        return Err(ApiError::conflict(
            "A matching record already exists; submit a correction instead",
        ));
    }
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO contributions(id,owner_hash,idempotency_key,request_hash,operation,target_id,base_updated_at,payload,duplicate_key) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
        .bind(id).bind(owner).bind(idempotency).bind(request_hash).bind(submission.operation.as_str()).bind(submission.target_id).bind(submission.base_updated_at).bind(json!(submission.draft)).bind(submission.draft.duplicate_key()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id,"replayed":false})))
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct QueueQuery {
    pub cursor: Option<Uuid>,
}
pub async fn queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<QueueQuery>,
) -> Result<Json<Value>, ApiError> {
    authorize(&headers, &state.config.moderator_token)?;
    let mut rows=sqlx::query_as::<_,Contribution>("SELECT * FROM contributions WHERE status='pending' AND ($1::uuid IS NULL OR id>$1) ORDER BY id LIMIT 51").bind(query.cursor).fetch_all(&state.pool).await?;
    let has_more = rows.len() > 50;
    rows.truncate(50);
    let cursor = if has_more {
        rows.last().map(|r| r.id)
    } else {
        None
    };
    Ok(Json(json!({"data":rows,"nextCursor":cursor})))
}

pub async fn contribution(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let moderator = authorize(&headers, &state.config.moderator_token).is_ok();
    let owner = if moderator {
        None
    } else {
        Some(authorize(&headers, &state.config.contributor_token)?)
    };
    let contribution = sqlx::query_as::<_, Contribution>(
        "SELECT * FROM contributions WHERE id=$1 AND ($2::text IS NULL OR owner_hash=$2)",
    )
    .bind(id)
    .bind(owner)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(ApiError::not_found)?;
    let scans:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'side',side,'mediaType',media_type,'sha256',sha256,'rightsStatus',rights_status,'candidateMetadata',candidate_metadata) FROM scan_uploads WHERE contribution_id=$1 ORDER BY side").bind(id).fetch_all(&state.pool).await?;
    let decisions:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('decision',decision,'reason',reason,'createdAt',created_at) FROM review_decisions WHERE contribution_id=$1 ORDER BY created_at").bind(id).fetch_all(&state.pool).await?;
    Ok(Json(
        json!({"data":contribution,"scans":scans,"decisions":decisions}),
    ))
}

pub async fn review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    body: Result<Json<Review>, JsonRejection>,
) -> Result<Json<Value>, ApiError> {
    let reviewer = authorize(&headers, &state.config.moderator_token)?;
    rate_limit(&state, &format!("review:{reviewer}"), 60).await?;
    let Json(review) = body.map_err(|e| ApiError::bad(e.body_text()))?;
    if !["approve", "reject"].contains(&review.decision.as_str())
        || review.reason.trim().len() < 5
        || review.reason.len() > 2000
    {
        return Err(ApiError::bad(
            "Choose approve or reject and provide a 5–2000 byte reason",
        ));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(71001)")
        .execute(&mut *tx)
        .await?;
    let contribution =
        sqlx::query_as::<_, Contribution>("SELECT * FROM contributions WHERE id=$1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if contribution.status != "pending" {
        return Err(ApiError::conflict(
            "This contribution has already been reviewed",
        ));
    }
    let mut record_id = None;
    let mut revision = None;
    if review.decision == "approve" {
        let draft: Draft = serde_json::from_value(contribution.payload)
            .map_err(|_| ApiError::bad("Stored draft is invalid"))?;
        let target = contribution.target_id.unwrap_or_else(Uuid::now_v7);
        let before:Option<Value>=sqlx::query_scalar("SELECT to_jsonb(r)-'search_document' FROM catalogue_records r WHERE id=$1 AND deleted_at IS NULL").bind(target).fetch_optional(&mut *tx).await?;
        if contribution.operation != "create" {
            let current = sqlx::query_as::<_, Record>(
                "SELECT * FROM catalogue_records WHERE id=$1 AND deleted_at IS NULL",
            )
            .bind(target)
            .fetch_one(&mut *tx)
            .await?;
            if Some(current.updated_at) != contribution.base_updated_at {
                return Err(ApiError::conflict(
                    "Draft is stale; reject it and submit a correction against the latest revision",
                ));
            }
            if current.kind != draft.kind.as_str()
                || current.game_id != draft.game_id
                || current.slug != draft.slug
            {
                return Err(ApiError::bad("Kind, game and slug are immutable"));
            }
        }
        if contribution.operation == "delete" {
            sqlx::query("UPDATE catalogue_records SET deleted_at=clock_timestamp(),updated_at=clock_timestamp() WHERE id=$1").bind(target).execute(&mut *tx).await?;
        } else {
            ingestion::write_record(&mut tx, target, &draft).await?;
        }
        revision = Some(ingestion::revision_and_outbox(&mut tx, target, Some(id), before).await?);
        record_id = Some(target);
    }
    sqlx::query("INSERT INTO review_decisions(id,contribution_id,reviewer_hash,decision,reason) VALUES($1,$2,$3,$4,$5)").bind(Uuid::now_v7()).bind(id).bind(reviewer).bind(&review.decision).bind(review.reason.trim()).execute(&mut *tx).await?;
    sqlx::query(
        "UPDATE contributions SET status=$2,record_id=$3,reviewed_at=clock_timestamp() WHERE id=$1",
    )
    .bind(id)
    .bind(if review.decision == "approve" {
        "approved"
    } else {
        "rejected"
    })
    .bind(record_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"id":id,"decision":review.decision,"recordId":record_id,"revision":revision}),
    ))
}

pub async fn upload(
    State(state): State<AppState>,
    Path((id, side)): Path<(Uuid, String)>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Result<Json<Value>, ApiError> {
    let owner = authorize(&headers, &state.config.contributor_token)?;
    rate_limit(&state, &format!("upload:{owner}"), 6).await?;
    if !["front", "back"].contains(&side.as_str()) {
        return Err(ApiError::bad("Scan side must be front or back"));
    }
    let mut tx = state.pool.begin().await?;
    let valid:Option<Uuid>=sqlx::query_scalar("SELECT id FROM contributions WHERE id=$1 AND owner_hash=$2 AND status='pending' FOR UPDATE").bind(id).bind(owner).fetch_optional(&mut *tx).await?;
    if valid.is_none() {
        return Err(ApiError::not_found());
    }
    let checked = bytes.clone();
    let (mime, digest) = tokio::task::spawn_blocking(move || ingestion::validate_scan(&checked))
        .await
        .map_err(|_| ApiError::bad("Scan decoding failed"))??;
    let scan_id = Uuid::now_v7();
    use ingestion::ScanExtractor;
    let candidate = ingestion::ManualExtractor.extract(&bytes)?;
    sqlx::query("INSERT INTO scan_uploads(id,contribution_id,side,media_type,sha256,original,candidate_metadata) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(scan_id).bind(id).bind(side).bind(mime).bind(digest).bind(bytes.to_vec()).bind(json!(candidate)).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(
        json!({"id":scan_id,"rightsStatus":"private_review_only","candidateMetadata":candidate}),
    ))
}

pub async fn scan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    authorize(&headers, &state.config.moderator_token)?;
    let (mime, bytes): (String, Vec<u8>) =
        sqlx::query_as("SELECT media_type,original FROM scan_uploads WHERE id=$1")
            .bind(id)
            .fetch_one(&state.pool)
            .await?;
    Ok((
        [
            ("content-type", mime),
            ("content-disposition", "attachment".into()),
            ("cache-control", "no-store".into()),
        ],
        bytes,
    )
        .into_response())
}

pub async fn outbox(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorize(&headers, &state.config.moderator_token)?;
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(o) FROM cache_invalidation_outbox o ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(json!({"data":rows,"limit":100})))
}
pub async fn replay(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authorize(&headers, &state.config.moderator_token)?;
    if !publishing::replay(&state.pool, id).await? {
        return Err(ApiError::conflict(
            "Only failed invalidations can be replayed",
        ));
    }
    Ok(Json(json!({"id":id,"status":"pending"})))
}
