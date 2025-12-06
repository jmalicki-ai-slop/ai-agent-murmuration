//! Backend abstraction for AI coding agents

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::Backend as BackendType;
use crate::{Error, Result};

use super::spawn::AgentHandle;

/// Trait for AI coding backends
#[async_trait]
pub trait Backend: Send + Sync {
    /// Get the name of this backend
    fn name(&self) -> &'static str;

    /// Build the command to spawn this backend
    fn build_command(&self, workdir: &Path) -> Command;

    /// Spawn an agent with a prompt
    async fn spawn(&self, prompt: &str, workdir: &Path) -> Result<AgentHandle>;

    /// Check if this backend is available on the system
    fn is_available(&self) -> bool;
}

/// Claude Code backend implementation
#[derive(Debug, Clone)]
pub struct ClaudeBackend {
    claude_path: String,
    model: Option<String>,
}

impl ClaudeBackend {
    /// Create a new Claude backend with default settings
    pub fn new() -> Self {
        Self {
            claude_path: "claude".to_string(),
            model: None,
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

/// Cursor backend implementation
#[derive(Debug, Clone)]
pub struct CursorBackend {
    cursor_path: String,
    model: Option<String>,
}

impl CursorBackend {
    /// Create a new Cursor backend with default settings
    pub fn new() -> Self {
        Self {
            cursor_path: "cursor-agent".to_string(),
            model: None,
        }
    }

    /// Create a Cursor backend with custom path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.cursor_path = path.into();
        self
    }

    /// Create a Cursor backend with a specific model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

impl Default for CursorBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Backend for CursorBackend {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn build_command(&self, workdir: &Path) -> Command {
        let mut cmd = Command::new(&self.cursor_path);
        cmd.arg("--print").arg("--output-format").arg("json");

        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
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
        cmd.arg("-p").arg(prompt);

        let child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Agent(format!(
                    "Cursor executable not found at '{}'. Is Cursor installed?",
                    self.cursor_path
                ))
            } else {
                Error::Io(e)
            }
        })?;

        Ok(AgentHandle::new(child, prompt.to_string(), workdir_str))
    }

    fn is_available(&self) -> bool {
        std::process::Command::new(&self.cursor_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}

/// Registry of available backends
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn Backend>>,
}

impl BackendRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            backends: HashMap::new(),
        }
    }

    /// Create a registry with default backends
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(ClaudeBackend::new()));
        registry.register(Box::new(CursorBackend::new()));
        registry
    }

    /// Register a backend
    pub fn register(&mut self, backend: Box<dyn Backend>) {
        self.backends.insert(backend.name().to_string(), backend);
    }

    /// Get a backend by name
    pub fn get(&self, name: &str) -> Option<&dyn Backend> {
        self.backends.get(name).map(|b| b.as_ref())
    }

    /// List all available backends (backends that are installed on the system)
    pub fn list_available(&self) -> Vec<&str> {
        self.backends
            .values()
            .filter(|b| b.is_available())
            .map(|b| b.name())
            .collect()
    }

    /// List all registered backends (whether available or not)
    pub fn list_registered(&self) -> Vec<&str> {
        self.backends.keys().map(|s| s.as_str()).collect()
    }

    /// Get a backend by type enum
    pub fn get_by_type(&self, backend_type: BackendType) -> Option<&dyn Backend> {
        match backend_type {
            BackendType::Claude => self.get("claude"),
            BackendType::Cursor => self.get("cursor"),
        }
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::with_defaults()
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
    fn test_cursor_backend_name() {
        let backend = CursorBackend::new();
        assert_eq!(backend.name(), "cursor");
    }

    #[test]
    fn test_registry_register() {
        let mut registry = BackendRegistry::new();
        assert!(registry.get("claude").is_none());

        registry.register(Box::new(ClaudeBackend::new()));
        assert!(registry.get("claude").is_some());
    }

    #[test]
    fn test_registry_get() {
        let registry = BackendRegistry::with_defaults();
        let backend = registry.get("claude");
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().name(), "claude");
    }

    #[test]
    fn test_registry_list_registered() {
        let registry = BackendRegistry::with_defaults();
        let registered = registry.list_registered();
        assert!(registered.contains(&"claude"));
        assert!(registered.contains(&"cursor"));
        assert_eq!(registered.len(), 2);
    }

    #[test]
    fn test_registry_get_by_type() {
        let registry = BackendRegistry::with_defaults();
        let backend = registry.get_by_type(BackendType::Claude);
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().name(), "claude");

        let backend = registry.get_by_type(BackendType::Cursor);
        assert!(backend.is_some());
        assert_eq!(backend.unwrap().name(), "cursor");
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
    fn test_cursor_backend_builder() {
        let backend = CursorBackend::new()
            .with_path("/custom/cursor-agent")
            .with_model("gpt-5");

        assert_eq!(backend.cursor_path, "/custom/cursor-agent");
        assert_eq!(backend.model, Some("gpt-5".to_string()));
    }

    #[tokio::test]
    async fn test_claude_spawn_invalid_workdir() {
        let backend = ClaudeBackend::new();
        let result = backend.spawn("test", Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cursor_spawn_invalid_workdir() {
        let backend = CursorBackend::new();
        let result = backend.spawn("test", Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
    }
}
