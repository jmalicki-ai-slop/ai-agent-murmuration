# Configuration Reference

Murmur is highly configurable, allowing you to customize agent behavior, choose different models for different tasks, and automate workflow steps.

## Quick Start

Murmur works out-of-the-box with sensible defaults. Configuration is optional but recommended for:

- Using different models for different agent types (cost optimization)
- Customizing workflow automation (auto-commit, auto-push, auto-PR)
- Switching between Claude and Cursor backends
- Setting custom executable paths

## Configuration File Location

```
~/.config/murmur/config.toml
```

If this file doesn't exist, Murmur uses built-in defaults. Create it to customize behavior.

## Full Example Configuration

Here's a complete example with all available options:

```toml
[agent]
# Global default backend: "claude" or "cursor"
backend = "claude"

# Global default model (leave empty to use backend's default)
model = "claude-sonnet-4-20250514"

# Path to the claude executable
claude_path = "claude"

# Path to the cursor executable (optional, only needed if using cursor backend)
cursor_path = "/usr/local/bin/cursor"

# Per-agent-type overrides for implement agents
[agent.implement]
# Override model for implement agents (code writing)
model = "claude-sonnet-4-20250514"
# backend = "claude"  # Uncomment to override backend for this type

# Per-agent-type overrides for test agents
[agent.test]
# Use a faster/cheaper model for test agents
model = "claude-sonnet-4-20250514"

# Per-agent-type overrides for review agents
[agent.review]
# Use a cheaper model for code review
model = "claude-haiku-4-20250514"

# Per-agent-type overrides for coordinator agents
[agent.coordinator]
# Use a balanced model for coordination
model = "claude-sonnet-4-20250514"

[workflow]
# Automatically commit changes after agent completion (default: true)
auto_commit = true

# Automatically push branch after agent completion (default: true)
auto_push = true

# Automatically create PR after agent completion (default: true)
auto_pr = true

# Automatically re-spawn agent to address review feedback (default: false)
# This is opt-in due to potential cost implications
auto_review_loop = false
```

## Configuration Sections

### `[agent]` - Global Agent Settings

Global defaults that apply to all agent types unless overridden.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `backend` | string | `"claude"` | Backend to use: `"claude"` or `"cursor"` |
| `model` | string | `null` | Model name. If not set, backend uses its default. |
| `claude_path` | string | `"claude"` | Path to the claude executable |
| `cursor_path` | string | `null` | Path to the cursor executable (only needed if using cursor backend) |

**Example:**
```toml
[agent]
backend = "claude"
model = "claude-sonnet-4-20250514"
claude_path = "/usr/local/bin/claude"
```

### Per-Agent-Type Configuration

You can override settings for specific agent types to optimize cost and performance.

#### `[agent.implement]` - Implement Agent Overrides

Implement agents write production code. You might want to use your most capable model here.

```toml
[agent.implement]
model = "claude-sonnet-4-20250514"  # Use Sonnet for code implementation
```

#### `[agent.test]` - Test Agent Overrides

Test agents write tests and verify implementations.

```toml
[agent.test]
model = "claude-sonnet-4-20250514"  # Use Sonnet for test writing
```

#### `[agent.review]` - Review Agent Overrides

Review agents analyze code for issues and provide feedback. This is a good candidate for cost optimization.

```toml
[agent.review]
model = "claude-haiku-4-20250514"  # Use cheaper Haiku for reviews
```

#### `[agent.coordinator]` - Coordinator Agent Overrides

Coordinator agents manage multi-agent workflows and orchestration.

```toml
[agent.coordinator]
model = "claude-sonnet-4-20250514"  # Use Sonnet for coordination
```

### `[workflow]` - Workflow Automation

Control which steps are automated after agent completion.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `auto_commit` | boolean | `true` | Automatically commit changes after agent completion |
| `auto_push` | boolean | `true` | Automatically push branch to remote |
| `auto_pr` | boolean | `true` | Automatically create pull request |
| `auto_review_loop` | boolean | `false` | Automatically re-spawn agent to address review feedback |

**Example - Manual workflow:**
```toml
[workflow]
auto_commit = false  # I'll commit manually
auto_push = false    # I'll push manually
auto_pr = false      # I'll create PRs manually
```

**Example - Fully automated workflow:**
```toml
[workflow]
auto_commit = true
auto_push = true
auto_pr = true
auto_review_loop = true  # Experimental: auto-address review comments
```

## Cost Optimization Strategies

Different models have different costs and capabilities. Here are common optimization strategies:

### Strategy 1: Sonnet for Everything (Balanced)

Best for: General use, balanced cost/quality

```toml
[agent]
model = "claude-sonnet-4-20250514"
```

### Strategy 2: Cost-Optimized (Recommended)

Use cheaper models where appropriate:

```toml
[agent]
model = "claude-sonnet-4-20250514"  # Default to Sonnet

[agent.implement]
model = "claude-sonnet-4-20250514"  # Keep Sonnet for code

[agent.review]
model = "claude-haiku-4-20250514"   # Use Haiku for reviews (cheaper)

[agent.coordinator]
model = "claude-sonnet-4-20250514"  # Sonnet for orchestration
```

### Strategy 3: Maximum Quality

Use the most capable model for critical tasks:

```toml
[agent]
model = "claude-sonnet-4-20250514"  # Default to Sonnet

[agent.implement]
model = "claude-sonnet-4-20250514"  # Sonnet for complex implementations

[agent.test]
model = "claude-sonnet-4-20250514"  # Sonnet for thorough testing

[agent.review]
model = "claude-sonnet-4-20250514"  # Sonnet for detailed review
```

### Strategy 4: Mixed Backends

Use different backends for different agent types:

```toml
[agent]
backend = "claude"
claude_path = "claude"
cursor_path = "cursor"

[agent.implement]
backend = "cursor"  # Use Cursor for implementation
model = "gpt-4"

[agent.review]
backend = "claude"  # Use Claude for review
model = "claude-haiku-4-20250514"
```

## Environment Variables

Environment variables override config file settings but are overridden by CLI flags.

| Variable | Description | Example |
|----------|-------------|---------|
| `MURMUR_CLAUDE_PATH` | Path to claude executable | `/usr/local/bin/claude` |
| `MURMUR_MODEL` | Model to use | `claude-sonnet-4-20250514` |
| `MURMUR_BACKEND` | Backend to use | `claude` or `cursor` |
| `GITHUB_TOKEN` | GitHub Personal Access Token | `ghp_xxxx...` or `github_pat_xxxx...` |

**Example:**
```bash
export MURMUR_MODEL=claude-sonnet-4-20250514
export MURMUR_BACKEND=claude
export GITHUB_TOKEN=ghp_your_token_here
```

## Override Precedence

Configuration is resolved with the following priority (highest to lowest):

1. **CLI flags** - Passed directly to `murmur` commands
2. **Environment variables** - `MURMUR_*` variables
3. **Config file** - `~/.config/murmur/config.toml`
4. **Default values** - Built-in defaults

### Examples

**CLI flag wins:**
```bash
# Config file says: model = "claude-haiku-4-20250514"
# This uses Sonnet instead:
murmur run --model claude-sonnet-4-20250514 "implement feature"
```

**Environment variable wins over config file:**
```bash
# Config file says: model = "claude-haiku-4-20250514"
export MURMUR_MODEL=claude-sonnet-4-20250514
# This uses Sonnet:
murmur run "implement feature"
```

**Config file wins over defaults:**
```toml
# Without this, defaults to "claude" in PATH
[agent]
claude_path = "/usr/local/bin/claude"
```

## Per-Agent-Type Resolution

When spawning an agent, settings are resolved as:

1. **Type-specific config** (e.g., `[agent.implement]`)
2. **Global agent config** (e.g., `[agent]`)
3. **Default values**

Type-specific settings can partially override - you can override just the model while keeping the global backend.

**Example:**
```toml
[agent]
backend = "claude"           # All agents use Claude
model = "claude-sonnet-4-20250514"  # Default to Sonnet

[agent.review]
model = "claude-haiku-4-20250514"   # Override ONLY model for review agents
# backend is still "claude" (inherited from global)
```

## Secrets Management

Sensitive credentials (like GitHub tokens) should not be stored in the config file. Use the separate secrets file instead.

### Creating the Secrets File

```bash
murmur secrets-init
```

This creates `~/.config/murmur/secrets.toml` with secure permissions (0600 on Unix).

### Secrets File Format

```toml
# ~/.config/murmur/secrets.toml

[github]
# GitHub Personal Access Token
# Create at: https://github.com/settings/tokens
token = "ghp_xxxxxxxxxxxx"  # or github_pat_xxxxxxxxxxxx
```

### Required Permissions

The file must have `600` permissions (owner read/write only). Murmur will refuse to read it if it's world-readable.

```bash
chmod 600 ~/.config/murmur/secrets.toml
```

### Token Loading Priority

1. `GITHUB_TOKEN` environment variable
2. `~/.config/murmur/secrets.toml`

See [GitHub Token Setup](github-token.md) for detailed information on creating tokens and required permissions.

## Viewing Current Configuration

Check what configuration is currently active:

```bash
murmur config
```

This shows:
- Current agent settings (backend, model, paths)
- Config file location and whether it exists
- Secrets file location and whether it exists

**Example output:**
```
Murmur Configuration
====================

Agent Settings:
  backend: claude
  claude_path: claude
  model: claude-sonnet-4-20250514

Config file: /home/user/.config/murmur/config.toml
  (exists)

Secrets file: /home/user/.config/murmur/secrets.toml
  (exists)
```

## Common Configuration Patterns

### Minimal Setup

Just use defaults, no config file needed:

```bash
# Install murmur
cargo install --path .

# Create secrets for GitHub access
murmur secrets-init
# Edit ~/.config/murmur/secrets.toml and add your token

# Start working
murmur work 123
```

### Development Setup

Optimize for fast iteration:

```toml
[agent]
model = "claude-sonnet-4-20250514"

[workflow]
auto_commit = true
auto_push = false    # Don't push automatically during dev
auto_pr = false      # Create PRs manually
```

### CI/CD Setup

Optimize for automation:

```toml
[workflow]
auto_commit = true
auto_push = true
auto_pr = true

[agent]
model = "claude-sonnet-4-20250514"
```

Use environment variables in CI:

```yaml
# .github/workflows/murmur.yml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  MURMUR_MODEL: claude-sonnet-4-20250514
```

### Cost-Conscious Setup

Minimize API costs:

```toml
[agent]
model = "claude-sonnet-4-20250514"  # Balanced default

[agent.review]
model = "claude-haiku-4-20250514"   # Cheaper for reviews

[workflow]
auto_review_loop = false  # Don't auto-retry (avoid cost loops)
```

### Multi-Repository Setup

Use the same config across all repos:

```toml
# ~/.config/murmur/config.toml applies to all repos
[agent]
backend = "claude"
model = "claude-sonnet-4-20250514"

[workflow]
auto_commit = true
auto_push = true
auto_pr = true
```

## Troubleshooting

### "claude: command not found"

The `claude` executable is not in your PATH.

**Solution:** Set the full path in config:
```toml
[agent]
claude_path = "/usr/local/bin/claude"
```

Or use an environment variable:
```bash
export MURMUR_CLAUDE_PATH=/usr/local/bin/claude
```

### Wrong model being used

Check the precedence chain:

```bash
# See what's actually configured
murmur config

# CLI flags override everything
murmur run --model claude-sonnet-4-20250514 "task"
```

### Secrets file permission errors

**Error:** `Secrets file has insecure permissions`

**Solution:**
```bash
chmod 600 ~/.config/murmur/secrets.toml
```

### Config file not being loaded

Ensure it's in the correct location:

```bash
# Should be here:
~/.config/murmur/config.toml

# Not here:
./config.toml
~/murmur/config.toml
```

### GitHub API "Bad credentials"

Check token loading:

1. Verify token is set:
   ```bash
   echo $GITHUB_TOKEN  # Should show your token
   # OR
   cat ~/.config/murmur/secrets.toml  # Should have token = "..."
   ```

2. Test the token:
   ```bash
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
   ```

3. Verify token type:
   - Fine-grained PAT: `github_pat_...` ✓
   - Classic PAT: `ghp_...` ✓
   - OAuth token: `gho_...` ✗ (not supported)

## Further Reading

- [GitHub Token Setup](github-token.md) - Detailed guide for creating GitHub tokens
- [CLAUDE.md](../CLAUDE.md) - Project overview and development guide
- [PLAN.md](../PLAN.md) - Roadmap and feature plans
