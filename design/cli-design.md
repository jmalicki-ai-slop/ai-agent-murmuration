# CLI Design

## Overview

This document defines the complete CLI interface for the Dispatch system using clap.

---

## Binary Name

```
dispatch
```

---

## Global Options

```
dispatch [OPTIONS] <COMMAND>

Options:
  -c, --config <FILE>     Path to config file [default: ~/.config/dispatch/config.toml]
  -d, --database <FILE>   Path to database file [default: ~/.local/share/dispatch/dispatch.db]
  -v, --verbose           Increase verbosity (-v, -vv, -vvv)
  -q, --quiet             Suppress non-essential output
  --json                  Output in JSON format
  -h, --help              Print help
  -V, --version           Print version
```

---

## Command Groups

```
dispatch <COMMAND>

Commands:
  issue       Manage issues
  epic        Manage epics and stages
  agent       Manage agents
  proposal    Manage proposals and voting
  worktree    Manage git worktrees
  sync        GitHub synchronization
  config      Configuration management
  tui         Launch terminal UI
  serve       Start web server
  init        Initialize dispatch in a repository
  status      Show system status overview
  help        Print help for a command
```

---

## Issue Commands

### dispatch issue

```
dispatch issue <COMMAND>

Commands:
  list        List issues
  show        Show issue details
  create      Create a new issue
  edit        Edit an issue
  assign      Assign issue to an agent
  unassign    Unassign issue from agent
  status      Change issue status
  delete      Delete an issue
  link-pr     Link a pull request to an issue
```

### dispatch issue list

```
dispatch issue list [OPTIONS]

Options:
  -s, --status <STATUS>       Filter by status [possible values: unassigned, queued, assigned,
                              in_progress, awaiting_review, in_review, done, blocked, cancelled]
  -p, --priority <PRIORITY>   Filter by priority [possible values: critical, high, medium, low]
  -t, --type <TYPE>           Filter by type [possible values: feature, bug, docs, refactor,
                              test, security, chore]
  -e, --epic <EPIC_ID>        Filter by epic
  -a, --agent <AGENT_ID>      Filter by assigned agent
  --repo <PATH>               Filter by repository path
  -n, --limit <N>             Limit results [default: 50]
  --all                       Show all issues (no limit)
  --sort <FIELD>              Sort by field [default: priority] [possible values: priority,
                              created, updated, status]
  --desc                      Sort descending

Examples:
  dispatch issue list --status in_progress
  dispatch issue list --priority critical --priority high
  dispatch issue list --epic abc123 --sort created
```

### dispatch issue show

```
dispatch issue show <ISSUE_ID>

Arguments:
  <ISSUE_ID>    Issue ID or GitHub issue number (prefixed with #)

Options:
  --full        Show full prompt/description
  --logs        Include agent logs

Examples:
  dispatch issue show abc123-def456
  dispatch issue show #42
```

### dispatch issue create

```
dispatch issue create [OPTIONS] <TITLE>

Arguments:
  <TITLE>       Issue title

Options:
  -p, --prompt <TEXT>         Issue prompt/description (or read from stdin with -)
  -f, --prompt-file <FILE>    Read prompt from file
  -t, --type <TYPE>           Issue type [default: feature]
  --priority <PRIORITY>       Priority [default: medium]
  -l, --label <LABEL>         Add label (can be repeated)
  -e, --epic <EPIC_ID>        Associate with epic
  -s, --stage <STAGE_ID>      Associate with stage
  --repo <PATH>               Repository path [default: current directory]
  --no-github                 Don't create GitHub issue
  --assign <AGENT_TYPE>       Immediately assign to agent type

Examples:
  dispatch issue create "Add user authentication" -p "Implement OAuth2 login"
  dispatch issue create "Fix memory leak" --type bug --priority critical
  echo "Long description..." | dispatch issue create "Feature X" -p -
  dispatch issue create "Write tests" --assign test
```

### dispatch issue edit

```
dispatch issue edit <ISSUE_ID> [OPTIONS]

Arguments:
  <ISSUE_ID>    Issue ID

Options:
  --title <TITLE>             Update title
  --prompt <TEXT>             Update prompt
  --type <TYPE>               Update type
  --priority <PRIORITY>       Update priority
  --add-label <LABEL>         Add label
  --remove-label <LABEL>      Remove label
  --epic <EPIC_ID>            Change epic (use 'none' to remove)
  --stage <STAGE_ID>          Change stage (use 'none' to remove)

Examples:
  dispatch issue edit abc123 --priority high
  dispatch issue edit abc123 --add-label security --add-label urgent
```

### dispatch issue assign

```
dispatch issue assign <ISSUE_ID> [OPTIONS]

Arguments:
  <ISSUE_ID>    Issue ID

Options:
  -t, --type <AGENT_TYPE>     Agent type [possible values: coder, reviewer, pm, security,
                              docs, test, architect]
  -a, --agent <AGENT_ID>      Specific agent ID (creates new if not specified)
  --start                     Start agent immediately after assignment

Examples:
  dispatch issue assign abc123 --type coder --start
  dispatch issue assign abc123 --agent agent-xyz
```

### dispatch issue unassign

```
dispatch issue unassign <ISSUE_ID> [OPTIONS]

Arguments:
  <ISSUE_ID>    Issue ID

Options:
  --stop        Stop the agent if running
  --keep-worktree   Don't clean up worktree

Examples:
  dispatch issue unassign abc123 --stop
```

### dispatch issue status

```
dispatch issue status <ISSUE_ID> <STATUS>

Arguments:
  <ISSUE_ID>    Issue ID
  <STATUS>      New status [possible values: unassigned, queued, assigned, in_progress,
                awaiting_review, in_review, done, blocked, cancelled]

Options:
  --reason <TEXT>     Reason for status change (for blocked/cancelled)

Examples:
  dispatch issue status abc123 in_progress
  dispatch issue status abc123 blocked --reason "Waiting for API access"
```

### dispatch issue delete

```
dispatch issue delete <ISSUE_ID> [OPTIONS]

Arguments:
  <ISSUE_ID>    Issue ID

Options:
  --force           Delete without confirmation
  --keep-github     Don't close GitHub issue
  --keep-worktree   Don't delete worktree

Examples:
  dispatch issue delete abc123 --force
```

### dispatch issue link-pr

```
dispatch issue link-pr <ISSUE_ID> <PR_NUMBER>

Arguments:
  <ISSUE_ID>     Issue ID
  <PR_NUMBER>    GitHub PR number

Examples:
  dispatch issue link-pr abc123 42
```

---

## Epic Commands

### dispatch epic

```
dispatch epic <COMMAND>

Commands:
  list        List epics
  show        Show epic details with stages
  create      Create a new epic
  edit        Edit an epic
  decompose   Decompose epic into issues (PM agent)
  advance     Advance to next stage
  gate        Manage gates
  delete      Delete an epic
```

### dispatch epic list

```
dispatch epic list [OPTIONS]

Options:
  -s, --status <STATUS>       Filter by status [possible values: draft, ready, in_progress,
                              awaiting_gate, completed, cancelled]
  --repo <PATH>               Filter by repository
  -n, --limit <N>             Limit results [default: 20]

Examples:
  dispatch epic list --status in_progress
```

### dispatch epic show

```
dispatch epic show <EPIC_ID>

Arguments:
  <EPIC_ID>     Epic ID or GitHub issue number (prefixed with #)

Options:
  --issues      Include child issues
  --stages      Show detailed stage information
  --full        Show full description and criteria

Examples:
  dispatch epic show abc123 --stages --issues
```

### dispatch epic create

```
dispatch epic create [OPTIONS] <TITLE>

Arguments:
  <TITLE>       Epic title

Options:
  -d, --description <TEXT>    Epic description
  -f, --description-file <FILE>   Read description from file
  --criteria <TEXT>           Acceptance criterion (can be repeated)
  --repo <PATH>               Repository path [default: current directory]
  --no-github                 Don't create GitHub issue
  --stages <JSON>             Define stages as JSON

Examples:
  dispatch epic create "User Authentication System" -d "Complete auth implementation"
  dispatch epic create "API Redesign" --criteria "All endpoints documented" --criteria "100% test coverage"
```

### dispatch epic decompose

```
dispatch epic decompose <EPIC_ID> [OPTIONS]

Arguments:
  <EPIC_ID>     Epic ID

Options:
  --pm          Use PM agent to decompose
  --stages <N>  Number of stages [default: 4]
  --approve     Auto-approve generated breakdown

Examples:
  dispatch epic decompose abc123 --pm --stages 5
```

### dispatch epic advance

```
dispatch epic advance <EPIC_ID> [OPTIONS]

Arguments:
  <EPIC_ID>     Epic ID

Options:
  --force       Skip gate approval check

Examples:
  dispatch epic advance abc123
```

### dispatch epic gate

```
dispatch epic gate <COMMAND>

Commands:
  list        List gates for an epic
  show        Show gate details
  approve     Approve a gate
  reject      Reject a gate
  skip        Skip a gate (human override)
  comment     Add comment to a gate

dispatch epic gate approve <GATE_ID>
dispatch epic gate reject <GATE_ID> --reason "Missing tests"
dispatch epic gate skip <GATE_ID> --reason "Emergency deployment"
dispatch epic gate comment <GATE_ID> "Looks good, minor suggestions in PR"
```

---

## Agent Commands

### dispatch agent

```
dispatch agent <COMMAND>

Commands:
  list        List agents
  show        Show agent details
  start       Start an agent
  stop        Stop an agent
  pause       Pause an agent
  resume      Resume a paused agent
  logs        View agent logs
  status      Show agent status overview
```

### dispatch agent list

```
dispatch agent list [OPTIONS]

Options:
  -s, --status <STATUS>       Filter by status [possible values: idle, starting, working,
                              waiting_for_input, waiting_for_vote, paused, errored, completed]
  -t, --type <TYPE>           Filter by type [possible values: coder, reviewer, pm, security,
                              docs, test, architect]
  --active                    Show only active agents
  --all                       Show all agents including completed

Examples:
  dispatch agent list --active
  dispatch agent list --type coder --status working
```

### dispatch agent show

```
dispatch agent show <AGENT_ID>

Arguments:
  <AGENT_ID>    Agent ID

Options:
  --metrics     Show performance metrics
  --history     Show issue history

Examples:
  dispatch agent show abc123 --metrics
```

### dispatch agent start

```
dispatch agent start [OPTIONS]

Options:
  -t, --type <TYPE>           Agent type [required]
  -i, --issue <ISSUE_ID>      Issue to work on
  --detach                    Run in background

Examples:
  dispatch agent start --type coder --issue abc123
  dispatch agent start --type reviewer --detach
```

### dispatch agent stop

```
dispatch agent stop <AGENT_ID> [OPTIONS]

Arguments:
  <AGENT_ID>    Agent ID

Options:
  --force       Force kill without graceful shutdown
  --keep-work   Don't reset issue status

Examples:
  dispatch agent stop abc123
  dispatch agent stop abc123 --force
```

### dispatch agent pause

```
dispatch agent pause <AGENT_ID>

Arguments:
  <AGENT_ID>    Agent ID

Examples:
  dispatch agent pause abc123
```

### dispatch agent resume

```
dispatch agent resume <AGENT_ID>

Arguments:
  <AGENT_ID>    Agent ID

Examples:
  dispatch agent resume abc123
```

### dispatch agent logs

```
dispatch agent logs <AGENT_ID> [OPTIONS]

Arguments:
  <AGENT_ID>    Agent ID

Options:
  -n, --lines <N>             Number of lines [default: 100]
  -f, --follow                Follow log output
  --level <LEVEL>             Filter by level [possible values: debug, info, warn, error]
  --since <DURATION>          Show logs since duration (e.g., "1h", "30m")

Examples:
  dispatch agent logs abc123 -f
  dispatch agent logs abc123 --level error --since 1h
```

### dispatch agent status

```
dispatch agent status [OPTIONS]

Options:
  --watch       Continuously update display

Examples:
  dispatch agent status
  dispatch agent status --watch
```

---

## Proposal Commands

### dispatch proposal

```
dispatch proposal <COMMAND>

Commands:
  list        List proposals
  show        Show proposal details
  create      Create a proposal (for human-initiated)
  vote        Cast a vote (simulation/testing)
  force       Force approve/reject (human override)
  veto        Veto an approved proposal
```

### dispatch proposal list

```
dispatch proposal list [OPTIONS]

Options:
  -s, --status <STATUS>       Filter by status [possible values: open, voting, approved,
                              rejected, executing, executed, rolled_back, vetoed]
  -t, --type <TYPE>           Filter by type [possible values: implementation_approach,
                              tech_stack_choice, architecture_decision, new_agent_type,
                              workflow_change, governance_rule, tool_integration,
                              prompt_improvement]
  --pending                   Show pending votes only
  -n, --limit <N>             Limit results [default: 20]

Examples:
  dispatch proposal list --pending
  dispatch proposal list --status approved --type architecture_decision
```

### dispatch proposal show

```
dispatch proposal show <PROPOSAL_ID>

Arguments:
  <PROPOSAL_ID>    Proposal ID

Options:
  --votes         Show all votes with reasoning
  --full          Show full description and rationale

Examples:
  dispatch proposal show abc123 --votes
```

### dispatch proposal force

```
dispatch proposal force <PROPOSAL_ID> <DECISION>

Arguments:
  <PROPOSAL_ID>    Proposal ID
  <DECISION>       Decision [possible values: approve, reject]

Options:
  --reason <TEXT>   Reason for forcing (required)

Examples:
  dispatch proposal force abc123 approve --reason "Urgent business need"
```

### dispatch proposal veto

```
dispatch proposal veto <PROPOSAL_ID>

Arguments:
  <PROPOSAL_ID>    Proposal ID

Options:
  --reason <TEXT>   Reason for veto (required)

Examples:
  dispatch proposal veto abc123 --reason "Security concerns not addressed"
```

---

## Worktree Commands

### dispatch worktree

```
dispatch worktree <COMMAND>

Commands:
  list        List worktrees
  create      Create a worktree for an issue
  delete      Delete a worktree
  clean       Clean up orphaned worktrees
```

### dispatch worktree list

```
dispatch worktree list [OPTIONS]

Options:
  --orphaned      Show only orphaned worktrees
  --repo <PATH>   Filter by repository

Examples:
  dispatch worktree list
  dispatch worktree list --orphaned
```

### dispatch worktree create

```
dispatch worktree create <ISSUE_ID> [OPTIONS]

Arguments:
  <ISSUE_ID>    Issue ID

Options:
  --base <BRANCH>     Base branch [default: main]
  --name <NAME>       Custom worktree name

Examples:
  dispatch worktree create abc123 --base develop
```

### dispatch worktree delete

```
dispatch worktree delete <ISSUE_ID> [OPTIONS]

Arguments:
  <ISSUE_ID>    Issue ID

Options:
  --force           Delete even if uncommitted changes
  --keep-branch     Don't delete the branch

Examples:
  dispatch worktree delete abc123
```

### dispatch worktree clean

```
dispatch worktree clean [OPTIONS]

Options:
  --dry-run     Show what would be deleted
  --force       Delete without confirmation

Examples:
  dispatch worktree clean --dry-run
```

---

## Sync Commands

### dispatch sync

```
dispatch sync <COMMAND>

Commands:
  pull        Pull changes from GitHub
  push        Push changes to GitHub
  status      Show sync status
  full        Full bidirectional sync
```

### dispatch sync pull

```
dispatch sync pull [OPTIONS]

Options:
  --issues      Sync issues only
  --prs         Sync PRs only
  --since <TIME>    Only sync changes since time

Examples:
  dispatch sync pull --issues
```

### dispatch sync push

```
dispatch sync push [OPTIONS]

Options:
  --issues      Push issues only
  --prs         Push PRs only
  --dry-run     Show what would be pushed

Examples:
  dispatch sync push --dry-run
```

### dispatch sync full

```
dispatch sync full [OPTIONS]

Options:
  --force       Force sync even if conflicts

Examples:
  dispatch sync full
```

---

## Config Commands

### dispatch config

```
dispatch config <COMMAND>

Commands:
  show        Show current configuration
  set         Set a configuration value
  get         Get a configuration value
  edit        Open config in editor
  init        Initialize default configuration
```

### dispatch config show

```
dispatch config show [OPTIONS]

Options:
  --defaults    Show default values
  --path        Show config file path only

Examples:
  dispatch config show
```

### dispatch config set

```
dispatch config set <KEY> <VALUE>

Arguments:
  <KEY>       Configuration key (dot notation)
  <VALUE>     Value to set

Examples:
  dispatch config set github.owner "myorg"
  dispatch config set agents.max_concurrent 5
  dispatch config set database.path "/custom/path/dispatch.db"
```

### dispatch config get

```
dispatch config get <KEY>

Arguments:
  <KEY>       Configuration key (dot notation)

Examples:
  dispatch config get github.owner
```

---

## TUI Command

### dispatch tui

```
dispatch tui [OPTIONS]

Options:
  --view <VIEW>     Initial view [default: dashboard] [possible values: dashboard, issues,
                    agents, epics, proposals, logs]

Examples:
  dispatch tui
  dispatch tui --view agents
```

---

## Serve Command

### dispatch serve

```
dispatch serve [OPTIONS]

Options:
  -p, --port <PORT>           Port to listen on [default: 8080]
  -H, --host <HOST>           Host to bind to [default: 127.0.0.1]
  --webhook-secret <SECRET>   GitHub webhook secret
  --cors-origin <ORIGIN>      Allowed CORS origin (can be repeated)
  --no-web                    Disable web UI, API only

Examples:
  dispatch serve --port 3000
  dispatch serve --host 0.0.0.0 --webhook-secret $WEBHOOK_SECRET
```

---

## Init Command

### dispatch init

```
dispatch init [OPTIONS] [PATH]

Arguments:
  [PATH]      Repository path [default: current directory]

Options:
  --github <REPO>     GitHub repository (owner/repo)
  --no-github         Don't configure GitHub integration
  --stages <JSON>     Default epic stages template

Examples:
  dispatch init
  dispatch init --github myorg/myrepo
  dispatch init /path/to/repo --no-github
```

---

## Status Command

### dispatch status

```
dispatch status [OPTIONS]

Options:
  --watch       Continuously update

Output:
  System Status
  ├── Active Agents: 3
  ├── Issues In Progress: 5
  ├── Pending Gates: 2
  ├── Open Proposals: 1
  └── Last Sync: 5 minutes ago

Examples:
  dispatch status
  dispatch status --watch
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Database error |
| 5 | Git error |
| 6 | GitHub API error |
| 7 | Agent error |
| 8 | Not found |
| 9 | Invalid state transition |
| 10 | Permission denied |

---

## Shell Completions

```
dispatch completions <SHELL>

Arguments:
  <SHELL>     Shell type [possible values: bash, zsh, fish, powershell, elvish]

Examples:
  dispatch completions bash > /etc/bash_completion.d/dispatch
  dispatch completions zsh > ~/.zfunc/_dispatch
```

---

## Implementation Structure

```rust
// dispatch-cli/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dispatch")]
#[command(about = "Issue-based agent orchestration system")]
#[command(version)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    database: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    quiet: bool,

    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Issue(IssueCommand),
    Epic(EpicCommand),
    Agent(AgentCommand),
    Proposal(ProposalCommand),
    Worktree(WorktreeCommand),
    Sync(SyncCommand),
    Config(ConfigCommand),
    Tui(TuiCommand),
    Serve(ServeCommand),
    Init(InitCommand),
    Status(StatusCommand),
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-006 | CLI skeleton + global args | `dispatch-cli/src/main.rs`, `dispatch-cli/src/output.rs` |
| PR-019 | Issue commands | `dispatch-cli/src/commands/issue.rs` |
| PR-026 | Epic commands | `dispatch-cli/src/commands/epic.rs` |
| PR-035 | Agent commands | `dispatch-cli/src/commands/agent.rs` |
| PR-044 | Sync commands | `dispatch-cli/src/commands/sync.rs` |
| PR-053 | Proposal commands | `dispatch-cli/src/commands/proposal.rs` |
| PR-068 | TUI command | `dispatch-cli/src/commands/tui.rs` |
