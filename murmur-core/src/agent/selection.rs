//! Agent selection logic
//!
//! This module provides automatic agent type selection based on task characteristics
//! and context.

use crate::agent::AgentType;

/// Hints for agent selection based on task content
#[derive(Debug, Clone, Default)]
pub struct TaskHints {
    /// Keywords that suggest implementation work
    pub implementation_keywords: Vec<String>,
    /// Keywords that suggest testing work
    pub test_keywords: Vec<String>,
    /// Keywords that suggest review work
    pub review_keywords: Vec<String>,
    /// Keywords that suggest coordination work
    pub coordination_keywords: Vec<String>,
}

impl TaskHints {
    /// Create default hints with common keywords
    pub fn default_keywords() -> Self {
        Self {
            implementation_keywords: vec![
                "implement".into(),
                "add".into(),
                "create".into(),
                "build".into(),
                "fix".into(),
                "refactor".into(),
                "update".into(),
                "modify".into(),
                "change".into(),
                "feature".into(),
                "bug".into(),
                "code".into(),
            ],
            test_keywords: vec![
                "test".into(),
                "verify".into(),
                "validate".into(),
                "check".into(),
                "assert".into(),
                "spec".into(),
                "coverage".into(),
                "unittest".into(),
                "integration".into(),
            ],
            review_keywords: vec![
                "review".into(),
                "feedback".into(),
                "critique".into(),
                "evaluate".into(),
                "assess".into(),
                "approve".into(),
                "comment".into(),
                "pr".into(),
                "pull request".into(),
            ],
            coordination_keywords: vec![
                "coordinate".into(),
                "orchestrate".into(),
                "manage".into(),
                "plan".into(),
                "breakdown".into(),
                "delegate".into(),
                "multi".into(),
                "phase".into(),
                "workflow".into(),
            ],
        }
    }
}

/// Select the appropriate agent type based on task description
pub fn select_agent_type(task: &str) -> AgentType {
    select_with_hints(task, &TaskHints::default_keywords())
}

/// Select agent type with custom hints
pub fn select_with_hints(task: &str, hints: &TaskHints) -> AgentType {
    let task_lower = task.to_lowercase();

    // Score each agent type based on keyword matches
    let impl_score = count_matches(&task_lower, &hints.implementation_keywords);
    let test_score = count_matches(&task_lower, &hints.test_keywords);
    let review_score = count_matches(&task_lower, &hints.review_keywords);
    let coord_score = count_matches(&task_lower, &hints.coordination_keywords);

    // Special cases: explicit "write test" or "run tests" should use Test agent
    if task_lower.contains("write test")
        || task_lower.contains("run test")
        || task_lower.contains("add test")
    {
        return AgentType::Test;
    }

    // Special case: "review" anywhere strongly suggests Review agent
    if task_lower.contains("review") {
        return AgentType::Review;
    }

    // Find the highest score
    let max_score = impl_score.max(test_score).max(review_score).max(coord_score);

    if max_score == 0 {
        // No keywords matched, default to Implement
        return AgentType::Implement;
    }

    // Return the type with highest score, with ties broken by preference order
    if coord_score == max_score {
        AgentType::Coordinator
    } else if review_score == max_score {
        AgentType::Review
    } else if test_score == max_score {
        AgentType::Test
    } else {
        AgentType::Implement
    }
}

/// Count how many keywords match in the task
fn count_matches(task: &str, keywords: &[String]) -> usize {
    keywords.iter().filter(|kw| task.contains(kw.as_str())).count()
}

/// Infer agent type from file patterns
pub fn infer_from_files(files: &[String]) -> Option<AgentType> {
    let mut test_files = 0;
    let mut impl_files = 0;

    for file in files {
        let lower = file.to_lowercase();
        if lower.contains("test")
            || lower.contains("spec")
            || lower.ends_with("_test.rs")
            || lower.ends_with("_test.go")
            || lower.ends_with(".test.ts")
            || lower.ends_with(".test.js")
            || lower.ends_with("_spec.rb")
        {
            test_files += 1;
        } else {
            impl_files += 1;
        }
    }

    if test_files > 0 && impl_files == 0 {
        Some(AgentType::Test)
    } else if impl_files > 0 && test_files == 0 {
        Some(AgentType::Implement)
    } else {
        None // Mixed files, can't determine
    }
}

/// Suggest agent based on task and files
pub fn suggest_agent(task: &str, files: &[String]) -> AgentType {
    // First check file patterns
    if let Some(agent) = infer_from_files(files) {
        return agent;
    }

    // Fall back to task analysis
    select_agent_type(task)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_implement() {
        assert_eq!(select_agent_type("Implement feature X"), AgentType::Implement);
        assert_eq!(select_agent_type("Add new button"), AgentType::Implement);
        assert_eq!(select_agent_type("Fix the login bug"), AgentType::Implement);
    }

    #[test]
    fn test_select_test() {
        assert_eq!(select_agent_type("Write tests for feature X"), AgentType::Test);
        assert_eq!(select_agent_type("Add test coverage"), AgentType::Test);
        assert_eq!(select_agent_type("Run the test suite"), AgentType::Test);
    }

    #[test]
    fn test_select_review() {
        assert_eq!(select_agent_type("Review the changes"), AgentType::Review);
        assert_eq!(select_agent_type("Provide feedback on PR"), AgentType::Review);
        assert_eq!(select_agent_type("Code review"), AgentType::Review);
    }

    #[test]
    fn test_select_coordinator() {
        assert_eq!(select_agent_type("Coordinate the implementation across modules"), AgentType::Coordinator);
        assert_eq!(select_agent_type("Orchestrate the multi-phase deployment"), AgentType::Coordinator);
        assert_eq!(select_agent_type("Plan and delegate the work"), AgentType::Coordinator);
    }

    #[test]
    fn test_default_to_implement() {
        assert_eq!(select_agent_type("Do the thing"), AgentType::Implement);
        assert_eq!(select_agent_type(""), AgentType::Implement);
    }

    #[test]
    fn test_infer_from_test_files() {
        let files = vec!["src/auth_test.rs".to_string()];
        assert_eq!(infer_from_files(&files), Some(AgentType::Test));
    }

    #[test]
    fn test_infer_from_impl_files() {
        let files = vec!["src/auth.rs".to_string()];
        assert_eq!(infer_from_files(&files), Some(AgentType::Implement));
    }

    #[test]
    fn test_infer_mixed_files() {
        let files = vec!["src/auth.rs".to_string(), "src/auth_test.rs".to_string()];
        assert_eq!(infer_from_files(&files), None);
    }

    #[test]
    fn test_suggest_prefers_files() {
        // Even though task says "implement", test files suggest Test agent
        let files = vec!["src/auth_test.rs".to_string()];
        assert_eq!(suggest_agent("Implement feature", &files), AgentType::Test);
    }

    #[test]
    fn test_suggest_fallback_to_task() {
        let files = vec![];
        assert_eq!(suggest_agent("Write tests", &files), AgentType::Test);
    }

    #[test]
    fn test_explicit_write_test() {
        // "write test" should always be Test, even with other keywords
        assert_eq!(select_agent_type("Implement and write tests"), AgentType::Test);
    }
}
