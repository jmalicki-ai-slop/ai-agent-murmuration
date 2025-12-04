# REST API Design

## Overview

HTTP REST API for the Dispatch server, providing programmatic access to issues, epics, agents, proposals, and system management. Designed primarily for the web UI and external integrations.

---

## Base Configuration

```
Base URL: http://localhost:8080/api/v1
Content-Type: application/json
```

All responses follow a standard envelope:

```typescript
// Success response
interface ApiResponse<T> {
  data: T;
  meta?: {
    page?: number;
    per_page?: number;
    total?: number;
    total_pages?: number;
  };
}

// Error response
interface ApiError {
  error: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}
```

---

## Authentication

### API Token Authentication

```http
Authorization: Bearer <token>
```

### Session Cookie

Set via `/api/v1/auth/login` endpoint. Used by web UI.

---

## Endpoints

### Health & Status

#### GET /health

Health check endpoint for load balancers.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

#### GET /api/v1/status

System status overview.

**Response:**
```json
{
  "data": {
    "issues": {
      "total": 42,
      "open": 15,
      "in_progress": 8,
      "blocked": 2
    },
    "agents": {
      "active": 4,
      "idle": 2,
      "max_concurrent": 8
    },
    "epics": {
      "active": 3,
      "completed": 12
    },
    "github": {
      "connected": true,
      "rate_limit_remaining": 4500,
      "last_sync": "2024-01-15T10:30:00Z"
    }
  }
}
```

---

### Issues

#### GET /api/v1/issues

List issues with filtering and pagination.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| status | string | Filter by status: `open`, `in_progress`, `blocked`, `completed`, `wont_fix` |
| priority | string | Filter by priority: `critical`, `high`, `medium`, `low` |
| epic_id | uuid | Filter by parent epic |
| assigned | boolean | Filter by assignment status |
| search | string | Full-text search in title/description |
| page | integer | Page number (default: 1) |
| per_page | integer | Items per page (default: 20, max: 100) |
| sort | string | Sort field: `created_at`, `updated_at`, `priority` |
| order | string | Sort order: `asc`, `desc` |

**Response:**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "github_number": 42,
      "title": "Fix authentication bug",
      "description": "Users are logged out unexpectedly...",
      "status": "in_progress",
      "priority": "high",
      "issue_type": "bug",
      "epic_id": "660e8400-e29b-41d4-a716-446655440000",
      "assigned_agent_id": "770e8400-e29b-41d4-a716-446655440000",
      "worktree_path": "/repo/.dispatch-worktrees/550e8400",
      "labels": ["bug", "auth", "P1"],
      "created_at": "2024-01-10T08:00:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 42,
    "total_pages": 3
  }
}
```

#### GET /api/v1/issues/:id

Get issue details.

**Response:**
```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "github_number": 42,
    "title": "Fix authentication bug",
    "description": "Full description...",
    "status": "in_progress",
    "priority": "high",
    "issue_type": "bug",
    "epic_id": "660e8400-e29b-41d4-a716-446655440000",
    "epic": {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "title": "Authentication System"
    },
    "assigned_agent": {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "agent_type": "coder",
      "status": "running"
    },
    "worktree_path": "/repo/.dispatch-worktrees/550e8400",
    "branch_name": "dispatch/42/fix-auth-bug",
    "labels": ["bug", "auth", "P1"],
    "dependencies": [],
    "dependents": [],
    "proposals": [
      {
        "id": "880e8400-e29b-41d4-a716-446655440000",
        "proposal_type": "implementation",
        "status": "approved"
      }
    ],
    "created_at": "2024-01-10T08:00:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

#### POST /api/v1/issues

Create a new issue (also creates GitHub issue).

**Request:**
```json
{
  "title": "Add dark mode support",
  "description": "Implement dark mode theme...",
  "priority": "medium",
  "issue_type": "feature",
  "epic_id": "660e8400-e29b-41d4-a716-446655440000",
  "labels": ["enhancement", "ui"]
}
```

**Response:** 201 Created
```json
{
  "data": {
    "id": "990e8400-e29b-41d4-a716-446655440000",
    "github_number": 43,
    "title": "Add dark mode support",
    ...
  }
}
```

#### PATCH /api/v1/issues/:id

Update issue fields.

**Request:**
```json
{
  "priority": "high",
  "status": "in_progress"
}
```

#### DELETE /api/v1/issues/:id

Close/delete an issue.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| close_github | boolean | Also close GitHub issue (default: true) |

---

### Epics

#### GET /api/v1/epics

List epics.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| status | string | Filter: `active`, `completed`, `paused` |
| page | integer | Page number |
| per_page | integer | Items per page |

**Response:**
```json
{
  "data": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440000",
      "github_number": 10,
      "title": "Authentication System",
      "description": "Complete auth implementation...",
      "status": "active",
      "current_stage": 1,
      "stages": [
        {
          "name": "Design",
          "status": "completed",
          "gate_type": "approval",
          "completed_at": "2024-01-12T10:00:00Z"
        },
        {
          "name": "Implementation",
          "status": "in_progress",
          "gate_type": "review",
          "completed_at": null
        },
        {
          "name": "Testing",
          "status": "pending",
          "gate_type": "checkpoint",
          "completed_at": null
        }
      ],
      "child_issues": ["550e8400...", "551e8400..."],
      "progress_percent": 45,
      "created_at": "2024-01-08T08:00:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "meta": { ... }
}
```

#### GET /api/v1/epics/:id

Get epic details including full stage info and child issues.

#### POST /api/v1/epics

Create a new epic.

**Request:**
```json
{
  "title": "Payment Integration",
  "description": "Integrate Stripe for payments...",
  "stages": [
    { "name": "Design", "gate": "approval" },
    { "name": "Implementation", "gate": "review" },
    { "name": "Testing", "gate": "checkpoint" },
    { "name": "Documentation", "gate": null }
  ],
  "approvers": ["@team-lead"]
}
```

#### POST /api/v1/epics/:id/advance

Advance epic to next stage (requires gate approval).

**Request:**
```json
{
  "approved": true,
  "comment": "Design looks good, proceeding to implementation"
}
```

#### POST /api/v1/epics/:id/issues

Add an existing issue to the epic.

**Request:**
```json
{
  "issue_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### Agents

#### GET /api/v1/agents

List agents.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| status | string | Filter: `idle`, `running`, `paused`, `completed`, `failed` |
| type | string | Filter by agent type: `coder`, `reviewer`, `pm`, etc. |
| issue_id | uuid | Filter by assigned issue |

**Response:**
```json
{
  "data": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "agent_type": "coder",
      "status": "running",
      "issue_id": "550e8400-e29b-41d4-a716-446655440000",
      "issue_title": "Fix authentication bug",
      "started_at": "2024-01-15T09:00:00Z",
      "last_heartbeat": "2024-01-15T10:29:30Z",
      "runtime_seconds": 5400,
      "model": "claude-sonnet-4-20250514"
    }
  ]
}
```

#### GET /api/v1/agents/:id

Get agent details including recent output.

**Response:**
```json
{
  "data": {
    "id": "770e8400-e29b-41d4-a716-446655440000",
    "agent_type": "coder",
    "status": "running",
    "issue_id": "550e8400-e29b-41d4-a716-446655440000",
    "issue": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "github_number": 42,
      "title": "Fix authentication bug"
    },
    "worktree_path": "/repo/.dispatch-worktrees/550e8400",
    "started_at": "2024-01-15T09:00:00Z",
    "last_heartbeat": "2024-01-15T10:29:30Z",
    "runtime_seconds": 5400,
    "model": "claude-sonnet-4-20250514",
    "recent_output": [
      {
        "timestamp": "2024-01-15T10:29:00Z",
        "type": "tool_use",
        "content": {
          "tool": "Edit",
          "file": "src/auth.rs",
          "status": "success"
        }
      },
      {
        "timestamp": "2024-01-15T10:29:15Z",
        "type": "text",
        "content": "Fixed the token expiration check..."
      }
    ]
  }
}
```

#### POST /api/v1/agents

Start a new agent for an issue.

**Request:**
```json
{
  "issue_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_type": "coder",
  "model": "claude-sonnet-4-20250514"
}
```

**Response:** 201 Created

#### POST /api/v1/agents/:id/pause

Pause a running agent.

#### POST /api/v1/agents/:id/resume

Resume a paused agent.

#### POST /api/v1/agents/:id/cancel

Cancel an agent's task.

**Request:**
```json
{
  "reason": "Switching to different approach"
}
```

#### GET /api/v1/agents/:id/output

Stream agent output (SSE).

**Response:** `text/event-stream`
```
event: tool_use
data: {"tool": "Read", "file": "src/main.rs", "status": "success"}

event: text
data: {"content": "I'll fix the authentication issue..."}

event: heartbeat
data: {"timestamp": "2024-01-15T10:30:00Z"}
```

---

### Proposals

#### GET /api/v1/proposals

List proposals.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| status | string | Filter: `pending`, `approved`, `rejected`, `expired` |
| type | string | Filter: `implementation`, `architecture`, `security`, `refactor` |
| issue_id | uuid | Filter by related issue |

**Response:**
```json
{
  "data": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440000",
      "proposal_type": "implementation",
      "title": "Fix auth token expiration",
      "description": "Update the token validation logic...",
      "status": "pending",
      "issue_id": "550e8400-e29b-41d4-a716-446655440000",
      "created_by_agent": "770e8400-e29b-41d4-a716-446655440000",
      "consensus_threshold": "simple_majority",
      "votes": {
        "approve": 2,
        "reject": 0,
        "abstain": 1
      },
      "deadline": "2024-01-16T10:00:00Z",
      "created_at": "2024-01-15T10:00:00Z"
    }
  ]
}
```

#### GET /api/v1/proposals/:id

Get proposal details including all votes.

**Response:**
```json
{
  "data": {
    "id": "880e8400-e29b-41d4-a716-446655440000",
    "proposal_type": "implementation",
    "title": "Fix auth token expiration",
    "description": "Update the token validation logic...",
    "content": "Full proposal content...",
    "status": "pending",
    "issue_id": "550e8400-e29b-41d4-a716-446655440000",
    "created_by_agent": {
      "id": "770e8400-e29b-41d4-a716-446655440000",
      "agent_type": "coder"
    },
    "consensus_threshold": "simple_majority",
    "votes": [
      {
        "agent_id": "771e8400-e29b-41d4-a716-446655440000",
        "agent_type": "reviewer",
        "decision": "approve",
        "comment": "Implementation looks correct",
        "voted_at": "2024-01-15T11:00:00Z"
      },
      {
        "agent_id": "772e8400-e29b-41d4-a716-446655440000",
        "agent_type": "security",
        "decision": "approve",
        "comment": "No security concerns",
        "voted_at": "2024-01-15T11:30:00Z"
      }
    ],
    "deadline": "2024-01-16T10:00:00Z",
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

#### POST /api/v1/proposals/:id/vote

Cast a vote (human override).

**Request:**
```json
{
  "decision": "approve",
  "comment": "Approved by human review"
}
```

#### POST /api/v1/proposals/:id/force

Force approve/reject (human override).

**Request:**
```json
{
  "decision": "approve",
  "reason": "Urgent fix needed for production issue"
}
```

---

### Workflows

#### GET /api/v1/workflows

List active workflows.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| type | string | Filter: `tdd`, `design_review`, `iteration` |
| status | string | Filter: `in_progress`, `completed`, `escalated` |
| issue_id | uuid | Filter by related issue |

**Response:**
```json
{
  "data": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440000",
      "workflow_type": "tdd",
      "status": "in_progress",
      "current_phase": "Implementation",
      "iteration": 1,
      "max_iterations": 3,
      "issue_id": "550e8400-e29b-41d4-a716-446655440000",
      "coordinator_id": "773e8400-e29b-41d4-a716-446655440000",
      "started_at": "2024-01-15T08:00:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

#### GET /api/v1/workflows/:id

Get workflow details including phase history and feedback.

**Response:**
```json
{
  "data": {
    "id": "990e8400-e29b-41d4-a716-446655440000",
    "workflow_type": "tdd",
    "status": "in_progress",
    "current_phase": "Implementation",
    "iteration": 1,
    "max_iterations": 3,
    "phases": [
      {
        "phase": "Specification",
        "status": "completed",
        "started_at": "2024-01-15T08:00:00Z",
        "completed_at": "2024-01-15T08:30:00Z"
      },
      {
        "phase": "DesignReview",
        "status": "completed",
        "started_at": "2024-01-15T08:30:00Z",
        "completed_at": "2024-01-15T09:00:00Z"
      },
      {
        "phase": "WriteTests",
        "status": "completed",
        "started_at": "2024-01-15T09:00:00Z",
        "completed_at": "2024-01-15T09:45:00Z"
      },
      {
        "phase": "TestReview",
        "status": "completed",
        "started_at": "2024-01-15T09:45:00Z",
        "completed_at": "2024-01-15T10:00:00Z"
      },
      {
        "phase": "VerifyRed",
        "status": "completed",
        "started_at": "2024-01-15T10:00:00Z",
        "completed_at": "2024-01-15T10:05:00Z",
        "test_results": {
          "passed": 0,
          "failed": 5,
          "skipped": 0
        }
      },
      {
        "phase": "Implementation",
        "status": "in_progress",
        "started_at": "2024-01-15T10:05:00Z"
      }
    ],
    "feedback": [
      {
        "id": "fb-001",
        "from_agent": "reviewer-1",
        "to_agent": "coder-1",
        "type": "code_review",
        "severity": "minor",
        "content": "Consider extracting this to a helper function",
        "resolved": true
      }
    ],
    "coordinator": {
      "id": "773e8400-e29b-41d4-a716-446655440000",
      "agent_type": "coordinator"
    },
    "issue_id": "550e8400-e29b-41d4-a716-446655440000",
    "started_at": "2024-01-15T08:00:00Z"
  }
}
```

---

### Worktrees

#### GET /api/v1/worktrees

List worktrees.

**Response:**
```json
{
  "data": [
    {
      "id": "wt-550e8400",
      "path": "/repo/.dispatch-worktrees/550e8400",
      "branch": "dispatch/42/fix-auth-bug",
      "issue_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "active",
      "created_at": "2024-01-15T08:00:00Z",
      "last_commit": "abc123f",
      "last_commit_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

#### DELETE /api/v1/worktrees/:id

Remove a worktree.

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| force | boolean | Force removal even if uncommitted changes |

---

### GitHub Sync

#### POST /api/v1/github/sync

Trigger manual GitHub sync.

**Request:**
```json
{
  "full": false
}
```

**Response:**
```json
{
  "data": {
    "sync_id": "sync-123",
    "status": "started",
    "message": "Sync initiated"
  }
}
```

#### GET /api/v1/github/sync/:id

Get sync status.

**Response:**
```json
{
  "data": {
    "sync_id": "sync-123",
    "status": "completed",
    "issues_synced": 15,
    "issues_created": 2,
    "issues_updated": 5,
    "duration_ms": 1234,
    "completed_at": "2024-01-15T10:31:00Z"
  }
}
```

---

### Configuration

#### GET /api/v1/config

Get current configuration (sensitive values redacted).

**Response:**
```json
{
  "data": {
    "github": {
      "owner": "myorg",
      "repo": "myrepo",
      "sync_interval": 300
    },
    "agents": {
      "max_concurrent": 4,
      "default_model": "claude-sonnet-4-20250514"
    },
    "governance": {
      "enabled": true,
      "voting_deadline_hours": 24,
      "max_iterations": 3
    }
  }
}
```

#### PATCH /api/v1/config

Update runtime configuration.

**Request:**
```json
{
  "agents.max_concurrent": 8,
  "governance.voting_deadline_hours": 48
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_REQUEST` | 400 | Malformed request body |
| `VALIDATION_ERROR` | 400 | Field validation failed |
| `UNAUTHORIZED` | 401 | Missing or invalid auth |
| `FORBIDDEN` | 403 | Permission denied |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | State conflict (e.g., agent already running) |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |
| `GITHUB_ERROR` | 502 | GitHub API error |

**Error Response Example:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid priority value",
    "details": {
      "field": "priority",
      "value": "urgent",
      "allowed": ["critical", "high", "medium", "low"]
    }
  }
}
```

---

## Key Data Structures

```rust
// dispatch-server/src/api/mod.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

/// Standard API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Standard API error
#[derive(Serialize)]
pub struct ApiError {
    pub error: ApiErrorBody,
}

#[derive(Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.error.code.as_str() {
            "INVALID_REQUEST" | "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "CONFLICT" => StatusCode::CONFLICT,
            "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
            "GITHUB_ERROR" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<DispatchError> for ApiError {
    fn from(err: DispatchError) -> Self {
        ApiError {
            error: ApiErrorBody {
                code: err.code().to_string(),
                message: err.to_string(),
                details: err.details(),
            },
        }
    }
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    pub fn limit(&self) -> u32 {
        self.per_page.min(100)
    }
}
```

---

## Route Handlers

```rust
// dispatch-server/src/api/issues.rs

pub async fn list_issues(
    State(state): State<AppState>,
    Query(params): Query<IssueQueryParams>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ApiResponse<Vec<IssueDto>>>, ApiError> {
    let issues = state.db.issues()
        .list(
            params.status,
            params.priority,
            params.epic_id,
            params.assigned,
            params.search.as_deref(),
            pagination.offset(),
            pagination.limit(),
        )
        .await
        .map_err(ApiError::from)?;

    let total = state.db.issues()
        .count(params.status, params.priority, params.epic_id, params.assigned)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ApiResponse {
        data: issues.into_iter().map(IssueDto::from).collect(),
        meta: Some(PaginationMeta {
            page: pagination.page,
            per_page: pagination.limit(),
            total,
            total_pages: (total + pagination.limit() - 1) / pagination.limit(),
        }),
    }))
}

pub async fn get_issue(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<IssueDetailDto>>, ApiError> {
    let issue = state.db.issues()
        .get(&IssueId(id))
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("Issue not found"))?;

    let epic = if let Some(epic_id) = &issue.epic_id {
        state.db.epics().get(epic_id).await.ok().flatten()
    } else {
        None
    };

    let agent = if let Some(agent_id) = &issue.assigned_agent_id {
        state.db.agents().get(agent_id).await.ok().flatten()
    } else {
        None
    };

    let proposals = state.db.proposals()
        .list_by_issue(&issue.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ApiResponse {
        data: IssueDetailDto::from_parts(issue, epic, agent, proposals),
        meta: None,
    }))
}

pub async fn create_issue(
    State(state): State<AppState>,
    Json(req): Json<CreateIssueRequest>,
) -> Result<(StatusCode, Json<ApiResponse<IssueDto>>), ApiError> {
    // Validate request
    req.validate().map_err(|e| ApiError::validation_error(e))?;

    // Create in database
    let issue = state.db.issues()
        .create(&req.title, &req.description, req.priority, req.issue_type)
        .await
        .map_err(ApiError::from)?;

    // Create on GitHub
    let github_issue = state.github
        .create_issue(&issue)
        .await
        .map_err(ApiError::from)?;

    // Update with GitHub info
    let issue = state.db.issues()
        .update_github_info(&issue.id, github_issue.number)
        .await
        .map_err(ApiError::from)?;

    // Publish event
    state.events.issue_created(&issue);

    Ok((StatusCode::CREATED, Json(ApiResponse {
        data: IssueDto::from(issue),
        meta: None,
    })))
}
```

---

## Router Configuration

```rust
// dispatch-server/src/routes.rs

pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health::health_check))

        // Issues
        .route("/api/v1/issues", get(issues::list_issues).post(issues::create_issue))
        .route("/api/v1/issues/:id", get(issues::get_issue).patch(issues::update_issue).delete(issues::delete_issue))

        // Epics
        .route("/api/v1/epics", get(epics::list_epics).post(epics::create_epic))
        .route("/api/v1/epics/:id", get(epics::get_epic).patch(epics::update_epic))
        .route("/api/v1/epics/:id/advance", post(epics::advance_stage))
        .route("/api/v1/epics/:id/issues", post(epics::add_issue))

        // Agents
        .route("/api/v1/agents", get(agents::list_agents).post(agents::start_agent))
        .route("/api/v1/agents/:id", get(agents::get_agent))
        .route("/api/v1/agents/:id/pause", post(agents::pause_agent))
        .route("/api/v1/agents/:id/resume", post(agents::resume_agent))
        .route("/api/v1/agents/:id/cancel", post(agents::cancel_agent))
        .route("/api/v1/agents/:id/output", get(agents::stream_output))

        // Proposals
        .route("/api/v1/proposals", get(proposals::list_proposals))
        .route("/api/v1/proposals/:id", get(proposals::get_proposal))
        .route("/api/v1/proposals/:id/vote", post(proposals::vote))
        .route("/api/v1/proposals/:id/force", post(proposals::force_decision))

        // Workflows
        .route("/api/v1/workflows", get(workflows::list_workflows))
        .route("/api/v1/workflows/:id", get(workflows::get_workflow))

        // Worktrees
        .route("/api/v1/worktrees", get(worktrees::list_worktrees))
        .route("/api/v1/worktrees/:id", delete(worktrees::delete_worktree))

        // GitHub
        .route("/api/v1/github/sync", post(github::trigger_sync))
        .route("/api/v1/github/sync/:id", get(github::get_sync_status))

        // Config
        .route("/api/v1/config", get(config::get_config).patch(config::update_config))

        // Status
        .route("/api/v1/status", get(status::get_status))

        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
```

---

## OpenAPI/Swagger

Generate OpenAPI spec using `utoipa`:

```rust
// dispatch-server/src/api/openapi.rs

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        issues::list_issues,
        issues::get_issue,
        issues::create_issue,
        issues::update_issue,
        issues::delete_issue,
        // ... more paths
    ),
    components(
        schemas(
            IssueDto,
            IssueDetailDto,
            CreateIssueRequest,
            // ... more schemas
        )
    ),
    tags(
        (name = "issues", description = "Issue management"),
        (name = "epics", description = "Epic management"),
        (name = "agents", description = "Agent management"),
        (name = "proposals", description = "Governance proposals"),
    )
)]
pub struct ApiDoc;

// Serve at /api/docs
pub fn openapi_routes() -> Router {
    Router::new()
        .route("/api/docs/openapi.json", get(|| async {
            Json(ApiDoc::openapi())
        }))
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
}
```

---

## Rate Limiting

```rust
// dispatch-server/src/middleware/rate_limit.rs

use tower_governor::{GovernorLayer, GovernorConfigBuilder};

pub fn rate_limit_layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    let config = GovernorConfigBuilder::default()
        .per_second(10)  // 10 requests per second
        .burst_size(50)  // Allow bursts up to 50
        .finish()
        .unwrap();

    GovernorLayer::new(&config)
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-009 | REST API infrastructure | `dispatch-server/src/api/mod.rs` |
| PR-009a | Issue endpoints | `dispatch-server/src/api/issues.rs` |
| PR-009b | Epic endpoints | `dispatch-server/src/api/epics.rs` |
| PR-009c | Agent endpoints | `dispatch-server/src/api/agents.rs` |
| PR-009d | Proposal endpoints | `dispatch-server/src/api/proposals.rs` |
| PR-009e | Workflow endpoints | `dispatch-server/src/api/workflows.rs` |
| PR-009f | OpenAPI documentation | `dispatch-server/src/api/openapi.rs` |
