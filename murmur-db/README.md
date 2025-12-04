# murmur-db

Database persistence layer for Murmuration using SQLite.

## Overview

This crate provides SQLite-based persistence for:
- **Agent run history**: Track execution history for debugging and analysis
- Issue state tracking (planned)
- Conversation logs (planned)

The database is stored at `~/.cache/murmur/murmur.db` by default.

## Features

### Agent Run History (PR-025)

Track agent execution with the following capabilities:

- **AgentRun Model**: Records agent type, exit code, duration, start/end times
- **Configuration Storage**: Stores agent configuration as JSON for reproducibility
- **Query Support**: Find runs by issue number, date range, or agent type
- **Statistics**: Count runs by issue or globally

## Usage

```rust
use murmur_db::{Database, models::AgentRun, repos::AgentRunRepository};

// Open database at default location
let db = Database::open()?;

// Create a repository
let repo = AgentRunRepository::new(&db);

// Create a new agent run record
let mut run = AgentRun::new(
    "implementer",
    "Fix bug #123",
    "/path/to/worktree",
    r#"{"model":"sonnet","claude_path":"claude"}"#
).with_issue_number(123);

// Insert the record
let id = repo.insert(&run)?;

// Mark as completed
run.id = Some(id);
run.complete(0); // exit code 0
repo.update(&run)?;

// Query by issue
let runs = repo.find_by_issue(123)?;

// Query by date range
let runs = repo.find_by_date_range(start_date, end_date)?;

// Query by agent type
let runs = repo.find_by_agent_type("implementer")?;
```

## Database Schema

### `agent_runs` Table

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key (auto-increment) |
| `agent_type` | TEXT | Type of agent (e.g., "planner", "implementer") |
| `issue_number` | INTEGER | GitHub issue number (nullable) |
| `prompt` | TEXT | The prompt given to the agent |
| `workdir` | TEXT | Working directory path |
| `config_json` | TEXT | Agent configuration as JSON |
| `start_time` | TEXT | ISO 8601 timestamp |
| `end_time` | TEXT | ISO 8601 timestamp (nullable) |
| `exit_code` | INTEGER | Process exit code (nullable) |
| `duration_seconds` | REAL | Duration in seconds (nullable) |
| `created_at` | TEXT | Record creation timestamp |

Indexes:
- `idx_agent_runs_issue` on `issue_number`
- `idx_agent_runs_start_time` on `start_time`
- `idx_agent_runs_agent_type` on `agent_type`

## Testing

```bash
# Run tests for this crate
cargo test -p murmur-db

# Run all tests
cargo test
```

All tests use in-memory databases for fast, isolated execution.

## Future Work

- PR-024: Issue state persistence (`repos/issues.rs`)
- PR-026: Conversation log storage (`repos/conversations.rs`)
- PR-027: Resume interrupted workflows integration
