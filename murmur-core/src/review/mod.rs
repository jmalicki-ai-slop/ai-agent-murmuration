//! Review module for code review workflows
//!
//! This module provides structured review request generation for the reviewer agent.
//! It supports different review types (spec, test, code, final) and generates
//! appropriate prompts with context.

pub mod request;

pub use request::{ReviewContext, ReviewRequest, ReviewRequestBuilder, ReviewType};
