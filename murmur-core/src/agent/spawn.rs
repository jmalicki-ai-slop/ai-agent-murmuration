//! Agent spawning logic for Claude Code subprocess management

use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};

use crate::{Error, Result};

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
    /// Get the prompt this agent is working on
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Get the working directory
    pub fn workdir(&self) -> &str {
        &self.workdir
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
#[derive(Debug, Default)]
pub struct AgentSpawner {
    /// Path to the claude executable (defaults to "claude" in PATH)
    claude_path: Option<String>,
}

impl AgentSpawner {
    /// Create a new agent spawner with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom path to the claude executable
    pub fn with_claude_path(mut self, path: impl Into<String>) -> Self {
        self.claude_path = Some(path.into());
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
    pub async fn spawn(&self, prompt: impl Into<String>, workdir: impl AsRef<Path>) -> Result<AgentHandle> {
        let prompt = prompt.into();
        let workdir_path = workdir.as_ref();
        let workdir_str = workdir_path
            .to_str()
            .ok_or_else(|| Error::Agent("Invalid working directory path".to_string()))?
            .to_string();

        // Verify working directory exists
        if !workdir_path.exists() {
            return Err(Error::Agent(format!(
                "Working directory does not exist: {}",
                workdir_str
            )));
        }

        let claude_cmd = self.claude_path.as_deref().unwrap_or("claude");

        let child = Command::new(claude_cmd)
            .arg("--print")
            .arg("--output-format")
            .arg("stream-json")
            .arg(&prompt)
            .current_dir(workdir_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Error::Agent(format!(
                        "Claude executable not found at '{}'. Is Claude Code installed?",
                        claude_cmd
                    ))
                } else {
                    Error::Io(e)
                }
            })?;

        Ok(AgentHandle {
            child,
            prompt,
            workdir: workdir_str,
        })
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
