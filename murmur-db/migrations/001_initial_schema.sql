-- Initial schema for murmur-db
-- Tables for tracking GitHub issues, agent runs, and conversations

-- Issues table: Track GitHub issue state
CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    github_issue_number INTEGER NOT NULL,
    repository TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL, -- 'open', 'closed'
    status TEXT, -- Custom status from metadata: 'ready', 'blocked', 'in_progress', 'complete'
    phase TEXT, -- Phase identifier (e.g., '3.5', '4')
    pr_number TEXT, -- Associated PR number
    labels TEXT, -- JSON array of labels
    assignees TEXT, -- JSON array of assignees
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    closed_at DATETIME,

    -- Ensure unique issues per repository
    UNIQUE(repository, github_issue_number)
);

-- Issue status history: Track changes to issue status over time
CREATE TABLE IF NOT EXISTS issue_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL,
    old_status TEXT,
    new_status TEXT NOT NULL,
    changed_at DATETIME NOT NULL,
    reason TEXT,

    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
);

-- Agent runs table: Track agent execution history
CREATE TABLE IF NOT EXISTS agent_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER,
    agent_type TEXT NOT NULL, -- 'coder', 'reviewer', 'test', 'coordinator'
    worktree_path TEXT,
    command TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    exit_code INTEGER,
    duration_ms INTEGER,
    config TEXT, -- JSON configuration used for this run

    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE SET NULL
);

-- Conversation logs table: Store agent conversation output
CREATE TABLE IF NOT EXISTS conversation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_run_id INTEGER NOT NULL,
    sequence_number INTEGER NOT NULL,
    timestamp DATETIME NOT NULL,
    message_type TEXT NOT NULL, -- 'assistant', 'tool_use', 'tool_result', 'error'
    content TEXT NOT NULL, -- JSON content

    FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_issues_repo_number ON issues(repository, github_issue_number);
CREATE INDEX IF NOT EXISTS idx_issues_state ON issues(state);
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
CREATE INDEX IF NOT EXISTS idx_issue_status_history_issue_id ON issue_status_history(issue_id);
CREATE INDEX IF NOT EXISTS idx_agent_runs_issue_id ON agent_runs(issue_id);
CREATE INDEX IF NOT EXISTS idx_agent_runs_start_time ON agent_runs(start_time);
CREATE INDEX IF NOT EXISTS idx_conversation_logs_agent_run_id ON conversation_logs(agent_run_id);
