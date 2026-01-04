
use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    pub epoch_id: u64,
    pub market_id: String,
    pub state: String,
}

pub async fn list() -> impl IntoResponse {
    Json(vec![
        Epoch { epoch_id: 1, market_id: "NBA_LAL_BOS".into(), state: "open".into() },
        Epoch { epoch_id: 2, market_id: "POL_US_ELECTION".into(), state: "closed".into() },
    ])
}
