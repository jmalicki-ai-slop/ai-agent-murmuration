//! Error types for database operations

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Issue not found: repository={0}, number={1}")]
    IssueNotFound(String, i64),

    #[error("Agent run not found: id={0}")]
    AgentRunNotFound(i64),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
}
