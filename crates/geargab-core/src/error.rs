use thiserror::Error;

#[derive(Error, Debug)]
pub enum GearGabError {
    #[error("OSC encoding error: {0}")]
    OscEncodeError(String),

    #[error("OSC decoding error: {0}")]
    OscDecodeError(String),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Invalid OSC address pattern: {0}")]
    InvalidAddressPattern(String),
}