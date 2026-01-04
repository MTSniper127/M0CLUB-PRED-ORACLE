
use axum::response::IntoResponse;

pub async fn get() -> impl IntoResponse {
    // Basic Prometheus text format for demo.
    let body = r#"
# HELP m0_api_requests_total Total API requests
# TYPE m0_api_requests_total counter
m0_api_requests_total 1
"#;
    body
}
