//! Agent module for spawning and managing Claude Code processes

mod backend;
mod output;
mod prompts;
mod selection;
mod spawn;
mod typed;
mod types;

pub use backend::{Backend, BackendRegistry, ClaudeBackend, CursorBackend};
pub use output::{CostInfo, OutputStreamer, PrintHandler, StreamHandler, StreamMessage};
pub use prompts::{get_template, render, PromptBuilder, PromptContext};
pub use spawn::{AgentHandle, AgentSpawner};
pub use typed::{
    AgentFactory, CoordinatorAgent, ImplementAgent, ReviewAgent, TestAgent, TypedAgent,
};
pub use types::AgentType;
