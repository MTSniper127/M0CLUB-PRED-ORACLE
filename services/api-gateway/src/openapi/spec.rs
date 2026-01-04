
use serde_json::json;

pub fn openapi_spec() -> serde_json::Value {
    // Minimal OpenAPI payload (expand as needed).
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "M0Club API Gateway",
            "version": "0.1.0"
        },
        "paths": {
            "/health": { "get": { "responses": { "200": { "description": "ok" } } } }
        }
    })
}
