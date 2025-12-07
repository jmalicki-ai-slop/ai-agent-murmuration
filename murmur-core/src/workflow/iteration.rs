//! Iteration tracking for workflow retry loops
//!
//! This module provides comprehensive iteration tracking and management for
//! workflows that involve retry loops, such as TDD verify phases and
//! review-fix cycles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for an iteration context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IterationContext {
    /// Type of iteration (e.g., "tdd_verify_red", "tdd_verify_green", "review_fix")
    pub context_type: String,
    /// Optional identifier (e.g., phase name, review type)
    pub identifier: Option<String>,
}

impl IterationContext {
    /// Create a new iteration context
    pub fn new(context_type: impl Into<String>) -> Self {
        Self {
            context_type: context_type.into(),
            identifier: None,
        }
    }

    /// Create a context with an identifier
    pub fn with_identifier(context_type: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self {
            context_type: context_type.into(),
            identifier: Some(identifier.into()),
        }
    }

    /// Create a TDD verify red context
    pub fn tdd_verify_red() -> Self {
        Self::new("tdd_verify_red")
    }

    /// Create a TDD verify green context
    pub fn tdd_verify_green() -> Self {
        Self::new("tdd_verify_green")
    }

    /// Create a review-fix cycle context
    pub fn review_fix(review_type: &str) -> Self {
        Self::with_identifier("review_fix", review_type)
    }

    /// Create a phase gate context
    pub fn phase_gate(phase: &str) -> Self {
        Self::with_identifier("phase_gate", phase)
    }
}

impl fmt::Display for IterationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.identifier {
            Some(id) => write!(f, "{}:{}", self.context_type, id),
            None => write!(f, "{}", self.context_type),
        }
    }
}

/// Record of a single iteration attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationAttempt {
    /// The iteration number (1-indexed)
    pub number: u32,
    /// When the attempt started
    pub started_at: DateTime<Utc>,
    /// When the attempt completed (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Whether the attempt was successful
    pub success: bool,
    /// Reason for failure (if failed)
    pub failure_reason: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl IterationAttempt {
    /// Create a new iteration attempt
    pub fn new(number: u32) -> Self {
        Self {
            number,
            started_at: Utc::now(),
            completed_at: None,
            success: false,
            failure_reason: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark the attempt as completed successfully
    pub fn complete_success(&mut self) {
        self.completed_at = Some(Utc::now());
        self.success = true;
    }

    /// Mark the attempt as failed
    pub fn complete_failure(&mut self, reason: impl Into<String>) {
        self.completed_at = Some(Utc::now());
        self.success = false;
        self.failure_reason = Some(reason.into());
    }

    /// Add metadata to the attempt
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if the attempt is still in progress
    pub fn is_in_progress(&self) -> bool {
        self.completed_at.is_none()
    }

    /// Get the duration of the attempt
    pub fn duration_ms(&self) -> Option<i64> {
        self.completed_at
            .map(|end| (end - self.started_at).num_milliseconds())
    }
}

/// Status of an iteration sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IterationStatus {
    /// No iterations have been attempted
    NotStarted,
    /// Currently in an iteration attempt
    InProgress,
    /// Completed successfully within the limit
    Succeeded,
    /// Max iterations reached without success
    Exhausted,
    /// Manually stopped
    Stopped,
}

impl IterationStatus {
    /// Check if iterations can continue
    pub fn can_continue(&self) -> bool {
        matches!(
            self,
            IterationStatus::NotStarted | IterationStatus::InProgress
        )
    }

    /// Check if the iteration sequence has ended
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            IterationStatus::Succeeded | IterationStatus::Exhausted | IterationStatus::Stopped
        )
    }
}

impl fmt::Display for IterationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IterationStatus::NotStarted => write!(f, "not_started"),
            IterationStatus::InProgress => write!(f, "in_progress"),
            IterationStatus::Succeeded => write!(f, "succeeded"),
            IterationStatus::Exhausted => write!(f, "exhausted"),
            IterationStatus::Stopped => write!(f, "stopped"),
        }
    }
}

/// Configuration for iteration limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationConfig {
    /// Maximum number of iterations allowed
    pub max_iterations: u32,
    /// Whether to warn on last iteration
    pub warn_on_last: bool,
    /// Whether to auto-stop on success
    pub auto_stop_on_success: bool,
}

impl Default for IterationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            warn_on_last: true,
            auto_stop_on_success: true,
        }
    }
}

impl IterationConfig {
    /// Create config with a specific max
    pub fn with_max(max_iterations: u32) -> Self {
        Self {
            max_iterations,
            ..Default::default()
        }
    }

    /// Set warn_on_last flag
    pub fn warn_on_last(mut self, warn: bool) -> Self {
        self.warn_on_last = warn;
        self
    }

    /// Set auto_stop_on_success flag
    pub fn auto_stop_on_success(mut self, stop: bool) -> Self {
        self.auto_stop_on_success = stop;
        self
    }
}

/// Tracker for a sequence of iterations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationTracker {
    /// The context for this iteration sequence
    pub context: IterationContext,
    /// Configuration for this tracker
    pub config: IterationConfig,
    /// All attempts in this sequence
    pub attempts: Vec<IterationAttempt>,
    /// Current status
    pub status: IterationStatus,
    /// When the sequence started
    pub started_at: DateTime<Utc>,
    /// When the sequence ended (if ended)
    pub ended_at: Option<DateTime<Utc>>,
}

impl IterationTracker {
    /// Create a new iteration tracker
    pub fn new(context: IterationContext) -> Self {
        Self {
            context,
            config: IterationConfig::default(),
            attempts: Vec::new(),
            status: IterationStatus::NotStarted,
            started_at: Utc::now(),
            ended_at: None,
        }
    }

    /// Create a tracker with config
    pub fn with_config(context: IterationContext, config: IterationConfig) -> Self {
        Self {
            context,
            config,
            attempts: Vec::new(),
            status: IterationStatus::NotStarted,
            started_at: Utc::now(),
            ended_at: None,
        }
    }

    /// Get the current iteration number (0 if not started)
    pub fn current_iteration(&self) -> u32 {
        self.attempts.len() as u32
    }

    /// Get the max iterations
    pub fn max_iterations(&self) -> u32 {
        self.config.max_iterations
    }

    /// Check if at the last iteration
    pub fn is_last_iteration(&self) -> bool {
        self.current_iteration() >= self.config.max_iterations
    }

    /// Check if iterations are exhausted
    pub fn is_exhausted(&self) -> bool {
        self.status == IterationStatus::Exhausted
    }

    /// Get remaining iterations
    pub fn remaining_iterations(&self) -> u32 {
        self.config
            .max_iterations
            .saturating_sub(self.current_iteration())
    }

    /// Start a new iteration attempt
    ///
    /// Returns None if max iterations reached
    pub fn start_attempt(&mut self) -> Option<&mut IterationAttempt> {
        if self.status.is_terminal() {
            return None;
        }

        if self.current_iteration() >= self.config.max_iterations {
            self.status = IterationStatus::Exhausted;
            self.ended_at = Some(Utc::now());
            return None;
        }

        let number = self.current_iteration() + 1;
        let attempt = IterationAttempt::new(number);
        self.attempts.push(attempt);
        self.status = IterationStatus::InProgress;

        self.attempts.last_mut()
    }

    /// Complete the current attempt as successful
    pub fn complete_success(&mut self) {
        if let Some(attempt) = self.attempts.last_mut() {
            attempt.complete_success();
        }

        if self.config.auto_stop_on_success {
            self.status = IterationStatus::Succeeded;
            self.ended_at = Some(Utc::now());
        }
    }

    /// Complete the current attempt as failed
    pub fn complete_failure(&mut self, reason: impl Into<String>) {
        let reason_str = reason.into();
        if let Some(attempt) = self.attempts.last_mut() {
            attempt.complete_failure(&reason_str);
        }

        // Check if we've exhausted iterations
        if self.current_iteration() >= self.config.max_iterations {
            self.status = IterationStatus::Exhausted;
            self.ended_at = Some(Utc::now());
        }
    }

    /// Manually stop the iteration sequence
    pub fn stop(&mut self) {
        self.status = IterationStatus::Stopped;
        self.ended_at = Some(Utc::now());
    }

    /// Get the last attempt
    pub fn last_attempt(&self) -> Option<&IterationAttempt> {
        self.attempts.last()
    }

    /// Get the last failure reason
    pub fn last_failure_reason(&self) -> Option<&str> {
        self.attempts
            .iter()
            .rev()
            .find(|a| !a.success)
            .and_then(|a| a.failure_reason.as_deref())
    }

    /// Count successful attempts
    pub fn successful_attempts(&self) -> usize {
        self.attempts.iter().filter(|a| a.success).count()
    }

    /// Count failed attempts
    pub fn failed_attempts(&self) -> usize {
        self.attempts.iter().filter(|a| !a.success).count()
    }

    /// Get a summary of the iteration sequence
    pub fn summary(&self) -> IterationSummary {
        IterationSummary {
            context: self.context.clone(),
            total_attempts: self.attempts.len() as u32,
            max_iterations: self.config.max_iterations,
            successful: self.successful_attempts() as u32,
            failed: self.failed_attempts() as u32,
            status: self.status,
            last_failure: self.last_failure_reason().map(String::from),
        }
    }

    /// Check if should warn about last iteration
    pub fn should_warn_last_iteration(&self) -> bool {
        self.config.warn_on_last && self.is_last_iteration()
    }
}

/// Summary of an iteration sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationSummary {
    /// The context
    pub context: IterationContext,
    /// Total attempts made
    pub total_attempts: u32,
    /// Max iterations allowed
    pub max_iterations: u32,
    /// Number of successful attempts
    pub successful: u32,
    /// Number of failed attempts
    pub failed: u32,
    /// Final status
    pub status: IterationStatus,
    /// Last failure reason
    pub last_failure: Option<String>,
}

impl fmt::Display for IterationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} attempts ({}/{} max), {} successful, {} failed - {}",
            self.context,
            self.total_attempts,
            self.total_attempts,
            self.max_iterations,
            self.successful,
            self.failed,
            self.status
        )
    }
}

/// Manager for multiple iteration trackers
#[derive(Debug, Clone, Default)]
pub struct IterationManager {
    /// Active trackers by context
    trackers: HashMap<IterationContext, IterationTracker>,
    /// Default configuration
    default_config: IterationConfig,
}

impl IterationManager {
    /// Create a new iteration manager
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
            default_config: IterationConfig::default(),
        }
    }

    /// Create with a custom default config
    pub fn with_default_config(config: IterationConfig) -> Self {
        Self {
            trackers: HashMap::new(),
            default_config: config,
        }
    }

    /// Get or create a tracker for a context
    pub fn get_or_create(&mut self, context: IterationContext) -> &mut IterationTracker {
        self.trackers
            .entry(context.clone())
            .or_insert_with(|| IterationTracker::with_config(context, self.default_config.clone()))
    }

    /// Get a tracker if it exists
    pub fn get(&self, context: &IterationContext) -> Option<&IterationTracker> {
        self.trackers.get(context)
    }

    /// Get a mutable tracker if it exists
    pub fn get_mut(&mut self, context: &IterationContext) -> Option<&mut IterationTracker> {
        self.trackers.get_mut(context)
    }

    /// Start an attempt for a context
    pub fn start_attempt(&mut self, context: IterationContext) -> Option<&mut IterationAttempt> {
        self.get_or_create(context).start_attempt()
    }

    /// Complete the current attempt as successful
    pub fn complete_success(&mut self, context: &IterationContext) {
        if let Some(tracker) = self.trackers.get_mut(context) {
            tracker.complete_success();
        }
    }

    /// Complete the current attempt as failed
    pub fn complete_failure(&mut self, context: &IterationContext, reason: impl Into<String>) {
        if let Some(tracker) = self.trackers.get_mut(context) {
            tracker.complete_failure(reason);
        }
    }

    /// Check if a context can continue iterating
    pub fn can_continue(&self, context: &IterationContext) -> bool {
        self.trackers
            .get(context)
            .map(|t| t.status.can_continue() && !t.is_exhausted())
            .unwrap_or(true) // Can start if not tracked yet
    }

    /// Get remaining iterations for a context
    pub fn remaining(&self, context: &IterationContext) -> u32 {
        self.trackers
            .get(context)
            .map(|t| t.remaining_iterations())
            .unwrap_or(self.default_config.max_iterations)
    }

    /// Reset a tracker
    pub fn reset(&mut self, context: &IterationContext) {
        self.trackers.remove(context);
    }

    /// Reset all trackers
    pub fn reset_all(&mut self) {
        self.trackers.clear();
    }

    /// Get all active trackers
    pub fn active_trackers(&self) -> impl Iterator<Item = &IterationTracker> {
        self.trackers.values().filter(|t| !t.status.is_terminal())
    }

    /// Get all tracker summaries
    pub fn summaries(&self) -> Vec<IterationSummary> {
        self.trackers.values().map(|t| t.summary()).collect()
    }
}

/// Result of checking iteration status
#[derive(Debug, Clone)]
pub enum IterationCheck {
    /// Can proceed with iteration
    CanProceed {
        /// Current iteration number
        iteration: u32,
        /// Max iterations
        max: u32,
        /// Whether this is the last iteration
        is_last: bool,
    },
    /// Cannot proceed - max iterations reached
    Exhausted {
        /// Total attempts made
        total_attempts: u32,
        /// Last failure reason
        last_reason: Option<String>,
    },
    /// Sequence already succeeded
    AlreadySucceeded,
    /// Sequence was stopped
    Stopped,
}

impl IterationCheck {
    /// Check if can proceed
    pub fn can_proceed(&self) -> bool {
        matches!(self, IterationCheck::CanProceed { .. })
    }

    /// Get iteration number if can proceed
    pub fn iteration(&self) -> Option<u32> {
        match self {
            IterationCheck::CanProceed { iteration, .. } => Some(*iteration),
            _ => None,
        }
    }
}

/// Check iteration status and return structured result
pub fn check_iteration(tracker: &IterationTracker) -> IterationCheck {
    match tracker.status {
        IterationStatus::Succeeded => IterationCheck::AlreadySucceeded,
        IterationStatus::Stopped => IterationCheck::Stopped,
        IterationStatus::Exhausted => IterationCheck::Exhausted {
            total_attempts: tracker.current_iteration(),
            last_reason: tracker.last_failure_reason().map(String::from),
        },
        _ => {
            let next_iteration = tracker.current_iteration() + 1;
            if next_iteration > tracker.config.max_iterations {
                IterationCheck::Exhausted {
                    total_attempts: tracker.current_iteration(),
                    last_reason: tracker.last_failure_reason().map(String::from),
                }
            } else {
                IterationCheck::CanProceed {
                    iteration: next_iteration,
                    max: tracker.config.max_iterations,
                    is_last: next_iteration == tracker.config.max_iterations,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_context_display() {
        let ctx = IterationContext::new("test");
        assert_eq!(ctx.to_string(), "test");

        let ctx = IterationContext::with_identifier("review_fix", "code");
        assert_eq!(ctx.to_string(), "review_fix:code");
    }

    #[test]
    fn test_iteration_context_factories() {
        let ctx = IterationContext::tdd_verify_red();
        assert_eq!(ctx.context_type, "tdd_verify_red");
        assert!(ctx.identifier.is_none());

        let ctx = IterationContext::review_fix("spec");
        assert_eq!(ctx.context_type, "review_fix");
        assert_eq!(ctx.identifier, Some("spec".to_string()));
    }

    #[test]
    fn test_iteration_attempt() {
        let mut attempt = IterationAttempt::new(1);
        assert!(attempt.is_in_progress());
        assert!(!attempt.success);
        assert!(attempt.duration_ms().is_none());

        attempt.complete_success();
        assert!(!attempt.is_in_progress());
        assert!(attempt.success);
        assert!(attempt.duration_ms().is_some());
    }

    #[test]
    fn test_iteration_attempt_failure() {
        let mut attempt = IterationAttempt::new(1);
        attempt.complete_failure("Tests failed");

        assert!(!attempt.is_in_progress());
        assert!(!attempt.success);
        assert_eq!(attempt.failure_reason, Some("Tests failed".to_string()));
    }

    #[test]
    fn test_iteration_attempt_metadata() {
        let attempt = IterationAttempt::new(1)
            .with_metadata("phase", "verify_red")
            .with_metadata("test_count", "5");

        assert_eq!(
            attempt.metadata.get("phase"),
            Some(&"verify_red".to_string())
        );
        assert_eq!(attempt.metadata.get("test_count"), Some(&"5".to_string()));
    }

    #[test]
    fn test_iteration_status() {
        assert!(IterationStatus::NotStarted.can_continue());
        assert!(IterationStatus::InProgress.can_continue());
        assert!(!IterationStatus::Succeeded.can_continue());
        assert!(!IterationStatus::Exhausted.can_continue());
        assert!(!IterationStatus::Stopped.can_continue());

        assert!(!IterationStatus::NotStarted.is_terminal());
        assert!(!IterationStatus::InProgress.is_terminal());
        assert!(IterationStatus::Succeeded.is_terminal());
        assert!(IterationStatus::Exhausted.is_terminal());
        assert!(IterationStatus::Stopped.is_terminal());
    }

    #[test]
    fn test_iteration_config() {
        let config = IterationConfig::default();
        assert_eq!(config.max_iterations, 3);
        assert!(config.warn_on_last);
        assert!(config.auto_stop_on_success);

        let config = IterationConfig::with_max(5)
            .warn_on_last(false)
            .auto_stop_on_success(false);
        assert_eq!(config.max_iterations, 5);
        assert!(!config.warn_on_last);
        assert!(!config.auto_stop_on_success);
    }

    #[test]
    fn test_iteration_tracker_basic() {
        let ctx = IterationContext::tdd_verify_red();
        let tracker = IterationTracker::new(ctx);

        assert_eq!(tracker.current_iteration(), 0);
        assert_eq!(tracker.max_iterations(), 3);
        assert_eq!(tracker.remaining_iterations(), 3);
        assert!(!tracker.is_exhausted());
        assert_eq!(tracker.status, IterationStatus::NotStarted);
    }

    #[test]
    fn test_iteration_tracker_attempts() {
        let ctx = IterationContext::tdd_verify_red();
        let mut tracker = IterationTracker::new(ctx);

        // First attempt
        let attempt = tracker.start_attempt();
        assert!(attempt.is_some());
        assert_eq!(tracker.current_iteration(), 1);
        assert_eq!(tracker.remaining_iterations(), 2);
        assert_eq!(tracker.status, IterationStatus::InProgress);

        // Complete failure
        tracker.complete_failure("Tests passed unexpectedly");
        assert_eq!(tracker.failed_attempts(), 1);

        // Second attempt
        let attempt = tracker.start_attempt();
        assert!(attempt.is_some());
        assert_eq!(tracker.current_iteration(), 2);

        // Complete success
        tracker.complete_success();
        assert_eq!(tracker.successful_attempts(), 1);
        assert_eq!(tracker.status, IterationStatus::Succeeded);
    }

    #[test]
    fn test_iteration_tracker_exhaustion() {
        let ctx = IterationContext::tdd_verify_green();
        let config = IterationConfig::with_max(2);
        let mut tracker = IterationTracker::with_config(ctx, config);

        // Attempt 1
        tracker.start_attempt();
        tracker.complete_failure("Tests fail");

        // Attempt 2
        tracker.start_attempt();
        tracker.complete_failure("Tests still fail");

        // Should be exhausted now
        assert!(tracker.is_exhausted());
        assert_eq!(tracker.status, IterationStatus::Exhausted);
        assert!(tracker.ended_at.is_some());

        // No more attempts allowed
        assert!(tracker.start_attempt().is_none());
    }

    #[test]
    fn test_iteration_tracker_stop() {
        let ctx = IterationContext::new("test");
        let mut tracker = IterationTracker::new(ctx);

        tracker.start_attempt();
        tracker.stop();

        assert_eq!(tracker.status, IterationStatus::Stopped);
        assert!(tracker.start_attempt().is_none());
    }

    #[test]
    fn test_iteration_tracker_last_iteration_warning() {
        let ctx = IterationContext::new("test");
        let config = IterationConfig::with_max(2);
        let mut tracker = IterationTracker::with_config(ctx, config);

        tracker.start_attempt();
        tracker.complete_failure("fail");
        assert!(!tracker.should_warn_last_iteration());

        tracker.start_attempt();
        // Now at iteration 2/2 - should warn
        assert!(tracker.should_warn_last_iteration());
    }

    #[test]
    fn test_iteration_tracker_summary() {
        let ctx = IterationContext::review_fix("code");
        let mut tracker = IterationTracker::new(ctx.clone());

        tracker.start_attempt();
        tracker.complete_failure("Changes needed");
        tracker.start_attempt();
        tracker.complete_success();

        let summary = tracker.summary();
        assert_eq!(summary.context, ctx);
        assert_eq!(summary.total_attempts, 2);
        assert_eq!(summary.successful, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.status, IterationStatus::Succeeded);
    }

    #[test]
    fn test_iteration_manager_basic() {
        let mut manager = IterationManager::new();
        let ctx = IterationContext::tdd_verify_red();

        // Can continue before tracking
        assert!(manager.can_continue(&ctx));

        // Start tracking
        let tracker = manager.get_or_create(ctx.clone());
        assert_eq!(tracker.current_iteration(), 0);

        // Start attempt via manager
        manager.start_attempt(ctx.clone());
        assert_eq!(manager.get(&ctx).unwrap().current_iteration(), 1);
    }

    #[test]
    fn test_iteration_manager_complete_success() {
        let mut manager = IterationManager::new();
        let ctx = IterationContext::new("test");

        manager.start_attempt(ctx.clone());
        manager.complete_success(&ctx);

        let tracker = manager.get(&ctx).unwrap();
        assert_eq!(tracker.status, IterationStatus::Succeeded);
    }

    #[test]
    fn test_iteration_manager_complete_failure() {
        let mut manager = IterationManager::new();
        let ctx = IterationContext::new("test");

        manager.start_attempt(ctx.clone());
        manager.complete_failure(&ctx, "Something went wrong");

        let tracker = manager.get(&ctx).unwrap();
        assert_eq!(tracker.failed_attempts(), 1);
        assert_eq!(tracker.last_failure_reason(), Some("Something went wrong"));
    }

    #[test]
    fn test_iteration_manager_remaining() {
        let config = IterationConfig::with_max(5);
        let mut manager = IterationManager::with_default_config(config);
        let ctx = IterationContext::new("test");

        // Before tracking
        assert_eq!(manager.remaining(&ctx), 5);

        // After one attempt
        manager.start_attempt(ctx.clone());
        manager.complete_failure(&ctx, "fail");
        assert_eq!(manager.remaining(&ctx), 4);
    }

    #[test]
    fn test_iteration_manager_reset() {
        let mut manager = IterationManager::new();
        let ctx = IterationContext::new("test");

        manager.start_attempt(ctx.clone());
        assert!(manager.get(&ctx).is_some());

        manager.reset(&ctx);
        assert!(manager.get(&ctx).is_none());
    }

    #[test]
    fn test_iteration_manager_summaries() {
        let mut manager = IterationManager::new();

        let ctx1 = IterationContext::tdd_verify_red();
        let ctx2 = IterationContext::review_fix("code");

        manager.start_attempt(ctx1.clone());
        manager.complete_failure(&ctx1, "fail");

        manager.start_attempt(ctx2.clone());
        manager.complete_success(&ctx2);

        let summaries = manager.summaries();
        assert_eq!(summaries.len(), 2);
    }

    #[test]
    fn test_check_iteration_can_proceed() {
        let ctx = IterationContext::new("test");
        let tracker = IterationTracker::new(ctx);

        let check = check_iteration(&tracker);
        assert!(check.can_proceed());

        match check {
            IterationCheck::CanProceed {
                iteration,
                max,
                is_last,
            } => {
                assert_eq!(iteration, 1);
                assert_eq!(max, 3);
                assert!(!is_last);
            }
            _ => panic!("Expected CanProceed"),
        }
    }

    #[test]
    fn test_check_iteration_exhausted() {
        let ctx = IterationContext::new("test");
        let config = IterationConfig::with_max(1);
        let mut tracker = IterationTracker::with_config(ctx, config);

        tracker.start_attempt();
        tracker.complete_failure("fail");

        let check = check_iteration(&tracker);
        assert!(!check.can_proceed());

        match check {
            IterationCheck::Exhausted {
                total_attempts,
                last_reason,
            } => {
                assert_eq!(total_attempts, 1);
                assert_eq!(last_reason, Some("fail".to_string()));
            }
            _ => panic!("Expected Exhausted"),
        }
    }

    #[test]
    fn test_check_iteration_succeeded() {
        let ctx = IterationContext::new("test");
        let mut tracker = IterationTracker::new(ctx);

        tracker.start_attempt();
        tracker.complete_success();

        let check = check_iteration(&tracker);
        assert!(matches!(check, IterationCheck::AlreadySucceeded));
    }

    #[test]
    fn test_check_iteration_stopped() {
        let ctx = IterationContext::new("test");
        let mut tracker = IterationTracker::new(ctx);

        tracker.stop();

        let check = check_iteration(&tracker);
        assert!(matches!(check, IterationCheck::Stopped));
    }

    #[test]
    fn test_check_iteration_is_last() {
        let ctx = IterationContext::new("test");
        let config = IterationConfig::with_max(2);
        let mut tracker = IterationTracker::with_config(ctx, config);

        // First attempt
        tracker.start_attempt();
        tracker.complete_failure("fail");

        // Check before second (last) attempt
        let check = check_iteration(&tracker);
        match check {
            IterationCheck::CanProceed { is_last, .. } => {
                assert!(is_last);
            }
            _ => panic!("Expected CanProceed"),
        }
    }

    #[test]
    fn test_iteration_summary_display() {
        let ctx = IterationContext::review_fix("test");
        let mut tracker = IterationTracker::new(ctx);

        tracker.start_attempt();
        tracker.complete_failure("fail");
        tracker.start_attempt();
        tracker.complete_success();

        let summary = tracker.summary();
        let display = format!("{}", summary);
        assert!(display.contains("review_fix:test"));
        assert!(display.contains("2 attempts"));
        assert!(display.contains("1 successful"));
        assert!(display.contains("1 failed"));
        assert!(display.contains("succeeded"));
    }
}
