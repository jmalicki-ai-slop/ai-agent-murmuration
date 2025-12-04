# Code Structure Design

## Overview

This document defines the internal Rust code organization within each crate, including traits, error handling patterns, and async conventions.

---

## Cross-Cutting Patterns

### Error Handling

All crates use a consistent error pattern based on `thiserror`:

```rust
// dispatch-core/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // Git errors
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    // GitHub API errors
    #[error("GitHub API error: {0}")]
    GitHub(String),

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    // Not found
    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    // State transition errors
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    // Agent errors
    #[error("Agent error: {0}")]
    Agent(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // Generic wrapped errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, DispatchError>;
```

### Async Conventions

```rust
// All async functions use tokio
// Prefer async traits via async-trait crate where needed

#[async_trait::async_trait]
pub trait Repository<T, Id> {
    async fn get(&self, id: &Id) -> Result<Option<T>>;
    async fn create(&self, item: &T) -> Result<Id>;
    async fn update(&self, item: &T) -> Result<()>;
    async fn delete(&self, id: &Id) -> Result<()>;
}

// Use tokio::spawn for background tasks
// Use tokio::select! for concurrent operations
// Use tokio::sync::mpsc for channels
```

### ID Types

All entity IDs are strongly typed wrappers:

```rust
// dispatch-core/src/types/ids.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

define_id!(IssueId);
define_id!(EpicId);
define_id!(StageId);
define_id!(GateId);
define_id!(AgentId);
define_id!(ProposalId);
define_id!(VoteId);
define_id!(DecisionId);
define_id!(PullRequestId);
```

---

## dispatch-core

### Module Structure

```
dispatch-core/src/
├── lib.rs
├── error.rs              # DispatchError, Result type alias
├── config.rs             # Configuration structs
├── events.rs             # Event enum for pub/sub
├── types/
│   ├── mod.rs
│   ├── ids.rs            # Strongly typed ID wrappers
│   ├── issue.rs          # Issue, IssueStatus, IssueType, Priority
│   ├── epic.rs           # Epic, Stage, Gate, GateType
│   ├── agent.rs          # Agent, AgentType, AgentStatus
│   ├── proposal.rs       # Proposal, Vote, ConsensusThreshold
│   ├── pr.rs             # PullRequest, PRStatus, ReviewStatus
│   └── decision.rs       # Decision, DecisionType
└── traits/
    ├── mod.rs
    └── repository.rs     # Repository trait
```

### Key Types

```rust
// dispatch-core/src/types/issue.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::ids::{AgentId, EpicId, IssueId, PullRequestId, StageId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Unassigned,
    Queued,
    Assigned,
    InProgress,
    AwaitingReview,
    InReview,
    Done,
    Blocked,
    Cancelled,
}

impl IssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unassigned => "unassigned",
            Self::Queued => "queued",
            Self::Assigned => "assigned",
            Self::InProgress => "in_progress",
            Self::AwaitingReview => "awaiting_review",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Cancelled => "cancelled",
        }
    }

    /// Valid transitions from this status
    pub fn valid_transitions(&self) -> &[IssueStatus] {
        use IssueStatus::*;
        match self {
            Unassigned => &[Queued, Assigned, Cancelled],
            Queued => &[Assigned, Unassigned, Cancelled],
            Assigned => &[InProgress, Unassigned, Cancelled],
            InProgress => &[AwaitingReview, Blocked, Done, Cancelled],
            AwaitingReview => &[InReview, InProgress, Cancelled],
            InReview => &[Done, InProgress, Cancelled],
            Done => &[],  // Terminal state
            Blocked => &[InProgress, Cancelled],
            Cancelled => &[],  // Terminal state
        }
    }

    pub fn can_transition_to(&self, target: IssueStatus) -> bool {
        self.valid_transitions().contains(&target)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Feature,
    Bug,
    Docs,
    Refactor,
    Test,
    Security,
    Chore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: IssueId,
    pub github_id: Option<u64>,
    pub github_url: Option<String>,

    // Epic relationship
    pub epic_id: Option<EpicId>,
    pub stage_id: Option<StageId>,

    // Repository
    pub repo_path: PathBuf,
    pub repo_url: Option<String>,
    pub worktree_path: Option<PathBuf>,
    pub branch_name: Option<String>,

    // Content
    pub title: String,
    pub prompt: String,  // Full description, serves as agent memory
    pub issue_type: IssueType,
    pub priority: Priority,
    pub labels: Vec<String>,

    // Assignment
    pub status: IssueStatus,
    pub assigned_agent_id: Option<AgentId>,
    pub agent_type: Option<AgentType>,

    // Linked PR
    pub linked_pr_id: Option<PullRequestId>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Issue {
    pub fn new(
        repo_path: PathBuf,
        title: String,
        prompt: String,
        issue_type: IssueType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: IssueId::new(),
            github_id: None,
            github_url: None,
            epic_id: None,
            stage_id: None,
            repo_path,
            repo_url: None,
            worktree_path: None,
            branch_name: None,
            title,
            prompt,
            issue_type,
            priority: Priority::Medium,
            labels: Vec::new(),
            status: IssueStatus::Unassigned,
            assigned_agent_id: None,
            agent_type: None,
            linked_pr_id: None,
            created_at: now,
            updated_at: now,
            assigned_at: None,
            completed_at: None,
        }
    }

    pub fn transition_to(&mut self, status: IssueStatus) -> Result<(), DispatchError> {
        if !self.status.can_transition_to(status) {
            return Err(DispatchError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: status.as_str().to_string(),
            });
        }
        self.status = status;
        self.updated_at = Utc::now();
        Ok(())
    }
}
```

```rust
// dispatch-core/src/types/epic.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::ids::{EpicId, GateId, StageId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpicStatus {
    Draft,
    Ready,
    InProgress,
    AwaitingGate,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    InProgress,
    AwaitingGate,
    Approved,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    Approval,     // Human must approve
    Review,       // Human reviews deliverables
    Checkpoint,   // Status check, auto-approve if criteria met
    Decision,     // Human chooses direction
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateApproval {
    Pending,
    Approved,
    Rejected,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epic {
    pub id: EpicId,
    pub github_id: Option<u64>,
    pub github_url: Option<String>,

    // Content
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,

    // Repository
    pub repo_path: PathBuf,
    pub repo_url: Option<String>,

    // Status
    pub status: EpicStatus,
    pub current_stage_id: Option<StageId>,
    pub blocked_at_gate_id: Option<GateId>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub id: StageId,
    pub epic_id: EpicId,

    pub name: String,
    pub description: Option<String>,
    pub order: u32,

    pub status: StageStatus,

    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub id: GateId,
    pub stage_id: StageId,

    pub gate_type: GateType,
    pub description: String,
    pub required_approvers: GateApprovers,

    pub status: GateApproval,
    pub approved_by: Option<String>,
    pub rejection_reason: Option<String>,

    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GateApprovers {
    Any,
    Specific(Vec<String>),
}
```

```rust
// dispatch-core/src/types/agent.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::ids::{AgentId, IssueId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Coder,
    Reviewer,
    Pm,
    Security,
    Docs,
    Test,
    Architect,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Coder => "coder",
            Self::Reviewer => "reviewer",
            Self::Pm => "pm",
            Self::Security => "security",
            Self::Docs => "docs",
            Self::Test => "test",
            Self::Architect => "architect",
        }
    }

    pub fn prompt_file(&self) -> &'static str {
        match self {
            Self::Coder => "prompts/coder.md",
            Self::Reviewer => "prompts/reviewer.md",
            Self::Pm => "prompts/pm.md",
            Self::Security => "prompts/security.md",
            Self::Docs => "prompts/docs.md",
            Self::Test => "prompts/test.md",
            Self::Architect => "prompts/architect.md",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Starting,
    Working,
    WaitingForInput,
    WaitingForVote,
    Paused,
    Errored,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub issues_completed: u32,
    pub avg_completion_time_secs: Option<f64>,
    pub tokens_used: u64,
    pub errors_encountered: u32,
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            issues_completed: 0,
            avg_completion_time_secs: None,
            tokens_used: 0,
            errors_encountered: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,

    pub agent_type: AgentType,
    pub status: AgentStatus,

    // Current work
    pub current_issue_id: Option<IssueId>,
    pub worktree_path: Option<PathBuf>,
    pub process_id: Option<u32>,
    pub claude_session_id: Option<String>,

    // Timestamps
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    // Metrics
    pub metrics: AgentMetrics,
}

impl Agent {
    pub fn new(agent_type: AgentType) -> Self {
        let now = Utc::now();
        Self {
            id: AgentId::new(),
            agent_type,
            status: AgentStatus::Idle,
            current_issue_id: None,
            worktree_path: None,
            process_id: None,
            claude_session_id: None,
            started_at: now,
            last_heartbeat: now,
            completed_at: None,
            metrics: AgentMetrics::default(),
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status, AgentStatus::Idle | AgentStatus::Completed)
    }

    pub fn is_working(&self) -> bool {
        matches!(
            self.status,
            AgentStatus::Starting | AgentStatus::Working | AgentStatus::WaitingForInput | AgentStatus::WaitingForVote
        )
    }
}
```

### Event System

```rust
// dispatch-core/src/events.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::ids::*;
use crate::types::{AgentStatus, GateApproval, IssueStatus, EpicStatus};

/// Events emitted by the system for pub/sub
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DispatchEvent {
    // Issue events
    IssueCreated { issue_id: IssueId },
    IssueUpdated { issue_id: IssueId },
    IssueStatusChanged { issue_id: IssueId, from: IssueStatus, to: IssueStatus },
    IssueAssigned { issue_id: IssueId, agent_id: AgentId },

    // Epic events
    EpicCreated { epic_id: EpicId },
    EpicStatusChanged { epic_id: EpicId, from: EpicStatus, to: EpicStatus },
    StageStarted { epic_id: EpicId, stage_id: StageId },
    StageCompleted { epic_id: EpicId, stage_id: StageId },

    // Gate events
    GateReached { epic_id: EpicId, gate_id: GateId },
    GateApproved { gate_id: GateId, approved_by: String },
    GateRejected { gate_id: GateId, reason: String },
    GateSkipped { gate_id: GateId, skipped_by: String },

    // Agent events
    AgentStarted { agent_id: AgentId },
    AgentStatusChanged { agent_id: AgentId, from: AgentStatus, to: AgentStatus },
    AgentCompleted { agent_id: AgentId, issue_id: IssueId },
    AgentErrored { agent_id: AgentId, error: String },
    AgentHeartbeat { agent_id: AgentId },

    // Proposal events
    ProposalCreated { proposal_id: ProposalId },
    VoteCast { proposal_id: ProposalId, vote_id: VoteId },
    ProposalApproved { proposal_id: ProposalId },
    ProposalRejected { proposal_id: ProposalId },
    ProposalVetoed { proposal_id: ProposalId, vetoed_by: String },

    // PR events
    PrCreated { issue_id: IssueId, pr_number: u64 },
    PrMerged { issue_id: IssueId, pr_number: u64 },
    PrClosed { issue_id: IssueId, pr_number: u64 },

    // Sync events
    GitHubSyncStarted,
    GitHubSyncCompleted { issues_synced: u32, prs_synced: u32 },
    GitHubSyncFailed { error: String },
}

impl DispatchEvent {
    pub fn timestamp(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
```

---

## dispatch-db

### Module Structure

```
dispatch-db/src/
├── lib.rs
├── pool.rs               # Connection pool setup
├── migrations.rs         # Migration runner
├── repos/
│   ├── mod.rs
│   ├── issue.rs          # IssueRepository
│   ├── epic.rs           # EpicRepository + StageRepository + GateRepository
│   ├── agent.rs          # AgentRepository
│   ├── proposal.rs       # ProposalRepository + VoteRepository
│   ├── decision.rs       # DecisionRepository
│   ├── pr.rs             # PullRequestRepository
│   └── log.rs            # AgentLogRepository
└── row_types.rs          # Database row structs for sqlx
```

### Repository Pattern

```rust
// dispatch-db/src/repos/issue.rs

use async_trait::async_trait;
use sqlx::SqlitePool;

use dispatch_core::error::Result;
use dispatch_core::types::ids::IssueId;
use dispatch_core::types::issue::{Issue, IssueStatus, Priority};

use crate::row_types::IssueRow;

pub struct IssueRepository {
    pool: SqlitePool,
}

impl IssueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: &IssueId) -> Result<Option<Issue>> {
        let row = sqlx::query_as!(
            IssueRow,
            r#"
            SELECT
                id, github_id, github_url,
                epic_id, stage_id,
                repo_path, repo_url, worktree_path, branch_name,
                title, prompt, issue_type, priority, labels,
                status, assigned_agent_id, agent_type,
                created_at, updated_at, assigned_at, completed_at
            FROM issues
            WHERE id = ?
            "#,
            id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Issue::from))
    }

    pub async fn create(&self, issue: &Issue) -> Result<()> {
        let labels_json = serde_json::to_string(&issue.labels)?;

        sqlx::query!(
            r#"
            INSERT INTO issues (
                id, github_id, github_url,
                epic_id, stage_id,
                repo_path, repo_url, worktree_path, branch_name,
                title, prompt, issue_type, priority, labels,
                status, assigned_agent_id, agent_type,
                created_at, updated_at, assigned_at, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            issue.id.to_string(),
            issue.github_id.map(|id| id as i64),
            issue.github_url,
            issue.epic_id.as_ref().map(|id| id.to_string()),
            issue.stage_id.as_ref().map(|id| id.to_string()),
            issue.repo_path.to_string_lossy(),
            issue.repo_url,
            issue.worktree_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            issue.branch_name,
            issue.title,
            issue.prompt,
            issue.issue_type.as_str(),
            issue.priority.as_str(),
            labels_json,
            issue.status.as_str(),
            issue.assigned_agent_id.as_ref().map(|id| id.to_string()),
            issue.agent_type.as_ref().map(|t| t.as_str()),
            issue.created_at.to_rfc3339(),
            issue.updated_at.to_rfc3339(),
            issue.assigned_at.map(|t| t.to_rfc3339()),
            issue.completed_at.map(|t| t.to_rfc3339()),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, issue: &Issue) -> Result<()> {
        let labels_json = serde_json::to_string(&issue.labels)?;

        sqlx::query!(
            r#"
            UPDATE issues SET
                github_id = ?,
                github_url = ?,
                epic_id = ?,
                stage_id = ?,
                repo_path = ?,
                repo_url = ?,
                worktree_path = ?,
                branch_name = ?,
                title = ?,
                prompt = ?,
                issue_type = ?,
                priority = ?,
                labels = ?,
                status = ?,
                assigned_agent_id = ?,
                agent_type = ?,
                updated_at = ?,
                assigned_at = ?,
                completed_at = ?
            WHERE id = ?
            "#,
            issue.github_id.map(|id| id as i64),
            issue.github_url,
            issue.epic_id.as_ref().map(|id| id.to_string()),
            issue.stage_id.as_ref().map(|id| id.to_string()),
            issue.repo_path.to_string_lossy(),
            issue.repo_url,
            issue.worktree_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            issue.branch_name,
            issue.title,
            issue.prompt,
            issue.issue_type.as_str(),
            issue.priority.as_str(),
            labels_json,
            issue.status.as_str(),
            issue.assigned_agent_id.as_ref().map(|id| id.to_string()),
            issue.agent_type.as_ref().map(|t| t.as_str()),
            issue.updated_at.to_rfc3339(),
            issue.assigned_at.map(|t| t.to_rfc3339()),
            issue.completed_at.map(|t| t.to_rfc3339()),
            issue.id.to_string(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &IssueId) -> Result<()> {
        sqlx::query!("DELETE FROM issues WHERE id = ?", id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Query methods

    pub async fn list_by_status(&self, status: IssueStatus) -> Result<Vec<Issue>> {
        let rows = sqlx::query_as!(
            IssueRow,
            r#"
            SELECT
                id, github_id, github_url,
                epic_id, stage_id,
                repo_path, repo_url, worktree_path, branch_name,
                title, prompt, issue_type, priority, labels,
                status, assigned_agent_id, agent_type,
                created_at, updated_at, assigned_at, completed_at
            FROM issues
            WHERE status = ?
            ORDER BY
                CASE priority
                    WHEN 'critical' THEN 0
                    WHEN 'high' THEN 1
                    WHEN 'medium' THEN 2
                    WHEN 'low' THEN 3
                END,
                created_at ASC
            "#,
            status.as_str()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Issue::from).collect())
    }

    pub async fn list_unassigned(&self) -> Result<Vec<Issue>> {
        self.list_by_status(IssueStatus::Unassigned).await
    }

    pub async fn list_by_epic(&self, epic_id: &EpicId) -> Result<Vec<Issue>> {
        let rows = sqlx::query_as!(
            IssueRow,
            r#"
            SELECT
                id, github_id, github_url,
                epic_id, stage_id,
                repo_path, repo_url, worktree_path, branch_name,
                title, prompt, issue_type, priority, labels,
                status, assigned_agent_id, agent_type,
                created_at, updated_at, assigned_at, completed_at
            FROM issues
            WHERE epic_id = ?
            ORDER BY created_at ASC
            "#,
            epic_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Issue::from).collect())
    }

    pub async fn list_by_agent(&self, agent_id: &AgentId) -> Result<Vec<Issue>> {
        let rows = sqlx::query_as!(
            IssueRow,
            r#"
            SELECT
                id, github_id, github_url,
                epic_id, stage_id,
                repo_path, repo_url, worktree_path, branch_name,
                title, prompt, issue_type, priority, labels,
                status, assigned_agent_id, agent_type,
                created_at, updated_at, assigned_at, completed_at
            FROM issues
            WHERE assigned_agent_id = ?
            "#,
            agent_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Issue::from).collect())
    }

    pub async fn count_by_status(&self) -> Result<Vec<(IssueStatus, i64)>> {
        let rows = sqlx::query!(
            r#"
            SELECT status, COUNT(*) as count
            FROM issues
            GROUP BY status
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let status = IssueStatus::from_str(&r.status).ok()?;
                Some((status, r.count))
            })
            .collect())
    }
}
```

### Row Type Conversions

```rust
// dispatch-db/src/row_types.rs

use chrono::{DateTime, Utc};
use dispatch_core::types::ids::*;
use dispatch_core::types::issue::{Issue, IssueStatus, IssueType, Priority};

#[derive(Debug, sqlx::FromRow)]
pub struct IssueRow {
    pub id: String,
    pub github_id: Option<i64>,
    pub github_url: Option<String>,
    pub epic_id: Option<String>,
    pub stage_id: Option<String>,
    pub repo_path: String,
    pub repo_url: Option<String>,
    pub worktree_path: Option<String>,
    pub branch_name: Option<String>,
    pub title: String,
    pub prompt: String,
    pub issue_type: String,
    pub priority: String,
    pub labels: String,  // JSON
    pub status: String,
    pub assigned_agent_id: Option<String>,
    pub agent_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub assigned_at: Option<String>,
    pub completed_at: Option<String>,
}

impl From<IssueRow> for Issue {
    fn from(row: IssueRow) -> Self {
        Issue {
            id: row.id.parse().unwrap(),
            github_id: row.github_id.map(|id| id as u64),
            github_url: row.github_url,
            epic_id: row.epic_id.and_then(|id| id.parse().ok()),
            stage_id: row.stage_id.and_then(|id| id.parse().ok()),
            repo_path: PathBuf::from(row.repo_path),
            repo_url: row.repo_url,
            worktree_path: row.worktree_path.map(PathBuf::from),
            branch_name: row.branch_name,
            title: row.title,
            prompt: row.prompt,
            issue_type: row.issue_type.parse().unwrap_or(IssueType::Feature),
            priority: row.priority.parse().unwrap_or(Priority::Medium),
            labels: serde_json::from_str(&row.labels).unwrap_or_default(),
            status: row.status.parse().unwrap_or(IssueStatus::Unassigned),
            assigned_agent_id: row.assigned_agent_id.and_then(|id| id.parse().ok()),
            agent_type: row.agent_type.and_then(|t| t.parse().ok()),
            linked_pr_id: None,  // Loaded separately if needed
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            assigned_at: row.assigned_at.and_then(|t|
                DateTime::parse_from_rfc3339(&t).ok().map(|dt| dt.with_timezone(&Utc))
            ),
            completed_at: row.completed_at.and_then(|t|
                DateTime::parse_from_rfc3339(&t).ok().map(|dt| dt.with_timezone(&Utc))
            ),
        }
    }
}
```

---

## dispatch-git

### Module Structure

```
dispatch-git/src/
├── lib.rs
├── repo.rs               # Repository detection and basic ops
├── worktree.rs           # Worktree creation/deletion
├── branch.rs             # Branch naming and creation
└── commit.rs             # Commit tracking utilities
```

### Key Implementation

```rust
// dispatch-git/src/worktree.rs

use git2::Repository;
use std::path::{Path, PathBuf};

use dispatch_core::error::{DispatchError, Result};
use dispatch_core::types::ids::IssueId;

pub struct WorktreeManager {
    repo: Repository,
    worktrees_base: PathBuf,
}

impl WorktreeManager {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        let worktrees_base = repo_path.join(".dispatch-worktrees");

        Ok(Self {
            repo,
            worktrees_base,
        })
    }

    /// Generate worktree path for an issue
    pub fn worktree_path(&self, issue_id: &IssueId) -> PathBuf {
        self.worktrees_base.join(issue_id.to_string())
    }

    /// Generate branch name for an issue
    pub fn branch_name(issue_id: &IssueId, title: &str) -> String {
        let sanitized = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let truncated = &sanitized[..sanitized.len().min(50)];
        format!("dispatch/{}/{}", issue_id, truncated.trim_end_matches('-'))
    }

    /// Create a worktree for an issue
    pub fn create_worktree(
        &self,
        issue_id: &IssueId,
        title: &str,
        base_branch: Option<&str>,
    ) -> Result<WorktreeInfo> {
        let worktree_path = self.worktree_path(issue_id);
        let branch_name = Self::branch_name(issue_id, title);
        let base = base_branch.unwrap_or("main");

        // Ensure worktrees directory exists
        std::fs::create_dir_all(&self.worktrees_base)?;

        // Get the base commit
        let base_ref = self.repo.find_branch(base, git2::BranchType::Local)?;
        let base_commit = base_ref.get().peel_to_commit()?;

        // Create the new branch
        self.repo.branch(&branch_name, &base_commit, false)?;

        // Create the worktree
        self.repo.worktree(
            &issue_id.to_string(),
            &worktree_path,
            Some(
                git2::WorktreeAddOptions::new()
                    .reference(Some(&format!("refs/heads/{}", branch_name))),
            ),
        )?;

        Ok(WorktreeInfo {
            path: worktree_path,
            branch: branch_name,
            issue_id: issue_id.clone(),
        })
    }

    /// Delete a worktree
    pub fn delete_worktree(&self, issue_id: &IssueId) -> Result<()> {
        let worktree_path = self.worktree_path(issue_id);

        // Find and remove the worktree
        if let Ok(worktree) = self.repo.find_worktree(&issue_id.to_string()) {
            worktree.prune(Some(
                git2::WorktreePruneOptions::new()
                    .working_tree(true)
                    .valid(true),
            ))?;
        }

        // Remove the directory if it exists
        if worktree_path.exists() {
            std::fs::remove_dir_all(&worktree_path)?;
        }

        Ok(())
    }

    /// List all dispatch worktrees
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let mut worktrees = Vec::new();

        for name in self.repo.worktrees()?.iter() {
            if let Some(name) = name {
                if let Ok(issue_id) = name.parse::<IssueId>() {
                    if let Ok(worktree) = self.repo.find_worktree(name) {
                        if let Some(path) = worktree.path().map(PathBuf::from) {
                            // Get branch from worktree
                            let branch = self.get_worktree_branch(&path)?;
                            worktrees.push(WorktreeInfo {
                                path,
                                branch,
                                issue_id,
                            });
                        }
                    }
                }
            }
        }

        Ok(worktrees)
    }

    fn get_worktree_branch(&self, worktree_path: &Path) -> Result<String> {
        let worktree_repo = Repository::open(worktree_path)?;
        let head = worktree_repo.head()?;
        let branch_name = head
            .shorthand()
            .ok_or_else(|| DispatchError::Git("Could not get branch name".into()))?;
        Ok(branch_name.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub issue_id: IssueId,
}
```

---

## dispatch-agents

### Module Structure

```
dispatch-agents/src/
├── lib.rs
├── executor.rs           # Subprocess spawning
├── types.rs              # Agent type prompts
├── lifecycle.rs          # Start/monitor/stop
├── heartbeat.rs          # Health monitoring
├── context.rs            # Issue → Agent context
└── output.rs             # Output parsing
```

### Agent Executor

```rust
// dispatch-agents/src/executor.rs

use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, BufReader};

use dispatch_core::error::Result;
use dispatch_core::types::agent::{Agent, AgentType};
use dispatch_core::types::issue::Issue;

pub struct AgentExecutor {
    prompts_dir: PathBuf,
}

impl AgentExecutor {
    pub fn new(prompts_dir: PathBuf) -> Self {
        Self { prompts_dir }
    }

    /// Spawn a Claude Code agent for an issue
    pub async fn spawn(
        &self,
        agent: &Agent,
        issue: &Issue,
        worktree_path: &Path,
    ) -> Result<AgentProcess> {
        let prompt = self.build_prompt(agent.agent_type, issue)?;
        let system_prompt = self.load_system_prompt(agent.agent_type)?;

        let mut cmd = Command::new("claude");
        cmd.arg("--print")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--system-prompt")
            .arg(&system_prompt)
            .arg(&prompt)
            .current_dir(worktree_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()?;
        let pid = child.id().unwrap_or(0);

        Ok(AgentProcess {
            child,
            pid,
            agent_id: agent.id.clone(),
        })
    }

    fn build_prompt(&self, agent_type: AgentType, issue: &Issue) -> Result<String> {
        Ok(format!(
            "# Issue: {}\n\n## Description\n\n{}\n\n## Labels\n\n{}\n\n## Priority\n\n{}",
            issue.title,
            issue.prompt,
            issue.labels.join(", "),
            issue.priority.as_str()
        ))
    }

    fn load_system_prompt(&self, agent_type: AgentType) -> Result<String> {
        let path = self.prompts_dir.join(agent_type.prompt_file());
        std::fs::read_to_string(&path).map_err(|e| {
            DispatchError::Config(format!("Could not load prompt file {:?}: {}", path, e))
        })
    }
}

pub struct AgentProcess {
    child: Child,
    pid: u32,
    agent_id: AgentId,
}

impl AgentProcess {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        Ok(self.child.wait().await?)
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }

    /// Stream stdout lines
    pub fn stdout_lines(&mut self) -> impl futures::Stream<Item = String> + '_ {
        let stdout = self.child.stdout.take().expect("stdout not captured");
        let reader = BufReader::new(stdout);
        tokio_stream::wrappers::LinesStream::new(reader.lines())
            .filter_map(|line| async { line.ok() })
    }
}
```

### Lifecycle Manager

```rust
// dispatch-agents/src/lifecycle.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use dispatch_core::error::Result;
use dispatch_core::types::agent::{Agent, AgentId, AgentStatus};
use dispatch_core::types::issue::Issue;
use dispatch_db::repos::agent::AgentRepository;
use dispatch_db::repos::issue::IssueRepository;
use dispatch_git::worktree::WorktreeManager;

use crate::executor::{AgentExecutor, AgentProcess};

pub struct AgentLifecycleManager {
    executor: AgentExecutor,
    agent_repo: AgentRepository,
    issue_repo: IssueRepository,
    worktree_manager: WorktreeManager,
    running_agents: Arc<RwLock<HashMap<AgentId, AgentProcess>>>,
}

impl AgentLifecycleManager {
    pub async fn start_agent(&self, agent: &mut Agent, issue: &Issue) -> Result<()> {
        // Create worktree if needed
        let worktree = if let Some(ref path) = issue.worktree_path {
            path.clone()
        } else {
            let info = self.worktree_manager.create_worktree(
                &issue.id,
                &issue.title,
                None,
            )?;
            info.path
        };

        // Update agent status
        agent.status = AgentStatus::Starting;
        agent.current_issue_id = Some(issue.id.clone());
        agent.worktree_path = Some(worktree.clone());
        self.agent_repo.update(agent).await?;

        // Spawn process
        let process = self.executor.spawn(agent, issue, &worktree).await?;
        agent.process_id = Some(process.pid());
        agent.status = AgentStatus::Working;
        self.agent_repo.update(agent).await?;

        // Track running process
        self.running_agents
            .write()
            .await
            .insert(agent.id.clone(), process);

        Ok(())
    }

    pub async fn stop_agent(&self, agent_id: &AgentId) -> Result<()> {
        let mut running = self.running_agents.write().await;
        if let Some(mut process) = running.remove(agent_id) {
            process.kill().await?;
        }

        // Update agent status
        if let Some(mut agent) = self.agent_repo.get(agent_id).await? {
            agent.status = AgentStatus::Completed;
            agent.completed_at = Some(Utc::now());
            self.agent_repo.update(&agent).await?;
        }

        Ok(())
    }

    pub async fn pause_agent(&self, agent_id: &AgentId) -> Result<()> {
        if let Some(mut agent) = self.agent_repo.get(agent_id).await? {
            agent.status = AgentStatus::Paused;
            self.agent_repo.update(&agent).await?;
        }
        // Note: Actual process pausing would require SIGSTOP on Unix
        Ok(())
    }

    pub async fn resume_agent(&self, agent_id: &AgentId) -> Result<()> {
        if let Some(mut agent) = self.agent_repo.get(agent_id).await? {
            if agent.status == AgentStatus::Paused {
                agent.status = AgentStatus::Working;
                self.agent_repo.update(&agent).await?;
            }
        }
        // Note: Actual process resuming would require SIGCONT on Unix
        Ok(())
    }
}
```

---

## dispatch-governance

### Module Structure

```
dispatch-governance/src/
├── lib.rs
├── proposals.rs          # Proposal creation
├── voting.rs             # Vote collection
├── consensus.rs          # Consensus algorithms
├── execution.rs          # Execute approved proposals
├── overrides.rs          # Human override logic
└── broadcast.rs          # Agent notification
```

### Consensus Implementation

```rust
// dispatch-governance/src/consensus.rs

use dispatch_core::types::proposal::{
    ConsensusThreshold, Proposal, ProposalStatus, Vote, VoteDecision,
};

pub struct ConsensusCalculator;

impl ConsensusCalculator {
    /// Calculate if consensus has been reached
    pub fn calculate(
        proposal: &Proposal,
        votes: &[Vote],
        required_voters: &[AgentType],
    ) -> ConsensusResult {
        let total_required = required_voters.len();
        let votes_cast = votes.len();

        // Check if all required voters have voted
        if votes_cast < total_required {
            return ConsensusResult::Pending {
                votes_received: votes_cast,
                votes_required: total_required,
            };
        }

        let approvals = votes.iter().filter(|v| v.decision == VoteDecision::Approve).count();
        let rejections = votes.iter().filter(|v| v.decision == VoteDecision::Reject).count();
        let abstentions = votes.iter().filter(|v| v.decision == VoteDecision::Abstain).count();
        let need_more_info = votes.iter().filter(|v| v.decision == VoteDecision::NeedMoreInfo).count();

        // If anyone needs more info, don't conclude yet
        if need_more_info > 0 {
            return ConsensusResult::NeedsMoreInfo {
                requesters: votes
                    .iter()
                    .filter(|v| v.decision == VoteDecision::NeedMoreInfo)
                    .map(|v| v.voter_id.clone())
                    .collect(),
            };
        }

        let voting_count = approvals + rejections; // Exclude abstentions from ratio
        if voting_count == 0 {
            return ConsensusResult::NoQuorum;
        }

        let approval_ratio = approvals as f64 / voting_count as f64;

        match proposal.threshold {
            ConsensusThreshold::Unanimous => {
                if rejections == 0 && approvals == voting_count {
                    ConsensusResult::Approved { approval_ratio }
                } else {
                    ConsensusResult::Rejected { approval_ratio }
                }
            }
            ConsensusThreshold::SuperMajority => {
                if approval_ratio >= 0.67 {
                    ConsensusResult::Approved { approval_ratio }
                } else {
                    ConsensusResult::Rejected { approval_ratio }
                }
            }
            ConsensusThreshold::SimpleMajority => {
                if approval_ratio > 0.5 {
                    ConsensusResult::Approved { approval_ratio }
                } else {
                    ConsensusResult::Rejected { approval_ratio }
                }
            }
            ConsensusThreshold::SingleApproval => {
                if approvals >= 1 {
                    ConsensusResult::Approved { approval_ratio }
                } else {
                    ConsensusResult::Rejected { approval_ratio }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConsensusResult {
    Pending {
        votes_received: usize,
        votes_required: usize,
    },
    NeedsMoreInfo {
        requesters: Vec<AgentId>,
    },
    NoQuorum,
    Approved {
        approval_ratio: f64,
    },
    Rejected {
        approval_ratio: f64,
    },
}

impl ConsensusResult {
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Approved { .. } | Self::Rejected { .. } | Self::NoQuorum)
    }
}
```

---

## Testing Patterns

### Unit Tests

```rust
// dispatch-core/src/types/issue_test.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_status_transitions() {
        let mut issue = Issue::new(
            PathBuf::from("/repo"),
            "Test issue".into(),
            "Do something".into(),
            IssueType::Feature,
        );

        // Valid transition
        assert!(issue.transition_to(IssueStatus::Queued).is_ok());
        assert_eq!(issue.status, IssueStatus::Queued);

        // Invalid transition
        assert!(issue.transition_to(IssueStatus::Done).is_err());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Medium);
        assert!(Priority::Medium < Priority::Low);
    }
}
```

### Integration Tests

```rust
// dispatch-db/tests/issue_repo_test.rs

use sqlx::SqlitePool;
use dispatch_db::repos::issue::IssueRepository;
use dispatch_core::types::issue::{Issue, IssueType};

#[sqlx::test]
async fn test_issue_crud(pool: SqlitePool) {
    let repo = IssueRepository::new(pool);

    // Create
    let issue = Issue::new(
        PathBuf::from("/repo"),
        "Test issue".into(),
        "Description".into(),
        IssueType::Feature,
    );
    repo.create(&issue).await.unwrap();

    // Read
    let fetched = repo.get(&issue.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Test issue");

    // Update
    let mut updated = fetched;
    updated.title = "Updated title".into();
    repo.update(&updated).await.unwrap();

    let fetched = repo.get(&issue.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, "Updated title");

    // Delete
    repo.delete(&issue.id).await.unwrap();
    assert!(repo.get(&issue.id).await.unwrap().is_none());
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-004 | Core data types | `dispatch-core/src/types/*.rs`, `dispatch-core/src/error.rs` |
| PR-005 | Database repositories | `dispatch-db/src/repos/*.rs`, `dispatch-db/src/row_types.rs` |
| PR-027 | Agent data model | `dispatch-agents/src/types.rs` |
| PR-028 | Agent executor | `dispatch-agents/src/executor.rs` |
| PR-045 | Governance types | `dispatch-governance/src/proposals.rs`, `dispatch-governance/src/voting.rs` |
| PR-047 | Consensus calculator | `dispatch-governance/src/consensus.rs` |
