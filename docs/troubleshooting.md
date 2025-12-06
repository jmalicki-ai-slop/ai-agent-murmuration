# Troubleshooting Guide

This guide covers common issues you may encounter when using Murmur and how to resolve them.

## Installation Issues

### Rust Toolchain Not Found

**Symptom**: `cargo: command not found` when trying to build

**Cause**: Rust toolchain is not installed or not in PATH

**Solution**:
```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell environment
source $HOME/.cargo/env

# Verify installation
cargo --version
```

**Prevention**: Add `source $HOME/.cargo/env` to your shell profile (`~/.bashrc`, `~/.zshrc`)

### Claude CLI Not Found

**Symptom**: `Executable not found at 'claude'. Is the agent backend installed?`

**Cause**: Claude Code CLI is not installed or not in PATH

**Solution**:
```bash
# Check if claude is installed
which claude

# If not found, install Claude Code from https://claude.ai/code
# Then verify it's accessible
claude --version

# If installed but not in PATH, configure murmur to use the full path
murmur config
# Or set environment variable
export MURMUR_CLAUDE_PATH=/path/to/claude
```

**Prevention**: Ensure Claude CLI is in your PATH or configure the full path in `~/.config/murmur/config.toml`:
```toml
[agent]
claude_path = "/full/path/to/claude"
```

### Build Fails with Dependency Errors

**Symptom**: `error: failed to compile murmur` with linking or dependency errors

**Cause**: Missing system dependencies for Rust crates

**Solution**:
```bash
# On Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# On macOS
xcode-select --install

# On Fedora/RHEL
sudo dnf install gcc pkg-config openssl-devel

# Then rebuild
cargo clean
cargo build --release
```

**Prevention**: Install build essentials before attempting to build Rust projects

### Permission Denied When Installing to ~/.local/bin

**Symptom**: `cp: cannot create regular file '/home/user/.local/bin/murmur': Permission denied`

**Cause**: ~/.local/bin directory doesn't exist or has wrong permissions

**Solution**:
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary
cp target/release/murmur ~/.local/bin/

# Ensure it's in PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Prevention**: Create `~/.local/bin` as part of your initial system setup

## Authentication Issues

### "Bad credentials" GitHub Error

**Symptom**: `GitHub authentication error: Bad credentials` or `GitHub API error: Bad credentials`

**Cause**: One of the following:
- GitHub token is invalid, expired, or revoked
- Using OAuth token (`gho_*`) instead of Personal Access Token
- Token doesn't have required permissions

**Solution**:
```bash
# 1. Generate a new Personal Access Token
# Go to https://github.com/settings/tokens?type=beta (Fine-grained)
# Or https://github.com/settings/tokens (Classic)

# 2. For fine-grained tokens, ensure these permissions:
#    - Contents: Read and write
#    - Issues: Read and write
#    - Pull requests: Read and write
#    - Metadata: Read (auto-included)

# 3. For classic tokens, select the 'repo' scope

# 4. Set the new token
export GITHUB_TOKEN=github_pat_xxxxxxxxxxxx

# Or use secrets file (recommended)
murmur secrets-init
# Edit ~/.config/murmur/secrets.toml and add:
# [github]
# token = "github_pat_xxxxxxxxxxxx"

# 5. Verify the token works
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
```

**Prevention**:
- Use fine-grained tokens with explicit expiration reminders
- Store tokens in the secrets file with restricted permissions (600)
- Never commit tokens to git repositories

### Missing Token Scopes

**Symptom**: `Not Found` error when accessing a specific repository, or `Insufficient permissions` errors

**Cause**: Fine-grained token doesn't have access to the repository or lacks required permissions

**Solution**:
```bash
# 1. Go to https://github.com/settings/tokens
# 2. Click on your token
# 3. Under "Repository access", ensure the repository is included
# 4. Under "Permissions", verify:
#    - Contents: Read and write
#    - Issues: Read and write
#    - Pull requests: Read and write

# 5. Save changes and test
murmur issue list --repo owner/repo
```

**Prevention**: When creating fine-grained tokens, explicitly select repositories and grant all required permissions upfront

### Claude Auth Failures

**Symptom**: Claude CLI prompts for authentication or fails with auth errors when murmur spawns agents

**Cause**: Claude CLI is not authenticated

**Solution**:
```bash
# Authenticate Claude CLI
claude auth login

# Verify authentication works
claude "Hello, can you hear me?"

# Then retry murmur command
murmur run "your task"
```

**Prevention**: Authenticate Claude CLI immediately after installation

### Secrets File Permission Error

**Symptom**: `Error: Secrets file has insecure permissions` or file is ignored

**Cause**: `~/.config/murmur/secrets.toml` has world-readable permissions

**Solution**:
```bash
# Fix file permissions
chmod 600 ~/.config/murmur/secrets.toml

# Verify
ls -l ~/.config/murmur/secrets.toml
# Should show: -rw------- (600)
```

**Prevention**: Use `murmur secrets-init` which sets correct permissions automatically

## Git/Worktree Issues

### "Worktree already exists"

**Symptom**: `Worktree already exists at /path/to/worktree. Use --force to recreate.`

**Cause**: A worktree with the same branch name already exists in the cache

**Solution**:
```bash
# Option 1: Use --force to recreate
murmur worktree create feature/my-branch --force

# Option 2: List and remove the old worktree
murmur worktree list
murmur worktree clean

# Option 3: Use a different branch name
murmur worktree create feature/my-branch-v2
```

**Prevention**: Clean up old worktrees regularly with `murmur worktree clean`

### Branch Already Exists Conflict

**Symptom**: `Branch 'murmur/issue-123' already exists. Use --force to recreate.`

**Cause**: Git branch already exists locally

**Solution**:
```bash
# Option 1: Use --force flag
murmur work 123 --force

# Option 2: Delete the branch manually
git branch -D murmur/issue-123
murmur work 123

# Option 3: Use git worktree to check if it's in use
git worktree list
git worktree remove /path/to/worktree  # if found
```

**Prevention**: Use `--force` flag when re-working issues or delete old branches before retrying

### "UNIQUE constraint failed: worktrees.path"

**Symptom**: Database error: `UNIQUE constraint failed: worktrees.path`

**Cause**: Database has stale worktree metadata that conflicts with the path being created

**Solution**:
```bash
# Option 1: Inspect and clean database
sqlite3 ~/.cache/murmur/murmur.db
# Run in sqlite3:
# SELECT * FROM worktrees WHERE path LIKE '%issue-123%';
# DELETE FROM worktrees WHERE id = <problematic_id>;
# .quit

# Option 2: Remove the entire database (loses history)
rm ~/.cache/murmur/murmur.db
# Murmur will recreate on next use

# Option 3: Clean up the worktree directory manually
rm -rf ~/.cache/murmur/worktrees/<repo>/<branch>
# Then retry the command
```

**Prevention**: This is a known edge case; proper cleanup when worktrees fail is being addressed in future updates

### Stale Worktrees Not Cleaning Up

**Symptom**: `murmur worktree list` shows many old worktrees that should be removed

**Cause**: Worktree cleanup only removes worktrees older than the configured max age (default: 7 days)

**Solution**:
```bash
# Manual cleanup of old worktrees
murmur worktree clean

# For aggressive cleanup, remove manually
rm -rf ~/.cache/murmur/worktrees/<repo>/<old-branch>

# Remove all git worktrees not tracked by main repo
cd /path/to/your/repo
git worktree prune
```

**Prevention**: Run `murmur worktree clean` regularly, or configure a shorter max age in future versions

### Detached Worktree State

**Symptom**: Git operations fail inside a worktree with "not a git repository" errors

**Cause**: Worktree was manually deleted without removing git's administrative data

**Solution**:
```bash
# From the main repository
cd /path/to/main/repo
git worktree prune

# List remaining worktrees
git worktree list

# If worktree still shows, force remove
git worktree remove --force /path/to/worktree
```

**Prevention**: Always use `murmur worktree clean` or `git worktree remove` instead of `rm -rf`

## Agent Issues

### Agent Fails to Start

**Symptom**: Error starting agent process or immediate exit with no output

**Cause**: Claude executable path is incorrect or claude is not accessible

**Solution**:
```bash
# Verify claude is working
which claude
claude --version

# Test spawning manually
claude "test prompt"

# Check murmur configuration
murmur config

# Set correct path if needed
export MURMUR_CLAUDE_PATH=$(which claude)
# Or in config file
mkdir -p ~/.config/murmur
cat > ~/.config/murmur/config.toml << EOF
[agent]
claude_path = "$(which claude)"
EOF
```

**Prevention**: Verify Claude CLI installation before using murmur

### Agent Exits Immediately

**Symptom**: Agent process exits with code 1 or 2 immediately after spawning

**Cause**:
- Invalid working directory
- Claude authentication failed
- Prompt contains invalid characters or is empty

**Solution**:
```bash
# Check the working directory exists
ls -la /path/to/workdir

# Ensure claude is authenticated
claude auth status

# Try a simple prompt first
murmur run "list files in current directory" --dir .

# Check for verbose output
RUST_LOG=debug murmur run "your task"
```

**Prevention**:
- Ensure working directory exists before spawning agents
- Use simple prompts to test agent spawning
- Keep Claude CLI authenticated

### Output Parsing Errors

**Symptom**: `Failed to parse agent output` or JSON parsing errors in logs

**Cause**: Claude CLI output format changed or output is corrupted

**Solution**:
```bash
# Update Claude CLI to latest version
# (Follow Claude Code update instructions)

# Test JSON output manually
claude --print --output-format stream-json "hello"

# If format has changed, report as issue to murmur
# Temporary workaround: use older claude version if available
```

**Prevention**: Keep murmur and Claude CLI both updated, or pin versions for stability

### Agent Hangs or Times Out

**Symptom**: Agent process runs indefinitely without completing

**Cause**:
- Agent is waiting for user input (shouldn't happen with `--dangerously-skip-permissions`)
- Task is genuinely complex and taking a long time
- Agent is stuck in an error loop

**Solution**:
```bash
# Check if agent process is still running
ps aux | grep claude

# View the worktree to see what agent has done
cd ~/.cache/murmur/worktrees/<repo>/<branch>
git status
git log

# Kill the stuck agent
pkill -f "claude.*--print"

# Check agent run history for patterns
sqlite3 ~/.cache/murmur/murmur.db "SELECT * FROM agent_runs ORDER BY start_time DESC LIMIT 5;"
```

**Prevention**: Start with smaller, well-defined tasks; implement timeout mechanisms (future enhancement)

## TDD Workflow Issues

### Tests Not Detected

**Symptom**: TDD workflow can't find tests or reports "no tests found"

**Cause**: Test runner doesn't recognize the test files in the project

**Solution**:
```bash
# Verify tests exist and are runnable
cd /path/to/worktree
cargo test   # For Rust
npm test     # For JavaScript
pytest       # For Python
# etc.

# Check test file naming conventions
# Rust: files in tests/ or functions with #[test]
# Python: test_*.py or *_test.py
# JavaScript: *.test.js or *.spec.js
```

**Prevention**: Follow language-specific test file naming conventions; ensure test frameworks are configured

### VerifyRed Passes Unexpectedly

**Symptom**: VerifyRed phase expects tests to fail, but they pass

**Cause**:
- Implementation code already exists (not true TDD)
- Tests are not actually testing the new behavior
- Tests have bugs and pass for wrong reasons

**Solution**:
```bash
# Review what the tests are actually testing
cd /path/to/worktree
git diff main -- tests/

# Manually verify tests fail without implementation
# Temporarily rename or delete implementation
mv src/feature.rs src/feature.rs.backup
cargo test  # Should fail

# If tests still pass, revise the tests
# The TDD workflow will automatically go back to WriteTests phase
```

**Prevention**:
- Start with true TDD (no implementation first)
- Write tests that check specific, non-existent behavior
- Review test failures to ensure they fail for the right reason

### Max Iterations Reached

**Symptom**: `Max iterations reached (3), giving up` in TDD workflow

**Cause**: Implementation keeps failing to make tests pass after multiple attempts

**Solution**:
```bash
# Review the test failures to understand what's wrong
cd /path/to/worktree
cargo test -- --nocapture  # or equivalent for your language

# Check if tests are too complex or poorly specified
# Break down into smaller behaviors

# Increase max iterations if needed (use with caution)
murmur tdd "behavior" --max-iterations 5

# Or manually fix the issue and complete
git add .
git commit -m "Fix implementation to pass tests"
```

**Prevention**:
- Start with simple, focused behaviors
- Write clear, unambiguous tests
- Break complex features into multiple TDD cycles

### Tests Pass but Behavior Incomplete

**Symptom**: VerifyGreen passes but the feature doesn't work as expected

**Cause**: Tests have insufficient coverage or don't test the right things

**Solution**:
```bash
# Review test coverage
# Rust:
cargo tarpaulin  # or cargo-llvm-cov

# Python:
pytest --cov

# Add missing test cases
# Go back to WriteTests phase and add comprehensive tests
```

**Prevention**:
- Write tests first that truly specify behavior
- Include edge cases and error conditions
- Review test suite before claiming completion

## GitHub Integration Issues

### Rate Limiting

**Symptom**: `GitHub rate limit exceeded, resets at <timestamp>`

**Cause**: Exceeded GitHub API rate limit (5,000 requests/hour for authenticated requests)

**Solution**:
```bash
# Check current rate limit status
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit

# Wait until the reset time shown in the error

# In the meantime, work locally without GitHub operations
murmur run "local task"

# Reduce API usage by caching issue data
```

**Prevention**:
- Avoid polling GitHub API in tight loops
- Cache issue and PR data locally
- Use webhooks for real-time updates (future enhancement)

### Issue Not Found

**Symptom**: `Issue #123 not found`

**Cause**:
- Issue number doesn't exist
- Issue is in a different repository
- Token doesn't have access to the repository

**Solution**:
```bash
# Verify issue exists in the repo
# Visit: https://github.com/owner/repo/issues/123

# Check you're using the correct repository
murmur issue list --repo owner/repo | grep 123

# Verify token has access
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/owner/repo/issues/123
```

**Prevention**: Double-check issue numbers and repository names before running commands

### PR Creation Failures

**Symptom**: `Failed to create pull request` or permission errors

**Cause**:
- Token lacks "Pull requests: Write" permission
- Branch not pushed to remote
- PR already exists for this branch

**Solution**:
```bash
# Verify token permissions
# Go to https://github.com/settings/tokens
# Ensure "Pull requests: Read and write" is enabled

# Ensure branch is pushed
cd /path/to/worktree
git push -u origin branch-name

# Check if PR already exists
gh pr list --head branch-name

# Create PR manually if needed
gh pr create --title "Title" --body "Description"
```

**Prevention**:
- Use fine-grained tokens with all required write permissions
- Verify git push succeeds before creating PR
- Check for existing PRs first

### "Not Found" for Valid Repository

**Symptom**: `Not Found` error for a repository you know exists

**Cause**: Repository is private and token doesn't have access, or wrong repo name

**Solution**:
```bash
# Verify repository name and ownership
# It's owner/repo, not repo/owner

# For private repos with fine-grained tokens:
# Go to https://github.com/settings/tokens
# Click on your token
# Under "Repository access", add the repository

# Test access
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/owner/repo
```

**Prevention**: Use correct owner/repo format; ensure fine-grained tokens explicitly include private repositories

## Debug Techniques

### Using --verbose Flag

Most murmur commands support verbose logging to see what's happening:

```bash
# Enable verbose output for any command
murmur --verbose work 123

# Or use environment variable for detailed logs
RUST_LOG=debug murmur work 123

# Different log levels
RUST_LOG=info murmur work 123    # General info
RUST_LOG=debug murmur work 123   # Detailed debugging
RUST_LOG=trace murmur work 123   # Everything
```

### Checking Database with sqlite3

The murmur database contains useful debugging information:

```bash
# Open the database
sqlite3 ~/.cache/murmur/murmur.db

# View schema
.schema

# Check agent run history
SELECT agent_type, issue_number, exit_code, duration_seconds, start_time
FROM agent_runs
ORDER BY start_time DESC
LIMIT 10;

# Find failed runs
SELECT * FROM agent_runs WHERE exit_code != 0;

# Check worktree metadata
SELECT * FROM worktrees;

# Exit sqlite3
.quit
```

### Inspecting Worktree State

Worktrees contain all the work done by agents:

```bash
# List all worktrees
murmur worktree list

# Navigate to a worktree
cd ~/.cache/murmur/worktrees/<repo-name>/<branch-name>

# Check git status
git status

# View commit history
git log --oneline

# See what files were changed
git diff main

# Check metadata file
cat .murmur-worktree.toml

# View any agent-created files
ls -la
```

### Reproducing Agent Actions Manually

To understand what an agent did or debug issues:

```bash
# Navigate to the worktree
cd ~/.cache/murmur/worktrees/<repo>/<branch>

# Manually run the same command the agent would run
cargo test  # or whatever command the agent uses

# Check environment variables the agent sees
env | grep -E '(GITHUB|MURMUR|CLAUDE)'

# Run claude manually with the same prompt
claude "the exact prompt from agent_runs table"
```

### Viewing Agent Logs

```bash
# Check recent agent runs in database
sqlite3 ~/.cache/murmur/murmur.db << EOF
SELECT
  id,
  agent_type,
  prompt,
  exit_code,
  start_time
FROM agent_runs
ORDER BY start_time DESC
LIMIT 5;
EOF

# If agent is currently running, monitor its output
# (Future: implement agent log files)

# For now, use verbose mode to see real-time output
RUST_LOG=debug murmur work 123 2>&1 | tee murmur-debug.log
```

### Common Debug Checklist

When something goes wrong, check these in order:

1. **Authentication**
   - [ ] `claude auth status` - Claude CLI authenticated?
   - [ ] `echo $GITHUB_TOKEN` - GitHub token set?
   - [ ] `curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user` - Token valid?

2. **Paths and Permissions**
   - [ ] `which claude` - Claude in PATH?
   - [ ] `which murmur` - Murmur in PATH?
   - [ ] `ls -l ~/.config/murmur/secrets.toml` - Secrets file permissions (600)?
   - [ ] `pwd` - In correct directory?

3. **Git State**
   - [ ] `git status` - Clean working directory?
   - [ ] `git branch -a` - Branches exist as expected?
   - [ ] `git worktree list` - Worktrees in expected state?

4. **Database State**
   - [ ] `sqlite3 ~/.cache/murmur/murmur.db "SELECT COUNT(*) FROM agent_runs;"` - Database accessible?
   - [ ] Check for stale worktree records

5. **System Resources**
   - [ ] `df -h ~/.cache/murmur` - Disk space available?
   - [ ] `ps aux | grep claude` - Runaway agent processes?

## Getting Help

If you've tried the solutions above and still have issues:

1. **Check existing issues**: https://github.com/jmalicki-ai-slop/ai-agent-murmuration/issues
2. **Enable debug logging**: `RUST_LOG=debug murmur <command> 2>&1 | tee debug.log`
3. **Gather information**:
   - Murmur version: `murmur --version`
   - Claude version: `claude --version`
   - Operating system and version
   - Exact error message
   - Steps to reproduce
4. **Create a new issue** with the debug log and information above

## See Also

- [GitHub Token Setup Guide](github-token.md) - Detailed token configuration
- [README](../README.md) - Installation and quick start
- [CLAUDE.md](../CLAUDE.md) - Architecture and development guide
