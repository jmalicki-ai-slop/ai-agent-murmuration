# Murmuration Architecture

This document describes the high-level architecture of Murmuration, a multi-agent orchestration system for software development using Claude Code.

## System Overview

Murmuration coordinates multiple AI agents (Claude Code instances) working collaboratively on software development tasks. It uses GitHub issues as the primary interface, git worktrees for workspace isolation, and SQLite for state persistence.

```
┌─────────────────────────────────────────────────────────────────┐
│                         User / GitHub                            │
│                 (Issues, PRs, Dependencies)                      │
└───────────────────┬─────────────────────────────────────────────┘
                    │
                    │ murmur work <issue>
                    │ murmur tdd <issue>
                    │
┌───────────────────▼─────────────────────────────────────────────┐
│                      murmur-cli                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Commands:                                                 │   │
│  │  - run:      Spawn single agent on task                  │   │
│  │  - agent:    Start typed agent (Implement/Test/Review)   │   │
│  │  - work:     Work on GitHub issue                        │   │
│  │  - tdd:      Run TDD workflow on issue                   │   │
│  │  - worktree: Manage git worktrees                        │   │
│  │  - issue:    View issues and dependencies                │   │
│  │  - status:   Show running agents                         │   │
│  └──────────────────────────────────────────────────────────┘   │
└───────────────────┬─────────────────────────────────────────────┘
                    │
                    │ Uses
                    │
┌───────────────────▼─────────────────────────────────────────────┐
│                      murmur-core                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Agent Management:                                        │    │
│  │  - AgentSpawner:   Spawn Claude Code subprocesses       │    │
│  │  - AgentTypes:     Implement, Test, Review, Coordinator │    │
│  │  - OutputStreamer: Parse JSON stream from agents        │    │
│  │  - AgentFactory:   Create typed agents with prompts     │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Git Operations:                                          │    │
│  │  - GitRepo:       Repository detection & operations     │    │
│  │  - Worktree:      Create/manage isolated worktrees      │    │
│  │  - WorktreePool:  Cache and reuse worktrees             │    │
│  │  - Branch:        Find branching points from main       │    │
│  │  - Clone:         Clone repos to cache                  │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Workflow Engine:                                         │    │
│  │  - TDD:           7-phase TDD cycle (Spec→Test→Red→     │    │
│  │                   Implement→Green→Refactor→Complete)    │    │
│  │  - TestRunner:    Run tests and validate red/green      │    │
│  │  - Transitions:   Phase transition validation           │    │
│  │  - Review:        Review workflow between phases        │    │
│  │  - Coordinator:   Orchestrate multi-agent workflows     │    │
│  └─────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Configuration & Utilities:                               │    │
│  │  - Config:        Load from ~/.config/murmur/config.toml│    │
│  │  - Secrets:       Load GitHub tokens from config/env    │    │
│  │  - PlanParser:    Parse PLAN.md into structured data    │    │
│  └─────────────────────────────────────────────────────────┘    │
└───────────────────┬──────────────────┬──────────────────────────┘
                    │                  │
        ┌───────────▼──────┐  ┌───────▼──────────┐
        │  murmur-github   │  │   murmur-db      │
        │                  │  │                  │
        │ - Client         │  │ - Connection     │
        │ - Issues         │  │ - Models         │
        │ - Dependencies   │  │ - Repositories:  │
        │ - Metadata       │  │   * agents       │
        │ - PR Status      │  │   * issues       │
        │ - Create Issues  │  │   * conversations│
        │                  │  │   * worktrees    │
        └──────────────────┘  └──────────────────┘
                │                      │
                │                      │
        ┌───────▼──────┐      ┌───────▼──────────────┐
        │   GitHub     │      │  ~/.murmur/state.db  │
        │   Issues     │      │  (SQLite Database)   │
        │   PRs        │      │                      │
        └──────────────┘      └──────────────────────┘

                ┌──────────────────────────────────┐
                │      Agent Processes             │
                │  (Claude Code Subprocesses)      │
                │                                  │
                │  ┌────────────────────────────┐  │
                │  │ claude --print \           │  │
                │  │   --output-format \        │  │
                │  │   stream-json \            │  │
                │  │   "<prompt>"               │  │
                │  │                            │  │
                │  │ → Streams JSON output      │  │
                │  │ → Tools: Read, Write, Bash │  │
                │  │ → Working in worktree dir  │  │
                │  └────────────────────────────┘  │
                └──────────────────────────────────┘
```

## Crate Structure

Murmuration is organized as a Cargo workspace with specialized crates:

### murmur-core

**Purpose**: Core library containing the fundamental logic for agent spawning, git operations, configuration, and workflow management.

**Key Modules**:

- `agent/`
  - `spawn.rs`: Spawns Claude Code as subprocess with `--print --output-format stream-json` flags
  - `output.rs`: Parses streaming JSON from Claude (messages, tool use, results, cost)
  - `types.rs`: Agent type definitions (Implement, Test, Review, Coordinator)
  - `typed.rs`: Typed agent interfaces with specialized prompts
  - `backend.rs`: Backend abstraction (Claude Code, Cursor)
  - `selection.rs`: Model selection logic

- `git/`
  - `repo.rs`: Git repository detection and remote information
  - `worktree.rs`: Worktree creation at `~/.cache/murmur/worktrees/`
  - `pool.rs`: Worktree caching and metadata persistence
  - `clone.rs`: Repository cloning to `~/.cache/murmur/repos/`
  - `branch.rs`: Finding branching points from origin/main

- `workflow/`
  - `tdd.rs`: 7-phase TDD state machine and workflow
  - `test_runner.rs`: Test execution and validation (red/green)
  - `transitions.rs`: Phase transition validation logic
  - `review.rs`: Review workflow coordination
  - `coordinator.rs`: Multi-agent orchestration
  - `resume.rs`: Resume interrupted workflows
  - `state.rs`: Workflow state management

- `plan/`
  - `parser.rs`: Parses PLAN.md markdown tables into structured phases/PRs

- `config.rs`: Configuration from `~/.config/murmur/config.toml` with env overrides
- `error.rs`: Error types and Result aliases

### murmur-cli

**Purpose**: Command-line interface binary providing user-facing commands.

**Key Modules**:

- `main.rs`: CLI entry point, argument parsing with clap
- `commands/`
  - `run.rs`: Simple agent spawning (`murmur run <prompt>`)
  - `agent.rs`: Typed agent spawning (`murmur agent --type implement`)
  - `work.rs`: Work on GitHub issues (`murmur work <issue-number>`)
  - `tdd.rs`: TDD workflow command (`murmur tdd <issue-number>`)
  - `worktree.rs`: Worktree management (`murmur worktree create/list/clean`)
  - `issue.rs`: GitHub issue commands (`murmur issue list/show/deps`)
  - `status.rs`: Show running agents and worktrees

### murmur-github

**Purpose**: GitHub API integration via octocrab for issues, PRs, and dependency tracking.

**Key Modules**:

- `client.rs`: GitHub API client using octocrab (requires GITHUB_TOKEN)
- `issues.rs`: Fetch and filter issues from repositories
- `metadata.rs`: Parse `<!-- murmur:metadata {...} -->` blocks from issue bodies
- `dependencies.rs`: Parse "Depends on #X" links and build dependency graphs
- `pr.rs`: Check PR merge status for dependency resolution
- `create.rs`: Create GitHub issues from parsed PLAN.md

### murmur-db

**Purpose**: SQLite database for state persistence across runs.

**Key Modules**:

- `connection.rs`: Database connection management
- `models.rs`: Data models (AgentRun, ConversationLog, WorktreeRecord, IssueState)
- `repos/`: Repository pattern for data access
  - `agents.rs`: Agent run history
  - `conversations.rs`: Conversation log storage
  - `worktrees.rs`: Worktree tracking
  - `issues.rs`: Issue state persistence
- `conversation_logger.rs`: Log streaming output to database

**Database Location**: `~/.murmur/state.db`

## Data Flow

### 1. Simple Agent Spawning

```
User: murmur run "fix the bug"
  │
  ├─> CLI parses arguments
  ├─> AgentSpawner.spawn("fix the bug", workdir)
  ├─> Spawns subprocess: claude --print --output-format stream-json "fix the bug"
  ├─> OutputStreamer reads stdout line-by-line
  ├─> Parses JSON: {"type":"assistant","message":{"content":[...]}}
  ├─> PrintHandler displays text to terminal
  └─> Exit when process completes
```

### 2. Working on GitHub Issue

```
User: murmur work 42
  │
  ├─> CLI loads GitHub client
  ├─> Fetch issue #42 from GitHub API
  ├─> Parse dependencies from issue body
  ├─> Check all dependency PRs are merged
  ├─> If blocked: show blockers and exit
  ├─> If ready:
  │   ├─> GitRepo.open(cwd)
  │   ├─> Find branching point (origin/main)
  │   ├─> Create worktree at ~/.cache/murmur/worktrees/<repo>/<issue>
  │   ├─> AgentSpawner.spawn(issue_prompt, worktree_path)
  │   └─> Stream output to terminal
  └─> Record run in database
```

### 3. TDD Workflow

```
User: murmur tdd 42
  │
  ├─> Load issue and check dependencies
  ├─> Create worktree
  ├─> TddWorkflow::new(issue_description, worktree_path)
  │   │
  │   ├─> Phase 1: WriteSpec
  │   │   ├─> Spawn Implement agent with spec prompt
  │   │   ├─> Wait for completion
  │   │   └─> Advance to WriteTests
  │   │
  │   ├─> Phase 2: WriteTests
  │   │   ├─> Spawn Test agent with test prompt
  │   │   ├─> Wait for completion
  │   │   └─> Advance to VerifyRed
  │   │
  │   ├─> Phase 3: VerifyRed
  │   │   ├─> TestRunner.run_tests(workdir)
  │   │   ├─> Validate tests FAIL (red phase)
  │   │   ├─> If pass unexpectedly: retry_tests()
  │   │   └─> If fail correctly: advance to Implement
  │   │
  │   ├─> Phase 4: Implement
  │   │   ├─> Spawn Implement agent with minimal implementation prompt
  │   │   ├─> Wait for completion
  │   │   └─> Advance to VerifyGreen
  │   │
  │   ├─> Phase 5: VerifyGreen
  │   │   ├─> TestRunner.run_tests(workdir)
  │   │   ├─> Validate tests PASS (green phase)
  │   │   ├─> If fail: retry_implement()
  │   │   ├─> If exceed max iterations: escalate to human
  │   │   └─> If pass: advance to Refactor
  │   │
  │   ├─> Phase 6: Refactor
  │   │   ├─> Spawn Implement agent with refactor prompt
  │   │   ├─> Run tests after each change
  │   │   └─> Advance to Complete
  │   │
  │   └─> Phase 7: Complete
  │       └─> TDD cycle finished
  │
  └─> Save workflow state to database
```

## Agent Spawning Details

### How Agents are Spawned

Murmuration spawns Claude Code as a subprocess with specific flags:

```rust
Command::new(executable_path)
    .arg("--print")              // Print only final response
    .arg("--verbose")            // Show tool usage
    .arg("--output-format")
    .arg("stream-json")          // Stream JSON output
    .arg("--dangerously-skip-permissions")  // Auto-approve tools
    .arg("--model")
    .arg(model)                  // e.g. "claude-sonnet-4-5-20250929"
    .arg(prompt)                 // The task prompt
    .current_dir(workdir)        // Working directory
    .stdout(Stdio::piped())      // Capture stdout
    .stderr(Stdio::piped())      // Capture stderr
    .spawn()
```

### Output Streaming and Parsing

The subprocess streams JSON to stdout, one message per line:

```json
{"type":"system","session_id":"abc123"}
{"type":"assistant","message":{"content":[{"type":"text","text":"I'll help with that."}]}}
{"type":"tool_use","tool":"Read","input":{"file_path":"/path/to/file"}}
{"type":"tool_result","output":"file contents","is_error":false}
{"type":"result","cost":{"input_tokens":100,"output_tokens":50},"duration_ms":1234}
```

The `OutputStreamer` reads line-by-line, parses each JSON object, and dispatches to a `StreamHandler`:

```rust
pub trait StreamHandler {
    fn on_system(&mut self, subtype: Option<&str>, session_id: Option<&str>);
    fn on_assistant_text(&mut self, text: &str);
    fn on_tool_use(&mut self, tool: &str, input: &serde_json::Value);
    fn on_tool_result(&mut self, output: &str, is_error: bool);
    fn on_complete(&mut self, cost: Option<&CostInfo>, duration_ms: Option<u64>);
}
```

This allows flexible handling:
- `PrintHandler`: Display to terminal
- `ConversationLogger`: Save to database
- `ReviewHandler`: Extract feedback for coordinator

## Worktree Management

### Why Worktrees?

Git worktrees provide isolated workspaces without cloning:
- Multiple agents can work on different issues simultaneously
- Each workspace has its own branch and working files
- No git state conflicts between concurrent agents
- Fast setup (no full clone needed)

### Worktree Lifecycle

1. **Creation**: `git worktree add -b <branch> <path> <base-commit>`
2. **Agent Work**: Claude Code runs in the worktree directory
3. **Completion**: Agent commits changes, creates PR
4. **Caching**: Worktree kept in `~/.cache/murmur/worktrees/` for reuse
5. **Cleanup**: LRU eviction when cache exceeds size limit

### Cache Structure

```
~/.cache/murmur/
├── repos/                    # Cloned repositories
│   └── owner/
│       └── repo/             # Main repo clone
└── worktrees/                # Active worktrees
    └── repo-name/
        └── murmur-issue-42/  # Worktree for issue #42
            ├── .git          # Git metadata
            └── ...           # Working files
```

## Configuration

Configuration is loaded from `~/.config/murmur/config.toml`:

```toml
[agent]
executable_path = "claude"
model = "claude-sonnet-4-5-20250929"
backend = "claude"  # or "cursor"

[github]
token = "ghp_..."
default_owner = "username"
default_repo = "myproject"

[worktree]
cache_dir = "~/.cache/murmur/worktrees"
max_cache_size_gb = 10

[database]
path = "~/.murmur/state.db"
```

Environment variables override config:
- `MURMUR_CLAUDE_PATH`: Override executable path
- `MURMUR_MODEL`: Override model selection
- `MURMUR_BACKEND`: Override backend (claude/cursor)
- `GITHUB_TOKEN`: GitHub personal access token

## Error Handling

Murmuration uses a Result-based error handling pattern:

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Agent(String),          // Agent spawn/execution errors
    Git(String),            // Git operation failures
    Config(String),         // Configuration issues
    GitHub(String),         // GitHub API errors
    Database(String),       // Database errors
    Io(std::io::Error),     // IO errors
    Other(String),          // Catch-all
}
```

Errors propagate up from core operations through CLI commands, providing context at each level.
