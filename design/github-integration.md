# GitHub Integration Design

## Overview

This document defines the GitHub integration layer, including API interactions, webhook handling, and bidirectional synchronization.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      GitHub                                  │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│   │  Issues  │  │   PRs    │  │ Comments │  │ Webhooks │   │
│   └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
└────────┼─────────────┼─────────────┼─────────────┼─────────┘
         │             │             │             │
         ▼             ▼             ▼             ▼
┌─────────────────────────────────────────────────────────────┐
│                   dispatch-github                            │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│   │  Client  │  │   Sync   │  │ Metadata │  │ Webhook  │   │
│   │ (octo)   │  │  Engine  │  │  Parser  │  │ Handler  │   │
│   └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│        │             │             │             │          │
│        └─────────────┴─────────────┴─────────────┘          │
│                          │                                   │
└──────────────────────────┼───────────────────────────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ dispatch-db │
                    └─────────────┘
```

---

## GitHub API Client

### Configuration

```rust
// dispatch-github/src/client.rs

use octocrab::Octocrab;

pub struct GitHubClient {
    octo: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubClient {
    pub fn new(token: &str, owner: String, repo: String) -> Result<Self> {
        let octo = Octocrab::builder()
            .personal_token(token.to_string())
            .build()?;

        Ok(Self { octo, owner, repo })
    }

    pub fn from_config(config: &GitHubConfig) -> Result<Self> {
        Self::new(&config.token, config.owner.clone(), config.repo.clone())
    }
}
```

### Configuration File

```toml
# config.toml

[github]
token = "ghp_xxxxxxxxxxxx"  # Or use GITHUB_TOKEN env var
owner = "myorg"
repo = "myrepo"

# Optional: Multiple repositories
[[github.repos]]
owner = "myorg"
repo = "frontend"
path = "/path/to/frontend"

[[github.repos]]
owner = "myorg"
repo = "backend"
path = "/path/to/backend"
```

---

## Issue Synchronization

### Metadata Storage

Store dispatch metadata in GitHub issue body using HTML comments:

```markdown
# Issue Title

Regular issue description visible to users.

This is a feature request for adding dark mode support.

## Acceptance Criteria
- [ ] Toggle in settings
- [ ] Persist preference
- [ ] Apply to all pages

<!-- dispatch:metadata
{
  "dispatch_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "in_progress",
  "priority": "high",
  "type": "feature",
  "epic_id": "660e8400-e29b-41d4-a716-446655440001",
  "stage_id": "770e8400-e29b-41d4-a716-446655440002",
  "assigned_agent_id": "880e8400-e29b-41d4-a716-446655440003",
  "agent_type": "coder",
  "worktree_path": "/repo/.dispatch-worktrees/550e8400",
  "branch_name": "dispatch/550e8400/add-dark-mode",
  "linked_pr": 42,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T14:22:00Z"
}
-->
```

### Metadata Parser

```rust
// dispatch-github/src/metadata.rs

use regex::Regex;
use serde::{Deserialize, Serialize};

const METADATA_START: &str = "<!-- dispatch:metadata";
const METADATA_END: &str = "-->";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchMetadata {
    pub dispatch_id: String,
    pub status: String,
    pub priority: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub epic_id: Option<String>,
    pub stage_id: Option<String>,
    pub assigned_agent_id: Option<String>,
    pub agent_type: Option<String>,
    pub worktree_path: Option<String>,
    pub branch_name: Option<String>,
    pub linked_pr: Option<u64>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn extract_metadata(body: &str) -> Option<DispatchMetadata> {
    let re = Regex::new(r"<!-- dispatch:metadata\n([\s\S]*?)\n-->").ok()?;
    let caps = re.captures(body)?;
    let json = caps.get(1)?.as_str();
    serde_json::from_str(json).ok()
}

pub fn inject_metadata(body: &str, metadata: &DispatchMetadata) -> String {
    let metadata_block = format!(
        "{}\n{}\n{}",
        METADATA_START,
        serde_json::to_string_pretty(metadata).unwrap(),
        METADATA_END
    );

    // Remove existing metadata if present
    let re = Regex::new(r"<!-- dispatch:metadata\n[\s\S]*?\n-->").unwrap();
    let cleaned = re.replace(body, "").trim().to_string();

    // Append new metadata
    format!("{}\n\n{}", cleaned, metadata_block)
}

pub fn strip_metadata(body: &str) -> String {
    let re = Regex::new(r"\n*<!-- dispatch:metadata\n[\s\S]*?\n-->").unwrap();
    re.replace(body, "").trim().to_string()
}
```

### Issue Operations

```rust
// dispatch-github/src/issues.rs

use octocrab::models::issues::Issue as GhIssue;
use dispatch_core::types::issue::{Issue, IssueStatus, IssueType, Priority};

impl GitHubClient {
    /// Create a GitHub issue from a local issue
    pub async fn create_issue(&self, issue: &Issue) -> Result<u64> {
        let metadata = DispatchMetadata::from(issue);
        let body = inject_metadata(&issue.prompt, &metadata);

        let labels: Vec<String> = self.build_labels(issue);

        let gh_issue = self.octo
            .issues(&self.owner, &self.repo)
            .create(&issue.title)
            .body(&body)
            .labels(&labels)
            .send()
            .await?;

        Ok(gh_issue.number)
    }

    /// Update GitHub issue from local issue
    pub async fn update_issue(&self, issue: &Issue) -> Result<()> {
        let github_id = issue.github_id.ok_or(DispatchError::Validation(
            "Issue has no GitHub ID".to_string()
        ))?;

        let metadata = DispatchMetadata::from(issue);
        let body = inject_metadata(&issue.prompt, &metadata);

        let labels: Vec<String> = self.build_labels(issue);

        self.octo
            .issues(&self.owner, &self.repo)
            .update(github_id)
            .title(&issue.title)
            .body(&body)
            .labels(&labels)
            .send()
            .await?;

        Ok(())
    }

    /// Fetch GitHub issue and convert to local issue
    pub async fn get_issue(&self, number: u64) -> Result<Option<Issue>> {
        let gh_issue = self.octo
            .issues(&self.owner, &self.repo)
            .get(number)
            .await?;

        Ok(Some(self.convert_gh_issue(&gh_issue)?))
    }

    /// List issues with optional filters
    pub async fn list_issues(
        &self,
        state: Option<octocrab::params::State>,
        labels: Option<Vec<String>>,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Issue>> {
        let mut request = self.octo
            .issues(&self.owner, &self.repo)
            .list();

        if let Some(state) = state {
            request = request.state(state);
        }

        if let Some(labels) = labels {
            request = request.labels(&labels);
        }

        if let Some(since) = since {
            request = request.since(since);
        }

        let issues = request.per_page(100).send().await?;

        issues
            .items
            .iter()
            .filter(|i| i.pull_request.is_none()) // Exclude PRs
            .map(|i| self.convert_gh_issue(i))
            .collect()
    }

    /// Close a GitHub issue
    pub async fn close_issue(&self, number: u64) -> Result<()> {
        self.octo
            .issues(&self.owner, &self.repo)
            .update(number)
            .state(octocrab::models::IssueState::Closed)
            .send()
            .await?;
        Ok(())
    }

    fn build_labels(&self, issue: &Issue) -> Vec<String> {
        let mut labels = issue.labels.clone();

        // Add dispatch labels
        labels.push("dispatch".to_string());
        labels.push(format!("priority:{}", issue.priority.as_str()));
        labels.push(format!("type:{}", issue.issue_type.as_str()));
        labels.push(format!("status:{}", issue.status.as_str()));

        if issue.epic_id.is_some() {
            labels.push("has-epic".to_string());
        }

        labels
    }

    fn convert_gh_issue(&self, gh: &GhIssue) -> Result<Issue> {
        let body = gh.body.as_deref().unwrap_or("");
        let metadata = extract_metadata(body);
        let prompt = strip_metadata(body);

        // Parse labels
        let (priority, issue_type, status) = self.parse_labels(&gh.labels);

        let issue = if let Some(meta) = metadata {
            // Issue was created by dispatch
            Issue {
                id: meta.dispatch_id.parse()?,
                github_id: Some(gh.number),
                github_url: Some(gh.html_url.to_string()),
                epic_id: meta.epic_id.and_then(|s| s.parse().ok()),
                stage_id: meta.stage_id.and_then(|s| s.parse().ok()),
                repo_path: self.repo_path.clone(),
                repo_url: Some(format!("https://github.com/{}/{}", self.owner, self.repo)),
                worktree_path: meta.worktree_path.map(PathBuf::from),
                branch_name: meta.branch_name,
                title: gh.title.clone(),
                prompt,
                issue_type: meta.issue_type.parse().unwrap_or(IssueType::Feature),
                priority: meta.priority.parse().unwrap_or(Priority::Medium),
                labels: gh.labels.iter().map(|l| l.name.clone()).collect(),
                status: meta.status.parse().unwrap_or(IssueStatus::Unassigned),
                assigned_agent_id: meta.assigned_agent_id.and_then(|s| s.parse().ok()),
                agent_type: meta.agent_type.and_then(|s| s.parse().ok()),
                linked_pr_id: meta.linked_pr.map(|_| todo!("lookup PR")),
                created_at: gh.created_at,
                updated_at: gh.updated_at.unwrap_or(gh.created_at),
                assigned_at: None,
                completed_at: None,
            }
        } else {
            // Issue created externally, import it
            Issue::new(
                self.repo_path.clone(),
                gh.title.clone(),
                prompt,
                issue_type.unwrap_or(IssueType::Feature),
            )
        };

        Ok(issue)
    }

    fn parse_labels(&self, labels: &[octocrab::models::Label]) -> (
        Option<Priority>,
        Option<IssueType>,
        Option<IssueStatus>,
    ) {
        let mut priority = None;
        let mut issue_type = None;
        let mut status = None;

        for label in labels {
            if label.name.starts_with("priority:") {
                priority = label.name.strip_prefix("priority:").and_then(|s| s.parse().ok());
            } else if label.name.starts_with("type:") {
                issue_type = label.name.strip_prefix("type:").and_then(|s| s.parse().ok());
            } else if label.name.starts_with("status:") {
                status = label.name.strip_prefix("status:").and_then(|s| s.parse().ok());
            }
        }

        (priority, issue_type, status)
    }
}
```

---

## Pull Request Operations

```rust
// dispatch-github/src/prs.rs

use octocrab::models::pulls::PullRequest as GhPr;
use dispatch_core::types::pr::{PullRequest, PRStatus, ReviewStatus};

impl GitHubClient {
    /// Create a pull request
    pub async fn create_pr(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
        issue_id: &IssueId,
    ) -> Result<PullRequest> {
        let gh_pr = self.octo
            .pulls(&self.owner, &self.repo)
            .create(title, head, base)
            .body(body)
            .send()
            .await?;

        // Link to issue via comment
        self.add_pr_link_comment(issue_id, gh_pr.number).await?;

        Ok(PullRequest {
            id: PullRequestId::new(),
            issue_id: issue_id.clone(),
            github_number: gh_pr.number,
            github_url: gh_pr.html_url.unwrap().to_string(),
            branch: head.to_string(),
            status: PRStatus::Open,
            checks_passing: false,
            review_status: ReviewStatus::Pending,
            created_at: gh_pr.created_at.unwrap(),
            merged_at: None,
        })
    }

    /// Get PR status including checks
    pub async fn get_pr_status(&self, number: u64) -> Result<(PRStatus, ReviewStatus, bool)> {
        let gh_pr = self.octo
            .pulls(&self.owner, &self.repo)
            .get(number)
            .await?;

        let status = match (gh_pr.merged, gh_pr.state.as_deref()) {
            (Some(true), _) => PRStatus::Merged,
            (_, Some("closed")) => PRStatus::Closed,
            (_, Some("open")) if gh_pr.draft.unwrap_or(false) => PRStatus::Draft,
            _ => PRStatus::Open,
        };

        // Get reviews
        let reviews = self.octo
            .pulls(&self.owner, &self.repo)
            .list_reviews(number)
            .send()
            .await?;

        let review_status = self.calculate_review_status(&reviews.items);

        // Get check runs
        let checks_passing = self.get_checks_status(&gh_pr.head.sha).await?;

        Ok((status, review_status, checks_passing))
    }

    /// Request review from users
    pub async fn request_review(&self, number: u64, reviewers: &[String]) -> Result<()> {
        self.octo
            .pulls(&self.owner, &self.repo)
            .request_reviews(number, reviewers.to_vec(), vec![])
            .await?;
        Ok(())
    }

    /// Merge a pull request
    pub async fn merge_pr(&self, number: u64, method: MergeMethod) -> Result<()> {
        let merge_method = match method {
            MergeMethod::Merge => octocrab::params::pulls::MergeMethod::Merge,
            MergeMethod::Squash => octocrab::params::pulls::MergeMethod::Squash,
            MergeMethod::Rebase => octocrab::params::pulls::MergeMethod::Rebase,
        };

        self.octo
            .pulls(&self.owner, &self.repo)
            .merge(number)
            .method(merge_method)
            .send()
            .await?;
        Ok(())
    }

    async fn get_checks_status(&self, sha: &str) -> Result<bool> {
        let check_runs = self.octo
            .checks(&self.owner, &self.repo)
            .list_check_runs_for_git_ref(sha.into())
            .send()
            .await?;

        // All checks must pass
        Ok(check_runs.check_runs.iter().all(|c| {
            c.conclusion.as_deref() == Some("success")
        }))
    }

    fn calculate_review_status(&self, reviews: &[octocrab::models::pulls::Review]) -> ReviewStatus {
        let mut approved = false;
        let mut changes_requested = false;

        for review in reviews {
            match review.state.as_deref() {
                Some("APPROVED") => approved = true,
                Some("CHANGES_REQUESTED") => changes_requested = true,
                _ => {}
            }
        }

        if changes_requested {
            ReviewStatus::ChangesRequested
        } else if approved {
            ReviewStatus::Approved
        } else {
            ReviewStatus::Pending
        }
    }

    async fn add_pr_link_comment(&self, issue_id: &IssueId, pr_number: u64) -> Result<()> {
        // Find the issue's GitHub number
        // Add comment linking PR
        // Update metadata in issue body
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}
```

---

## Webhook Handling

### Webhook Setup

```rust
// dispatch-web/src/routes/webhooks.rs

use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub async fn github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    // Verify signature
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("sha256="));

    if !verify_signature(&body, signature, &state.webhook_secret) {
        return StatusCode::UNAUTHORIZED;
    }

    // Parse event type
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // Parse payload
    let payload: serde_json::Value = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // Handle event
    match event_type {
        "issues" => handle_issue_event(&state, &payload).await,
        "issue_comment" => handle_issue_comment_event(&state, &payload).await,
        "pull_request" => handle_pr_event(&state, &payload).await,
        "pull_request_review" => handle_pr_review_event(&state, &payload).await,
        "check_run" => handle_check_run_event(&state, &payload).await,
        "ping" => StatusCode::OK,
        _ => StatusCode::OK, // Ignore unknown events
    }
}

fn verify_signature(body: &str, signature: Option<&str>, secret: &str) -> bool {
    let signature = match signature {
        Some(s) => s,
        None => return false,
    };

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    // Constant-time comparison
    signature == expected
}
```

### Event Handlers

```rust
// dispatch-github/src/webhooks.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IssueEvent {
    pub action: String,
    pub issue: IssuePayload,
    pub repository: RepositoryPayload,
    pub sender: UserPayload,
}

#[derive(Debug, Deserialize)]
pub struct IssuePayload {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<LabelPayload>,
    pub user: UserPayload,
}

pub async fn handle_issue_event(state: &AppState, payload: &serde_json::Value) -> StatusCode {
    let event: IssueEvent = match serde_json::from_value(payload.clone()) {
        Ok(e) => e,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match event.action.as_str() {
        "opened" => handle_issue_opened(state, &event).await,
        "edited" => handle_issue_edited(state, &event).await,
        "closed" => handle_issue_closed(state, &event).await,
        "reopened" => handle_issue_reopened(state, &event).await,
        "labeled" => handle_issue_labeled(state, &event).await,
        "unlabeled" => handle_issue_unlabeled(state, &event).await,
        "assigned" => handle_issue_assigned(state, &event).await,
        _ => StatusCode::OK,
    }
}

async fn handle_issue_opened(state: &AppState, event: &IssueEvent) -> StatusCode {
    // Check if this is a dispatch-created issue (has metadata)
    let body = event.issue.body.as_deref().unwrap_or("");
    if extract_metadata(body).is_some() {
        // Already managed by dispatch, ignore
        return StatusCode::OK;
    }

    // Import external issue into dispatch
    let issue = Issue::new(
        state.repo_path.clone(),
        event.issue.title.clone(),
        strip_metadata(body),
        parse_issue_type(&event.issue.labels),
    );

    // Save to database
    if let Err(e) = state.issue_repo.create(&issue).await {
        tracing::error!("Failed to import issue: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // Update GitHub issue with dispatch metadata
    let mut issue = issue;
    issue.github_id = Some(event.issue.number);
    if let Err(e) = state.github_client.update_issue(&issue).await {
        tracing::error!("Failed to update GitHub issue: {}", e);
    }

    // Emit event
    state.events.send(DispatchEvent::IssueCreated { issue_id: issue.id });

    StatusCode::OK
}

async fn handle_issue_edited(state: &AppState, event: &IssueEvent) -> StatusCode {
    let body = event.issue.body.as_deref().unwrap_or("");
    let metadata = match extract_metadata(body) {
        Some(m) => m,
        None => return StatusCode::OK, // Not a dispatch issue
    };

    // Fetch current issue
    let issue_id: IssueId = match metadata.dispatch_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    let mut issue = match state.issue_repo.get(&issue_id).await {
        Ok(Some(i)) => i,
        _ => return StatusCode::NOT_FOUND,
    };

    // Update from GitHub
    issue.title = event.issue.title.clone();
    issue.prompt = strip_metadata(body);

    // Check for command comments in issue body
    // e.g., `/dispatch assign coder` or `/dispatch priority high`
    process_commands(&mut issue, body);

    if let Err(e) = state.issue_repo.update(&issue).await {
        tracing::error!("Failed to update issue: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    state.events.send(DispatchEvent::IssueUpdated { issue_id: issue.id });

    StatusCode::OK
}
```

### Comment Commands

```rust
// dispatch-github/src/commands.rs

use regex::Regex;

/// Parse dispatch commands from issue/PR comments
/// Format: /dispatch <command> [args...]
pub fn parse_command(text: &str) -> Option<DispatchCommand> {
    let re = Regex::new(r"/dispatch\s+(\w+)(?:\s+(.*))?").ok()?;
    let caps = re.captures(text)?;

    let cmd = caps.get(1)?.as_str();
    let args = caps.get(2).map(|m| m.as_str()).unwrap_or("");

    match cmd {
        "assign" => {
            let agent_type = args.parse().ok()?;
            Some(DispatchCommand::Assign { agent_type })
        }
        "unassign" => Some(DispatchCommand::Unassign),
        "priority" => {
            let priority = args.parse().ok()?;
            Some(DispatchCommand::SetPriority { priority })
        }
        "status" => {
            let status = args.parse().ok()?;
            Some(DispatchCommand::SetStatus { status })
        }
        "pause" => Some(DispatchCommand::Pause),
        "resume" => Some(DispatchCommand::Resume),
        "cancel" => Some(DispatchCommand::Cancel),
        "label" => {
            let labels: Vec<String> = args.split_whitespace().map(String::from).collect();
            Some(DispatchCommand::AddLabels { labels })
        }
        "epic" => {
            let epic_id = args.parse().ok()?;
            Some(DispatchCommand::SetEpic { epic_id })
        }
        "help" => Some(DispatchCommand::Help),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum DispatchCommand {
    Assign { agent_type: AgentType },
    Unassign,
    SetPriority { priority: Priority },
    SetStatus { status: IssueStatus },
    Pause,
    Resume,
    Cancel,
    AddLabels { labels: Vec<String> },
    SetEpic { epic_id: EpicId },
    Help,
}

/// Execute a dispatch command
pub async fn execute_command(
    state: &AppState,
    issue_id: &IssueId,
    command: DispatchCommand,
    executor: &str, // GitHub username
) -> Result<String> {
    match command {
        DispatchCommand::Assign { agent_type } => {
            // Assign issue to agent type
            let mut issue = state.issue_repo.get(issue_id).await?.ok_or(
                DispatchError::NotFound { entity: "Issue", id: issue_id.to_string() }
            )?;

            let agent = Agent::new(agent_type);
            state.agent_repo.create(&agent).await?;

            issue.assigned_agent_id = Some(agent.id.clone());
            issue.agent_type = Some(agent_type);
            issue.status = IssueStatus::Assigned;
            state.issue_repo.update(&issue).await?;

            Ok(format!("Assigned to {} agent `{}`", agent_type.as_str(), agent.id))
        }
        DispatchCommand::SetPriority { priority } => {
            let mut issue = state.issue_repo.get(issue_id).await?.ok_or(
                DispatchError::NotFound { entity: "Issue", id: issue_id.to_string() }
            )?;

            issue.priority = priority;
            state.issue_repo.update(&issue).await?;

            Ok(format!("Priority set to {}", priority.as_str()))
        }
        DispatchCommand::Help => {
            Ok(HELP_TEXT.to_string())
        }
        // ... other commands
        _ => Ok("Command not implemented".to_string())
    }
}

const HELP_TEXT: &str = r#"
**Dispatch Commands**

- `/dispatch assign <type>` - Assign to agent (coder, reviewer, pm, security, docs, test, architect)
- `/dispatch unassign` - Remove agent assignment
- `/dispatch priority <level>` - Set priority (critical, high, medium, low)
- `/dispatch status <status>` - Set status
- `/dispatch pause` - Pause assigned agent
- `/dispatch resume` - Resume paused agent
- `/dispatch cancel` - Cancel issue
- `/dispatch label <labels>` - Add labels
- `/dispatch epic <id>` - Associate with epic
- `/dispatch help` - Show this help
"#;
```

---

## Synchronization Engine

### Sync State

```rust
// dispatch-github/src/sync.rs

use chrono::{DateTime, Utc};

pub struct SyncEngine {
    github: GitHubClient,
    issue_repo: IssueRepository,
    pr_repo: PullRequestRepository,
    sync_state_repo: SyncStateRepository,
}

#[derive(Debug, Clone)]
pub struct SyncState {
    pub last_sync_at: DateTime<Utc>,
    pub last_issue_number: Option<u64>,
    pub last_pr_number: Option<u64>,
    pub etag: Option<String>,
}

impl SyncEngine {
    /// Pull changes from GitHub
    pub async fn pull(&self) -> Result<SyncResult> {
        let state = self.sync_state_repo.get().await?.unwrap_or_default();
        let mut result = SyncResult::default();

        // Fetch issues modified since last sync
        let gh_issues = self.github
            .list_issues(None, None, Some(state.last_sync_at))
            .await?;

        for gh_issue in gh_issues {
            match self.sync_issue_from_github(&gh_issue).await {
                Ok(SyncAction::Created) => result.issues_created += 1,
                Ok(SyncAction::Updated) => result.issues_updated += 1,
                Ok(SyncAction::Unchanged) => result.issues_unchanged += 1,
                Err(e) => {
                    result.errors.push(format!("Issue {}: {}", gh_issue.github_id.unwrap_or(0), e));
                }
            }
        }

        // Fetch PRs
        // ... similar logic

        // Update sync state
        self.sync_state_repo.update(&SyncState {
            last_sync_at: Utc::now(),
            last_issue_number: gh_issues.last().and_then(|i| i.github_id),
            last_pr_number: None,
            etag: None,
        }).await?;

        Ok(result)
    }

    /// Push local changes to GitHub
    pub async fn push(&self) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Find issues without GitHub IDs (created locally)
        let local_only = self.issue_repo.list_without_github_id().await?;
        for issue in local_only {
            match self.github.create_issue(&issue).await {
                Ok(github_id) => {
                    let mut issue = issue;
                    issue.github_id = Some(github_id);
                    self.issue_repo.update(&issue).await?;
                    result.issues_created += 1;
                }
                Err(e) => {
                    result.errors.push(format!("Issue {}: {}", issue.id, e));
                }
            }
        }

        // Find issues with pending updates
        let pending_updates = self.issue_repo.list_pending_sync().await?;
        for issue in pending_updates {
            match self.github.update_issue(&issue).await {
                Ok(_) => {
                    result.issues_updated += 1;
                }
                Err(e) => {
                    result.errors.push(format!("Issue {}: {}", issue.id, e));
                }
            }
        }

        Ok(result)
    }

    /// Full bidirectional sync
    pub async fn full_sync(&self) -> Result<SyncResult> {
        // Pull first to get latest state
        let mut result = self.pull().await?;

        // Then push local changes
        let push_result = self.push().await?;

        // Merge results
        result.issues_created += push_result.issues_created;
        result.issues_updated += push_result.issues_updated;
        result.errors.extend(push_result.errors);

        Ok(result)
    }

    async fn sync_issue_from_github(&self, gh_issue: &Issue) -> Result<SyncAction> {
        let github_id = gh_issue.github_id.ok_or(
            DispatchError::Validation("Issue has no GitHub ID".to_string())
        )?;

        // Check if we have this issue locally
        match self.issue_repo.get_by_github_id(github_id).await? {
            Some(local) => {
                // Compare and update if needed
                if gh_issue.updated_at > local.updated_at {
                    // GitHub is newer, update local
                    let mut updated = local;
                    updated.title = gh_issue.title.clone();
                    updated.prompt = gh_issue.prompt.clone();
                    // ... other fields
                    self.issue_repo.update(&updated).await?;
                    Ok(SyncAction::Updated)
                } else {
                    Ok(SyncAction::Unchanged)
                }
            }
            None => {
                // New issue from GitHub
                let mut issue = gh_issue.clone();
                self.issue_repo.create(&issue).await?;
                Ok(SyncAction::Created)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct SyncResult {
    pub issues_created: u32,
    pub issues_updated: u32,
    pub issues_unchanged: u32,
    pub prs_created: u32,
    pub prs_updated: u32,
    pub prs_unchanged: u32,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub enum SyncAction {
    Created,
    Updated,
    Unchanged,
}
```

---

## Epic Integration

### Epic as GitHub Issue

```markdown
# [Epic] User Authentication System

## Description
Implement complete user authentication including login, registration, and session management.

## Stages
1. Design (current)
2. Implementation
3. Testing
4. Documentation

## Acceptance Criteria
- [ ] Users can register with email/password
- [ ] Users can login and receive JWT
- [ ] Session expires after 24 hours
- [ ] Password reset flow works

<!-- dispatch:epic
{
  "dispatch_id": "epic-123",
  "status": "in_progress",
  "current_stage": 0,
  "stages": [
    {"name": "Design", "status": "in_progress", "gate": "approval"},
    {"name": "Implementation", "status": "pending", "gate": "review"},
    {"name": "Testing", "status": "pending", "gate": "checkpoint"},
    {"name": "Documentation", "status": "pending", "gate": null}
  ],
  "child_issues": [1, 2, 3, 4]
}
-->
```

### Gate Comments

```rust
// dispatch-github/src/gates.rs

impl GitHubClient {
    /// Post gate approval request as issue comment
    pub async fn post_gate_comment(&self, issue_number: u64, gate: &Gate) -> Result<()> {
        let comment = format!(
            r#"## 🚧 Gate: {}

**Stage:** {} → {}
**Type:** {}

{}

### Actions
- `/dispatch approve` - Approve and continue
- `/dispatch reject <reason>` - Reject with reason
- `/dispatch skip <reason>` - Skip gate (emergency)

**Required approvers:** {}
"#,
            gate.description,
            "Current Stage",
            "Next Stage",
            gate.gate_type.as_str(),
            gate.description,
            match &gate.required_approvers {
                GateApprovers::Any => "Any team member".to_string(),
                GateApprovers::Specific(users) => users.join(", "),
            }
        );

        self.octo
            .issues(&self.owner, &self.repo)
            .create_comment(issue_number, comment)
            .await?;

        Ok(())
    }

    /// Update gate status in epic metadata
    pub async fn update_gate_status(&self, epic_number: u64, gate_id: &GateId, status: GateApproval, by: &str) -> Result<()> {
        // Fetch epic, update metadata, push back
        todo!()
    }
}
```

---

## Rate Limiting

```rust
// dispatch-github/src/client.rs

use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

impl GitHubClient {
    pub fn with_rate_limit(token: &str, owner: String, repo: String) -> Result<Self> {
        let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(10).unwrap()));

        // ... setup with limiter
        todo!()
    }
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-036 | GitHub API client | `dispatch-github/src/client.rs` |
| PR-037 | Issue sync: GitHub → Local | `dispatch-github/src/sync.rs`, `dispatch-github/src/issues.rs` |
| PR-038 | Issue sync: Local → GitHub | `dispatch-github/src/sync.rs` |
| PR-039 | Metadata storage | `dispatch-github/src/metadata.rs` |
| PR-040 | Webhook receiver | `dispatch-web/src/routes/webhooks.rs` |
| PR-041 | Webhook event handlers | `dispatch-github/src/webhooks.rs` |
| PR-042 | PR creation and linking | `dispatch-github/src/prs.rs` |
| PR-043 | PR status tracking | `dispatch-github/src/prs.rs` |
| PR-044 | CLI sync commands | `dispatch-cli/src/commands/sync.rs` |
