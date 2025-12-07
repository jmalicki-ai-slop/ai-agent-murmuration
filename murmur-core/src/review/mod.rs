//! Review module for code review workflows
//!
//! This module provides structured review request generation, reviewer agent invocation,
//! feedback parsing, and routing feedback back to coder agents. It supports different
//! review types (spec, test, code, final) and generates appropriate prompts with context.

pub mod feedback;
pub mod request;
pub mod reviewer;
pub mod routing;

pub use feedback::{
    FeedbackItem, FeedbackParser, FeedbackSeverity, ReviewDecision, ReviewFeedback,
};
pub use request::{ReviewContext, ReviewRequest, ReviewRequestBuilder, ReviewType};
pub use reviewer::{invoke_review, invoke_review_with_config, Reviewer, ReviewerConfig};
pub use routing::{
    route_feedback, route_feedback_with_config, FeedbackRouter, FixRequest, FixRequestBuilder,
    RoutingConfig, RoutingStrategy,
};
