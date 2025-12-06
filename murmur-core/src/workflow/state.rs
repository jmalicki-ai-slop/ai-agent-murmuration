//! Core workflow state machine
//!
//! This module provides a generic state machine for workflow management,
//! supporting phases, transitions, and validation.

use crate::error::{Error, Result};
use std::fmt::Debug;

/// Validation result for a workflow phase
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhaseValidation {
    /// Phase is valid and ready to proceed
    Valid,
    /// Phase is invalid and cannot proceed
    Invalid { reason: String },
    /// Phase needs work before proceeding
    NeedsWork { feedback: String },
}

impl PhaseValidation {
    /// Check if the validation is successful (Valid)
    pub fn is_valid(&self) -> bool {
        matches!(self, PhaseValidation::Valid)
    }

    /// Check if the validation indicates work is needed
    pub fn needs_work(&self) -> bool {
        matches!(self, PhaseValidation::NeedsWork { .. })
    }

    /// Check if the validation is invalid
    pub fn is_invalid(&self) -> bool {
        matches!(self, PhaseValidation::Invalid { .. })
    }

    /// Get the reason/feedback message if available
    pub fn message(&self) -> Option<&str> {
        match self {
            PhaseValidation::Valid => None,
            PhaseValidation::Invalid { reason } => Some(reason),
            PhaseValidation::NeedsWork { feedback } => Some(feedback),
        }
    }
}

/// Core workflow trait for state machine management
///
/// This trait defines the interface for workflow state machines,
/// allowing different workflow types (TDD, review, coordinator, etc.)
/// to implement their own phase logic while sharing common state management.
pub trait Workflow {
    /// The phase type for this workflow
    type Phase: Clone + PartialEq + Debug;

    /// Get the current phase of the workflow
    fn current_phase(&self) -> &Self::Phase;

    /// Check if a transition to the given phase is valid
    fn can_transition_to(&self, phase: &Self::Phase) -> bool;

    /// Attempt to transition to a new phase
    ///
    /// Returns an error if the transition is not valid.
    /// Implementations should log the transition.
    fn transition_to(&mut self, phase: Self::Phase) -> Result<()>;

    /// Validate the current phase
    ///
    /// This allows workflows to implement custom validation logic
    /// to determine if the current phase is complete and ready to proceed.
    fn validate_phase(&self) -> Result<PhaseValidation>;
}

/// A basic state machine implementation
///
/// This provides a simple implementation of the Workflow trait
/// for use cases that need basic state tracking without complex logic.
#[derive(Debug, Clone)]
pub struct StateMachine<P: Clone + PartialEq + Debug> {
    current_phase: P,
    valid_transitions: Vec<(P, P)>,
}

impl<P: Clone + PartialEq + Debug> StateMachine<P> {
    /// Create a new state machine with the given initial phase
    pub fn new(initial_phase: P) -> Self {
        Self {
            current_phase: initial_phase,
            valid_transitions: Vec::new(),
        }
    }

    /// Add a valid transition from one phase to another
    pub fn add_transition(mut self, from: P, to: P) -> Self {
        self.valid_transitions.push((from, to));
        self
    }

    /// Add multiple valid transitions
    pub fn add_transitions(mut self, transitions: Vec<(P, P)>) -> Self {
        self.valid_transitions.extend(transitions);
        self
    }

    /// Check if a transition is in the valid transitions list
    fn is_valid_transition(&self, from: &P, to: &P) -> bool {
        self.valid_transitions
            .iter()
            .any(|(f, t)| f == from && t == to)
    }
}

impl<P: Clone + PartialEq + Debug> Workflow for StateMachine<P> {
    type Phase = P;

    fn current_phase(&self) -> &Self::Phase {
        &self.current_phase
    }

    fn can_transition_to(&self, phase: &Self::Phase) -> bool {
        self.is_valid_transition(&self.current_phase, phase)
    }

    fn transition_to(&mut self, phase: Self::Phase) -> Result<()> {
        if !self.can_transition_to(&phase) {
            return Err(Error::Other(format!(
                "Invalid transition from {:?} to {:?}",
                self.current_phase, phase
            )));
        }

        tracing::info!(
            from = ?self.current_phase,
            to = ?phase,
            "Workflow phase transition"
        );

        self.current_phase = phase;
        Ok(())
    }

    fn validate_phase(&self) -> Result<PhaseValidation> {
        // Basic implementation always returns Valid
        // Specific workflows should override this
        Ok(PhaseValidation::Valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestPhase {
        Start,
        Middle,
        End,
    }

    #[test]
    fn test_phase_validation_methods() {
        let valid = PhaseValidation::Valid;
        assert!(valid.is_valid());
        assert!(!valid.needs_work());
        assert!(!valid.is_invalid());
        assert_eq!(valid.message(), None);

        let invalid = PhaseValidation::Invalid {
            reason: "test error".to_string(),
        };
        assert!(!invalid.is_valid());
        assert!(!invalid.needs_work());
        assert!(invalid.is_invalid());
        assert_eq!(invalid.message(), Some("test error"));

        let needs_work = PhaseValidation::NeedsWork {
            feedback: "needs fixing".to_string(),
        };
        assert!(!needs_work.is_valid());
        assert!(needs_work.needs_work());
        assert!(!needs_work.is_invalid());
        assert_eq!(needs_work.message(), Some("needs fixing"));
    }

    #[test]
    fn test_state_machine_creation() {
        let sm = StateMachine::new(TestPhase::Start);
        assert_eq!(sm.current_phase(), &TestPhase::Start);
    }

    #[test]
    fn test_valid_transition() {
        let mut sm = StateMachine::new(TestPhase::Start)
            .add_transition(TestPhase::Start, TestPhase::Middle)
            .add_transition(TestPhase::Middle, TestPhase::End);

        assert!(sm.can_transition_to(&TestPhase::Middle));
        assert!(!sm.can_transition_to(&TestPhase::End));

        assert!(sm.transition_to(TestPhase::Middle).is_ok());
        assert_eq!(sm.current_phase(), &TestPhase::Middle);

        assert!(sm.can_transition_to(&TestPhase::End));
        assert!(!sm.can_transition_to(&TestPhase::Start));
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm =
            StateMachine::new(TestPhase::Start).add_transition(TestPhase::Start, TestPhase::Middle);

        let result = sm.transition_to(TestPhase::End);
        assert!(result.is_err());
        assert_eq!(sm.current_phase(), &TestPhase::Start);
    }

    #[test]
    fn test_add_transitions_bulk() {
        let transitions = vec![
            (TestPhase::Start, TestPhase::Middle),
            (TestPhase::Middle, TestPhase::End),
            (TestPhase::End, TestPhase::Start),
        ];

        let sm = StateMachine::new(TestPhase::Start).add_transitions(transitions);

        assert!(sm.can_transition_to(&TestPhase::Middle));
    }

    #[test]
    fn test_default_validation() {
        let sm = StateMachine::new(TestPhase::Start);
        let validation = sm.validate_phase().unwrap();
        assert!(validation.is_valid());
    }
}
