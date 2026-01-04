
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleEvent {
    pub slot: u64,
    pub market_id: String,
    pub epoch_id: u64,
    pub payload: serde_json::Value,
}

pub fn parse_oracle_log(slot: u64, line: &str) -> Option<OracleEvent> {
    // Skeleton parser: looks for "M0_ORACLE" prefix.
    if !line.contains("M0_ORACLE") { return None; }
    Some(OracleEvent {
        slot,
        market_id: "DEMO".into(),
        epoch_id: 1,
        payload: serde_json::json!({ "raw": line }),
    })
}
