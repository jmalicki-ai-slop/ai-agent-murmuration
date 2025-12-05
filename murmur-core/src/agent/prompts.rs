//! Agent prompt templates
//!
//! This module provides embedded prompt templates for different agent types.
//! Templates use `{{VARIABLE}}` placeholders that can be rendered with context.

use crate::agent::AgentType;
use std::collections::HashMap;

/// Embedded prompt templates for each agent type
const IMPLEMENT_PROMPT: &str = include_str!("prompts/implement.md");
const TEST_PROMPT: &str = include_str!("prompts/test.md");
const REVIEW_PROMPT: &str = include_str!("prompts/review.md");
const COORDINATOR_PROMPT: &str = include_str!("prompts/coordinator.md");

/// Get the raw prompt template for an agent type
pub fn get_template(agent_type: AgentType) -> &'static str {
    match agent_type {
        AgentType::Implement => IMPLEMENT_PROMPT,
        AgentType::Test => TEST_PROMPT,
        AgentType::Review => REVIEW_PROMPT,
        AgentType::Coordinator => COORDINATOR_PROMPT,
    }
}

/// Context for rendering a prompt template
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    /// Variable substitutions
    variables: HashMap<String, String>,
}

impl PromptContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Set a variable value (builder pattern)
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.set(key, value);
        self
    }

    /// Set the task description
    pub fn with_task(self, description: impl Into<String>) -> Self {
        self.with("TASK_DESCRIPTION", description)
    }

    /// Set the files to work on
    pub fn with_files(self, files: &[String]) -> Self {
        let files_str = if files.is_empty() {
            "(no specific files)".to_string()
        } else {
            files
                .iter()
                .map(|f| format!("- `{}`", f))
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.with("FILES", files_str)
    }

    /// Set the dependencies
    pub fn with_dependencies(self, deps: &[String]) -> Self {
        let deps_str = if deps.is_empty() {
            "(no dependencies)".to_string()
        } else {
            deps.iter()
                .map(|d| format!("- {}", d))
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.with("DEPENDENCIES", deps_str)
    }

    /// Set the diff for review
    pub fn with_diff(self, diff: impl Into<String>) -> Self {
        self.with("DIFF", diff)
    }

    /// Set the repository
    pub fn with_repo(self, repo: impl Into<String>) -> Self {
        self.with("REPO", repo)
    }

    /// Set the main branch
    pub fn with_main_branch(self, branch: impl Into<String>) -> Self {
        self.with("MAIN_BRANCH", branch)
    }
}

/// Render a prompt template with the given context
pub fn render(agent_type: AgentType, context: &PromptContext) -> String {
    let template = get_template(agent_type);
    render_template(template, context)
}

/// Render a template string with variable substitution
fn render_template(template: &str, context: &PromptContext) -> String {
    let mut result = template.to_string();

    for (key, value) in &context.variables {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    // Remove any remaining unset placeholders (simple pattern matching)
    // Replace {{UPPERCASE_NAME}} with "(not specified)"
    loop {
        let start = result.find("{{");
        let end = result.find("}}");

        match (start, end) {
            (Some(s), Some(e)) if s < e => {
                let placeholder = &result[s..=e + 1];
                // Check if it's an uppercase placeholder
                let inside = &result[s + 2..e];
                if inside.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                    result = result.replacen(placeholder, "(not specified)", 1);
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    result
}

/// Build a complete prompt for an agent
pub struct PromptBuilder {
    agent_type: AgentType,
    context: PromptContext,
}

impl PromptBuilder {
    /// Create a new prompt builder for the given agent type
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            context: PromptContext::new(),
        }
    }

    /// Set the task description
    pub fn task(mut self, description: impl Into<String>) -> Self {
        self.context = self.context.with_task(description);
        self
    }

    /// Set the files to work on
    pub fn files(mut self, files: &[String]) -> Self {
        self.context = self.context.with_files(files);
        self
    }

    /// Set the dependencies
    pub fn dependencies(mut self, deps: &[String]) -> Self {
        self.context = self.context.with_dependencies(deps);
        self
    }

    /// Set the diff (for review agent)
    pub fn diff(mut self, diff: impl Into<String>) -> Self {
        self.context = self.context.with_diff(diff);
        self
    }

    /// Set the repository (for coordinator)
    pub fn repo(mut self, repo: impl Into<String>) -> Self {
        self.context = self.context.with_repo(repo);
        self
    }

    /// Set the main branch (for coordinator)
    pub fn main_branch(mut self, branch: impl Into<String>) -> Self {
        self.context = self.context.with_main_branch(branch);
        self
    }

    /// Set a custom variable
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context = self.context.with(key, value);
        self
    }

    /// Build the final prompt
    pub fn build(self) -> String {
        render(self.agent_type, &self.context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template() {
        let template = get_template(AgentType::Implement);
        assert!(template.contains("# Implement Agent"));
        assert!(template.contains("{{TASK_DESCRIPTION}}"));
    }

    #[test]
    fn test_render_with_variables() {
        let context = PromptContext::new()
            .with_task("Implement feature X")
            .with_files(&["src/main.rs".to_string()]);

        let rendered = render(AgentType::Implement, &context);
        assert!(rendered.contains("Implement feature X"));
        assert!(rendered.contains("`src/main.rs`"));
    }

    #[test]
    fn test_render_empty_context() {
        let context = PromptContext::new();
        let rendered = render(AgentType::Implement, &context);
        assert!(rendered.contains("(not specified)"));
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new(AgentType::Test)
            .task("Test the login function")
            .files(&["src/auth.rs".to_string()])
            .build();

        assert!(prompt.contains("# Test Agent"));
        assert!(prompt.contains("Test the login function"));
        assert!(prompt.contains("`src/auth.rs`"));
    }

    #[test]
    fn test_with_dependencies() {
        let context = PromptContext::new()
            .with_task("Implement Y")
            .with_dependencies(&["PR-001".to_string(), "PR-002".to_string()]);

        let rendered = render(AgentType::Implement, &context);
        assert!(rendered.contains("- PR-001"));
        assert!(rendered.contains("- PR-002"));
    }

    #[test]
    fn test_review_prompt_with_diff() {
        let prompt = PromptBuilder::new(AgentType::Review)
            .task("Review changes")
            .diff("+ added line\n- removed line")
            .build();

        assert!(prompt.contains("# Review Agent"));
        assert!(prompt.contains("+ added line"));
    }

    #[test]
    fn test_coordinator_prompt() {
        let prompt = PromptBuilder::new(AgentType::Coordinator)
            .task("Implement feature")
            .repo("owner/repo")
            .main_branch("main")
            .build();

        assert!(prompt.contains("# Coordinator Agent"));
        assert!(prompt.contains("owner/repo"));
        assert!(prompt.contains("main"));
    }

    #[test]
    fn test_empty_files_list() {
        let context = PromptContext::new().with_task("Task").with_files(&[]);

        let rendered = render(AgentType::Implement, &context);
        assert!(rendered.contains("(no specific files)"));
    }

    #[test]
    fn test_empty_dependencies_list() {
        let context = PromptContext::new()
            .with_task("Task")
            .with_dependencies(&[]);

        let rendered = render(AgentType::Implement, &context);
        assert!(rendered.contains("(no dependencies)"));
    }
}
