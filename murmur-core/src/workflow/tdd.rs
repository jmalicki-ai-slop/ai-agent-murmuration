//! Test-Driven Development workflow
//!
//! This module implements the TDD cycle:
//! 1. RED: Write a failing test
//! 2. GREEN: Write minimal code to make test pass
//! 3. REFACTOR: Clean up code while keeping tests green

use crate::agent::{AgentFactory, ImplementAgent, TestAgent};
use crate::config::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The current phase of the TDD cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TddPhase {
    /// Write a failing test that describes the expected behavior
    Red,
    /// Write minimal code to make the test pass
    Green,
    /// Clean up code while keeping tests passing (optional)
    Refactor,
    /// TDD cycle complete
    Complete,
}

impl TddPhase {
    /// Get the next phase in the cycle
    pub fn next(&self) -> Option<TddPhase> {
        match self {
            TddPhase::Red => Some(TddPhase::Green),
            TddPhase::Green => Some(TddPhase::Refactor),
            TddPhase::Refactor => Some(TddPhase::Complete),
            TddPhase::Complete => None,
        }
    }

    /// Skip refactor and go directly to complete
    pub fn skip_refactor(&self) -> Option<TddPhase> {
        match self {
            TddPhase::Green => Some(TddPhase::Complete),
            other => other.next(),
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            TddPhase::Red => "Writing failing test",
            TddPhase::Green => "Implementing to pass test",
            TddPhase::Refactor => "Refactoring while keeping tests green",
            TddPhase::Complete => "TDD cycle complete",
        }
    }
}

impl std::fmt::Display for TddPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// State tracking for a TDD workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TddState {
    /// Current phase
    pub phase: TddPhase,
    /// The behavior being tested
    pub behavior: String,
    /// Files involved in the test
    pub test_files: Vec<String>,
    /// Files involved in the implementation
    pub impl_files: Vec<String>,
    /// Working directory
    pub workdir: PathBuf,
    /// Number of RED->GREEN iterations completed
    pub iterations: u32,
    /// Maximum iterations before giving up
    pub max_iterations: u32,
    /// Whether to include refactoring phase
    pub include_refactor: bool,
    /// History of phase transitions
    pub history: Vec<TddTransition>,
}

/// A transition between TDD phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TddTransition {
    /// Phase we transitioned from
    pub from: TddPhase,
    /// Phase we transitioned to
    pub to: TddPhase,
    /// Whether the transition was successful
    pub success: bool,
    /// Optional message about the transition
    pub message: Option<String>,
}

impl TddState {
    /// Create a new TDD state
    pub fn new(behavior: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            phase: TddPhase::Red,
            behavior: behavior.into(),
            test_files: Vec::new(),
            impl_files: Vec::new(),
            workdir: workdir.into(),
            iterations: 0,
            max_iterations: 3,
            include_refactor: false,
            history: Vec::new(),
        }
    }

    /// Set test files
    pub fn with_test_files(mut self, files: Vec<String>) -> Self {
        self.test_files = files;
        self
    }

    /// Set implementation files
    pub fn with_impl_files(mut self, files: Vec<String>) -> Self {
        self.impl_files = files;
        self
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Enable refactoring phase
    pub fn with_refactor(mut self) -> Self {
        self.include_refactor = true;
        self
    }

    /// Transition to the next phase
    pub fn advance(&mut self, success: bool, message: Option<String>) -> Option<TddPhase> {
        let from = self.phase;
        let to = if self.include_refactor {
            self.phase.next()
        } else {
            self.phase.skip_refactor()
        };

        if let Some(next_phase) = to {
            self.history.push(TddTransition {
                from,
                to: next_phase,
                success,
                message,
            });
            self.phase = next_phase;

            if from == TddPhase::Green {
                self.iterations += 1;
            }

            Some(next_phase)
        } else {
            None
        }
    }

    /// Go back to RED phase (test still failing)
    pub fn retry_red(&mut self, message: Option<String>) {
        self.history.push(TddTransition {
            from: self.phase,
            to: TddPhase::Red,
            success: false,
            message,
        });
        self.phase = TddPhase::Red;
    }

    /// Check if we've exceeded max iterations
    pub fn exceeded_max_iterations(&self) -> bool {
        self.iterations >= self.max_iterations
    }

    /// Check if the workflow is complete
    pub fn is_complete(&self) -> bool {
        self.phase == TddPhase::Complete
    }
}

/// Workflow coordinator for TDD
#[derive(Debug)]
pub struct TddWorkflow {
    /// Current state
    state: TddState,
    /// Agent factory for creating agents
    factory: AgentFactory,
}

impl TddWorkflow {
    /// Create a new TDD workflow
    pub fn new(behavior: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            state: TddState::new(behavior, workdir),
            factory: AgentFactory::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        behavior: impl Into<String>,
        workdir: impl Into<PathBuf>,
        config: AgentConfig,
    ) -> Self {
        Self {
            state: TddState::new(behavior, workdir),
            factory: AgentFactory::with_config(config),
        }
    }

    /// Get the current state
    pub fn state(&self) -> &TddState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut TddState {
        &mut self.state
    }

    /// Get the current phase
    pub fn phase(&self) -> TddPhase {
        self.state.phase
    }

    /// Get the test agent
    pub fn test_agent(&self) -> TestAgent {
        self.factory.test()
    }

    /// Get the implement agent
    pub fn implement_agent(&self) -> ImplementAgent {
        self.factory.implement()
    }

    /// Get the prompt for the current phase
    pub fn current_prompt(&self) -> String {
        match self.state.phase {
            TddPhase::Red => self.red_prompt(),
            TddPhase::Green => self.green_prompt(),
            TddPhase::Refactor => self.refactor_prompt(),
            TddPhase::Complete => "TDD cycle complete.".to_string(),
        }
    }

    fn red_prompt(&self) -> String {
        format!(
            "Write a failing test that describes the following behavior:\n\n{}\n\n\
             The test should fail because the behavior is not yet implemented.\n\
             Focus on testing the expected behavior, not implementation details.\n\
             After writing the test, run it to confirm it fails.",
            self.state.behavior
        )
    }

    fn green_prompt(&self) -> String {
        format!(
            "The test for the following behavior is now failing:\n\n{}\n\n\
             Write the MINIMAL code necessary to make the test pass.\n\
             Do not add extra features or optimizations.\n\
             After implementing, run the tests to confirm they pass.",
            self.state.behavior
        )
    }

    fn refactor_prompt(&self) -> String {
        format!(
            "The test for the following behavior is now passing:\n\n{}\n\n\
             Review the code and make any necessary improvements:\n\
             - Remove duplication\n\
             - Improve naming\n\
             - Simplify complex logic\n\n\
             Keep running tests to ensure they stay green.",
            self.state.behavior
        )
    }

    /// Advance to the next phase
    pub fn advance(&mut self, success: bool, message: Option<String>) -> Option<TddPhase> {
        self.state.advance(success, message)
    }

    /// Go back to RED phase
    pub fn retry(&mut self, message: Option<String>) {
        self.state.retry_red(message);
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Check if should give up
    pub fn should_give_up(&self) -> bool {
        self.state.exceeded_max_iterations()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transitions() {
        assert_eq!(TddPhase::Red.next(), Some(TddPhase::Green));
        assert_eq!(TddPhase::Green.next(), Some(TddPhase::Refactor));
        assert_eq!(TddPhase::Refactor.next(), Some(TddPhase::Complete));
        assert_eq!(TddPhase::Complete.next(), None);
    }

    #[test]
    fn test_skip_refactor() {
        assert_eq!(TddPhase::Green.skip_refactor(), Some(TddPhase::Complete));
        assert_eq!(TddPhase::Red.skip_refactor(), Some(TddPhase::Green));
    }

    #[test]
    fn test_state_new() {
        let state = TddState::new("test behavior", "/tmp/test");
        assert_eq!(state.phase, TddPhase::Red);
        assert_eq!(state.behavior, "test behavior");
        assert_eq!(state.iterations, 0);
    }

    #[test]
    fn test_state_advance() {
        let mut state = TddState::new("test", "/tmp");
        assert_eq!(state.advance(true, None), Some(TddPhase::Green));
        assert_eq!(state.phase, TddPhase::Green);
        assert_eq!(state.history.len(), 1);
    }

    #[test]
    fn test_state_skip_refactor() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None); // Red -> Green
        state.advance(true, None); // Green -> Complete (skip refactor)
        assert_eq!(state.phase, TddPhase::Complete);
    }

    #[test]
    fn test_state_with_refactor() {
        let mut state = TddState::new("test", "/tmp").with_refactor();
        state.advance(true, None); // Red -> Green
        state.advance(true, None); // Green -> Refactor
        assert_eq!(state.phase, TddPhase::Refactor);
        state.advance(true, None); // Refactor -> Complete
        assert_eq!(state.phase, TddPhase::Complete);
    }

    #[test]
    fn test_state_retry() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None); // Red -> Green
        state.retry_red(Some("Test still failing".to_string()));
        assert_eq!(state.phase, TddPhase::Red);
        assert_eq!(state.history.len(), 2);
    }

    #[test]
    fn test_state_max_iterations() {
        let mut state = TddState::new("test", "/tmp").with_max_iterations(2);
        state.advance(true, None); // Red -> Green, iter=0
        state.advance(true, None); // Green -> Complete, iter=1
        // Would need to reset to test more, but we can check the counter
        assert_eq!(state.iterations, 1);
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = TddWorkflow::new("Add login feature", "/tmp/project");
        assert_eq!(workflow.phase(), TddPhase::Red);
        assert!(!workflow.is_complete());
    }

    #[test]
    fn test_workflow_prompts() {
        let workflow = TddWorkflow::new("Add login feature", "/tmp/project");
        let prompt = workflow.current_prompt();
        assert!(prompt.contains("Add login feature"));
        assert!(prompt.contains("failing test"));
    }

    #[test]
    fn test_workflow_advance() {
        let mut workflow = TddWorkflow::new("test", "/tmp");
        workflow.advance(true, None);
        assert_eq!(workflow.phase(), TddPhase::Green);

        let prompt = workflow.current_prompt();
        assert!(prompt.contains("MINIMAL code"));
    }

    #[test]
    fn test_workflow_complete() {
        let mut workflow = TddWorkflow::new("test", "/tmp");
        workflow.advance(true, None); // Red -> Green
        workflow.advance(true, None); // Green -> Complete
        assert!(workflow.is_complete());
    }
}
