use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}
impl ApiError {
    pub fn bad(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_request",
            message: message.into(),
        }
    }
    pub fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: "Resource not found".into(),
        }
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "conflict",
            message: message.into(),
        }
    }
    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "A valid access token is required for this operation".into(),
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = (
            self.status,
            Json(ErrorBody {
                error: ErrorDetail {
                    code: self.code.into(),
                    message: self.message,
                },
            }),
        )
            .into_response();
        response
            .headers_mut()
            .insert("cache-control", "no-store".parse().unwrap());
        response
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db) = error {
            if db.is_unique_violation() {
                return Self::conflict("Duplicate record or pending contribution");
            }
            if db.is_foreign_key_violation() || db.is_check_violation() {
                return Self::bad("Invalid catalogue relationship or required field");
            }
        }
        if matches!(error, sqlx::Error::RowNotFound) {
            return Self::not_found();
        }
        tracing::error!(%error,"database operation failed");
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "database_unavailable",
            message: "Database is temporarily unavailable".into(),
        }
    }
}
impl From<catalogue::ValidationError> for ApiError {
    fn from(error: catalogue::ValidationError) -> Self {
        Self::bad(error.to_string())
    }
}
impl From<ingestion::IngestionError> for ApiError {
    fn from(error: ingestion::IngestionError) -> Self {
        match error {
            ingestion::IngestionError::Database(error) => error.into(),
            other => Self::bad(other.to_string()),
        }
    }
}
