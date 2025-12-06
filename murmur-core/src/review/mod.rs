//! Review module for code review workflows
//!
//! This module provides structured review request generation and reviewer agent invocation.
//! It supports different review types (spec, test, code, final) and generates
//! appropriate prompts with context.

pub mod request;
pub mod reviewer;

pub use request::{ReviewContext, ReviewRequest, ReviewRequestBuilder, ReviewType};
pub use reviewer::{invoke_review, invoke_review_with_config, Reviewer, ReviewerConfig};
