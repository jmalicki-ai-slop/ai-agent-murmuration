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
}
