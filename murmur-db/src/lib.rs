//! Database layer for Murmuration
//!
//! Provides SQLite persistence for:
//! - GitHub issue state and metadata
//! - Agent run history and logs
//! - Conversation transcripts
//! - Workflow state for resumption

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

pub mod schema;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Database path error: {0}")]
    Path(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;

/// Database connection pool and configuration
pub struct Database {
    pool: SqlitePool,
    path: PathBuf,
}

impl Database {
    /// Create a new database connection at the default location
    /// (~/.cache/murmur/murmur.db)
    pub async fn new() -> Result<Self> {
        let path = Self::default_path()?;
        Self::with_path(path).await
    }

    /// Create a new database connection at a specific path
    pub async fn with_path(path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DbError::Path(format!("Failed to create directory: {}", e)))?;
        }

        tracing::info!("Opening database at {:?}", path);

        let options = SqliteConnectOptions::from_str(
            path.to_str()
                .ok_or_else(|| DbError::Path("Invalid UTF-8 in path".to_string()))?,
        )?
        .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool, path })
    }

    /// Run pending migrations
    pub async fn migrate(&self) -> Result<()> {
        tracing::info!("Running database migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the database file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get the default database path
    fn default_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| DbError::Path("Could not determine cache directory".to_string()))?;

        Ok(cache_dir.join("murmur").join("murmur.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("murmur_test.db");

        // Clean up if exists
        let _ = std::fs::remove_file(&db_path);

        let db = Database::with_path(db_path.clone()).await.unwrap();
        assert_eq!(db.path(), &db_path);

        // Clean up
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_migrations() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("murmur_test_migrations.db");

        // Clean up if exists
        let _ = std::fs::remove_file(&db_path);

        let db = Database::with_path(db_path.clone()).await.unwrap();

        // Run migrations
        db.migrate().await.unwrap();

        // Verify tables exist
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(db.pool())
            .await
            .unwrap();

        assert!(!result.is_empty());

        // Clean up
        drop(db);
        let _ = std::fs::remove_file(&db_path);
    }
}
