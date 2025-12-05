//! Data models for database records

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Agent run record tracking execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    /// Unique identifier for this run
    pub id: Option<i64>,

    /// Type of agent (e.g., "planner", "implementer", "reviewer")
    pub agent_type: String,

    /// GitHub issue number if this run is associated with an issue
    pub issue_number: Option<i64>,

    /// The prompt given to the agent
    pub prompt: String,

    /// Working directory for the agent
    pub workdir: String,

    /// Agent configuration as JSON
    pub config_json: String,

    /// Process ID of the running agent (None if not tracked or completed)
    pub pid: Option<i32>,

    /// When the agent started
    pub start_time: DateTime<Utc>,

    /// When the agent finished (None if still running)
    pub end_time: Option<DateTime<Utc>>,

    /// Exit code from the agent process (None if still running)
    pub exit_code: Option<i32>,

    /// Duration in seconds (computed from start_time and end_time)
    pub duration_seconds: Option<f64>,

    /// When this record was created
    pub created_at: DateTime<Utc>,
}

impl AgentRun {
    /// Create a new agent run record
    pub fn new(
        agent_type: impl Into<String>,
        prompt: impl Into<String>,
        workdir: impl Into<String>,
        config_json: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            agent_type: agent_type.into(),
            issue_number: None,
            prompt: prompt.into(),
            workdir: workdir.into(),
            config_json: config_json.into(),
            pid: None,
            start_time: now,
            end_time: None,
            exit_code: None,
            duration_seconds: None,
            created_at: now,
        }
    }

    /// Set the issue number for this run
    pub fn with_issue_number(mut self, issue_number: i64) -> Self {
        self.issue_number = Some(issue_number);
        self
    }

    /// Set the process ID for this run
    pub fn with_pid(mut self, pid: i32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Mark the run as completed
    pub fn complete(&mut self, exit_code: i32) {
        let now = Utc::now();
        self.end_time = Some(now);
        self.exit_code = Some(exit_code);
        self.duration_seconds = Some((now - self.start_time).num_milliseconds() as f64 / 1000.0);
    }

    /// Check if the run is completed
    pub fn is_completed(&self) -> bool {
        self.end_time.is_some()
    }

    /// Check if the run was successful (exit code 0)
    pub fn is_successful(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Conversation log entry storing JSON output from agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationLog {
    /// Unique identifier for this log entry
    pub id: Option<i64>,

    /// The agent run this conversation belongs to
    pub agent_run_id: i64,

    /// Sequence number for ordering messages in the conversation
    pub sequence: i64,

    /// Timestamp when this message was received
    pub timestamp: DateTime<Utc>,

    /// The type of message (system, user, assistant, tool_use, tool_result, result)
    pub message_type: String,

    /// The full JSON message as received from Claude Code
    pub message_json: String,

    /// When this record was created
    pub created_at: DateTime<Utc>,
}

impl ConversationLog {
    /// Create a new conversation log entry
    pub fn new(
        agent_run_id: i64,
        sequence: i64,
        message_type: impl Into<String>,
        message_json: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            agent_run_id,
            sequence,
            timestamp: now,
            message_type: message_type.into(),
            message_json: message_json.into(),
            created_at: now,
        }
    }

    /// Create a new entry with a custom timestamp
    pub fn with_timestamp(
        agent_run_id: i64,
        sequence: i64,
        message_type: impl Into<String>,
        message_json: impl Into<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            agent_run_id,
            sequence,
            timestamp,
            message_type: message_type.into(),
            message_json: message_json.into(),
            created_at: Utc::now(),
        }
    }

    /// Parse the JSON message into a structured type
    pub fn parse_message<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_str(&self.message_json)
    }
}

/// Worktree tracking record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeRecord {
    /// Unique identifier for this worktree
    pub id: Option<i64>,

    /// Full path to the worktree directory
    pub path: String,

    /// Branch name
    pub branch_name: String,

    /// GitHub issue number if associated with an issue
    pub issue_number: Option<i64>,

    /// Agent run ID if associated with a run
    pub agent_run_id: Option<i64>,

    /// Path to the main repository (for finding git repo when worktree is cached)
    pub main_repo_path: Option<String>,

    /// Status: active, completed, abandoned, stale
    pub status: String,

    /// When the worktree was created
    pub created_at: DateTime<Utc>,

    /// When the worktree was last updated
    pub updated_at: DateTime<Utc>,
}

impl WorktreeRecord {
    /// Create a new worktree record
    pub fn new(path: impl Into<String>, branch_name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            path: path.into(),
            branch_name: branch_name.into(),
            issue_number: None,
            agent_run_id: None,
            main_repo_path: None,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the issue number for this worktree
    pub fn with_issue_number(mut self, issue_number: i64) -> Self {
        self.issue_number = Some(issue_number);
        self
    }

    /// Set the agent run ID for this worktree
    pub fn with_agent_run_id(mut self, agent_run_id: i64) -> Self {
        self.agent_run_id = Some(agent_run_id);
        self
    }

    /// Set the main repository path for this worktree
    pub fn with_main_repo_path(mut self, main_repo_path: impl Into<String>) -> Self {
        self.main_repo_path = Some(main_repo_path.into());
        self
    }

    /// Mark the worktree as completed
    pub fn mark_completed(&mut self) {
        self.status = "completed".to_string();
        self.updated_at = Utc::now();
    }

    /// Mark the worktree as abandoned
    pub fn mark_abandoned(&mut self) {
        self.status = "abandoned".to_string();
        self.updated_at = Utc::now();
    }

    /// Mark the worktree as stale (directory missing or agent not running)
    pub fn mark_stale(&mut self) {
        self.status = "stale".to_string();
        self.updated_at = Utc::now();
    }

    /// Check if the worktree is active
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_run_new() {
        let run = AgentRun::new("implementer", "Fix bug #123", "/tmp/workdir", "{}");
        assert_eq!(run.agent_type, "implementer");
        assert_eq!(run.prompt, "Fix bug #123");
        assert_eq!(run.workdir, "/tmp/workdir");
        assert_eq!(run.config_json, "{}");
        assert!(run.id.is_none());
        assert!(!run.is_completed());
    }

    #[test]
    fn test_agent_run_with_issue_number() {
        let run = AgentRun::new("planner", "Plan feature", "/tmp", "{}").with_issue_number(42);
        assert_eq!(run.issue_number, Some(42));
    }

    #[test]
    fn test_agent_run_complete() {
        let mut run = AgentRun::new("reviewer", "Review code", "/tmp", "{}");
        assert!(!run.is_completed());
        assert!(!run.is_successful());

        run.complete(0);
        assert!(run.is_completed());
        assert!(run.is_successful());
        assert!(run.duration_seconds.is_some());
        assert!(run.duration_seconds.unwrap() >= 0.0);
    }

    #[test]
    fn test_agent_run_complete_with_error() {
        let mut run = AgentRun::new("implementer", "Run task", "/tmp", "{}");
        run.complete(1);

        assert!(run.is_completed());
        assert!(!run.is_successful());
        assert_eq!(run.exit_code, Some(1));
    }

    #[test]
    fn test_conversation_log_new() {
        let log = ConversationLog::new(
            123,
            0,
            "assistant",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello"}]}}"#,
        );
        assert_eq!(log.agent_run_id, 123);
        assert_eq!(log.sequence, 0);
        assert_eq!(log.message_type, "assistant");
        assert!(log.id.is_none());
    }

    #[test]
    fn test_conversation_log_with_timestamp() {
        let timestamp = Utc::now();
        let log = ConversationLog::with_timestamp(
            456,
            1,
            "tool_use",
            r#"{"type":"tool_use","tool":"Read"}"#,
            timestamp,
        );
        assert_eq!(log.agent_run_id, 456);
        assert_eq!(log.sequence, 1);
        assert_eq!(log.timestamp, timestamp);
    }

    #[test]
    fn test_conversation_log_parse_message() {
        use serde_json::Value;

        let json = r#"{"type":"assistant","message":{"content":[]}}"#;
        let log = ConversationLog::new(1, 0, "assistant", json);

        let parsed: Value = log.parse_message().unwrap();
        assert_eq!(parsed["type"], "assistant");
    }
}
