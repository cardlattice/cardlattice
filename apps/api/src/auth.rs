use crate::{AppState, error::ApiError};
use axum::http::{HeaderMap, StatusCode};
use sha2::{Digest, Sha256};

pub fn hash(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
pub fn authorize(headers: &HeaderMap, expected: &str) -> Result<String, ApiError> {
    let provided = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(ApiError::unauthorized)?;
    // Compare fixed-size hashes without short-circuiting byte comparisons.
    let actual = Sha256::digest(provided.as_bytes());
    let wanted = Sha256::digest(expected.as_bytes());
    let difference = actual
        .iter()
        .zip(wanted.iter())
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b));
    if difference != 0 {
        return Err(ApiError::unauthorized());
    }
    Ok(hash(provided.as_bytes()))
}
pub async fn rate_limit(state: &AppState, subject: &str, maximum: i32) -> Result<(), ApiError> {
    let hits: i32 = sqlx::query_scalar("INSERT INTO rate_limits(subject,bucket,hits) VALUES($1,date_trunc('minute',now()),1) ON CONFLICT(subject,bucket) DO UPDATE SET hits=rate_limits.hits+1 RETURNING hits").bind(subject).fetch_one(&state.pool).await?;
    if hits > maximum {
        return Err(ApiError {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            message: "Request budget exceeded; retry in one minute".into(),
        });
    }
    Ok(())
}
