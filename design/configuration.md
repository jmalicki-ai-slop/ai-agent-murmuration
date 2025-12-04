# Configuration Design

## Overview

Configuration management for the Dispatch system, supporting file-based config, environment variables, and runtime overrides.

---

## Configuration Hierarchy

```
Priority (highest to lowest):
1. CLI arguments
2. Environment variables
3. Project config (.dispatch/config.toml)
4. User config (~/.config/dispatch/config.toml)
5. Default values
```

---

## File Locations

```
~/.config/dispatch/
├── config.toml           # User-level configuration
├── credentials.toml      # Sensitive credentials (optional)
└── prompts/              # Custom agent prompts (optional)
    └── *.md

~/.local/share/dispatch/
├── dispatch.db           # SQLite database
└── logs/                 # Log files

.dispatch/                # Project-level (in repo root)
├── config.toml           # Project configuration
└── prompts/              # Project-specific prompts
    └── *.md
```

---

## Configuration File Format

### Full Configuration Schema

```toml
# dispatch configuration file

#──────────────────────────────────────────────────────────────────────────────
# Database
#──────────────────────────────────────────────────────────────────────────────
[database]
path = "~/.local/share/dispatch/dispatch.db"
# Max connections in pool
pool_size = 5
# Connection timeout in seconds
timeout = 30

#──────────────────────────────────────────────────────────────────────────────
# GitHub Integration
#──────────────────────────────────────────────────────────────────────────────
[github]
# GitHub personal access token (prefer GITHUB_TOKEN env var)
# token = "ghp_xxxxxxxxxxxx"

# Default repository owner/organization
owner = "myorg"

# Default repository name
repo = "myrepo"

# Webhook secret for verifying GitHub webhooks
# webhook_secret = "secret"

# Sync interval in seconds (0 to disable auto-sync)
sync_interval = 300

# Labels to apply to all dispatch-managed issues
default_labels = ["dispatch"]

# Custom label mappings
[github.labels]
priority_critical = "P0"
priority_high = "P1"
priority_medium = "P2"
priority_low = "P3"
type_feature = "enhancement"
type_bug = "bug"

#──────────────────────────────────────────────────────────────────────────────
# Agents
#──────────────────────────────────────────────────────────────────────────────
[agents]
# Maximum concurrent agents
max_concurrent = 4

# Path to Claude binary
claude_path = "claude"

# Default model for agents (can be overridden per agent type)
default_model = "claude-sonnet-4-20250514"

# Default timeout for agent tasks in seconds
default_timeout = 3600

# Heartbeat check interval in seconds
heartbeat_interval = 30

# Consider agent dead after this many seconds without heartbeat
heartbeat_timeout = 120

# Directory containing agent prompts
prompts_dir = "~/.config/dispatch/prompts"

# Per-agent-type configuration
[agents.coder]
model = "claude-sonnet-4-20250514"
timeout = 7200
allowed_tools = ["Read", "Write", "Edit", "Bash", "Grep", "Glob"]

[agents.reviewer]
model = "claude-sonnet-4-20250514"
timeout = 1800
allowed_tools = ["Read", "Grep", "Glob"]
denied_tools = ["Write", "Edit", "Bash"]

[agents.pm]
model = "claude-sonnet-4-20250514"
timeout = 1800

[agents.security]
model = "claude-sonnet-4-20250514"
timeout = 3600
allowed_tools = ["Read", "Grep", "Glob", "Bash"]

[agents.test]
model = "claude-sonnet-4-20250514"
timeout = 3600
allowed_tools = ["Read", "Write", "Edit", "Bash", "Grep", "Glob"]

[agents.docs]
model = "claude-sonnet-4-20250514"
timeout = 1800

[agents.architect]
model = "claude-sonnet-4-20250514"
timeout = 3600

[agents.coordinator]
model = "claude-sonnet-4-20250514"
timeout = 7200

#──────────────────────────────────────────────────────────────────────────────
# Worktrees
#──────────────────────────────────────────────────────────────────────────────
[worktrees]
# Base directory for worktrees (relative to repo root)
base_dir = ".dispatch-worktrees"

# Auto-cleanup orphaned worktrees
auto_cleanup = true

# Days to keep completed worktrees before cleanup
retention_days = 7

# Branch naming pattern
# Available variables: {issue_id}, {title}, {type}
branch_pattern = "dispatch/{issue_id}/{title}"

#──────────────────────────────────────────────────────────────────────────────
# Governance
#──────────────────────────────────────────────────────────────────────────────
[governance]
# Enable Sangha voting system
enabled = true

# Default voting deadline in hours
voting_deadline_hours = 24

# Auto-execute approved proposals
auto_execute = true

# Maximum iterations before escalating to human
max_iterations = 3

# Test coverage target percentage
test_coverage_target = 80.0

# Required reviewers for different review types
[governance.reviewers]
architecture = ["architect", "reviewer"]
security = ["security"]
code = ["reviewer"]
test = ["test", "reviewer"]

#──────────────────────────────────────────────────────────────────────────────
# Epics and Stages
#──────────────────────────────────────────────────────────────────────────────
[epics]
# Default stages for new epics
default_stages = [
    { name = "Design", gate = "approval" },
    { name = "Implementation", gate = "review" },
    { name = "Testing", gate = "checkpoint" },
    { name = "Documentation", gate = null },
]

# Default gate approvers (GitHub usernames or "any")
default_approvers = ["any"]

#──────────────────────────────────────────────────────────────────────────────
# Web Server
#──────────────────────────────────────────────────────────────────────────────
[server]
# Host to bind to
host = "127.0.0.1"

# Port to listen on
port = 8080

# Enable CORS
cors_enabled = true

# Allowed CORS origins (empty = all)
cors_origins = []

# Static files directory for web UI
static_dir = "~/.local/share/dispatch/web"

#──────────────────────────────────────────────────────────────────────────────
# Logging
#──────────────────────────────────────────────────────────────────────────────
[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: pretty, json
format = "pretty"

# Log to file
file_enabled = true
file_path = "~/.local/share/dispatch/logs/dispatch.log"

# Max log file size in MB before rotation
max_size_mb = 100

# Number of rotated files to keep
max_files = 5

#──────────────────────────────────────────────────────────────────────────────
# TUI
#──────────────────────────────────────────────────────────────────────────────
[tui]
# Refresh rate in milliseconds
tick_rate = 250

# Default view on startup
default_view = "dashboard"

# Theme (default, dark, light)
theme = "default"

# Show keyboard shortcuts in status bar
show_shortcuts = true
```

---

## Environment Variables

```bash
# Core
DISPATCH_CONFIG      # Path to config file
DISPATCH_DATABASE    # Path to database file
DISPATCH_LOG_LEVEL   # Override log level

# GitHub
GITHUB_TOKEN         # GitHub personal access token
GITHUB_WEBHOOK_SECRET # Webhook secret

# Agents
DISPATCH_CLAUDE_PATH # Path to claude binary
DISPATCH_MAX_AGENTS  # Max concurrent agents

# Server
DISPATCH_HOST        # Server host
DISPATCH_PORT        # Server port
```

---

## Key Data Structures

```rust
// dispatch-core/src/config.rs

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub database: DatabaseConfig,
    pub github: GitHubConfig,
    pub agents: AgentsConfig,
    pub worktrees: WorktreesConfig,
    pub governance: GovernanceConfig,
    pub epics: EpicsConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub tui: TuiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub pool_size: u32,
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub owner: String,
    pub repo: String,
    pub webhook_secret: Option<String>,
    pub sync_interval: u64,
    pub default_labels: Vec<String>,
    pub labels: LabelMappings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentsConfig {
    pub max_concurrent: usize,
    pub claude_path: PathBuf,
    pub default_model: String,
    pub default_timeout: u64,
    pub heartbeat_interval: u64,
    pub heartbeat_timeout: u64,
    pub prompts_dir: PathBuf,

    // Per-type config
    pub coder: AgentTypeConfig,
    pub reviewer: AgentTypeConfig,
    pub pm: AgentTypeConfig,
    pub security: AgentTypeConfig,
    pub test: AgentTypeConfig,
    pub docs: AgentTypeConfig,
    pub architect: AgentTypeConfig,
    pub coordinator: AgentTypeConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentTypeConfig {
    pub model: Option<String>,
    pub timeout: Option<u64>,
    pub allowed_tools: Option<Vec<String>>,
    pub denied_tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GovernanceConfig {
    pub enabled: bool,
    pub voting_deadline_hours: u64,
    pub auto_execute: bool,
    pub max_iterations: u32,
    pub test_coverage_target: f64,
    pub reviewers: ReviewersConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            github: GitHubConfig::default(),
            agents: AgentsConfig::default(),
            worktrees: WorktreesConfig::default(),
            governance: GovernanceConfig::default(),
            epics: EpicsConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            tui: TuiConfig::default(),
        }
    }
}
```

---

## Configuration Loading

```rust
// dispatch-core/src/config.rs

impl Config {
    /// Load configuration from all sources
    pub fn load(cli_config_path: Option<&Path>) -> Result<Self> {
        let mut config = Config::default();

        // 1. Load user config
        if let Some(user_config) = Self::user_config_path() {
            if user_config.exists() {
                config.merge_from_file(&user_config)?;
            }
        }

        // 2. Load project config
        if let Some(project_config) = Self::find_project_config() {
            config.merge_from_file(&project_config)?;
        }

        // 3. Load CLI-specified config
        if let Some(path) = cli_config_path {
            config.merge_from_file(path)?;
        }

        // 4. Apply environment variables
        config.apply_env_overrides();

        // 5. Validate
        config.validate()?;

        Ok(config)
    }

    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("dispatch/config.toml"))
    }

    fn find_project_config() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let config_path = current.join(".dispatch/config.toml");
            if config_path.exists() {
                return Some(config_path);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    fn merge_from_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let file_config: Config = toml::from_str(&content)?;
        self.merge(file_config);
        Ok(())
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(path) = std::env::var("DISPATCH_DATABASE") {
            self.database.path = PathBuf::from(path);
        }
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            self.github.token = Some(token);
        }
        if let Ok(secret) = std::env::var("GITHUB_WEBHOOK_SECRET") {
            self.github.webhook_secret = Some(secret);
        }
        if let Ok(level) = std::env::var("DISPATCH_LOG_LEVEL") {
            self.logging.level = level;
        }
        // ... more overrides
    }

    fn validate(&self) -> Result<()> {
        // Validate required fields
        if self.github.owner.is_empty() {
            return Err(DispatchError::Config("github.owner is required".into()));
        }
        if self.github.repo.is_empty() {
            return Err(DispatchError::Config("github.repo is required".into()));
        }

        // Validate paths
        if !self.agents.claude_path.exists() {
            // Try to find in PATH
            if which::which("claude").is_err() {
                return Err(DispatchError::Config(
                    "claude binary not found".into()
                ));
            }
        }

        Ok(())
    }
}
```

---

## Runtime Configuration Store

```rust
// dispatch-db/src/repos/config.rs

/// Key-value store for runtime configuration
pub struct ConfigStore {
    pool: SqlitePool,
}

impl ConfigStore {
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let row = sqlx::query!(
            "SELECT value FROM config_store WHERE key = ?",
            key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.value))
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            r#"
            INSERT INTO config_store (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?
            "#,
            key, value, now, value, now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        sqlx::query!("DELETE FROM config_store WHERE key = ?", key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<(String, String)>> {
        let rows = if let Some(prefix) = prefix {
            let pattern = format!("{}%", prefix);
            sqlx::query!(
                "SELECT key, value FROM config_store WHERE key LIKE ?",
                pattern
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query!("SELECT key, value FROM config_store")
                .fetch_all(&self.pool)
                .await?
        };

        Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
    }
}
```

---

## CLI Config Commands

```
dispatch config show              # Show current configuration
dispatch config show --defaults   # Show with default values highlighted
dispatch config get <key>         # Get specific value
dispatch config set <key> <value> # Set value (updates runtime store)
dispatch config edit              # Open in $EDITOR
dispatch config init              # Initialize default config file
dispatch config validate          # Validate current config
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-007 | Configuration loading | `dispatch-core/src/config.rs` |
| PR-007a | Config CLI commands | `dispatch-cli/src/commands/config.rs` |
| PR-007b | Runtime config store | `dispatch-db/src/repos/config.rs` |
