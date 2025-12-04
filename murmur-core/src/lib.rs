//! Murmur Core - Core library for Murmuration multi-agent orchestration
//!
//! This crate provides the core functionality for orchestrating multiple
//! AI agents working collaboratively on software development tasks.

pub mod agent;
pub mod config;
pub mod error;
pub mod git;
pub mod plan;
pub mod secrets;

pub use agent::{
    AgentHandle, AgentSpawner, CostInfo, OutputStreamer, PrintHandler, StreamHandler, StreamMessage,
};
pub use config::{AgentConfig, Config};
pub use error::{Error, Result};
pub use secrets::{GitHubSecrets, Secrets};
pub use git::{
    cached_repo_path, clone_repo, default_cache_dir, default_repos_cache_dir, fetch_repo,
    is_repo_cached, worktree_path, BranchingOptions, BranchingPoint, CachedWorktree, GitRepo,
    PoolConfig, RemoteInfo, RepoUrl, WorktreeInfo, WorktreeMetadata, WorktreeOptions, WorktreePool,
    WorktreeStatus,
};
pub use plan::{parse_plan, Phase, Plan, PlannedPR};
