//! Review module for code review workflows
//!
//! This module provides structured review request generation, reviewer agent invocation,
//! and feedback parsing. It supports different review types (spec, test, code, final)
//! and generates appropriate prompts with context.

pub mod feedback;
pub mod request;
pub mod reviewer;

pub use feedback::{
    FeedbackItem, FeedbackParser, FeedbackSeverity, ReviewDecision, ReviewFeedback,
};
pub use request::{ReviewContext, ReviewRequest, ReviewRequestBuilder, ReviewType};
pub use reviewer::{invoke_review, invoke_review_with_config, Reviewer, ReviewerConfig};
