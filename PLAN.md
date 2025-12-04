# Murmuration: Issue-Based Multi-Agent Orchestration

## Master Plan

This document tracks the implementation of Murmuration - a system that uses GitHub issues as the primary interface for orchestrating multiple AI agents working collaboratively on software development tasks.

**Goal: Bootstrap as early as possible. Use Murmuration to build Murmuration.**

---

## System Overview

**Core Philosophy:**
- Get to self-hosting fast, then progressively enhance
- Distinct specialized agents (not one agent doing everything)
- TDD workflow with enforced red-green validation
- Review agents at every phase transition
- Human approval (voting comes later)

**MVP Features (for self-hosting):**
- GitHub Issues as task source
- Git worktrees with smart branching (from updated main, cached pool, clone unknown repos)
- Agent spawning: Coder, Reviewer, Test agents
- Red-green TDD: tests must fail → implement → tests must pass
- Coordinator routing feedback between agents
- Basic CLI to trigger and monitor

**Deferred Features:**
- Democratic voting/proposals (human approves for now)
- TUI and Web UI
- Webhooks (manual trigger initially)
- Complex consensus thresholds

---

## Bootstrap Phases

### Phase 1: Minimal CLI + Agent Spawning
*Goal: Can spawn a Claude Code agent on a task*

| PR | Description | Files |
|----|-------------|-------|
| PR-001 | Rust workspace with minimal crates | `Cargo.toml`, `murmur-core/`, `murmur-cli/` |
| PR-002 | Basic CLI with `murmur run <prompt>` | `murmur-cli/src/main.rs` |
| PR-003 | Claude Code subprocess spawning | `murmur-core/src/agent/spawn.rs` |
| PR-004 | Agent output streaming to terminal | `murmur-core/src/agent/output.rs` |
| PR-005 | Basic config (claude path, model) | `murmur-core/src/config.rs` |

**Checkpoint:** Can run `murmur run "fix the typo in README"` and see Claude work.

---

### Phase 2: Git Worktrees (Smart)
*Goal: Isolated workspaces with intelligent branching*

| PR | Description | Files |
|----|-------------|-------|
| PR-006 | Git repo detection and validation | `murmur-core/src/git/repo.rs` |
| PR-007 | Fetch and find best branching point | `murmur-core/src/git/branch.rs` |
| PR-007a | Detect default branch (main/master) | |
| PR-007b | Fetch latest from remote | |
| PR-007c | Find merge-base for existing branches | |
| PR-008 | Worktree creation from branching point | `murmur-core/src/git/worktree.rs` |
| PR-009 | Worktree pool/cache management | `murmur-core/src/git/pool.rs` |
| PR-009a | Cache directory (~/.cache/murmur/worktrees/) | |
| PR-009b | Reuse existing worktrees when possible | |
| PR-009c | LRU cleanup of old worktrees | |
| PR-010 | Clone unknown repos on demand | `murmur-core/src/git/clone.rs` |
| PR-011 | CLI: `murmur worktree create/list/clean` | `murmur-cli/src/commands/worktree.rs` |

**Checkpoint:** Can run agents in isolated worktrees, worktrees branch from fresh main.

---

### Phase 3: GitHub Integration (Read + Dependencies)
*Goal: Pull issues from GitHub, understand dependencies*

| PR | Description | Files |
|----|-------------|-------|
| PR-012 | GitHub API client (octocrab) | `murmur-github/src/client.rs` |
| PR-013 | Fetch issues from repo | `murmur-github/src/issues.rs` |
| PR-014 | Parse issue metadata from body | `murmur-github/src/metadata.rs` |
| PR-015 | Parse issue dependencies | `murmur-github/src/dependencies.rs` |
| PR-015a | "Depends on #X" / "Blocked by #X" parsing | |
| PR-015b | "Parent: #X" for epic linking | |
| PR-015c | Build dependency graph | |
| PR-016 | Check PR merge status | `murmur-github/src/pr.rs` |
| PR-016a | Is linked PR merged? | |
| PR-016b | Are all dependency PRs merged? | |
| PR-017 | CLI: `murmur issue list/show/deps` | `murmur-cli/src/commands/issue.rs` |
| PR-018 | CLI: `murmur work <issue-number>` | `murmur-cli/src/commands/work.rs` |
| PR-018a | Block if dependencies not met | |
| PR-018b | Show what's blocking | |

**Checkpoint:** Can run `murmur work 42` - blocks if issue #42 depends on unmerged PRs.

---

### Phase 3b: Plan Import to GitHub
*Goal: Import PLAN.md phases as epics/issues with dependencies*

| PR | Description | Files |
|----|-------------|-------|
| PR-019 | Parse PLAN.md structure | `murmur-core/src/plan/parser.rs` |
| PR-019a | Extract phases as epics | |
| PR-019b | Extract PRs as issues | |
| PR-019c | Infer dependencies from ordering | |
| PR-020 | Create GitHub issues from plan | `murmur-github/src/create.rs` |
| PR-020a | Create epic issue with checklist | |
| PR-020b | Create child issues with "Parent: #X" | |
| PR-020c | Add "Depends on #X" for sequential items | |
| PR-021 | CLI: `murmur plan import` | `murmur-cli/src/commands/plan.rs` |
| PR-022 | CLI: `murmur plan status` (show tree) | `murmur-cli/src/commands/plan.rs` |

**Checkpoint:** Can run `murmur plan import` to create all GitHub issues from PLAN.md.

---

### Phase 3.5: Persistence (SQLite)
*Goal: Track state across runs, store conversation logs*

| PR | Description | Files |
|----|-------------|-------|
| PR-023 | Database schema + migrations | `murmur-db/migrations/` |
| PR-024 | Issue state persistence | `murmur-db/src/repos/issues.rs` |
| PR-025 | Agent run history | `murmur-db/src/repos/agents.rs` |
| PR-026 | Conversation log storage | `murmur-db/src/repos/conversations.rs` |
| PR-027 | Resume interrupted workflows | `murmur-core/src/workflow/resume.rs` |

**Checkpoint:** Agent conversations are logged to SQLite, can resume interrupted work.

---

### Phase 4: Agent Types + Prompts
*Goal: Distinct specialized agents with role-specific prompts*

| PR | Description | Files |
|----|-------------|-------|
| PR-028 | Agent type enum and config | `murmur-core/src/agent/types.rs` |
| PR-029 | System prompt loading from files | `murmur-core/src/agent/prompts.rs` |
| PR-030 | Coder agent prompt | `prompts/coder.md` |
| PR-031 | Reviewer agent prompt | `prompts/reviewer.md` |
| PR-032 | Test agent prompt | `prompts/test.md` |
| PR-033 | Context building (issue, files, history) | `murmur-core/src/agent/context.rs` |
| PR-034 | CLI: `murmur agent start --type coder` | `murmur-cli/src/commands/agent.rs` |

**Checkpoint:** Can spawn different agent types with appropriate prompts.

---

### Phase 5: TDD Workflow (Red-Green)
*Goal: Enforced test-first development with validation*

| PR | Description | Files |
|----|-------------|-------|
| PR-035 | Workflow state machine | `murmur-core/src/workflow/mod.rs` |
| PR-036 | TDD phases enum | `murmur-core/src/workflow/tdd.rs` |
| PR-036a | WriteSpec phase | |
| PR-036b | WriteTests phase | |
| PR-036c | VerifyRed phase (tests MUST fail) | |
| PR-036d | Implement phase | |
| PR-036e | VerifyGreen phase (tests MUST pass) | |
| PR-036f | Refactor phase | |
| PR-037 | Test runner integration | `murmur-core/src/workflow/test_runner.rs` |
| PR-037a | Detect test framework (cargo test, pytest, jest, etc.) | |
| PR-037b | Run tests and parse results | |
| PR-037c | Validate red (>0 failures) / green (0 failures) | |
| PR-038 | Phase transition logic | `murmur-core/src/workflow/transitions.rs` |
| PR-039 | CLI: `murmur tdd <issue>` | `murmur-cli/src/commands/tdd.rs` |

**Checkpoint:** Can run TDD workflow that enforces tests fail before implementation.

---

### Phase 6: Review Agents at Each Phase
*Goal: Distinct reviewer agents gate each phase transition*

| PR | Description | Files |
|----|-------------|-------|
| PR-040 | Review request generation | `murmur-core/src/review/request.rs` |
| PR-041 | Reviewer agent invocation | `murmur-core/src/review/reviewer.rs` |
| PR-042 | Review feedback parsing | `murmur-core/src/review/feedback.rs` |
| PR-043 | Phase gates with review requirement | `murmur-core/src/workflow/gates.rs` |
| PR-043a | Spec review before WriteTests | |
| PR-043b | Test review before VerifyRed | |
| PR-043c | Code review before VerifyGreen | |
| PR-043d | Final review before complete | |
| PR-044 | Feedback routing back to coder | `murmur-core/src/review/routing.rs` |
| PR-045 | Iteration tracking (attempt count) | `murmur-core/src/workflow/iteration.rs` |

**Checkpoint:** Reviews happen between phases, feedback loops back to coder.

---

### Phase 7: Coordinator Agent
*Goal: Meta-agent orchestrating the workflow*

| PR | Description | Files |
|----|-------------|-------|
| PR-046 | Coordinator agent type | `murmur-core/src/coordinator/mod.rs` |
| PR-047 | Coordinator prompt | `prompts/coordinator.md` |
| PR-048 | Agent-to-agent communication | `murmur-core/src/coordinator/comms.rs` |
| PR-049 | Workflow orchestration loop | `murmur-core/src/coordinator/orchestrate.rs` |
| PR-050 | Human escalation on max iterations | `murmur-core/src/coordinator/escalate.rs` |
| PR-051 | CLI: `murmur orchestrate <issue>` | `murmur-cli/src/commands/orchestrate.rs` |

**Checkpoint:** Single command runs full TDD workflow with coordinator managing agents.

---

### 🎯 BOOTSTRAP MILESTONE
*Murmuration can now build itself!*

At this point:
- `murmur orchestrate 42` runs full TDD workflow on issue #42
- Coordinator spawns coder, test, reviewer agents as needed
- Worktrees created from fresh main
- Red-green validation enforced
- Reviews gate each phase

**Start using Murmuration to build remaining features.**

---

## Post-Bootstrap Phases

### Phase 8: GitHub Integration (Write)
*Goal: Push results back to GitHub*

| PR | Description | Files |
|----|-------------|-------|
| PR-052 | Update issue metadata in body | `murmur-github/src/metadata.rs` |
| PR-053 | Post progress comments | `murmur-github/src/comments.rs` |
| PR-054 | Create PR on completion | `murmur-github/src/pr.rs` |
| PR-055 | Link PR to issue | `murmur-github/src/pr.rs` |
| PR-056 | CLI: `murmur sync` | `murmur-cli/src/commands/sync.rs` |

---

### Phase 9: Background Daemon
*Goal: Run continuously, watch for new issues*

| PR | Description | Files |
|----|-------------|-------|
| PR-057 | Daemon mode | `murmur-cli/src/commands/daemon.rs` |
| PR-058 | Issue polling loop | `murmur-core/src/daemon/poll.rs` |
| PR-059 | Webhook receiver (optional) | `murmur-server/src/webhook.rs` |
| PR-060 | Concurrent agent management | `murmur-core/src/daemon/scheduler.rs` |
| PR-061 | Max concurrent agents config | `murmur-core/src/config.rs` |

---

### Phase 10: Human Override & Control
*Goal: Pause, resume, redirect agents*

| PR | Description | Files |
|----|-------------|-------|
| PR-062 | Pause/resume agents | `murmur-core/src/agent/control.rs` |
| PR-063 | Cancel workflow | `murmur-core/src/workflow/cancel.rs` |
| PR-064 | Skip phase (force advance) | `murmur-core/src/workflow/skip.rs` |
| PR-065 | Inject human feedback | `murmur-core/src/review/human.rs` |
| PR-066 | CLI: `murmur pause/resume/cancel` | `murmur-cli/src/commands/control.rs` |

---

### Phase 11: Additional Agent Types
*Goal: More specialized agents*

| PR | Description | Files |
|----|-------------|-------|
| PR-067 | Security agent + prompt | `prompts/security.md` |
| PR-068 | Docs agent + prompt | `prompts/docs.md` |
| PR-069 | Architect agent + prompt | `prompts/architect.md` |
| PR-070 | PM agent (issue decomposition) | `prompts/pm.md` |
| PR-071 | Configurable agent pipelines | `murmur-core/src/workflow/pipeline.rs` |

---

### Phase 12: Epics & Stages
*Goal: Large features with gates*

| PR | Description | Files |
|----|-------------|-------|
| PR-072 | Epic data model | `murmur-core/src/types/epic.rs` |
| PR-073 | Stage management | `murmur-core/src/epic/stages.rs` |
| PR-074 | Gate approval workflow | `murmur-core/src/epic/gates.rs` |
| PR-075 | Issue-to-epic linking | `murmur-db/src/repos/epics.rs` |
| PR-076 | CLI: `murmur epic` commands | `murmur-cli/src/commands/epic.rs` |

---

### Phase 13: Sangha Governance (Voting)
*Goal: Democratic agent decision-making*

| PR | Description | Files |
|----|-------------|-------|
| PR-077 | Proposal creation | `murmur-core/src/governance/proposal.rs` |
| PR-078 | Voting mechanism | `murmur-core/src/governance/voting.rs` |
| PR-079 | Consensus calculation | `murmur-core/src/governance/consensus.rs` |
| PR-080 | Proposal execution | `murmur-core/src/governance/execute.rs` |
| PR-081 | Human override (force/veto) | `murmur-core/src/governance/override.rs` |

---

### Phase 14: TUI
*Goal: Terminal UI for monitoring*

| PR | Description | Files |
|----|-------------|-------|
| PR-082 | TUI framework (ratatui) | `murmur-tui/src/` |
| PR-083 | Dashboard view | `murmur-tui/src/views/dashboard.rs` |
| PR-084 | Agent status view | `murmur-tui/src/views/agents.rs` |
| PR-085 | Workflow progress view | `murmur-tui/src/views/workflow.rs` |
| PR-086 | Log viewer | `murmur-tui/src/views/logs.rs` |

---

### Phase 15: Web UI
*Goal: Browser-based monitoring and control*

| PR | Description | Files |
|----|-------------|-------|
| PR-087 | REST API | `murmur-server/src/api/` |
| PR-088 | WebSocket events | `murmur-server/src/websocket/` |
| PR-089 | Frontend setup | `web/` |
| PR-090 | Dashboard page | `web/src/pages/` |
| PR-091 | Agent terminal streaming | `web/src/components/Terminal.tsx` |

---

### Phase 16: Polish & Production
*Goal: Production-ready*

| PR | Description | Files |
|----|-------------|-------|
| PR-092 | Property tests (proptest) | `tests/` |
| PR-093 | Error recovery | Various |
| PR-094 | Graceful shutdown | Various |
| PR-095 | Documentation | `docs/` |
| PR-096 | Installation scripts | `scripts/` |

---

## Issue Dependencies & PR Verification

Murmuration understands issue dependencies and blocks work until prerequisites are met.

### Dependency Syntax (in issue body)

```markdown
## Dependencies
- Depends on #12
- Blocked by #15
- Parent: #8

## Metadata
<!-- murmur:metadata
{
  "phase": 3,
  "pr_number": null,
  "status": "ready"
}
-->
```

### Dependency Resolution Flow

```
murmur work 42
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. Parse issue #42 dependencies                             │
│    - Extract "Depends on #X" references                     │
│    - Extract "Blocked by #X" references                     │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. For each dependency, check status:                       │
│    - Is the issue closed?                                   │
│    - Does it have a linked PR?                              │
│    - Is that PR merged?                                     │
└─────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. If ALL dependencies satisfied:                           │
│    → Proceed with work                                      │
│                                                             │
│    If ANY dependency NOT satisfied:                         │
│    → Show blocking issues                                   │
│    → Optionally: work on unblocked dependency first         │
└─────────────────────────────────────────────────────────────┘
```

### CLI Output Example

```
$ murmur work 42

Issue #42: Implement TDD workflow
Status: ready

Dependencies:
  ✅ #38 - Agent type definitions [PR #51 merged]
  ✅ #39 - System prompt loading [PR #52 merged]
  ❌ #40 - Context building [PR #53 open, not merged]

Blocked by 1 unmerged dependency.

Options:
  1. Wait for PR #53 to merge
  2. Run `murmur work 40` to help finish the blocking issue
  3. Run `murmur work 42 --force` to proceed anyway (not recommended)
```

### Dependency Graph Visualization

```
$ murmur plan status

Phase 1: Minimal CLI ━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% [5/5]
  ✅ #1 Rust workspace
  ✅ #2 Basic CLI
  ✅ #3 Agent spawning
  ✅ #4 Output streaming
  ✅ #5 Basic config

Phase 2: Git Worktrees ━━━━━━━━━━━━━━━━━━━━━━━━ 60% [3/5]
  ✅ #6 Git detection
  ✅ #7 Branching point
  🔄 #8 Worktree creation ← IN PROGRESS
  ⏳ #9 Cache management (blocked by #8)
  ⏳ #10 Clone on demand (blocked by #8)

Phase 3: GitHub Integration ━━━━━━━━━━━━━━━━━━ 0% [0/8]
  ⏳ #11 API client (blocked by #5)
  ...
```

---

## Worktree Intelligence

The worktree system must be smart about branching:

```
Worktree Creation Flow:
┌─────────────────────────────────────────────────────────────┐
│ murmur work <issue> --repo <url-or-path>                    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. Is repo already cloned in cache?                         │
│    ~/.cache/murmur/repos/<owner>/<repo>/                    │
│    NO → Clone it                                            │
│    YES → Continue                                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Fetch latest from origin                                 │
│    git fetch origin                                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Determine branching point                                │
│    - Default: origin/main or origin/master                  │
│    - If issue specifies base branch, use that               │
│    - If continuing existing work, find merge-base           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Check worktree cache                                     │
│    ~/.cache/murmur/worktrees/<repo>/<issue-id>/             │
│    EXISTS + CLEAN → Reuse, rebase onto new base             │
│    EXISTS + DIRTY → Warn, ask to stash or new               │
│    NOT EXISTS → Create new                                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Create/update worktree                                   │
│    git worktree add <path> -b murmur/<issue> <base>         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Agent works in worktree                                  │
└─────────────────────────────────────────────────────────────┘
```

**Cache Structure:**
```
~/.cache/murmur/
├── repos/                    # Cloned repositories
│   └── <owner>/
│       └── <repo>/           # Bare or regular clone
├── worktrees/                # Active worktrees
│   └── <owner>-<repo>/
│       └── <issue-id>/       # Worktree for issue
└── config.toml               # Cache settings
```

**Cleanup Policy:**
- LRU eviction when cache exceeds size limit
- Completed worktrees retained for N days (configurable)
- `murmur worktree clean` for manual cleanup
- `murmur worktree clean --all` to nuke everything

---

## Crate Structure (Renamed to murmur-*)

```
murmuration/
├── Cargo.toml                 # Workspace
├── murmur-core/               # Core logic, types, workflows
├── murmur-cli/                # CLI binary
├── murmur-github/             # GitHub API integration
├── murmur-db/                 # SQLite persistence (post-bootstrap)
├── murmur-server/             # HTTP/WebSocket server (post-bootstrap)
├── murmur-tui/                # Terminal UI (post-bootstrap)
├── prompts/                   # Agent system prompts
│   ├── coder.md
│   ├── reviewer.md
│   ├── test.md
│   └── coordinator.md
├── web/                       # Web UI (post-bootstrap)
└── design/                    # Design documents
```

---

## Progress Tracking

| Phase | Description | PRs | Status |
|-------|-------------|-----|--------|
| 1 | Minimal CLI + Agent Spawning | PR-001 to PR-005 | ✅ |
| 2 | Git Worktrees (Smart) | PR-006 to PR-011 | ✅ |
| 3 | GitHub Integration + Dependencies | PR-012 to PR-018 | ✅ |
| 3b | Plan Import to GitHub | PR-019 to PR-022 | ✅ |
| 3.5 | Persistence (SQLite) | PR-023 to PR-027 | 🔲 |
| 4 | Agent Types + Prompts | PR-028 to PR-034 | 🔲 |
| 5 | TDD Workflow (Red-Green) | PR-035 to PR-039 | 🔲 |
| 6 | Review Agents at Each Phase | PR-040 to PR-045 | 🔲 |
| 7 | Coordinator Agent | PR-046 to PR-051 | 🔲 |
| **🎯** | **BOOTSTRAP MILESTONE** | | 🔲 |
| 8 | GitHub Integration (Write) | PR-052 to PR-056 | 🔲 |
| 9 | Background Daemon | PR-057 to PR-061 | 🔲 |
| 10 | Human Override & Control | PR-062 to PR-066 | 🔲 |
| 11 | Additional Agent Types | PR-067 to PR-071 | 🔲 |
| 12 | Epics & Stages | PR-072 to PR-076 | 🔲 |
| 13 | Sangha Governance | PR-077 to PR-081 | 🔲 |
| 14 | TUI | PR-082 to PR-086 | 🔲 |
| 15 | Web UI | PR-087 to PR-091 | 🔲 |
| 16 | Polish & Production | PR-092 to PR-096 | 🔲 |

**Bootstrap = 9 phases (including 3b and 3.5), ~51 PRs**
**Full system = 17 phases, ~96 PRs**

---

## Next Steps

1. Begin Phase 1: Minimal CLI + Agent Spawning
2. Get to `murmur run "task"` working
3. Add worktrees, GitHub, TDD workflow
4. Reach bootstrap milestone
5. **Use Murmuration to build the rest of Murmuration**
