//! Agent spawning logic for Claude Code subprocess management

use std::path::Path;
use tokio::process::Child;

use crate::config::AgentConfig;
use crate::{Error, Result};

use super::backends::{Backend, ClaudeBackend};

/// Handle to a running Claude Code agent process
pub struct AgentHandle {
    /// The child process (not Debug, so we skip it)
    child: Child,
    /// The prompt that was given to the agent
    prompt: String,
    /// Working directory for the agent
    workdir: String,
}

impl std::fmt::Debug for AgentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentHandle")
            .field("prompt", &self.prompt)
            .field("workdir", &self.workdir)
            .field("child", &"<Child>")
            .finish()
    }
}

impl AgentHandle {
    /// Create a new agent handle
    pub(crate) fn new(child: Child, prompt: String, workdir: String) -> Self {
        Self {
            child,
            prompt,
            workdir,
        }
    }

    /// Get the prompt this agent is working on
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Get the working directory
    pub fn workdir(&self) -> &str {
        &self.workdir
    }

    /// Get the process ID if available
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    /// Get mutable access to the child process for output streaming
    pub fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    /// Wait for the process to complete and return the exit status
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        self.child.wait().await.map_err(Error::Io)
    }

    /// Kill the agent process
    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await.map_err(Error::Io)
    }
}

/// Spawner for Claude Code agent processes
#[derive(Clone)]
pub struct AgentSpawner {
    /// Backend implementation
    backend: ClaudeBackend,
}

impl AgentSpawner {
    /// Create a new agent spawner with default settings
    pub fn new() -> Self {
        Self {
            backend: ClaudeBackend::new(),
        }
    }

    /// Create an agent spawner from configuration
    pub fn from_config(config: AgentConfig) -> Self {
        // Convert config to backend
        let backend = ClaudeBackend::new().with_path(config.claude_path);

        let backend = if let Some(model) = config.model {
            backend.with_model(model)
        } else {
            backend
        };

        Self { backend }
    }

    /// Set a custom path to the claude executable
    pub fn with_claude_path(mut self, path: impl Into<String>) -> Self {
        self.backend = self.backend.with_path(path);
        self
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.backend = self.backend.with_model(model);
        self
    }

    /// Add an environment variable to pass to the spawned agent
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.backend = self.backend.with_env(key, value);
        self
    }

    /// Spawn a new Claude Code agent with the given prompt
    ///
    /// # Arguments
    /// * `prompt` - The task prompt for the agent
    /// * `workdir` - Working directory for the agent
    ///
    /// # Returns
    /// An `AgentHandle` that can be used to monitor and control the process
    pub async fn spawn(
        &self,
        prompt: impl Into<String>,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        self.backend.spawn(&prompt.into(), workdir.as_ref()).await
    }
}

impl Default for AgentSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AgentSpawner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentSpawner")
            .field("backend", &"ClaudeBackend")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_spawn_invalid_workdir() {
        let spawner = AgentSpawner::new();
        let result = spawner.spawn("test", "/nonexistent/path/12345").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Agent(_)));
    }

    #[tokio::test]
    async fn test_spawn_with_custom_claude_path() {
        let spawner = AgentSpawner::new().with_claude_path("/usr/bin/nonexistent-claude-binary");
        let result = spawner.spawn("test", env::current_dir().unwrap()).await;
        assert!(result.is_err());
        // Should fail to find the executable
    }
}
