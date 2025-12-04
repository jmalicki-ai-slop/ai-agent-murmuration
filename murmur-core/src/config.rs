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

/// Agent-related configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Path to the claude executable
    pub claude_path: String,

    /// Model to use for Claude
    pub model: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            claude_path: "claude".to_string(),
            model: None, // Let claude use its default
        }
    }
}

/// Root configuration structure
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// Agent configuration
    pub agent: AgentConfig,
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
        assert_eq!(config.agent.claude_path, "claude");
        assert!(config.agent.model.is_none());
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
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.agent.claude_path, "/usr/local/bin/claude");
        assert_eq!(
            config.agent.model,
            Some("claude-sonnet-4-20250514".to_string())
        );
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
}
