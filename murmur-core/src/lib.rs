//! Murmur Core - Core library for Murmuration multi-agent orchestration
//!
//! This crate provides the core functionality for orchestrating multiple
//! AI agents working collaboratively on software development tasks.

pub mod agent;
pub mod error;

pub use agent::{
    AgentHandle, AgentSpawner, CostInfo, OutputStreamer, PrintHandler, StreamHandler, StreamMessage,
};
pub use error::{Error, Result};
