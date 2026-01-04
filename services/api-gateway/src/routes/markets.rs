
use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub market_id: String,
    pub domain: String,
    pub status: String,
}

pub async fn list() -> impl IntoResponse {
    Json(vec![
        Market { market_id: "NBA_LAL_BOS".into(), domain: "sports".into(), status: "active".into() },
        Market { market_id: "POL_US_ELECTION".into(), domain: "politics".into(), status: "active".into() },
    ])
}
