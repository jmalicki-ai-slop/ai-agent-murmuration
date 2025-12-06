//! Phase transition logic with test-based validation
//!
//! This module provides async validation for TDD phase transitions,
//! integrating with the TestRunner to enforce red/green requirements.

use super::tdd::{TddPhase, TddState, TddWorkflow};
use super::test_runner::{TestResults, TestRunner};
use crate::error::Result;
use std::path::Path;

/// Result of attempting a phase transition
#[derive(Debug, Clone)]
pub enum TransitionResult {
    /// Transition is allowed to proceed
    Allowed,
    /// Transition is blocked due to validation failure
    Blocked {
        /// Reason why the transition is blocked
        reason: String,
        /// Suggestion for how to fix the issue
        suggestion: String,
    },
    /// Transition completed successfully
    Completed {
        /// The phase we transitioned to
        new_phase: TddPhase,
        /// Optional message about the transition
        message: Option<String>,
    },
}

impl TransitionResult {
    /// Check if the transition is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            TransitionResult::Allowed | TransitionResult::Completed { .. }
        )
    }

    /// Check if the transition is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, TransitionResult::Blocked { .. })
    }

    /// Get the blocking reason if blocked
    pub fn blocking_reason(&self) -> Option<&str> {
        match self {
            TransitionResult::Blocked { reason, .. } => Some(reason),
            _ => None,
        }
    }

    /// Get the suggestion if blocked
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            TransitionResult::Blocked { suggestion, .. } => Some(suggestion),
            _ => None,
        }
    }
}

/// Extension trait for TddWorkflow to add async validation
#[allow(async_fn_in_trait)]
pub trait TddTransitionValidator {
    /// Validate that a transition to the target phase is allowed
    ///
    /// This performs the actual test execution for VerifyRed and VerifyGreen phases
    /// to ensure the TDD invariants are maintained.
    fn validate_transition(
        &self,
        to: TddPhase,
        test_results: Option<&TestResults>,
    ) -> TransitionResult;

    /// Run tests and validate transition based on results
    async fn validate_transition_with_tests(&self, to: TddPhase) -> Result<TransitionResult>;

    /// Advance to the next phase with validation
    ///
    /// This is a convenience method that validates the transition to the next phase
    /// and advances if allowed.
    async fn validated_advance(&mut self) -> Result<TransitionResult>;
}

impl TddTransitionValidator for TddWorkflow {
    fn validate_transition(
        &self,
        to: TddPhase,
        test_results: Option<&TestResults>,
    ) -> TransitionResult {
        let from = self.phase();

        // Check if the transition is structurally allowed
        if !from.can_transition_to(&to) {
            return TransitionResult::Blocked {
                reason: format!("Cannot transition from {:?} to {:?}", from, to),
                suggestion: format!(
                    "Valid transitions from {:?}: {:?}",
                    from,
                    from.valid_transitions()
                ),
            };
        }

        // For VerifyRed and VerifyGreen, we need test results
        let validation = to.validation_requirements();

        if validation.requires_test_failure {
            match test_results {
                Some(results) => {
                    if results.is_green() {
                        return TransitionResult::Blocked {
                            reason: "Tests should FAIL in red phase but all pass".to_string(),
                            suggestion:
                                "Write tests that verify unimplemented behavior, or ensure tests target the new functionality"
                                    .to_string(),
                        };
                    }
                    if results.execution_error.is_some() {
                        return TransitionResult::Blocked {
                            reason: format!(
                                "Tests have execution errors: {}",
                                results.execution_error.as_ref().unwrap()
                            ),
                            suggestion: "Fix test setup errors before proceeding".to_string(),
                        };
                    }
                }
                None => {
                    return TransitionResult::Blocked {
                        reason: "Test results required to verify red phase".to_string(),
                        suggestion: "Run tests first to verify they fail".to_string(),
                    };
                }
            }
        }

        if validation.requires_test_success {
            match test_results {
                Some(results) => {
                    if results.is_red() {
                        return TransitionResult::Blocked {
                            reason: format!("{} tests still failing", results.failed),
                            suggestion: "Fix implementation to make tests pass".to_string(),
                        };
                    }
                    if results.execution_error.is_some() {
                        return TransitionResult::Blocked {
                            reason: format!(
                                "Tests have execution errors: {}",
                                results.execution_error.as_ref().unwrap()
                            ),
                            suggestion: "Fix test or implementation errors before proceeding"
                                .to_string(),
                        };
                    }
                    if results.passed == 0 {
                        return TransitionResult::Blocked {
                            reason: "No tests passed".to_string(),
                            suggestion: "Ensure there are tests to run and they're being executed"
                                .to_string(),
                        };
                    }
                }
                None => {
                    return TransitionResult::Blocked {
                        reason: "Test results required to verify green phase".to_string(),
                        suggestion: "Run tests first to verify they pass".to_string(),
                    };
                }
            }
        }

        TransitionResult::Allowed
    }

    async fn validate_transition_with_tests(&self, to: TddPhase) -> Result<TransitionResult> {
        let validation = to.validation_requirements();

        // Only run tests if the target phase requires them
        if validation.requires_test_failure || validation.requires_test_success {
            let workdir = &self.state().workdir;
            let runner = TestRunner::new(workdir);
            let results = runner.run();
            Ok(self.validate_transition(to, Some(&results)))
        } else {
            Ok(self.validate_transition(to, None))
        }
    }

    async fn validated_advance(&mut self) -> Result<TransitionResult> {
        let next_phase = match self.phase().next() {
            Some(phase) => phase,
            None => {
                return Ok(TransitionResult::Blocked {
                    reason: "Already at terminal phase".to_string(),
                    suggestion: "Workflow is complete".to_string(),
                });
            }
        };

        // Handle skip configurations
        let target_phase = match self.phase() {
            TddPhase::WriteSpec if self.state().skip_spec => TddPhase::WriteTests,
            TddPhase::VerifyGreen if self.state().skip_refactor => TddPhase::Complete,
            _ => next_phase,
        };

        let result = self.validate_transition_with_tests(target_phase).await?;

        match &result {
            TransitionResult::Allowed => {
                self.advance(true, None);
                Ok(TransitionResult::Completed {
                    new_phase: self.phase(),
                    message: None,
                })
            }
            TransitionResult::Blocked { reason, .. } => {
                // Record the failed attempt in history
                // Store phase before mutable borrow
                let from_phase = self.phase();
                self.state_mut().history.push(super::tdd::TddTransition {
                    from: from_phase,
                    to: target_phase,
                    success: false,
                    message: Some(reason.clone()),
                });
                Ok(result)
            }
            TransitionResult::Completed { .. } => Ok(result),
        }
    }
}

/// Helper functions for running tests and validating phases
pub struct PhaseValidator<'a> {
    workdir: &'a Path,
    test_runner: TestRunner,
}

impl<'a> PhaseValidator<'a> {
    /// Create a new phase validator for the given working directory
    pub fn new(workdir: &'a Path) -> Self {
        Self {
            workdir,
            test_runner: TestRunner::new(workdir),
        }
    }

    /// Run tests and return the results
    pub fn run_tests(&self) -> TestResults {
        self.test_runner.run()
    }

    /// Validate that we're in a valid red state (tests fail)
    pub fn validate_red(&self) -> TransitionResult {
        let results = self.run_tests();

        if results.execution_error.is_some() {
            return TransitionResult::Blocked {
                reason: format!(
                    "Tests have execution errors: {}",
                    results.execution_error.as_ref().unwrap()
                ),
                suggestion: "Fix test setup errors before proceeding".to_string(),
            };
        }

        if results.is_green() {
            return TransitionResult::Blocked {
                reason: "Tests should FAIL in red phase but all pass".to_string(),
                suggestion:
                    "Write tests that verify unimplemented behavior, or ensure tests target the new functionality"
                        .to_string(),
            };
        }

        TransitionResult::Allowed
    }

    /// Validate that we're in a valid green state (tests pass)
    pub fn validate_green(&self) -> TransitionResult {
        let results = self.run_tests();

        if results.execution_error.is_some() {
            return TransitionResult::Blocked {
                reason: format!(
                    "Tests have execution errors: {}",
                    results.execution_error.as_ref().unwrap()
                ),
                suggestion: "Fix test or implementation errors before proceeding".to_string(),
            };
        }

        if results.is_red() {
            return TransitionResult::Blocked {
                reason: format!("{} tests still failing", results.failed),
                suggestion: "Fix implementation to make tests pass".to_string(),
            };
        }

        if results.passed == 0 {
            return TransitionResult::Blocked {
                reason: "No tests passed".to_string(),
                suggestion: "Ensure there are tests to run and they're being executed".to_string(),
            };
        }

        TransitionResult::Allowed
    }

    /// Get the working directory
    pub fn workdir(&self) -> &Path {
        self.workdir
    }
}

/// Retry helper for TDD iterations
pub struct TddIterator<'a> {
    state: &'a mut TddState,
    max_retries: u32,
    current_retries: u32,
}

impl<'a> TddIterator<'a> {
    /// Create a new TDD iterator
    pub fn new(state: &'a mut TddState) -> Self {
        let max_retries = state.max_iterations;
        Self {
            state,
            max_retries,
            current_retries: 0,
        }
    }

    /// Record a retry attempt
    pub fn record_retry(&mut self, phase: TddPhase, reason: &str) -> bool {
        self.current_retries += 1;
        if self.current_retries > self.max_retries {
            return false;
        }

        match phase {
            TddPhase::VerifyRed => {
                self.state.retry_tests(Some(reason.to_string()));
            }
            TddPhase::VerifyGreen => {
                self.state.retry_implement(Some(reason.to_string()));
            }
            _ => {
                // Other phases don't have specific retry methods
                return false;
            }
        }
        true
    }

    /// Check if we've exceeded the retry limit
    pub fn exceeded_limit(&self) -> bool {
        self.current_retries >= self.max_retries
    }

    /// Get the current retry count
    pub fn retry_count(&self) -> u32 {
        self.current_retries
    }

    /// Get the max retries
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mock_passing_results() -> TestResults {
        TestResults {
            passed: 5,
            failed: 0,
            skipped: 0,
            duration_ms: 100,
            output: "5 passed".to_string(),
            execution_error: None,
        }
    }

    fn mock_failing_results() -> TestResults {
        TestResults {
            passed: 3,
            failed: 2,
            skipped: 0,
            duration_ms: 100,
            output: "3 passed, 2 failed".to_string(),
            execution_error: None,
        }
    }

    fn mock_error_results() -> TestResults {
        TestResults {
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            output: String::new(),
            execution_error: Some("Test execution failed".to_string()),
        }
    }

    #[test]
    fn test_transition_result_allowed() {
        let result = TransitionResult::Allowed;
        assert!(result.is_allowed());
        assert!(!result.is_blocked());
        assert!(result.blocking_reason().is_none());
    }

    #[test]
    fn test_transition_result_blocked() {
        let result = TransitionResult::Blocked {
            reason: "Tests pass".to_string(),
            suggestion: "Write failing tests".to_string(),
        };
        assert!(!result.is_allowed());
        assert!(result.is_blocked());
        assert_eq!(result.blocking_reason(), Some("Tests pass"));
        assert_eq!(result.suggestion(), Some("Write failing tests"));
    }

    #[test]
    fn test_transition_result_completed() {
        let result = TransitionResult::Completed {
            new_phase: TddPhase::WriteTests,
            message: Some("Advanced".to_string()),
        };
        assert!(result.is_allowed());
        assert!(!result.is_blocked());
    }

    #[test]
    fn test_validate_transition_verify_red_with_passing_tests() {
        let workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        let results = mock_passing_results();
        let result = workflow.validate_transition(TddPhase::VerifyRed, Some(&results));
        assert!(result.is_blocked());
        assert!(result
            .blocking_reason()
            .unwrap()
            .contains("FAIL in red phase"));
    }

    #[test]
    fn test_validate_transition_verify_red_with_failing_tests() {
        let mut workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        workflow.advance(true, None); // WriteTests -> VerifyRed
        let results = mock_failing_results();
        let result = workflow.validate_transition(TddPhase::Implement, Some(&results));
        assert!(result.is_allowed());
    }

    #[test]
    fn test_validate_transition_verify_green_with_failing_tests() {
        let mut workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        workflow.advance(true, None); // WriteTests -> VerifyRed
        workflow.advance(true, None); // VerifyRed -> Implement
        workflow.advance(true, None); // Implement -> VerifyGreen
        let results = mock_failing_results();
        let result = workflow.validate_transition(TddPhase::Refactor, Some(&results));
        assert!(result.is_blocked());
        assert!(result.blocking_reason().unwrap().contains("still failing"));
    }

    #[test]
    fn test_validate_transition_verify_green_with_passing_tests() {
        let mut workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        workflow.advance(true, None); // WriteTests -> VerifyRed
        workflow.advance(true, None); // VerifyRed -> Implement
        workflow.advance(true, None); // Implement -> VerifyGreen
        let results = mock_passing_results();
        let result = workflow.validate_transition(TddPhase::Refactor, Some(&results));
        assert!(result.is_allowed());
    }

    #[test]
    fn test_validate_transition_with_execution_error() {
        let workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        let results = mock_error_results();
        let result = workflow.validate_transition(TddPhase::VerifyRed, Some(&results));
        assert!(result.is_blocked());
        assert!(result
            .blocking_reason()
            .unwrap()
            .contains("execution error"));
    }

    #[test]
    fn test_validate_transition_without_results_for_verify_phase() {
        let workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        let result = workflow.validate_transition(TddPhase::VerifyRed, None);
        assert!(result.is_blocked());
        assert!(result.blocking_reason().unwrap().contains("required"));
    }

    #[test]
    fn test_validate_transition_invalid_transition() {
        let workflow = TddWorkflow::new("test", PathBuf::from("/tmp"));
        // Can't skip from WriteSpec to Implement
        let result = workflow.validate_transition(TddPhase::Implement, None);
        assert!(result.is_blocked());
        assert!(result
            .blocking_reason()
            .unwrap()
            .contains("Cannot transition"));
    }

    #[test]
    fn test_validate_transition_no_tests_passed() {
        let mut workflow = TddWorkflow::new_without_spec("test", PathBuf::from("/tmp"));
        workflow.advance(true, None); // WriteTests -> VerifyRed
        workflow.advance(true, None); // VerifyRed -> Implement
        workflow.advance(true, None); // Implement -> VerifyGreen
        let results = TestResults {
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 100,
            output: String::new(),
            execution_error: None,
        };
        let result = workflow.validate_transition(TddPhase::Refactor, Some(&results));
        assert!(result.is_blocked());
        assert!(result
            .blocking_reason()
            .unwrap()
            .contains("No tests passed"));
    }

    #[test]
    fn test_tdd_iterator() {
        let mut state = TddState::new_without_spec("test", PathBuf::from("/tmp"));
        state.advance(true, None); // WriteTests -> VerifyRed

        let mut iterator = TddIterator::new(&mut state);
        assert!(!iterator.exceeded_limit());
        assert_eq!(iterator.retry_count(), 0);

        // Record retries up to limit
        assert!(iterator.record_retry(TddPhase::VerifyRed, "Tests passed"));
        assert!(iterator.record_retry(TddPhase::VerifyRed, "Tests passed"));
        assert!(iterator.record_retry(TddPhase::VerifyRed, "Tests passed"));

        // Should hit the limit (default is 3)
        assert!(iterator.exceeded_limit());
        assert!(!iterator.record_retry(TddPhase::VerifyRed, "Tests passed"));
    }
}
