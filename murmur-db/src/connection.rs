//! Database connection and initialization

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{Error, Result};

/// Database handle
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the default location
    ///
    /// Location: `~/.cache/murmur/murmur.db`
    pub fn open() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open_at(&path)
    }

    /// Open or create database at a specific path
    pub fn open_at(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::InvalidData(format!("Failed to create database directory: {}", e))
            })?;
        }

        let conn = Connection::open(path)?;
        let mut db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Create an in-memory database for testing
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let mut db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Get the default database path
    ///
    /// Returns `~/.cache/murmur/murmur.db`
    pub fn default_path() -> Result<PathBuf> {
        dirs::cache_dir()
            .map(|p| p.join("murmur").join("murmur.db"))
            .ok_or_else(|| Error::InvalidData("Failed to determine cache directory".to_string()))
    }

    /// Initialize database schema
    fn initialize(&mut self) -> Result<()> {
        // Enable foreign keys
        self.conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Create agent_runs table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_type TEXT NOT NULL,
                issue_number INTEGER,
                prompt TEXT NOT NULL,
                workdir TEXT NOT NULL,
                config_json TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                exit_code INTEGER,
                duration_seconds REAL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create indexes for common queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_runs_issue
             ON agent_runs(issue_number)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_runs_start_time
             ON agent_runs(start_time)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agent_runs_agent_type
             ON agent_runs(agent_type)",
            [],
        )?;

        // Create conversation_logs table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS conversation_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_run_id INTEGER NOT NULL,
                sequence INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                message_type TEXT NOT NULL,
                message_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE,
                UNIQUE(agent_run_id, sequence)
            )",
            [],
        )?;

        // Create indexes for conversation_logs
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversation_logs_agent_run
             ON conversation_logs(agent_run_id, sequence)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversation_logs_timestamp
             ON conversation_logs(timestamp)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversation_logs_message_type
             ON conversation_logs(message_type)",
            [],
        )?;

        Ok(())
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database() {
        let db = Database::in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn test_schema_initialization() {
        let db = Database::in_memory().unwrap();

        // Verify agent_runs table exists
        let count: i32 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_runs'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);

        // Verify conversation_logs table exists
        let count: i32 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='conversation_logs'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }
}
