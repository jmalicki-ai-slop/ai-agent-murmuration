//! Agent module for spawning and managing Claude Code processes

mod output;
mod prompts;
mod selection;
mod spawn;
mod typed;
mod types;

pub use output::{CostInfo, OutputStreamer, PrintHandler, StreamHandler, StreamMessage};
pub use prompts::{get_template, render, PromptBuilder, PromptContext};
pub use spawn::{AgentHandle, AgentSpawner};
pub use typed::{
    AgentFactory, CoordinatorAgent, ImplementAgent, ReviewAgent, TestAgent, TypedAgent,
};
pub use types::AgentType;
