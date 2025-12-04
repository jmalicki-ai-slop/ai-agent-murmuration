//! Conversation log storage repository

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::Result;

/// A message in a conversation log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationLog {
    /// Unique ID
    pub id: i64,
    /// Associated agent run ID
    pub run_id: i64,
    /// Sequence number in conversation
    pub sequence: i64,
    /// Timestamp of the message
    pub timestamp: DateTime<Utc>,
    /// Type of message (system, assistant, tool_use, tool_result, result)
    pub message_type: String,
    /// JSON-serialized message content
    pub content: String,
    /// When this record was created
    pub created_at: DateTime<Utc>,
}

/// Repository for conversation logs
#[derive(Clone)]
pub struct ConversationLogsRepo {
    pool: SqlitePool,
}

impl ConversationLogsRepo {
    /// Create a new repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Append a message to the conversation log
    pub async fn append(
        &self,
        run_id: i64,
        sequence: i64,
        message_type: &str,
        content: &str,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO conversation_logs (run_id, sequence, timestamp, message_type, content, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(run_id)
        .bind(sequence)
        .bind(&now)
        .bind(message_type)
        .bind(content)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get all messages for a run
    pub async fn get_by_run(&self, run_id: i64) -> Result<Vec<ConversationLog>> {
        let rows = sqlx::query(
            r#"
            SELECT id, run_id, sequence, timestamp, message_type, content, created_at
            FROM conversation_logs
            WHERE run_id = ?
            ORDER BY sequence ASC
            "#
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ConversationLog {
                id: row.get("id"),
                run_id: row.get("run_id"),
                sequence: row.get("sequence"),
                timestamp: row.get("timestamp"),
                message_type: row.get("message_type"),
                content: row.get("content"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    /// Get the last sequence number for a run
    pub async fn get_last_sequence(&self, run_id: i64) -> Result<Option<i64>> {
        // First check if there are any rows for this run
        let count: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM conversation_logs
            WHERE run_id = ?
            "#
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?
        .get("count");

        if count == 0 {
            return Ok(None);
        }

        let row = sqlx::query(
            r#"
            SELECT MAX(sequence) as max_seq
            FROM conversation_logs
            WHERE run_id = ?
            "#
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.try_get("max_seq").ok())
    }

    /// Count messages for a run
    pub async fn count_by_run(&self, run_id: i64) -> Result<i64> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM conversation_logs
            WHERE run_id = ?
            "#
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("count"))
    }

    /// Delete all messages for a run
    pub async fn delete_by_run(&self, run_id: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM conversation_logs
            WHERE run_id = ?
            "#
        )
        .bind(run_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_append_and_get_messages() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();

        let run_id = db
            .agent_runs()
            .create("issue-42", "coder", "/tmp/worktree", "Test")
            .await
            .unwrap();

        let repo = db.conversation_logs();

        repo.append(run_id, 0, "system", r#"{"type":"system"}"#)
            .await
            .unwrap();
        repo.append(run_id, 1, "assistant", r#"{"type":"assistant"}"#)
            .await
            .unwrap();

        let logs = repo.get_by_run(run_id).await.unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].sequence, 0);
        assert_eq!(logs[1].sequence, 1);
    }

    #[tokio::test]
    async fn test_get_last_sequence() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();

        let run_id = db
            .agent_runs()
            .create("issue-42", "coder", "/tmp/worktree", "Test")
            .await
            .unwrap();

        let repo = db.conversation_logs();

        let last_seq = repo.get_last_sequence(run_id).await.unwrap();
        assert_eq!(last_seq, None);

        repo.append(run_id, 0, "system", r#"{"type":"system"}"#)
            .await
            .unwrap();

        let last_seq = repo.get_last_sequence(run_id).await.unwrap();
        assert!(last_seq.is_some(), "Expected Some(0) but got None");
        assert_eq!(last_seq.unwrap(), 0);

        repo.append(run_id, 5, "assistant", r#"{"type":"assistant"}"#)
            .await
            .unwrap();
        assert_eq!(repo.get_last_sequence(run_id).await.unwrap(), Some(5));
    }
}
