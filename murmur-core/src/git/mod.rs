//! Git operations for Murmuration
//!
//! This module provides git repository detection, validation, and worktree management.

mod branch;
mod repo;
mod worktree;

pub use branch::{BranchingOptions, BranchingPoint};
pub use repo::{GitRepo, RemoteInfo};
pub use worktree::{default_cache_dir, worktree_path, WorktreeInfo, WorktreeOptions};
