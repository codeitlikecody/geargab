use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareEvent {
    pub source_ip: String,
    pub raw_address: String,
    pub arguments_summary: String,
    pub timestamp: i64,
}