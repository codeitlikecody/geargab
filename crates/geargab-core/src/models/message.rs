use crate::models::presence::ClientType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalMessage {
    pub msg_uuid: String,
    pub client_uuid: String,
    pub username: String,
    pub client_type: ClientType,
    pub room: String,
    pub timestamp: i64,
    pub text: String,
}
