//! Database schema types and models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub issue stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub assignee: Option<String>,
    pub labels: String, // JSON array
    pub phase: Option<i64>,
    pub pr_number: Option<String>,
    pub parent_issue: Option<i64>,
    pub metadata: Option<String>, // JSON object
}

/// Agent run record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentRun {
    pub id: i64,
    pub issue_id: Option<i64>,
    pub agent_type: String,
    pub status: String,
    pub worktree_path: Option<String>,
    pub branch_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i64>,
    pub error_message: Option<String>,
}

/// Conversation message in an agent run
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationMessage {
    pub id: i64,
    pub agent_run_id: i64,
    pub role: String,
    pub content: String,
    pub tool_use: Option<String>, // JSON object
    pub tool_result: Option<String>, // JSON object
    pub timestamp: DateTime<Utc>,
    pub tokens_input: Option<i64>,
    pub tokens_output: Option<i64>,
    pub cost_usd: Option<f64>,
}

/// Issue status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueStatus {
    Open,
    Closed,
}

impl IssueStatus {
    pub fn as_str(&self) -> &str {
        match self {
            IssueStatus::Open => "open",
            IssueStatus::Closed => "closed",
        }
    }
}

/// Agent run status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Completed,
    Failed,
    Interrupted,
}

impl AgentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AgentStatus::Running => "running",
            AgentStatus::Completed => "completed",
            AgentStatus::Failed => "failed",
            AgentStatus::Interrupted => "interrupted",
        }
    }
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn as_str(&self) -> &str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }
}
