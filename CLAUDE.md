# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Murmuration is a multi-agent orchestration system that uses GitHub issues as the primary interface for coordinating multiple AI agents (Claude Code instances) working collaboratively on software development tasks. The goal is to bootstrap the system to build itself.

## Build and Test Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Run a single test
cargo test test_name

# Run tests for a specific crate
cargo test -p murmur-core
cargo test -p murmur-github
cargo test -p murmur-cli

# Run the CLI
./target/debug/murmur --help
cargo run -- --help
```

## Architecture

### Workspace Structure

Three crates in a Cargo workspace:

- **murmur-core**: Core library with agent spawning, git operations, config, and plan parsing
- **murmur-cli**: Binary crate with CLI commands (`run`, `worktree`, `issue`, `work`)
- **murmur-github**: GitHub API integration via octocrab for issues, PRs, and dependencies

### Key Modules in murmur-core

- `agent/spawn.rs`: Spawns Claude Code as subprocess with `--print` flag for JSON output
- `agent/output.rs`: Parses streaming JSON output from Claude (assistant messages, tool use, results, cost)
- `git/repo.rs`: Git repository detection and remote info
- `git/worktree.rs`: Worktree creation at `~/.cache/murmur/worktrees/`
- `git/pool.rs`: Worktree caching and metadata persistence
- `git/clone.rs`: Repository cloning to `~/.cache/murmur/repos/`
- `git/branch.rs`: Finding branching points from origin/main
- `config.rs`: Configuration from `~/.config/murmur/config.toml` with env overrides
- `plan/parser.rs`: Parses PLAN.md markdown tables into structured phases/PRs

### Key Modules in murmur-github

- `client.rs`: GitHub API client using octocrab (requires GITHUB_TOKEN env var)
- `issues.rs`: Fetch and filter issues
- `metadata.rs`: Parse `<!-- murmur:metadata {...} -->` blocks from issue bodies
- `dependencies.rs`: Parse "Depends on #X" links and build dependency graphs
- `pr.rs`: Check PR merge status for dependency resolution
- `create.rs`: Create GitHub issues from parsed PLAN.md

### CLI Commands

- `murmur run <prompt>`: Spawn Claude agent on a task in current or specified directory
- `murmur worktree create/list/clean`: Manage isolated git worktrees
- `murmur issue list/show/deps`: View GitHub issues and their dependencies
- `murmur work <issue-number>`: Work on an issue (creates worktree, checks deps, spawns agent)

## Environment Variables

- `GITHUB_TOKEN`: Required for GitHub API operations (PAT, not OAuth token)
- `MURMUR_CLAUDE_PATH`: Override path to claude executable
- `MURMUR_MODEL`: Override model selection

## Issue Dependency Syntax

Issues use this format in their body for dependency tracking:

```markdown
## Dependencies
Depends on #12
Blocked by #15
Parent: #8

<!-- murmur:metadata
{
  "phase": 3,
  "pr": "023",
  "status": "blocked"
}
-->
```

## Development Status

Phases 1-3b are complete (CLI, git worktrees, GitHub read integration, plan import). Phases 4-7 (agent types, TDD workflow, review workflow, coordinator) are in progress. See PLAN.md for the full bootstrap roadmap.
