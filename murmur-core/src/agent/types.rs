//! Agent type definitions for Murmuration
//!
//! Different agent types have specialized prompts and behaviors:
//! - Implement: Writes code to implement features
//! - Test: Writes tests and validates implementations
//! - Review: Reviews code changes and provides feedback
//! - Coordinator: Orchestrates other agents and manages workflow

use serde::{Deserialize, Serialize};
use std::fmt;

/// The type of agent to spawn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    /// Implementation agent - writes code to implement features
    #[default]
    Implement,
    /// Test agent - writes tests and validates implementations
    Test,
    /// Review agent - reviews code changes and provides feedback
    Review,
    /// Coordinator agent - orchestrates workflow and delegates tasks
    Coordinator,
}

impl AgentType {
    /// Get all available agent types
    pub fn all() -> &'static [AgentType] {
        &[
            AgentType::Implement,
            AgentType::Test,
            AgentType::Review,
            AgentType::Coordinator,
        ]
    }

    /// Get the short name for this agent type
    pub fn name(&self) -> &'static str {
        match self {
            AgentType::Implement => "implement",
            AgentType::Test => "test",
            AgentType::Review => "review",
            AgentType::Coordinator => "coordinator",
        }
    }

    /// Get a description of what this agent type does
    pub fn description(&self) -> &'static str {
        match self {
            AgentType::Implement => "Writes code to implement features and fix bugs",
            AgentType::Test => "Writes tests and validates implementations",
            AgentType::Review => "Reviews code changes and provides feedback",
            AgentType::Coordinator => "Orchestrates workflow and delegates to other agents",
        }
    }

    /// Whether this agent type can spawn other agents
    pub fn can_spawn_agents(&self) -> bool {
        matches!(self, AgentType::Coordinator)
    }

    /// Whether this agent type should run in isolation
    pub fn runs_isolated(&self) -> bool {
        // Review agents see the diff but don't modify code
        matches!(self, AgentType::Review)
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "implement" | "impl" | "i" => Ok(AgentType::Implement),
            "test" | "t" => Ok(AgentType::Test),
            "review" | "r" => Ok(AgentType::Review),
            "coordinator" | "coord" | "c" => Ok(AgentType::Coordinator),
            _ => Err(format!("Unknown agent type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_names() {
        assert_eq!(AgentType::Implement.name(), "implement");
        assert_eq!(AgentType::Test.name(), "test");
        assert_eq!(AgentType::Review.name(), "review");
        assert_eq!(AgentType::Coordinator.name(), "coordinator");
    }

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Implement.to_string(), "implement");
        assert_eq!(AgentType::Test.to_string(), "test");
    }

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!(
            "implement".parse::<AgentType>().unwrap(),
            AgentType::Implement
        );
        assert_eq!("impl".parse::<AgentType>().unwrap(), AgentType::Implement);
        assert_eq!("test".parse::<AgentType>().unwrap(), AgentType::Test);
        assert_eq!("review".parse::<AgentType>().unwrap(), AgentType::Review);
        assert_eq!(
            "coordinator".parse::<AgentType>().unwrap(),
            AgentType::Coordinator
        );
        assert_eq!(
            "coord".parse::<AgentType>().unwrap(),
            AgentType::Coordinator
        );
    }

    #[test]
    fn test_agent_type_from_str_case_insensitive() {
        assert_eq!(
            "IMPLEMENT".parse::<AgentType>().unwrap(),
            AgentType::Implement
        );
        assert_eq!("Test".parse::<AgentType>().unwrap(), AgentType::Test);
    }

    #[test]
    fn test_agent_type_from_str_invalid() {
        assert!("invalid".parse::<AgentType>().is_err());
    }

    #[test]
    fn test_can_spawn_agents() {
        assert!(!AgentType::Implement.can_spawn_agents());
        assert!(!AgentType::Test.can_spawn_agents());
        assert!(!AgentType::Review.can_spawn_agents());
        assert!(AgentType::Coordinator.can_spawn_agents());
    }

    #[test]
    fn test_runs_isolated() {
        assert!(!AgentType::Implement.runs_isolated());
        assert!(!AgentType::Test.runs_isolated());
        assert!(AgentType::Review.runs_isolated());
        assert!(!AgentType::Coordinator.runs_isolated());
    }

    #[test]
    fn test_all_agent_types() {
        let all = AgentType::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&AgentType::Implement));
        assert!(all.contains(&AgentType::Test));
        assert!(all.contains(&AgentType::Review));
        assert!(all.contains(&AgentType::Coordinator));
    }

    #[test]
    fn test_default() {
        assert_eq!(AgentType::default(), AgentType::Implement);
    }

    #[test]
    fn test_serde_roundtrip() {
        let agent_type = AgentType::Test;
        let json = serde_json::to_string(&agent_type).unwrap();
        assert_eq!(json, "\"test\"");
        let parsed: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, agent_type);
    }
}
