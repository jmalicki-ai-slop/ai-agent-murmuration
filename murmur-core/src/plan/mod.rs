//! Plan parsing and management
//!
//! This module handles parsing of PLAN.md files that describe
//! the project structure, phases, and PRs.

mod parser;

pub use parser::{parse_plan, Phase, Plan, PlannedPR};
