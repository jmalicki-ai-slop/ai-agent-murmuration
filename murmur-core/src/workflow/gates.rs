//! Phase gates with review requirements
//!
//! This module provides review gates between TDD phases. Each gate requires
//! a successful review before allowing the workflow to proceed to the next phase.
//!
//! Gates are positioned between TDD phases:
//! - After WriteSpec -> Spec review
//! - After WriteTests -> Test review
//! - After Implement -> Code review
//! - After Refactor -> Final review

use crate::review::{
    FeedbackParser, ReviewFeedback, ReviewRequest, ReviewType, Reviewer, ReviewerConfig,
};
use crate::Result;

use super::tdd::{TddPhase, TddState};

/// Which TDD phases require a review gate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewGate {
    /// Review the specification before writing tests
    AfterSpec,
    /// Review tests before running VerifyRed
    AfterTests,
    /// Review implementation before VerifyGreen
    AfterImplement,
    /// Final review before completing the workflow
    BeforeComplete,
}

impl ReviewGate {
    /// Get the TDD phase this gate is positioned after
    pub fn after_phase(&self) -> TddPhase {
        match self {
            ReviewGate::AfterSpec => TddPhase::WriteSpec,
            ReviewGate::AfterTests => TddPhase::WriteTests,
            ReviewGate::AfterImplement => TddPhase::Implement,
            ReviewGate::BeforeComplete => TddPhase::Refactor,
        }
    }

    /// Get the TDD phase this gate is positioned before
    pub fn before_phase(&self) -> TddPhase {
        match self {
            ReviewGate::AfterSpec => TddPhase::WriteTests,
            ReviewGate::AfterTests => TddPhase::VerifyRed,
            ReviewGate::AfterImplement => TddPhase::VerifyGreen,
            ReviewGate::BeforeComplete => TddPhase::Complete,
        }
    }

    /// Get the corresponding review type for this gate
    pub fn review_type(&self) -> ReviewType {
        match self {
            ReviewGate::AfterSpec => ReviewType::Spec,
            ReviewGate::AfterTests => ReviewType::Test,
            ReviewGate::AfterImplement => ReviewType::Code,
            ReviewGate::BeforeComplete => ReviewType::Final,
        }
    }

    /// Get a human-readable description of this gate
    pub fn description(&self) -> &'static str {
        match self {
            ReviewGate::AfterSpec => "Specification review gate",
            ReviewGate::AfterTests => "Test review gate",
            ReviewGate::AfterImplement => "Code review gate",
            ReviewGate::BeforeComplete => "Final review gate",
        }
    }

    /// Get all review gates in order
    pub fn all() -> &'static [ReviewGate] {
        &[
            ReviewGate::AfterSpec,
            ReviewGate::AfterTests,
            ReviewGate::AfterImplement,
            ReviewGate::BeforeComplete,
        ]
    }

    /// Get the gate that should be checked when transitioning from the given phase
    pub fn for_transition_from(phase: TddPhase) -> Option<ReviewGate> {
        match phase {
            TddPhase::WriteSpec => Some(ReviewGate::AfterSpec),
            TddPhase::WriteTests => Some(ReviewGate::AfterTests),
            TddPhase::Implement => Some(ReviewGate::AfterImplement),
            TddPhase::Refactor => Some(ReviewGate::BeforeComplete),
            _ => None,
        }
    }
}

impl std::fmt::Display for ReviewGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Configuration for phase gates
#[derive(Debug, Clone)]
pub struct GateConfig {
    /// Whether spec review is required
    pub require_spec_review: bool,
    /// Whether test review is required
    pub require_test_review: bool,
    /// Whether code review is required
    pub require_code_review: bool,
    /// Whether final review is required
    pub require_final_review: bool,
    /// Maximum review iterations per gate
    pub max_iterations: u32,
    /// Reviewer configuration
    pub reviewer_config: ReviewerConfig,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            require_spec_review: true,
            require_test_review: true,
            require_code_review: true,
            require_final_review: true,
            max_iterations: 3,
            reviewer_config: ReviewerConfig::default(),
        }
    }
}

impl GateConfig {
    /// Create a new gate configuration with all gates enabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a minimal configuration with only code review
    pub fn code_review_only() -> Self {
        Self {
            require_spec_review: false,
            require_test_review: false,
            require_code_review: true,
            require_final_review: false,
            ..Default::default()
        }
    }

    /// Create a configuration with no review gates
    pub fn no_reviews() -> Self {
        Self {
            require_spec_review: false,
            require_test_review: false,
            require_code_review: false,
            require_final_review: false,
            ..Default::default()
        }
    }

    /// Check if a specific gate is required
    pub fn is_required(&self, gate: ReviewGate) -> bool {
        match gate {
            ReviewGate::AfterSpec => self.require_spec_review,
            ReviewGate::AfterTests => self.require_test_review,
            ReviewGate::AfterImplement => self.require_code_review,
            ReviewGate::BeforeComplete => self.require_final_review,
        }
    }

    /// Set spec review requirement
    pub fn with_spec_review(mut self, required: bool) -> Self {
        self.require_spec_review = required;
        self
    }

    /// Set test review requirement
    pub fn with_test_review(mut self, required: bool) -> Self {
        self.require_test_review = required;
        self
    }

    /// Set code review requirement
    pub fn with_code_review(mut self, required: bool) -> Self {
        self.require_code_review = required;
        self
    }

    /// Set final review requirement
    pub fn with_final_review(mut self, required: bool) -> Self {
        self.require_final_review = required;
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set reviewer configuration
    pub fn with_reviewer_config(mut self, config: ReviewerConfig) -> Self {
        self.reviewer_config = config;
        self
    }
}

/// Result of a gate check
#[derive(Debug, Clone)]
pub enum GateResult {
    /// Gate passed, can proceed
    Passed {
        /// The gate that was checked
        gate: ReviewGate,
        /// The review feedback
        feedback: ReviewFeedback,
    },
    /// Gate failed, cannot proceed
    Failed {
        /// The gate that was checked
        gate: ReviewGate,
        /// The review feedback with blocking issues
        feedback: ReviewFeedback,
        /// Formatted feedback string for the coder
        feedback_for_coder: String,
    },
    /// Gate is not required, can proceed
    Skipped {
        /// The gate that was skipped
        gate: ReviewGate,
    },
    /// Maximum iterations exceeded
    MaxIterationsExceeded {
        /// The gate that exceeded iterations
        gate: ReviewGate,
        /// Number of iterations completed
        iterations: u32,
    },
}

impl GateResult {
    /// Check if the gate allows proceeding
    pub fn can_proceed(&self) -> bool {
        matches!(self, GateResult::Passed { .. } | GateResult::Skipped { .. })
    }

    /// Check if the gate failed
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            GateResult::Failed { .. } | GateResult::MaxIterationsExceeded { .. }
        )
    }

    /// Get the gate that was checked
    pub fn gate(&self) -> ReviewGate {
        match self {
            GateResult::Passed { gate, .. } => *gate,
            GateResult::Failed { gate, .. } => *gate,
            GateResult::Skipped { gate } => *gate,
            GateResult::MaxIterationsExceeded { gate, .. } => *gate,
        }
    }

    /// Get the review feedback if available
    pub fn feedback(&self) -> Option<&ReviewFeedback> {
        match self {
            GateResult::Passed { feedback, .. } => Some(feedback),
            GateResult::Failed { feedback, .. } => Some(feedback),
            _ => None,
        }
    }

    /// Get feedback formatted for the coder to address
    pub fn feedback_for_coder(&self) -> Option<&str> {
        match self {
            GateResult::Failed {
                feedback_for_coder, ..
            } => Some(feedback_for_coder),
            _ => None,
        }
    }
}

/// Gate state tracking for a single gate
#[derive(Debug, Clone, Default)]
pub struct GateState {
    /// Number of review iterations completed
    pub iterations: u32,
    /// Previous feedback (for re-reviews)
    pub previous_feedback: Option<String>,
    /// Whether the gate has been passed
    pub passed: bool,
    /// All feedback received at this gate
    pub feedback_history: Vec<ReviewFeedback>,
}

impl GateState {
    /// Create a new gate state
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a review result
    pub fn record_review(&mut self, feedback: ReviewFeedback) {
        self.iterations += 1;
        self.previous_feedback = Some(feedback.to_feedback_string());
        self.passed = feedback.can_proceed();
        self.feedback_history.push(feedback);
    }

    /// Reset the gate state
    pub fn reset(&mut self) {
        self.iterations = 0;
        self.previous_feedback = None;
        self.passed = false;
        self.feedback_history.clear();
    }
}

/// Phase gate manager for TDD workflows
///
/// This manages the review gates between TDD phases, tracking state
/// and coordinating with the reviewer agent.
#[derive(Debug)]
pub struct PhaseGateManager {
    /// Gate configuration
    config: GateConfig,
    /// State for each gate
    gate_states: std::collections::HashMap<ReviewGate, GateState>,
    /// The reviewer to use
    reviewer: Reviewer,
    /// Feedback parser
    parser: FeedbackParser,
}

impl Default for PhaseGateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseGateManager {
    /// Create a new phase gate manager with default configuration
    pub fn new() -> Self {
        Self {
            config: GateConfig::default(),
            gate_states: std::collections::HashMap::new(),
            reviewer: Reviewer::new(),
            parser: FeedbackParser::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: GateConfig) -> Self {
        let reviewer = Reviewer::with_config(config.reviewer_config.clone());
        Self {
            config,
            gate_states: std::collections::HashMap::new(),
            reviewer,
            parser: FeedbackParser::new(),
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &GateConfig {
        &self.config
    }

    /// Get the state for a specific gate
    pub fn gate_state(&self, gate: ReviewGate) -> Option<&GateState> {
        self.gate_states.get(&gate)
    }

    /// Get mutable state for a specific gate
    pub fn gate_state_mut(&mut self, gate: ReviewGate) -> &mut GateState {
        self.gate_states.entry(gate).or_default()
    }

    /// Check if a gate is required based on configuration
    pub fn is_gate_required(&self, gate: ReviewGate) -> bool {
        self.config.is_required(gate)
    }

    /// Check if a gate has been passed
    pub fn is_gate_passed(&self, gate: ReviewGate) -> bool {
        self.gate_states
            .get(&gate)
            .map(|s| s.passed)
            .unwrap_or(false)
    }

    /// Get the gate that should be checked for a phase transition
    pub fn gate_for_transition(&self, from: TddPhase) -> Option<ReviewGate> {
        let gate = ReviewGate::for_transition_from(from)?;
        if self.config.is_required(gate) {
            Some(gate)
        } else {
            None
        }
    }

    /// Check if transition is allowed (gate passed or not required)
    pub fn can_transition(&self, from: TddPhase) -> bool {
        match ReviewGate::for_transition_from(from) {
            Some(gate) => !self.config.is_required(gate) || self.is_gate_passed(gate),
            None => true,
        }
    }

    /// Build a review request for a gate
    pub fn build_review_request(
        &self,
        gate: ReviewGate,
        state: &TddState,
        diff: Option<&str>,
    ) -> ReviewRequest {
        let review_type = gate.review_type();
        let gate_state = self.gate_states.get(&gate);

        let mut builder = ReviewRequest::builder(review_type, &state.workdir)
            .task(&state.behavior)
            .max_iterations(self.config.max_iterations);

        // Add iteration info if this is a re-review
        if let Some(gs) = gate_state {
            if gs.iterations > 0 {
                builder = builder.iteration(gs.iterations);
            }
            if let Some(ref prev) = gs.previous_feedback {
                builder = builder.previous_feedback(prev.clone());
            }
        }

        // Add spec content for spec reviews
        if let Some(ref spec_file) = state.spec_file {
            builder = builder.spec(spec_file.clone());
        }

        // Add test files for test reviews
        if !state.test_files.is_empty() {
            builder = builder.test_files(state.test_files.clone());
        }

        // Add impl files for code reviews
        if !state.impl_files.is_empty() {
            builder = builder.impl_files(state.impl_files.clone());
        }

        // Add diff if provided
        if let Some(diff_content) = diff {
            builder = builder.diff(diff_content);
        }

        builder.build()
    }

    /// Invoke a gate check with the reviewer agent
    ///
    /// This spawns the reviewer agent and parses the feedback.
    /// Returns a GateResult indicating whether the gate passed.
    ///
    /// Note: This method spawns the reviewer agent and waits for it to complete,
    /// collecting all stdout output for feedback parsing. For streaming output
    /// or custom output handling, use `invoke_gate()` followed by
    /// `check_gate_with_output()`.
    pub async fn check_gate(
        &mut self,
        gate: ReviewGate,
        state: &TddState,
        diff: Option<&str>,
    ) -> Result<GateResult> {
        // Check if gate is required
        if !self.config.is_required(gate) {
            return Ok(GateResult::Skipped { gate });
        }

        // Extract config values before taking mutable borrow
        let max_iterations = self.config.max_iterations;
        let current_iterations = self
            .gate_states
            .get(&gate)
            .map(|s| s.iterations)
            .unwrap_or(0);

        if current_iterations >= max_iterations {
            return Ok(GateResult::MaxIterationsExceeded {
                gate,
                iterations: current_iterations,
            });
        }

        // Build and invoke the review request
        let request = self.build_review_request(gate, state, diff);
        let mut handle = self.reviewer.invoke(request).await?;

        // Collect output from the child process
        use tokio::io::AsyncReadExt;
        let output = if let Some(stdout) = handle.child_mut().stdout.take() {
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut output = String::new();
            reader
                .read_to_string(&mut output)
                .await
                .map_err(crate::Error::Io)?;
            output
        } else {
            String::new()
        };

        // Wait for the process to complete
        let _ = handle.wait().await?;

        // Parse the feedback
        let feedback = self.parser.parse(&output);

        // Record the review
        let gate_state = self.gate_state_mut(gate);
        gate_state.record_review(feedback.clone());

        // Return result based on whether we can proceed
        if feedback.can_proceed() {
            Ok(GateResult::Passed { gate, feedback })
        } else {
            let feedback_for_coder = feedback.to_feedback_string();
            Ok(GateResult::Failed {
                gate,
                feedback,
                feedback_for_coder,
            })
        }
    }

    /// Check a gate with pre-provided reviewer output (for testing or manual review)
    pub fn check_gate_with_output(
        &mut self,
        gate: ReviewGate,
        reviewer_output: &str,
    ) -> GateResult {
        // Check if gate is required
        if !self.config.is_required(gate) {
            return GateResult::Skipped { gate };
        }

        // Extract config values and check iterations before taking mutable borrow
        let max_iterations = self.config.max_iterations;
        let current_iterations = self
            .gate_states
            .get(&gate)
            .map(|s| s.iterations)
            .unwrap_or(0);

        if current_iterations >= max_iterations {
            return GateResult::MaxIterationsExceeded {
                gate,
                iterations: current_iterations,
            };
        }

        // Parse the feedback before taking mutable borrow
        let feedback = self.parser.parse(reviewer_output);

        // Now take mutable borrow and record the review
        let gate_state = self.gate_state_mut(gate);
        gate_state.record_review(feedback.clone());

        // Return result based on whether we can proceed
        if feedback.can_proceed() {
            GateResult::Passed { gate, feedback }
        } else {
            let feedback_for_coder = feedback.to_feedback_string();
            GateResult::Failed {
                gate,
                feedback,
                feedback_for_coder,
            }
        }
    }

    /// Reset a specific gate's state
    pub fn reset_gate(&mut self, gate: ReviewGate) {
        if let Some(state) = self.gate_states.get_mut(&gate) {
            state.reset();
        }
    }

    /// Reset all gate states
    pub fn reset_all(&mut self) {
        self.gate_states.clear();
    }

    /// Get summary of all gate states
    pub fn status_summary(&self) -> String {
        let mut summary = String::new();
        for gate in ReviewGate::all() {
            let required = if self.config.is_required(*gate) {
                "required"
            } else {
                "optional"
            };
            let status = if let Some(state) = self.gate_states.get(gate) {
                if state.passed {
                    "PASSED"
                } else if state.iterations > 0 {
                    "FAILED"
                } else {
                    "pending"
                }
            } else {
                "pending"
            };
            summary.push_str(&format!("  {} ({}) - {}\n", gate, required, status));
        }
        summary
    }
}

/// Extension trait to add gate checking to TddState
pub trait GatedTddState {
    /// Check if we can proceed through a gate at the current phase
    fn check_gate(&self, manager: &PhaseGateManager) -> Option<ReviewGate>;

    /// Get the next required gate if any
    fn next_required_gate(&self, manager: &PhaseGateManager) -> Option<ReviewGate>;
}

impl GatedTddState for TddState {
    fn check_gate(&self, manager: &PhaseGateManager) -> Option<ReviewGate> {
        manager.gate_for_transition(self.phase)
    }

    fn next_required_gate(&self, manager: &PhaseGateManager) -> Option<ReviewGate> {
        let gate = ReviewGate::for_transition_from(self.phase)?;
        if manager.config.is_required(gate) && !manager.is_gate_passed(gate) {
            Some(gate)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::ReviewDecision;
    use std::path::PathBuf;

    #[test]
    fn test_review_gate_phases() {
        assert_eq!(ReviewGate::AfterSpec.after_phase(), TddPhase::WriteSpec);
        assert_eq!(ReviewGate::AfterSpec.before_phase(), TddPhase::WriteTests);

        assert_eq!(ReviewGate::AfterTests.after_phase(), TddPhase::WriteTests);
        assert_eq!(ReviewGate::AfterTests.before_phase(), TddPhase::VerifyRed);

        assert_eq!(
            ReviewGate::AfterImplement.after_phase(),
            TddPhase::Implement
        );
        assert_eq!(
            ReviewGate::AfterImplement.before_phase(),
            TddPhase::VerifyGreen
        );

        assert_eq!(ReviewGate::BeforeComplete.after_phase(), TddPhase::Refactor);
        assert_eq!(
            ReviewGate::BeforeComplete.before_phase(),
            TddPhase::Complete
        );
    }

    #[test]
    fn test_review_gate_review_types() {
        assert_eq!(ReviewGate::AfterSpec.review_type(), ReviewType::Spec);
        assert_eq!(ReviewGate::AfterTests.review_type(), ReviewType::Test);
        assert_eq!(ReviewGate::AfterImplement.review_type(), ReviewType::Code);
        assert_eq!(ReviewGate::BeforeComplete.review_type(), ReviewType::Final);
    }

    #[test]
    fn test_review_gate_for_transition() {
        assert_eq!(
            ReviewGate::for_transition_from(TddPhase::WriteSpec),
            Some(ReviewGate::AfterSpec)
        );
        assert_eq!(
            ReviewGate::for_transition_from(TddPhase::WriteTests),
            Some(ReviewGate::AfterTests)
        );
        assert_eq!(
            ReviewGate::for_transition_from(TddPhase::Implement),
            Some(ReviewGate::AfterImplement)
        );
        assert_eq!(
            ReviewGate::for_transition_from(TddPhase::Refactor),
            Some(ReviewGate::BeforeComplete)
        );
        assert_eq!(ReviewGate::for_transition_from(TddPhase::VerifyRed), None);
        assert_eq!(ReviewGate::for_transition_from(TddPhase::VerifyGreen), None);
        assert_eq!(ReviewGate::for_transition_from(TddPhase::Complete), None);
    }

    #[test]
    fn test_gate_config_default() {
        let config = GateConfig::default();
        assert!(config.require_spec_review);
        assert!(config.require_test_review);
        assert!(config.require_code_review);
        assert!(config.require_final_review);
        assert_eq!(config.max_iterations, 3);
    }

    #[test]
    fn test_gate_config_code_review_only() {
        let config = GateConfig::code_review_only();
        assert!(!config.require_spec_review);
        assert!(!config.require_test_review);
        assert!(config.require_code_review);
        assert!(!config.require_final_review);
    }

    #[test]
    fn test_gate_config_no_reviews() {
        let config = GateConfig::no_reviews();
        assert!(!config.require_spec_review);
        assert!(!config.require_test_review);
        assert!(!config.require_code_review);
        assert!(!config.require_final_review);
    }

    #[test]
    fn test_gate_config_is_required() {
        let config = GateConfig::new()
            .with_spec_review(false)
            .with_code_review(true);

        assert!(!config.is_required(ReviewGate::AfterSpec));
        assert!(config.is_required(ReviewGate::AfterTests));
        assert!(config.is_required(ReviewGate::AfterImplement));
    }

    #[test]
    fn test_gate_result_can_proceed() {
        let passed = GateResult::Passed {
            gate: ReviewGate::AfterSpec,
            feedback: ReviewFeedback::with_decision(ReviewDecision::Approved, "LGTM"),
        };
        assert!(passed.can_proceed());

        let skipped = GateResult::Skipped {
            gate: ReviewGate::AfterSpec,
        };
        assert!(skipped.can_proceed());

        let failed = GateResult::Failed {
            gate: ReviewGate::AfterSpec,
            feedback: ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Fix bugs"),
            feedback_for_coder: "Fix bugs".to_string(),
        };
        assert!(!failed.can_proceed());

        let exceeded = GateResult::MaxIterationsExceeded {
            gate: ReviewGate::AfterSpec,
            iterations: 3,
        };
        assert!(!exceeded.can_proceed());
    }

    #[test]
    fn test_gate_result_is_failed() {
        let passed = GateResult::Passed {
            gate: ReviewGate::AfterSpec,
            feedback: ReviewFeedback::with_decision(ReviewDecision::Approved, "LGTM"),
        };
        assert!(!passed.is_failed());

        let failed = GateResult::Failed {
            gate: ReviewGate::AfterSpec,
            feedback: ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Fix bugs"),
            feedback_for_coder: "Fix bugs".to_string(),
        };
        assert!(failed.is_failed());
    }

    #[test]
    fn test_gate_state_record_review() {
        let mut state = GateState::new();
        assert_eq!(state.iterations, 0);
        assert!(!state.passed);

        let feedback = ReviewFeedback::with_decision(ReviewDecision::ChangesRequested, "Fix bugs");
        state.record_review(feedback);

        assert_eq!(state.iterations, 1);
        assert!(!state.passed);
        assert!(state.previous_feedback.is_some());

        let feedback = ReviewFeedback::with_decision(ReviewDecision::Approved, "LGTM");
        state.record_review(feedback);

        assert_eq!(state.iterations, 2);
        assert!(state.passed);
    }

    #[test]
    fn test_gate_state_reset() {
        let mut state = GateState::new();
        state.record_review(ReviewFeedback::with_decision(
            ReviewDecision::Approved,
            "LGTM",
        ));
        assert!(state.passed);

        state.reset();
        assert_eq!(state.iterations, 0);
        assert!(!state.passed);
        assert!(state.previous_feedback.is_none());
    }

    #[test]
    fn test_phase_gate_manager_new() {
        let manager = PhaseGateManager::new();
        assert!(manager.config.require_spec_review);
        assert!(manager.config.require_code_review);
    }

    #[test]
    fn test_phase_gate_manager_with_config() {
        let config = GateConfig::code_review_only();
        let manager = PhaseGateManager::with_config(config);
        assert!(!manager.config.require_spec_review);
        assert!(manager.config.require_code_review);
    }

    #[test]
    fn test_phase_gate_manager_gate_for_transition() {
        let manager = PhaseGateManager::new();

        // With all gates required
        assert_eq!(
            manager.gate_for_transition(TddPhase::WriteSpec),
            Some(ReviewGate::AfterSpec)
        );
        assert_eq!(
            manager.gate_for_transition(TddPhase::Implement),
            Some(ReviewGate::AfterImplement)
        );

        // With only code review
        let config = GateConfig::code_review_only();
        let manager = PhaseGateManager::with_config(config);
        assert_eq!(manager.gate_for_transition(TddPhase::WriteSpec), None);
        assert_eq!(
            manager.gate_for_transition(TddPhase::Implement),
            Some(ReviewGate::AfterImplement)
        );
    }

    #[test]
    fn test_phase_gate_manager_can_transition() {
        let mut manager = PhaseGateManager::new();

        // Initially cannot transition through required gates
        assert!(!manager.can_transition(TddPhase::WriteSpec));

        // After passing the gate
        let state = manager.gate_state_mut(ReviewGate::AfterSpec);
        state.record_review(ReviewFeedback::with_decision(
            ReviewDecision::Approved,
            "LGTM",
        ));

        assert!(manager.can_transition(TddPhase::WriteSpec));
    }

    #[test]
    fn test_phase_gate_manager_check_gate_with_output() {
        let mut manager = PhaseGateManager::new();

        // Test approved review
        let result = manager.check_gate_with_output(ReviewGate::AfterSpec, "LGTM! Approved.");
        assert!(result.can_proceed());

        // Test rejected review (on next gate)
        let result = manager.check_gate_with_output(
            ReviewGate::AfterTests,
            "Changes requested: fix the security issue",
        );
        assert!(!result.can_proceed());
        assert!(result.feedback_for_coder().is_some());
    }

    #[test]
    fn test_phase_gate_manager_check_gate_skipped() {
        let config = GateConfig::no_reviews();
        let mut manager = PhaseGateManager::with_config(config);

        let result = manager.check_gate_with_output(ReviewGate::AfterSpec, "Anything");
        assert!(matches!(result, GateResult::Skipped { .. }));
        assert!(result.can_proceed());
    }

    #[test]
    fn test_phase_gate_manager_max_iterations() {
        let config = GateConfig::new().with_max_iterations(2);
        let mut manager = PhaseGateManager::with_config(config);

        // First review
        manager.check_gate_with_output(ReviewGate::AfterSpec, "Changes requested");
        // Second review
        manager.check_gate_with_output(ReviewGate::AfterSpec, "Changes requested");
        // Third review should be blocked
        let result = manager.check_gate_with_output(ReviewGate::AfterSpec, "Anything");

        assert!(matches!(result, GateResult::MaxIterationsExceeded { .. }));
        assert!(!result.can_proceed());
    }

    #[test]
    fn test_phase_gate_manager_reset_gate() {
        let mut manager = PhaseGateManager::new();

        manager.check_gate_with_output(ReviewGate::AfterSpec, "Approved");
        assert!(manager.is_gate_passed(ReviewGate::AfterSpec));

        manager.reset_gate(ReviewGate::AfterSpec);
        assert!(!manager.is_gate_passed(ReviewGate::AfterSpec));
    }

    #[test]
    fn test_phase_gate_manager_reset_all() {
        let mut manager = PhaseGateManager::new();

        manager.check_gate_with_output(ReviewGate::AfterSpec, "Approved");
        manager.check_gate_with_output(ReviewGate::AfterTests, "Approved");

        assert!(manager.is_gate_passed(ReviewGate::AfterSpec));
        assert!(manager.is_gate_passed(ReviewGate::AfterTests));

        manager.reset_all();

        assert!(!manager.is_gate_passed(ReviewGate::AfterSpec));
        assert!(!manager.is_gate_passed(ReviewGate::AfterTests));
    }

    #[test]
    fn test_build_review_request() {
        let manager = PhaseGateManager::new();
        let state = TddState::new("Test feature", PathBuf::from("/tmp/test"))
            .with_test_files(vec!["tests/test.rs".to_string()])
            .with_impl_files(vec!["src/lib.rs".to_string()]);

        let request =
            manager.build_review_request(ReviewGate::AfterImplement, &state, Some("+ code"));

        assert_eq!(request.review_type, ReviewType::Code);
        assert_eq!(request.context.task, "Test feature");
        assert!(request.context.diff.is_some());
    }

    #[test]
    fn test_gated_tdd_state_trait() {
        let manager = PhaseGateManager::new();
        let state = TddState::new("Test", PathBuf::from("/tmp"));

        // At WriteSpec, should need AfterSpec gate
        assert_eq!(state.check_gate(&manager), Some(ReviewGate::AfterSpec));
        assert_eq!(
            state.next_required_gate(&manager),
            Some(ReviewGate::AfterSpec)
        );
    }

    #[test]
    fn test_status_summary() {
        let manager = PhaseGateManager::new();
        let summary = manager.status_summary();

        assert!(summary.contains("Specification review gate"));
        assert!(summary.contains("required"));
        assert!(summary.contains("pending"));
    }
}
