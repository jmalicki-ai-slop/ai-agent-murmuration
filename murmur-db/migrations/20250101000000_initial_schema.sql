-- Initial schema for Murmuration database
-- Phase 3.5: Persistence (SQLite)

-- GitHub issues table
-- Stores issue metadata and state from GitHub
CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL CHECK(state IN ('open', 'closed')),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    closed_at DATETIME,
    assignee TEXT,
    labels TEXT NOT NULL DEFAULT '[]', -- JSON array of label names
    phase INTEGER, -- Phase number from metadata
    pr_number TEXT, -- PR identifier (e.g., "PR-023")
    parent_issue INTEGER, -- Reference to parent epic issue
    metadata TEXT, -- JSON object with additional metadata
    UNIQUE(number)
);

CREATE INDEX idx_issues_state ON issues(state);
CREATE INDEX idx_issues_phase ON issues(phase);
CREATE INDEX idx_issues_parent ON issues(parent_issue);
CREATE INDEX idx_issues_updated ON issues(updated_at DESC);

-- Agent runs table
-- Tracks each time an agent is spawned to work on a task
CREATE TABLE IF NOT EXISTS agent_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER, -- NULL for ad-hoc runs
    agent_type TEXT NOT NULL, -- 'coder', 'reviewer', 'test', etc.
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'interrupted')),
    worktree_path TEXT,
    branch_name TEXT,
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    exit_code INTEGER,
    error_message TEXT,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE SET NULL
);

CREATE INDEX idx_agent_runs_issue ON agent_runs(issue_id);
CREATE INDEX idx_agent_runs_status ON agent_runs(status);
CREATE INDEX idx_agent_runs_started ON agent_runs(started_at DESC);

-- Conversation messages table
-- Stores the full conversation transcript for each agent run
CREATE TABLE IF NOT EXISTS conversation_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_run_id INTEGER NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    tool_use TEXT, -- JSON object for tool use details
    tool_result TEXT, -- JSON object for tool result details
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tokens_input INTEGER,
    tokens_output INTEGER,
    cost_usd REAL,
    FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
);

CREATE INDEX idx_conversation_agent_run ON conversation_messages(agent_run_id);
CREATE INDEX idx_conversation_timestamp ON conversation_messages(timestamp);
