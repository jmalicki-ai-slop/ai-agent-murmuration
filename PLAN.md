# Dispatch: Issue-Based Agent Orchestration System

## Master Plan

This document tracks the high-level planning and implementation process for Dispatch - a system that uses GitHub issues as the primary interface for orchestrating multiple AI agents working collaboratively on software development tasks.

---

## System Overview

**Key Features:**
- GitHub Issues as the primary UI for task management
- Multiple specialized agents (Coder, Reviewer, PM, Security, Docs, Test, Architect, Coordinator)
- Sangha governance model with democratic voting and human override
- TDD workflow with red-green-refactor cycles
- Coordinator agents for multi-agent orchestration
- Git worktrees for parallel isolated development
- Epics with stage gates and approval workflows
- TUI and Web UI for monitoring and control

---

## Phase 0: Planning & Design

| Item | Status | Document |
|------|--------|----------|
| 0.1 High-level architecture requirements | ✅ Done | `issue-dispatch-system-design.md` |
| 0.2 Core data models | ✅ Done | `issue-dispatch-system-design.md` |
| 0.3 Workflow designs | ✅ Done | `issue-dispatch-system-design.md` |
| 0.4 Technology choices | ✅ Done | `issue-dispatch-system-design.md` |
| 0.5 Rust module/crate design | ✅ Done | `design/code-structure.md` |
| 0.6 CLI command design | ✅ Done | `design/cli-design.md` |
| 0.7 GitHub integration design | ✅ Done | `design/github-integration.md` |
| 0.8 Agent execution design | ✅ Done | `design/agent-execution.md` |
| 0.9 Sangha governance design | ✅ Done | `design/sangha-governance.md` |
| 0.10 TUI design | ✅ Done | `design/tui-design.md` |
| 0.11 Configuration design | ✅ Done | `design/configuration.md` |
| 0.12 Error handling strategy | ✅ Done | `design/error-handling.md` |
| 0.13 Testing strategy | ✅ Done | `design/testing-strategy.md` |
| 0.14 WebSocket API design | ✅ Done | `design/api/websocket-api.md` |
| 0.15 REST API design | ✅ Done | `design/api/rest-api.md` |

---

## Phase 1: Foundation

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-001 | Initialize Rust workspace with crate structure | 🔲 Todo | `Cargo.toml`, workspace setup |
| PR-001a | dispatch-core crate skeleton | 🔲 Todo | `dispatch-core/` |
| PR-001b | dispatch-db crate skeleton | 🔲 Todo | `dispatch-db/` |
| PR-001c | dispatch-github crate skeleton | 🔲 Todo | `dispatch-github/` |
| PR-001d | dispatch-agent crate skeleton | 🔲 Todo | `dispatch-agent/` |
| PR-001e | dispatch-cli crate skeleton | 🔲 Todo | `dispatch-cli/` |
| PR-001f | dispatch-tui crate skeleton | 🔲 Todo | `dispatch-tui/` |
| PR-001g | dispatch-server crate skeleton | 🔲 Todo | `dispatch-server/` |
| PR-002 | CI/CD setup (GitHub Actions) | 🔲 Todo | `.github/workflows/` |
| PR-002a | Test workflow | 🔲 Todo | `ci.yml` |
| PR-002b | Release workflow | 🔲 Todo | `release.yml` |
| PR-003 | SQLite schema + migrations | 🔲 Todo | `dispatch-db/migrations/` |
| PR-003a | Core tables (issues, epics, agents) | 🔲 Todo | `001_core.sql` |
| PR-003b | Governance tables (proposals, votes) | 🔲 Todo | `002_governance.sql` |
| PR-003c | Workflow tables (workflows, feedback, reviews) | 🔲 Todo | `003_workflows.sql` |
| PR-003d | Config and sync tables | 🔲 Todo | `004_config.sql` |
| PR-004 | Core types and error definitions | 🔲 Todo | `dispatch-core/src/` |
| PR-004a | ID types (IssueId, AgentId, etc.) | 🔲 Todo | `types/ids.rs` |
| PR-004b | Issue type and state machine | 🔲 Todo | `types/issue.rs` |
| PR-004c | Epic and Stage types | 🔲 Todo | `types/epic.rs` |
| PR-004d | Agent types | 🔲 Todo | `types/agent.rs` |
| PR-004e | Proposal and Vote types | 🔲 Todo | `types/governance.rs` |
| PR-004f | Workflow types (TDD, reviews) | 🔲 Todo | `types/workflow.rs` |
| PR-004g | Error types with thiserror | 🔲 Todo | `error.rs` |
| PR-005 | Database layer implementation | 🔲 Todo | `dispatch-db/src/` |
| PR-005a | Database connection pool | 🔲 Todo | `db.rs` |
| PR-005b | Issue repository | 🔲 Todo | `repos/issues.rs` |
| PR-005c | Epic repository | 🔲 Todo | `repos/epics.rs` |
| PR-005d | Agent repository | 🔲 Todo | `repos/agents.rs` |
| PR-005e | Proposal repository | 🔲 Todo | `repos/proposals.rs` |
| PR-005f | Workflow repository | 🔲 Todo | `repos/workflows.rs` |
| PR-006 | CLI skeleton with clap | 🔲 Todo | `dispatch-cli/src/` |
| PR-006a | Main entry point and arg parsing | 🔲 Todo | `main.rs` |
| PR-006b | Issue subcommands | 🔲 Todo | `commands/issue.rs` |
| PR-006c | Epic subcommands | 🔲 Todo | `commands/epic.rs` |
| PR-006d | Agent subcommands | 🔲 Todo | `commands/agent.rs` |
| PR-006e | Proposal subcommands | 🔲 Todo | `commands/proposal.rs` |
| PR-007 | Configuration loading | 🔲 Todo | `dispatch-core/src/config.rs` |
| PR-007a | Config file parsing (TOML) | 🔲 Todo | `config.rs` |
| PR-007b | Environment variable overrides | 🔲 Todo | `config.rs` |
| PR-007c | Config CLI commands | 🔲 Todo | `dispatch-cli/src/commands/config.rs` |
| PR-007d | Runtime config store | 🔲 Todo | `dispatch-db/src/repos/config.rs` |
| PR-008 | Logging infrastructure | 🔲 Todo | `dispatch-core/src/logging.rs` |
| PR-008a | tracing setup | 🔲 Todo | `logging.rs` |
| PR-008b | File appender with rotation | 🔲 Todo | `logging.rs` |

---

## Phase 2: Git & Worktree Management

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-009 | Git repository detection | 🔲 Todo | `dispatch-core/src/git.rs` |
| PR-010 | Worktree creation | 🔲 Todo | `dispatch-core/src/worktree.rs` |
| PR-011 | Worktree cleanup | 🔲 Todo | `worktree.rs` |
| PR-012 | Branch naming conventions | 🔲 Todo | `worktree.rs` |
| PR-013 | Worktree ↔ Issue linking | 🔲 Todo | `dispatch-db/src/repos/worktrees.rs` |
| PR-014 | CLI: `dispatch worktree` commands | 🔲 Todo | `dispatch-cli/src/commands/worktree.rs` |

---

## Phase 3: Issue Management

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-015 | Issue state machine implementation | 🔲 Todo | `dispatch-core/src/types/issue.rs` |
| PR-016 | Issue creation (local) | 🔲 Todo | `dispatch-db/src/repos/issues.rs` |
| PR-017 | Issue querying/filtering | 🔲 Todo | `repos/issues.rs` |
| PR-018 | Issue assignment logic | 🔲 Todo | `dispatch-core/src/assignment.rs` |
| PR-019 | CLI: `dispatch issue` commands | 🔲 Todo | `dispatch-cli/src/commands/issue.rs` |
| PR-020 | Issue priorities and ordering | 🔲 Todo | `repos/issues.rs` |

---

## Phase 4: Epic & Stage Management

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-021 | Epic data model implementation | 🔲 Todo | `dispatch-core/src/types/epic.rs` |
| PR-022 | Stage management | 🔲 Todo | `dispatch-core/src/stage.rs` |
| PR-023 | Gate implementation | 🔲 Todo | `dispatch-core/src/gate.rs` |
| PR-024 | Stage transitions | 🔲 Todo | `dispatch-core/src/epic.rs` |
| PR-025 | CLI: `dispatch epic` commands | 🔲 Todo | `dispatch-cli/src/commands/epic.rs` |

---

## Phase 5: Agent Execution

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-026 | Agent executor base | 🔲 Todo | `dispatch-agent/src/executor.rs` |
| PR-027 | Claude Code subprocess spawning | 🔲 Todo | `dispatch-agent/src/spawn.rs` |
| PR-028 | Agent lifecycle (start, monitor, stop) | 🔲 Todo | `dispatch-agent/src/lifecycle.rs` |
| PR-029 | Heartbeat monitoring | 🔲 Todo | `dispatch-agent/src/heartbeat.rs` |
| PR-030 | Agent type definitions | 🔲 Todo | `dispatch-agent/src/types/` |
| PR-030a | Coder agent prompt | 🔲 Todo | `prompts/coder.md` |
| PR-030b | Reviewer agent prompt | 🔲 Todo | `prompts/reviewer.md` |
| PR-030c | PM agent prompt | 🔲 Todo | `prompts/pm.md` |
| PR-030d | Security agent prompt | 🔲 Todo | `prompts/security.md` |
| PR-030e | Test agent prompt | 🔲 Todo | `prompts/test.md` |
| PR-030f | Docs agent prompt | 🔲 Todo | `prompts/docs.md` |
| PR-030g | Architect agent prompt | 🔲 Todo | `prompts/architect.md` |
| PR-030h | Coordinator agent prompt | 🔲 Todo | `prompts/coordinator.md` |
| PR-031 | Issue → Agent context passing | 🔲 Todo | `dispatch-agent/src/context.rs` |
| PR-032 | Agent output collection | 🔲 Todo | `dispatch-agent/src/output.rs` |
| PR-033 | Agent failure handling | 🔲 Todo | `dispatch-agent/src/error.rs` |
| PR-034 | CLI: `dispatch agent` commands | 🔲 Todo | `dispatch-cli/src/commands/agent.rs` |

---

## Phase 6: GitHub Integration

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-035 | GitHub API client setup (octocrab) | 🔲 Todo | `dispatch-github/src/client.rs` |
| PR-036 | Issue sync: GitHub → Local | 🔲 Todo | `dispatch-github/src/sync/inbound.rs` |
| PR-037 | Issue sync: Local → GitHub | 🔲 Todo | `dispatch-github/src/sync/outbound.rs` |
| PR-038 | Metadata storage in issue body | 🔲 Todo | `dispatch-github/src/metadata.rs` |
| PR-039 | Comment command parsing | 🔲 Todo | `dispatch-github/src/commands.rs` |
| PR-040 | Webhook receiver (axum) | 🔲 Todo | `dispatch-github/src/webhook/mod.rs` |
| PR-041 | Webhook event handlers | 🔲 Todo | `dispatch-github/src/webhook/handlers.rs` |
| PR-042 | PR creation and linking | 🔲 Todo | `dispatch-github/src/pr.rs` |
| PR-043 | PR status tracking | 🔲 Todo | `dispatch-github/src/pr.rs` |
| PR-044 | Sync engine with rate limiting | 🔲 Todo | `dispatch-github/src/sync/engine.rs` |
| PR-045 | CLI: `dispatch sync` commands | 🔲 Todo | `dispatch-cli/src/commands/sync.rs` |

---

## Phase 7: Sangha Governance

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-046 | Proposal creation | 🔲 Todo | `dispatch-core/src/governance/proposal.rs` |
| PR-047 | Voting mechanism | 🔲 Todo | `dispatch-core/src/governance/voting.rs` |
| PR-048 | Consensus calculation | 🔲 Todo | `dispatch-core/src/governance/consensus.rs` |
| PR-048a | Property tests for consensus | 🔲 Todo | `tests/proptest_consensus.rs` |
| PR-049 | Proposal execution | 🔲 Todo | `dispatch-core/src/governance/execution.rs` |
| PR-050 | Human override: force decision | 🔲 Todo | `dispatch-core/src/governance/override.rs` |
| PR-051 | Human override: veto | 🔲 Todo | `governance/override.rs` |
| PR-052 | Decision logging | 🔲 Todo | `dispatch-db/src/repos/decisions.rs` |
| PR-053 | CLI: `dispatch proposal` commands | 🔲 Todo | `dispatch-cli/src/commands/proposal.rs` |

---

## Phase 8: TDD Workflows & Coordinator

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-054 | Coordinator agent infrastructure | 🔲 Todo | `dispatch-agent/src/coordinator/mod.rs` |
| PR-055 | Workflow state machine | 🔲 Todo | `dispatch-core/src/workflow/mod.rs` |
| PR-056 | TDD workflow implementation | 🔲 Todo | `dispatch-core/src/workflow/tdd.rs` |
| PR-056a | Specification phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056b | DesignReview phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056c | WriteTests phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056d | TestReview phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056e | VerifyRed phase (tests must fail) | 🔲 Todo | `workflow/tdd.rs` |
| PR-056f | Implementation phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056g | CodeReview phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056h | VerifyGreen phase (tests must pass) | 🔲 Todo | `workflow/tdd.rs` |
| PR-056i | Refactor phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-056j | FinalReview phase | 🔲 Todo | `workflow/tdd.rs` |
| PR-057 | Design review workflow | 🔲 Todo | `dispatch-core/src/workflow/design_review.rs` |
| PR-058 | Feedback routing system | 🔲 Todo | `dispatch-agent/src/coordinator/feedback.rs` |
| PR-059 | Iteration management | 🔲 Todo | `dispatch-core/src/workflow/iteration.rs` |
| PR-060 | Max iterations escalation | 🔲 Todo | `workflow/iteration.rs` |

---

## Phase 9: Human Override & Control

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-061 | Pause/resume agents | 🔲 Todo | `dispatch-agent/src/control.rs` |
| PR-062 | Reassign issues | 🔲 Todo | `dispatch-core/src/assignment.rs` |
| PR-063 | Cancel issues | 🔲 Todo | `repos/issues.rs` |
| PR-064 | Skip gates | 🔲 Todo | `dispatch-core/src/gate.rs` |
| PR-065 | Direct instruction via comments | 🔲 Todo | `dispatch-github/src/commands.rs` |
| PR-066 | Gate approval workflow | 🔲 Todo | `dispatch-core/src/gate.rs` |

---

## Phase 10: TUI

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-067 | TUI framework setup (ratatui) | 🔲 Todo | `dispatch-tui/src/` |
| PR-068 | App state management | 🔲 Todo | `dispatch-tui/src/app.rs` |
| PR-069 | Dashboard view | 🔲 Todo | `dispatch-tui/src/views/dashboard.rs` |
| PR-070 | Issue list view | 🔲 Todo | `dispatch-tui/src/views/issues.rs` |
| PR-071 | Agent status view | 🔲 Todo | `dispatch-tui/src/views/agents.rs` |
| PR-072 | Epic/stage view | 🔲 Todo | `dispatch-tui/src/views/epics.rs` |
| PR-073 | Proposal/voting view | 🔲 Todo | `dispatch-tui/src/views/proposals.rs` |
| PR-074 | Log viewer | 🔲 Todo | `dispatch-tui/src/views/logs.rs` |
| PR-075 | Keyboard navigation | 🔲 Todo | `dispatch-tui/src/input.rs` |
| PR-076 | Command mode | 🔲 Todo | `dispatch-tui/src/command.rs` |
| PR-077 | CLI: `dispatch tui` | 🔲 Todo | `dispatch-cli/src/commands/tui.rs` |

---

## Phase 11: Web Server & API

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-078 | Axum server setup | 🔲 Todo | `dispatch-server/src/` |
| PR-079 | REST API infrastructure | 🔲 Todo | `dispatch-server/src/api/mod.rs` |
| PR-079a | Health endpoints | 🔲 Todo | `api/health.rs` |
| PR-079b | Issue endpoints | 🔲 Todo | `api/issues.rs` |
| PR-079c | Epic endpoints | 🔲 Todo | `api/epics.rs` |
| PR-079d | Agent endpoints | 🔲 Todo | `api/agents.rs` |
| PR-079e | Proposal endpoints | 🔲 Todo | `api/proposals.rs` |
| PR-079f | Workflow endpoints | 🔲 Todo | `api/workflows.rs` |
| PR-079g | OpenAPI documentation | 🔲 Todo | `api/openapi.rs` |
| PR-080 | WebSocket infrastructure | 🔲 Todo | `dispatch-server/src/websocket/mod.rs` |
| PR-080a | Connection handler | 🔲 Todo | `websocket/handler.rs` |
| PR-080b | Event publishing | 🔲 Todo | `dispatch-core/src/events.rs` |
| PR-080c | Channel subscriptions | 🔲 Todo | `websocket/channels.rs` |
| PR-081 | Authentication middleware | 🔲 Todo | `dispatch-server/src/auth.rs` |
| PR-082 | CLI: `dispatch serve` | 🔲 Todo | `dispatch-cli/src/commands/serve.rs` |

---

## Phase 12: Web UI (Frontend)

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-083 | Frontend project setup | 🔲 Todo | `web/` |
| PR-084 | WebSocket client library | 🔲 Todo | `web/src/lib/websocket.ts` |
| PR-085 | Dashboard page | 🔲 Todo | `web/src/pages/dashboard.tsx` |
| PR-086 | Issue management page | 🔲 Todo | `web/src/pages/issues.tsx` |
| PR-087 | Epic/gate approval page | 🔲 Todo | `web/src/pages/epics.tsx` |
| PR-088 | Agent status page | 🔲 Todo | `web/src/pages/agents.tsx` |
| PR-089 | Proposal voting page | 🔲 Todo | `web/src/pages/proposals.tsx` |

---

## Phase 13: Interactive Watch Mode

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-090 | PTY-based agent execution | 🔲 Todo | `dispatch-agent/src/pty.rs` |
| PR-091 | PTY output capture | 🔲 Todo | `pty.rs` |
| PR-092 | WebSocket terminal streaming | 🔲 Todo | `dispatch-server/src/terminal.rs` |
| PR-093 | xterm.js integration | 🔲 Todo | `web/src/components/Terminal.tsx` |
| PR-094 | Bidirectional input | 🔲 Todo | `terminal.rs` |
| PR-095 | Attach/detach functionality | 🔲 Todo | `dispatch-agent/src/attach.rs` |

---

## Phase 14: Testing Infrastructure

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-096 | Unit test setup | 🔲 Todo | All crates |
| PR-097 | Property tests (proptest) | 🔲 Todo | `tests/proptest_*.rs` |
| PR-097a | Consensus calculator proptests | 🔲 Todo | `proptest_consensus.rs` |
| PR-097b | State machine proptests | 🔲 Todo | `proptest_state.rs` |
| PR-097c | Branch naming proptests | 🔲 Todo | `proptest_branch.rs` |
| PR-098 | Network mocking (wiremock) | 🔲 Todo | `tests/fixtures/github_mock.rs` |
| PR-099 | Database integration tests | 🔲 Todo | `dispatch-db/tests/` |
| PR-100 | End-to-end workflow tests | 🔲 Todo | `tests/e2e/` |

---

## Phase 15: Polish & Production

| PR | Description | Status | Files |
|----|-------------|--------|-------|
| PR-101 | Error recovery improvements | 🔲 Todo | Various |
| PR-102 | Graceful shutdown | 🔲 Todo | All crates |
| PR-103 | Metrics collection | 🔲 Todo | `dispatch-core/src/metrics.rs` |
| PR-104 | User documentation | 🔲 Todo | `docs/` |
| PR-105 | Installation scripts | 🔲 Todo | `scripts/install.sh` |
| PR-106 | Release automation | 🔲 Todo | `.github/workflows/release.yml` |

---

## Design Documents Index

All detailed design documents in `design/`:

```
design/
├── code-structure.md         # Rust module organization, patterns, types
├── cli-design.md             # CLI commands, arguments, examples
├── github-integration.md     # GitHub API, metadata, webhooks, sync
├── agent-execution.md        # Agent spawning, prompts, lifecycle
├── sangha-governance.md      # Proposals, voting, coordinator, TDD
├── tui-design.md             # Terminal UI layouts, views, keys
├── configuration.md          # Config hierarchy, TOML schema
├── error-handling.md         # Error types, recovery, logging
├── testing-strategy.md       # Testing pyramid, proptests, mocking
└── api/
    ├── websocket-api.md      # WebSocket events and messages
    └── rest-api.md           # REST endpoints and schemas
```

---

## Key Architecture Decisions

1. **Rust Workspace**: 7 crates for separation of concerns
   - `dispatch-core`: Types, errors, business logic
   - `dispatch-db`: SQLite with sqlx (compile-time checked)
   - `dispatch-github`: Octocrab for GitHub API
   - `dispatch-agent`: Claude Code spawning
   - `dispatch-cli`: Clap-based CLI
   - `dispatch-tui`: Ratatui terminal UI
   - `dispatch-server`: Axum HTTP/WebSocket server

2. **GitHub as UI**: Issues are the primary interface, with metadata stored in HTML comments

3. **TDD Workflow**: 11-phase red-green-refactor cycle with mandatory test failure verification

4. **Coordinator Pattern**: Meta-agents orchestrate multi-agent workflows

5. **Sangha Governance**: Democratic voting with human override capability

6. **Property Testing**: Proptest for exhaustive testing of compute functions

7. **Network Mocking**: Wiremock for GitHub API testing

---

## Progress Summary

| Phase | Items | Complete | Progress |
|-------|-------|----------|----------|
| Phase 0: Design | 15 | 15 | 100% |
| Phase 1: Foundation | ~25 | 0 | 0% |
| Phase 2: Git/Worktrees | 6 | 0 | 0% |
| Phase 3: Issues | 6 | 0 | 0% |
| Phase 4: Epics | 5 | 0 | 0% |
| Phase 5: Agents | ~15 | 0 | 0% |
| Phase 6: GitHub | 11 | 0 | 0% |
| Phase 7: Governance | 9 | 0 | 0% |
| Phase 8: TDD/Coordinator | ~15 | 0 | 0% |
| Phase 9: Control | 6 | 0 | 0% |
| Phase 10: TUI | 11 | 0 | 0% |
| Phase 11: Web Server | ~12 | 0 | 0% |
| Phase 12: Web UI | 7 | 0 | 0% |
| Phase 13: Watch Mode | 6 | 0 | 0% |
| Phase 14: Testing | ~8 | 0 | 0% |
| Phase 15: Polish | 6 | 0 | 0% |

**Overall: Phase 0 complete, ready for implementation**

---

## Next Steps

1. ~~Complete all design documents~~ ✅
2. Begin Phase 1: Foundation
   - Start with PR-001: Initialize Rust workspace
   - Set up CI/CD pipeline
   - Implement database schema and migrations
3. Work through phases sequentially, using TDD approach
4. Each PR should include tests before implementation
