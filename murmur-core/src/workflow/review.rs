//! Review workflow for code changes
//!
//! This module provides review workflow that can be inserted at various points
//! in the development process.

use crate::agent::{AgentFactory, ReviewAgent};
use crate::config::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of a code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// Overall verdict
    pub verdict: ReviewVerdict,
    /// Blocking issues that must be fixed
    pub blocking: Vec<ReviewIssue>,
    /// Important issues that should be fixed
    pub important: Vec<ReviewIssue>,
    /// Suggestions for improvement
    pub suggestions: Vec<ReviewIssue>,
    /// Positive feedback
    pub positives: Vec<String>,
}

impl Default for ReviewResult {
    fn default() -> Self {
        Self {
            verdict: ReviewVerdict::Pending,
            blocking: Vec::new(),
            important: Vec::new(),
            suggestions: Vec::new(),
            positives: Vec::new(),
        }
    }
}

/// The overall review verdict
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReviewVerdict {
    /// Review not yet complete
    #[default]
    Pending,
    /// Changes approved
    Approved,
    /// Changes need work
    RequestChanges,
    /// General comment, no strong opinion
    Comment,
}

impl ReviewVerdict {
    /// Check if approved
    pub fn is_approved(&self) -> bool {
        matches!(self, ReviewVerdict::Approved)
    }

    /// Check if blocking
    pub fn is_blocking(&self) -> bool {
        matches!(self, ReviewVerdict::RequestChanges)
    }
}

impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewVerdict::Pending => write!(f, "Pending"),
            ReviewVerdict::Approved => write!(f, "Approved"),
            ReviewVerdict::RequestChanges => write!(f, "Request Changes"),
            ReviewVerdict::Comment => write!(f, "Comment"),
        }
    }
}

/// A single review issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// The file where the issue was found
    pub file: Option<String>,
    /// The line number (if applicable)
    pub line: Option<u32>,
    /// Description of the issue
    pub description: String,
    /// Suggested fix (if any)
    pub suggestion: Option<String>,
}

impl ReviewIssue {
    /// Create a new review issue
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            file: None,
            line: None,
            description: description.into(),
            suggestion: None,
        }
    }

    /// Add file location
    pub fn at_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Add line number
    pub fn at_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    /// Add suggested fix
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for ReviewIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref file) = self.file {
            if let Some(line) = self.line {
                write!(f, "{}:{}: ", file, line)?;
            } else {
                write!(f, "{}: ", file)?;
            }
        }
        write!(f, "{}", self.description)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

/// When to trigger a review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewTrigger {
    /// Review after each implementation phase
    AfterImplementation,
    /// Review after tests pass
    AfterTestsPass,
    /// Review before creating PR
    BeforePR,
    /// Review on explicit request
    OnDemand,
}

/// State for a review workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    /// When the review was triggered
    pub trigger: ReviewTrigger,
    /// The task being reviewed
    pub task: String,
    /// Working directory
    pub workdir: PathBuf,
    /// The diff being reviewed
    pub diff: String,
    /// Current result
    pub result: ReviewResult,
    /// Number of review iterations
    pub iterations: u32,
    /// Maximum iterations
    pub max_iterations: u32,
}

impl ReviewState {
    /// Create a new review state
    pub fn new(
        trigger: ReviewTrigger,
        task: impl Into<String>,
        workdir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            trigger,
            task: task.into(),
            workdir: workdir.into(),
            diff: String::new(),
            result: ReviewResult::default(),
            iterations: 0,
            max_iterations: 2,
        }
    }

    /// Set the diff to review
    pub fn with_diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = diff.into();
        self
    }

    /// Check if review is approved
    pub fn is_approved(&self) -> bool {
        self.result.verdict.is_approved()
    }

    /// Check if review is blocking
    pub fn needs_changes(&self) -> bool {
        self.result.verdict.is_blocking()
    }

    /// Record review iteration
    pub fn record_iteration(&mut self) {
        self.iterations += 1;
    }

    /// Check if exceeded max iterations
    pub fn exceeded_max_iterations(&self) -> bool {
        self.iterations >= self.max_iterations
    }
}

/// Workflow for code reviews
#[derive(Debug)]
pub struct ReviewWorkflow {
    state: ReviewState,
    factory: AgentFactory,
}

impl ReviewWorkflow {
    /// Create a new review workflow
    pub fn new(trigger: ReviewTrigger, task: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            state: ReviewState::new(trigger, task, workdir),
            factory: AgentFactory::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(
        trigger: ReviewTrigger,
        task: impl Into<String>,
        workdir: impl Into<PathBuf>,
        config: AgentConfig,
    ) -> Self {
        Self {
            state: ReviewState::new(trigger, task, workdir),
            factory: AgentFactory::with_config(config),
        }
    }

    /// Get the state
    pub fn state(&self) -> &ReviewState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut ReviewState {
        &mut self.state
    }

    /// Get the review agent
    pub fn review_agent(&self) -> ReviewAgent {
        self.factory.review()
    }

    /// Get the prompt for the review
    pub fn review_prompt(&self) -> String {
        format!(
            "Review the following changes for the task:\n\n{}\n\n\
             Diff:\n```\n{}\n```\n\n\
             Provide your review in the following format:\n\
             - VERDICT: APPROVE/REQUEST_CHANGES/COMMENT\n\
             - BLOCKING: List any blocking issues\n\
             - IMPORTANT: List important but non-blocking issues\n\
             - SUGGESTIONS: List nice-to-have improvements\n\
             - POSITIVE: List good patterns observed",
            self.state.task, self.state.diff
        )
    }

    /// Set the diff
    pub fn set_diff(&mut self, diff: impl Into<String>) {
        self.state.diff = diff.into();
    }

    /// Record a review result
    pub fn record_result(&mut self, result: ReviewResult) {
        self.state.result = result;
        self.state.record_iteration();
    }

    /// Check if approved
    pub fn is_approved(&self) -> bool {
        self.state.is_approved()
    }

    /// Check if needs changes
    pub fn needs_changes(&self) -> bool {
        self.state.needs_changes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_verdict() {
        assert!(ReviewVerdict::Approved.is_approved());
        assert!(!ReviewVerdict::RequestChanges.is_approved());
        assert!(ReviewVerdict::RequestChanges.is_blocking());
        assert!(!ReviewVerdict::Approved.is_blocking());
    }

    #[test]
    fn test_review_issue() {
        let issue = ReviewIssue::new("Fix the bug")
            .at_file("src/main.rs")
            .at_line(42)
            .with_suggestion("Use a different approach");

        assert_eq!(issue.file, Some("src/main.rs".to_string()));
        assert_eq!(issue.line, Some(42));
        assert!(issue.to_string().contains("src/main.rs:42"));
    }

    #[test]
    fn test_review_state() {
        let state = ReviewState::new(ReviewTrigger::AfterImplementation, "task", "/tmp");
        assert!(!state.is_approved());
        assert!(!state.needs_changes());
    }

    #[test]
    fn test_review_workflow() {
        let mut workflow = ReviewWorkflow::new(ReviewTrigger::BeforePR, "task", "/tmp");
        workflow.set_diff("+ added line");

        let prompt = workflow.review_prompt();
        assert!(prompt.contains("task"));
        assert!(prompt.contains("+ added line"));
    }

    #[test]
    fn test_review_result_approved() {
        let mut workflow = ReviewWorkflow::new(ReviewTrigger::OnDemand, "task", "/tmp");
        workflow.record_result(ReviewResult {
            verdict: ReviewVerdict::Approved,
            ..Default::default()
        });

        assert!(workflow.is_approved());
        assert!(!workflow.needs_changes());
    }

    #[test]
    fn test_review_iterations() {
        let mut state = ReviewState::new(ReviewTrigger::OnDemand, "task", "/tmp");
        assert!(!state.exceeded_max_iterations());

        state.record_iteration();
        state.record_iteration();
        assert!(state.exceeded_max_iterations());
    }
}
