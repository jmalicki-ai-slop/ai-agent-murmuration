//! Workflow resume functionality for interrupted agent sessions
//!
//! This module provides the ability to resume interrupted agent workflows by:
//! - Detecting incomplete/interrupted runs
//! - Reconstructing conversation history from the database
//! - Spawning agents with conversation context to continue work

use murmur_db::{repos::AgentRunRepository, repos::ConversationRepository, Database};
use serde_json::Value;
use std::path::PathBuf;

use crate::{Error, Result};

/// Information about a resumable agent run
#[derive(Debug, Clone)]
pub struct ResumableRun {
    /// The agent run ID
    pub run_id: i64,
    /// Issue number this run was working on
    pub issue_number: Option<i64>,
    /// The original prompt
    pub prompt: String,
    /// Working directory
    pub workdir: PathBuf,
    /// When the run started
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Exit code if the run ended abnormally
    pub exit_code: Option<i32>,
    /// Number of conversation messages logged
    pub message_count: i64,
}

impl ResumableRun {
    /// Check if this run ended with an error
    pub fn had_error(&self) -> bool {
        self.exit_code.map(|c| c != 0).unwrap_or(false)
    }

    /// Check if this run was interrupted (no exit code recorded)
    pub fn was_interrupted(&self) -> bool {
        self.exit_code.is_none()
    }
}

/// Find incomplete or interrupted agent runs for a specific issue
///
/// Returns runs that either:
/// - Have no end_time (still running or interrupted)
/// - Exited with a non-zero code
///
/// Results are ordered by start_time descending (most recent first)
pub fn find_incomplete_runs(db: &Database, issue_number: i64) -> Result<Vec<ResumableRun>> {
    let repo = AgentRunRepository::new(db);
    let conv_repo = ConversationRepository::new(db);

    // Get all runs for this issue
    let runs = repo
        .find_by_issue(issue_number)
        .map_err(|e| Error::Agent(format!("Failed to query agent runs: {}", e)))?;

    let mut incomplete = Vec::new();

    for run in runs {
        // Include runs that are incomplete (no end_time) or failed (non-zero exit)
        let is_incomplete = !run.is_completed() || !run.is_successful();

        if is_incomplete {
            let message_count = conv_repo
                .count_by_agent_run(run.id.unwrap_or(0))
                .map_err(|e| Error::Agent(format!("Failed to count messages: {}", e)))?;

            incomplete.push(ResumableRun {
                run_id: run.id.unwrap_or(0),
                issue_number: run.issue_number,
                prompt: run.prompt,
                workdir: PathBuf::from(run.workdir),
                start_time: run.start_time,
                exit_code: run.exit_code,
                message_count,
            });
        }
    }

    Ok(incomplete)
}

/// Find the most recent incomplete run for an issue
pub fn find_latest_incomplete_run(
    db: &Database,
    issue_number: i64,
) -> Result<Option<ResumableRun>> {
    let runs = find_incomplete_runs(db, issue_number)?;
    Ok(runs.into_iter().next())
}

/// Conversation message for resume
#[derive(Debug, Clone)]
pub struct ConversationMessage {
    /// Message sequence number
    pub sequence: i64,
    /// Message type (system, user, assistant, tool_use, tool_result, result)
    pub message_type: String,
    /// The full message as JSON
    pub message_json: Value,
}

/// Reconstruct conversation history from database for a given agent run
///
/// Returns messages in chronological order (by sequence number)
pub fn reconstruct_conversation(db: &Database, run_id: i64) -> Result<Vec<ConversationMessage>> {
    let repo = ConversationRepository::new(db);

    let logs = repo
        .find_by_agent_run(run_id)
        .map_err(|e| Error::Agent(format!("Failed to load conversation logs: {}", e)))?;

    let messages = logs
        .into_iter()
        .map(|log| {
            let message_json = serde_json::from_str(&log.message_json).map_err(|e| {
                Error::Agent(format!("Failed to parse conversation message JSON: {}", e))
            })?;

            Ok(ConversationMessage {
                sequence: log.sequence,
                message_type: log.message_type,
                message_json,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(messages)
}

/// Build a resume prompt that includes conversation history
///
/// This creates a prompt that:
/// 1. Explains that this is a resumed session
/// 2. Includes a summary of what was attempted
/// 3. Asks the agent to continue where it left off
pub fn build_resume_prompt(
    original_prompt: &str,
    messages: &[ConversationMessage],
    reason: &str,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("RESUMING INTERRUPTED SESSION\n\n");
    prompt.push_str(&format!("Reason for resume: {}\n\n", reason));
    prompt.push_str("Original task:\n");
    prompt.push_str(original_prompt);
    prompt.push_str("\n\n");

    // Add conversation summary
    if !messages.is_empty() {
        prompt.push_str(&format!(
            "Previous session had {} messages. ",
            messages.len()
        ));

        // Count message types for summary
        let mut assistant_msgs = 0;
        let mut tool_uses = 0;

        for msg in messages {
            match msg.message_type.as_str() {
                "assistant" => assistant_msgs += 1,
                "tool_use" => tool_uses += 1,
                _ => {}
            }
        }

        if assistant_msgs > 0 {
            prompt.push_str(&format!("Assistant sent {} messages. ", assistant_msgs));
        }
        if tool_uses > 0 {
            prompt.push_str(&format!("Used {} tools. ", tool_uses));
        }

        prompt.push_str("\n\n");
    }

    prompt.push_str("Please review what was done in the previous session and continue the work. ");
    prompt.push_str("Check the current state of the files and complete any remaining tasks.\n");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use murmur_db::models::{AgentRun, ConversationLog};

    fn setup_test_db() -> Database {
        Database::in_memory().unwrap()
    }

    #[test]
    fn test_find_incomplete_runs_empty() {
        let db = setup_test_db();
        let runs = find_incomplete_runs(&db, 42).unwrap();
        assert_eq!(runs.len(), 0);
    }

    #[test]
    fn test_find_incomplete_runs_with_incomplete() {
        let db = setup_test_db();
        let agent_repo = AgentRunRepository::new(&db);

        // Create an incomplete run (no end_time)
        let mut run = AgentRun::new("implementer", "Fix bug", "/tmp/work", "{}");
        run.issue_number = Some(42);
        let run_id = agent_repo.insert(&run).unwrap();

        let incomplete = find_incomplete_runs(&db, 42).unwrap();
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].run_id, run_id);
        assert!(incomplete[0].was_interrupted());
    }

    #[test]
    fn test_find_incomplete_runs_with_failed() {
        let db = setup_test_db();
        let agent_repo = AgentRunRepository::new(&db);

        // Create a failed run (exit code 1)
        let mut run = AgentRun::new("implementer", "Fix bug", "/tmp/work", "{}");
        run.issue_number = Some(42);
        run.complete(1);
        let run_id = agent_repo.insert(&run).unwrap();

        let incomplete = find_incomplete_runs(&db, 42).unwrap();
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].run_id, run_id);
        assert!(incomplete[0].had_error());
    }

    #[test]
    fn test_find_incomplete_runs_excludes_successful() {
        let db = setup_test_db();
        let agent_repo = AgentRunRepository::new(&db);

        // Create a successful run
        let mut run = AgentRun::new("implementer", "Fix bug", "/tmp/work", "{}");
        run.issue_number = Some(42);
        run.complete(0);
        agent_repo.insert(&run).unwrap();

        let incomplete = find_incomplete_runs(&db, 42).unwrap();
        assert_eq!(incomplete.len(), 0);
    }

    #[test]
    fn test_find_latest_incomplete_run() {
        let db = setup_test_db();
        let agent_repo = AgentRunRepository::new(&db);

        // Create two incomplete runs
        let mut run1 = AgentRun::new("implementer", "First attempt", "/tmp/1", "{}");
        run1.issue_number = Some(42);
        run1.start_time = Utc::now() - chrono::Duration::hours(2);
        agent_repo.insert(&run1).unwrap();

        let mut run2 = AgentRun::new("implementer", "Second attempt", "/tmp/2", "{}");
        run2.issue_number = Some(42);
        run2.start_time = Utc::now() - chrono::Duration::hours(1);
        let run2_id = agent_repo.insert(&run2).unwrap();

        let latest = find_latest_incomplete_run(&db, 42).unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().run_id, run2_id);
    }

    #[test]
    fn test_reconstruct_conversation() {
        let db = setup_test_db();
        let agent_repo = AgentRunRepository::new(&db);
        let conv_repo = ConversationRepository::new(&db);

        let run = AgentRun::new("implementer", "Task", "/tmp", "{}");
        let run_id = agent_repo.insert(&run).unwrap();

        // Add conversation logs
        conv_repo
            .insert(&ConversationLog::new(
                run_id,
                0,
                "system",
                r#"{"type":"system","subtype":"init"}"#,
            ))
            .unwrap();
        conv_repo
            .insert(&ConversationLog::new(
                run_id,
                1,
                "assistant",
                r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello"}]}}"#,
            ))
            .unwrap();

        let messages = reconstruct_conversation(&db, run_id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].sequence, 0);
        assert_eq!(messages[0].message_type, "system");
        assert_eq!(messages[1].sequence, 1);
        assert_eq!(messages[1].message_type, "assistant");
    }

    #[test]
    fn test_build_resume_prompt() {
        let original = "Fix the authentication bug in issue #42";
        let messages = vec![
            ConversationMessage {
                sequence: 0,
                message_type: "system".to_string(),
                message_json: serde_json::json!({"type": "system"}),
            },
            ConversationMessage {
                sequence: 1,
                message_type: "assistant".to_string(),
                message_json: serde_json::json!({"type": "assistant"}),
            },
            ConversationMessage {
                sequence: 2,
                message_type: "tool_use".to_string(),
                message_json: serde_json::json!({"type": "tool_use"}),
            },
        ];

        let prompt = build_resume_prompt(original, &messages, "Session was interrupted");

        assert!(prompt.contains("RESUMING INTERRUPTED SESSION"));
        assert!(prompt.contains("Session was interrupted"));
        assert!(prompt.contains(original));
        assert!(prompt.contains("3 messages"));
        assert!(prompt.contains("Assistant sent 1 messages"));
        assert!(prompt.contains("Used 1 tools"));
    }

    #[test]
    fn test_build_resume_prompt_empty_history() {
        let original = "Fix bug";
        let messages = vec![];

        let prompt = build_resume_prompt(original, &messages, "Timeout");

        assert!(prompt.contains("RESUMING INTERRUPTED SESSION"));
        assert!(prompt.contains("Timeout"));
        assert!(prompt.contains(original));
    }
}
