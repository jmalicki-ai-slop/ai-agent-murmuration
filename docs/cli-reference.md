# CLI Command Reference

Comprehensive reference for all Murmur CLI commands.

## Table of Contents

- [Global Flags](#global-flags)
- [murmur run](#murmur-run)
- [murmur work](#murmur-work)
- [murmur agent start](#murmur-agent-start)
- [murmur tdd](#murmur-tdd)
- [murmur worktree](#murmur-worktree)
  - [worktree create](#worktree-create)
  - [worktree list](#worktree-list)
  - [worktree clean](#worktree-clean)
  - [worktree show](#worktree-show)
- [murmur issue](#murmur-issue)
  - [issue list](#issue-list)
  - [issue show](#issue-show)
  - [issue deps](#issue-deps)
- [murmur status](#murmur-status)
- [murmur config](#murmur-config)
- [murmur secrets-init](#murmur-secrets-init)

## Global Flags

These flags are available for all commands:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--verbose`, `-v` | boolean | false | Enable verbose output for debugging |
| `--model <MODEL>` | string | from config | Override the AI model to use (env: `MURMUR_MODEL`) |
| `--backend <BACKEND>` | string | from config | Backend to use: `claude` or `cursor` (env: `MURMUR_BACKEND`) |
| `--claude-path <PATH>` | path | from config | Override path to claude executable (env: `MURMUR_CLAUDE_PATH`) |
| `--no-emoji` | boolean | false | Disable emoji output, use ASCII alternatives instead |
| `--help`, `-h` | boolean | - | Print help information |
| `--version`, `-V` | boolean | - | Print version information |

---

## murmur run

Run a task with Murmuration agents.

### Syntax

```bash
murmur run [OPTIONS] <PROMPT>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<PROMPT>` | Yes | The task prompt describing what to accomplish |

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--workdir <WORKDIR>`, `-d` | path | `.` | Working directory for the task |
| `--agents <AGENTS>`, `-n` | number | `1` | Number of parallel agents to use |
| `--dry-run` | boolean | false | Show what would be executed without running |
| `--verbose`, `-v` | boolean | false | Enable verbose output |

### Examples

Run a simple task in the current directory:
```bash
murmur run "Add error handling to the parser module"
```

Run with multiple agents in a specific directory:
```bash
murmur run --workdir ~/projects/myapp --agents 3 "Refactor authentication logic"
```

Dry run to preview what would happen:
```bash
murmur run --dry-run "Update dependencies to latest versions"
```

### Related Commands

- [murmur work](#murmur-work) - Work on a GitHub issue with automatic worktree management
- [murmur agent start](#murmur-agent-start) - Start a typed agent with specialized behavior

---

## murmur work

Work on a GitHub issue with automatic worktree and dependency management.

### Syntax

```bash
murmur work [OPTIONS] <ISSUE>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<ISSUE>` | Yes | Issue number to work on |

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | current repo | Repository in `owner/repo` format |
| `--force`, `-f` | boolean | false | Skip dependency checking |
| `--prompt <PROMPT>`, `-p` | string | issue body | Custom prompt to send to the agent |
| `--no-agent` | boolean | false | Don't start the agent, just create the worktree |
| `--resume` | boolean | false | Resume from the last interrupted or failed run |
| `--verbose`, `-v` | boolean | false | Enable verbose output |

### Examples

Work on issue #42 in the current repository:
```bash
murmur work 42
```

Work on an issue in a specific repository:
```bash
murmur work --repo acme/myapp 123
```

Create worktree without starting agent:
```bash
murmur work --no-agent 42
```

Resume a previous interrupted run:
```bash
murmur work --resume 42
```

Override the issue description with a custom prompt:
```bash
murmur work --prompt "Fix the login bug focusing on OAuth flow" 42
```

### Related Commands

- [murmur run](#murmur-run) - Run ad-hoc tasks without GitHub integration
- [murmur issue show](#issue-show) - View issue details before working on it
- [murmur issue deps](#issue-deps) - Check issue dependencies
- [murmur worktree](#murmur-worktree) - Manage worktrees manually

---

## murmur agent start

Start a typed agent with specialized behavior.

### Syntax

```bash
murmur agent start [OPTIONS] --type <AGENT_TYPE> <PROMPT>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<PROMPT>` | Yes | The task prompt describing what to accomplish |

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--type <AGENT_TYPE>`, `-t` | string | - | Agent type: `implement`, `test`, `review`, or `coordinator` |
| `--workdir <WORKDIR>`, `-d` | path | `.` | Working directory for the task |
| `--dry-run` | boolean | false | Show what would be executed without running |
| `--verbose`, `-v` | boolean | false | Enable verbose output |

### Agent Types

- **implement**: Focused on writing implementation code
- **test**: Specialized in writing and running tests
- **review**: Reviews code for quality, bugs, and best practices
- **coordinator**: Orchestrates multiple agents and manages workflow

### Examples

Start an implementation agent:
```bash
murmur agent start --type implement "Add user profile caching"
```

Start a test agent to write tests:
```bash
murmur agent start --type test "Write integration tests for API endpoints"
```

Start a review agent:
```bash
murmur agent start --type review "Review the authentication module"
```

### Related Commands

- [murmur run](#murmur-run) - Run tasks without specifying agent type
- [murmur tdd](#murmur-tdd) - Automated TDD workflow with multiple agent types

---

## murmur tdd

Run a Test-Driven Development workflow with automated agent coordination.

### Syntax

```bash
murmur tdd [OPTIONS] <BEHAVIOR>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<BEHAVIOR>` | Yes | The behavior to implement using TDD |

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--workdir <WORKDIR>`, `-d` | path | `.` | Working directory |
| `--skip-spec` | boolean | false | Skip the WriteSpec phase (start from WriteTests) |
| `--skip-refactor` | boolean | false | Skip the Refactor phase (go straight to Complete) |
| `--max-iterations <N>` | number | `3` | Maximum iterations for Implement->VerifyGreen loop |
| `--dry-run` | boolean | false | Show what would be executed without running agents |
| `--verbose`, `-v` | boolean | false | Enable verbose output |

### TDD Workflow Phases

1. **WriteSpec**: Write specification for the behavior
2. **WriteTests**: Write failing tests based on spec
3. **Implement**: Implement the behavior to pass tests
4. **VerifyGreen**: Run tests and verify they pass
5. **Refactor**: Improve code quality while keeping tests green
6. **Complete**: Final verification and completion

### Examples

Run full TDD workflow:
```bash
murmur tdd "User authentication with JWT tokens"
```

Skip specification phase:
```bash
murmur tdd --skip-spec "Add rate limiting to API endpoints"
```

Limit implementation iterations:
```bash
murmur tdd --max-iterations 5 "Implement binary search tree"
```

Skip refactoring phase:
```bash
murmur tdd --skip-refactor "Add email validation"
```

### Related Commands

- [murmur agent start](#murmur-agent-start) - Start individual typed agents manually
- [murmur run](#murmur-run) - Run ad-hoc tasks without TDD workflow

---

## murmur worktree

Manage git worktrees for isolated development environments.

Worktrees are stored in `~/.cache/murmur/worktrees/` and allow working on multiple tasks simultaneously without branch switching.

### Subcommands

- [create](#worktree-create) - Create a worktree for a task
- [list](#worktree-list) - List all worktrees
- [clean](#worktree-clean) - Clean old worktrees
- [show](#worktree-show) - Show worktree details

---

### worktree create

Create a worktree for a task.

#### Syntax

```bash
murmur worktree create [OPTIONS] <TASK>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<TASK>` | Yes | Task identifier (e.g., issue number or slug) |

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | current repo | Repository URL or shorthand |
| `--base <BASE>`, `-b` | string | `main` | Base branch to create from |
| `--force`, `-f` | boolean | false | Force recreate if exists |
| `--verbose`, `-v` | boolean | false | Enable verbose output |

#### Examples

Create worktree for issue #42:
```bash
murmur worktree create 42
```

Create with custom base branch:
```bash
murmur worktree create --base develop 42
```

Force recreate existing worktree:
```bash
murmur worktree create --force 42
```

---

### worktree list

List all worktrees.

#### Syntax

```bash
murmur worktree list [OPTIONS]
```

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | all repos | Filter by repository name |
| `--verbose`, `-v` | boolean | false | Show detailed information |

#### Examples

List all worktrees:
```bash
murmur worktree list
```

List worktrees for specific repository:
```bash
murmur worktree list --repo myapp
```

---

### worktree clean

Clean old or stale worktrees.

#### Syntax

```bash
murmur worktree clean [OPTIONS]
```

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--all` | boolean | false | Clean all non-active worktrees |
| `--older-than <DAYS>` | number | - | Clean worktrees older than N days |
| `--repo <REPO>`, `-r` | string | all repos | Filter by repository name |
| `--stale-only` | boolean | false | Only clean orphaned worktrees (exist on disk but no running agent) |
| `--delete-branches` | boolean | false | Also delete associated git branches |
| `--verbose`, `-v` | boolean | false | Show detailed information |

#### Examples

Clean worktrees older than 7 days:
```bash
murmur worktree clean --older-than 7
```

Clean all inactive worktrees:
```bash
murmur worktree clean --all
```

Clean orphaned worktrees (exist on disk but have no running agent):
```bash
murmur worktree clean --stale-only
```

Clean and delete branches:
```bash
murmur worktree clean --all --delete-branches
```

---

### worktree show

Show detailed information about a specific worktree.

#### Syntax

```bash
murmur worktree show [OPTIONS] <TASK>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<TASK>` | Yes | Task identifier |

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | current repo | Repository name |
| `--verbose`, `-v` | boolean | false | Show detailed information |

#### Examples

Show worktree details:
```bash
murmur worktree show 42
```

---

## murmur issue

Manage GitHub issues and their dependencies.

Requires `GITHUB_TOKEN` environment variable with a GitHub Personal Access Token.

### Subcommands

- [list](#issue-list) - List issues from repository
- [show](#issue-show) - Show issue details
- [deps](#issue-deps) - Show issue dependency tree

---

### issue list

List issues from a repository.

#### Syntax

```bash
murmur issue list [OPTIONS]
```

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | current repo | Repository in `owner/repo` format |
| `--state <STATE>`, `-s` | string | `open` | Filter by state: `open`, `closed`, or `all` |
| `--label <LABEL>`, `-l` | string | - | Filter by label |
| `--verbose`, `-v` | boolean | false | Show detailed information |

#### Examples

List all open issues:
```bash
murmur issue list
```

List closed issues:
```bash
murmur issue list --state closed
```

Filter by label:
```bash
murmur issue list --label bug
```

List issues from specific repository:
```bash
murmur issue list --repo acme/myapp
```

---

### issue show

Show detailed information about a specific issue.

#### Syntax

```bash
murmur issue show [OPTIONS] <NUMBER>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<NUMBER>` | Yes | Issue number |

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | current repo | Repository in `owner/repo` format |
| `--verbose`, `-v` | boolean | false | Show detailed information |

#### Examples

Show issue #42:
```bash
murmur issue show 42
```

Show issue from specific repository:
```bash
murmur issue show --repo acme/myapp 123
```

---

### issue deps

Show issue dependency tree.

#### Syntax

```bash
murmur issue deps [OPTIONS] [NUMBER]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `[NUMBER]` | No | Issue number (or 'all' for complete graph) |

#### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--repo <REPO>`, `-r` | string | current repo | Repository in `owner/repo` format |
| `--verbose`, `-v` | boolean | false | Show detailed information |

#### Examples

Show dependencies for issue #42:
```bash
murmur issue deps 42
```

Show complete dependency graph:
```bash
murmur issue deps all
```

Show dependencies from specific repository:
```bash
murmur issue deps --repo acme/myapp 42
```

---

## murmur status

Show status of running agents and worktrees.

### Syntax

```bash
murmur status [OPTIONS]
```

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--verbose`, `-v` | boolean | false | Show verbose output including completed runs |

### Examples

Show current status:
```bash
murmur status
```

Show detailed status including history:
```bash
murmur status --verbose
```

### Related Commands

- [murmur worktree list](#worktree-list) - List all worktrees
- [murmur work](#murmur-work) - Start working on an issue

---

## murmur config

Show current Murmur configuration.

### Syntax

```bash
murmur config [OPTIONS]
```

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--verbose`, `-v` | boolean | false | Show detailed configuration |

### Examples

Show configuration:
```bash
murmur config
```

Show detailed configuration:
```bash
murmur config --verbose
```

### Configuration File

Configuration is loaded from `~/.config/murmur/config.toml`. See [Configuration Guide](configuration.md) for details.

### Related Commands

- [murmur secrets-init](#murmur-secrets-init) - Initialize secrets file

---

## murmur secrets-init

Initialize the secrets file at `~/.config/murmur/secrets.toml`.

### Syntax

```bash
murmur secrets-init [OPTIONS]
```

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--verbose`, `-v` | boolean | false | Show detailed output |

### Examples

Initialize secrets file:
```bash
murmur secrets-init
```

### Secrets File

The secrets file stores sensitive information like API tokens. It should never be committed to version control.

### Related Commands

- [murmur config](#murmur-config) - Show current configuration

---

## Environment Variables

Murmur respects these environment variables:

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | GitHub Personal Access Token for API access (required for GitHub features) |
| `MURMUR_CLAUDE_PATH` | Path to claude executable (can be overridden by `--claude-path`) |
| `MURMUR_MODEL` | AI model to use (can be overridden by `--model`) |
| `MURMUR_BACKEND` | Backend to use: `claude` or `cursor` (can be overridden by `--backend`) |

---

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 130 | Interrupted by user (Ctrl+C) |

---

## See Also

- [Getting Started Guide](getting-started.md)
- [Configuration Guide](configuration.md)
- [Architecture Documentation](architecture.md)
- [Troubleshooting Guide](troubleshooting.md)
