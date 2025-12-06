//! Cursor backend implementation

use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::{Error, Result};

use super::super::spawn::AgentHandle;
use super::Backend;

/// Cursor backend implementation
#[derive(Debug, Clone)]
pub struct CursorBackend {
    cursor_path: String,
}

impl CursorBackend {
    /// Create a new Cursor backend with default settings
    pub fn new() -> Self {
        Self {
            cursor_path: "cursor".to_string(),
        }
    }

    /// Create a Cursor backend with custom path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.cursor_path = path.into();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_backend_name() {
        let backend = CursorBackend::new();
        assert_eq!(backend.name(), "cursor");
    }

    #[test]
    fn test_cursor_backend_builder() {
        let backend = CursorBackend::new().with_path("/custom/cursor");
        assert_eq!(backend.cursor_path, "/custom/cursor");
    }

    #[tokio::test]
    async fn test_cursor_spawn_invalid_workdir() {
        let backend = CursorBackend::new();
        let result = backend.spawn("test", Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
    }
}
