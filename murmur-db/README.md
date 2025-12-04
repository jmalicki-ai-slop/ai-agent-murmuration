# murmur-db

SQLite database layer for Murmuration persistence.

## Overview

This crate provides database schema and connection management for storing:
- GitHub issue state and metadata
- Agent run history
- Conversation transcripts and logs
- Workflow state for resumption

## Database Location

By default, the database is stored at:
```
~/.cache/murmur/murmur.db
```

## Schema

### Tables

#### `issues`
Stores GitHub issue metadata:
- Issue number, title, body, state
- Labels, assignee, timestamps
- Phase and PR metadata
- Parent issue references

#### `agent_runs`
Tracks each agent execution:
- Agent type (coder, reviewer, test)
- Associated issue
- Worktree path and branch
- Status, timestamps, exit codes

#### `conversation_messages`
Stores full conversation transcripts:
- Role (user, assistant, system)
- Message content
- Tool use and results (JSON)
- Token counts and cost tracking

## Usage

```rust
use murmur_db::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database at default location
    let db = Database::new().await?;

    // Run migrations
    db.migrate().await?;

    // Use the connection pool
    let pool = db.pool();

    // Query example
    let issues = sqlx::query_as::<_, murmur_db::schema::Issue>(
        "SELECT * FROM issues WHERE state = 'open'"
    )
    .fetch_all(pool)
    .await?;

    Ok(())
}
```

## Migrations

Migrations are stored in `migrations/` and are automatically applied when calling `Database::migrate()`.

Current migrations:
- `20250101000000_initial_schema.sql` - Initial tables for issues, agents, and conversations

## Development

Run tests:
```bash
cargo test -p murmur-db
```

Build:
```bash
cargo build -p murmur-db
```
