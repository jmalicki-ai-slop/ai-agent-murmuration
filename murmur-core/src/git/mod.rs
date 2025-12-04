//! Git operations for Murmuration
//!
//! This module provides git repository detection, validation, and worktree management.

mod branch;
mod pool;
mod repo;
mod worktree;

pub use branch::{BranchingOptions, BranchingPoint};
pub use pool::{CachedWorktree, PoolConfig, WorktreeMetadata, WorktreePool, WorktreeStatus};
pub use repo::{GitRepo, RemoteInfo};
pub use worktree::{default_cache_dir, worktree_path, WorktreeInfo, WorktreeOptions};
