//! Database layer for Murmuration
//!
//! Provides persistence for agent runs, conversation logs, and workflow state.

pub mod error;
pub mod repos;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub use error::{Error, Result};
pub use repos::{
    agents::{AgentRun, AgentRunStatus, AgentRunsRepo},
    conversations::{ConversationLog, ConversationLogsRepo},
};

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection from a file path
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref();

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Io(format!("Failed to create database directory: {}", e)))?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| Error::Migration(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get the default database path (~/.cache/murmur/murmur.db)
    pub fn default_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| Error::Io("Could not determine cache directory".to_string()))?;
        Ok(cache_dir.join("murmur").join("murmur.db"))
    }

    /// Create a database connection at the default path
    pub async fn default() -> Result<Self> {
        Self::new(Self::default_path()?).await
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the agent runs repository
    pub fn agent_runs(&self) -> AgentRunsRepo {
        AgentRunsRepo::new(self.pool.clone())
    }

    /// Get the conversation logs repository
    pub fn conversation_logs(&self) -> ConversationLogsRepo {
        ConversationLogsRepo::new(self.pool.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.unwrap();
        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn test_database_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.unwrap();

        // Verify tables exist
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_runs'")
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert_eq!(result.0, 1);

        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='conversation_logs'")
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert_eq!(result.0, 1);
    }
}
