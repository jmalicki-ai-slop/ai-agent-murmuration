//! Conversation logger that implements StreamHandler to persist conversations to database

use crate::models::ConversationLog;
use crate::repos::ConversationRepository;
use crate::{Database, Result};

/// A StreamHandler implementation that logs conversation messages to the database
///
/// This handler can be used to stream agent output directly to the database
/// for debugging and analysis purposes.
pub struct ConversationLogger<'db> {
    pub agent_run_id: i64,
    pub sequence: i64,
    pub repo: ConversationRepository<'db>,
}

impl<'db> ConversationLogger<'db> {
    /// Create a new conversation logger for an agent run
    pub fn new(db: &'db Database, agent_run_id: i64) -> Result<Self> {
        let repo = ConversationRepository::new(db);
        let sequence = repo.next_sequence(agent_run_id)?;

        Ok(Self {
            agent_run_id,
            sequence,
            repo,
        })
    }

    /// Log a message with the given type and JSON content
    pub fn log_message(&mut self, message_type: &str, message_json: &str) -> Result<()> {
        let log =
            ConversationLog::new(self.agent_run_id, self.sequence, message_type, message_json);

        self.repo.insert(&log)?;
        self.sequence += 1;
        Ok(())
    }

    /// Get the current sequence number (number of messages logged)
    pub fn message_count(&self) -> i64 {
        self.sequence
    }
}

// Note: We can't implement murmur_core::agent::output::StreamHandler here because
// murmur-db doesn't depend on murmur-core. Instead, we'll create a wrapper in murmur-core
// that uses ConversationLogger.

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
    fn test_conversation_logger_new() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);

        let logger = ConversationLogger::new(&db, agent_run_id).unwrap();
        assert_eq!(logger.agent_run_id, agent_run_id);
        assert_eq!(logger.sequence, 0);
    }

    #[test]
    fn test_log_message() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let mut logger = ConversationLogger::new(&db, agent_run_id).unwrap();

        logger
            .log_message("system", r#"{"type":"system","subtype":"init"}"#)
            .unwrap();
        assert_eq!(logger.sequence, 1);

        logger
            .log_message(
                "assistant",
                r#"{"type":"assistant","message":{"content":[]}}"#,
            )
            .unwrap();
        assert_eq!(logger.sequence, 2);

        // Verify messages were logged
        let repo = ConversationRepository::new(&db);
        let logs = repo.find_by_agent_run(agent_run_id).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message_type, "system");
        assert_eq!(logs[1].message_type, "assistant");
    }

    #[test]
    fn test_message_count() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);
        let mut logger = ConversationLogger::new(&db, agent_run_id).unwrap();

        assert_eq!(logger.message_count(), 0);

        logger
            .log_message("system", r#"{"type":"system"}"#)
            .unwrap();
        assert_eq!(logger.message_count(), 1);

        logger
            .log_message("assistant", r#"{"type":"assistant"}"#)
            .unwrap();
        assert_eq!(logger.message_count(), 2);
    }

    #[test]
    fn test_resume_logging() {
        let db = setup_db();
        let agent_run_id = create_test_agent_run(&db);

        // First logger session
        {
            let mut logger = ConversationLogger::new(&db, agent_run_id).unwrap();
            logger
                .log_message("system", r#"{"type":"system"}"#)
                .unwrap();
            logger
                .log_message("assistant", r#"{"type":"assistant"}"#)
                .unwrap();
        }

        // Second logger session (resume)
        {
            let mut logger = ConversationLogger::new(&db, agent_run_id).unwrap();
            // Should start at sequence 2
            assert_eq!(logger.sequence, 2);

            logger
                .log_message("tool_use", r#"{"type":"tool_use"}"#)
                .unwrap();
        }

        // Verify all messages
        let repo = ConversationRepository::new(&db);
        let logs = repo.find_by_agent_run(agent_run_id).unwrap();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[2].sequence, 2);
    }
}
