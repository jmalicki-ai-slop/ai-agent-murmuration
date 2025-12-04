//! Stream handler for capturing agent conversations to database

use crate::repos::ConversationRepository;
use crate::schema::MessageRole;
use crate::Result;
use serde_json::json;
use sqlx::SqlitePool;

/// Stream handler that writes conversation messages to the database
///
/// This implements the murmur-core StreamHandler trait to capture
/// real-time conversation data from agent execution.
pub struct DatabaseStreamHandler<'a> {
    repo: ConversationRepository<'a>,
    agent_run_id: i64,
    current_assistant_text: String,
    current_tool_use: Option<serde_json::Value>,
    verbose: bool,
}

impl<'a> DatabaseStreamHandler<'a> {
    /// Create a new database stream handler
    pub fn new(pool: &'a SqlitePool, agent_run_id: i64, verbose: bool) -> Self {
        Self {
            repo: ConversationRepository::new(pool),
            agent_run_id,
            current_assistant_text: String::new(),
            current_tool_use: None,
            verbose,
        }
    }

    /// Flush the current assistant message to the database
    async fn flush_assistant_message(&mut self) -> Result<()> {
        if !self.current_assistant_text.is_empty() {
            if let Some(tool_use) = self.current_tool_use.take() {
                self.repo
                    .create_with_tool_use(
                        self.agent_run_id,
                        MessageRole::Assistant,
                        &self.current_assistant_text,
                        tool_use,
                    )
                    .await?;
            } else {
                self.repo
                    .create(
                        self.agent_run_id,
                        MessageRole::Assistant,
                        &self.current_assistant_text,
                    )
                    .await?;
            }
            self.current_assistant_text.clear();
        }
        Ok(())
    }

    /// Store a system message
    pub async fn store_system(&mut self, subtype: Option<&str>, session_id: Option<&str>) -> Result<()> {
        let content = format!(
            "System message: {} (session: {})",
            subtype.unwrap_or("unknown"),
            session_id.unwrap_or("none")
        );

        self.repo
            .create(self.agent_run_id, MessageRole::System, content)
            .await?;

        Ok(())
    }

    /// Store assistant text (accumulates until flushed)
    pub async fn store_assistant_text(&mut self, text: &str) -> Result<()> {
        self.current_assistant_text.push_str(text);
        Ok(())
    }

    /// Store tool use information
    pub async fn store_tool_use(&mut self, tool: &str, input: &serde_json::Value) -> Result<()> {
        // Flush any pending assistant text first
        self.flush_assistant_message().await?;

        let tool_data = json!({
            "tool": tool,
            "input": input,
        });

        self.current_tool_use = Some(tool_data);

        Ok(())
    }

    /// Store tool result
    pub async fn store_tool_result(&mut self, output: &str, is_error: bool) -> Result<()> {
        let tool_result = json!({
            "output": output,
            "is_error": is_error,
        });

        self.repo
            .create_with_tool_result(
                self.agent_run_id,
                MessageRole::System,
                if is_error { "Tool error" } else { "Tool result" },
                tool_result,
            )
            .await?;

        Ok(())
    }

    /// Store completion information with cost
    pub async fn store_complete(
        &mut self,
        cost: Option<&CostInfo>,
        duration_ms: Option<u64>,
    ) -> Result<()> {
        // Flush any pending assistant text
        self.flush_assistant_message().await?;

        if let Some(c) = cost {
            let content = format!(
                "Completion: {} input tokens, {} output tokens, duration: {}ms",
                c.input_tokens,
                c.output_tokens,
                duration_ms.unwrap_or(0)
            );

            self.repo
                .create_with_cost(
                    self.agent_run_id,
                    MessageRole::System,
                    content,
                    Some(c.input_tokens as i64),
                    Some(c.output_tokens as i64),
                    c.cost_usd,
                )
                .await?;
        }

        Ok(())
    }

    /// Get the agent run ID
    pub fn agent_run_id(&self) -> i64 {
        self.agent_run_id
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
}

/// Cost information (mirrored from murmur-core::agent::output::CostInfo)
#[derive(Debug, Clone)]
pub struct CostInfo {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    async fn setup_test_db() -> (Database, i64) {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!(
            "murmur_test_stream_handler_{}.db",
            uuid::Uuid::new_v4()
        ));

        let db = Database::with_path(db_path.clone()).await.unwrap();
        db.migrate().await.unwrap();

        // Create a test agent run
        let result = sqlx::query(
            r#"
            INSERT INTO agent_runs (agent_type, status, started_at)
            VALUES ('test', 'running', CURRENT_TIMESTAMP)
            "#,
        )
        .execute(db.pool())
        .await
        .unwrap();

        (db, result.last_insert_rowid())
    }

    #[tokio::test]
    async fn test_store_assistant_text() {
        let (db, run_id) = setup_test_db().await;
        let mut handler = DatabaseStreamHandler::new(db.pool(), run_id, false);

        handler.store_assistant_text("Hello, ").await.unwrap();
        handler.store_assistant_text("world!").await.unwrap();
        handler.flush_assistant_message().await.unwrap();

        let repo = ConversationRepository::new(db.pool());
        let messages = repo.get_by_agent_run(run_id).await.unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, world!");
        assert_eq!(messages[0].role, "assistant");
    }

    #[tokio::test]
    async fn test_store_tool_use() {
        let (db, run_id) = setup_test_db().await;
        let mut handler = DatabaseStreamHandler::new(db.pool(), run_id, false);

        let input = json!({"file_path": "/test.txt"});
        handler.store_tool_use("Read", &input).await.unwrap();
        handler.store_assistant_text("Reading...").await.unwrap();
        handler.flush_assistant_message().await.unwrap();

        let repo = ConversationRepository::new(db.pool());
        let messages = repo.get_by_agent_run(run_id).await.unwrap();

        assert_eq!(messages.len(), 1);
        assert!(messages[0].tool_use.is_some());
    }

    #[tokio::test]
    async fn test_store_complete_with_cost() {
        let (db, run_id) = setup_test_db().await;
        let mut handler = DatabaseStreamHandler::new(db.pool(), run_id, false);

        let cost = CostInfo {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: None,
            cache_write_tokens: None,
            cost_usd: Some(0.001),
        };

        handler.store_complete(Some(&cost), Some(1234)).await.unwrap();

        let repo = ConversationRepository::new(db.pool());
        let (input, output) = repo.get_token_usage(run_id).await.unwrap();

        assert_eq!(input, 100);
        assert_eq!(output, 50);
    }
}
