use crate::error::GearGabError;
use crate::models::{CanonicalMessage, Heartbeat};

pub fn message_to_json(msg: &CanonicalMessage) -> Result<String, GearGabError> {
    serde_json::to_string(msg).map_err(GearGabError::from)
}

pub fn message_from_json(json: &str) -> Result<CanonicalMessage, GearGabError> {
    serde_json::from_str(json).map_err(GearGabError::from)
}

pub fn heartbeat_to_json(hb: &Heartbeat) -> Result<String, GearGabError> {
    serde_json::to_string(hb).map_err(GearGabError::from)
}

pub fn heartbeat_from_json(json: &str) -> Result<Heartbeat, GearGabError> {
    serde_json::from_str(json).map_err(GearGabError::from)
}