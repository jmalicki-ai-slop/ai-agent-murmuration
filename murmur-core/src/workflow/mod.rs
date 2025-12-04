//! Workflow module for coordinating agent interactions
//!
//! This module provides workflow patterns like TDD (Test-Driven Development)
//! that coordinate multiple agents working together.

pub mod coordinator;
pub mod review;
pub mod tdd;

pub use coordinator::{
    CoordinatorConfig, CoordinatorPhase, CoordinatorState, CoordinatorWorkflow, PhaseTransition,
    SubTask, SubTaskStatus,
};
pub use review::{
    ReviewIssue, ReviewResult, ReviewState, ReviewTrigger, ReviewVerdict, ReviewWorkflow,
};
pub use tdd::{TddPhase, TddState, TddWorkflow};
