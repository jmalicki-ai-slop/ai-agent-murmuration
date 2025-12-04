//! Git operations for Murmuration
//!
//! This module provides git repository detection, validation, and worktree management.

mod branch;
mod clone;
mod pool;
mod repo;
mod worktree;

pub use branch::{BranchingOptions, BranchingPoint};
pub use clone::{
    cached_repo_path, clone_repo, default_repos_cache_dir, fetch_repo, is_repo_cached, RepoUrl,
};
pub use pool::{CachedWorktree, PoolConfig, WorktreeMetadata, WorktreePool, WorktreeStatus};
pub use repo::{GitRepo, RemoteInfo};
pub use worktree::{default_cache_dir, worktree_path, WorktreeInfo, WorktreeOptions};
