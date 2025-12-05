//! Repository for issue state operations

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};

use crate::models::IssueState;
use crate::{Database, Error, Result};

/// Repository for managing issue state records
pub struct IssueStateRepository<'db> {
    db: &'db Database,
}

impl<'db> IssueStateRepository<'db> {
    /// Create a new repository instance
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Insert a new issue state record
    pub fn insert(&self, state: &IssueState) -> Result<i64> {
        let conn = self.db.connection();

        conn.execute(
            "INSERT INTO issue_states (
                issue_number, repository, title, status, labels_json,
                dependencies_json, last_agent_run_id, last_worked_at,
                last_error, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                state.issue_number,
                state.repository,
                state.title,
                state.status,
                state.labels_json,
                state.dependencies_json,
                state.last_agent_run_id,
                state.last_worked_at.map(|dt| dt.to_rfc3339()),
                state.last_error,
                state.created_at.to_rfc3339(),
                state.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Update an existing issue state record
    pub fn update(&self, state: &IssueState) -> Result<()> {
        let id = state
            .id
            .ok_or_else(|| Error::InvalidData("Cannot update record without ID".to_string()))?;

        let conn = self.db.connection();
        let affected = conn.execute(
            "UPDATE issue_states SET
                issue_number = ?1,
                repository = ?2,
                title = ?3,
                status = ?4,
                labels_json = ?5,
                dependencies_json = ?6,
                last_agent_run_id = ?7,
                last_worked_at = ?8,
                last_error = ?9,
                updated_at = ?10
             WHERE id = ?11",
            params![
                state.issue_number,
                state.repository,
                state.title,
                state.status,
                state.labels_json,
                state.dependencies_json,
                state.last_agent_run_id,
                state.last_worked_at.map(|dt| dt.to_rfc3339()),
                state.last_error,
                state.updated_at.to_rfc3339(),
                id,
            ],
        )?;

        if affected == 0 {
            return Err(Error::NotFound(format!(
                "Issue state with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Find an issue state by ID
    pub fn find_by_id(&self, id: i64) -> Result<IssueState> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, issue_number, repository, title, status, labels_json,
                    dependencies_json, last_agent_run_id, last_worked_at,
                    last_error, created_at, updated_at
             FROM issue_states
             WHERE id = ?1",
            params![id],
            Self::map_row,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                Error::NotFound(format!("Issue state with id {} not found", id))
            }
            _ => Error::Sqlite(e),
        })
    }

    /// Find an issue state by issue number and repository
    pub fn find_by_issue(&self, issue_number: i64, repository: &str) -> Result<IssueState> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, issue_number, repository, title, status, labels_json,
                    dependencies_json, last_agent_run_id, last_worked_at,
                    last_error, created_at, updated_at
             FROM issue_states
             WHERE issue_number = ?1 AND repository = ?2",
            params![issue_number, repository],
            Self::map_row,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(format!(
                "Issue state for #{} in {} not found",
                issue_number, repository
            )),
            _ => Error::Sqlite(e),
        })
    }

    /// Find or create an issue state
    pub fn find_or_create(
        &self,
        issue_number: i64,
        repository: &str,
        title: &str,
    ) -> Result<IssueState> {
        match self.find_by_issue(issue_number, repository) {
            Ok(state) => Ok(state),
            Err(Error::NotFound(_)) => {
                let state = IssueState::new(issue_number, repository, title);
                let id = self.insert(&state)?;
                self.find_by_id(id)
            }
            Err(e) => Err(e),
        }
    }

    /// Find all issue states for a repository
    pub fn find_by_repository(&self, repository: &str) -> Result<Vec<IssueState>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, issue_number, repository, title, status, labels_json,
                    dependencies_json, last_agent_run_id, last_worked_at,
                    last_error, created_at, updated_at
             FROM issue_states
             WHERE repository = ?1
             ORDER BY issue_number ASC",
        )?;

        let states = stmt
            .query_map(params![repository], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    /// Find all issue states with a specific status
    pub fn find_by_status(&self, status: &str) -> Result<Vec<IssueState>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, issue_number, repository, title, status, labels_json,
                    dependencies_json, last_agent_run_id, last_worked_at,
                    last_error, created_at, updated_at
             FROM issue_states
             WHERE status = ?1
             ORDER BY updated_at DESC",
        )?;

        let states = stmt
            .query_map(params![status], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    /// Find all issue states in a repository with a specific status
    pub fn find_by_repository_and_status(
        &self,
        repository: &str,
        status: &str,
    ) -> Result<Vec<IssueState>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, issue_number, repository, title, status, labels_json,
                    dependencies_json, last_agent_run_id, last_worked_at,
                    last_error, created_at, updated_at
             FROM issue_states
             WHERE repository = ?1 AND status = ?2
             ORDER BY issue_number ASC",
        )?;

        let states = stmt
            .query_map(params![repository, status], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    /// Find issues that have failed
    pub fn find_failed(&self) -> Result<Vec<IssueState>> {
        self.find_by_status("failed")
    }

    /// Find issues that are in progress
    pub fn find_in_progress(&self) -> Result<Vec<IssueState>> {
        self.find_by_status("in_progress")
    }

    /// Find issues that are blocked
    pub fn find_blocked(&self) -> Result<Vec<IssueState>> {
        self.find_by_status("blocked")
    }

    /// Find all issue states (with optional limit)
    pub fn find_all(&self, limit: Option<usize>) -> Result<Vec<IssueState>> {
        let conn = self.db.connection();

        let query = if let Some(limit) = limit {
            format!(
                "SELECT id, issue_number, repository, title, status, labels_json,
                        dependencies_json, last_agent_run_id, last_worked_at,
                        last_error, created_at, updated_at
                 FROM issue_states
                 ORDER BY updated_at DESC
                 LIMIT {}",
                limit
            )
        } else {
            "SELECT id, issue_number, repository, title, status, labels_json,
                    dependencies_json, last_agent_run_id, last_worked_at,
                    last_error, created_at, updated_at
             FROM issue_states
             ORDER BY updated_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let states = stmt
            .query_map([], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(states)
    }

    /// Delete an issue state by ID
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.db.connection();
        let affected = conn.execute("DELETE FROM issue_states WHERE id = ?1", params![id])?;

        if affected == 0 {
            return Err(Error::NotFound(format!(
                "Issue state with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Delete an issue state by issue number and repository
    pub fn delete_by_issue(&self, issue_number: i64, repository: &str) -> Result<()> {
        let conn = self.db.connection();
        let affected = conn.execute(
            "DELETE FROM issue_states WHERE issue_number = ?1 AND repository = ?2",
            params![issue_number, repository],
        )?;

        if affected == 0 {
            return Err(Error::NotFound(format!(
                "Issue state for #{} in {} not found",
                issue_number, repository
            )));
        }

        Ok(())
    }

    /// Count total issue states
    pub fn count(&self) -> Result<i64> {
        let conn = self.db.connection();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM issue_states", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Count issue states for a repository
    pub fn count_by_repository(&self, repository: &str) -> Result<i64> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issue_states WHERE repository = ?1",
            params![repository],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Count issue states by status
    pub fn count_by_status(&self, status: &str) -> Result<i64> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issue_states WHERE status = ?1",
            params![status],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Map a database row to an IssueState model
    fn map_row(row: &Row) -> rusqlite::Result<IssueState> {
        let last_worked_at_str: Option<String> = row.get(8)?;
        let created_at_str: String = row.get(10)?;
        let updated_at_str: String = row.get(11)?;

        Ok(IssueState {
            id: Some(row.get(0)?),
            issue_number: row.get(1)?,
            repository: row.get(2)?,
            title: row.get(3)?,
            status: row.get(4)?,
            labels_json: row.get(5)?,
            dependencies_json: row.get(6)?,
            last_agent_run_id: row.get(7)?,
            last_worked_at: last_worked_at_str
                .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
            last_error: row.get(9)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        10,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        11,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::in_memory().unwrap()
    }

    #[test]
    fn test_insert_and_find_by_id() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let state = IssueState::new(42, "owner/repo", "Fix the bug");
        let id = repo.insert(&state).unwrap();

        let retrieved = repo.find_by_id(id).unwrap();
        assert_eq!(retrieved.issue_number, 42);
        assert_eq!(retrieved.repository, "owner/repo");
        assert_eq!(retrieved.title, "Fix the bug");
        assert_eq!(retrieved.status, "open");
    }

    #[test]
    fn test_find_by_issue() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let state = IssueState::new(123, "test/repo", "Test issue");
        repo.insert(&state).unwrap();

        let retrieved = repo.find_by_issue(123, "test/repo").unwrap();
        assert_eq!(retrieved.issue_number, 123);
        assert_eq!(retrieved.repository, "test/repo");
    }

    #[test]
    fn test_find_by_issue_not_found() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let result = repo.find_by_issue(999, "nonexistent/repo");
        assert!(matches!(result, Err(Error::NotFound(_))));
    }

    #[test]
    fn test_find_or_create_new() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let state = repo.find_or_create(42, "owner/repo", "New issue").unwrap();
        assert_eq!(state.issue_number, 42);
        assert!(state.id.is_some());
    }

    #[test]
    fn test_find_or_create_existing() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        // Create first
        let state1 = IssueState::new(42, "owner/repo", "Original title");
        let id1 = repo.insert(&state1).unwrap();

        // Find or create should return existing
        let state2 = repo
            .find_or_create(42, "owner/repo", "Different title")
            .unwrap();
        assert_eq!(state2.id, Some(id1));
        assert_eq!(state2.title, "Original title"); // Should keep original
    }

    #[test]
    fn test_update() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let mut state = IssueState::new(42, "owner/repo", "Fix bug");
        let id = repo.insert(&state).unwrap();

        state.id = Some(id);
        state.start_work();

        repo.update(&state).unwrap();

        let updated = repo.find_by_id(id).unwrap();
        assert_eq!(updated.status, "in_progress");
        assert!(updated.last_worked_at.is_some());
    }

    #[test]
    fn test_find_by_repository() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        repo.insert(&IssueState::new(1, "owner/repo1", "Issue 1"))
            .unwrap();
        repo.insert(&IssueState::new(2, "owner/repo1", "Issue 2"))
            .unwrap();
        repo.insert(&IssueState::new(3, "owner/repo2", "Issue 3"))
            .unwrap();

        let states = repo.find_by_repository("owner/repo1").unwrap();
        assert_eq!(states.len(), 2);

        let states = repo.find_by_repository("owner/repo2").unwrap();
        assert_eq!(states.len(), 1);
    }

    #[test]
    fn test_find_by_status() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let mut state1 = IssueState::new(1, "owner/repo", "Issue 1");
        let id1 = repo.insert(&state1).unwrap();
        state1.id = Some(id1);
        state1.start_work();
        repo.update(&state1).unwrap();

        let mut state2 = IssueState::new(2, "owner/repo", "Issue 2");
        let id2 = repo.insert(&state2).unwrap();
        state2.id = Some(id2);
        state2.fail_work("Something went wrong");
        repo.update(&state2).unwrap();

        repo.insert(&IssueState::new(3, "owner/repo", "Issue 3"))
            .unwrap();

        let in_progress = repo.find_in_progress().unwrap();
        assert_eq!(in_progress.len(), 1);
        assert_eq!(in_progress[0].issue_number, 1);

        let failed = repo.find_failed().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].issue_number, 2);

        let open = repo.find_by_status("open").unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].issue_number, 3);
    }

    #[test]
    fn test_find_by_repository_and_status() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let mut state1 = IssueState::new(1, "owner/repo1", "Issue 1");
        let id1 = repo.insert(&state1).unwrap();
        state1.id = Some(id1);
        state1.start_work();
        repo.update(&state1).unwrap();

        let mut state2 = IssueState::new(2, "owner/repo2", "Issue 2");
        let id2 = repo.insert(&state2).unwrap();
        state2.id = Some(id2);
        state2.start_work();
        repo.update(&state2).unwrap();

        let states = repo
            .find_by_repository_and_status("owner/repo1", "in_progress")
            .unwrap();
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].issue_number, 1);
    }

    #[test]
    fn test_find_all() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        for i in 0..5 {
            repo.insert(&IssueState::new(i, "owner/repo", format!("Issue {}", i)))
                .unwrap();
        }

        let all = repo.find_all(None).unwrap();
        assert_eq!(all.len(), 5);

        let limited = repo.find_all(Some(3)).unwrap();
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn test_delete() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let state = IssueState::new(42, "owner/repo", "To be deleted");
        let id = repo.insert(&state).unwrap();

        assert!(repo.find_by_id(id).is_ok());

        repo.delete(id).unwrap();
        assert!(repo.find_by_id(id).is_err());
    }

    #[test]
    fn test_delete_by_issue() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let state = IssueState::new(42, "owner/repo", "To be deleted");
        repo.insert(&state).unwrap();

        assert!(repo.find_by_issue(42, "owner/repo").is_ok());

        repo.delete_by_issue(42, "owner/repo").unwrap();
        assert!(repo.find_by_issue(42, "owner/repo").is_err());
    }

    #[test]
    fn test_count() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);

        repo.insert(&IssueState::new(1, "owner/repo", "Issue 1"))
            .unwrap();
        repo.insert(&IssueState::new(2, "owner/repo", "Issue 2"))
            .unwrap();

        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn test_count_by_repository() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        repo.insert(&IssueState::new(1, "owner/repo1", "Issue 1"))
            .unwrap();
        repo.insert(&IssueState::new(2, "owner/repo1", "Issue 2"))
            .unwrap();
        repo.insert(&IssueState::new(3, "owner/repo2", "Issue 3"))
            .unwrap();

        assert_eq!(repo.count_by_repository("owner/repo1").unwrap(), 2);
        assert_eq!(repo.count_by_repository("owner/repo2").unwrap(), 1);
        assert_eq!(repo.count_by_repository("owner/repo3").unwrap(), 0);
    }

    #[test]
    fn test_count_by_status() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let mut state1 = IssueState::new(1, "owner/repo", "Issue 1");
        let id1 = repo.insert(&state1).unwrap();
        state1.id = Some(id1);
        state1.start_work();
        repo.update(&state1).unwrap();

        repo.insert(&IssueState::new(2, "owner/repo", "Issue 2"))
            .unwrap();
        repo.insert(&IssueState::new(3, "owner/repo", "Issue 3"))
            .unwrap();

        assert_eq!(repo.count_by_status("open").unwrap(), 2);
        assert_eq!(repo.count_by_status("in_progress").unwrap(), 1);
        assert_eq!(repo.count_by_status("failed").unwrap(), 0);
    }

    #[test]
    fn test_unique_constraint() {
        let db = setup_db();
        let repo = IssueStateRepository::new(&db);

        let state1 = IssueState::new(42, "owner/repo", "First");
        repo.insert(&state1).unwrap();

        // Same issue_number + repository should fail
        let state2 = IssueState::new(42, "owner/repo", "Second");
        let result = repo.insert(&state2);
        assert!(result.is_err());

        // Different repo is OK
        let state3 = IssueState::new(42, "other/repo", "Third");
        assert!(repo.insert(&state3).is_ok());

        // Different issue number is OK
        let state4 = IssueState::new(43, "owner/repo", "Fourth");
        assert!(repo.insert(&state4).is_ok());
    }
}
