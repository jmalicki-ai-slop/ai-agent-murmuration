//! Error types for database operations

use thiserror::Error;

/// Database error types
#[derive(Error, Debug)]
pub enum Error {
    /// SQLx database error
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Result type alias for database operations
pub type Result<T> = std::result::Result<T, Error>;
