
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    pub slot: u64,
    pub market_id: String,
    pub action: String,
}

pub fn parse_registry_log(slot: u64, line: &str) -> Option<RegistryEvent> {
    if !line.contains("M0_REGISTRY") { return None; }
    Some(RegistryEvent { slot, market_id: "DEMO".into(), action: "upsert".into() })
}
