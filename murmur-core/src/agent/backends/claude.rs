//! Claude Code backend implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::{Error, Result};

use super::super::spawn::AgentHandle;
use super::Backend;

/// Claude Code backend implementation
#[derive(Debug, Clone)]
pub struct ClaudeBackend {
    pub claude_path: String,
    pub model: Option<String>,
    pub allowed_tools: Vec<String>,
    pub env_vars: HashMap<String, String>,
}

impl ClaudeBackend {
    /// Create a new Claude backend with default settings
    pub fn new() -> Self {
        Self {
            claude_path: "claude".to_string(),
            model: None,
            allowed_tools: Vec::new(),
            env_vars: HashMap::new(),
        }
    }

    /// Create a Claude backend with custom path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.claude_path = path.into();
        self
    }

    /// Create a Claude backend with a specific model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Add an allowed tool
    pub fn with_allowed_tool(mut self, tool: impl Into<String>) -> Self {
        self.allowed_tools.push(tool.into());
        self
    }

    /// Set allowed tools
    pub fn with_allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// Set environment variables
    pub fn with_env_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.env_vars = vars;
        self
    }
}

impl Default for ClaudeBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Backend for ClaudeBackend {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn build_command(&self, workdir: &Path) -> Command {
        let mut cmd = Command::new(&self.claude_path);
        cmd.arg("--print")
            .arg("--verbose")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--dangerously-skip-permissions");

        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }

        // Add allowed tools
        for tool in &self.allowed_tools {
            cmd.arg("--allowed-tool").arg(tool);
        }

        // Add environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        cmd.current_dir(workdir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd
    }

    async fn spawn(&self, prompt: &str, workdir: &Path) -> Result<AgentHandle> {
        let workdir_str = workdir
            .to_str()
            .ok_or_else(|| Error::Agent("Invalid working directory path".to_string()))?
            .to_string();

        if !workdir.exists() {
            return Err(Error::Agent(format!(
                "Working directory does not exist: {}",
                workdir_str
            )));
        }

        let mut cmd = self.build_command(workdir);
        cmd.arg(prompt);

        let child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Agent(format!(
                    "Claude executable not found at '{}'. Is Claude Code installed?",
                    self.claude_path
                ))
            } else {
                Error::Io(e)
            }
        })?;

        Ok(AgentHandle::new(child, prompt.to_string(), workdir_str))
    }

    fn is_available(&self) -> bool {
        std::process::Command::new(&self.claude_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_backend_name() {
        let backend = ClaudeBackend::new();
        assert_eq!(backend.name(), "claude");
    }

    #[test]
    fn test_claude_backend_builder() {
        let backend = ClaudeBackend::new()
            .with_path("/custom/claude")
            .with_model("opus");

        assert_eq!(backend.claude_path, "/custom/claude");
        assert_eq!(backend.model, Some("opus".to_string()));
    }

    #[test]
    fn test_claude_backend_with_allowed_tools() {
        let backend = ClaudeBackend::new()
            .with_allowed_tool("bash")
            .with_allowed_tool("read");

        assert_eq!(backend.allowed_tools.len(), 2);
        assert!(backend.allowed_tools.contains(&"bash".to_string()));
        assert!(backend.allowed_tools.contains(&"read".to_string()));
    }

    #[test]
    fn test_claude_backend_with_allowed_tools_vec() {
        let tools = vec!["bash".to_string(), "read".to_string(), "write".to_string()];
        let backend = ClaudeBackend::new().with_allowed_tools(tools.clone());

        assert_eq!(backend.allowed_tools, tools);
    }

    #[test]
    fn test_claude_backend_with_env_vars() {
        let backend = ClaudeBackend::new()
            .with_env("FOO", "bar")
            .with_env("BAZ", "qux");

        assert_eq!(backend.env_vars.len(), 2);
        assert_eq!(backend.env_vars.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(backend.env_vars.get("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_claude_backend_with_env_vars_hashmap() {
        let mut vars = HashMap::new();
        vars.insert("KEY1".to_string(), "value1".to_string());
        vars.insert("KEY2".to_string(), "value2".to_string());

        let backend = ClaudeBackend::new().with_env_vars(vars.clone());

        assert_eq!(backend.env_vars, vars);
    }

    #[test]
    fn test_claude_backend_full_configuration() {
        let mut env_vars = HashMap::new();
        env_vars.insert("GITHUB_TOKEN".to_string(), "test-token".to_string());

        let backend = ClaudeBackend::new()
            .with_path("/usr/local/bin/claude")
            .with_model("claude-sonnet-4-20250514")
            .with_allowed_tools(vec!["bash".to_string(), "read".to_string()])
            .with_env_vars(env_vars.clone());

        assert_eq!(backend.claude_path, "/usr/local/bin/claude");
        assert_eq!(backend.model, Some("claude-sonnet-4-20250514".to_string()));
        assert_eq!(backend.allowed_tools.len(), 2);
        assert_eq!(backend.env_vars, env_vars);
    }

    #[tokio::test]
    async fn test_claude_spawn_invalid_workdir() {
        let backend = ClaudeBackend::new();
        let result = backend.spawn("test", Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
    }
}
