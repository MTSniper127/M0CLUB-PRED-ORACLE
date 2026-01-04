
use axum::{extract::Path, Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub market_id: String,
    pub epoch_id: u64,
    pub outcomes: serde_json::Value,
}

pub async fn latest(Path(market_id): Path<String>) -> impl IntoResponse {
    let payload = json!({
        "A": { "p": 0.58, "ci": [0.52, 0.64] },
        "B": { "p": 0.42, "ci": [0.36, 0.48] }
    });
    Json(Prediction { market_id, epoch_id: 1, outcomes: payload })
}
