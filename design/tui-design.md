# TUI Design

## Overview

Terminal UI using ratatui for real-time monitoring and control of the Dispatch system.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              TUI Application                             │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐               │
│  │   App State   │  │    Router     │  │  Event Loop   │               │
│  │               │  │               │  │               │               │
│  │ - views       │  │ - navigation  │  │ - key events  │               │
│  │ - data cache  │  │ - view stack  │  │ - tick events │               │
│  │ - selection   │  │ - modals      │  │ - db events   │               │
│  └───────────────┘  └───────────────┘  └───────────────┘               │
├─────────────────────────────────────────────────────────────────────────┤
│                               Views                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│  │Dashboard│ │ Issues  │ │ Agents  │ │  Epics  │ │Proposals│          │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘          │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐                                   │
│  │  Logs   │ │ Details │ │ Command │                                   │
│  └─────────┘ └─────────┘ └─────────┘                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                             Widgets                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│  │  Table  │ │ Status  │ │Progress │ │ Sparkln │ │  Modal  │          │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Screen Layout

### Dashboard View (Default)

```
┌─ Dispatch ──────────────────────────────────────────────────────────────┐
│ [D]ashboard  [I]ssues  [A]gents  [E]pics  [P]roposals  [L]ogs  [?]Help │
├─────────────────────────────────────────────────────────────────────────┤
│  System Status                          │  Active Agents               │
│  ──────────────────────────────────────│  ──────────────────────────── │
│  Issues:  12 total, 3 in progress       │  [coder-1]    Working  #42   │
│  Agents:  4 active, 2 idle              │  [coder-2]    Working  #45   │
│  Epics:   2 active, 1 at gate           │  [reviewer-1] Idle           │
│  PRs:     5 open, 2 ready               │  [test-1]     Working  #48   │
│  ──────────────────────────────────────│                               │
│  Last sync: 2 minutes ago               │                               │
├─────────────────────────────────────────┴───────────────────────────────┤
│  Recent Activity                                                         │
│  ────────────────────────────────────────────────────────────────────── │
│  14:32  Issue #48 assigned to test-1                                    │
│  14:30  PR #23 ready for review                                         │
│  14:28  Agent coder-1 started working on #42                            │
│  14:25  Epic "Auth System" reached gate: Design Review                  │
│  14:20  Proposal "Use JWT" approved (3/3 votes)                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Pending Gates                          │  Open Proposals               │
│  ──────────────────────────────────────│  ──────────────────────────── │
│  > Auth System: Design Review           │  > "Caching Strategy" 2/3    │
│    Payment Flow: Security Gate          │    "Error Format" 1/3        │
└─────────────────────────────────────────┴───────────────────────────────┤
│ q:Quit  r:Refresh  /:Search  Enter:Select                   14:35:22   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Issues View

```
┌─ Dispatch > Issues ─────────────────────────────────────────────────────┐
│ [D]ashboard  [I]ssues  [A]gents  [E]pics  [P]roposals  [L]ogs  [?]Help │
├─────────────────────────────────────────────────────────────────────────┤
│  Filter: [All] [In Progress] [Unassigned] [Blocked]    Sort: Priority↓ │
├─────────────────────────────────────────────────────────────────────────┤
│  ID      │ Title                        │ Status      │ Agent   │ Pri  │
│  ────────┼──────────────────────────────┼─────────────┼─────────┼───── │
│  #42     │ Implement OAuth login        │ In Progress │ coder-1 │ High │
│  #45     │ Add rate limiting            │ In Progress │ coder-2 │ Med  │
│ >#48     │ Write auth unit tests        │ In Progress │ test-1  │ Med  │
│  #51     │ Fix memory leak in parser    │ Unassigned  │ -       │ Crit │
│  #52     │ Update API documentation     │ Unassigned  │ -       │ Low  │
│  #53     │ Security audit: auth module  │ Blocked     │ sec-1   │ High │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Issue #48: Write auth unit tests                                       │
│  ────────────────────────────────────────────────────────────────────── │
│  Type: test    Epic: Auth System    Stage: Testing                      │
│  Branch: dispatch/48/write-auth-tests                                   │
│  PR: #24 (draft)                                                        │
│  ────────────────────────────────────────────────────────────────────── │
│  Write comprehensive unit tests for the authentication module...        │
└─────────────────────────────────────────────────────────────────────────┤
│ a:Assign  u:Unassign  p:Priority  s:Status  Enter:Details  Esc:Back    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Agents View

```
┌─ Dispatch > Agents ─────────────────────────────────────────────────────┐
│ [D]ashboard  [I]ssues  [A]gents  [E]pics  [P]roposals  [L]ogs  [?]Help │
├─────────────────────────────────────────────────────────────────────────┤
│  ID         │ Type     │ Status  │ Issue │ Runtime │ Tokens  │ Cost   │
│  ───────────┼──────────┼─────────┼───────┼─────────┼─────────┼─────── │
│ >coder-1    │ Coder    │ Working │ #42   │ 45m     │ 125.4k  │ $2.45  │
│  coder-2    │ Coder    │ Working │ #45   │ 23m     │ 67.2k   │ $1.12  │
│  reviewer-1 │ Reviewer │ Idle    │ -     │ -       │ -       │ -      │
│  test-1     │ Test     │ Working │ #48   │ 12m     │ 34.1k   │ $0.56  │
│  sec-1      │ Security │ Paused  │ #53   │ 8m      │ 22.3k   │ $0.38  │
│  arch-1     │ Architect│ Idle    │ -     │ -       │ -       │ -      │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Agent: coder-1                         │  Recent Output               │
│  ──────────────────────────────────────│  ──────────────────────────── │
│  Type:    Coder                         │  Reading src/auth/oauth.rs   │
│  Status:  Working                       │  Analyzing dependencies...   │
│  Issue:   #42 - Implement OAuth login   │  Writing implementation...   │
│  PID:     12345                         │  Running tests...            │
│  Session: claude-sess-abc123            │  ✓ 12 tests passed          │
│  ──────────────────────────────────────│                               │
│  Heartbeat: 5s ago                      │                               │
└─────────────────────────────────────────┴───────────────────────────────┤
│ s:Start  k:Stop  p:Pause  r:Resume  l:Logs  Enter:Details  Esc:Back    │
└─────────────────────────────────────────────────────────────────────────┘
```

### Epics View

```
┌─ Dispatch > Epics ──────────────────────────────────────────────────────┐
│ [D]ashboard  [I]ssues  [A]gents  [E]pics  [P]roposals  [L]ogs  [?]Help │
├─────────────────────────────────────────────────────────────────────────┤
│  Epic: Auth System                                                      │
│  ════════════════════════════════════════════════════════════════════  │
│                                                                         │
│  [████████████░░░░░░░░] 60% Complete                                   │
│                                                                         │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐             │
│  │ Design  │───>│  Impl   │───>│ Testing │───>│  Docs   │             │
│  │   ✓     │    │ ▶ Gate  │    │         │    │         │             │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘             │
│                      │                                                  │
│               [AWAITING APPROVAL]                                       │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Stage: Implementation                  │  Gate: Design Review         │
│  ──────────────────────────────────────│  ──────────────────────────── │
│  Issues in stage:                       │  Type: Approval              │
│    ✓ #40 - Design auth flow             │  Status: Pending             │
│    ✓ #41 - Design OAuth integration     │  Required: @lead, @security  │
│    ▶ #42 - Implement OAuth login        │  Approved: @lead (1/2)       │
│    ○ #43 - Implement session mgmt       │                               │
│    ○ #44 - Add remember me feature      │  [a] Approve  [r] Reject     │
└─────────────────────────────────────────┴───────────────────────────────┤
│ a:Approve Gate  r:Reject  s:Skip  c:Comment  Enter:Issue  Esc:Back     │
└─────────────────────────────────────────────────────────────────────────┘
```

### Proposals View

```
┌─ Dispatch > Proposals ──────────────────────────────────────────────────┐
│ [D]ashboard  [I]ssues  [A]gents  [E]pics  [P]roposals  [L]ogs  [?]Help │
├─────────────────────────────────────────────────────────────────────────┤
│  Open Proposals                                                         │
│  ────────────────────────────────────────────────────────────────────── │
│  ID     │ Title                    │ Type       │ Votes │ Deadline     │
│  ───────┼──────────────────────────┼────────────┼───────┼───────────── │
│ >prop-1 │ Use Redis for caching    │ Tech Stack │ 2/3   │ 2h remaining │
│  prop-2 │ Standardize error format │ Arch       │ 1/3   │ 5h remaining │
│  prop-3 │ Add rate limiter lib     │ Tool       │ 0/2   │ 12h remain   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  Proposal: Use Redis for caching                                        │
│  ════════════════════════════════════════════════════════════════════  │
│  Type: Tech Stack Choice    Threshold: Simple Majority                  │
│  Proposed by: architect-1   Related: #50                                │
│  ────────────────────────────────────────────────────────────────────── │
│  Description:                                                           │
│  We should use Redis instead of in-memory caching for the session       │
│  store. This will allow horizontal scaling and persistence.             │
│  ────────────────────────────────────────────────────────────────────── │
│  Votes:                                                                 │
│    ✓ coder-1:    Approve - "Makes sense for our scale"                 │
│    ✓ architect-1: Approve - "Aligns with infra plans"                  │
│    ? reviewer-1:  Pending                                               │
└─────────────────────────────────────────────────────────────────────────┤
│ f:Force Approve  v:Veto  Enter:Details  Esc:Back                        │
└─────────────────────────────────────────────────────────────────────────┘
```

### Logs View

```
┌─ Dispatch > Logs ───────────────────────────────────────────────────────┐
│ [D]ashboard  [I]ssues  [A]gents  [E]pics  [P]roposals  [L]ogs  [?]Help │
├─────────────────────────────────────────────────────────────────────────┤
│  Filter: [All] [Error] [Warn] [Info] [Debug]    Agent: [All ▼]         │
├─────────────────────────────────────────────────────────────────────────┤
│  14:35:22 INFO  coder-1   Writing implementation for OAuth flow        │
│  14:35:20 DEBUG test-1    Running test suite...                        │
│  14:35:18 INFO  coder-2   Completed rate limiter implementation        │
│  14:35:15 WARN  sec-1     Potential SQL injection in query builder     │
│  14:35:12 INFO  system    PR #23 checks passed                         │
│  14:35:10 DEBUG coder-1   Reading oauth2 library documentation         │
│  14:35:08 INFO  reviewer-1 Code review completed for PR #22            │
│  14:35:05 ERROR coder-1   Test failure: auth_flow_test::test_invalid   │
│  14:35:02 INFO  system    Agent coder-1 heartbeat OK                   │
│  14:35:00 DEBUG arch-1    Analyzing dependency graph                   │
│  14:34:58 INFO  test-1    12 tests passed, 0 failed                    │
│  14:34:55 WARN  system    GitHub rate limit: 4500/5000 remaining       │
│                                                                         │
│                                                                         │
│                                                                         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┤
│ f:Filter  a:Agent Filter  c:Clear  /: Search  PgUp/PgDn:Scroll         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Key Bindings

### Global

| Key | Action |
|-----|--------|
| `q` | Quit |
| `?` | Help |
| `/` | Search |
| `r` | Refresh |
| `Esc` | Back/Cancel |
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `D` | Dashboard view |
| `I` | Issues view |
| `A` | Agents view |
| `E` | Epics view |
| `P` | Proposals view |
| `L` | Logs view |
| `:` | Command mode |

### List Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to top |
| `G` | Go to bottom |
| `Enter` | Select/Details |
| `Space` | Toggle selection |

### Issues View

| Key | Action |
|-----|--------|
| `a` | Assign to agent |
| `u` | Unassign |
| `p` | Change priority |
| `s` | Change status |
| `c` | Cancel issue |
| `n` | New issue |

### Agents View

| Key | Action |
|-----|--------|
| `s` | Start agent |
| `k` | Kill/Stop agent |
| `p` | Pause agent |
| `r` | Resume agent |
| `l` | View logs |

### Epics View

| Key | Action |
|-----|--------|
| `a` | Approve gate |
| `r` | Reject gate |
| `s` | Skip gate |
| `c` | Add comment |

### Proposals View

| Key | Action |
|-----|--------|
| `f` | Force approve |
| `v` | Veto |

---

## Key Data Structures

```rust
// dispatch-tui/src/app.rs

pub struct App {
    pub state: AppState,
    pub router: Router,
    pub should_quit: bool,
    pub tick_rate: Duration,
}

pub struct AppState {
    // Data caches (refreshed periodically)
    pub issues: Vec<Issue>,
    pub agents: Vec<Agent>,
    pub epics: Vec<Epic>,
    pub proposals: Vec<Proposal>,
    pub logs: VecDeque<LogEntry>,

    // Selection state per view
    pub issue_list_state: ListState,
    pub agent_list_state: ListState,
    pub epic_list_state: ListState,
    pub proposal_list_state: ListState,
    pub log_list_state: ListState,

    // Filters
    pub issue_filter: IssueFilter,
    pub log_filter: LogFilter,

    // Stats
    pub stats: SystemStats,
    pub last_refresh: Instant,
}

pub struct Router {
    pub current_view: View,
    pub view_stack: Vec<View>,
    pub modal: Option<Modal>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Issues,
    IssueDetail(IssueId),
    Agents,
    AgentDetail(AgentId),
    Epics,
    EpicDetail(EpicId),
    Proposals,
    ProposalDetail(ProposalId),
    Logs,
    Help,
}

pub enum Modal {
    Confirm { title: String, message: String, action: ConfirmAction },
    Input { title: String, prompt: String, value: String, action: InputAction },
    Select { title: String, options: Vec<String>, selected: usize, action: SelectAction },
}
```

---

## Event Handling

```rust
// dispatch-tui/src/events.rs

pub enum AppEvent {
    // User input
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),

    // System events
    Tick,
    Refresh,

    // Data updates (from database/events)
    IssueUpdated(IssueId),
    AgentUpdated(AgentId),
    EpicUpdated(EpicId),
    ProposalUpdated(ProposalId),
    NewLog(LogEntry),
    GateReached(EpicId, GateId),
    ProposalNeedsVote(ProposalId),
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    tick_rate: Duration,
}

impl EventHandler {
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
```

---

## Rendering

```rust
// dispatch-tui/src/ui/mod.rs

pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Tab bar
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    render_tab_bar(app, frame, chunks[0]);

    match app.router.current_view {
        View::Dashboard => dashboard::render(app, frame, chunks[1]),
        View::Issues => issues::render(app, frame, chunks[1]),
        View::IssueDetail(id) => issues::render_detail(app, frame, chunks[1], &id),
        View::Agents => agents::render(app, frame, chunks[1]),
        View::AgentDetail(id) => agents::render_detail(app, frame, chunks[1], &id),
        View::Epics => epics::render(app, frame, chunks[1]),
        View::EpicDetail(id) => epics::render_detail(app, frame, chunks[1], &id),
        View::Proposals => proposals::render(app, frame, chunks[1]),
        View::ProposalDetail(id) => proposals::render_detail(app, frame, chunks[1], &id),
        View::Logs => logs::render(app, frame, chunks[1]),
        View::Help => help::render(app, frame, chunks[1]),
    }

    render_status_bar(app, frame, chunks[2]);

    // Render modal on top if present
    if let Some(ref modal) = app.router.modal {
        render_modal(modal, frame);
    }
}
```

---

## Color Scheme

```rust
// dispatch-tui/src/theme.rs

pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,

    // Status colors
    pub status_working: Color,
    pub status_idle: Color,
    pub status_paused: Color,
    pub status_error: Color,

    // Priority colors
    pub priority_critical: Color,
    pub priority_high: Color,
    pub priority_medium: Color,
    pub priority_low: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::White,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,

            status_working: Color::Green,
            status_idle: Color::Gray,
            status_paused: Color::Yellow,
            status_error: Color::Red,

            priority_critical: Color::Red,
            priority_high: Color::LightRed,
            priority_medium: Color::Yellow,
            priority_low: Color::Gray,
        }
    }
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-060 | TUI framework setup | `dispatch-tui/src/app.rs`, `dispatch-tui/src/events.rs` |
| PR-061 | Dashboard view | `dispatch-tui/src/ui/dashboard.rs` |
| PR-062 | Issues view | `dispatch-tui/src/ui/issues.rs` |
| PR-063 | Agents view | `dispatch-tui/src/ui/agents.rs` |
| PR-064 | Epics view | `dispatch-tui/src/ui/epics.rs` |
| PR-065 | Proposals view | `dispatch-tui/src/ui/proposals.rs` |
| PR-066 | Logs view | `dispatch-tui/src/ui/logs.rs` |
| PR-067 | Command mode | `dispatch-tui/src/command.rs` |
| PR-068 | CLI integration | `dispatch-cli/src/commands/tui.rs` |
