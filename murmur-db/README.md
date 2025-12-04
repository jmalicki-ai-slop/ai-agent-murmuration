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

### Basic Database Setup

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

### Conversation Storage (PR-026)

Store and query conversation logs from agent runs:

```rust
use murmur_db::{Database, repos::ConversationRepository};
use murmur_db::schema::MessageRole;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new().await?;
    db.migrate().await?;

    let repo = ConversationRepository::new(db.pool());

    // Store a conversation message
    let message_id = repo.create(
        agent_run_id,
        MessageRole::Assistant,
        "I'll help you with that task."
    ).await?;

    // Store a message with tool use
    let tool_use = serde_json::json!({
        "tool": "Read",
        "input": {"file_path": "/test.txt"}
    });
    repo.create_with_tool_use(
        agent_run_id,
        MessageRole::Assistant,
        "Reading the file...",
        tool_use
    ).await?;

    // Store a message with cost tracking
    repo.create_with_cost(
        agent_run_id,
        MessageRole::Assistant,
        "Task completed.",
        Some(1500),  // input tokens
        Some(200),   // output tokens
        Some(0.005)  // cost in USD
    ).await?;

    // Retrieve all messages for an agent run
    let messages = repo.get_by_agent_run(agent_run_id).await?;

    // Get token usage statistics
    let (input_tokens, output_tokens) = repo.get_token_usage(agent_run_id).await?;
    let total_cost = repo.get_total_cost(agent_run_id).await?;

    // Search messages
    let results = repo.search(Some(agent_run_id), "error").await?;

    Ok(())
}
```

### Streaming Conversation Capture

Capture conversations in real-time during agent execution:

```rust
use murmur_db::{Database, repos::DatabaseStreamHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new().await?;
    db.migrate().await?;

    let agent_run_id = 123; // ID of the agent run
    let mut handler = DatabaseStreamHandler::new(db.pool(), agent_run_id, true);

    // As the agent streams output, store it
    handler.store_assistant_text("Let me help with that.").await?;
    handler.store_tool_use("Read", &serde_json::json!({"file": "test.txt"})).await?;
    handler.store_tool_result("File contents here", false).await?;

    // Store completion with cost info
    let cost = murmur_db::repos::CostInfo {
        input_tokens: 1000,
        output_tokens: 500,
        cache_read_tokens: None,
        cache_write_tokens: None,
        cost_usd: Some(0.003),
    };
    handler.store_complete(Some(&cost), Some(1234)).await?;

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
