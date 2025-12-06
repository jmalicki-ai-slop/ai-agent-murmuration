# Getting Started with Murmuration

This guide will help you get up and running with Murmuration, a multi-agent orchestration system for software development using Claude Code.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Initial Configuration](#initial-configuration)
- [First GitHub Workflow](#first-github-workflow)
- [Next Steps](#next-steps)

## Prerequisites

Before installing Murmuration, ensure you have the following:

### Required

1. **Rust Toolchain** (version 1.70 or later)
   ```bash
   # Install Rust using rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Claude CLI** - Installed and authenticated
   ```bash
   # Install Claude CLI (visit https://claude.ai/code for latest instructions)
   # macOS/Linux:
   curl -sL https://claude.ai/install.sh | sh

   # Verify installation
   claude --version

   # Authenticate (required for murmur to spawn agents)
   claude auth login
   ```

3. **Git** (version 2.20 or later)
   ```bash
   # Verify installation
   git --version
   ```

### Optional (for GitHub features)

4. **GitHub Account** - Required for issue-based workflows

5. **GitHub Personal Access Token** - Required for GitHub API operations
   - See [docs/github-token.md](github-token.md) for detailed setup instructions
   - You'll need this to use `murmur issue`, `murmur work`, and auto-PR features

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/jmalicki-ai-slop/ai-agent-murmuration.git
cd ai-agent-murmuration
```

### 2. Build the Project

```bash
# Build in release mode for optimal performance
cargo build --release

# This will take a few minutes on first build
# The binary will be created at: target/release/murmur
```

### 3. Add to PATH

Choose one of these methods:

**Option A: Copy to local bin directory**
```bash
mkdir -p ~/.local/bin
cp target/release/murmur ~/.local/bin/

# Add to PATH in ~/.bashrc or ~/.zshrc if not already there
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Option B: Create a symlink**
```bash
mkdir -p ~/.local/bin
ln -s "$(pwd)/target/release/murmur" ~/.local/bin/murmur
```

**Option C: Use cargo install (from local directory)**
```bash
cargo install --path murmur-cli
```

### 4. Verify Installation

```bash
murmur --version
```

Expected output:
```
murmur 0.1.0
```

## Quick Start

Try Murmuration with a simple task to verify everything works.

### Run Your First Agent

```bash
# Navigate to any directory with code
cd ~/my-project

# Run a simple task
murmur run "explain the structure of this codebase"
```

**What happens:**
1. Murmur spawns a Claude Code agent as a subprocess
2. The agent analyzes your codebase
3. Output streams to your terminal in real-time
4. The agent provides a structured explanation

Example output:
```
$ murmur run "explain the structure of this codebase"

[Agent spawned with PID 12345]
[Model: claude-sonnet-4-5-20250929]

I'll analyze the structure of this codebase...

[The agent examines files and provides analysis]

Project Structure:
- src/
  - main.rs: Entry point
  - lib.rs: Core library
...

[Agent completed successfully]
Cost: $0.05 | Input: 1.2k tokens | Output: 850 tokens
```

### Try More Commands

```bash
# Get help
murmur --help

# Show configuration
murmur config

# Run with a specific model
murmur run --model opus "refactor the error handling"

# Run in verbose mode for debugging
murmur -v run "add a new feature"
```

## Initial Configuration

Murmuration works with sensible defaults, but you can customize its behavior.

### 1. Create Configuration File

Configuration is optional but recommended for customization.

```bash
# Create config directory
mkdir -p ~/.config/murmur

# Copy example config
cd ~/ai-agent-murmuration
cp config.example.toml ~/.config/murmur/config.toml
```

Edit `~/.config/murmur/config.toml`:

```toml
[agent]
# Backend to use (claude or cursor)
backend = "claude"

# Path to claude executable (if not in PATH)
claude_path = "claude"

# Default model (optional - uses claude's default if not set)
model = "claude-sonnet-4-5-20250929"

# Per-agent-type configuration
[agent.implement]
backend = "claude"
model = "claude-opus-4-5-20251101"

[agent.test]
backend = "claude"

[agent.review]
backend = "claude"
model = "claude-opus-4-5-20251101"

[workflow]
# Automatically push branch after agent completion
auto_push = true

# Automatically create PR after agent completion
auto_pr = true
```

### 2. Set Up Secrets (for GitHub features)

If you want to use GitHub integration:

```bash
# Initialize secrets file with secure permissions
murmur secrets-init
```

This creates `~/.config/murmur/secrets.toml` with `600` permissions (owner read/write only).

Edit the secrets file:

```toml
[github]
token = "github_pat_xxxxxxxxxxxx"
```

**Getting a GitHub Token:**

1. Visit https://github.com/settings/tokens?type=beta
2. Click "Generate new token" (fine-grained)
3. Set repository access (specific repos or all)
4. Add permissions:
   - **Contents**: Read and write
   - **Issues**: Read and write
   - **Pull requests**: Read and write
5. Generate and copy the token
6. Paste into `secrets.toml`

See [docs/github-token.md](github-token.md) for detailed instructions.

### 3. Environment Variables (Alternative)

Instead of config files, you can use environment variables:

```bash
# Add to ~/.bashrc or ~/.zshrc

# Claude path (if not in PATH)
export MURMUR_CLAUDE_PATH=/path/to/claude

# Default model
export MURMUR_MODEL=claude-opus-4-5-20251101

# GitHub token (alternative to secrets.toml)
export GITHUB_TOKEN=github_pat_xxxxxxxxxxxx
```

**Priority order:**
1. Command-line flags (`--claude-path`, `--model`)
2. Environment variables
3. Config file (`config.toml`)
4. Built-in defaults

## First GitHub Workflow

Now let's try the full workflow: working on a GitHub issue with automatic worktree management.

### 1. Create a Test Issue

In your GitHub repository:

1. Go to Issues → New Issue
2. Title: "Add documentation for setup process"
3. Body:
   ```markdown
   Add a setup guide to the README.

   Tasks:
   - [ ] Document prerequisites
   - [ ] Add installation steps
   - [ ] Include examples
   ```
4. Create issue (note the issue number, e.g., #42)

### 2. List Issues

```bash
# View all open issues in the repository
murmur issue list --repo owner/repo

# Or if you're in a git repo, it auto-detects:
cd ~/my-repo
murmur issue list
```

Expected output:
```
Open issues for owner/repo:

#42  Add documentation for setup process
     Created 2 minutes ago by username
     Labels: documentation

#38  Fix bug in error handling
     Created 2 days ago by username
     Labels: bug

Total: 2 issues
```

### 3. Show Issue Details

```bash
murmur issue show 42
```

Output shows full issue body, labels, dependencies, and metadata.

### 4. Work on the Issue

This is where the magic happens:

```bash
murmur work 42
```

**What happens:**

1. **Dependency Check**: Murmur checks if issue #42 has dependencies
   ```
   Checking dependencies for issue #42...
   No blocking dependencies found.
   ```

2. **Worktree Creation**: Creates an isolated git worktree
   ```
   Creating worktree for issue #42...
   Branch: murmur/issue-42
   Location: ~/.cache/murmur/worktrees/my-repo/murmur-issue-42
   Branching from: origin/main (latest)
   ```

3. **Agent Spawning**: Starts a Claude Code agent in the worktree
   ```
   Spawning agent in worktree...
   Task: "Work on issue #42: Add documentation for setup process"
   ```

4. **Agent Works**: The agent reads the issue, analyzes the code, and implements changes
   ```
   [Agent output streams here...]

   I'll add documentation for the setup process...
   [Creates/edits files...]
   ```

5. **Commit Created**: Agent commits the changes
   ```
   Committed: "docs: add setup guide to README

   Closes #42"
   ```

6. **Branch Pushed** (if `auto_push = true`):
   ```
   Pushed branch murmur/issue-42 to origin
   ```

7. **PR Created** (if `auto_pr = true`):
   ```
   Created pull request #89: Add documentation for setup process
   https://github.com/owner/repo/pull/89
   ```

### 5. Review the Work

```bash
# View the worktree location
murmur worktree list

# Navigate to see the changes
cd ~/.cache/murmur/worktrees/my-repo/murmur-issue-42
git diff origin/main

# Or review the PR on GitHub
# Click the PR link from the output
```

### 6. Clean Up (Optional)

Worktrees are cached for reuse, but you can clean old ones:

```bash
# List all worktrees
murmur worktree list

# Clean up old/unused worktrees
murmur worktree clean

# Remove a specific worktree
murmur worktree remove murmur/issue-42
```

## Next Steps

Now that you're set up, explore more features:

### Working with Dependencies

Create issues with dependencies:

```markdown
## Dependencies
Depends on #38
Blocked by #41
```

Murmur will prevent work on an issue until its dependencies are merged.

```bash
murmur issue deps 42
# Shows the dependency tree

murmur work 42
# Blocks if dependencies aren't met:
# "Error: Issue #42 is blocked by unmerged dependencies: #38, #41"
```

### Agent Types

Use specialized agents for different tasks:

```bash
# Run a test-writing agent
murmur agent test "write tests for user authentication"

# Run an implementation agent
murmur agent implement "add login endpoint"

# Run a review agent
murmur agent review "review the PR for issue #42"
```

### TDD Workflow

Run the full Test-Driven Development cycle:

```bash
murmur tdd "implement user authentication" --issue 42
```

This:
1. Spawns test agent to write failing tests
2. Verifies tests fail (red)
3. Spawns implement agent to make them pass
4. Verifies tests pass (green)
5. Spawns review agent for code review

### Check Status

Monitor running agents and worktrees:

```bash
murmur status

# Shows:
# - Active agents and their progress
# - Worktrees in use
# - Recent completions
```

### Import a Plan

If you have a `PLAN.md` file with a structured roadmap:

```bash
murmur plan import PLAN.md --repo owner/repo

# Creates GitHub issues for each phase/PR
# Links dependencies automatically
# Adds metadata for tracking
```

### Advanced Configuration

Customize per-agent-type backends:

```toml
[agent.implement]
backend = "claude"
model = "claude-opus-4-5-20251101"  # Best model for implementation

[agent.test]
backend = "cursor"  # Use unlimited tier for test generation

[agent.review]
backend = "claude"
model = "claude-opus-4-5-20251101"  # Thorough review
```

## Troubleshooting

### "claude: command not found"

- Claude CLI not in PATH
- Install from https://claude.ai/code
- Or set `MURMUR_CLAUDE_PATH` to the full path

### "Bad credentials" (GitHub)

- Token expired or invalid
- Generate new token at https://github.com/settings/tokens
- Verify permissions (Contents, Issues, Pull requests)
- Update `secrets.toml` or `GITHUB_TOKEN` env var

### "Worktree creation failed"

- Ensure you're in a git repository
- Run `git fetch` to update remote refs
- Check disk space in `~/.cache/murmur/`

### "Agent failed to spawn"

- Verify Claude CLI works: `claude --version`
- Check authentication: `claude auth status`
- Try verbose mode: `murmur -v run "test"`

### Need Help?

```bash
# Get help for any command
murmur --help
murmur run --help
murmur work --help

# Check configuration
murmur config

# Enable verbose logging
murmur -v <command>
```

## Learn More

- [PLAN.md](../PLAN.md) - Full bootstrap roadmap
- [docs/github-token.md](github-token.md) - GitHub token setup details
- [CLAUDE.md](../CLAUDE.md) - Project architecture and development guide
- [README.md](../README.md) - Project overview

## Getting Help

- **Issues**: https://github.com/jmalicki-ai-slop/ai-agent-murmuration/issues
- **Discussions**: https://github.com/jmalicki-ai-slop/ai-agent-murmuration/discussions

Welcome to Murmuration! Happy orchestrating!
