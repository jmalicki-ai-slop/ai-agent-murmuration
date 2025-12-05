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
                pid INTEGER,
                start_time TEXT NOT NULL,
                end_time TEXT,
                exit_code INTEGER,
                duration_seconds REAL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Add PID column if it doesn't exist (for existing databases)
        self.conn
            .execute("ALTER TABLE agent_runs ADD COLUMN pid INTEGER", [])
            .ok(); // Ignore error if column already exists

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

        // Create worktrees table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS worktrees (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                branch_name TEXT NOT NULL,
                issue_number INTEGER,
                agent_run_id INTEGER,
                main_repo_path TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE SET NULL
            )",
            [],
        )?;

        // Create indexes for worktrees
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_worktrees_branch
             ON worktrees(branch_name)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_worktrees_issue
             ON worktrees(issue_number)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_worktrees_status
             ON worktrees(status)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_worktrees_agent_run
             ON worktrees(agent_run_id)",
            [],
        )?;

        // Migrate existing worktrees table to add main_repo_path column if it doesn't exist
        let has_main_repo_path = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('worktrees') WHERE name='main_repo_path'",
            [],
            |row| row.get::<_, i32>(0),
        )?;

        if has_main_repo_path == 0 {
            self.conn
                .execute("ALTER TABLE worktrees ADD COLUMN main_repo_path TEXT", [])?;
        }

        // Create issue_states table for tracking GitHub issue status
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS issue_states (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                issue_number INTEGER NOT NULL,
                repository TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                labels_json TEXT,
                dependencies_json TEXT,
                last_agent_run_id INTEGER,
                last_worked_at TEXT,
                last_error TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (last_agent_run_id) REFERENCES agent_runs(id) ON DELETE SET NULL,
                UNIQUE(issue_number, repository)
            )",
            [],
        )?;

        // Create indexes for issue_states
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issue_states_repo
             ON issue_states(repository)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issue_states_status
             ON issue_states(status)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issue_states_issue_repo
             ON issue_states(issue_number, repository)",
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

        // Verify issue_states table exists
        let count: i32 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='issue_states'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }
}
