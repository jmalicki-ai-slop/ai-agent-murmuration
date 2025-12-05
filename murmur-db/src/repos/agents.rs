//! Repository for agent run operations

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};

use crate::models::AgentRun;
use crate::{Database, Error, Result};

/// Repository for managing agent run records
pub struct AgentRunRepository<'db> {
    db: &'db Database,
}

impl<'db> AgentRunRepository<'db> {
    /// Create a new repository instance
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Insert a new agent run record
    pub fn insert(&self, run: &AgentRun) -> Result<i64> {
        let conn = self.db.connection();

        conn.execute(
            "INSERT INTO agent_runs (
                agent_type, issue_number, prompt, workdir, config_json, pid,
                start_time, end_time, exit_code, duration_seconds, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                run.agent_type,
                run.issue_number,
                run.prompt,
                run.workdir,
                run.config_json,
                run.pid,
                run.start_time.to_rfc3339(),
                run.end_time.map(|dt| dt.to_rfc3339()),
                run.exit_code,
                run.duration_seconds,
                run.created_at.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Update an existing agent run record
    pub fn update(&self, run: &AgentRun) -> Result<()> {
        let id = run
            .id
            .ok_or_else(|| Error::InvalidData("Cannot update record without ID".to_string()))?;

        let conn = self.db.connection();
        let affected = conn.execute(
            "UPDATE agent_runs SET
                agent_type = ?1,
                issue_number = ?2,
                prompt = ?3,
                workdir = ?4,
                config_json = ?5,
                pid = ?6,
                start_time = ?7,
                end_time = ?8,
                exit_code = ?9,
                duration_seconds = ?10
             WHERE id = ?11",
            params![
                run.agent_type,
                run.issue_number,
                run.prompt,
                run.workdir,
                run.config_json,
                run.pid,
                run.start_time.to_rfc3339(),
                run.end_time.map(|dt| dt.to_rfc3339()),
                run.exit_code,
                run.duration_seconds,
                id,
            ],
        )?;

        if affected == 0 {
            return Err(Error::NotFound(format!(
                "Agent run with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Find an agent run by ID
    pub fn find_by_id(&self, id: i64) -> Result<AgentRun> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                    start_time, end_time, exit_code, duration_seconds, created_at
             FROM agent_runs
             WHERE id = ?1",
            params![id],
            Self::map_row,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                Error::NotFound(format!("Agent run with id {} not found", id))
            }
            _ => Error::Sqlite(e),
        })
    }

    /// Find all agent runs for a specific issue
    pub fn find_by_issue(&self, issue_number: i64) -> Result<Vec<AgentRun>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                    start_time, end_time, exit_code, duration_seconds, created_at
             FROM agent_runs
             WHERE issue_number = ?1
             ORDER BY start_time DESC",
        )?;

        let runs = stmt
            .query_map(params![issue_number], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Find agent runs within a date range
    pub fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AgentRun>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                    start_time, end_time, exit_code, duration_seconds, created_at
             FROM agent_runs
             WHERE start_time >= ?1 AND start_time <= ?2
             ORDER BY start_time DESC",
        )?;

        let runs = stmt
            .query_map(params![start.to_rfc3339(), end.to_rfc3339()], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Find all agent runs of a specific type
    pub fn find_by_agent_type(&self, agent_type: &str) -> Result<Vec<AgentRun>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                    start_time, end_time, exit_code, duration_seconds, created_at
             FROM agent_runs
             WHERE agent_type = ?1
             ORDER BY start_time DESC",
        )?;

        let runs = stmt
            .query_map(params![agent_type], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Find all agent runs (with optional limit)
    pub fn find_all(&self, limit: Option<usize>) -> Result<Vec<AgentRun>> {
        let conn = self.db.connection();

        let query = if let Some(limit) = limit {
            format!(
                "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                        start_time, end_time, exit_code, duration_seconds, created_at
                 FROM agent_runs
                 ORDER BY start_time DESC
                 LIMIT {}",
                limit
            )
        } else {
            "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                    start_time, end_time, exit_code, duration_seconds, created_at
             FROM agent_runs
             ORDER BY start_time DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let runs = stmt
            .query_map([], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Delete an agent run by ID
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.db.connection();
        let affected = conn.execute("DELETE FROM agent_runs WHERE id = ?1", params![id])?;

        if affected == 0 {
            return Err(Error::NotFound(format!(
                "Agent run with id {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count total agent runs
    pub fn count(&self) -> Result<i64> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM agent_runs", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Count agent runs for a specific issue
    pub fn count_by_issue(&self, issue_number: i64) -> Result<i64> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE issue_number = ?1",
            params![issue_number],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Find all running agents (no end_time, has PID)
    pub fn find_running(&self) -> Result<Vec<AgentRun>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_type, issue_number, prompt, workdir, config_json, pid,
                    start_time, end_time, exit_code, duration_seconds, created_at
             FROM agent_runs
             WHERE end_time IS NULL AND pid IS NOT NULL
             ORDER BY start_time DESC",
        )?;

        let runs = stmt
            .query_map([], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Map a database row to an AgentRun model
    fn map_row(row: &Row) -> rusqlite::Result<AgentRun> {
        let start_time_str: String = row.get(7)?;
        let end_time_str: Option<String> = row.get(8)?;
        let created_at_str: String = row.get(11)?;

        Ok(AgentRun {
            id: Some(row.get(0)?),
            agent_type: row.get(1)?,
            issue_number: row.get(2)?,
            prompt: row.get(3)?,
            workdir: row.get(4)?,
            config_json: row.get(5)?,
            pid: row.get(6)?,
            start_time: DateTime::parse_from_rfc3339(&start_time_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        7,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc),
            end_time: end_time_str
                .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        8,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
            exit_code: row.get(9)?,
            duration_seconds: row.get(10)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
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
        let repo = AgentRunRepository::new(&db);

        let run = AgentRun::new(
            "implementer",
            "Fix bug",
            "/tmp/work",
            r#"{"model":"sonnet"}"#,
        );
        let id = repo.insert(&run).unwrap();

        let retrieved = repo.find_by_id(id).unwrap();
        assert_eq!(retrieved.agent_type, "implementer");
        assert_eq!(retrieved.prompt, "Fix bug");
        assert_eq!(retrieved.workdir, "/tmp/work");
    }

    #[test]
    fn test_update() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        let mut run = AgentRun::new("planner", "Plan feature", "/tmp/work", "{}");
        let id = repo.insert(&run).unwrap();

        run.id = Some(id);
        run.complete(0);

        repo.update(&run).unwrap();

        let updated = repo.find_by_id(id).unwrap();
        assert!(updated.is_completed());
        assert!(updated.is_successful());
    }

    #[test]
    fn test_find_by_issue() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        let run1 = AgentRun::new("implementer", "Task 1", "/tmp/1", "{}").with_issue_number(42);
        let run2 = AgentRun::new("reviewer", "Task 2", "/tmp/2", "{}").with_issue_number(42);
        let run3 = AgentRun::new("planner", "Task 3", "/tmp/3", "{}").with_issue_number(99);

        repo.insert(&run1).unwrap();
        repo.insert(&run2).unwrap();
        repo.insert(&run3).unwrap();

        let runs_42 = repo.find_by_issue(42).unwrap();
        assert_eq!(runs_42.len(), 2);

        let runs_99 = repo.find_by_issue(99).unwrap();
        assert_eq!(runs_99.len(), 1);
    }

    #[test]
    fn test_find_by_agent_type() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        repo.insert(&AgentRun::new("implementer", "Task 1", "/tmp/1", "{}"))
            .unwrap();
        repo.insert(&AgentRun::new("implementer", "Task 2", "/tmp/2", "{}"))
            .unwrap();
        repo.insert(&AgentRun::new("planner", "Task 3", "/tmp/3", "{}"))
            .unwrap();

        let implementers = repo.find_by_agent_type("implementer").unwrap();
        assert_eq!(implementers.len(), 2);

        let planners = repo.find_by_agent_type("planner").unwrap();
        assert_eq!(planners.len(), 1);
    }

    #[test]
    fn test_find_by_date_range() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        let now = Utc::now();
        let run = AgentRun::new("implementer", "Task", "/tmp", "{}");
        repo.insert(&run).unwrap();

        let start = now - chrono::Duration::hours(1);
        let end = now + chrono::Duration::hours(1);

        let runs = repo.find_by_date_range(start, end).unwrap();
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn test_find_all() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        for i in 0..5 {
            let run = AgentRun::new("implementer", format!("Task {}", i), "/tmp", "{}");
            repo.insert(&run).unwrap();
        }

        let all_runs = repo.find_all(None).unwrap();
        assert_eq!(all_runs.len(), 5);

        let limited = repo.find_all(Some(3)).unwrap();
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn test_delete() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        let run = AgentRun::new("implementer", "Task", "/tmp", "{}");
        let id = repo.insert(&run).unwrap();

        assert!(repo.find_by_id(id).is_ok());

        repo.delete(id).unwrap();
        assert!(repo.find_by_id(id).is_err());
    }

    #[test]
    fn test_count() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        assert_eq!(repo.count().unwrap(), 0);

        repo.insert(&AgentRun::new("implementer", "Task 1", "/tmp", "{}"))
            .unwrap();
        repo.insert(&AgentRun::new("planner", "Task 2", "/tmp", "{}"))
            .unwrap();

        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn test_count_by_issue() {
        let db = setup_db();
        let repo = AgentRunRepository::new(&db);

        repo.insert(&AgentRun::new("implementer", "Task 1", "/tmp", "{}").with_issue_number(42))
            .unwrap();
        repo.insert(&AgentRun::new("planner", "Task 2", "/tmp", "{}").with_issue_number(42))
            .unwrap();
        repo.insert(&AgentRun::new("reviewer", "Task 3", "/tmp", "{}").with_issue_number(99))
            .unwrap();

        assert_eq!(repo.count_by_issue(42).unwrap(), 2);
        assert_eq!(repo.count_by_issue(99).unwrap(), 1);
        assert_eq!(repo.count_by_issue(123).unwrap(), 0);
    }
}
