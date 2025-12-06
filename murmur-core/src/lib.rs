//! Murmur Core - Core library for Murmuration multi-agent orchestration
//!
//! This crate provides the core functionality for orchestrating multiple
//! AI agents working collaboratively on software development tasks.

pub mod agent;
pub mod config;
pub mod error;
pub mod git;
pub mod plan;
pub mod review;
pub mod secrets;
pub mod workflow;

pub use agent::{
    AgentFactory, AgentHandle, AgentSpawner, AgentType, CoordinatorAgent, CostInfo, ImplementAgent,
    OutputStreamer, PrintHandler, PromptBuilder, PromptContext, ReviewAgent, StreamHandler,
    StreamMessage, TestAgent, TypedAgent,
};
pub use config::{AgentConfig, Config};
pub use error::{Error, Result};
pub use git::{
    cached_repo_path, clone_repo, default_cache_dir, default_repos_cache_dir, fetch_repo,
    is_repo_cached, worktree_path, BranchingOptions, BranchingPoint, CachedWorktree, GitRepo,
    PoolConfig, RemoteInfo, RepoUrl, WorktreeInfo, WorktreeMetadata, WorktreeOptions, WorktreePool,
    WorktreeStatus,
};
pub use plan::{parse_plan, Phase, Plan, PlannedPR};
pub use review::{ReviewContext, ReviewRequest, ReviewRequestBuilder, ReviewType};
pub use secrets::{GitHubSecrets, Secrets};
pub use workflow::{
    build_resume_prompt, find_incomplete_runs, find_latest_incomplete_run,
    reconstruct_conversation, ConversationMessage, PhaseValidation, ResumableRun, StateMachine,
    TddPhase, TddState, TddTransition, TddWorkflow, Workflow,
};
