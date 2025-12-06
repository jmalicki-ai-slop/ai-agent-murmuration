//! Repository for worktree records

use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{models::WorktreeRecord, Database, Error, Result};

/// Repository for managing worktree records
pub struct WorktreeRepository<'a> {
    db: &'a Database,
}

impl<'a> WorktreeRepository<'a> {
    /// Create a new repository
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Insert a new worktree record
    pub fn insert(&self, record: &WorktreeRecord) -> Result<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO worktrees (path, branch_name, issue_number, agent_run_id, main_repo_path, base_commit, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.path,
                record.branch_name,
                record.issue_number,
                record.agent_run_id,
                record.main_repo_path,
                record.base_commit,
                record.status,
                record.created_at.to_rfc3339(),
                record.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Update an existing worktree record
    pub fn update(&self, record: &WorktreeRecord) -> Result<()> {
        let id = record
            .id
            .ok_or_else(|| Error::InvalidData("Worktree record has no ID".to_string()))?;

        let conn = self.db.connection();
        conn.execute(
            "UPDATE worktrees SET path = ?1, branch_name = ?2, issue_number = ?3, agent_run_id = ?4, main_repo_path = ?5, base_commit = ?6, status = ?7, updated_at = ?8 WHERE id = ?9",
            params![
                record.path,
                record.branch_name,
                record.issue_number,
                record.agent_run_id,
                record.main_repo_path,
                record.base_commit,
                record.status,
                record.updated_at.to_rfc3339(),
                id,
            ],
        )?;

        Ok(())
    }

    /// Find a worktree by path
    pub fn find_by_path(&self, path: &str) -> Result<Option<WorktreeRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, path, branch_name, issue_number, agent_run_id, main_repo_path, base_commit, status, created_at, updated_at
             FROM worktrees WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    /// Find a worktree by branch name
    pub fn find_by_branch(&self, branch_name: &str) -> Result<Option<WorktreeRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, path, branch_name, issue_number, agent_run_id, main_repo_path, base_commit, status, created_at, updated_at
             FROM worktrees WHERE branch_name = ?1",
        )?;

        let mut rows = stmt.query(params![branch_name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    /// Find all worktrees by status
    pub fn find_by_status(&self, status: &str) -> Result<Vec<WorktreeRecord>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, path, branch_name, issue_number, agent_run_id, main_repo_path, base_commit, status, created_at, updated_at
             FROM worktrees WHERE status = ?1 ORDER BY created_at DESC",
        )?;

        let mut rows = stmt.query(params![status])?;

        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(self.row_to_record(row)?);
        }

        Ok(records)
    }

    /// Find all active worktrees
    pub fn find_active(&self) -> Result<Vec<WorktreeRecord>> {
        self.find_by_status("active")
    }

    /// Find all stale worktrees
    pub fn find_stale(&self) -> Result<Vec<WorktreeRecord>> {
        self.find_by_status("stale")
    }

    /// Delete a worktree record by path
    pub fn delete_by_path(&self, path: &str) -> Result<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM worktrees WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// Mark all active worktrees as stale (useful on startup)
    pub fn mark_all_active_as_stale(&self) -> Result<usize> {
        let conn = self.db.connection();
        let count = conn.execute(
            "UPDATE worktrees SET status = 'stale', updated_at = ?1 WHERE status = 'active'",
            params![Utc::now().to_rfc3339()],
        )?;
        Ok(count)
    }

    /// Convert a database row to a WorktreeRecord
    fn row_to_record(&self, row: &rusqlite::Row) -> Result<WorktreeRecord> {
        let created_at_str: String = row.get(8)?;
        let updated_at_str: String = row.get(9)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| Error::InvalidData(format!("Invalid created_at timestamp: {}", e)))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| Error::InvalidData(format!("Invalid updated_at timestamp: {}", e)))?
            .with_timezone(&Utc);

        Ok(WorktreeRecord {
            id: Some(row.get(0)?),
            path: row.get(1)?,
            branch_name: row.get(2)?,
            issue_number: row.get(3)?,
            agent_run_id: row.get(4)?,
            main_repo_path: row.get(5)?,
            base_commit: row.get(6)?,
            status: row.get(7)?,
            created_at,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_find_worktree() {
        let db = Database::in_memory().unwrap();
        let repo = WorktreeRepository::new(&db);

        let record =
            WorktreeRecord::new("/tmp/test-worktree", "murmur/issue-123").with_issue_number(123);

        let id = repo.insert(&record).unwrap();
        assert!(id > 0);

        let found = repo.find_by_path("/tmp/test-worktree").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.branch_name, "murmur/issue-123");
        assert_eq!(found.issue_number, Some(123));
    }

    #[test]
    fn test_find_by_branch() {
        let db = Database::in_memory().unwrap();
        let repo = WorktreeRepository::new(&db);

        let record = WorktreeRecord::new("/tmp/test-worktree", "murmur/issue-456");
        repo.insert(&record).unwrap();

        let found = repo.find_by_branch("murmur/issue-456").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().path, "/tmp/test-worktree");
    }

    #[test]
    fn test_find_active() {
        let db = Database::in_memory().unwrap();
        let repo = WorktreeRepository::new(&db);

        let record1 = WorktreeRecord::new("/tmp/wt1", "murmur/issue-1");
        let record2 = WorktreeRecord::new("/tmp/wt2", "murmur/issue-2");
        repo.insert(&record1).unwrap();
        repo.insert(&record2).unwrap();

        let active = repo.find_active().unwrap();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_mark_all_active_as_stale() {
        let db = Database::in_memory().unwrap();
        let repo = WorktreeRepository::new(&db);

        let record1 = WorktreeRecord::new("/tmp/wt1", "murmur/issue-1");
        let record2 = WorktreeRecord::new("/tmp/wt2", "murmur/issue-2");
        repo.insert(&record1).unwrap();
        repo.insert(&record2).unwrap();

        let count = repo.mark_all_active_as_stale().unwrap();
        assert_eq!(count, 2);

        let stale = repo.find_stale().unwrap();
        assert_eq!(stale.len(), 2);
    }
}
