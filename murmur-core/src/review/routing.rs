//! Feedback routing module for routing review feedback back to coder agents
//!
//! This module provides functionality to take review feedback and generate prompts
//! for coder agents to address the identified issues. It supports different routing
//! strategies and handles iteration tracking.

use std::path::{Path, PathBuf};

use crate::agent::{AgentHandle, AgentSpawner, AgentType};
use crate::config::AgentConfig;
use crate::Result;

use super::{FeedbackItem, FeedbackSeverity, ReviewFeedback, ReviewType};

/// Strategy for routing feedback to the coder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    /// Route all feedback items to the coder
    #[default]
    All,
    /// Only route blocking issues (critical and major)
    BlockingOnly,
    /// Route blocking issues first, then others in subsequent iterations
    Prioritized,
}

/// Configuration for feedback routing
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// The routing strategy to use
    pub strategy: RoutingStrategy,
    /// Maximum number of fix iterations
    pub max_iterations: u32,
    /// Whether to include suggestions in the fix prompt
    pub include_suggestions: bool,
    /// Whether to include positive feedback (for context)
    pub include_positive: bool,
    /// Base agent configuration
    pub agent_config: AgentConfig,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::All,
            max_iterations: 3,
            include_suggestions: true,
            include_positive: false,
            agent_config: AgentConfig::default(),
        }
    }
}

impl RoutingConfig {
    /// Create a new routing configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the routing strategy
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set whether to include suggestions
    pub fn with_include_suggestions(mut self, include: bool) -> Self {
        self.include_suggestions = include;
        self
    }

    /// Set whether to include positive feedback
    pub fn with_include_positive(mut self, include: bool) -> Self {
        self.include_positive = include;
        self
    }

    /// Set the agent configuration
    pub fn with_agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = config;
        self
    }
}

/// A fix request to be sent to the coder agent
#[derive(Debug, Clone)]
pub struct FixRequest {
    /// The original task description
    pub task: String,
    /// The review type that generated this feedback
    pub review_type: ReviewType,
    /// The feedback to address
    pub feedback: ReviewFeedback,
    /// Current iteration number
    pub iteration: u32,
    /// Maximum iterations allowed
    pub max_iterations: u32,
    /// Working directory for the fix
    pub workdir: PathBuf,
    /// Files that need to be fixed
    pub files: Vec<String>,
    /// The routing configuration
    pub config: RoutingConfig,
}

impl FixRequest {
    /// Create a new fix request
    pub fn new(
        task: impl Into<String>,
        review_type: ReviewType,
        feedback: ReviewFeedback,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            task: task.into(),
            review_type,
            feedback,
            iteration: 0,
            max_iterations: 3,
            workdir: workdir.into(),
            files: Vec::new(),
            config: RoutingConfig::default(),
        }
    }

    /// Set the iteration number
    pub fn with_iteration(mut self, iteration: u32) -> Self {
        self.iteration = iteration;
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set the files to fix
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    /// Set the routing configuration
    pub fn with_config(mut self, config: RoutingConfig) -> Self {
        self.config = config;
        self
    }

    /// Check if we've exceeded max iterations
    pub fn exceeded_max_iterations(&self) -> bool {
        self.iteration >= self.max_iterations
    }

    /// Get the items to include based on routing strategy
    pub fn items_to_fix(&self) -> Vec<&FeedbackItem> {
        match self.config.strategy {
            RoutingStrategy::All => {
                let mut items: Vec<&FeedbackItem> = self
                    .feedback
                    .items
                    .iter()
                    .filter(|i| {
                        // Filter out positive feedback unless configured to include
                        if i.severity == FeedbackSeverity::Praise {
                            return self.config.include_positive;
                        }
                        // Filter out suggestions unless configured to include
                        if i.severity == FeedbackSeverity::Suggestion {
                            return self.config.include_suggestions;
                        }
                        true
                    })
                    .collect();
                // Sort by severity (critical first)
                items.sort_by_key(|i| match i.severity {
                    FeedbackSeverity::Critical => 0,
                    FeedbackSeverity::Major => 1,
                    FeedbackSeverity::Minor => 2,
                    FeedbackSeverity::Suggestion => 3,
                    FeedbackSeverity::Praise => 4,
                });
                items
            }
            RoutingStrategy::BlockingOnly => self.feedback.blocking_items(),
            RoutingStrategy::Prioritized => {
                // On first iteration, only blocking items
                // On subsequent iterations, include all
                if self.iteration == 0 {
                    self.feedback.blocking_items()
                } else {
                    self.feedback
                        .items
                        .iter()
                        .filter(|i| i.severity != FeedbackSeverity::Praise)
                        .collect()
                }
            }
        }
    }

    /// Generate the fix prompt for the coder agent
    pub fn to_prompt(&self) -> String {
        let mut prompt = String::new();

        // Header
        prompt.push_str("# Fix Request: Address Review Feedback\n\n");

        // Iteration info
        prompt.push_str(&format!(
            "## Fix Iteration: {}/{}\n\n",
            self.iteration + 1,
            self.max_iterations
        ));

        // Original task context
        if !self.task.is_empty() {
            prompt.push_str("## Original Task\n\n");
            prompt.push_str(&self.task);
            prompt.push_str("\n\n");
        }

        // Review context
        prompt.push_str(&format!(
            "## Review Type: {}\n\n",
            self.review_type.description()
        ));

        // Review decision
        prompt.push_str(&format!(
            "## Review Decision: {}\n\n",
            self.feedback.decision
        ));

        // Summary from reviewer
        if !self.feedback.summary.is_empty() {
            prompt.push_str("## Reviewer Summary\n\n");
            prompt.push_str(&self.feedback.summary);
            prompt.push_str("\n\n");
        }

        // Issues to fix
        let items = self.items_to_fix();
        if !items.is_empty() {
            prompt.push_str("## Issues to Address\n\n");
            prompt.push_str(
                "Please address the following issues from the review. \
                Fix them in order of severity (critical first):\n\n",
            );

            for (i, item) in items.iter().enumerate() {
                prompt.push_str(&format!("### Issue {} [{}]\n\n", i + 1, item.severity));

                if let Some(ref category) = item.category {
                    prompt.push_str(&format!("**Category:** {}\n\n", category));
                }

                prompt.push_str(&format!("**Problem:** {}\n\n", item.message));

                if let Some(ref file) = item.file {
                    if let Some(line) = item.line {
                        prompt.push_str(&format!("**Location:** `{}:{}`\n\n", file, line));
                    } else {
                        prompt.push_str(&format!("**File:** `{}`\n\n", file));
                    }
                }

                if let Some(ref suggestion) = item.suggestion {
                    prompt.push_str(&format!("**Suggested Fix:** {}\n\n", suggestion));
                }
            }
        }

        // Files to modify
        if !self.files.is_empty() {
            prompt.push_str("## Files to Modify\n\n");
            for file in &self.files {
                prompt.push_str(&format!("- `{}`\n", file));
            }
            prompt.push('\n');
        }

        // Instructions
        prompt.push_str("## Instructions\n\n");
        prompt.push_str("1. Read and understand each issue listed above\n");
        prompt.push_str("2. Fix the issues in order of severity (critical → major → minor)\n");
        prompt.push_str("3. For each fix, ensure you don't introduce new issues\n");
        prompt.push_str("4. After making changes, verify the code still compiles and tests pass\n");
        prompt.push_str(
            "5. If an issue is unclear, make a reasonable interpretation and document it\n\n",
        );

        // Warning about iterations
        if self.iteration + 1 >= self.max_iterations {
            prompt.push_str("⚠️ **WARNING:** This is the last iteration. ");
            prompt.push_str("Please ensure all critical and major issues are fully resolved.\n\n");
        }

        prompt
    }
}

/// Router for sending feedback to coder agents
#[derive(Debug, Clone)]
pub struct FeedbackRouter {
    /// The spawner for creating coder agent processes
    spawner: AgentSpawner,
    /// Configuration for routing
    config: RoutingConfig,
}

impl Default for FeedbackRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedbackRouter {
    /// Create a new feedback router with default configuration
    pub fn new() -> Self {
        Self {
            spawner: AgentSpawner::from_config(AgentConfig::default(), AgentType::Implement),
            config: RoutingConfig::default(),
        }
    }

    /// Create a router with custom configuration
    pub fn with_config(config: RoutingConfig) -> Self {
        Self {
            spawner: AgentSpawner::from_config(config.agent_config.clone(), AgentType::Implement),
            config,
        }
    }

    /// Create a router from an existing spawner
    pub fn from_spawner(spawner: AgentSpawner) -> Self {
        Self {
            spawner,
            config: RoutingConfig::default(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &RoutingConfig {
        &self.config
    }

    /// Route feedback to the coder agent
    ///
    /// This method takes a `FixRequest` and spawns a coder agent to address
    /// the feedback.
    ///
    /// # Arguments
    /// * `request` - The fix request containing feedback and configuration
    ///
    /// # Returns
    /// An `AgentHandle` that can be used to monitor and control the coder process
    pub async fn route(&self, request: FixRequest) -> Result<AgentHandle> {
        let prompt = request.to_prompt();
        let workdir = &request.workdir;

        self.spawner.spawn(prompt, workdir).await
    }

    /// Route feedback to the coder agent in a specific directory
    pub async fn route_in_dir(
        &self,
        request: FixRequest,
        workdir: impl AsRef<Path>,
    ) -> Result<AgentHandle> {
        let prompt = request.to_prompt();
        self.spawner.spawn(prompt, workdir).await
    }

    /// Check if feedback should be routed (i.e., there are issues to fix)
    pub fn should_route(&self, feedback: &ReviewFeedback) -> bool {
        match self.config.strategy {
            RoutingStrategy::All => !feedback.items.is_empty(),
            RoutingStrategy::BlockingOnly | RoutingStrategy::Prioritized => {
                feedback.has_blocking_issues()
            }
        }
    }
}

/// Convenience function to route feedback without creating a router instance
pub async fn route_feedback(request: FixRequest) -> Result<AgentHandle> {
    let router = FeedbackRouter::new();
    router.route(request).await
}

/// Convenience function to route feedback with custom configuration
pub async fn route_feedback_with_config(
    request: FixRequest,
    config: RoutingConfig,
) -> Result<AgentHandle> {
    let router = FeedbackRouter::with_config(config);
    router.route(request).await
}

/// Builder for creating fix requests with a fluent API
#[derive(Debug, Clone)]
pub struct FixRequestBuilder {
    request: FixRequest,
}

impl FixRequestBuilder {
    /// Create a new builder
    pub fn new(
        task: impl Into<String>,
        review_type: ReviewType,
        feedback: ReviewFeedback,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            request: FixRequest::new(task, review_type, feedback, workdir),
        }
    }

    /// Set the iteration number
    pub fn iteration(mut self, iteration: u32) -> Self {
        self.request.iteration = iteration;
        self
    }

    /// Set max iterations
    pub fn max_iterations(mut self, max: u32) -> Self {
        self.request.max_iterations = max;
        self
    }

    /// Set the files to fix
    pub fn files(mut self, files: Vec<String>) -> Self {
        self.request.files = files;
        self
    }

    /// Set the routing strategy
    pub fn strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.request.config.strategy = strategy;
        self
    }

    /// Include suggestions in the fix prompt
    pub fn include_suggestions(mut self, include: bool) -> Self {
        self.request.config.include_suggestions = include;
        self
    }

    /// Include positive feedback in the fix prompt
    pub fn include_positive(mut self, include: bool) -> Self {
        self.request.config.include_positive = include;
        self
    }

    /// Build the fix request
    pub fn build(self) -> FixRequest {
        self.request
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::{FeedbackItem, ReviewDecision};

    fn make_feedback_with_items() -> ReviewFeedback {
        let mut feedback =
            ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Needs fixes");
        feedback.add_item(
            FeedbackItem::new("Critical security issue")
                .with_severity(FeedbackSeverity::Critical)
                .with_category("security")
                .with_file("src/auth.rs")
                .with_line(42),
        );
        feedback.add_item(
            FeedbackItem::new("Major performance issue")
                .with_severity(FeedbackSeverity::Major)
                .with_file("src/db.rs"),
        );
        feedback.add_item(
            FeedbackItem::new("Minor style issue")
                .with_severity(FeedbackSeverity::Minor)
                .with_suggestion("Use snake_case"),
        );
        feedback.add_item(
            FeedbackItem::new("Consider using a cache").with_severity(FeedbackSeverity::Suggestion),
        );
        feedback.add_item(
            FeedbackItem::new("Good error handling").with_severity(FeedbackSeverity::Praise),
        );
        feedback
    }

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();
        assert_eq!(config.strategy, RoutingStrategy::All);
        assert_eq!(config.max_iterations, 3);
        assert!(config.include_suggestions);
        assert!(!config.include_positive);
    }

    #[test]
    fn test_routing_config_builder() {
        let config = RoutingConfig::new()
            .with_strategy(RoutingStrategy::BlockingOnly)
            .with_max_iterations(5)
            .with_include_suggestions(false)
            .with_include_positive(true);

        assert_eq!(config.strategy, RoutingStrategy::BlockingOnly);
        assert_eq!(config.max_iterations, 5);
        assert!(!config.include_suggestions);
        assert!(config.include_positive);
    }

    #[test]
    fn test_fix_request_new() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new(
            "Implement login",
            ReviewType::Code,
            feedback,
            "/tmp/project",
        );

        assert_eq!(request.task, "Implement login");
        assert_eq!(request.review_type, ReviewType::Code);
        assert_eq!(request.iteration, 0);
        assert_eq!(request.max_iterations, 3);
    }

    #[test]
    fn test_fix_request_builder() {
        let feedback = make_feedback_with_items();
        let request = FixRequestBuilder::new(
            "Implement feature",
            ReviewType::Code,
            feedback,
            "/tmp/project",
        )
        .iteration(1)
        .max_iterations(5)
        .files(vec!["src/main.rs".to_string()])
        .strategy(RoutingStrategy::BlockingOnly)
        .include_suggestions(false)
        .build();

        assert_eq!(request.iteration, 1);
        assert_eq!(request.max_iterations, 5);
        assert_eq!(request.files, vec!["src/main.rs".to_string()]);
        assert_eq!(request.config.strategy, RoutingStrategy::BlockingOnly);
        assert!(!request.config.include_suggestions);
    }

    #[test]
    fn test_fix_request_exceeded_max_iterations() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_iteration(2)
            .with_max_iterations(3);
        assert!(!request.exceeded_max_iterations());

        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_iteration(3)
            .with_max_iterations(3);
        assert!(request.exceeded_max_iterations());
    }

    #[test]
    fn test_items_to_fix_all_strategy() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_config(RoutingConfig::new().with_include_suggestions(true));

        let items = request.items_to_fix();
        // Should include critical, major, minor, suggestion but not praise
        assert_eq!(items.len(), 4);
        // Should be sorted by severity
        assert_eq!(items[0].severity, FeedbackSeverity::Critical);
        assert_eq!(items[1].severity, FeedbackSeverity::Major);
    }

    #[test]
    fn test_items_to_fix_blocking_only_strategy() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_config(RoutingConfig::new().with_strategy(RoutingStrategy::BlockingOnly));

        let items = request.items_to_fix();
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.is_blocking()));
    }

    #[test]
    fn test_items_to_fix_prioritized_strategy_first_iteration() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_iteration(0)
            .with_config(RoutingConfig::new().with_strategy(RoutingStrategy::Prioritized));

        let items = request.items_to_fix();
        // First iteration: only blocking
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.is_blocking()));
    }

    #[test]
    fn test_items_to_fix_prioritized_strategy_later_iteration() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_iteration(1)
            .with_config(RoutingConfig::new().with_strategy(RoutingStrategy::Prioritized));

        let items = request.items_to_fix();
        // Later iterations: all except praise
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn test_items_to_fix_with_positive_feedback() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp").with_config(
            RoutingConfig::new()
                .with_include_suggestions(true)
                .with_include_positive(true),
        );

        let items = request.items_to_fix();
        // Should include all 5 items
        assert_eq!(items.len(), 5);
    }

    #[test]
    fn test_fix_request_to_prompt_contains_header() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Implement login", ReviewType::Code, feedback, "/tmp");

        let prompt = request.to_prompt();
        assert!(prompt.contains("# Fix Request"));
        assert!(prompt.contains("Address Review Feedback"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_iteration() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_iteration(1)
            .with_max_iterations(3);

        let prompt = request.to_prompt();
        assert!(prompt.contains("Fix Iteration: 2/3"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_task() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new(
            "Implement the login feature",
            ReviewType::Code,
            feedback,
            "/tmp",
        );

        let prompt = request.to_prompt();
        assert!(prompt.contains("Original Task"));
        assert!(prompt.contains("Implement the login feature"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_review_type() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");

        let prompt = request.to_prompt();
        assert!(prompt.contains("Review Type: Code Review"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_decision() {
        let feedback =
            ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Needs work");
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");

        let prompt = request.to_prompt();
        assert!(prompt.contains("Review Decision: changes requested"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_summary() {
        let feedback =
            ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Please fix the bugs");
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");

        let prompt = request.to_prompt();
        assert!(prompt.contains("Reviewer Summary"));
        assert!(prompt.contains("Please fix the bugs"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_issues() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");

        let prompt = request.to_prompt();
        assert!(prompt.contains("Issues to Address"));
        assert!(prompt.contains("[critical]"));
        assert!(prompt.contains("Critical security issue"));
        assert!(prompt.contains("**Location:** `src/auth.rs:42`"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_files() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_files(vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]);

        let prompt = request.to_prompt();
        assert!(prompt.contains("Files to Modify"));
        assert!(prompt.contains("`src/main.rs`"));
        assert!(prompt.contains("`src/lib.rs`"));
    }

    #[test]
    fn test_fix_request_to_prompt_contains_instructions() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");

        let prompt = request.to_prompt();
        assert!(prompt.contains("## Instructions"));
        assert!(prompt.contains("Fix the issues in order of severity"));
    }

    #[test]
    fn test_fix_request_to_prompt_last_iteration_warning() {
        let feedback = ReviewFeedback::new();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp")
            .with_iteration(2)
            .with_max_iterations(3);

        let prompt = request.to_prompt();
        assert!(prompt.contains("WARNING"));
        assert!(prompt.contains("last iteration"));
    }

    #[test]
    fn test_feedback_router_new() {
        let router = FeedbackRouter::new();
        assert_eq!(router.config().strategy, RoutingStrategy::All);
    }

    #[test]
    fn test_feedback_router_with_config() {
        let config = RoutingConfig::new().with_strategy(RoutingStrategy::BlockingOnly);
        let router = FeedbackRouter::with_config(config);
        assert_eq!(router.config().strategy, RoutingStrategy::BlockingOnly);
    }

    #[test]
    fn test_feedback_router_should_route() {
        let router = FeedbackRouter::new();

        // Empty feedback
        let empty_feedback = ReviewFeedback::new();
        assert!(!router.should_route(&empty_feedback));

        // Feedback with items
        let feedback = make_feedback_with_items();
        assert!(router.should_route(&feedback));

        // Blocking only router
        let router = FeedbackRouter::with_config(
            RoutingConfig::new().with_strategy(RoutingStrategy::BlockingOnly),
        );

        // Feedback with only minor issues
        let mut minor_feedback = ReviewFeedback::new();
        minor_feedback.add_item(FeedbackItem::new("Minor").with_severity(FeedbackSeverity::Minor));
        assert!(!router.should_route(&minor_feedback));

        // Feedback with blocking issues
        assert!(router.should_route(&feedback));
    }

    #[tokio::test]
    async fn test_route_invalid_workdir() {
        let router = FeedbackRouter::new();
        let feedback = make_feedback_with_items();
        let request = FixRequest::new(
            "Task",
            ReviewType::Code,
            feedback,
            "/nonexistent/path/12345",
        );

        let result = router.route(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_in_dir_invalid_workdir() {
        let router = FeedbackRouter::new();
        let feedback = make_feedback_with_items();
        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");

        let result = router
            .route_in_dir(request, "/nonexistent/path/12345")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_feedback_function_invalid_workdir() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new(
            "Task",
            ReviewType::Code,
            feedback,
            "/nonexistent/path/12345",
        );

        let result = route_feedback(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_feedback_with_config_invalid_workdir() {
        let feedback = make_feedback_with_items();
        let request = FixRequest::new(
            "Task",
            ReviewType::Code,
            feedback,
            "/nonexistent/path/12345",
        );
        let config = RoutingConfig::new().with_strategy(RoutingStrategy::BlockingOnly);

        let result = route_feedback_with_config(request, config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_request_with_suggestion_from_item() {
        let mut feedback = ReviewFeedback::new();
        feedback.add_item(
            FeedbackItem::new("Use Result instead of unwrap")
                .with_severity(FeedbackSeverity::Major)
                .with_suggestion("Replace .unwrap() with .expect() or proper error handling"),
        );

        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");
        let prompt = request.to_prompt();

        assert!(prompt.contains("Suggested Fix:"));
        assert!(prompt.contains("Replace .unwrap()"));
    }

    #[test]
    fn test_fix_request_with_category() {
        let mut feedback = ReviewFeedback::new();
        feedback.add_item(
            FeedbackItem::new("SQL injection vulnerability")
                .with_severity(FeedbackSeverity::Critical)
                .with_category("security"),
        );

        let request = FixRequest::new("Task", ReviewType::Code, feedback, "/tmp");
        let prompt = request.to_prompt();

        assert!(prompt.contains("**Category:** security"));
    }
}
