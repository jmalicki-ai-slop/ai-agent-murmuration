# Database Schema Design

## Overview

SQLite database schema for the Dispatch system. Uses sqlx for compile-time checked queries and migrations.

---

## Database File

```
~/.local/share/dispatch/dispatch.db
```

Or configurable via `config.toml`:
```toml
[database]
path = "/custom/path/dispatch.db"
```

---

## Schema Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    epics    │────<│   stages    │────<│    gates    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │
       │                   │
       ▼                   ▼
┌─────────────┐     ┌─────────────┐
│   issues    │────>│   agents    │
└─────────────┘     └─────────────┘
       │                   │
       │                   │
       ▼                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ pull_requests│    │  proposals  │────<│    votes    │
└─────────────┘     └─────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │  decisions  │
                    └─────────────┘
```

---

## Tables

### epics

```sql
CREATE TABLE epics (
    id TEXT PRIMARY KEY,                    -- UUID
    github_id INTEGER,                      -- GitHub issue number (nullable)
    github_url TEXT,                        -- Full GitHub URL

    -- Content
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    acceptance_criteria TEXT,               -- JSON array of strings

    -- Repository
    repo_path TEXT NOT NULL,                -- Local path
    repo_url TEXT,                          -- GitHub URL

    -- Status
    status TEXT NOT NULL DEFAULT 'draft',   -- draft, ready, in_progress, awaiting_gate, completed, cancelled
    current_stage_id TEXT,                  -- FK to stages
    blocked_at_gate_id TEXT,                -- FK to gates (if waiting)

    -- Timestamps
    created_at TEXT NOT NULL,               -- ISO 8601
    updated_at TEXT NOT NULL,
    completed_at TEXT,

    FOREIGN KEY (current_stage_id) REFERENCES stages(id),
    FOREIGN KEY (blocked_at_gate_id) REFERENCES gates(id)
);

CREATE INDEX idx_epics_status ON epics(status);
CREATE INDEX idx_epics_repo ON epics(repo_path);
CREATE INDEX idx_epics_github ON epics(github_id);
```

### stages

```sql
CREATE TABLE stages (
    id TEXT PRIMARY KEY,                    -- UUID
    epic_id TEXT NOT NULL,                  -- FK to epics

    -- Content
    name TEXT NOT NULL,                     -- "Design", "Implementation", etc.
    description TEXT,
    stage_order INTEGER NOT NULL,           -- Order within epic (0, 1, 2, ...)

    -- Status
    status TEXT NOT NULL DEFAULT 'pending', -- pending, in_progress, awaiting_gate, approved, skipped

    -- Timestamps
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,

    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE
);

CREATE INDEX idx_stages_epic ON stages(epic_id);
CREATE INDEX idx_stages_order ON stages(epic_id, stage_order);
```

### gates

```sql
CREATE TABLE gates (
    id TEXT PRIMARY KEY,                    -- UUID
    stage_id TEXT NOT NULL,                 -- FK to stages

    -- Content
    gate_type TEXT NOT NULL,                -- approval, review, checkpoint, decision
    description TEXT NOT NULL,
    required_approvers TEXT,                -- JSON array of GitHub usernames, or "any"

    -- Status
    status TEXT NOT NULL DEFAULT 'pending', -- pending, approved, rejected, skipped
    approved_by TEXT,                       -- GitHub username
    rejection_reason TEXT,

    -- Timestamps
    created_at TEXT NOT NULL,
    approved_at TEXT,

    FOREIGN KEY (stage_id) REFERENCES stages(id) ON DELETE CASCADE
);

CREATE INDEX idx_gates_stage ON gates(stage_id);
CREATE INDEX idx_gates_status ON gates(status);
```

### gate_comments

```sql
CREATE TABLE gate_comments (
    id TEXT PRIMARY KEY,                    -- UUID
    gate_id TEXT NOT NULL,                  -- FK to gates

    author TEXT NOT NULL,                   -- GitHub username
    content TEXT NOT NULL,

    created_at TEXT NOT NULL,

    FOREIGN KEY (gate_id) REFERENCES gates(id) ON DELETE CASCADE
);

CREATE INDEX idx_gate_comments_gate ON gate_comments(gate_id);
```

### issues

```sql
CREATE TABLE issues (
    id TEXT PRIMARY KEY,                    -- UUID
    github_id INTEGER,                      -- GitHub issue number (nullable)
    github_url TEXT,

    -- Epic relationship
    epic_id TEXT,                           -- FK to epics (nullable for standalone issues)
    stage_id TEXT,                          -- FK to stages (nullable)

    -- Repository
    repo_path TEXT NOT NULL,
    repo_url TEXT,
    worktree_path TEXT,                     -- Created when assigned
    branch_name TEXT,

    -- Content
    title TEXT NOT NULL,
    prompt TEXT NOT NULL,                   -- Full description, serves as agent memory
    issue_type TEXT NOT NULL,               -- feature, bug, docs, refactor, test, security, chore
    priority TEXT NOT NULL DEFAULT 'medium', -- critical, high, medium, low
    labels TEXT,                            -- JSON array of strings

    -- Assignment
    status TEXT NOT NULL DEFAULT 'unassigned', -- unassigned, queued, assigned, in_progress, awaiting_review, in_review, done, blocked, cancelled
    assigned_agent_id TEXT,                 -- FK to agents
    agent_type TEXT,                        -- coder, reviewer, pm, security, docs, test, architect

    -- Timestamps
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    assigned_at TEXT,
    completed_at TEXT,

    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE SET NULL,
    FOREIGN KEY (stage_id) REFERENCES stages(id) ON DELETE SET NULL,
    FOREIGN KEY (assigned_agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_epic ON issues(epic_id);
CREATE INDEX idx_issues_stage ON issues(stage_id);
CREATE INDEX idx_issues_agent ON issues(assigned_agent_id);
CREATE INDEX idx_issues_github ON issues(github_id);
CREATE INDEX idx_issues_repo ON issues(repo_path);
```

### pull_requests

```sql
CREATE TABLE pull_requests (
    id TEXT PRIMARY KEY,                    -- UUID
    issue_id TEXT NOT NULL,                 -- FK to issues

    github_number INTEGER NOT NULL,
    github_url TEXT NOT NULL,
    branch TEXT NOT NULL,

    status TEXT NOT NULL DEFAULT 'open',    -- draft, open, merged, closed
    checks_passing INTEGER DEFAULT 0,       -- boolean
    review_status TEXT DEFAULT 'pending',   -- pending, approved, changes_requested, dismissed

    created_at TEXT NOT NULL,
    merged_at TEXT,

    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

CREATE INDEX idx_prs_issue ON pull_requests(issue_id);
CREATE INDEX idx_prs_status ON pull_requests(status);
```

### agents

```sql
CREATE TABLE agents (
    id TEXT PRIMARY KEY,                    -- UUID

    agent_type TEXT NOT NULL,               -- coder, reviewer, pm, security, docs, test, architect
    status TEXT NOT NULL DEFAULT 'idle',    -- idle, starting, working, waiting_for_input, waiting_for_vote, paused, errored, completed

    -- Current work
    current_issue_id TEXT,                  -- FK to issues
    worktree_path TEXT,
    process_id INTEGER,                     -- OS PID
    claude_session_id TEXT,                 -- For resume capability

    -- Timestamps
    started_at TEXT NOT NULL,
    last_heartbeat TEXT NOT NULL,
    completed_at TEXT,

    -- Metrics (JSON for flexibility)
    metrics TEXT,                           -- JSON: {issues_completed, avg_completion_time, tokens_used, ...}

    FOREIGN KEY (current_issue_id) REFERENCES issues(id) ON DELETE SET NULL
);

CREATE INDEX idx_agents_type ON agents(agent_type);
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_issue ON agents(current_issue_id);
```

### proposals

```sql
CREATE TABLE proposals (
    id TEXT PRIMARY KEY,                    -- UUID

    proposal_type TEXT NOT NULL,            -- implementation_approach, tech_stack_choice, architecture_decision, new_agent_type, workflow_change, governance_rule, tool_integration, prompt_improvement
    proposer_id TEXT NOT NULL,              -- FK to agents

    -- Content
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,

    -- Context
    related_issue_id TEXT,                  -- FK to issues (if implementation proposal)
    affected_components TEXT,               -- JSON array of component names

    -- Options (for decisions)
    options TEXT,                           -- JSON array of options
    chosen_option TEXT,                     -- Selected option after vote

    -- Voting
    status TEXT NOT NULL DEFAULT 'open',    -- open, voting, approved, rejected, executing, executed, rolled_back, vetoed
    required_voters TEXT,                   -- JSON array of agent types
    threshold TEXT NOT NULL,                -- unanimous, super_majority, simple_majority, single_approval

    -- Execution
    implementation_plan TEXT,
    rollback_plan TEXT,
    execution_result TEXT,                  -- JSON with outcome details

    -- Human override
    forced_by TEXT,                         -- Human who forced decision (nullable)
    force_reason TEXT,
    vetoed_by TEXT,                         -- Human who vetoed (nullable)
    veto_reason TEXT,

    -- Timestamps
    created_at TEXT NOT NULL,
    voting_deadline TEXT,
    resolved_at TEXT,
    executed_at TEXT,

    FOREIGN KEY (proposer_id) REFERENCES agents(id),
    FOREIGN KEY (related_issue_id) REFERENCES issues(id) ON DELETE SET NULL
);

CREATE INDEX idx_proposals_status ON proposals(status);
CREATE INDEX idx_proposals_type ON proposals(proposal_type);
CREATE INDEX idx_proposals_issue ON proposals(related_issue_id);
```

### votes

```sql
CREATE TABLE votes (
    id TEXT PRIMARY KEY,                    -- UUID
    proposal_id TEXT NOT NULL,              -- FK to proposals

    voter_id TEXT NOT NULL,                 -- FK to agents
    voter_type TEXT NOT NULL,               -- Agent type at time of vote

    decision TEXT NOT NULL,                 -- approve, reject, abstain, need_more_info
    reasoning TEXT NOT NULL,
    confidence REAL,                        -- 0.0 - 1.0

    created_at TEXT NOT NULL,

    FOREIGN KEY (proposal_id) REFERENCES proposals(id) ON DELETE CASCADE,
    FOREIGN KEY (voter_id) REFERENCES agents(id)
);

CREATE INDEX idx_votes_proposal ON votes(proposal_id);
CREATE INDEX idx_votes_voter ON votes(voter_id);
CREATE UNIQUE INDEX idx_votes_unique ON votes(proposal_id, voter_id);
```

### decisions

```sql
CREATE TABLE decisions (
    id TEXT PRIMARY KEY,                    -- UUID

    -- What was decided
    proposal_id TEXT,                       -- FK to proposals (nullable for non-proposal decisions)
    issue_id TEXT,                          -- FK to issues (nullable)
    epic_id TEXT,                           -- FK to epics (nullable)

    decision_type TEXT NOT NULL,            -- proposal_approved, proposal_rejected, human_override, human_veto, gate_approved, gate_rejected
    description TEXT NOT NULL,

    -- Outcome
    outcome TEXT NOT NULL,                  -- JSON with decision details

    -- Who made it
    decided_by TEXT NOT NULL,               -- "sangha", "human:username", "agent:id"

    created_at TEXT NOT NULL,

    FOREIGN KEY (proposal_id) REFERENCES proposals(id) ON DELETE SET NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE SET NULL,
    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE SET NULL
);

CREATE INDEX idx_decisions_proposal ON decisions(proposal_id);
CREATE INDEX idx_decisions_issue ON decisions(issue_id);
CREATE INDEX idx_decisions_epic ON decisions(epic_id);
CREATE INDEX idx_decisions_type ON decisions(decision_type);
CREATE INDEX idx_decisions_date ON decisions(created_at);
```

### agent_logs

```sql
CREATE TABLE agent_logs (
    id TEXT PRIMARY KEY,                    -- UUID
    agent_id TEXT NOT NULL,                 -- FK to agents
    issue_id TEXT,                          -- FK to issues

    level TEXT NOT NULL,                    -- debug, info, warn, error
    message TEXT NOT NULL,
    context TEXT,                           -- JSON with additional context

    created_at TEXT NOT NULL,

    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE SET NULL
);

CREATE INDEX idx_logs_agent ON agent_logs(agent_id);
CREATE INDEX idx_logs_issue ON agent_logs(issue_id);
CREATE INDEX idx_logs_level ON agent_logs(level);
CREATE INDEX idx_logs_date ON agent_logs(created_at);
```

### config_store

```sql
-- Key-value store for runtime configuration
CREATE TABLE config_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### sync_state

```sql
-- Track GitHub sync state
CREATE TABLE sync_state (
    id TEXT PRIMARY KEY,                    -- Usually just "github"
    last_sync_at TEXT NOT NULL,
    last_issue_id INTEGER,                  -- Last processed GitHub issue ID
    last_pr_id INTEGER,                     -- Last processed GitHub PR ID
    etag TEXT,                              -- For conditional requests
    sync_errors TEXT                        -- JSON array of recent errors
);
```

---

## Migrations

### Migration 001: Initial Schema

```sql
-- migrations/001_initial_schema.sql

-- Core tables
CREATE TABLE issues ( ... );
CREATE TABLE agents ( ... );
CREATE TABLE pull_requests ( ... );
CREATE TABLE agent_logs ( ... );
CREATE TABLE config_store ( ... );
CREATE TABLE sync_state ( ... );

-- Indexes
CREATE INDEX ...;
```

### Migration 002: Add Epics

```sql
-- migrations/002_add_epics.sql

CREATE TABLE epics ( ... );
CREATE TABLE stages ( ... );
CREATE TABLE gates ( ... );
CREATE TABLE gate_comments ( ... );

-- Add FK columns to issues
ALTER TABLE issues ADD COLUMN epic_id TEXT REFERENCES epics(id);
ALTER TABLE issues ADD COLUMN stage_id TEXT REFERENCES stages(id);
```

### Migration 003: Add Governance

```sql
-- migrations/003_add_governance.sql

CREATE TABLE proposals ( ... );
CREATE TABLE votes ( ... );
CREATE TABLE decisions ( ... );
```

---

## sqlx Usage

### Compile-Time Checked Queries

```rust
// In dispatch-db/src/repos/issue.rs

use sqlx::FromRow;
use dispatch_core::types::issue::{Issue, IssueId, IssueStatus};

#[derive(FromRow)]
struct IssueRow {
    id: String,
    github_id: Option<i64>,
    title: String,
    prompt: String,
    status: String,
    // ... other fields
}

impl IssueRepository {
    pub async fn get(&self, id: &IssueId) -> Result<Option<Issue>> {
        let row = sqlx::query_as!(
            IssueRow,
            r#"
            SELECT id, github_id, title, prompt, status, ...
            FROM issues
            WHERE id = ?
            "#,
            id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_by_status(&self, status: IssueStatus) -> Result<Vec<Issue>> {
        let rows = sqlx::query_as!(
            IssueRow,
            r#"
            SELECT id, github_id, title, prompt, status, ...
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

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}
```

### Connection Pool

```rust
// In dispatch-db/src/pool.rs

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-003 | SQLite schema + migrations | `migrations/*.sql`, `dispatch-db/src/pool.rs`, `dispatch-db/src/migrations.rs` |
| PR-005 | Database CRUD operations | `dispatch-db/src/repos/*.rs` |
