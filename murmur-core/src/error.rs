//! Error types for Murmuration

use thiserror::Error;

/// Result type alias for Murmuration operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Murmuration operations
#[derive(Error, Debug)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Agent execution error
    #[error("Agent error: {0}")]
    Agent(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}
