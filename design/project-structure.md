# Project Structure Design

## Overview

This document defines the Rust workspace structure for the Dispatch system.

---

## Workspace Layout

```
dispatch/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml                # Build, test, lint
│       └── release.yml           # Binary releases
├── config/
│   └── default.toml              # Default configuration
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_add_epics.sql
│   └── ...
├── prompts/                      # Agent system prompts
│   ├── coder.md
│   ├── reviewer.md
│   ├── pm.md
│   ├── security.md
│   ├── docs.md
│   ├── test.md
│   └── architect.md
├── crates/
│   ├── dispatch-core/            # Core library
│   ├── dispatch-cli/             # CLI binary
│   ├── dispatch-db/              # Database layer
│   ├── dispatch-github/          # GitHub integration
│   ├── dispatch-agents/          # Agent execution
│   ├── dispatch-governance/      # Sangha voting
│   ├── dispatch-git/             # Git/worktree operations
│   ├── dispatch-tui/             # Terminal UI
│   ├── dispatch-web/             # Web server & WebSocket
│   └── dispatch-pty/             # PTY handling (Phase 11)
└── web/                          # Frontend (Phase 10)
    ├── package.json
    └── src/
```

---

## Crate Dependency Graph

```
                    ┌─────────────────┐
                    │  dispatch-cli   │  (binary)
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ dispatch-tui  │  │  dispatch-web   │  │ dispatch-agents │
└───────┬───────┘  └────────┬────────┘  └────────┬────────┘
        │                   │                    │
        │                   │                    │
        └─────────┬─────────┴─────────┬──────────┘
                  │                   │
                  ▼                   ▼
        ┌─────────────────┐  ┌─────────────────────┐
        │ dispatch-github │  │ dispatch-governance │
        └────────┬────────┘  └──────────┬──────────┘
                 │                      │
                 └──────────┬───────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │  dispatch-core  │
                   └────────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
      ┌─────────────┐ ┌───────────┐ ┌─────────────┐
      │ dispatch-db │ │dispatch-git│ │  (std libs) │
      └─────────────┘ └───────────┘ └─────────────┘
```

---

## Crate Descriptions

### dispatch-core

**Purpose:** Shared types, traits, and utilities used across all crates.

**Contents:**
```rust
// Core data types
pub mod types {
    pub mod issue;      // Issue, IssueId, IssueStatus, IssueType, Priority
    pub mod epic;       // Epic, EpicId, Stage, StageId, Gate, GateId
    pub mod agent;      // Agent, AgentId, AgentType, AgentStatus
    pub mod proposal;   // Proposal, ProposalId, Vote, ConsensusThreshold
    pub mod pr;         // PullRequest, PRStatus, ReviewStatus
}

// Shared traits
pub mod traits {
    pub trait Repository<T, Id> {
        async fn get(&self, id: Id) -> Result<Option<T>>;
        async fn create(&self, item: T) -> Result<Id>;
        async fn update(&self, item: T) -> Result<()>;
        async fn delete(&self, id: Id) -> Result<()>;
    }
}

// Error types
pub mod error {
    pub enum DispatchError { ... }
    pub type Result<T> = std::result::Result<T, DispatchError>;
}

// Configuration types
pub mod config {
    pub struct Config { ... }
}

// Event system (for internal pub/sub)
pub mod events {
    pub enum DispatchEvent { ... }
}
```

**Dependencies:** `serde`, `chrono`, `uuid`, `thiserror`

---

### dispatch-db

**Purpose:** SQLite database layer using sqlx.

**Contents:**
```rust
pub mod pool;           // Connection pool management
pub mod migrations;     // Migration runner
pub mod repos {
    pub mod issue;      // IssueRepository
    pub mod epic;       // EpicRepository
    pub mod agent;      // AgentRepository
    pub mod proposal;   // ProposalRepository
    pub mod decision;   // DecisionLogRepository
}
pub mod queries;        // Raw SQL queries (compile-time checked)
```

**Dependencies:** `sqlx`, `dispatch-core`

---

### dispatch-git

**Purpose:** Git operations and worktree management.

**Contents:**
```rust
pub mod repo;           // Repository detection and operations
pub mod worktree;       // Worktree create/list/delete
pub mod branch;         // Branch naming, creation
pub mod commit;         // Commit tracking
```

**Dependencies:** `git2`, `dispatch-core`

---

### dispatch-github

**Purpose:** GitHub API integration.

**Contents:**
```rust
pub mod client;         // GitHub API client wrapper
pub mod issues;         // Issue CRUD
pub mod prs;            // PR operations
pub mod webhooks;       // Webhook event types
pub mod sync;           // Bidirectional sync logic
pub mod metadata;       // Issue body metadata parser
```

**Dependencies:** `octocrab`, `reqwest`, `dispatch-core`

---

### dispatch-agents

**Purpose:** Agent spawning, lifecycle, and communication.

**Contents:**
```rust
pub mod executor;       // Agent subprocess management
pub mod types;          // Agent type definitions and prompts
pub mod lifecycle;      // Start, monitor, stop
pub mod heartbeat;      // Health checking
pub mod context;        // Issue → Agent context building
pub mod output;         // Output collection and parsing
```

**Dependencies:** `tokio`, `dispatch-core`, `dispatch-db`, `dispatch-git`

---

### dispatch-governance

**Purpose:** Sangha voting and decision-making.

**Contents:**
```rust
pub mod proposals;      // Proposal creation and management
pub mod voting;         // Vote collection
pub mod consensus;      // Consensus algorithms
pub mod execution;      // Approved proposal execution
pub mod overrides;      // Human override logic
pub mod broadcast;      // Agent-to-agent communication
```

**Dependencies:** `dispatch-core`, `dispatch-db`, `dispatch-agents`

---

### dispatch-cli

**Purpose:** CLI binary and command handlers.

**Contents:**
```rust
pub mod main;           // Entry point
pub mod commands {
    pub mod issue;      // dispatch issue *
    pub mod epic;       // dispatch epic *
    pub mod agent;      // dispatch agent *
    pub mod proposal;   // dispatch proposal *
    pub mod worktree;   // dispatch worktree *
    pub mod sync;       // dispatch sync
    pub mod config;     // dispatch config *
    pub mod tui;        // dispatch tui
    pub mod serve;      // dispatch serve (web server)
}
pub mod output;         // CLI output formatting (table, json)
```

**Dependencies:** `clap`, `dispatch-core`, `dispatch-db`, `dispatch-agents`, `dispatch-github`, `dispatch-governance`, `dispatch-tui`

---

### dispatch-tui

**Purpose:** Terminal UI using ratatui.

**Contents:**
```rust
pub mod app;            // TUI application state
pub mod ui {
    pub mod dashboard;  // Main dashboard
    pub mod issues;     // Issue list/detail
    pub mod agents;     // Agent status
    pub mod epics;      // Epic/stage view
    pub mod proposals;  // Voting view
    pub mod logs;       // Log viewer
}
pub mod input;          // Key handling
pub mod widgets;        // Custom widgets
```

**Dependencies:** `ratatui`, `crossterm`, `dispatch-core`, `dispatch-db`

---

### dispatch-web

**Purpose:** Web server, REST API, and WebSocket.

**Contents:**
```rust
pub mod server;         // Axum server setup
pub mod routes {
    pub mod webhooks;   // POST /webhooks/github
    pub mod api;        // REST endpoints
    pub mod ws;         // WebSocket upgrade
}
pub mod events;         // WebSocket event broadcasting
pub mod handlers;       // Request handlers
pub mod auth;           // Token/secret validation
```

**Dependencies:** `axum`, `tokio-tungstenite`, `tower`, `dispatch-core`, `dispatch-db`, `dispatch-github`

---

### dispatch-pty (Phase 11)

**Purpose:** PTY management for interactive watch mode.

**Contents:**
```rust
pub mod pty;            // PTY creation and management
pub mod stream;         // Output streaming
pub mod input;          // Input injection
pub mod session;        // Session attach/detach
```

**Dependencies:** `portable-pty` or `tokio-pty-process`, `dispatch-core`

---

## Cargo.toml (Workspace Root)

```toml
[workspace]
resolver = "2"
members = [
    "crates/dispatch-core",
    "crates/dispatch-db",
    "crates/dispatch-git",
    "crates/dispatch-github",
    "crates/dispatch-agents",
    "crates/dispatch-governance",
    "crates/dispatch-cli",
    "crates/dispatch-tui",
    "crates/dispatch-web",
    # "crates/dispatch-pty",  # Phase 11
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/user/dispatch"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }

# CLI
clap = { version = "4", features = ["derive"] }

# TUI
ratatui = "0.28"
crossterm = "0.28"

# Web
axum = { version = "0.7", features = ["ws"] }
tokio-tungstenite = "0.24"
tower = "0.5"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# HTTP client
reqwest = { version = "0.12", features = ["json"] }
octocrab = "0.41"

# Git
git2 = "0.19"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Internal crates
dispatch-core = { path = "crates/dispatch-core" }
dispatch-db = { path = "crates/dispatch-db" }
dispatch-git = { path = "crates/dispatch-git" }
dispatch-github = { path = "crates/dispatch-github" }
dispatch-agents = { path = "crates/dispatch-agents" }
dispatch-governance = { path = "crates/dispatch-governance" }
dispatch-tui = { path = "crates/dispatch-tui" }
dispatch-web = { path = "crates/dispatch-web" }
```

---

## File Naming Conventions

- Rust files: `snake_case.rs`
- Modules with multiple files: directory with `mod.rs`
- Test files: `*_test.rs` or `tests/*.rs`
- SQL migrations: `NNN_description.sql`
- Config files: `*.toml`
- Markdown docs: `kebab-case.md`

---

## Binary Output

Single binary: `dispatch`

```bash
# After build
./target/release/dispatch

# Installed
dispatch issue list
dispatch agent status
dispatch tui
dispatch serve  # Web server
```

---

## Implementation PRs

| PR | Description | Files Created |
|----|-------------|---------------|
| PR-001 | Initialize Rust workspace | `Cargo.toml`, all crate `Cargo.toml` files, `.gitignore` |
| PR-002 | CI/CD setup | `.github/workflows/*.yml` |
