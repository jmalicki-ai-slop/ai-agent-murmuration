-- Initial schema for Murmuration database

-- Agent runs track each execution of an agent
CREATE TABLE agent_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,           -- Issue number or task identifier
    agent_type TEXT NOT NULL,         -- Type of agent (coder, reviewer, etc.)
    worktree_path TEXT NOT NULL,      -- Path to the worktree
    started_at TEXT NOT NULL,         -- ISO 8601 timestamp
    completed_at TEXT,                -- ISO 8601 timestamp (NULL if incomplete)
    status TEXT NOT NULL,             -- 'running', 'completed', 'failed', 'interrupted'
    exit_code INTEGER,                -- Process exit code if completed
    prompt TEXT NOT NULL,             -- Initial prompt given to agent
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Conversation logs store the message stream from agents
CREATE TABLE conversation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,          -- Foreign key to agent_runs.id
    sequence INTEGER NOT NULL,        -- Order of messages in conversation
    timestamp TEXT NOT NULL,          -- ISO 8601 timestamp
    message_type TEXT NOT NULL,       -- 'system', 'assistant', 'tool_use', 'tool_result', 'result'
    content TEXT NOT NULL,            -- JSON-serialized message content
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (run_id) REFERENCES agent_runs(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX idx_agent_runs_task_id ON agent_runs(task_id);
CREATE INDEX idx_agent_runs_status ON agent_runs(status);
CREATE INDEX idx_conversation_logs_run_id ON conversation_logs(run_id);
CREATE INDEX idx_conversation_logs_sequence ON conversation_logs(run_id, sequence);
