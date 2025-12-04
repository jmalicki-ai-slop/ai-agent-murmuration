//! Conversation log repository
//!
//! Stores and retrieves conversation messages from agent runs.

use crate::schema::{ConversationMessage, MessageRole};
use crate::Result;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{Row, SqlitePool};

/// Repository for conversation messages
pub struct ConversationRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ConversationRepository<'a> {
    /// Create a new conversation repository
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new conversation message
    ///
    /// This is designed to be called during agent execution to stream
    /// conversation messages to the database in real-time.
    pub async fn create(
        &self,
        agent_run_id: i64,
        role: MessageRole,
        content: impl Into<String>,
    ) -> Result<i64> {
        let content = content.into();
        let role_str = role.as_str();

        let result = sqlx::query(
            r#"
            INSERT INTO conversation_messages (agent_run_id, role, content)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(agent_run_id)
        .bind(role_str)
        .bind(content)
        .execute(self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Create a conversation message with tool use details
    pub async fn create_with_tool_use(
        &self,
        agent_run_id: i64,
        role: MessageRole,
        content: impl Into<String>,
        tool_use: JsonValue,
    ) -> Result<i64> {
        let content = content.into();
        let role_str = role.as_str();
        let tool_use_json = serde_json::to_string(&tool_use)?;

        let result = sqlx::query(
            r#"
            INSERT INTO conversation_messages (agent_run_id, role, content, tool_use)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(agent_run_id)
        .bind(role_str)
        .bind(content)
        .bind(tool_use_json)
        .execute(self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Create a conversation message with tool result details
    pub async fn create_with_tool_result(
        &self,
        agent_run_id: i64,
        role: MessageRole,
        content: impl Into<String>,
        tool_result: JsonValue,
    ) -> Result<i64> {
        let content = content.into();
        let role_str = role.as_str();
        let tool_result_json = serde_json::to_string(&tool_result)?;

        let result = sqlx::query(
            r#"
            INSERT INTO conversation_messages (agent_run_id, role, content, tool_result)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(agent_run_id)
        .bind(role_str)
        .bind(content)
        .bind(tool_result_json)
        .execute(self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Create a conversation message with token cost information
    pub async fn create_with_cost(
        &self,
        agent_run_id: i64,
        role: MessageRole,
        content: impl Into<String>,
        tokens_input: Option<i64>,
        tokens_output: Option<i64>,
        cost_usd: Option<f64>,
    ) -> Result<i64> {
        let content = content.into();
        let role_str = role.as_str();

        let result = sqlx::query(
            r#"
            INSERT INTO conversation_messages
                (agent_run_id, role, content, tokens_input, tokens_output, cost_usd)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(agent_run_id)
        .bind(role_str)
        .bind(content)
        .bind(tokens_input)
        .bind(tokens_output)
        .bind(cost_usd)
        .execute(self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get all conversation messages for an agent run
    pub async fn get_by_agent_run(&self, agent_run_id: i64) -> Result<Vec<ConversationMessage>> {
        let messages = sqlx::query_as::<_, ConversationMessage>(
            r#"
            SELECT id, agent_run_id, role, content, tool_use, tool_result,
                   timestamp, tokens_input, tokens_output, cost_usd
            FROM conversation_messages
            WHERE agent_run_id = ?
            ORDER BY timestamp ASC, id ASC
            "#,
        )
        .bind(agent_run_id)
        .fetch_all(self.pool)
        .await?;

        Ok(messages)
    }

    /// Get messages for an agent run within a time range
    pub async fn get_by_agent_run_range(
        &self,
        agent_run_id: i64,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ConversationMessage>> {
        let messages = sqlx::query_as::<_, ConversationMessage>(
            r#"
            SELECT id, agent_run_id, role, content, tool_use, tool_result,
                   timestamp, tokens_input, tokens_output, cost_usd
            FROM conversation_messages
            WHERE agent_run_id = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp ASC, id ASC
            "#,
        )
        .bind(agent_run_id)
        .bind(start)
        .bind(end)
        .fetch_all(self.pool)
        .await?;

        Ok(messages)
    }

    /// Get the latest N messages for an agent run
    pub async fn get_latest(
        &self,
        agent_run_id: i64,
        limit: i64,
    ) -> Result<Vec<ConversationMessage>> {
        let messages = sqlx::query_as::<_, ConversationMessage>(
            r#"
            SELECT id, agent_run_id, role, content, tool_use, tool_result,
                   timestamp, tokens_input, tokens_output, cost_usd
            FROM conversation_messages
            WHERE agent_run_id = ?
            ORDER BY timestamp DESC, id DESC
            LIMIT ?
            "#,
        )
        .bind(agent_run_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        // Reverse to get chronological order
        Ok(messages.into_iter().rev().collect())
    }

    /// Count messages for an agent run
    pub async fn count(&self, agent_run_id: i64) -> Result<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM conversation_messages
            WHERE agent_run_id = ?
            "#,
        )
        .bind(agent_run_id)
        .fetch_one(self.pool)
        .await?;

        let count: i64 = result.get("count");
        Ok(count)
    }

    /// Get total token usage for an agent run
    pub async fn get_token_usage(&self, agent_run_id: i64) -> Result<(i64, i64)> {
        let result = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(tokens_input), 0) as total_input,
                COALESCE(SUM(tokens_output), 0) as total_output
            FROM conversation_messages
            WHERE agent_run_id = ?
            "#,
        )
        .bind(agent_run_id)
        .fetch_one(self.pool)
        .await?;

        let total_input: i64 = result.get("total_input");
        let total_output: i64 = result.get("total_output");
        Ok((total_input, total_output))
    }

    /// Get total cost for an agent run
    pub async fn get_total_cost(&self, agent_run_id: i64) -> Result<f64> {
        let result = sqlx::query(
            r#"
            SELECT COALESCE(SUM(cost_usd), 0.0) as total_cost
            FROM conversation_messages
            WHERE agent_run_id = ?
            "#,
        )
        .bind(agent_run_id)
        .fetch_one(self.pool)
        .await?;

        let total_cost: f64 = result.get("total_cost");
        Ok(total_cost)
    }

    /// Delete all messages for an agent run
    ///
    /// This is automatically done via CASCADE when an agent run is deleted,
    /// but can also be called explicitly.
    pub async fn delete_by_agent_run(&self, agent_run_id: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM conversation_messages
            WHERE agent_run_id = ?
            "#,
        )
        .bind(agent_run_id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Search messages by content
    pub async fn search(
        &self,
        agent_run_id: Option<i64>,
        search_term: &str,
    ) -> Result<Vec<ConversationMessage>> {
        let search_pattern = format!("%{}%", search_term);

        let messages = if let Some(run_id) = agent_run_id {
            sqlx::query_as::<_, ConversationMessage>(
                r#"
                SELECT id, agent_run_id, role, content, tool_use, tool_result,
                       timestamp, tokens_input, tokens_output, cost_usd
                FROM conversation_messages
                WHERE agent_run_id = ? AND content LIKE ?
                ORDER BY timestamp ASC, id ASC
                "#,
            )
            .bind(run_id)
            .bind(search_pattern)
            .fetch_all(self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ConversationMessage>(
                r#"
                SELECT id, agent_run_id, role, content, tool_use, tool_result,
                       timestamp, tokens_input, tokens_output, cost_usd
                FROM conversation_messages
                WHERE content LIKE ?
                ORDER BY timestamp ASC, id ASC
                "#,
            )
            .bind(search_pattern)
            .fetch_all(self.pool)
            .await?
        };

        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    async fn setup_test_db() -> Database {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("murmur_test_conversations_{}.db", uuid::Uuid::new_v4()));

        let db = Database::with_path(db_path.clone()).await.unwrap();
        db.migrate().await.unwrap();

        // Create a test agent run
        sqlx::query(
            r#"
            INSERT INTO agent_runs (id, agent_type, status, started_at)
            VALUES (1, 'test', 'running', CURRENT_TIMESTAMP)
            "#,
        )
        .execute(db.pool())
        .await
        .unwrap();

        db
    }

    #[tokio::test]
    async fn test_create_message() {
        let db = setup_test_db().await;
        let repo = ConversationRepository::new(db.pool());

        let message_id = repo
            .create(1, MessageRole::User, "Hello, Claude!")
            .await
            .unwrap();

        assert_eq!(message_id, 1);

        let messages = repo.get_by_agent_run(1).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, Claude!");
        assert_eq!(messages[0].role, "user");
    }

    #[tokio::test]
    async fn test_create_with_tool_use() {
        let db = setup_test_db().await;
        let repo = ConversationRepository::new(db.pool());

        let tool_use = serde_json::json!({
            "tool": "Read",
            "file_path": "/test.txt"
        });

        let message_id = repo
            .create_with_tool_use(1, MessageRole::Assistant, "Reading file...", tool_use)
            .await
            .unwrap();

        assert!(message_id > 0);

        let messages = repo.get_by_agent_run(1).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].tool_use.is_some());
    }

    #[tokio::test]
    async fn test_get_latest() {
        let db = setup_test_db().await;
        let repo = ConversationRepository::new(db.pool());

        // Create multiple messages
        for i in 1..=5 {
            repo.create(1, MessageRole::User, format!("Message {}", i))
                .await
                .unwrap();
        }

        let latest = repo.get_latest(1, 3).await.unwrap();
        assert_eq!(latest.len(), 3);
        assert_eq!(latest[0].content, "Message 3");
        assert_eq!(latest[2].content, "Message 5");
    }

    #[tokio::test]
    async fn test_count() {
        let db = setup_test_db().await;
        let repo = ConversationRepository::new(db.pool());

        let count = repo.count(1).await.unwrap();
        assert_eq!(count, 0);

        repo.create(1, MessageRole::User, "Test").await.unwrap();
        let count = repo.count(1).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_token_usage() {
        let db = setup_test_db().await;
        let repo = ConversationRepository::new(db.pool());

        repo.create_with_cost(1, MessageRole::Assistant, "Response", Some(100), Some(50), Some(0.001))
            .await
            .unwrap();
        repo.create_with_cost(1, MessageRole::Assistant, "Another", Some(150), Some(75), Some(0.002))
            .await
            .unwrap();

        let (input, output) = repo.get_token_usage(1).await.unwrap();
        assert_eq!(input, 250);
        assert_eq!(output, 125);

        let cost = repo.get_total_cost(1).await.unwrap();
        assert!((cost - 0.003).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_search() {
        let db = setup_test_db().await;
        let repo = ConversationRepository::new(db.pool());

        repo.create(1, MessageRole::User, "Find this message")
            .await
            .unwrap();
        repo.create(1, MessageRole::User, "Another message")
            .await
            .unwrap();
        repo.create(1, MessageRole::User, "Find this too")
            .await
            .unwrap();

        let results = repo.search(Some(1), "Find").await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
