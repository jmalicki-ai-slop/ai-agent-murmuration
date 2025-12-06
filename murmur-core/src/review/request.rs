//! Review request generation for the reviewer agent
//!
//! This module provides structured review requests that can be sent to the reviewer agent.
//! Review requests include the type of review, the context (diff, files, spec), and
//! any additional metadata needed for the review.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The type of review being requested
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewType {
    /// Review a specification document before writing tests
    Spec,
    /// Review test cases before running them (VerifyRed)
    Test,
    /// Review implementation code before verifying tests pass
    Code,
    /// Final review before completing the workflow
    Final,
}

impl ReviewType {
    /// Get a human-readable description of the review type
    pub fn description(&self) -> &'static str {
        match self {
            ReviewType::Spec => "Specification Review",
            ReviewType::Test => "Test Review",
            ReviewType::Code => "Code Review",
            ReviewType::Final => "Final Review",
        }
    }

    /// Get the review focus areas for this type
    pub fn focus_areas(&self) -> &'static [&'static str] {
        match self {
            ReviewType::Spec => &[
                "Completeness of requirements",
                "Clarity of expected behavior",
                "Edge cases covered",
                "Testability of requirements",
            ],
            ReviewType::Test => &[
                "Test coverage of requirements",
                "Test correctness",
                "Edge case handling",
                "Test maintainability",
            ],
            ReviewType::Code => &[
                "Correctness of implementation",
                "Code quality and readability",
                "Security considerations",
                "Performance implications",
            ],
            ReviewType::Final => &[
                "Overall implementation quality",
                "Test coverage adequacy",
                "Documentation completeness",
                "Production readiness",
            ],
        }
    }

    /// Get the corresponding TDD phase gate description
    pub fn gate_description(&self) -> &'static str {
        match self {
            ReviewType::Spec => "Gate: Specification must be approved before writing tests",
            ReviewType::Test => "Gate: Tests must be approved before running VerifyRed",
            ReviewType::Code => "Gate: Code must be approved before VerifyGreen",
            ReviewType::Final => "Gate: Final approval required before marking complete",
        }
    }
}

impl std::fmt::Display for ReviewType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Additional context for the review
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewContext {
    /// The issue or task being worked on
    pub task: String,
    /// The diff to review (if applicable)
    pub diff: Option<String>,
    /// Files involved in the review
    pub files: Vec<String>,
    /// Specification content (for spec reviews or as reference)
    pub spec_content: Option<String>,
    /// Test file paths
    pub test_files: Vec<String>,
    /// Implementation file paths
    pub impl_files: Vec<String>,
    /// Previous review feedback (for iteration)
    pub previous_feedback: Option<String>,
    /// Iteration number (how many review rounds)
    pub iteration: u32,
}

impl ReviewContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the task description
    pub fn with_task(mut self, task: impl Into<String>) -> Self {
        self.task = task.into();
        self
    }

    /// Set the diff to review
    pub fn with_diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = Some(diff.into());
        self
    }

    /// Set the files to review
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    /// Set the specification content
    pub fn with_spec(mut self, spec: impl Into<String>) -> Self {
        self.spec_content = Some(spec.into());
        self
    }

    /// Set the test files
    pub fn with_test_files(mut self, files: Vec<String>) -> Self {
        self.test_files = files;
        self
    }

    /// Set the implementation files
    pub fn with_impl_files(mut self, files: Vec<String>) -> Self {
        self.impl_files = files;
        self
    }

    /// Set previous feedback for iteration
    pub fn with_previous_feedback(mut self, feedback: impl Into<String>) -> Self {
        self.previous_feedback = Some(feedback.into());
        self
    }

    /// Set the iteration number
    pub fn with_iteration(mut self, iteration: u32) -> Self {
        self.iteration = iteration;
        self
    }
}

/// A structured review request for the reviewer agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    /// The type of review being requested
    pub review_type: ReviewType,
    /// The working directory for the review
    pub workdir: PathBuf,
    /// Context for the review
    pub context: ReviewContext,
    /// Maximum iterations allowed
    pub max_iterations: u32,
}

impl ReviewRequest {
    /// Create a new review request
    pub fn new(review_type: ReviewType, workdir: impl Into<PathBuf>) -> Self {
        Self {
            review_type,
            workdir: workdir.into(),
            context: ReviewContext::default(),
            max_iterations: 3,
        }
    }

    /// Create a new review request with a builder
    pub fn builder(review_type: ReviewType, workdir: impl Into<PathBuf>) -> ReviewRequestBuilder {
        ReviewRequestBuilder::new(review_type, workdir)
    }

    /// Set the context
    pub fn with_context(mut self, context: ReviewContext) -> Self {
        self.context = context;
        self
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Check if this is a re-review (iteration > 0)
    pub fn is_rereview(&self) -> bool {
        self.context.iteration > 0
    }

    /// Check if we've exceeded max iterations
    pub fn exceeded_max_iterations(&self) -> bool {
        self.context.iteration >= self.max_iterations
    }

    /// Generate the review prompt for the reviewer agent
    pub fn to_prompt(&self) -> String {
        let mut prompt = String::new();

        // Header
        prompt.push_str(&format!("# {} Request\n\n", self.review_type));
        prompt.push_str(&format!("{}\n\n", self.review_type.gate_description()));

        // Task context
        if !self.context.task.is_empty() {
            prompt.push_str("## Task\n\n");
            prompt.push_str(&self.context.task);
            prompt.push_str("\n\n");
        }

        // Iteration info
        if self.context.iteration > 0 {
            prompt.push_str(&format!(
                "## Review Iteration: {}/{}\n\n",
                self.context.iteration + 1,
                self.max_iterations
            ));
        }

        // Previous feedback (for re-reviews)
        if let Some(ref feedback) = self.context.previous_feedback {
            prompt.push_str("## Previous Review Feedback\n\n");
            prompt.push_str("The following issues were raised in the previous review:\n\n");
            prompt.push_str(feedback);
            prompt.push_str("\n\n");
            prompt.push_str(
                "Please verify that these issues have been addressed in the current changes.\n\n",
            );
        }

        // Focus areas
        prompt.push_str("## Focus Areas\n\n");
        prompt.push_str(&format!(
            "For this {}, please focus on:\n\n",
            self.review_type.description().to_lowercase()
        ));
        for area in self.review_type.focus_areas() {
            prompt.push_str(&format!("- {}\n", area));
        }
        prompt.push('\n');

        // Specification content (for spec reviews or as reference)
        if let Some(ref spec) = self.context.spec_content {
            prompt.push_str("## Specification\n\n");
            prompt.push_str("```\n");
            prompt.push_str(spec);
            prompt.push_str("\n```\n\n");
        }

        // Files to review
        let all_files: Vec<&String> = match self.review_type {
            ReviewType::Spec => self.context.files.iter().collect(),
            ReviewType::Test => self.context.test_files.iter().collect(),
            ReviewType::Code => self.context.impl_files.iter().collect(),
            ReviewType::Final => self
                .context
                .files
                .iter()
                .chain(self.context.test_files.iter())
                .chain(self.context.impl_files.iter())
                .collect(),
        };

        if !all_files.is_empty() {
            prompt.push_str("## Files to Review\n\n");
            for file in all_files {
                prompt.push_str(&format!("- `{}`\n", file));
            }
            prompt.push('\n');
        }

        // Diff
        if let Some(ref diff) = self.context.diff {
            prompt.push_str("## Changes to Review\n\n");
            prompt.push_str("```diff\n");
            prompt.push_str(diff);
            prompt.push_str("\n```\n\n");
        }

        // Expected output format
        prompt.push_str("## Expected Output Format\n\n");
        prompt.push_str("Please provide your review in the following format:\n\n");
        prompt.push_str("```\n");
        prompt.push_str("REVIEW SUMMARY: [APPROVE/REQUEST_CHANGES/COMMENT]\n\n");
        prompt.push_str("BLOCKING:\n");
        prompt.push_str("- Issue description with file:line reference\n\n");
        prompt.push_str("IMPORTANT:\n");
        prompt.push_str("- Issue description with file:line reference\n\n");
        prompt.push_str("SUGGESTIONS:\n");
        prompt.push_str("- Suggestion with file:line reference\n\n");
        prompt.push_str("POSITIVE:\n");
        prompt.push_str("- Good patterns observed\n");
        prompt.push_str("```\n");

        prompt
    }
}

/// Builder for creating review requests with a fluent API
#[derive(Debug, Clone)]
pub struct ReviewRequestBuilder {
    request: ReviewRequest,
}

impl ReviewRequestBuilder {
    /// Create a new builder
    pub fn new(review_type: ReviewType, workdir: impl Into<PathBuf>) -> Self {
        Self {
            request: ReviewRequest::new(review_type, workdir),
        }
    }

    /// Set the task description
    pub fn task(mut self, task: impl Into<String>) -> Self {
        self.request.context.task = task.into();
        self
    }

    /// Set the diff to review
    pub fn diff(mut self, diff: impl Into<String>) -> Self {
        self.request.context.diff = Some(diff.into());
        self
    }

    /// Set the files to review
    pub fn files(mut self, files: Vec<String>) -> Self {
        self.request.context.files = files;
        self
    }

    /// Set the specification content
    pub fn spec(mut self, spec: impl Into<String>) -> Self {
        self.request.context.spec_content = Some(spec.into());
        self
    }

    /// Set the test files
    pub fn test_files(mut self, files: Vec<String>) -> Self {
        self.request.context.test_files = files;
        self
    }

    /// Set the implementation files
    pub fn impl_files(mut self, files: Vec<String>) -> Self {
        self.request.context.impl_files = files;
        self
    }

    /// Set previous feedback
    pub fn previous_feedback(mut self, feedback: impl Into<String>) -> Self {
        self.request.context.previous_feedback = Some(feedback.into());
        self
    }

    /// Set the iteration number
    pub fn iteration(mut self, iteration: u32) -> Self {
        self.request.context.iteration = iteration;
        self
    }

    /// Set max iterations
    pub fn max_iterations(mut self, max: u32) -> Self {
        self.request.max_iterations = max;
        self
    }

    /// Build the review request
    pub fn build(self) -> ReviewRequest {
        self.request
    }
}

/// Convenience functions for creating specific review types
impl ReviewRequest {
    /// Create a specification review request
    pub fn spec_review(
        task: impl Into<String>,
        spec_content: impl Into<String>,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self::builder(ReviewType::Spec, workdir)
            .task(task)
            .spec(spec_content)
            .build()
    }

    /// Create a test review request
    pub fn test_review(
        task: impl Into<String>,
        test_files: Vec<String>,
        diff: impl Into<String>,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self::builder(ReviewType::Test, workdir)
            .task(task)
            .test_files(test_files)
            .diff(diff)
            .build()
    }

    /// Create a code review request
    pub fn code_review(
        task: impl Into<String>,
        impl_files: Vec<String>,
        diff: impl Into<String>,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self::builder(ReviewType::Code, workdir)
            .task(task)
            .impl_files(impl_files)
            .diff(diff)
            .build()
    }

    /// Create a final review request
    pub fn final_review(
        task: impl Into<String>,
        files: Vec<String>,
        diff: impl Into<String>,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self::builder(ReviewType::Final, workdir)
            .task(task)
            .files(files)
            .diff(diff)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_type_description() {
        assert_eq!(ReviewType::Spec.description(), "Specification Review");
        assert_eq!(ReviewType::Test.description(), "Test Review");
        assert_eq!(ReviewType::Code.description(), "Code Review");
        assert_eq!(ReviewType::Final.description(), "Final Review");
    }

    #[test]
    fn test_review_type_focus_areas() {
        let spec_areas = ReviewType::Spec.focus_areas();
        assert!(spec_areas.len() >= 3);
        assert!(spec_areas.iter().any(|a| a.contains("requirements")));

        let code_areas = ReviewType::Code.focus_areas();
        assert!(code_areas.iter().any(|a| a.contains("Security")));
    }

    #[test]
    fn test_review_type_display() {
        assert_eq!(format!("{}", ReviewType::Spec), "Specification Review");
        assert_eq!(format!("{}", ReviewType::Code), "Code Review");
    }

    #[test]
    fn test_review_context_builder() {
        let context = ReviewContext::new()
            .with_task("Implement feature X")
            .with_diff("+ added line")
            .with_files(vec!["src/main.rs".to_string()])
            .with_iteration(1);

        assert_eq!(context.task, "Implement feature X");
        assert_eq!(context.diff, Some("+ added line".to_string()));
        assert_eq!(context.files, vec!["src/main.rs".to_string()]);
        assert_eq!(context.iteration, 1);
    }

    #[test]
    fn test_review_request_new() {
        let request = ReviewRequest::new(ReviewType::Code, "/tmp/project");
        assert_eq!(request.review_type, ReviewType::Code);
        assert_eq!(request.workdir, PathBuf::from("/tmp/project"));
        assert_eq!(request.max_iterations, 3);
    }

    #[test]
    fn test_review_request_builder() {
        let request = ReviewRequest::builder(ReviewType::Test, "/tmp/project")
            .task("Test login feature")
            .test_files(vec!["tests/login_test.rs".to_string()])
            .diff("+ new test")
            .iteration(0)
            .max_iterations(5)
            .build();

        assert_eq!(request.review_type, ReviewType::Test);
        assert_eq!(request.context.task, "Test login feature");
        assert_eq!(
            request.context.test_files,
            vec!["tests/login_test.rs".to_string()]
        );
        assert_eq!(request.context.diff, Some("+ new test".to_string()));
        assert_eq!(request.max_iterations, 5);
    }

    #[test]
    fn test_review_request_is_rereview() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .iteration(0)
            .build();
        assert!(!request.is_rereview());

        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .iteration(1)
            .build();
        assert!(request.is_rereview());
    }

    #[test]
    fn test_review_request_exceeded_max_iterations() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .iteration(2)
            .max_iterations(3)
            .build();
        assert!(!request.exceeded_max_iterations());

        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .iteration(3)
            .max_iterations(3)
            .build();
        assert!(request.exceeded_max_iterations());
    }

    #[test]
    fn test_spec_review_convenience() {
        let request = ReviewRequest::spec_review(
            "Design login feature",
            "# Login Spec\n\nUser should be able to login with email/password",
            "/tmp/project",
        );

        assert_eq!(request.review_type, ReviewType::Spec);
        assert_eq!(request.context.task, "Design login feature");
        assert!(request.context.spec_content.is_some());
    }

    #[test]
    fn test_test_review_convenience() {
        let request = ReviewRequest::test_review(
            "Test login feature",
            vec!["tests/login_test.rs".to_string()],
            "+ #[test]\n+ fn test_login() {}",
            "/tmp/project",
        );

        assert_eq!(request.review_type, ReviewType::Test);
        assert!(request.context.diff.is_some());
        assert!(!request.context.test_files.is_empty());
    }

    #[test]
    fn test_code_review_convenience() {
        let request = ReviewRequest::code_review(
            "Implement login",
            vec!["src/auth.rs".to_string()],
            "+ fn login() {}",
            "/tmp/project",
        );

        assert_eq!(request.review_type, ReviewType::Code);
        assert!(!request.context.impl_files.is_empty());
    }

    #[test]
    fn test_final_review_convenience() {
        let request = ReviewRequest::final_review(
            "Complete login feature",
            vec!["src/auth.rs".to_string(), "tests/auth_test.rs".to_string()],
            "+ complete diff",
            "/tmp/project",
        );

        assert_eq!(request.review_type, ReviewType::Final);
        assert_eq!(request.context.files.len(), 2);
    }

    #[test]
    fn test_to_prompt_contains_review_type() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Implement feature")
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Code Review"));
        assert!(prompt.contains("Gate:"));
    }

    #[test]
    fn test_to_prompt_contains_task() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Implement the login feature")
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Implement the login feature"));
    }

    #[test]
    fn test_to_prompt_contains_focus_areas() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Task")
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Focus Areas"));
        assert!(prompt.contains("Correctness"));
        assert!(prompt.contains("Security"));
    }

    #[test]
    fn test_to_prompt_contains_diff() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Task")
            .diff("+ added line\n- removed line")
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("```diff"));
        assert!(prompt.contains("+ added line"));
        assert!(prompt.contains("- removed line"));
    }

    #[test]
    fn test_to_prompt_contains_files_for_code_review() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Task")
            .impl_files(vec!["src/main.rs".to_string(), "src/lib.rs".to_string()])
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Files to Review"));
        assert!(prompt.contains("`src/main.rs`"));
        assert!(prompt.contains("`src/lib.rs`"));
    }

    #[test]
    fn test_to_prompt_contains_test_files_for_test_review() {
        let request = ReviewRequest::builder(ReviewType::Test, "/tmp")
            .task("Task")
            .test_files(vec!["tests/test_main.rs".to_string()])
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("`tests/test_main.rs`"));
    }

    #[test]
    fn test_to_prompt_contains_spec() {
        let request = ReviewRequest::builder(ReviewType::Spec, "/tmp")
            .task("Task")
            .spec("# Specification\n\nThis is the spec content")
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("## Specification"));
        assert!(prompt.contains("This is the spec content"));
    }

    #[test]
    fn test_to_prompt_contains_previous_feedback() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Task")
            .previous_feedback("- Fix the null check in login()")
            .iteration(1)
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Previous Review Feedback"));
        assert!(prompt.contains("Fix the null check"));
        assert!(prompt.contains("Review Iteration: 2/3"));
    }

    #[test]
    fn test_to_prompt_contains_output_format() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp")
            .task("Task")
            .build();

        let prompt = request.to_prompt();
        assert!(prompt.contains("Expected Output Format"));
        assert!(prompt.contains("REVIEW SUMMARY"));
        assert!(prompt.contains("BLOCKING"));
        assert!(prompt.contains("APPROVE/REQUEST_CHANGES/COMMENT"));
    }

    #[test]
    fn test_review_context_with_all_file_types() {
        let context = ReviewContext::new()
            .with_files(vec!["README.md".to_string()])
            .with_test_files(vec!["tests/test.rs".to_string()])
            .with_impl_files(vec!["src/lib.rs".to_string()]);

        assert_eq!(context.files.len(), 1);
        assert_eq!(context.test_files.len(), 1);
        assert_eq!(context.impl_files.len(), 1);
    }

    #[test]
    fn test_final_review_includes_all_files() {
        let request = ReviewRequest::builder(ReviewType::Final, "/tmp")
            .task("Final review")
            .files(vec!["README.md".to_string()])
            .test_files(vec!["tests/test.rs".to_string()])
            .impl_files(vec!["src/lib.rs".to_string()])
            .build();

        let prompt = request.to_prompt();
        // Final review should include all file types
        assert!(prompt.contains("`README.md`"));
        assert!(prompt.contains("`tests/test.rs`"));
        assert!(prompt.contains("`src/lib.rs`"));
    }

    #[test]
    fn test_serde_roundtrip_review_type() {
        let review_type = ReviewType::Code;
        let json = serde_json::to_string(&review_type).unwrap();
        let parsed: ReviewType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, review_type);
    }

    #[test]
    fn test_serde_roundtrip_review_request() {
        let request = ReviewRequest::builder(ReviewType::Code, "/tmp/project")
            .task("Review this")
            .diff("+ line")
            .impl_files(vec!["src/main.rs".to_string()])
            .iteration(1)
            .build();

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ReviewRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.review_type, ReviewType::Code);
        assert_eq!(parsed.context.task, "Review this");
        assert_eq!(parsed.context.iteration, 1);
    }
}
