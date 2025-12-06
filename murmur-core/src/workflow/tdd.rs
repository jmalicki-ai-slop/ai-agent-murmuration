//! Test-Driven Development workflow
//!
//! This module implements the full TDD cycle:
//! 1. WriteSpec: Write specification/design document
//! 2. WriteTests: Write tests based on spec
//! 3. VerifyRed: Verify tests fail (red phase)
//! 4. Implement: Write code to make tests pass
//! 5. VerifyGreen: Verify tests pass (green phase)
//! 6. Refactor: Clean up code while keeping tests green
//! 7. Complete: TDD cycle finished

use crate::agent::{AgentFactory, ImplementAgent, TestAgent};
use crate::config::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The current phase of the TDD cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TddPhase {
    /// Write specification/design document
    #[default]
    WriteSpec,
    /// Write tests based on spec
    WriteTests,
    /// Verify tests fail (red phase)
    VerifyRed,
    /// Implement to make tests pass
    Implement,
    /// Verify tests pass (green phase)
    VerifyGreen,
    /// Refactor and clean up
    Refactor,
    /// Complete
    Complete,
}

impl TddPhase {
    /// Get the next phase in the cycle
    pub fn next(&self) -> Option<TddPhase> {
        match self {
            TddPhase::WriteSpec => Some(TddPhase::WriteTests),
            TddPhase::WriteTests => Some(TddPhase::VerifyRed),
            TddPhase::VerifyRed => Some(TddPhase::Implement),
            TddPhase::Implement => Some(TddPhase::VerifyGreen),
            TddPhase::VerifyGreen => Some(TddPhase::Refactor),
            TddPhase::Refactor => Some(TddPhase::Complete),
            TddPhase::Complete => None,
        }
    }

    /// Get the previous phase in the cycle (for iteration)
    pub fn previous(&self) -> Option<TddPhase> {
        match self {
            TddPhase::WriteSpec => None,
            TddPhase::WriteTests => Some(TddPhase::WriteSpec),
            TddPhase::VerifyRed => Some(TddPhase::WriteTests),
            TddPhase::Implement => Some(TddPhase::VerifyRed),
            TddPhase::VerifyGreen => Some(TddPhase::Implement),
            TddPhase::Refactor => Some(TddPhase::VerifyGreen),
            TddPhase::Complete => Some(TddPhase::Refactor),
        }
    }

    /// Check if transitioning to the target phase is allowed.
    ///
    /// Forward transitions follow the standard flow.
    /// Backward transitions are allowed for iteration:
    /// - From VerifyRed can go back to WriteTests (tests don't fail properly)
    /// - From VerifyGreen can go back to Implement (tests still failing)
    /// - From Refactor can go back to VerifyGreen (refactoring broke tests)
    /// - Any phase can restart from WriteSpec
    pub fn can_transition_to(&self, target: &TddPhase) -> bool {
        // Same phase is not a transition
        if self == target {
            return false;
        }

        // Forward to next phase is always allowed
        if self.next().as_ref() == Some(target) {
            return true;
        }

        // Can always restart from WriteSpec
        if *target == TddPhase::WriteSpec {
            return true;
        }

        // Specific backward transitions for iteration
        match (self, target) {
            // Tests don't fail as expected - revise tests
            (TddPhase::VerifyRed, TddPhase::WriteTests) => true,
            // Implementation didn't make tests pass - keep implementing
            (TddPhase::VerifyGreen, TddPhase::Implement) => true,
            // Refactoring broke tests - go back to verify
            (TddPhase::Refactor, TddPhase::VerifyGreen) => true,
            // Can go back from Complete to Refactor for additional cleanup
            (TddPhase::Complete, TddPhase::Refactor) => true,
            _ => false,
        }
    }

    /// Get all phases that can be transitioned to from this phase
    pub fn valid_transitions(&self) -> Vec<TddPhase> {
        let all_phases = [
            TddPhase::WriteSpec,
            TddPhase::WriteTests,
            TddPhase::VerifyRed,
            TddPhase::Implement,
            TddPhase::VerifyGreen,
            TddPhase::Refactor,
            TddPhase::Complete,
        ];
        all_phases
            .into_iter()
            .filter(|p| self.can_transition_to(p))
            .collect()
    }

    /// Skip WriteSpec and start directly with WriteTests
    pub fn skip_spec(&self) -> Option<TddPhase> {
        match self {
            TddPhase::WriteSpec => Some(TddPhase::WriteTests),
            other => other.next(),
        }
    }

    /// Skip refactor and go directly to complete
    pub fn skip_refactor(&self) -> Option<TddPhase> {
        match self {
            TddPhase::VerifyGreen => Some(TddPhase::Complete),
            other => other.next(),
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            TddPhase::WriteSpec => "Writing specification document",
            TddPhase::WriteTests => "Writing tests based on spec",
            TddPhase::VerifyRed => "Verifying tests fail (red phase)",
            TddPhase::Implement => "Implementing to make tests pass",
            TddPhase::VerifyGreen => "Verifying tests pass (green phase)",
            TddPhase::Refactor => "Refactoring while keeping tests green",
            TddPhase::Complete => "TDD cycle complete",
        }
    }

    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, TddPhase::Complete)
    }

    /// Get the validation requirements for this phase
    pub fn validation_requirements(&self) -> PhaseValidation {
        match self {
            TddPhase::WriteSpec => PhaseValidation {
                description: "Specification document must exist and describe expected behavior",
                requires_spec: false,
                requires_tests: false,
                requires_test_failure: false,
                requires_test_success: false,
                requires_implementation: false,
            },
            TddPhase::WriteTests => PhaseValidation {
                description: "Tests must be written based on the specification",
                requires_spec: true,
                requires_tests: false,
                requires_test_failure: false,
                requires_test_success: false,
                requires_implementation: false,
            },
            TddPhase::VerifyRed => PhaseValidation {
                description: "Tests must exist and fail (proving they test something)",
                requires_spec: true,
                requires_tests: true,
                requires_test_failure: true,
                requires_test_success: false,
                requires_implementation: false,
            },
            TddPhase::Implement => PhaseValidation {
                description: "Implementation code to make tests pass",
                requires_spec: true,
                requires_tests: true,
                requires_test_failure: false,
                requires_test_success: false,
                requires_implementation: false,
            },
            TddPhase::VerifyGreen => PhaseValidation {
                description: "All tests must pass",
                requires_spec: true,
                requires_tests: true,
                requires_test_failure: false,
                requires_test_success: true,
                requires_implementation: true,
            },
            TddPhase::Refactor => PhaseValidation {
                description: "Code can be refactored while keeping tests green",
                requires_spec: true,
                requires_tests: true,
                requires_test_failure: false,
                requires_test_success: true,
                requires_implementation: true,
            },
            TddPhase::Complete => PhaseValidation {
                description: "TDD cycle is complete",
                requires_spec: true,
                requires_tests: true,
                requires_test_failure: false,
                requires_test_success: true,
                requires_implementation: true,
            },
        }
    }
}

impl std::fmt::Display for TddPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Validation requirements for a TDD phase
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseValidation {
    /// Human-readable description of what this phase requires
    pub description: &'static str,
    /// Whether a specification document is required
    pub requires_spec: bool,
    /// Whether test files must exist
    pub requires_tests: bool,
    /// Whether tests must be failing (VerifyRed)
    pub requires_test_failure: bool,
    /// Whether tests must be passing (VerifyGreen, Refactor, Complete)
    pub requires_test_success: bool,
    /// Whether implementation code must exist
    pub requires_implementation: bool,
}

/// State tracking for a TDD workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TddState {
    /// Current phase
    pub phase: TddPhase,
    /// The behavior being tested
    pub behavior: String,
    /// Specification document path (if any)
    pub spec_file: Option<String>,
    /// Files involved in the test
    pub test_files: Vec<String>,
    /// Files involved in the implementation
    pub impl_files: Vec<String>,
    /// Working directory
    pub workdir: PathBuf,
    /// Number of Implement->VerifyGreen iterations completed
    pub iterations: u32,
    /// Maximum iterations before giving up
    pub max_iterations: u32,
    /// Whether to skip the spec phase
    pub skip_spec: bool,
    /// Whether to skip the refactor phase
    pub skip_refactor: bool,
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
            phase: TddPhase::WriteSpec,
            behavior: behavior.into(),
            spec_file: None,
            test_files: Vec::new(),
            impl_files: Vec::new(),
            workdir: workdir.into(),
            iterations: 0,
            max_iterations: 3,
            skip_spec: false,
            skip_refactor: false,
            history: Vec::new(),
        }
    }

    /// Create a new TDD state starting from WriteTests (skipping spec)
    pub fn new_without_spec(behavior: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            phase: TddPhase::WriteTests,
            behavior: behavior.into(),
            spec_file: None,
            test_files: Vec::new(),
            impl_files: Vec::new(),
            workdir: workdir.into(),
            iterations: 0,
            max_iterations: 3,
            skip_spec: true,
            skip_refactor: false,
            history: Vec::new(),
        }
    }

    /// Set specification file
    pub fn with_spec_file(mut self, file: impl Into<String>) -> Self {
        self.spec_file = Some(file.into());
        self
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

    /// Skip the WriteSpec phase
    pub fn with_skip_spec(mut self) -> Self {
        self.skip_spec = true;
        if self.phase == TddPhase::WriteSpec {
            self.phase = TddPhase::WriteTests;
        }
        self
    }

    /// Skip the Refactor phase
    pub fn with_skip_refactor(mut self) -> Self {
        self.skip_refactor = true;
        self
    }

    /// Transition to the next phase
    pub fn advance(&mut self, success: bool, message: Option<String>) -> Option<TddPhase> {
        let from = self.phase;
        let to = self.compute_next_phase();

        if let Some(next_phase) = to {
            self.history.push(TddTransition {
                from,
                to: next_phase,
                success,
                message,
            });
            self.phase = next_phase;

            // Track iterations when moving from Implement to VerifyGreen
            if from == TddPhase::Implement {
                self.iterations += 1;
            }

            Some(next_phase)
        } else {
            None
        }
    }

    /// Compute the next phase based on current configuration
    fn compute_next_phase(&self) -> Option<TddPhase> {
        match self.phase {
            TddPhase::WriteSpec if self.skip_spec => Some(TddPhase::WriteTests),
            TddPhase::VerifyGreen if self.skip_refactor => Some(TddPhase::Complete),
            _ => self.phase.next(),
        }
    }

    /// Transition to a specific phase if allowed
    pub fn transition_to(&mut self, target: TddPhase, message: Option<String>) -> bool {
        if !self.phase.can_transition_to(&target) {
            return false;
        }

        let from = self.phase;
        self.history.push(TddTransition {
            from,
            to: target,
            success: true,
            message,
        });
        self.phase = target;
        true
    }

    /// Go back to WriteTests phase (tests don't fail as expected)
    pub fn retry_tests(&mut self, message: Option<String>) {
        if self.phase.can_transition_to(&TddPhase::WriteTests) {
            self.history.push(TddTransition {
                from: self.phase,
                to: TddPhase::WriteTests,
                success: false,
                message,
            });
            self.phase = TddPhase::WriteTests;
        }
    }

    /// Go back to Implement phase (tests still failing)
    pub fn retry_implement(&mut self, message: Option<String>) {
        if self.phase.can_transition_to(&TddPhase::Implement) {
            self.history.push(TddTransition {
                from: self.phase,
                to: TddPhase::Implement,
                success: false,
                message,
            });
            self.phase = TddPhase::Implement;
        }
    }

    /// Restart from the beginning (WriteSpec or WriteTests depending on config)
    pub fn restart(&mut self, message: Option<String>) {
        let target = if self.skip_spec {
            TddPhase::WriteTests
        } else {
            TddPhase::WriteSpec
        };
        self.history.push(TddTransition {
            from: self.phase,
            to: target,
            success: false,
            message,
        });
        self.phase = target;
        self.iterations = 0;
    }

    /// Check if we've exceeded max iterations
    pub fn exceeded_max_iterations(&self) -> bool {
        self.iterations >= self.max_iterations
    }

    /// Check if the workflow is complete
    pub fn is_complete(&self) -> bool {
        self.phase == TddPhase::Complete
    }

    /// Get validation requirements for current phase
    pub fn current_validation(&self) -> PhaseValidation {
        self.phase.validation_requirements()
    }

    /// Get valid transitions from current phase
    pub fn valid_transitions(&self) -> Vec<TddPhase> {
        self.phase.valid_transitions()
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
    /// Create a new TDD workflow (starts from WriteSpec)
    pub fn new(behavior: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            state: TddState::new(behavior, workdir),
            factory: AgentFactory::new(),
        }
    }

    /// Create a new TDD workflow without spec phase (starts from WriteTests)
    pub fn new_without_spec(behavior: impl Into<String>, workdir: impl Into<PathBuf>) -> Self {
        Self {
            state: TddState::new_without_spec(behavior, workdir),
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
            TddPhase::WriteSpec => self.write_spec_prompt(),
            TddPhase::WriteTests => self.write_tests_prompt(),
            TddPhase::VerifyRed => self.verify_red_prompt(),
            TddPhase::Implement => self.implement_prompt(),
            TddPhase::VerifyGreen => self.verify_green_prompt(),
            TddPhase::Refactor => self.refactor_prompt(),
            TddPhase::Complete => "TDD cycle complete.".to_string(),
        }
    }

    fn write_spec_prompt(&self) -> String {
        format!(
            "Write a specification document for the following behavior:\n\n{}\n\n\
             The specification should:\n\
             - Clearly describe the expected behavior\n\
             - Define inputs and outputs\n\
             - List edge cases and error conditions\n\
             - Be detailed enough to write tests from",
            self.state.behavior
        )
    }

    fn write_tests_prompt(&self) -> String {
        let spec_note = if let Some(ref spec_file) = self.state.spec_file {
            format!("\n\nRefer to the specification in: {}", spec_file)
        } else {
            String::new()
        };
        format!(
            "Write tests for the following behavior:\n\n{}{}\n\n\
             The tests should:\n\
             - Cover the main functionality\n\
             - Include edge cases\n\
             - Be clear and readable\n\
             - NOT include any implementation code",
            self.state.behavior, spec_note
        )
    }

    fn verify_red_prompt(&self) -> String {
        format!(
            "Run the tests to verify they FAIL:\n\n{}\n\n\
             Expected: All new tests should fail because the behavior is not implemented.\n\
             If tests pass, they may not be testing the right thing.\n\
             If tests fail with unexpected errors, the test setup may need fixing.",
            self.state.behavior
        )
    }

    fn implement_prompt(&self) -> String {
        format!(
            "Implement the MINIMAL code to make the tests pass:\n\n{}\n\n\
             Guidelines:\n\
             - Write only enough code to make the tests pass\n\
             - Do not add extra features or optimizations\n\
             - Do not refactor yet - that comes later\n\
             - Focus on making tests green, not on perfect code",
            self.state.behavior
        )
    }

    fn verify_green_prompt(&self) -> String {
        format!(
            "Run all tests to verify they PASS:\n\n{}\n\n\
             Expected: All tests should pass.\n\
             If tests fail, the implementation needs more work.\n\
             Do not modify tests at this stage - fix the implementation instead.",
            self.state.behavior
        )
    }

    fn refactor_prompt(&self) -> String {
        format!(
            "Refactor the code while keeping tests green:\n\n{}\n\n\
             Review and improve the code:\n\
             - Remove duplication\n\
             - Improve naming\n\
             - Simplify complex logic\n\
             - Ensure code follows project conventions\n\n\
             Run tests after each change to ensure they stay green.",
            self.state.behavior
        )
    }

    /// Advance to the next phase
    pub fn advance(&mut self, success: bool, message: Option<String>) -> Option<TddPhase> {
        self.state.advance(success, message)
    }

    /// Transition to a specific phase (if allowed)
    pub fn transition_to(&mut self, target: TddPhase, message: Option<String>) -> bool {
        self.state.transition_to(target, message)
    }

    /// Go back to WriteTests phase (tests don't fail as expected)
    pub fn retry_tests(&mut self, message: Option<String>) {
        self.state.retry_tests(message);
    }

    /// Go back to Implement phase (tests still failing)
    pub fn retry_implement(&mut self, message: Option<String>) {
        self.state.retry_implement(message);
    }

    /// Restart the TDD cycle from the beginning
    pub fn restart(&mut self, message: Option<String>) {
        self.state.restart(message);
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Check if should give up
    pub fn should_give_up(&self) -> bool {
        self.state.exceeded_max_iterations()
    }

    /// Get validation requirements for current phase
    pub fn current_validation(&self) -> PhaseValidation {
        self.state.current_validation()
    }

    /// Get valid transitions from current phase
    pub fn valid_transitions(&self) -> Vec<TddPhase> {
        self.state.valid_transitions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TddPhase Tests ====================

    #[test]
    fn test_phase_transitions_next() {
        assert_eq!(TddPhase::WriteSpec.next(), Some(TddPhase::WriteTests));
        assert_eq!(TddPhase::WriteTests.next(), Some(TddPhase::VerifyRed));
        assert_eq!(TddPhase::VerifyRed.next(), Some(TddPhase::Implement));
        assert_eq!(TddPhase::Implement.next(), Some(TddPhase::VerifyGreen));
        assert_eq!(TddPhase::VerifyGreen.next(), Some(TddPhase::Refactor));
        assert_eq!(TddPhase::Refactor.next(), Some(TddPhase::Complete));
        assert_eq!(TddPhase::Complete.next(), None);
    }

    #[test]
    fn test_phase_transitions_previous() {
        assert_eq!(TddPhase::WriteSpec.previous(), None);
        assert_eq!(TddPhase::WriteTests.previous(), Some(TddPhase::WriteSpec));
        assert_eq!(TddPhase::VerifyRed.previous(), Some(TddPhase::WriteTests));
        assert_eq!(TddPhase::Implement.previous(), Some(TddPhase::VerifyRed));
        assert_eq!(TddPhase::VerifyGreen.previous(), Some(TddPhase::Implement));
        assert_eq!(TddPhase::Refactor.previous(), Some(TddPhase::VerifyGreen));
        assert_eq!(TddPhase::Complete.previous(), Some(TddPhase::Refactor));
    }

    #[test]
    fn test_can_transition_to_next() {
        // Forward transitions to next phase are always allowed
        assert!(TddPhase::WriteSpec.can_transition_to(&TddPhase::WriteTests));
        assert!(TddPhase::WriteTests.can_transition_to(&TddPhase::VerifyRed));
        assert!(TddPhase::VerifyRed.can_transition_to(&TddPhase::Implement));
        assert!(TddPhase::Implement.can_transition_to(&TddPhase::VerifyGreen));
        assert!(TddPhase::VerifyGreen.can_transition_to(&TddPhase::Refactor));
        assert!(TddPhase::Refactor.can_transition_to(&TddPhase::Complete));
    }

    #[test]
    fn test_can_transition_to_restart() {
        // Can always restart from WriteSpec
        assert!(TddPhase::WriteTests.can_transition_to(&TddPhase::WriteSpec));
        assert!(TddPhase::VerifyRed.can_transition_to(&TddPhase::WriteSpec));
        assert!(TddPhase::Implement.can_transition_to(&TddPhase::WriteSpec));
        assert!(TddPhase::VerifyGreen.can_transition_to(&TddPhase::WriteSpec));
        assert!(TddPhase::Refactor.can_transition_to(&TddPhase::WriteSpec));
        assert!(TddPhase::Complete.can_transition_to(&TddPhase::WriteSpec));
    }

    #[test]
    fn test_can_transition_to_iteration_backward() {
        // VerifyRed -> WriteTests (tests don't fail as expected)
        assert!(TddPhase::VerifyRed.can_transition_to(&TddPhase::WriteTests));
        // VerifyGreen -> Implement (tests still failing)
        assert!(TddPhase::VerifyGreen.can_transition_to(&TddPhase::Implement));
        // Refactor -> VerifyGreen (refactoring broke tests)
        assert!(TddPhase::Refactor.can_transition_to(&TddPhase::VerifyGreen));
        // Complete -> Refactor (additional cleanup)
        assert!(TddPhase::Complete.can_transition_to(&TddPhase::Refactor));
    }

    #[test]
    fn test_can_transition_to_invalid() {
        // Same phase is not a transition
        assert!(!TddPhase::WriteSpec.can_transition_to(&TddPhase::WriteSpec));
        // Cannot skip phases forward
        assert!(!TddPhase::WriteSpec.can_transition_to(&TddPhase::VerifyRed));
        assert!(!TddPhase::WriteTests.can_transition_to(&TddPhase::Implement));
        // Cannot go backward arbitrarily
        assert!(!TddPhase::VerifyGreen.can_transition_to(&TddPhase::WriteTests));
        assert!(!TddPhase::Refactor.can_transition_to(&TddPhase::Implement));
    }

    #[test]
    fn test_valid_transitions() {
        let transitions = TddPhase::WriteSpec.valid_transitions();
        assert_eq!(transitions, vec![TddPhase::WriteTests]);

        let transitions = TddPhase::VerifyRed.valid_transitions();
        assert!(transitions.contains(&TddPhase::WriteSpec));
        assert!(transitions.contains(&TddPhase::WriteTests));
        assert!(transitions.contains(&TddPhase::Implement));

        let transitions = TddPhase::Complete.valid_transitions();
        assert!(transitions.contains(&TddPhase::WriteSpec));
        assert!(transitions.contains(&TddPhase::Refactor));
    }

    #[test]
    fn test_skip_spec() {
        assert_eq!(TddPhase::WriteSpec.skip_spec(), Some(TddPhase::WriteTests));
        // Other phases behave like next()
        assert_eq!(TddPhase::WriteTests.skip_spec(), Some(TddPhase::VerifyRed));
    }

    #[test]
    fn test_skip_refactor() {
        assert_eq!(
            TddPhase::VerifyGreen.skip_refactor(),
            Some(TddPhase::Complete)
        );
        // Other phases behave like next()
        assert_eq!(
            TddPhase::Implement.skip_refactor(),
            Some(TddPhase::VerifyGreen)
        );
    }

    #[test]
    fn test_phase_is_terminal() {
        assert!(!TddPhase::WriteSpec.is_terminal());
        assert!(!TddPhase::Refactor.is_terminal());
        assert!(TddPhase::Complete.is_terminal());
    }

    #[test]
    fn test_phase_description() {
        assert!(TddPhase::WriteSpec.description().contains("specification"));
        assert!(TddPhase::VerifyRed.description().contains("red"));
        assert!(TddPhase::VerifyGreen.description().contains("green"));
    }

    #[test]
    fn test_phase_validation_requirements() {
        let validation = TddPhase::WriteSpec.validation_requirements();
        assert!(!validation.requires_spec);
        assert!(!validation.requires_tests);

        let validation = TddPhase::WriteTests.validation_requirements();
        assert!(validation.requires_spec);
        assert!(!validation.requires_tests);

        let validation = TddPhase::VerifyRed.validation_requirements();
        assert!(validation.requires_spec);
        assert!(validation.requires_tests);
        assert!(validation.requires_test_failure);
        assert!(!validation.requires_test_success);

        let validation = TddPhase::VerifyGreen.validation_requirements();
        assert!(validation.requires_tests);
        assert!(!validation.requires_test_failure);
        assert!(validation.requires_test_success);
        assert!(validation.requires_implementation);
    }

    // ==================== TddState Tests ====================

    #[test]
    fn test_state_new() {
        let state = TddState::new("test behavior", "/tmp/test");
        assert_eq!(state.phase, TddPhase::WriteSpec);
        assert_eq!(state.behavior, "test behavior");
        assert_eq!(state.iterations, 0);
        assert!(!state.skip_spec);
        assert!(!state.skip_refactor);
    }

    #[test]
    fn test_state_new_without_spec() {
        let state = TddState::new_without_spec("test behavior", "/tmp/test");
        assert_eq!(state.phase, TddPhase::WriteTests);
        assert!(state.skip_spec);
    }

    #[test]
    fn test_state_with_spec_file() {
        let state = TddState::new("test", "/tmp").with_spec_file("SPEC.md");
        assert_eq!(state.spec_file, Some("SPEC.md".to_string()));
    }

    #[test]
    fn test_state_advance() {
        let mut state = TddState::new("test", "/tmp");
        assert_eq!(state.advance(true, None), Some(TddPhase::WriteTests));
        assert_eq!(state.phase, TddPhase::WriteTests);
        assert_eq!(state.history.len(), 1);
    }

    #[test]
    fn test_state_full_cycle() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None); // WriteSpec -> WriteTests
        state.advance(true, None); // WriteTests -> VerifyRed
        state.advance(true, None); // VerifyRed -> Implement
        state.advance(true, None); // Implement -> VerifyGreen
        state.advance(true, None); // VerifyGreen -> Refactor
        state.advance(true, None); // Refactor -> Complete
        assert_eq!(state.phase, TddPhase::Complete);
        assert!(state.is_complete());
    }

    #[test]
    fn test_state_skip_spec() {
        let mut state = TddState::new("test", "/tmp").with_skip_spec();
        assert_eq!(state.phase, TddPhase::WriteTests);
        state.advance(true, None); // WriteTests -> VerifyRed
        assert_eq!(state.phase, TddPhase::VerifyRed);
    }

    #[test]
    fn test_state_skip_refactor() {
        let mut state = TddState::new_without_spec("test", "/tmp").with_skip_refactor();
        state.advance(true, None); // WriteTests -> VerifyRed
        state.advance(true, None); // VerifyRed -> Implement
        state.advance(true, None); // Implement -> VerifyGreen
        state.advance(true, None); // VerifyGreen -> Complete (skip Refactor)
        assert_eq!(state.phase, TddPhase::Complete);
    }

    #[test]
    fn test_state_transition_to() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None); // WriteSpec -> WriteTests
        state.advance(true, None); // WriteTests -> VerifyRed

        // Valid backward transition
        assert!(state.transition_to(TddPhase::WriteTests, Some("Tests not right".into())));
        assert_eq!(state.phase, TddPhase::WriteTests);

        // Invalid transition
        assert!(!state.transition_to(TddPhase::Implement, None));
        assert_eq!(state.phase, TddPhase::WriteTests);
    }

    #[test]
    fn test_state_retry_tests() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None); // WriteSpec -> WriteTests
        state.advance(true, None); // WriteTests -> VerifyRed

        state.retry_tests(Some("Tests passed unexpectedly".into()));
        assert_eq!(state.phase, TddPhase::WriteTests);
        assert_eq!(state.history.len(), 3);
    }

    #[test]
    fn test_state_retry_implement() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None); // WriteSpec -> WriteTests
        state.advance(true, None); // WriteTests -> VerifyRed
        state.advance(true, None); // VerifyRed -> Implement
        state.advance(true, None); // Implement -> VerifyGreen

        state.retry_implement(Some("Tests still failing".into()));
        assert_eq!(state.phase, TddPhase::Implement);
    }

    #[test]
    fn test_state_restart() {
        let mut state = TddState::new("test", "/tmp");
        state.advance(true, None);
        state.advance(true, None);
        state.advance(true, None);
        state.iterations = 2;

        state.restart(Some("Starting over".into()));
        assert_eq!(state.phase, TddPhase::WriteSpec);
        assert_eq!(state.iterations, 0);
    }

    #[test]
    fn test_state_restart_without_spec() {
        let mut state = TddState::new_without_spec("test", "/tmp");
        state.advance(true, None);
        state.advance(true, None);

        state.restart(Some("Starting over".into()));
        assert_eq!(state.phase, TddPhase::WriteTests);
    }

    #[test]
    fn test_state_max_iterations() {
        let mut state = TddState::new_without_spec("test", "/tmp").with_max_iterations(2);
        state.advance(true, None); // WriteTests -> VerifyRed
        state.advance(true, None); // VerifyRed -> Implement
        assert_eq!(state.iterations, 0);
        state.advance(true, None); // Implement -> VerifyGreen (iterations=1)
        assert_eq!(state.iterations, 1);
        assert!(!state.exceeded_max_iterations());

        // Retry and complete again
        state.retry_implement(None);
        state.advance(true, None); // Implement -> VerifyGreen (iterations=2)
        assert_eq!(state.iterations, 2);
        assert!(state.exceeded_max_iterations());
    }

    // ==================== TddWorkflow Tests ====================

    #[test]
    fn test_workflow_creation() {
        let workflow = TddWorkflow::new("Add login feature", "/tmp/project");
        assert_eq!(workflow.phase(), TddPhase::WriteSpec);
        assert!(!workflow.is_complete());
    }

    #[test]
    fn test_workflow_creation_without_spec() {
        let workflow = TddWorkflow::new_without_spec("Add login feature", "/tmp/project");
        assert_eq!(workflow.phase(), TddPhase::WriteTests);
    }

    #[test]
    fn test_workflow_prompts() {
        let workflow = TddWorkflow::new("Add login feature", "/tmp/project");
        let prompt = workflow.current_prompt();
        assert!(prompt.contains("Add login feature"));
        assert!(prompt.contains("specification"));
    }

    #[test]
    fn test_workflow_prompt_write_tests() {
        let mut workflow = TddWorkflow::new("test feature", "/tmp");
        workflow.advance(true, None); // WriteSpec -> WriteTests
        let prompt = workflow.current_prompt();
        assert!(prompt.contains("Write tests"));
    }

    #[test]
    fn test_workflow_prompt_verify_red() {
        let mut workflow = TddWorkflow::new_without_spec("test feature", "/tmp");
        workflow.advance(true, None); // WriteTests -> VerifyRed
        let prompt = workflow.current_prompt();
        assert!(prompt.contains("FAIL"));
    }

    #[test]
    fn test_workflow_prompt_implement() {
        let mut workflow = TddWorkflow::new_without_spec("test feature", "/tmp");
        workflow.advance(true, None); // WriteTests -> VerifyRed
        workflow.advance(true, None); // VerifyRed -> Implement
        let prompt = workflow.current_prompt();
        assert!(prompt.contains("MINIMAL"));
    }

    #[test]
    fn test_workflow_advance() {
        let mut workflow = TddWorkflow::new("test", "/tmp");
        workflow.advance(true, None);
        assert_eq!(workflow.phase(), TddPhase::WriteTests);

        let prompt = workflow.current_prompt();
        assert!(prompt.contains("tests"));
    }

    #[test]
    fn test_workflow_transition_to() {
        let mut workflow = TddWorkflow::new("test", "/tmp");
        workflow.advance(true, None); // WriteSpec -> WriteTests
        workflow.advance(true, None); // WriteTests -> VerifyRed

        assert!(workflow.transition_to(TddPhase::WriteTests, None));
        assert_eq!(workflow.phase(), TddPhase::WriteTests);
    }

    #[test]
    fn test_workflow_retry_tests() {
        let mut workflow = TddWorkflow::new_without_spec("test", "/tmp");
        workflow.advance(true, None); // WriteTests -> VerifyRed

        workflow.retry_tests(None);
        assert_eq!(workflow.phase(), TddPhase::WriteTests);
    }

    #[test]
    fn test_workflow_retry_implement() {
        let mut workflow = TddWorkflow::new_without_spec("test", "/tmp");
        workflow.advance(true, None); // WriteTests -> VerifyRed
        workflow.advance(true, None); // VerifyRed -> Implement
        workflow.advance(true, None); // Implement -> VerifyGreen

        workflow.retry_implement(None);
        assert_eq!(workflow.phase(), TddPhase::Implement);
    }

    #[test]
    fn test_workflow_restart() {
        let mut workflow = TddWorkflow::new("test", "/tmp");
        workflow.advance(true, None);
        workflow.advance(true, None);

        workflow.restart(None);
        assert_eq!(workflow.phase(), TddPhase::WriteSpec);
    }

    #[test]
    fn test_workflow_complete() {
        let mut workflow = TddWorkflow::new_without_spec("test", "/tmp");
        workflow.state_mut().skip_refactor = true;
        workflow.advance(true, None); // WriteTests -> VerifyRed
        workflow.advance(true, None); // VerifyRed -> Implement
        workflow.advance(true, None); // Implement -> VerifyGreen
        workflow.advance(true, None); // VerifyGreen -> Complete
        assert!(workflow.is_complete());
    }

    #[test]
    fn test_workflow_validation() {
        let workflow = TddWorkflow::new("test", "/tmp");
        let validation = workflow.current_validation();
        assert!(!validation.requires_spec);

        let workflow = TddWorkflow::new_without_spec("test", "/tmp");
        let validation = workflow.current_validation();
        assert!(validation.requires_spec);
    }

    #[test]
    fn test_workflow_valid_transitions() {
        let workflow = TddWorkflow::new("test", "/tmp");
        let transitions = workflow.valid_transitions();
        assert_eq!(transitions, vec![TddPhase::WriteTests]);
    }
}
