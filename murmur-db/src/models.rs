//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub issue state in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueState {
    pub id: i64,
    pub github_issue_number: i64,
    pub repository: String,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub pr_number: Option<String>,
    pub labels: Option<String>,
    pub assignees: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Input for creating a new issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssue {
    pub github_issue_number: i64,
    pub repository: String,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub pr_number: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
}

/// Input for updating an existing issue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateIssue {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub pr_number: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Issue status change history entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueStatusHistory {
    pub id: i64,
    pub issue_id: i64,
    pub old_status: Option<String>,
    pub new_status: String,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

/// Agent run record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentRun {
    pub id: i64,
    pub issue_id: Option<i64>,
    pub agent_type: String,
    pub worktree_path: Option<String>,
    pub command: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub config: Option<String>,
}

/// Input for creating a new agent run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRun {
    pub issue_id: Option<i64>,
    pub agent_type: String,
    pub worktree_path: Option<String>,
    pub command: String,
    pub config: Option<serde_json::Value>,
}

/// Input for updating an agent run (typically on completion)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAgentRun {
    pub end_time: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
}

/// Conversation log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationLog {
    pub id: i64,
    pub agent_run_id: i64,
    pub sequence_number: i64,
    pub timestamp: DateTime<Utc>,
    pub message_type: String,
    pub content: String,
}

/// Input for creating a conversation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationLog {
    pub agent_run_id: i64,
    pub sequence_number: i64,
    pub message_type: String,
    pub content: serde_json::Value,
}
