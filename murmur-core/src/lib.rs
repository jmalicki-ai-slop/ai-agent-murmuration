//! Murmur Core - Core library for Murmuration multi-agent orchestration
//!
//! This crate provides the core functionality for orchestrating multiple
//! AI agents working collaboratively on software development tasks.

pub mod agent;
pub mod config;
pub mod error;
pub mod git;

pub use agent::{
    AgentHandle, AgentSpawner, CostInfo, OutputStreamer, PrintHandler, StreamHandler, StreamMessage,
};
pub use config::{AgentConfig, Config};
pub use error::{Error, Result};
pub use git::{
    default_cache_dir, worktree_path, BranchingOptions, BranchingPoint, CachedWorktree, GitRepo,
    PoolConfig, RemoteInfo, WorktreeInfo, WorktreeMetadata, WorktreeOptions, WorktreePool,
    WorktreeStatus,
};
