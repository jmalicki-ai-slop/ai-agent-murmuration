//! Database connection and configuration

use crate::error::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::path::PathBuf;
use std::str::FromStr;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    pub path: PathBuf,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let db_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("murmur")
            .join("murmur.db");

        Self {
            path: db_path,
            max_connections: 5,
        }
    }
}

impl DatabaseConfig {
    /// Create a new database config with the given path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_connections: 5,
        }
    }

    /// Set the maximum number of connections
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }
}

/// Database connection pool
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the database with the given configuration
    pub async fn connect(config: DatabaseConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create connection options
        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", config.path.display()))?
                .create_if_missing(true)
                .disable_statement_logging();

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        // Read migration file
        let migration_sql = include_str!("../migrations/001_initial_schema.sql");

        // Execute migration
        sqlx::query(migration_sql).execute(&self.pool).await?;

        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig::new(&db_path);
        let db = Database::connect(config).await.unwrap();
        db.migrate().await.unwrap();

        assert!(db_path.exists());
        db.close().await;
    }
}
