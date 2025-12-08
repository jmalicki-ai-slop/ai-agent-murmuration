//! Coordinator module for orchestrating multi-agent workflows
//!
//! The coordinator is a meta-agent that manages the execution of other agents
//! (Implement, Test, Review) to complete complex tasks. It handles:
//!
//! - Task decomposition and delegation
//! - Agent spawning and lifecycle management
//! - Workflow state tracking
//! - Error handling and retry logic
//! - Escalation to humans when needed

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::agent::AgentType;

/// Configuration for the coordinator agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoordinatorConfig {
    /// Maximum number of retry attempts for a failed agent task
    pub max_retries: u32,

    /// Maximum number of iterations for the feedback loop
    pub max_iterations: u32,

    /// Timeout for individual agent tasks
    #[serde(with = "humantime_serde")]
    pub agent_timeout: Duration,

    /// Timeout for the entire coordination workflow
    #[serde(with = "humantime_serde")]
    pub workflow_timeout: Duration,

    /// Whether to automatically escalate to human on max retries
    pub auto_escalate: bool,

    /// Whether to require review before completing
    pub require_review: bool,

    /// Whether to use TDD workflow for implementation tasks
    pub use_tdd: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            max_iterations: 5,
            agent_timeout: Duration::from_secs(600), // 10 minutes
            workflow_timeout: Duration::from_secs(3600), // 1 hour
            auto_escalate: true,
            require_review: true,
            use_tdd: true,
        }
    }
}

/// The current phase of the coordination workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinatorPhase {
    /// Initial phase - analyzing the task
    Analyzing,
    /// Planning phase - breaking down the task
    Planning,
    /// Test writing phase (TDD RED)
    WritingTests,
    /// Verifying tests fail (TDD RED verification)
    VerifyingRed,
    /// Implementation phase
    Implementing,
    /// Verifying tests pass (TDD GREEN verification)
    VerifyingGreen,
    /// Review phase
    Reviewing,
    /// Handling review feedback
    AddressingFeedback,
    /// Finalizing (commit, push, PR)
    Finalizing,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed and escalated
    Escalated,
}

impl CoordinatorPhase {
    /// Get the display name for this phase
    pub fn name(&self) -> &'static str {
        match self {
            CoordinatorPhase::Analyzing => "analyzing",
            CoordinatorPhase::Planning => "planning",
            CoordinatorPhase::WritingTests => "writing_tests",
            CoordinatorPhase::VerifyingRed => "verifying_red",
            CoordinatorPhase::Implementing => "implementing",
            CoordinatorPhase::VerifyingGreen => "verifying_green",
            CoordinatorPhase::Reviewing => "reviewing",
            CoordinatorPhase::AddressingFeedback => "addressing_feedback",
            CoordinatorPhase::Finalizing => "finalizing",
            CoordinatorPhase::Completed => "completed",
            CoordinatorPhase::Escalated => "escalated",
        }
    }

    /// Check if this is a terminal phase
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            CoordinatorPhase::Completed | CoordinatorPhase::Escalated
        )
    }

    /// Get the agent type that handles this phase, if any
    pub fn agent_type(&self) -> Option<AgentType> {
        match self {
            CoordinatorPhase::WritingTests
            | CoordinatorPhase::VerifyingRed
            | CoordinatorPhase::VerifyingGreen => Some(AgentType::Test),
            CoordinatorPhase::Implementing | CoordinatorPhase::AddressingFeedback => {
                Some(AgentType::Implement)
            }
            CoordinatorPhase::Reviewing => Some(AgentType::Review),
            _ => None,
        }
    }
}

impl std::fmt::Display for CoordinatorPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A sub-task that the coordinator delegates to another agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    /// Unique identifier for this sub-task
    pub id: String,

    /// Description of what needs to be done
    pub description: String,

    /// The type of agent to handle this task
    pub agent_type: AgentType,

    /// Files relevant to this task
    pub files: Vec<String>,

    /// Dependencies on other sub-tasks (by ID)
    pub depends_on: Vec<String>,

    /// Current status of the sub-task
    pub status: SubTaskStatus,

    /// Number of retry attempts so far
    pub retry_count: u32,

    /// Output/result from the agent (if completed)
    pub output: Option<String>,
}

impl SubTask {
    /// Create a new sub-task
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        agent_type: AgentType,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            agent_type,
            files: Vec::new(),
            depends_on: Vec::new(),
            status: SubTaskStatus::Pending,
            retry_count: 0,
            output: None,
        }
    }

    /// Add files relevant to this task
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = files;
        self
    }

    /// Add dependencies on other sub-tasks
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.depends_on = deps;
        self
    }

    /// Check if this task is ready to execute (all dependencies satisfied)
    pub fn is_ready(&self, completed_tasks: &[String]) -> bool {
        self.status == SubTaskStatus::Pending
            && self
                .depends_on
                .iter()
                .all(|dep| completed_tasks.contains(dep))
    }
}

/// Status of a sub-task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubTaskStatus {
    /// Not yet started
    Pending,
    /// Currently running
    Running,
    /// Completed successfully
    Completed,
    /// Failed (may be retried)
    Failed,
    /// Skipped (dependency failed or not needed)
    Skipped,
}

impl SubTaskStatus {
    /// Check if this status represents a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SubTaskStatus::Completed | SubTaskStatus::Failed | SubTaskStatus::Skipped
        )
    }
}

impl std::fmt::Display for SubTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SubTaskStatus::Pending => "pending",
            SubTaskStatus::Running => "running",
            SubTaskStatus::Completed => "completed",
            SubTaskStatus::Failed => "failed",
            SubTaskStatus::Skipped => "skipped",
        };
        write!(f, "{}", s)
    }
}

/// The complete state of a coordinator workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorState {
    /// The main task being coordinated
    pub task: String,

    /// Repository being worked on
    pub repo: String,

    /// Branch being worked on
    pub branch: String,

    /// Current phase of the workflow
    pub phase: CoordinatorPhase,

    /// Sub-tasks that have been identified
    pub sub_tasks: Vec<SubTask>,

    /// Current iteration count
    pub iteration: u32,

    /// Worktree path where work is happening
    pub worktree_path: Option<String>,

    /// Whether human escalation has been triggered
    pub escalated: bool,

    /// Reason for escalation, if any
    pub escalation_reason: Option<String>,
}

impl CoordinatorState {
    /// Create a new coordinator state for a task
    pub fn new(
        task: impl Into<String>,
        repo: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        Self {
            task: task.into(),
            repo: repo.into(),
            branch: branch.into(),
            phase: CoordinatorPhase::Analyzing,
            sub_tasks: Vec::new(),
            iteration: 0,
            worktree_path: None,
            escalated: false,
            escalation_reason: None,
        }
    }

    /// Advance to the next phase
    pub fn advance_phase(&mut self, next: CoordinatorPhase) {
        self.phase = next;
    }

    /// Add a sub-task
    pub fn add_sub_task(&mut self, task: SubTask) {
        self.sub_tasks.push(task);
    }

    /// Get completed task IDs
    pub fn completed_task_ids(&self) -> Vec<String> {
        self.sub_tasks
            .iter()
            .filter(|t| t.status == SubTaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect()
    }

    /// Get the next task that is ready to run
    pub fn next_ready_task(&self) -> Option<&SubTask> {
        let completed = self.completed_task_ids();
        self.sub_tasks.iter().find(|t| t.is_ready(&completed))
    }

    /// Mark a sub-task as running
    pub fn start_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.sub_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = SubTaskStatus::Running;
            true
        } else {
            false
        }
    }

    /// Mark a sub-task as completed
    pub fn complete_task(&mut self, task_id: &str, output: Option<String>) -> bool {
        if let Some(task) = self.sub_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = SubTaskStatus::Completed;
            task.output = output;
            true
        } else {
            false
        }
    }

    /// Mark a sub-task as failed
    pub fn fail_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.sub_tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = SubTaskStatus::Failed;
            task.retry_count += 1;
            true
        } else {
            false
        }
    }

    /// Reset a failed task for retry
    pub fn retry_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.sub_tasks.iter_mut().find(|t| t.id == task_id) {
            if task.status == SubTaskStatus::Failed {
                task.status = SubTaskStatus::Pending;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if workflow is complete
    pub fn is_complete(&self) -> bool {
        self.phase.is_terminal()
    }

    /// Escalate the workflow to human intervention
    pub fn escalate(&mut self, reason: impl Into<String>) {
        self.escalated = true;
        self.escalation_reason = Some(reason.into());
        self.phase = CoordinatorPhase::Escalated;
    }

    /// Increment the iteration counter
    pub fn increment_iteration(&mut self) {
        self.iteration += 1;
    }
}

/// A transition between coordinator phases
#[derive(Debug, Clone)]
pub struct PhaseTransition {
    /// The phase we're transitioning from
    pub from: CoordinatorPhase,

    /// The phase we're transitioning to
    pub to: CoordinatorPhase,

    /// Reason for the transition
    pub reason: String,

    /// Whether this was a successful progression or a fallback
    pub success: bool,
}

impl PhaseTransition {
    /// Create a successful transition
    pub fn success(
        from: CoordinatorPhase,
        to: CoordinatorPhase,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            from,
            to,
            reason: reason.into(),
            success: true,
        }
    }

    /// Create a fallback transition (error handling)
    pub fn fallback(
        from: CoordinatorPhase,
        to: CoordinatorPhase,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            from,
            to,
            reason: reason.into(),
            success: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_config_defaults() {
        let config = CoordinatorConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.agent_timeout, Duration::from_secs(600));
        assert!(config.auto_escalate);
        assert!(config.require_review);
        assert!(config.use_tdd);
    }

    #[test]
    fn test_coordinator_phase_names() {
        assert_eq!(CoordinatorPhase::Analyzing.name(), "analyzing");
        assert_eq!(CoordinatorPhase::Implementing.name(), "implementing");
        assert_eq!(CoordinatorPhase::Completed.name(), "completed");
    }

    #[test]
    fn test_coordinator_phase_terminal() {
        assert!(!CoordinatorPhase::Analyzing.is_terminal());
        assert!(!CoordinatorPhase::Implementing.is_terminal());
        assert!(CoordinatorPhase::Completed.is_terminal());
        assert!(CoordinatorPhase::Escalated.is_terminal());
    }

    #[test]
    fn test_coordinator_phase_agent_type() {
        assert_eq!(
            CoordinatorPhase::WritingTests.agent_type(),
            Some(AgentType::Test)
        );
        assert_eq!(
            CoordinatorPhase::Implementing.agent_type(),
            Some(AgentType::Implement)
        );
        assert_eq!(
            CoordinatorPhase::Reviewing.agent_type(),
            Some(AgentType::Review)
        );
        assert_eq!(CoordinatorPhase::Analyzing.agent_type(), None);
    }

    #[test]
    fn test_sub_task_creation() {
        let task = SubTask::new("task-1", "Implement feature", AgentType::Implement);
        assert_eq!(task.id, "task-1");
        assert_eq!(task.description, "Implement feature");
        assert_eq!(task.agent_type, AgentType::Implement);
        assert_eq!(task.status, SubTaskStatus::Pending);
        assert_eq!(task.retry_count, 0);
    }

    #[test]
    fn test_sub_task_with_files() {
        let task = SubTask::new("task-1", "Fix bug", AgentType::Implement)
            .with_files(vec!["src/main.rs".to_string()]);
        assert_eq!(task.files, vec!["src/main.rs"]);
    }

    #[test]
    fn test_sub_task_dependencies() {
        let task = SubTask::new("task-2", "Test feature", AgentType::Test)
            .with_dependencies(vec!["task-1".to_string()]);

        // Not ready if dependency not completed
        assert!(!task.is_ready(&[]));

        // Ready when dependency is completed
        assert!(task.is_ready(&["task-1".to_string()]));
    }

    #[test]
    fn test_sub_task_status_terminal() {
        assert!(!SubTaskStatus::Pending.is_terminal());
        assert!(!SubTaskStatus::Running.is_terminal());
        assert!(SubTaskStatus::Completed.is_terminal());
        assert!(SubTaskStatus::Failed.is_terminal());
        assert!(SubTaskStatus::Skipped.is_terminal());
    }

    #[test]
    fn test_coordinator_state_creation() {
        let state = CoordinatorState::new("Implement feature X", "owner/repo", "main");
        assert_eq!(state.task, "Implement feature X");
        assert_eq!(state.repo, "owner/repo");
        assert_eq!(state.branch, "main");
        assert_eq!(state.phase, CoordinatorPhase::Analyzing);
        assert!(state.sub_tasks.is_empty());
        assert_eq!(state.iteration, 0);
        assert!(!state.escalated);
    }

    #[test]
    fn test_coordinator_state_add_sub_task() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        state.add_sub_task(SubTask::new("task-1", "Sub task", AgentType::Implement));
        assert_eq!(state.sub_tasks.len(), 1);
    }

    #[test]
    fn test_coordinator_state_task_lifecycle() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        state.add_sub_task(SubTask::new("task-1", "Sub task", AgentType::Implement));

        // Start task
        assert!(state.start_task("task-1"));
        assert_eq!(state.sub_tasks[0].status, SubTaskStatus::Running);

        // Complete task
        assert!(state.complete_task("task-1", Some("Done".to_string())));
        assert_eq!(state.sub_tasks[0].status, SubTaskStatus::Completed);
        assert_eq!(state.sub_tasks[0].output, Some("Done".to_string()));
    }

    #[test]
    fn test_coordinator_state_task_failure_and_retry() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        state.add_sub_task(SubTask::new("task-1", "Sub task", AgentType::Implement));
        state.start_task("task-1");

        // Fail task
        assert!(state.fail_task("task-1"));
        assert_eq!(state.sub_tasks[0].status, SubTaskStatus::Failed);
        assert_eq!(state.sub_tasks[0].retry_count, 1);

        // Retry task
        assert!(state.retry_task("task-1"));
        assert_eq!(state.sub_tasks[0].status, SubTaskStatus::Pending);
        assert_eq!(state.sub_tasks[0].retry_count, 1); // Count preserved
    }

    #[test]
    fn test_coordinator_state_next_ready_task() {
        let mut state = CoordinatorState::new("Task", "repo", "main");

        // Add independent task
        state.add_sub_task(SubTask::new("task-1", "First", AgentType::Implement));

        // Add dependent task
        state.add_sub_task(
            SubTask::new("task-2", "Second", AgentType::Test)
                .with_dependencies(vec!["task-1".to_string()]),
        );

        // task-1 should be ready first
        let next = state.next_ready_task();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, "task-1");

        // Complete task-1
        state.start_task("task-1");
        state.complete_task("task-1", None);

        // Now task-2 should be ready
        let next = state.next_ready_task();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, "task-2");
    }

    #[test]
    fn test_coordinator_state_escalation() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        assert!(!state.escalated);
        assert!(!state.is_complete());

        state.escalate("Max retries exceeded");
        assert!(state.escalated);
        assert_eq!(state.phase, CoordinatorPhase::Escalated);
        assert_eq!(
            state.escalation_reason,
            Some("Max retries exceeded".to_string())
        );
        assert!(state.is_complete());
    }

    #[test]
    fn test_coordinator_state_advance_phase() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        assert_eq!(state.phase, CoordinatorPhase::Analyzing);

        state.advance_phase(CoordinatorPhase::Planning);
        assert_eq!(state.phase, CoordinatorPhase::Planning);
    }

    #[test]
    fn test_phase_transition_success() {
        let transition = PhaseTransition::success(
            CoordinatorPhase::Implementing,
            CoordinatorPhase::Reviewing,
            "Implementation complete",
        );
        assert!(transition.success);
        assert_eq!(transition.from, CoordinatorPhase::Implementing);
        assert_eq!(transition.to, CoordinatorPhase::Reviewing);
    }

    #[test]
    fn test_phase_transition_fallback() {
        let transition = PhaseTransition::fallback(
            CoordinatorPhase::Implementing,
            CoordinatorPhase::AddressingFeedback,
            "Review found issues",
        );
        assert!(!transition.success);
        assert_eq!(transition.to, CoordinatorPhase::AddressingFeedback);
    }

    #[test]
    fn test_completed_task_ids() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        state.add_sub_task(SubTask::new("task-1", "First", AgentType::Implement));
        state.add_sub_task(SubTask::new("task-2", "Second", AgentType::Test));

        state.start_task("task-1");
        state.complete_task("task-1", None);

        let completed = state.completed_task_ids();
        assert_eq!(completed, vec!["task-1".to_string()]);
    }

    #[test]
    fn test_increment_iteration() {
        let mut state = CoordinatorState::new("Task", "repo", "main");
        assert_eq!(state.iteration, 0);

        state.increment_iteration();
        assert_eq!(state.iteration, 1);

        state.increment_iteration();
        assert_eq!(state.iteration, 2);
    }
}
