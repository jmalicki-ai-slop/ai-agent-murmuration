//! Repository for conversation log operations

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};

use crate::models::ConversationLog;
use crate::{Database, Error, Result};

/// Repository for managing conversation log records
pub struct ConversationRepository<'db> {
    db: &'db Database,
}

impl<'db> ConversationRepository<'db> {
    /// Create a new repository instance
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Insert a new conversation log entry
    pub fn insert(&self, log: &ConversationLog) -> Result<i64> {
        let conn = self.db.connection();

        conn.execute(
            "INSERT INTO conversation_logs (
                agent_run_id, sequence, timestamp, message_type, message_json, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                log.agent_run_id,
                log.sequence,
                log.timestamp.to_rfc3339(),
                log.message_type,
                log.message_json,
                log.created_at.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Insert multiple log entries in a single transaction (for batch inserts)
    pub fn insert_batch(&self, logs: &[ConversationLog]) -> Result<()> {
        let conn = self.db.connection();

        // Start transaction
        conn.execute("BEGIN TRANSACTION", [])?;

        let result = (|| {
            for log in logs {
                conn.execute(
                    "INSERT INTO conversation_logs (
                        agent_run_id, sequence, timestamp, message_type, message_json, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        log.agent_run_id,
                        log.sequence,
                        log.timestamp.to_rfc3339(),
                        log.message_type,
                        log.message_json,
                        log.created_at.to_rfc3339(),
                    ],
                )?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK", [])?;
                Err(e)
            }
        }
    }

    /// Find a conversation log entry by ID
    pub fn find_by_id(&self, id: i64) -> Result<ConversationLog> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, agent_run_id, sequence, timestamp, message_type, message_json, created_at
             FROM conversation_logs
             WHERE id = ?1",
            params![id],
            Self::map_row,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                Error::NotFound(format!("Conversation log with id {} not found", id))
            }
            _ => Error::Sqlite(e),
        })
    }

    /// Find all conversation logs for a specific agent run, ordered by sequence
    pub fn find_by_agent_run(&self, agent_run_id: i64) -> Result<Vec<ConversationLog>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_run_id, sequence, timestamp, message_type, message_json, created_at
             FROM conversation_logs
             WHERE agent_run_id = ?1
             ORDER BY sequence ASC",
        )?;

        let logs = stmt
            .query_map(params![agent_run_id], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Find conversation logs by message type
    pub fn find_by_message_type(&self, message_type: &str) -> Result<Vec<ConversationLog>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_run_id, sequence, timestamp, message_type, message_json, created_at
             FROM conversation_logs
             WHERE message_type = ?1
             ORDER BY timestamp DESC",
        )?;

        let logs = stmt
            .query_map(params![message_type], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Find conversation logs within a time range
    pub fn find_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ConversationLog>> {
        let conn = self.db.connection();
        let mut stmt = conn.prepare(
            "SELECT id, agent_run_id, sequence, timestamp, message_type, message_json, created_at
             FROM conversation_logs
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;

        let logs = stmt
            .query_map(params![start.to_rfc3339(), end.to_rfc3339()], Self::map_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Get the next sequence number for an agent run (for streaming writes)
    pub fn next_sequence(&self, agent_run_id: i64) -> Result<i64> {
        let conn = self.db.connection();
        let max_seq: Option<i64> = conn
            .query_row(
                "SELECT MAX(sequence) FROM conversation_logs WHERE agent_run_id = ?1",
                params![agent_run_id],
                |row| row.get(0),
            )
            .ok();

        Ok(max_seq.map(|s| s + 1).unwrap_or(0))
    }

    /// Count conversation logs for a specific agent run
    pub fn count_by_agent_run(&self, agent_run_id: i64) -> Result<i64> {
        let conn = self.db.connection();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM conversation_logs WHERE agent_run_id = ?1",
            params![agent_run_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Delete all conversation logs for a specific agent run
    pub fn delete_by_agent_run(&self, agent_run_id: i64) -> Result<usize> {
        let conn = self.db.connection();
        let affected = conn.execute(
            "DELETE FROM conversation_logs WHERE agent_run_id = ?1",
            params![agent_run_id],
        )?;
        Ok(affected)
    }

    /// Map a database row to a ConversationLog model
    fn map_row(row: &Row) -> rusqlite::Result<ConversationLog> {
        let timestamp_str: String = row.get(3)?;
        let created_at_str: String = row.get(6)?;

        Ok(ConversationLog {
            id: Some(row.get(0)?),
            agent_run_id: row.get(1)?,
            sequence: row.get(2)?,
            timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc),
            message_type: row.get(4)?,
            message_json: row.get(5)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
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
    use crate::models::AgentRun;
    use crate::repos::AgentRunRepository;

    fn setup_db() -> Database {
        Database::in_memory().unwrap()
    }

    fn create_test_agent_run(db: &Database) -> i64 {
        let repo = AgentRunRepository::new(db);
        let run = AgentRun::new("implementer", "Test task", "/tmp", "{}");
        repo.insert(&run).unwrap()
    }

    #[test]
    fn test_insert_and_find_by_id() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        let log = ConversationLog::new(
            agent_run_id,
            0,
            "assistant",
            r#"{"type":"assistant","message":{"content":[]}}"#,
        );
        let id = repo.insert(&log).unwrap();

        let retrieved = repo.find_by_id(id).unwrap();
        assert_eq!(retrieved.agent_run_id, agent_run_id);
        assert_eq!(retrieved.sequence, 0);
        assert_eq!(retrieved.message_type, "assistant");
    }

    #[test]
    fn test_find_by_agent_run() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        let log1 = ConversationLog::new(agent_run_id, 0, "system", r#"{"type":"system"}"#);
        let log2 = ConversationLog::new(agent_run_id, 1, "assistant", r#"{"type":"assistant"}"#);
        let log3 = ConversationLog::new(agent_run_id, 2, "tool_use", r#"{"type":"tool_use"}"#);

        repo.insert(&log1).unwrap();
        repo.insert(&log2).unwrap();
        repo.insert(&log3).unwrap();

        let logs = repo.find_by_agent_run(agent_run_id).unwrap();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].sequence, 0);
        assert_eq!(logs[1].sequence, 1);
        assert_eq!(logs[2].sequence, 2);
    }

    #[test]
    fn test_find_by_message_type() {
        let db = setup_db();
        let agent_run_id1 = create_test_agent_run(&db);
        let agent_run_id2 = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        repo.insert(&ConversationLog::new(
            agent_run_id1,
            0,
            "assistant",
            r#"{"type":"assistant"}"#,
        ))
        .unwrap();
        repo.insert(&ConversationLog::new(
            agent_run_id2,
            0,
            "assistant",
            r#"{"type":"assistant"}"#,
        ))
        .unwrap();
        repo.insert(&ConversationLog::new(
            agent_run_id1,
            1,
            "tool_use",
            r#"{"type":"tool_use"}"#,
        ))
        .unwrap();

        let assistants = repo.find_by_message_type("assistant").unwrap();
        assert_eq!(assistants.len(), 2);

        let tool_uses = repo.find_by_message_type("tool_use").unwrap();
        assert_eq!(tool_uses.len(), 1);
    }

    #[test]
    fn test_next_sequence() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        // First sequence should be 0
        assert_eq!(repo.next_sequence(agent_run_id).unwrap(), 0);

        // Insert a log
        repo.insert(&ConversationLog::new(
            agent_run_id,
            0,
            "system",
            r#"{"type":"system"}"#,
        ))
        .unwrap();

        // Next sequence should be 1
        assert_eq!(repo.next_sequence(agent_run_id).unwrap(), 1);

        // Insert another log
        repo.insert(&ConversationLog::new(
            agent_run_id,
            1,
            "assistant",
            r#"{"type":"assistant"}"#,
        ))
        .unwrap();

        // Next sequence should be 2
        assert_eq!(repo.next_sequence(agent_run_id).unwrap(), 2);
    }

    #[test]
    fn test_insert_batch() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        let logs = vec![
            ConversationLog::new(agent_run_id, 0, "system", r#"{"type":"system"}"#),
            ConversationLog::new(agent_run_id, 1, "assistant", r#"{"type":"assistant"}"#),
            ConversationLog::new(agent_run_id, 2, "tool_use", r#"{"type":"tool_use"}"#),
        ];

        repo.insert_batch(&logs).unwrap();

        let retrieved = repo.find_by_agent_run(agent_run_id).unwrap();
        assert_eq!(retrieved.len(), 3);
    }

    #[test]
    fn test_count_by_agent_run() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        assert_eq!(repo.count_by_agent_run(agent_run_id).unwrap(), 0);

        repo.insert(&ConversationLog::new(
            agent_run_id,
            0,
            "system",
            r#"{"type":"system"}"#,
        ))
        .unwrap();
        repo.insert(&ConversationLog::new(
            agent_run_id,
            1,
            "assistant",
            r#"{"type":"assistant"}"#,
        ))
        .unwrap();

        assert_eq!(repo.count_by_agent_run(agent_run_id).unwrap(), 2);
    }

    #[test]
    fn test_delete_by_agent_run() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        repo.insert(&ConversationLog::new(
            agent_run_id,
            0,
            "system",
            r#"{"type":"system"}"#,
        ))
        .unwrap();
        repo.insert(&ConversationLog::new(
            agent_run_id,
            1,
            "assistant",
            r#"{"type":"assistant"}"#,
        ))
        .unwrap();

        assert_eq!(repo.count_by_agent_run(agent_run_id).unwrap(), 2);

        let deleted = repo.delete_by_agent_run(agent_run_id).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(repo.count_by_agent_run(agent_run_id).unwrap(), 0);
    }

    #[test]
    fn test_unique_constraint() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        let log1 = ConversationLog::new(agent_run_id, 0, "system", r#"{"type":"system"}"#);
        repo.insert(&log1).unwrap();

        // Try to insert duplicate sequence for same agent_run
        let log2 = ConversationLog::new(agent_run_id, 0, "assistant", r#"{"type":"assistant"}"#);
        let result = repo.insert(&log2);
        assert!(result.is_err());
    }

    #[test]
    fn test_cascade_delete() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let conv_repo = ConversationRepository::new(&db);
        let agent_repo = AgentRunRepository::new(&db);

        // Insert conversation logs
        conv_repo
            .insert(&ConversationLog::new(
                agent_run_id,
                0,
                "system",
                r#"{"type":"system"}"#,
            ))
            .unwrap();
        conv_repo
            .insert(&ConversationLog::new(
                agent_run_id,
                1,
                "assistant",
                r#"{"type":"assistant"}"#,
            ))
            .unwrap();

        assert_eq!(conv_repo.count_by_agent_run(agent_run_id).unwrap(), 2);

        // Delete the agent run
        agent_repo.delete(agent_run_id).unwrap();

        // Conversation logs should be deleted automatically due to CASCADE
        assert_eq!(conv_repo.count_by_agent_run(agent_run_id).unwrap(), 0);
    }

    #[test]
    fn test_find_by_time_range() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let repo = ConversationRepository::new(&db);

        let now = Utc::now();
        let log =
            ConversationLog::with_timestamp(agent_run_id, 0, "system", r#"{"type":"system"}"#, now);
        repo.insert(&log).unwrap();

        let start = now - chrono::Duration::hours(1);
        let end = now + chrono::Duration::hours(1);

        let logs = repo.find_by_time_range(start, end).unwrap();
        assert_eq!(logs.len(), 1);
    }
}
