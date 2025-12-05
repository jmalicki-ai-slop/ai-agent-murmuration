//! Murmur GitHub - GitHub integration for Murmuration
//!
//! This crate provides GitHub API access for reading issues, managing PRs,
//! and tracking dependencies between work items.

mod client;
mod create;
mod dependencies;
mod error;
mod graphql;
mod issues;
mod metadata;
mod pr;

pub use client::GitHubClient;
pub use create::{ImportOptions, ImportResult};
pub use dependencies::{DependencyGraph, IssueDependencies, IssueRef};
pub use error::{Error, Result};
pub use issues::{Issue, IssueFilter, IssueState, SubIssuesSummary};
pub use metadata::{parse_depends_on_links, IssueMetadata};
pub use pr::{DependencyStatus, PrState, PullRequest};
