
use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn get() -> impl IntoResponse {
    Json(json!({"ok": true}))
}
