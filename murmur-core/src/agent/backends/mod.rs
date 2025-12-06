//! Backend abstraction for AI coding agents

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use tokio::process::Command;

use crate::config::Backend as BackendType;
use crate::Result;

use super::spawn::AgentHandle;

mod claude;
mod cursor;

pub use claude::ClaudeBackend;
pub use cursor::CursorBackend;

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
}
