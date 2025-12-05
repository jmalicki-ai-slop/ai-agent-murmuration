//! Configuration management for Murmuration
//!
//! Configuration is loaded with the following priority (highest to lowest):
//! 1. CLI flags
//! 2. Environment variables (MURMUR_*)
//! 3. Config file (~/.config/murmur/config.toml)
//! 4. Default values

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Backend type for agent execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Claude,
    Cursor,
}

/// Per-agent-type configuration overrides
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct TypeConfig {
    /// Override backend for this agent type
    pub backend: Option<Backend>,

    /// Override model for this agent type
    pub model: Option<String>,
}

/// Agent-related configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Global default backend
    pub backend: Backend,

    /// Global default model
    pub model: Option<String>,

    /// Path to the claude executable
    pub claude_path: String,

    /// Path to the cursor executable (optional)
    pub cursor_path: Option<String>,

    /// Configuration overrides for implement agent type
    pub implement: Option<TypeConfig>,

    /// Configuration overrides for test agent type
    pub test: Option<TypeConfig>,

    /// Configuration overrides for review agent type
    pub review: Option<TypeConfig>,

    /// Configuration overrides for coordinator agent type
    pub coordinator: Option<TypeConfig>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Claude,
            model: None, // Let claude use its default
            claude_path: "claude".to_string(),
            cursor_path: None,
            implement: None,
            test: None,
            review: None,
            coordinator: None,
        }
    }
}

/// Workflow automation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WorkflowConfig {
    /// Automatically push branch after agent completion
    pub auto_push: bool,

    /// Automatically create PR after agent completion
    pub auto_pr: bool,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            auto_push: true,
            auto_pr: true,
        }
    }
}

/// Root configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Agent configuration
    pub agent: AgentConfig,

    /// Workflow automation configuration
    pub workflow: WorkflowConfig,
}

impl Config {
    /// Load configuration from the default config file location
    ///
    /// Returns default config if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path();

        if let Some(path) = config_path {
            if path.exists() {
                return Self::load_from_file(&path);
            }
        }

        Ok(Self::default())
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(Error::Io)?;
        toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))
    }

    /// Get the default config file path
    ///
    /// Returns `~/.config/murmur/config.toml` on Unix
    pub fn default_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("murmur").join("config.toml"))
    }

    /// Apply environment variable overrides
    ///
    /// Supported variables:
    /// - MURMUR_CLAUDE_PATH: Path to claude executable
    /// - MURMUR_MODEL: Model to use
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(claude_path) = std::env::var("MURMUR_CLAUDE_PATH") {
            self.agent.claude_path = claude_path;
        }

        if let Ok(model) = std::env::var("MURMUR_MODEL") {
            self.agent.model = Some(model);
        }

        self
    }

    /// Apply CLI flag overrides
    pub fn with_cli_overrides(
        mut self,
        claude_path: Option<String>,
        model: Option<String>,
    ) -> Self {
        if let Some(path) = claude_path {
            self.agent.claude_path = path;
        }

        if let Some(m) = model {
            self.agent.model = Some(m);
        }

        self
    }

    /// Load configuration with all overrides applied
    ///
    /// Priority: CLI > env > config file > defaults
    pub fn load_with_overrides(claude_path: Option<String>, model: Option<String>) -> Result<Self> {
        Ok(Self::load()?
            .with_env_overrides()
            .with_cli_overrides(claude_path, model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.agent.backend, Backend::Claude);
        assert_eq!(config.agent.claude_path, "claude");
        assert!(config.agent.model.is_none());
        assert!(config.agent.cursor_path.is_none());
        assert!(config.agent.implement.is_none());
        assert!(config.agent.test.is_none());
        assert!(config.agent.review.is_none());
        assert!(config.agent.coordinator.is_none());
    }

    #[test]
    fn test_cli_overrides() {
        let config = Config::default()
            .with_cli_overrides(Some("/custom/claude".to_string()), Some("opus".to_string()));

        assert_eq!(config.agent.claude_path, "/custom/claude");
        assert_eq!(config.agent.model, Some("opus".to_string()));
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
[agent]
claude_path = "/usr/local/bin/claude"
model = "claude-sonnet-4-20250514"

[workflow]
auto_push = false
auto_pr = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.agent.claude_path, "/usr/local/bin/claude");
        assert_eq!(
            config.agent.model,
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert!(!config.workflow.auto_push);
        assert!(config.workflow.auto_pr);
    }

    #[test]
    fn test_partial_toml() {
        let toml = r#"
[agent]
model = "opus"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        // claude_path should use default
        assert_eq!(config.agent.claude_path, "claude");
        assert_eq!(config.agent.model, Some("opus".to_string()));
    }

    #[test]
    fn test_backend_enum_deserialization() {
        let toml = r#"
[agent]
backend = "claude"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.agent.backend, Backend::Claude);

        let toml = r#"
[agent]
backend = "cursor"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.agent.backend, Backend::Cursor);
    }

    #[test]
    fn test_per_type_config() {
        let toml = r#"
[agent]
backend = "claude"
model = "claude-sonnet-4-20250514"
claude_path = "claude"
cursor_path = "cursor"

[agent.implement]
model = "claude-sonnet-4-20250514"

[agent.review]
model = "claude-haiku-4-20250514"
"#;
        let config: Config = toml::from_str(toml).unwrap();

        // Global config
        assert_eq!(config.agent.backend, Backend::Claude);
        assert_eq!(
            config.agent.model,
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert_eq!(config.agent.claude_path, "claude");
        assert_eq!(config.agent.cursor_path, Some("cursor".to_string()));

        // Implement type config
        assert!(config.agent.implement.is_some());
        let implement = config.agent.implement.as_ref().unwrap();
        assert_eq!(
            implement.model,
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert!(implement.backend.is_none());

        // Review type config
        assert!(config.agent.review.is_some());
        let review = config.agent.review.as_ref().unwrap();
        assert_eq!(review.model, Some("claude-haiku-4-20250514".to_string()));
        assert!(review.backend.is_none());

        // Unspecified types should be None
        assert!(config.agent.test.is_none());
        assert!(config.agent.coordinator.is_none());
    }

    #[test]
    fn test_per_type_backend_override() {
        let toml = r#"
[agent]
backend = "claude"
claude_path = "claude"
cursor_path = "/usr/local/bin/cursor"

[agent.implement]
backend = "cursor"
model = "gpt-4"
"#;
        let config: Config = toml::from_str(toml).unwrap();

        assert_eq!(config.agent.backend, Backend::Claude);

        let implement = config.agent.implement.as_ref().unwrap();
        assert_eq!(implement.backend, Some(Backend::Cursor));
        assert_eq!(implement.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_typeconfig_default() {
        let type_config = TypeConfig::default();
        assert!(type_config.backend.is_none());
        assert!(type_config.model.is_none());
    }

    #[test]
    fn test_empty_per_type_sections() {
        let toml = r#"
[agent]
backend = "claude"

[agent.implement]

[agent.test]
"#;
        let config: Config = toml::from_str(toml).unwrap();

        // Empty sections should parse as Some with default (None) values
        assert!(config.agent.implement.is_some());
        assert!(config.agent.test.is_some());

        let implement = config.agent.implement.as_ref().unwrap();
        assert!(implement.backend.is_none());
        assert!(implement.model.is_none());
    }

    #[test]
    fn test_issue_125_example() {
        // Test the exact example from issue #125
        let toml = r#"
[agent]
backend = "claude"
model = "claude-sonnet-4-20250514"
claude_path = "claude"
cursor_path = "cursor"

[agent.implement]
model = "claude-sonnet-4-20250514"

[agent.review]
model = "claude-haiku-4-20250514"
"#;
        let config: Config = toml::from_str(toml).unwrap();

        // Verify global config
        assert_eq!(config.agent.backend, Backend::Claude);
        assert_eq!(
            config.agent.model,
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert_eq!(config.agent.claude_path, "claude");
        assert_eq!(config.agent.cursor_path, Some("cursor".to_string()));

        // Verify implement config
        let implement = config.agent.implement.as_ref().unwrap();
        assert_eq!(
            implement.model,
            Some("claude-sonnet-4-20250514".to_string())
        );
        assert!(implement.backend.is_none());

        // Verify review config
        let review = config.agent.review.as_ref().unwrap();
        assert_eq!(review.model, Some("claude-haiku-4-20250514".to_string()));
        assert!(review.backend.is_none());
    }
}
