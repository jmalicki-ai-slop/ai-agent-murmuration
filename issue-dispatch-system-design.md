# Issue-Based Agent Dispatch System

## Design Document

A self-organizing multi-agent orchestration system with issue-based task dispatch, GitHub integration, and autonomous governance.

---

## 1. High-Level Architecture Requirements

### 1.1 Core Principles

1. **Issue as Central Entity** - Every piece of work is an Issue with full lifecycle tracking
2. **GitHub as UI** - Leverage GitHub Issues/PRs for web+mobile interface, no custom UI needed
3. **Local Execution** - Agents run locally on your infrastructure, not cloud
4. **Autonomous Governance** - Agents propose, vote, and decide without human gating
5. **Human Observability** - Humans monitor via GitHub, can intervene but don't have to
6. **Rust Implementation** - Performance, safety, single binary distribution

### 1.2 Functional Requirements

| Requirement | Description |
|-------------|-------------|
| **FR-1** | Create issues via CLI or GitHub, system picks them up |
| **FR-2** | Issues track: repo, worktree path, agent assignment, status, PR link |
| **FR-3** | Agents are specialized (coder, reviewer, PM, security, packaging) |
| **FR-4** | Dashboard shows in-flight issues with agent status |
| **FR-5** | Agents vote on implementation approaches for issues |
| **FR-6** | Agents propose and vote on system improvements |
| **FR-7** | Consensus decisions execute automatically |
| **FR-8** | PRs auto-link to issues, issues update when PRs merge |
| **FR-9** | **Epics** break into child issues with defined handoff stages |
| **FR-10** | **Human gates** at handoff stages - agent pauses, human approves to continue |
| **FR-11** | **Human override** - force any decision, reassign, cancel, or direct agents |

### 1.3 Non-Functional Requirements

| Requirement | Description |
|-------------|-------------|
| **NFR-1** | Offline-capable: works without GitHub (sync when online) |
| **NFR-2** | Persistent: survives restarts, state in SQLite |
| **NFR-3** | Observable: structured logs, TUI for local monitoring |
| **NFR-4** | Extensible: add new agent types, new governance rules |
| **NFR-5** | Secure: GitHub tokens scoped minimally, no secrets in issues |

---

## 2. System Architecture

### 2.1 High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                              GitHub                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │   Issues    │  │    PRs      │  │  Webhooks   │                 │
│  │  (UI/State) │  │  (Output)   │  │  (Events)   │                 │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                 │
└─────────┼────────────────┼────────────────┼─────────────────────────┘
          │                │                │
          │ sync           │ create         │ POST
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Orchestrator (Rust)                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      GitHub Sync Layer                       │   │
│  │            (webhook receiver + gh API client)                │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      Issue Manager                           │   │
│  │         (SQLite persistence, state machine, queue)           │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Sangha Governance                         │   │
│  │      (proposals, voting, consensus, execution)               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Agent Dispatcher                          │   │
│  │          (assignment, worktree setup, lifecycle)             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│       ┌───────────┬───────────┼───────────┬───────────┐            │
│       ▼           ▼           ▼           ▼           ▼            │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐      │
│  │  Coder  │ │Reviewer │ │   PM    │ │Security │ │  Docs   │      │
│  │  Agent  │ │  Agent  │ │  Agent  │ │  Agent  │ │  Agent  │      │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘      │
│       │           │           │           │           │            │
│       └───────────┴───────────┴───────────┴───────────┘            │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   Claude Code Interface                      │   │
│  │         (subprocess spawn, ACP, or SDK - TBD)                │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   Git Worktree Manager                       │   │
│  │        (create, track, cleanup worktrees per issue)          │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Responsibilities

#### GitHub Sync Layer
- Receive webhooks (issues.opened, issues.labeled, issue_comment.created, pull_request.*)
- Poll for changes when webhooks unavailable
- Push state updates back to GitHub (comments, labels, PR links)
- Handle rate limiting and retries

#### Issue Manager
- SQLite persistence for all issues
- State machine: `unassigned → assigned → in_progress → review → done`
- Queue management: priority ordering, dependency resolution
- Sync bidirectionally with GitHub Issues

#### Sangha Governance
- Collect proposals from agents (implementation approaches, system improvements)
- Broadcast proposals to all relevant agents
- Collect votes with reasoning
- Calculate consensus (configurable threshold)
- Execute approved proposals
- Log decisions for audit trail

#### Agent Dispatcher
- Match issues to appropriate agent type(s)
- Spawn agent processes with issue context
- Track agent health (heartbeat, progress)
- Handle agent failures (retry, reassign)

#### Agents (Coder, Reviewer, PM, Security, Docs)
- Each runs as Claude Code subprocess in dedicated worktree
- Receives issue prompt as initial context
- Reports progress back to dispatcher
- Can propose votes (implementation choices, system improvements)
- Creates branches, commits, PRs

#### Claude Code Interface
- Abstract how we talk to Claude Code
- Options: subprocess `claude` CLI, ACP WebSocket, direct SDK
- Session persistence (resume interrupted work)

#### Git Worktree Manager
- Create worktree per issue: `/repo-worktrees/issue-{id}/`
- Track worktree ↔ issue mapping
- Cleanup on issue completion
- Handle branch naming: `issue/{id}-{slug}`

---

## 3. Data Models

### 3.1 Epic

```rust
pub struct Epic {
    pub id: EpicId,
    pub github_id: Option<u64>,         // GitHub issue with "epic" label
    pub github_url: Option<String>,

    // Content
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,

    // Repository
    pub repo_path: PathBuf,
    pub repo_url: Option<String>,

    // Decomposition
    pub child_issues: Vec<IssueId>,     // Ordered list of child issues
    pub stages: Vec<Stage>,             // Handoff stages with gates

    // Status
    pub status: EpicStatus,
    pub current_stage: usize,           // Index into stages
    pub blocked_at_gate: Option<GateId>, // If waiting for human

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum EpicStatus {
    Draft,              // Being planned
    Ready,              // Has stages and child issues
    InProgress,         // Agents working on it
    AwaitingGate,       // Paused at human gate
    Completed,
    Cancelled,
}

pub struct Stage {
    pub id: StageId,
    pub name: String,               // "Design", "Implementation", "Review", "Deploy"
    pub description: String,
    pub issues: Vec<IssueId>,       // Issues that belong to this stage
    pub gate: Option<Gate>,         // Human approval gate at end of stage
    pub status: StageStatus,
}

pub enum StageStatus {
    Pending,
    InProgress,
    AwaitingGate,
    Approved,
    Skipped,
}

pub struct Gate {
    pub id: GateId,
    pub gate_type: GateType,
    pub description: String,
    pub required_approvers: Vec<String>,   // GitHub usernames or "any"
    pub approval_status: GateApproval,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub comments: Vec<GateComment>,
}

pub enum GateType {
    Approval,           // Human must explicitly approve
    Review,             // Human must review artifacts
    Checkpoint,         // Informational pause, auto-continues after timeout
    Decision,           // Human must choose between options
}

pub enum GateApproval {
    Pending,
    Approved,
    Rejected,
    RequestedChanges,
}

pub struct GateComment {
    pub author: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

### 3.2 Issue

```rust
pub struct Issue {
    // Identity
    pub id: IssueId,                    // Local UUID
    pub github_id: Option<u64>,         // GitHub issue number
    pub github_url: Option<String>,     // Full GitHub URL

    // Epic relationship
    pub parent_epic: Option<EpicId>,    // If part of an epic
    pub stage: Option<StageId>,         // Which stage in the epic

    // Repository
    pub repo_path: PathBuf,             // Local master repo path
    pub repo_url: Option<String>,       // GitHub repo URL
    pub worktree_path: Option<PathBuf>, // Created when assigned
    pub branch_name: Option<String>,    // issue/42-add-auth

    // Content
    pub title: String,
    pub prompt: String,                 // Full description, serves as agent memory
    pub issue_type: IssueType,          // Feature, Bug, Docs, Refactor, Test
    pub priority: Priority,             // High, Medium, Low
    pub labels: Vec<String>,

    // Assignment
    pub status: IssueStatus,
    pub assigned_agent: Option<AgentId>,
    pub agent_type: Option<AgentType>,  // Coder, Reviewer, etc.

    // Outputs
    pub linked_pr: Option<PullRequest>,
    pub commits: Vec<CommitRef>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    // Governance
    pub proposals: Vec<ProposalId>,     // Related proposals
    pub decision_log: Vec<Decision>,    // Votes that affected this issue
}

pub enum IssueStatus {
    Unassigned,
    Queued,
    Assigned,
    InProgress,
    AwaitingReview,
    InReview,
    Done,
    Blocked,
    Cancelled,
}

pub enum IssueType {
    Feature,
    Bug,
    Docs,
    Refactor,
    Test,
    Security,
    Chore,
}

pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}
```

### 3.2 Agent

```rust
pub struct Agent {
    pub id: AgentId,                    // UUID for this session
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub current_issue: Option<IssueId>,
    pub worktree_path: Option<PathBuf>,
    pub process_id: Option<u32>,        // OS PID
    pub claude_session_id: Option<String>, // For resume
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub metrics: AgentMetrics,
}

pub enum AgentType {
    Coder,          // Implements features and fixes
    Reviewer,       // Reviews code for quality
    PM,             // Triages, prioritizes, breaks down epics
    Security,       // Security review, vulnerability scanning
    Docs,           // Documentation writer
    Test,           // Test writer
    Architect,      // Design decisions, technical proposals
}

pub enum AgentStatus {
    Idle,
    Starting,
    Working,
    WaitingForInput,    // Needs human or vote resolution
    WaitingForVote,     // Proposed something, awaiting votes
    Paused,
    Errored,
    Completed,
}

pub struct AgentMetrics {
    pub issues_completed: u32,
    pub avg_completion_time: Duration,
    pub tokens_used: u64,
    pub proposals_made: u32,
    pub votes_cast: u32,
}
```

### 3.3 Proposal (Sangha)

```rust
pub struct Proposal {
    pub id: ProposalId,
    pub proposal_type: ProposalType,
    pub proposer: AgentId,
    pub title: String,
    pub description: String,
    pub rationale: String,

    // Context
    pub related_issue: Option<IssueId>,  // If about implementation
    pub affected_components: Vec<String>, // If system improvement

    // Voting
    pub status: ProposalStatus,
    pub votes: Vec<Vote>,
    pub required_voters: Vec<AgentType>, // Who should vote
    pub threshold: ConsensusThreshold,

    // Execution
    pub implementation_plan: Option<String>,
    pub rollback_plan: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub execution_result: Option<ExecutionResult>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub voting_deadline: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub enum ProposalType {
    // Implementation decisions (per-issue)
    ImplementationApproach,     // How to solve this issue
    TechStackChoice,            // Which library/tool to use
    ArchitectureDecision,       // Design pattern, structure

    // System improvements (global)
    NewAgentType,               // Add a new specialist
    WorkflowChange,             // Change how issues flow
    GovernanceRule,             // Change voting rules
    ToolIntegration,            // Add new tool/MCP server
    PromptImprovement,          // Better agent instructions
}

pub enum ProposalStatus {
    Open,
    Voting,
    Approved,
    Rejected,
    Executing,
    Executed,
    RolledBack,
}

pub struct Vote {
    pub voter: AgentId,
    pub voter_type: AgentType,
    pub decision: VoteDecision,
    pub reasoning: String,
    pub confidence: f32,        // 0.0 - 1.0
    pub voted_at: DateTime<Utc>,
}

pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
    NeedMoreInfo,
}

pub enum ConsensusThreshold {
    Unanimous,
    SuperMajority,      // 2/3
    SimpleMajority,     // > 50%
    SingleApproval,     // Any one yes
}
```

### 3.4 Pull Request Link

```rust
pub struct PullRequest {
    pub github_number: u64,
    pub github_url: String,
    pub branch: String,
    pub status: PRStatus,
    pub checks_passing: bool,
    pub review_status: ReviewStatus,
    pub merged_at: Option<DateTime<Utc>>,
}

pub enum PRStatus {
    Draft,
    Open,
    Merged,
    Closed,
}

pub enum ReviewStatus {
    Pending,
    Approved,
    ChangesRequested,
    Dismissed,
}
```

---

## 4. Workflows

### 4.1 Issue Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                      Issue Created                               │
│              (via GitHub UI or CLI)                              │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Unassigned                                  │
│         Issue Manager receives, persists to SQLite               │
│         PM Agent may triage, add labels, estimate                │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Dispatcher assigns based on type/priority
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Assigned                                   │
│         Worktree created: /worktrees/issue-{id}/                 │
│         Branch created: issue/{id}-{slug}                        │
│         Agent spawned with issue prompt as context               │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      In Progress                                 │
│         Agent working, may:                                      │
│         - Propose implementation approach → Sangha vote          │
│         - Request info → Comment on issue                        │
│         - Make commits → Push to branch                          │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Agent creates PR
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Awaiting Review                               │
│         PR opened, linked to issue                               │
│         Reviewer Agent assigned                                  │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      In Review                                   │
│         Reviewer Agent examines code                             │
│         Security Agent runs checks                               │
│         May request changes → back to In Progress                │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Approved & merged
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Done                                      │
│         PR merged, issue closed                                  │
│         Worktree cleaned up                                      │
│         Metrics recorded                                         │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Sangha Voting Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                  Agent Encounters Decision                       │
│    "Should I use JWT or session-based auth for issue #42?"       │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Create Proposal                                 │
│    Type: ImplementationApproach                                  │
│    Options: [JWT, Session-based]                                 │
│    Required voters: [Architect, Security, Coder]                 │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Broadcast to Voters                             │
│    Each voter agent receives proposal context                    │
│    Can request more info or research                             │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Collect Votes                                   │
│    Architect: JWT (+1) "Stateless, scales better"                │
│    Security: JWT (+1) "Easier to revoke with short expiry"       │
│    Coder: JWT (+1) "Library support is excellent"                │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Consensus reached
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Execute Decision                                │
│    Proposal approved: JWT                                        │
│    Decision logged to issue                                      │
│    Original agent proceeds with JWT implementation               │
└─────────────────────────────────────────────────────────────────┘
```

### 4.3 Epic with Handoff Gates

```
┌─────────────────────────────────────────────────────────────────┐
│                      Epic Created                                │
│          "Build user authentication system"                      │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                   PM Agent Decomposes                            │
│    Creates stages with gates:                                    │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │ Stage 1: Design                                          │  │
│    │   - Issue: "Design auth API schema"                      │  │
│    │   - Issue: "Choose auth library"                         │  │
│    │   - GATE: Human reviews design docs [Approval]           │  │
│    ├─────────────────────────────────────────────────────────┤  │
│    │ Stage 2: Implementation                                  │  │
│    │   - Issue: "Implement JWT tokens"                        │  │
│    │   - Issue: "Implement refresh flow"                      │  │
│    │   - Issue: "Add auth middleware"                         │  │
│    │   - GATE: Human reviews implementation [Review]          │  │
│    ├─────────────────────────────────────────────────────────┤  │
│    │ Stage 3: Testing                                         │  │
│    │   - Issue: "Write auth unit tests"                       │  │
│    │   - Issue: "Write auth integration tests"                │  │
│    │   - GATE: Human verifies test coverage [Checkpoint]      │  │
│    ├─────────────────────────────────────────────────────────┤  │
│    │ Stage 4: Deploy                                          │  │
│    │   - Issue: "Deploy auth to staging"                      │  │
│    │   - Issue: "Update API docs"                             │  │
│    │   - GATE: Human approves production deploy [Approval]    │  │
│    └─────────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Stage 1: Design (InProgress)                     │
│    Agents work on design issues...                               │
│    All Stage 1 issues complete                                   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                 GATE: Human Review Required                      │
│    Epic status: AwaitingGate                                     │
│    GitHub comment posted: "@human please review design docs"     │
│    Notification sent (webhook, email, Slack)                     │
│                                                                  │
│    Human reviews, can:                                           │
│    - Approve → Continue to Stage 2                               │
│    - Request changes → Reopen design issues                      │
│    - Reject → Cancel or major rework                             │
│    - Add comments → Fed to agents as context                     │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Human approves
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Stage 2: Implementation (InProgress)             │
│    ... continues through stages and gates ...                    │
└─────────────────────────────────────────────────────────────────┘
```

### 4.4 Human Override Flow

```
┌─────────────────────────────────────────────────────────────────┐
│            Human Can Override At Any Point                       │
└─────────────────────────────────────────────────────────────────┘

OVERRIDE MECHANISMS:

1. FORCE DECISION (bypass Sangha vote)
   ┌──────────────────────────────────────────────────────────┐
   │  Agent proposes: "Use JWT vs Session auth?"               │
   │  Votes in progress...                                     │
   │                                                           │
   │  Human: dispatch proposal force 123 --choice jwt          │
   │         --reason "Company policy requires JWT"            │
   │                                                           │
   │  Result: Vote cancelled, JWT decision recorded            │
   │          with human override flag                         │
   └──────────────────────────────────────────────────────────┘

2. REASSIGN ISSUE
   ┌──────────────────────────────────────────────────────────┐
   │  Issue #42 assigned to Coder agent, stuck                 │
   │                                                           │
   │  Human: dispatch issue reassign 42 --agent security       │
   │         --reason "Needs security expertise"               │
   │                                                           │
   │  Result: Coder agent stopped, Security agent takes over   │
   │          Full context transferred                         │
   └──────────────────────────────────────────────────────────┘

3. DIRECT INSTRUCTION
   ┌──────────────────────────────────────────────────────────┐
   │  Human comments on GitHub issue:                          │
   │  "@dispatch Use the existing UserService, don't create    │
   │   a new one"                                              │
   │                                                           │
   │  Result: Instruction fed to agent as high-priority        │
   │          context, agent adjusts approach                  │
   └──────────────────────────────────────────────────────────┘

4. PAUSE/RESUME
   ┌──────────────────────────────────────────────────────────┐
   │  Human: dispatch agent pause agent-abc123                 │
   │         --reason "Need to review direction"               │
   │                                                           │
   │  ... human reviews, adds comments ...                     │
   │                                                           │
   │  Human: dispatch agent resume agent-abc123                │
   │                                                           │
   │  Result: Agent resumes with new context                   │
   └──────────────────────────────────────────────────────────┘

5. CANCEL/ABORT
   ┌──────────────────────────────────────────────────────────┐
   │  Human: dispatch issue cancel 42                          │
   │         --reason "Requirements changed"                   │
   │                                                           │
   │  Result: Agent stopped, worktree preserved (or cleaned),  │
   │          issue closed with cancellation note              │
   └──────────────────────────────────────────────────────────┘

6. SKIP GATE
   ┌──────────────────────────────────────────────────────────┐
   │  Epic waiting at gate, human trusts the work              │
   │                                                           │
   │  Human: dispatch epic gate skip epic-123 --gate design    │
   │         --reason "Design reviewed offline"                │
   │                                                           │
   │  Result: Gate marked skipped, continues to next stage     │
   └──────────────────────────────────────────────────────────┘

7. VETO SYSTEM CHANGE
   ┌──────────────────────────────────────────────────────────┐
   │  Agents voted to add "Blockchain Agent" (bad idea)        │
   │  Proposal approved, about to execute                      │
   │                                                           │
   │  Human: dispatch proposal veto 456                        │
   │         --reason "Not aligned with project goals"         │
   │                                                           │
   │  Result: Execution cancelled, proposal marked vetoed      │
   └──────────────────────────────────────────────────────────┘
```

### 4.5 System Improvement Flow

```
┌─────────────────────────────────────────────────────────────────┐
│              Agent Identifies Improvement                        │
│    "We keep having packaging issues. Propose: Packaging Agent"   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Create System Proposal                          │
│    Type: NewAgentType                                            │
│    Title: "Add Packaging Agent"                                  │
│    Description: Agent specialized in npm/cargo/pip packaging     │
│    Required voters: ALL active agents                            │
│    Threshold: SuperMajority                                      │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Research Phase (Optional)                       │
│    Agents may research implications                              │
│    Add findings to proposal discussion                           │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Voting                                          │
│    PM: +1 "Addresses recurring pain point"                       │
│    Coder: +1 "Will save time on dependency issues"               │
│    Security: +1 "Can audit dependencies too"                     │
│    Reviewer: 0 (Abstain)                                         │
│    Architect: +1 "Clean separation of concerns"                  │
└─────────────────────────┬───────────────────────────────────────┘
                          │ 4/5 = 80% > 66% threshold
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Auto-Execute                                    │
│    1. Create Packaging Agent configuration                       │
│    2. Add to agent registry                                      │
│    3. Update dispatcher rules                                    │
│    4. Log decision for audit                                     │
│    (Rollback plan: Remove config, revert dispatcher)             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. GitHub Integration

### 5.1 Issue Metadata Storage

Since GitHub Issues lack custom fields, store metadata in issue body:

```markdown
<!-- orchestrator:meta
{
  "local_id": "550e8400-e29b-41d4-a716-446655440000",
  "worktree": "/home/user/project-worktrees/issue-42",
  "agent_id": "agent-abc123",
  "agent_type": "Coder",
  "status": "in_progress",
  "branch": "issue/42-add-auth"
}
-->

## Task

Implement JWT authentication with refresh tokens for the API.

### Requirements
- Access tokens expire in 15 minutes
- Refresh tokens expire in 7 days
- Store refresh tokens in Redis
- Add /auth/login, /auth/refresh, /auth/logout endpoints

### Acceptance Criteria
- [ ] Tokens are properly signed
- [ ] Refresh rotation works
- [ ] Logout invalidates refresh token
```

### 5.2 Webhook Events

| Event | Action |
|-------|--------|
| `issues.opened` | Create local issue, queue for assignment |
| `issues.labeled` | Update priority/type |
| `issues.assigned` | (If human assigns, respect it) |
| `issues.closed` | Mark done, cleanup worktree |
| `issue_comment.created` | Feed to agent as context, or human instruction |
| `pull_request.opened` | Link PR to issue |
| `pull_request.closed` | If merged, close issue |
| `pull_request_review.*` | Update review status |

### 5.3 Webhook Receiver

```
POST /webhook/github
Headers:
  X-GitHub-Event: issues
  X-Hub-Signature-256: sha256=...

Body: { action: "opened", issue: { ... } }
```

Expose via:
- Cloudflare Tunnel (your preference)
- ngrok
- Direct if you have public IP

---

## 6. CLI Interface

```bash
# Epic management
dispatch epic create "Build auth system" --repo /path/to/repo
dispatch epic decompose <id>              # PM agent breaks into stages/issues
dispatch epic list [--status in_progress]
dispatch epic show <id>                   # Shows stages, gates, progress
dispatch epic gate approve <epic-id> --gate <gate-id> [--comment "Looks good"]
dispatch epic gate reject <epic-id> --gate <gate-id> --reason "Need X"
dispatch epic gate skip <epic-id> --gate <gate-id> --reason "Reviewed offline"

# Issue management
dispatch issue create "Add user auth" --type feature --priority high --repo /path/to/repo
dispatch issue create "Implement JWT" --epic <epic-id> --stage implementation
dispatch issue list [--status in_progress] [--agent coder] [--epic <id>]
dispatch issue show <id>
dispatch issue assign <id> --agent coder
dispatch issue reassign <id> --agent security --reason "Needs expertise"
dispatch issue cancel <id> --reason "Requirements changed"

# Agent management
dispatch agent list
dispatch agent status <id>
dispatch agent logs <id> [--follow]
dispatch agent pause <id>
dispatch agent resume <id>

# Governance
dispatch proposal list [--status open]
dispatch proposal show <id>
dispatch proposal vote <id> --approve --reason "Good approach"  # Manual human vote
dispatch proposal force <id> --choice <option> --reason "Policy"  # Override vote
dispatch proposal veto <id> --reason "Not aligned with goals"     # Cancel approved
dispatch decision-log [--issue <id>]

# Worktree management
dispatch worktree list
dispatch worktree cleanup [--dry-run]

# GitHub sync
dispatch sync [--force]
dispatch webhook-test

# System
dispatch status              # Dashboard summary
dispatch config show
dispatch config set <key> <value>
```

### TUI Mode

```bash
dispatch tui
```

Shows:
- In-flight issues with agent assignments
- Agent status (working, idle, waiting)
- Active proposals awaiting votes
- Recent decisions
- PR status

---

## 7. Configuration

```toml
# ~/.config/dispatch/config.toml

[general]
data_dir = "~/.local/share/dispatch"
log_level = "info"

[github]
token_env = "GITHUB_TOKEN"       # Or token_file
webhook_secret_env = "GITHUB_WEBHOOK_SECRET"
default_repo = "owner/repo"
sync_interval_secs = 60

[agents]
max_concurrent = 4
default_model = "claude-sonnet-4"
worktree_base = "~/worktrees"

[agents.types.coder]
enabled = true
model = "claude-sonnet-4"
system_prompt_file = "~/.config/dispatch/prompts/coder.md"

[agents.types.reviewer]
enabled = true
model = "claude-sonnet-4"
system_prompt_file = "~/.config/dispatch/prompts/reviewer.md"

[agents.types.security]
enabled = true
model = "claude-sonnet-4"
system_prompt_file = "~/.config/dispatch/prompts/security.md"

[governance]
default_threshold = "simple_majority"
voting_timeout_hours = 24
implementation_proposals_require = ["coder", "architect"]
system_proposals_require_all = true

[governance.auto_approve]
# Minor changes don't need full vote
prompt_improvements = "single_approval"
```

---

## 8. Implementation Phases

### Phase 1: Foundation
- [ ] Project setup (Rust workspace, dependencies)
- [ ] SQLite schema and migrations
- [ ] Issue data model and persistence
- [ ] Basic CLI (create, list, show issues)
- [ ] Git worktree manager

### Phase 2: Agent Execution
- [ ] Agent data model
- [ ] Claude Code subprocess spawning
- [ ] Agent lifecycle management (start, monitor, stop)
- [ ] Issue → Agent assignment logic
- [ ] Basic TUI showing agents and issues

### Phase 3: GitHub Integration
- [ ] GitHub API client (issues, PRs, comments)
- [ ] Webhook receiver (axum or actix-web)
- [ ] Bidirectional sync (local ↔ GitHub)
- [ ] PR linking and status tracking
- [ ] Metadata in issue body

### Phase 4: Sangha Governance
- [ ] Proposal data model
- [ ] Voting mechanism
- [ ] Consensus calculation
- [ ] Agent-to-agent proposal broadcast
- [ ] Decision logging

### Phase 5: Self-Improvement
- [ ] System improvement proposals
- [ ] Auto-execution of approved changes
- [ ] Rollback capability
- [ ] Governance rule modifications

### Phase 6: Polish
- [ ] Full TUI with all views
- [ ] Metrics and reporting
- [ ] Documentation
- [ ] Error recovery and edge cases

### Phase 7: Web UI
- [ ] Basic web dashboard (axum + WebSocket)
- [ ] Real-time agent status, issue updates
- [ ] Gate approval UI
- [ ] Proposal voting UI

### Phase 8: Interactive Watch Mode
- [ ] PTY-based agent execution (instead of simple subprocess)
- [ ] WebSocket terminal streaming (xterm.js on frontend)
- [ ] Live watch mode - see agent working in real-time
- [ ] Bidirectional interaction - send input to running agent
- [ ] Attach/detach to any running agent from browser

---

## 9. Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust | Performance, safety, single binary |
| Async runtime | tokio | Standard for Rust async |
| Database | SQLite via **sqlx** | Compile-time checked queries, async API, built-in migrations |
| CLI | clap | Standard Rust CLI library |
| TUI | ratatui | Modern Rust TUI, active development |
| HTTP server | axum | Async, ergonomic, WebSocket support for webhooks + web UI |
| HTTP client | reqwest | GitHub API calls |
| GitHub API | octocrab | Typed GitHub API client |
| Serialization | serde + serde_json | JSON for GitHub, config |
| Git operations | git2 | libgit2 bindings, worktree support |
| Process management | tokio::process | Spawn and manage Claude Code |
| PTY (Phase 8) | portable-pty or tokio-pty | Terminal emulation for watch mode |
| WebSocket | axum + tokio-tungstenite | Real-time frontend communication |

---

## 10. Open Questions

1. **Claude Code communication**: Subprocess CLI vs ACP vs SDK?
2. **Agent memory**: Just issue prompt, or maintain conversation history?
3. **Conflict resolution**: What if two agents edit same file?
4. **Cost controls**: Token budgets per issue/agent?
5. **Human override**: How to handle human pushing to agent's branch?
6. **Multi-repo**: Support multiple repos or one at a time?

---

## 11. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Agents make bad decisions | Decision logging, rollback capability, can add human approval threshold |
| Runaway token costs | Per-issue token budgets, alerts at thresholds |
| Agent conflicts | Worktree isolation, file locking, Sangha coordination |
| GitHub rate limits | Caching, exponential backoff, webhook preference |
| Agent hangs | Heartbeat monitoring, timeout and restart |
| Data loss | SQLite WAL mode, periodic backups |

---

## 12. Success Metrics

- Issues completed per day
- Average time from issue creation to PR merge
- Proposal consensus rate
- Agent utilization (% time working vs idle)
- Self-improvement proposals approved and successful
- Human intervention rate (lower = more autonomous)
