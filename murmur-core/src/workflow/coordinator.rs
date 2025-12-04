//! Coordinator workflow for orchestrating multi-agent development
//!
//! The coordinator manages the full development workflow:
//! 1. Analyzes tasks and breaks them down
//! 2. Delegates to appropriate agents (Implement, Test, Review)
//! 3. Manages worktrees and branches
//! 4. Creates PRs when work is complete

use crate::agent::{AgentFactory, AgentType, CoordinatorAgent};
use crate::config::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Overall state of the coordination workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CoordinatorPhase {
    /// Analyzing the task and planning
    #[default]
    Planning,
    /// Creating worktree for isolated development
    SetupWorktree,
    /// Running implementation (possibly with TDD)
    Implementing,
    /// Running tests to verify implementation
    Testing,
    /// Running code review
    Reviewing,
    /// Creating the PR
    CreatingPR,
    /// Workflow complete
    Complete,
    /// Workflow failed
    Failed,
}

impl CoordinatorPhase {
    /// Get the next phase in a typical workflow
    pub fn next(&self) -> Option<CoordinatorPhase> {
        match self {
            CoordinatorPhase::Planning => Some(CoordinatorPhase::SetupWorktree),
            CoordinatorPhase::SetupWorktree => Some(CoordinatorPhase::Implementing),
            CoordinatorPhase::Implementing => Some(CoordinatorPhase::Testing),
            CoordinatorPhase::Testing => Some(CoordinatorPhase::Reviewing),
            CoordinatorPhase::Reviewing => Some(CoordinatorPhase::CreatingPR),
            CoordinatorPhase::CreatingPR => Some(CoordinatorPhase::Complete),
            CoordinatorPhase::Complete | CoordinatorPhase::Failed => None,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CoordinatorPhase::Planning => "Analyzing task and planning work breakdown",
            CoordinatorPhase::SetupWorktree => "Setting up isolated worktree",
            CoordinatorPhase::Implementing => "Implementing changes",
            CoordinatorPhase::Testing => "Running tests",
            CoordinatorPhase::Reviewing => "Reviewing code changes",
            CoordinatorPhase::CreatingPR => "Creating pull request",
            CoordinatorPhase::Complete => "Workflow complete",
            CoordinatorPhase::Failed => "Workflow failed",
        }
    }

    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, CoordinatorPhase::Complete | CoordinatorPhase::Failed)
    }
}

impl std::fmt::Display for CoordinatorPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A subtask identified during planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    /// Unique ID for the subtask
    pub id: String,
    /// Description of what needs to be done
    pub description: String,
    /// Files involved
    pub files: Vec<String>,
    /// Agent type to handle this subtask
    pub agent_type: AgentType,
    /// Dependencies on other subtasks
    pub depends_on: Vec<String>,
    /// Current status
    pub status: SubTaskStatus,
}

/// Status of a subtask
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SubTaskStatus {
    #[default]
    Pending,
    InProgress,
    Complete,
    Failed,
}

impl SubTask {
    /// Create a new subtask
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            files: Vec::new(),
            agent_type: AgentType::Implement,
            depends_on: Vec::new(),
            status: SubTaskStatus::Pending,
        }
    }

    /// Set the agent type
    pub fn with_agent_type(mut self, agent_type: AgentType) -> Self {
        self.agent_type = agent_type;
        self
    }

    /// Add files
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    /// Add dependencies
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.depends_on = deps;
        self
    }

    /// Check if ready to execute (all dependencies complete)
    pub fn is_ready(&self, completed: &[String]) -> bool {
        self.status == SubTaskStatus::Pending
            && self.depends_on.iter().all(|dep| completed.contains(dep))
    }
}

/// Configuration for the coordinator workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorConfig {
    /// Whether to use TDD workflow
    pub use_tdd: bool,
    /// Whether to run review after implementation
    pub run_review: bool,
    /// Whether to auto-create PR
    pub auto_pr: bool,
    /// Maximum retries per phase
    pub max_retries: u32,
    /// Repository (owner/repo)
    pub repo: Option<String>,
    /// Main branch name
    pub main_branch: String,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            use_tdd: false,
            run_review: true,
            auto_pr: false,
            max_retries: 2,
            repo: None,
            main_branch: "main".to_string(),
        }
    }
}

/// State for the coordinator workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorState {
    /// Current phase
    pub phase: CoordinatorPhase,
    /// The main task being coordinated
    pub task: String,
    /// Working directory for the project
    pub project_dir: PathBuf,
    /// Worktree path (set after setup)
    pub worktree_path: Option<PathBuf>,
    /// Branch name
    pub branch_name: Option<String>,
    /// Subtasks identified during planning
    pub subtasks: Vec<SubTask>,
    /// Completed subtask IDs
    pub completed_subtasks: Vec<String>,
    /// Phase history
    pub history: Vec<PhaseTransition>,
    /// Number of retries in current phase
    pub retries: u32,
    /// Configuration
    pub config: CoordinatorConfig,
    /// Any error message
    pub error: Option<String>,
}

/// A phase transition in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub from: CoordinatorPhase,
    pub to: CoordinatorPhase,
    pub success: bool,
    pub message: Option<String>,
}

impl CoordinatorState {
    /// Create a new coordinator state
    pub fn new(task: impl Into<String>, project_dir: impl Into<PathBuf>) -> Self {
        Self {
            phase: CoordinatorPhase::Planning,
            task: task.into(),
            project_dir: project_dir.into(),
            worktree_path: None,
            branch_name: None,
            subtasks: Vec::new(),
            completed_subtasks: Vec::new(),
            history: Vec::new(),
            retries: 0,
            config: CoordinatorConfig::default(),
            error: None,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: CoordinatorConfig) -> Self {
        self.config = config;
        self
    }

    /// Advance to the next phase
    pub fn advance(&mut self, success: bool, message: Option<String>) -> Option<CoordinatorPhase> {
        let from = self.phase;
        let to = if success {
            self.phase.next()
        } else {
            Some(CoordinatorPhase::Failed)
        };

        if let Some(next_phase) = to {
            self.history.push(PhaseTransition {
                from,
                to: next_phase,
                success,
                message,
            });
            self.phase = next_phase;
            self.retries = 0;
            Some(next_phase)
        } else {
            None
        }
    }

    /// Retry the current phase
    pub fn retry(&mut self) -> bool {
        if self.retries < self.config.max_retries {
            self.retries += 1;
            true
        } else {
            false
        }
    }

    /// Check if workflow is complete
    pub fn is_complete(&self) -> bool {
        self.phase == CoordinatorPhase::Complete
    }

    /// Check if workflow failed
    pub fn is_failed(&self) -> bool {
        self.phase == CoordinatorPhase::Failed
    }

    /// Set worktree info
    pub fn set_worktree(&mut self, path: PathBuf, branch: String) {
        self.worktree_path = Some(path);
        self.branch_name = Some(branch);
    }

    /// Add subtasks from planning
    pub fn set_subtasks(&mut self, subtasks: Vec<SubTask>) {
        self.subtasks = subtasks;
    }

    /// Mark a subtask as complete
    pub fn complete_subtask(&mut self, id: &str) {
        if let Some(subtask) = self.subtasks.iter_mut().find(|s| s.id == id) {
            subtask.status = SubTaskStatus::Complete;
            self.completed_subtasks.push(id.to_string());
        }
    }

    /// Get the next ready subtask
    pub fn next_subtask(&self) -> Option<&SubTask> {
        self.subtasks
            .iter()
            .find(|s| s.is_ready(&self.completed_subtasks))
    }

    /// Check if all subtasks are complete
    pub fn all_subtasks_complete(&self) -> bool {
        self.subtasks
            .iter()
            .all(|s| s.status == SubTaskStatus::Complete)
    }
}

/// The coordinator workflow
#[derive(Debug)]
pub struct CoordinatorWorkflow {
    state: CoordinatorState,
    factory: AgentFactory,
}

impl CoordinatorWorkflow {
    /// Create a new coordinator workflow
    pub fn new(task: impl Into<String>, project_dir: impl Into<PathBuf>) -> Self {
        Self {
            state: CoordinatorState::new(task, project_dir),
            factory: AgentFactory::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        task: impl Into<String>,
        project_dir: impl Into<PathBuf>,
        config: CoordinatorConfig,
        agent_config: AgentConfig,
    ) -> Self {
        Self {
            state: CoordinatorState::new(task, project_dir).with_config(config),
            factory: AgentFactory::with_config(agent_config),
        }
    }

    /// Get the current state
    pub fn state(&self) -> &CoordinatorState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut CoordinatorState {
        &mut self.state
    }

    /// Get the current phase
    pub fn phase(&self) -> CoordinatorPhase {
        self.state.phase
    }

    /// Get the coordinator agent
    pub fn coordinator_agent(&self) -> CoordinatorAgent {
        self.factory.coordinator()
    }

    /// Get the prompt for the current phase
    pub fn current_prompt(&self) -> String {
        match self.state.phase {
            CoordinatorPhase::Planning => self.planning_prompt(),
            CoordinatorPhase::SetupWorktree => self.worktree_prompt(),
            CoordinatorPhase::Implementing => self.implement_prompt(),
            CoordinatorPhase::Testing => self.test_prompt(),
            CoordinatorPhase::Reviewing => self.review_prompt(),
            CoordinatorPhase::CreatingPR => self.pr_prompt(),
            CoordinatorPhase::Complete => "Workflow complete.".to_string(),
            CoordinatorPhase::Failed => format!(
                "Workflow failed: {}",
                self.state.error.as_deref().unwrap_or("Unknown error")
            ),
        }
    }

    fn planning_prompt(&self) -> String {
        format!(
            "Analyze the following task and break it down into subtasks:\n\n{}\n\n\
             For each subtask, identify:\n\
             1. What needs to be done\n\
             2. Which files are involved\n\
             3. What type of work it is (implement, test, review)\n\
             4. Dependencies on other subtasks\n\n\
             Output your plan in a structured format.",
            self.state.task
        )
    }

    fn worktree_prompt(&self) -> String {
        "Create a new git worktree for isolated development.".to_string()
    }

    fn implement_prompt(&self) -> String {
        if let Some(subtask) = self.state.next_subtask() {
            format!(
                "Implement the following subtask:\n\n{}\n\nFiles: {:?}",
                subtask.description, subtask.files
            )
        } else {
            "Implementation phase - no subtasks remaining.".to_string()
        }
    }

    fn test_prompt(&self) -> String {
        "Run the test suite and verify all tests pass.".to_string()
    }

    fn review_prompt(&self) -> String {
        "Review the code changes and provide feedback.".to_string()
    }

    fn pr_prompt(&self) -> String {
        format!(
            "Create a pull request for the changes.\n\
             Branch: {}\n\
             Task: {}",
            self.state.branch_name.as_deref().unwrap_or("(unknown)"),
            self.state.task
        )
    }

    /// Advance to the next phase
    pub fn advance(&mut self, success: bool, message: Option<String>) -> Option<CoordinatorPhase> {
        self.state.advance(success, message)
    }

    /// Retry the current phase
    pub fn retry(&mut self) -> bool {
        self.state.retry()
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Check if failed
    pub fn is_failed(&self) -> bool {
        self.state.is_failed()
    }

    /// Set an error and transition to failed state
    pub fn fail(&mut self, error: impl Into<String>) {
        self.state.error = Some(error.into());
        self.state.phase = CoordinatorPhase::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transitions() {
        assert_eq!(
            CoordinatorPhase::Planning.next(),
            Some(CoordinatorPhase::SetupWorktree)
        );
        assert_eq!(
            CoordinatorPhase::Implementing.next(),
            Some(CoordinatorPhase::Testing)
        );
        assert!(CoordinatorPhase::Complete.is_terminal());
        assert!(CoordinatorPhase::Failed.is_terminal());
    }

    #[test]
    fn test_subtask_ready() {
        let task = SubTask::new("task-1", "Do something").with_dependencies(vec!["task-0".into()]);
        assert!(!task.is_ready(&[]));
        assert!(task.is_ready(&["task-0".into()]));
    }

    #[test]
    fn test_state_new() {
        let state = CoordinatorState::new("Build feature", "/tmp/project");
        assert_eq!(state.phase, CoordinatorPhase::Planning);
        assert!(state.subtasks.is_empty());
    }

    #[test]
    fn test_state_advance() {
        let mut state = CoordinatorState::new("task", "/tmp");
        let next = state.advance(true, None);
        assert_eq!(next, Some(CoordinatorPhase::SetupWorktree));
        assert_eq!(state.phase, CoordinatorPhase::SetupWorktree);
    }

    #[test]
    fn test_state_advance_failure() {
        let mut state = CoordinatorState::new("task", "/tmp");
        let next = state.advance(false, Some("error".into()));
        assert_eq!(next, Some(CoordinatorPhase::Failed));
        assert_eq!(state.phase, CoordinatorPhase::Failed);
    }

    #[test]
    fn test_state_retry() {
        let mut state = CoordinatorState::new("task", "/tmp");
        assert!(state.retry()); // 1
        assert!(state.retry()); // 2
        assert!(!state.retry()); // exceeds max
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = CoordinatorWorkflow::new("Build feature", "/tmp/project");
        assert_eq!(workflow.phase(), CoordinatorPhase::Planning);
        assert!(!workflow.is_complete());
    }

    #[test]
    fn test_workflow_prompts() {
        let workflow = CoordinatorWorkflow::new("Build feature", "/tmp/project");
        let prompt = workflow.current_prompt();
        assert!(prompt.contains("Build feature"));
        assert!(prompt.contains("subtasks"));
    }

    #[test]
    fn test_subtask_completion() {
        let mut state = CoordinatorState::new("task", "/tmp");
        state.set_subtasks(vec![
            SubTask::new("task-1", "First task"),
            SubTask::new("task-2", "Second task").with_dependencies(vec!["task-1".into()]),
        ]);

        // task-2 is not ready yet
        let next = state.next_subtask();
        assert_eq!(next.map(|s| s.id.as_str()), Some("task-1"));

        // Complete task-1
        state.complete_subtask("task-1");
        let next = state.next_subtask();
        assert_eq!(next.map(|s| s.id.as_str()), Some("task-2"));
    }

    #[test]
    fn test_all_subtasks_complete() {
        let mut state = CoordinatorState::new("task", "/tmp");
        state.set_subtasks(vec![SubTask::new("task-1", "Task")]);

        assert!(!state.all_subtasks_complete());
        state.complete_subtask("task-1");
        assert!(state.all_subtasks_complete());
    }

    #[test]
    fn test_workflow_fail() {
        let mut workflow = CoordinatorWorkflow::new("task", "/tmp");
        workflow.fail("Something went wrong");

        assert!(workflow.is_failed());
        assert_eq!(
            workflow.state().error,
            Some("Something went wrong".to_string())
        );
    }
}
