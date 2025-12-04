//! Error types for database operations

use thiserror::Error;

/// Database-related errors
#[derive(Debug, Error)]
pub enum Error {
    /// SQLite errors
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Not found errors
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Invalid data errors
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Result type alias for database operations
pub type Result<T> = std::result::Result<T, Error>;
