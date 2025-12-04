//! Murmur GitHub - GitHub integration for Murmuration
//!
//! This crate provides GitHub API access for reading issues, managing PRs,
//! and tracking dependencies between work items.

mod client;
mod error;

pub use client::GitHubClient;
pub use error::{Error, Result};
