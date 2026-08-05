// crates/geargab-core/src/error.rs

use thiserror::Error;

/// Master error type for geargab-core operations.
#[derive(Error, Debug)]
pub enum GearGabError {
    /// Failure during OSC wire encoding or decoding.
    #[error("OSC codec error: {0}")]
    OscCodec(#[from] rosc::OscError),

    /// Standard I/O operations failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// OSC address does not conform to expected protocol paths.
    #[error("Invalid OSC address path: {0}")]
    InvalidOscAddress(String),

    /// Missing mandatory OSC argument in packet payload.
    #[error("Missing required OSC argument '{expected}' at index {index}")]
    MissingArgument {
        expected: &'static str,
        index: usize,
    },

    /// OSC argument was found but had an unexpected type tag.
    #[error("OSC argument type mismatch at index {index}: expected {expected}, found {found}")]
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
        index: usize,
    },

    /// Provided string failed UUID v4 validation.
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(String),

    /// Unrecognized hardware OSC packet that could not be mapped to domain events.
    #[error("Unmatched hardware OSC address: {0}")]
    UnmatchedHardwareOsc(String),
}

/// Helper Result type alias for geargab-core.
pub type Result<T> = std::result::Result<T, GearGabError>;

impl GearGabError {
    /// Validates whether a given string is a valid UUID v4 format.
    ///
    /// UUIDs must match standard 8-4-4-4-12 hexadecimal layout.
    pub fn validate_uuid(uuid_str: &str) -> Result<()> {
        todo!("Implement UUID v4 format validation logic")
    }
}