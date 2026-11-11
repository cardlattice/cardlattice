use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn cached_json<T: Serialize>(
    value: &T,
    request: &HeaderMap,
    modified: DateTime<Utc>,
    edge_ttl: u32,
) -> Response {
    let bytes = serde_json::to_vec(value).expect("JSON response serialization");
    let etag = format!("\"{:x}\"", Sha256::digest(&bytes));
    let modified_time: std::time::SystemTime = modified.into();
    let not_modified = if let Some(value) = request.get("if-none-match") {
        value.to_str().unwrap_or_default().split(',').any(|tag| {
            let tag = tag.trim().trim_start_matches("W/");
            tag == etag || tag == "*"
        })
    } else {
        request
            .get("if-modified-since")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| httpdate::parse_http_date(v).ok())
            .is_some_and(|time| {
                // Conservative sub-second comparison avoids missing two writes in one second.
                modified_time <= time
            })
    };
    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        Response::new(Body::from(bytes))
    };
    let headers = response.headers_mut();
    headers.insert(
        "content-type",
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert("etag", etag.parse().unwrap());
    headers.insert(
        "last-modified",
        httpdate::fmt_http_date(modified_time).parse().unwrap(),
    );
    headers.insert(
        "cache-control",
        format!("public, max-age=15, s-maxage={edge_ttl}, must-revalidate")
            .parse()
            .unwrap(),
    );
    headers.insert(
        "cdn-cache-control",
        format!("public, max-age={edge_ttl}").parse().unwrap(),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weak_validator_and_precedence() {
        let now = Utc::now();
        let mut headers = HeaderMap::new();
        let first = cached_json(&serde_json::json!({"id":1}), &headers, now, 60);
        headers.insert(
            "if-none-match",
            format!("W/{}", first.headers()["etag"].to_str().unwrap())
                .parse()
                .unwrap(),
        );
        assert_eq!(
            cached_json(&serde_json::json!({"id":1}), &headers, now, 60).status(),
            StatusCode::NOT_MODIFIED
        );
        headers.insert("if-none-match", "\"different\"".parse().unwrap());
        headers.insert(
            "if-modified-since",
            "Wed, 21 Oct 2099 07:28:00 GMT".parse().unwrap(),
        );
        assert_eq!(
            cached_json(&serde_json::json!({"id":1}), &headers, now, 60).status(),
            StatusCode::OK
        );
    }
}
