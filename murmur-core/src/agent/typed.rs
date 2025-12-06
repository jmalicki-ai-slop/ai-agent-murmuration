//! Typed agents for Murmuration
//!
//! This module provides concrete agent implementations that use type-specific
//! prompts and behaviors.

use crate::agent::{AgentHandle, AgentSpawner, AgentType, PromptBuilder};
use crate::config::AgentConfig;
use crate::{Error, Result};
use std::path::Path;

/// A typed agent with specific prompts and behaviors
#[derive(Debug, Clone)]
pub struct TypedAgent {
    /// The type of this agent
    agent_type: AgentType,
    /// The spawner used to create processes
    spawner: AgentSpawner,
}

impl TypedAgent {
    /// Create a new typed agent
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            spawner: AgentSpawner::from_config(AgentConfig::default(), agent_type),
        }
    }

    /// Create a typed agent with custom configuration
    pub fn with_config(agent_type: AgentType, config: AgentConfig) -> Self {
        Self {
            agent_type,
            spawner: AgentSpawner::from_config(config, agent_type),
        }
    }

    /// Get the agent type
    pub fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    /// Build and spawn the agent with the given task
    pub async fn spawn_with_task(
        &self,
        task: impl Into<String>,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(self.agent_type).task(task).build();

        self.spawner.spawn(prompt, workdir).await
    }

    /// Build and spawn with files context
    pub async fn spawn_with_files(
        &self,
        task: impl Into<String>,
        files: &[String],
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(self.agent_type)
            .task(task)
            .files(files)
            .build();

        self.spawner.spawn(prompt, workdir).await
    }
}

/// Implement agent - writes code to implement features
#[derive(Debug, Clone)]
pub struct ImplementAgent {
    inner: TypedAgent,
}

impl Default for ImplementAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ImplementAgent {
    /// Create a new implement agent
    pub fn new() -> Self {
        Self {
            inner: TypedAgent::new(AgentType::Implement),
        }
    }

    /// Create with custom config
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            inner: TypedAgent::with_config(AgentType::Implement, config),
        }
    }

    /// Spawn to implement a task
    pub async fn implement(
        &self,
        task: impl Into<String>,
        files: &[String],
        dependencies: &[String],
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(AgentType::Implement)
            .task(task)
            .files(files)
            .dependencies(dependencies)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }
}

/// Test agent - writes and runs tests
#[derive(Debug, Clone)]
pub struct TestAgent {
    inner: TypedAgent,
}

impl Default for TestAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl TestAgent {
    /// Create a new test agent
    pub fn new() -> Self {
        Self {
            inner: TypedAgent::new(AgentType::Test),
        }
    }

    /// Create with custom config
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            inner: TypedAgent::with_config(AgentType::Test, config),
        }
    }

    /// Spawn to test a task
    pub async fn test(
        &self,
        task: impl Into<String>,
        files: &[String],
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(AgentType::Test)
            .task(task)
            .files(files)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }

    /// Spawn to write tests first (RED phase in TDD)
    pub async fn write_failing_test(
        &self,
        behavior: impl Into<String>,
        files: &[String],
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let task = format!(
            "Write a failing test that describes the following behavior: {}. \
             The test should fail because the behavior is not yet implemented.",
            behavior.into()
        );

        let prompt = PromptBuilder::new(AgentType::Test)
            .task(task)
            .files(files)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }

    /// Spawn to verify tests pass (GREEN phase in TDD)
    pub async fn verify_tests_pass(
        &self,
        files: &[String],
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(AgentType::Test)
            .task("Run the test suite and verify all tests pass. Report any failures.")
            .files(files)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }
}

/// Review agent - reviews code changes
#[derive(Debug, Clone)]
pub struct ReviewAgent {
    inner: TypedAgent,
}

impl Default for ReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewAgent {
    /// Create a new review agent
    pub fn new() -> Self {
        Self {
            inner: TypedAgent::new(AgentType::Review),
        }
    }

    /// Create with custom config
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            inner: TypedAgent::with_config(AgentType::Review, config),
        }
    }

    /// Spawn to review a diff
    pub async fn review(
        &self,
        task: impl Into<String>,
        diff: impl Into<String>,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(AgentType::Review)
            .task(task)
            .diff(diff)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }

    /// Spawn to review changes from git diff
    pub async fn review_changes(
        &self,
        task: impl Into<String>,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        // Get the diff from git
        let workdir_path = workdir.as_ref();
        let diff = get_git_diff(workdir_path)?;

        let prompt = PromptBuilder::new(AgentType::Review)
            .task(task)
            .diff(diff)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }
}

/// Coordinator agent - orchestrates other agents
#[derive(Debug, Clone)]
pub struct CoordinatorAgent {
    inner: TypedAgent,
}

impl Default for CoordinatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinatorAgent {
    /// Create a new coordinator agent
    pub fn new() -> Self {
        Self {
            inner: TypedAgent::new(AgentType::Coordinator),
        }
    }

    /// Create with custom config
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            inner: TypedAgent::with_config(AgentType::Coordinator, config),
        }
    }

    /// Spawn to coordinate a task
    pub async fn coordinate(
        &self,
        task: impl Into<String>,
        repo: impl Into<String>,
        main_branch: impl Into<String>,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = PromptBuilder::new(AgentType::Coordinator)
            .task(task)
            .repo(repo)
            .main_branch(main_branch)
            .build();

        self.inner.spawner.spawn(prompt, workdir).await
    }
}

/// Get git diff from a working directory
fn get_git_diff(workdir: &Path) -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("diff")
        .arg("HEAD")
        .current_dir(workdir)
        .output()
        .map_err(Error::Io)?;

    if !output.status.success() {
        return Err(Error::Agent(format!(
            "Failed to get git diff: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Factory for creating typed agents
#[derive(Debug, Clone, Default)]
pub struct AgentFactory {
    config: Option<AgentConfig>,
}

impl AgentFactory {
    /// Create a new agent factory
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a factory with custom config
    pub fn with_config(config: AgentConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    /// Create an implement agent
    pub fn implement(&self) -> ImplementAgent {
        match &self.config {
            Some(config) => ImplementAgent::with_config(config.clone()),
            None => ImplementAgent::new(),
        }
    }

    /// Create a test agent
    pub fn test(&self) -> TestAgent {
        match &self.config {
            Some(config) => TestAgent::with_config(config.clone()),
            None => TestAgent::new(),
        }
    }

    /// Create a review agent
    pub fn review(&self) -> ReviewAgent {
        match &self.config {
            Some(config) => ReviewAgent::with_config(config.clone()),
            None => ReviewAgent::new(),
        }
    }

    /// Create a coordinator agent
    pub fn coordinator(&self) -> CoordinatorAgent {
        match &self.config {
            Some(config) => CoordinatorAgent::with_config(config.clone()),
            None => CoordinatorAgent::new(),
        }
    }

    /// Create a typed agent by type
    pub fn create(&self, agent_type: AgentType) -> TypedAgent {
        match &self.config {
            Some(config) => TypedAgent::with_config(agent_type, config.clone()),
            None => TypedAgent::new(agent_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implement_agent_creation() {
        let agent = ImplementAgent::new();
        assert_eq!(agent.inner.agent_type, AgentType::Implement);
    }

    #[test]
    fn test_test_agent_creation() {
        let agent = TestAgent::new();
        assert_eq!(agent.inner.agent_type, AgentType::Test);
    }

    #[test]
    fn test_review_agent_creation() {
        let agent = ReviewAgent::new();
        assert_eq!(agent.inner.agent_type, AgentType::Review);
    }

    #[test]
    fn test_coordinator_agent_creation() {
        let agent = CoordinatorAgent::new();
        assert_eq!(agent.inner.agent_type, AgentType::Coordinator);
    }

    #[test]
    fn test_factory_creates_all_types() {
        let factory = AgentFactory::new();

        assert_eq!(factory.implement().inner.agent_type, AgentType::Implement);
        assert_eq!(factory.test().inner.agent_type, AgentType::Test);
        assert_eq!(factory.review().inner.agent_type, AgentType::Review);
        assert_eq!(
            factory.coordinator().inner.agent_type,
            AgentType::Coordinator
        );
    }

    #[test]
    fn test_factory_create_by_type() {
        let factory = AgentFactory::new();

        assert_eq!(
            factory.create(AgentType::Implement).agent_type,
            AgentType::Implement
        );
        assert_eq!(factory.create(AgentType::Test).agent_type, AgentType::Test);
    }

    #[test]
    fn test_typed_agent_default() {
        let agent = TypedAgent::new(AgentType::Review);
        assert_eq!(agent.agent_type(), AgentType::Review);
    }
}
