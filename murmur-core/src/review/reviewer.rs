//! Reviewer agent invocation module
//!
//! This module provides functionality to invoke the reviewer agent with a structured
//! review request. It takes a `ReviewRequest` and spawns a reviewer agent with the
//! appropriate prompt and configuration.

use std::path::Path;

use crate::agent::{AgentHandle, AgentSpawner, AgentType};
use crate::config::AgentConfig;
use crate::Result;

use super::ReviewRequest;

/// Configuration for the reviewer agent
#[derive(Debug, Clone, Default)]
pub struct ReviewerConfig {
    /// Base agent configuration
    pub agent_config: AgentConfig,
    /// Whether to use verbose output
    pub verbose: bool,
}

impl ReviewerConfig {
    /// Create a new reviewer configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base agent configuration
    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = config;
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// Reviewer agent for executing code reviews
///
/// The Reviewer takes a `ReviewRequest` and spawns a review agent with the appropriate
/// prompt generated from the request context.
#[derive(Debug, Clone)]
pub struct Reviewer {
    /// The spawner for creating reviewer agent processes
    spawner: AgentSpawner,
    /// Configuration for the reviewer
    config: ReviewerConfig,
}

impl Default for Reviewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Reviewer {
    /// Create a new reviewer with default configuration
    pub fn new() -> Self {
        Self {
            spawner: AgentSpawner::from_config(AgentConfig::default(), AgentType::Review),
            config: ReviewerConfig::default(),
        }
    }

    /// Create a reviewer with custom configuration
    pub fn with_config(config: ReviewerConfig) -> Self {
        Self {
            spawner: AgentSpawner::from_config(config.agent_config.clone(), AgentType::Review),
            config,
        }
    }

    /// Create a reviewer from an existing spawner
    pub fn from_spawner(spawner: AgentSpawner) -> Self {
        Self {
            spawner,
            config: ReviewerConfig::default(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &ReviewerConfig {
        &self.config
    }

    /// Invoke the reviewer agent with a review request
    ///
    /// This method takes a `ReviewRequest`, generates the appropriate prompt using
    /// `ReviewRequest::to_prompt()`, and spawns a reviewer agent in the specified
    /// working directory.
    ///
    /// # Arguments
    /// * `request` - The review request containing context and configuration
    ///
    /// # Returns
    /// An `AgentHandle` that can be used to monitor and control the reviewer process
    ///
    /// # Example
    /// ```ignore
    /// use murmur_core::review::{ReviewRequest, ReviewType, Reviewer};
    ///
    /// let request = ReviewRequest::code_review(
    ///     "Implement login feature",
    ///     vec!["src/auth.rs".to_string()],
    ///     "+ fn login() {}",
    ///     "/path/to/project",
    /// );
    ///
    /// let reviewer = Reviewer::new();
    /// let handle = reviewer.invoke(request).await?;
    /// ```
    pub async fn invoke(&self, request: ReviewRequest) -> Result<AgentHandle> {
        let prompt = request.to_prompt();
        let workdir = &request.workdir;

        self.spawner.spawn(prompt, workdir).await
    }

    /// Invoke the reviewer agent with a custom working directory
    ///
    /// This is useful when you want to review code in a different directory
    /// than the one specified in the request.
    pub async fn invoke_in_dir(
        &self,
        request: ReviewRequest,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = request.to_prompt();
        self.spawner.spawn(prompt, workdir).await
    }
}

/// Convenience function to invoke a review without creating a Reviewer instance
///
/// This is a shorthand for creating a Reviewer with default configuration and invoking it.
///
/// # Example
/// ```ignore
/// use murmur_core::review::{ReviewRequest, ReviewType, invoke_review};
///
/// let request = ReviewRequest::spec_review(
///     "Design user authentication",
///     "# Auth Spec\n\nUsers should authenticate with email/password",
///     "/path/to/project",
/// );
///
/// let handle = invoke_review(request).await?;
/// ```
pub async fn invoke_review(request: ReviewRequest) -> Result<AgentHandle> {
    let reviewer = Reviewer::new();
    reviewer.invoke(request).await
}

/// Convenience function to invoke a review with custom configuration
pub async fn invoke_review_with_config(
    request: ReviewRequest,
    config: ReviewerConfig,
) -> Result<AgentHandle> {
    let reviewer = Reviewer::with_config(config);
    reviewer.invoke(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::ReviewType;
    use std::env;

    #[test]
    fn test_reviewer_config_default() {
        let config = ReviewerConfig::default();
        assert!(!config.verbose);
    }

    #[test]
    fn test_reviewer_config_builder() {
        let config = ReviewerConfig::new()
            .with_verbose(true)
            .with_agent_config(AgentConfig::default());

        assert!(config.verbose);
    }

    #[test]
    fn test_reviewer_new() {
        let reviewer = Reviewer::new();
        assert!(!reviewer.config().verbose);
    }

    #[test]
    fn test_reviewer_with_config() {
        let config = ReviewerConfig::new().with_verbose(true);
        let reviewer = Reviewer::with_config(config);
        assert!(reviewer.config().verbose);
    }

    #[test]
    fn test_reviewer_from_spawner() {
        let spawner = AgentSpawner::from_config(AgentConfig::default(), AgentType::Review);
        let reviewer = Reviewer::from_spawner(spawner);
        assert!(!reviewer.config().verbose);
    }

    #[tokio::test]
    async fn test_invoke_invalid_workdir() {
        let reviewer = Reviewer::new();
        let request = ReviewRequest::code_review(
            "Test review",
            vec!["src/main.rs".to_string()],
            "+ test diff",
            "/nonexistent/path/12345",
        );

        let result = reviewer.invoke(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invoke_generates_correct_prompt() {
        // We can't easily test the full invocation without a real claude binary,
        // but we can verify that the request generates a valid prompt
        let request = ReviewRequest::code_review(
            "Implement login feature",
            vec!["src/auth.rs".to_string()],
            "+ fn login() { }",
            env::current_dir().unwrap(),
        );

        let prompt = request.to_prompt();
        assert!(prompt.contains("Code Review"));
        assert!(prompt.contains("Implement login feature"));
        assert!(prompt.contains("src/auth.rs"));
        assert!(prompt.contains("+ fn login()"));
    }

    #[tokio::test]
    async fn test_invoke_in_dir_invalid_workdir() {
        let reviewer = Reviewer::new();
        let request = ReviewRequest::code_review(
            "Test review",
            vec!["src/main.rs".to_string()],
            "+ test diff",
            "/some/path", // This path doesn't matter, we override it
        );

        let result = reviewer
            .invoke_in_dir(request, "/nonexistent/path/12345")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invoke_review_function_invalid_workdir() {
        let request = ReviewRequest::spec_review(
            "Design feature",
            "# Spec\n\nFeature description",
            "/nonexistent/path/12345",
        );

        let result = invoke_review(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invoke_review_with_config_invalid_workdir() {
        let request = ReviewRequest::test_review(
            "Test feature",
            vec!["tests/test.rs".to_string()],
            "+ #[test]",
            "/nonexistent/path/12345",
        );

        let config = ReviewerConfig::new().with_verbose(true);
        let result = invoke_review_with_config(request, config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_review_request_prompt_for_spec_review() {
        let request = ReviewRequest::spec_review(
            "Design authentication system",
            "# Authentication Spec\n\n- Users login with email\n- Password must be 8+ chars",
            "/tmp/project",
        );

        let prompt = request.to_prompt();
        assert!(prompt.contains("Specification Review"));
        assert!(prompt.contains("Gate:"));
        assert!(prompt.contains("Design authentication system"));
        assert!(prompt.contains("Authentication Spec"));
        assert!(prompt.contains("Completeness of requirements"));
    }

    #[test]
    fn test_review_request_prompt_for_test_review() {
        let request = ReviewRequest::test_review(
            "Test login functionality",
            vec!["tests/auth_test.rs".to_string()],
            "+ #[test]\n+ fn test_login_success() {\n+     assert!(true);\n+ }",
            "/tmp/project",
        );

        let prompt = request.to_prompt();
        assert!(prompt.contains("Test Review"));
        assert!(prompt.contains("Test login functionality"));
        assert!(prompt.contains("tests/auth_test.rs"));
        assert!(prompt.contains("test_login_success"));
        assert!(prompt.contains("Test coverage"));
    }

    #[test]
    fn test_review_request_prompt_for_final_review() {
        let request = ReviewRequest::final_review(
            "Complete authentication feature",
            vec!["src/auth.rs".to_string(), "tests/auth_test.rs".to_string()],
            "+ complete implementation",
            "/tmp/project",
        );

        let prompt = request.to_prompt();
        assert!(prompt.contains("Final Review"));
        assert!(prompt.contains("Complete authentication feature"));
        assert!(prompt.contains("Production readiness"));
    }

    #[test]
    fn test_review_request_prompt_with_previous_feedback() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Fix issues from review")
            .diff("+ fixed code")
            .impl_files(vec!["src/fix.rs".to_string()])
            .previous_feedback("- Missing error handling in login()\n- Add input validation")
            .iteration(1)
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Previous Review Feedback"));
        assert!(prompt.contains("Missing error handling"));
        assert!(prompt.contains("Add input validation"));
        assert!(prompt.contains("Review Iteration: 2/3"));
    }
}
