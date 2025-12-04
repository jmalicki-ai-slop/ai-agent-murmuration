//! Git operations for Murmuration
//!
//! This module provides git repository detection, validation, and worktree management.

mod branch;
mod repo;

pub use branch::{BranchingOptions, BranchingPoint};
pub use repo::{GitRepo, RemoteInfo};
