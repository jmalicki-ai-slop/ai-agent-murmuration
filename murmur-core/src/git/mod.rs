//! Git operations for Murmuration
//!
//! This module provides git repository detection, validation, and worktree management.

mod repo;

pub use repo::{GitRepo, RemoteInfo};
