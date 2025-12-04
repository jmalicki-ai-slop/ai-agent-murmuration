# Murmuration

Multi-agent orchestration for software development using Claude Code.

Murmuration coordinates multiple AI agents working collaboratively on software development tasks, using GitHub issues as the primary interface for task management and dependency tracking.

## Features

- **Agent Spawning**: Run Claude Code agents on tasks with streaming output
- **Git Worktrees**: Isolated workspaces branched from latest main, with caching
- **GitHub Integration**: Fetch issues, parse dependencies, check PR status
- **Dependency Resolution**: Block work until prerequisite PRs are merged
- **Plan Import**: Create GitHub issues from PLAN.md with automatic dependency linking

## Installation

```bash
# Clone the repository
git clone https://github.com/jmalicki-ai-slop/ai-agent-murmuration.git
cd ai-agent-murmuration

# Build
cargo build --release

# Add to PATH (optional)
cp target/release/murmur ~/.local/bin/
```

## Requirements

- Rust 1.70+
- [Claude Code CLI](https://claude.ai/code) installed and authenticated
- GitHub Personal Access Token (for GitHub features) - see [docs/github-token.md](docs/github-token.md)

## Quick Start

```bash
# Run a task in the current directory
murmur run "fix the typo in README"

# Create an isolated worktree for development
murmur worktree create feature/my-feature

# List open issues from a GitHub repo
export GITHUB_TOKEN=your_token
murmur issue list --repo owner/repo

# Work on a specific issue (creates worktree, checks dependencies)
murmur work 42 --repo owner/repo
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `murmur run <prompt>` | Run an agent on a task |
| `murmur worktree create <branch>` | Create isolated worktree |
| `murmur worktree list` | List active worktrees |
| `murmur worktree clean` | Remove old worktrees |
| `murmur issue list` | List GitHub issues |
| `murmur issue show <number>` | Show issue details |
| `murmur issue deps <number>` | Show issue dependencies |
| `murmur work <number>` | Work on an issue |
| `murmur config` | Show current configuration |

## Configuration

Create `~/.config/murmur/config.toml`:

```toml
[agent]
claude_path = "claude"  # Path to claude executable
model = "sonnet"        # Model to use (optional)
```

Or use environment variables:

```bash
export MURMUR_CLAUDE_PATH=/path/to/claude
export MURMUR_MODEL=sonnet
```

## Project Structure

```
murmuration/
├── murmur-core/      # Core library (agent, git, config, plan parsing)
├── murmur-cli/       # CLI binary
├── murmur-github/    # GitHub API integration
├── design/           # Design documents
└── PLAN.md           # Bootstrap roadmap
```

## Development Status

Currently implementing the bootstrap phases to make Murmuration self-hosting:

- [x] Phase 1: Minimal CLI + Agent Spawning
- [x] Phase 2: Git Worktrees
- [x] Phase 3: GitHub Integration (Read)
- [x] Phase 3b: Plan Import
- [ ] Phase 4: Agent Types + Prompts
- [ ] Phase 5: TDD Workflow
- [ ] Phase 6: Review Workflow
- [ ] Phase 7: Coordinator Agent

See [PLAN.md](PLAN.md) for the complete roadmap.

## License

AGPL-3.0-or-later - See [LICENSE](LICENSE) for details.
