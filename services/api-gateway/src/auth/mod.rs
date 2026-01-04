
pub mod api_keys;
pub mod jwt;

use axum::http::HeaderMap;

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let v = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    let v = v.trim();
    if let Some(rest) = v.strip_prefix("Bearer ") {
        Some(rest.trim().to_string())
    } else {
        None
    }
}
