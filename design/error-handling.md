# Error Handling Design

## Overview

Consistent error handling strategy across all crates using thiserror for library errors and anyhow for application-level handling.

---

## Error Type Hierarchy

```
DispatchError (dispatch-core)
├── Database errors (from sqlx)
├── Git errors (from git2)
├── GitHub errors
├── Agent errors
├── Validation errors
├── State machine errors
├── Configuration errors
├── IO errors
└── Other (anyhow)

Each crate may define its own specific error enum that implements Into<DispatchError>
```

---

## Core Error Type

```rust
// dispatch-core/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchError {
    //──────────────────────────────────────────────────────────────────────
    // Database errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    //──────────────────────────────────────────────────────────────────────
    // Git errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Not a git repository: {path}")]
    NotARepository { path: String },

    #[error("Worktree error: {0}")]
    Worktree(String),

    //──────────────────────────────────────────────────────────────────────
    // GitHub API errors
    //──────────────────────────────────────────────────────────────────────
    #[error("GitHub API error: {message}")]
    GitHub { message: String, status: Option<u16> },

    #[error("GitHub rate limit exceeded, resets at {reset_at}")]
    RateLimited { reset_at: String },

    #[error("GitHub authentication failed")]
    GitHubAuth,

    //──────────────────────────────────────────────────────────────────────
    // Agent errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Agent spawn failed: {0}")]
    AgentSpawn(String),

    #[error("Agent {agent_id} timed out after {timeout_secs}s")]
    AgentTimeout { agent_id: String, timeout_secs: u64 },

    #[error("Agent {agent_id} crashed: {reason}")]
    AgentCrash { agent_id: String, reason: String },

    #[error("Agent {agent_id} is not responding")]
    AgentUnresponsive { agent_id: String },

    #[error("No available agent of type {agent_type}")]
    NoAvailableAgent { agent_type: String },

    //──────────────────────────────────────────────────────────────────────
    // Validation errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("{entity} already exists: {id}")]
    AlreadyExists { entity: &'static str, id: String },

    //──────────────────────────────────────────────────────────────────────
    // State machine errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Invalid state transition for {entity}: {from} -> {to}")]
    InvalidStateTransition {
        entity: &'static str,
        from: String,
        to: String,
    },

    #[error("Operation not allowed in state {state}: {operation}")]
    OperationNotAllowed { state: String, operation: String },

    //──────────────────────────────────────────────────────────────────────
    // Governance errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Voting not open for proposal {proposal_id}")]
    VotingNotOpen { proposal_id: String },

    #[error("Agent {agent_id} is not eligible to vote on proposal {proposal_id}")]
    NotEligibleToVote { agent_id: String, proposal_id: String },

    #[error("Agent {agent_id} has already voted on proposal {proposal_id}")]
    AlreadyVoted { agent_id: String, proposal_id: String },

    #[error("Consensus not reached: {reason}")]
    NoConsensus { reason: String },

    //──────────────────────────────────────────────────────────────────────
    // Configuration errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Missing required configuration: {field}")]
    MissingConfig { field: String },

    //──────────────────────────────────────────────────────────────────────
    // IO and serialization errors
    //──────────────────────────────────────────────────────────────────────
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    //──────────────────────────────────────────────────────────────────────
    // Workflow errors
    //──────────────────────────────────────────────────────────────────────
    #[error("Workflow error: {0}")]
    Workflow(String),

    #[error("Maximum iterations ({max}) reached for workflow {workflow_id}")]
    MaxIterationsReached { workflow_id: String, max: u32 },

    #[error("Blocking feedback not resolved: {count} items pending")]
    BlockingFeedback { count: usize },

    //──────────────────────────────────────────────────────────────────────
    // Generic
    //──────────────────────────────────────────────────────────────────────
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type alias for dispatch operations
pub type Result<T> = std::result::Result<T, DispatchError>;
```

---

## Error Context and Chaining

```rust
// dispatch-core/src/error.rs

impl DispatchError {
    /// Add context to an error
    pub fn context<C: Into<String>>(self, context: C) -> Self {
        match self {
            Self::Other(e) => Self::Other(e.context(context.into())),
            other => Self::Other(anyhow::Error::from(other).context(context.into())),
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Database(_)
                | Self::RateLimited { .. }
                | Self::AgentUnresponsive { .. }
                | Self::Io(_)
        )
    }

    /// Check if this requires human intervention
    pub fn requires_human(&self) -> bool {
        matches!(
            self,
            Self::MaxIterationsReached { .. }
                | Self::NoConsensus { .. }
                | Self::GitHubAuth
                | Self::MissingConfig { .. }
        )
    }

    /// Get HTTP status code for this error (for REST API)
    pub fn http_status(&self) -> u16 {
        match self {
            Self::NotFound { .. } => 404,
            Self::AlreadyExists { .. } => 409,
            Self::Validation(_) => 400,
            Self::InvalidStateTransition { .. } => 400,
            Self::OperationNotAllowed { .. } => 403,
            Self::GitHubAuth => 401,
            Self::RateLimited { .. } => 429,
            Self::NotEligibleToVote { .. } => 403,
            Self::AlreadyVoted { .. } => 409,
            _ => 500,
        }
    }

    /// Convert to JSON-serializable error response
    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            error: self.error_code(),
            message: self.to_string(),
            details: self.details(),
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Database(_) | Self::Migration(_) => "database_error",
            Self::Git(_) | Self::NotARepository { .. } | Self::Worktree(_) => "git_error",
            Self::GitHub { .. } | Self::RateLimited { .. } | Self::GitHubAuth => "github_error",
            Self::AgentSpawn(_) | Self::AgentTimeout { .. } | Self::AgentCrash { .. } |
            Self::AgentUnresponsive { .. } | Self::NoAvailableAgent { .. } => "agent_error",
            Self::Validation(_) | Self::NotFound { .. } | Self::AlreadyExists { .. } => "validation_error",
            Self::InvalidStateTransition { .. } | Self::OperationNotAllowed { .. } => "state_error",
            Self::VotingNotOpen { .. } | Self::NotEligibleToVote { .. } |
            Self::AlreadyVoted { .. } | Self::NoConsensus { .. } => "governance_error",
            Self::Config(_) | Self::MissingConfig { .. } => "config_error",
            Self::Workflow(_) | Self::MaxIterationsReached { .. } | Self::BlockingFeedback { .. } => "workflow_error",
            _ => "internal_error",
        }
    }

    fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::NotFound { entity, id } => Some(serde_json::json!({
                "entity": entity,
                "id": id,
            })),
            Self::InvalidStateTransition { entity, from, to } => Some(serde_json::json!({
                "entity": entity,
                "from_state": from,
                "to_state": to,
            })),
            Self::RateLimited { reset_at } => Some(serde_json::json!({
                "reset_at": reset_at,
            })),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

---

## Error Recovery Strategies

### Retry Logic

```rust
// dispatch-core/src/retry.rs

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

pub async fn with_retry<T, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_retryable() && attempt < config.max_attempts => {
                tracing::warn!(
                    "Attempt {}/{} failed: {}. Retrying in {:?}",
                    attempt,
                    config.max_attempts,
                    e,
                    delay
                );
                tokio::time::sleep(delay).await;
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_factor).min(config.max_delay.as_secs_f64())
                );
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Error Escalation

```rust
// dispatch-governance/src/escalation.rs

pub struct ErrorEscalation {
    events: broadcast::Sender<DispatchEvent>,
}

impl ErrorEscalation {
    /// Escalate an error to human oversight if required
    pub async fn maybe_escalate(&self, error: &DispatchError, context: &EscalationContext) -> Result<()> {
        if !error.requires_human() {
            return Ok(());
        }

        let notification = HumanNotification {
            severity: NotificationSeverity::Error,
            title: format!("Error requires attention: {}", error.error_code()),
            message: error.to_string(),
            context: context.clone(),
            suggested_actions: self.suggest_actions(error),
        };

        // Log to decisions table
        self.log_escalation(&notification).await?;

        // Send notification
        self.events.send(DispatchEvent::HumanAttentionRequired {
            notification_id: notification.id.clone(),
        })?;

        Ok(())
    }

    fn suggest_actions(&self, error: &DispatchError) -> Vec<String> {
        match error {
            DispatchError::MaxIterationsReached { .. } => vec![
                "Review the feedback and implementation".into(),
                "Provide additional guidance".into(),
                "Manually complete the task".into(),
            ],
            DispatchError::NoConsensus { .. } => vec![
                "Review the proposal and votes".into(),
                "Force a decision".into(),
                "Request more information".into(),
            ],
            DispatchError::GitHubAuth => vec![
                "Check GITHUB_TOKEN environment variable".into(),
                "Regenerate GitHub token".into(),
            ],
            _ => vec!["Review the error details".into()],
        }
    }
}
```

---

## Logging Errors

```rust
// dispatch-core/src/error.rs

impl DispatchError {
    /// Log this error with appropriate context
    pub fn log(&self) {
        match self {
            Self::AgentCrash { agent_id, reason } => {
                tracing::error!(
                    agent_id = %agent_id,
                    reason = %reason,
                    "Agent crashed"
                );
            }
            Self::AgentTimeout { agent_id, timeout_secs } => {
                tracing::error!(
                    agent_id = %agent_id,
                    timeout_secs = %timeout_secs,
                    "Agent timed out"
                );
            }
            Self::RateLimited { reset_at } => {
                tracing::warn!(
                    reset_at = %reset_at,
                    "GitHub rate limit exceeded"
                );
            }
            Self::Database(e) => {
                tracing::error!(
                    error = %e,
                    "Database error"
                );
            }
            _ => {
                tracing::error!(error = %self, "Error occurred");
            }
        }
    }
}
```

---

## CLI Error Display

```rust
// dispatch-cli/src/output.rs

pub fn display_error(error: &DispatchError, verbose: bool) {
    // Color-coded output
    eprintln!("{} {}", "error:".red().bold(), error);

    if verbose {
        // Show additional context
        if let Some(source) = std::error::Error::source(error) {
            eprintln!("{} {}", "caused by:".yellow(), source);
        }

        // Show suggestions
        if let Some(suggestion) = error_suggestion(error) {
            eprintln!("\n{} {}", "suggestion:".cyan(), suggestion);
        }
    }
}

fn error_suggestion(error: &DispatchError) -> Option<&'static str> {
    match error {
        DispatchError::NotARepository { .. } => {
            Some("Run 'dispatch init' to initialize dispatch in this repository")
        }
        DispatchError::GitHubAuth => {
            Some("Set GITHUB_TOKEN environment variable or add token to config")
        }
        DispatchError::MissingConfig { .. } => {
            Some("Run 'dispatch config init' to create default configuration")
        }
        DispatchError::NoAvailableAgent { .. } => {
            Some("Check max_concurrent setting or wait for an agent to finish")
        }
        _ => None,
    }
}
```

---

## Axum Error Handling

```rust
// dispatch-web/src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

impl IntoResponse for DispatchError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.http_status())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = Json(self.to_error_response());

        (status, body).into_response()
    }
}

// Handler that can return DispatchError
pub async fn get_issue(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Issue>, DispatchError> {
    let issue_id = id.parse().map_err(|_| {
        DispatchError::Validation(format!("Invalid issue ID: {}", id))
    })?;

    let issue = state.issue_repo.get(&issue_id).await?.ok_or_else(|| {
        DispatchError::NotFound {
            entity: "Issue",
            id: id.clone(),
        }
    })?;

    Ok(Json(issue))
}
```

---

## Exit Codes (CLI)

| Code | Meaning | Example Errors |
|------|---------|----------------|
| 0 | Success | - |
| 1 | General error | Unknown/unexpected errors |
| 2 | Invalid arguments | Bad CLI arguments |
| 3 | Configuration error | Missing config, invalid config |
| 4 | Database error | Connection failed, query failed |
| 5 | Git error | Not a repo, worktree failed |
| 6 | GitHub API error | Rate limited, auth failed |
| 7 | Agent error | Spawn failed, crashed, timeout |
| 8 | Not found | Issue/Agent/Epic not found |
| 9 | State error | Invalid state transition |
| 10 | Permission denied | Not eligible, operation not allowed |

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-004a | Core error types | `dispatch-core/src/error.rs` |
| PR-008a | Error logging integration | `dispatch-core/src/error.rs` |
| PR-008b | CLI error display | `dispatch-cli/src/output.rs` |
| PR-040a | Web error handling | `dispatch-web/src/error.rs` |
| PR-083 | Error recovery | `dispatch-core/src/retry.rs`, `dispatch-governance/src/escalation.rs` |
