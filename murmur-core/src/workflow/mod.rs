//! Workflow module for coordinating agent interactions
//!
//! This module provides workflow patterns like TDD (Test-Driven Development)
//! that coordinate multiple agents working together.

// Temporarily commented out due to unresolved imports - these are existing issues
// pub mod coordinator;
pub mod resume;
// pub mod review;
pub mod state;
// pub mod tdd;

// pub use coordinator::{
//     CoordinatorConfig, CoordinatorPhase, CoordinatorState, CoordinatorWorkflow, PhaseTransition,
//     SubTask, SubTaskStatus,
// };
pub use resume::{
    build_resume_prompt, find_incomplete_runs, find_latest_incomplete_run,
    reconstruct_conversation, ConversationMessage, ResumableRun,
};
// pub use review::{
//     ReviewIssue, ReviewResult, ReviewState, ReviewTrigger, ReviewVerdict, ReviewWorkflow,
// };
pub use state::{PhaseValidation, StateMachine, Workflow};
// pub use tdd::{TddPhase, TddState, TddWorkflow};
