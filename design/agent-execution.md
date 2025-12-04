# Agent Execution Design

## Overview

This document defines how agents are spawned, monitored, and managed. Agents are Claude Code instances running in isolated git worktrees.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     AgentLifecycleManager                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │  Executor   │  │ Heartbeat   │  │     Output Collector    │ │
│  │             │  │  Monitor    │  │                         │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
└─────────┼────────────────┼─────────────────────┼───────────────┘
          │                │                     │
          ▼                ▼                     ▼
    ┌──────────┐    ┌──────────┐          ┌──────────┐
    │ Process  │    │ Database │          │  Events  │
    │ Spawn    │    │ Updates  │          │ Broadcast│
    └──────────┘    └──────────┘          └──────────┘
          │
          ▼
    ┌──────────────────────────────────────────────┐
    │              Git Worktree                     │
    │  ┌─────────────────────────────────────────┐ │
    │  │           Claude Code Process           │ │
    │  │  - System prompt (agent type)           │ │
    │  │  - Issue context                        │ │
    │  │  - Tool access                          │ │
    │  └─────────────────────────────────────────┘ │
    └──────────────────────────────────────────────┘
```

---

## Agent Types

### Type Definitions

```rust
// dispatch-agents/src/types.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Coder,      // Implements features, fixes bugs
    Reviewer,   // Reviews code, suggests improvements
    Pm,         // Decomposes epics, manages scope
    Security,   // Security audits, vulnerability checks
    Docs,       // Documentation writing
    Test,       // Test writing, coverage analysis
    Architect,  // System design, architecture decisions
}

impl AgentType {
    pub fn capabilities(&self) -> AgentCapabilities {
        match self {
            Self::Coder => AgentCapabilities {
                can_write_code: true,
                can_create_pr: true,
                can_run_tests: true,
                can_propose: true,
                can_vote: true,
                vote_weight: 1.0,
            },
            Self::Reviewer => AgentCapabilities {
                can_write_code: false,  // Can suggest, not write
                can_create_pr: false,
                can_run_tests: true,
                can_propose: true,
                can_vote: true,
                vote_weight: 1.5,  // Higher weight on code decisions
            },
            Self::Pm => AgentCapabilities {
                can_write_code: false,
                can_create_pr: false,
                can_run_tests: false,
                can_propose: true,
                can_vote: true,
                vote_weight: 1.0,
            },
            Self::Security => AgentCapabilities {
                can_write_code: true,  // Security fixes
                can_create_pr: true,
                can_run_tests: true,
                can_propose: true,
                can_vote: true,
                vote_weight: 2.0,  // Veto power on security issues
            },
            Self::Docs => AgentCapabilities {
                can_write_code: false,
                can_create_pr: true,  // Doc PRs
                can_run_tests: false,
                can_propose: true,
                can_vote: true,
                vote_weight: 0.5,
            },
            Self::Test => AgentCapabilities {
                can_write_code: true,  // Test code
                can_create_pr: true,
                can_run_tests: true,
                can_propose: true,
                can_vote: true,
                vote_weight: 1.0,
            },
            Self::Architect => AgentCapabilities {
                can_write_code: false,
                can_create_pr: false,
                can_run_tests: false,
                can_propose: true,
                can_vote: true,
                vote_weight: 1.5,  // Higher weight on arch decisions
            },
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Coder | Self::Architect | Self::Security => "claude-sonnet-4-20250514",
            Self::Reviewer | Self::Pm => "claude-sonnet-4-20250514",
            Self::Docs | Self::Test => "claude-sonnet-4-20250514",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentCapabilities {
    pub can_write_code: bool,
    pub can_create_pr: bool,
    pub can_run_tests: bool,
    pub can_propose: bool,
    pub can_vote: bool,
    pub vote_weight: f64,
}
```

### System Prompts

```
prompts/
├── base.md           # Common instructions for all agents
├── coder.md          # Implementation focus
├── reviewer.md       # Code review focus
├── pm.md             # Project management focus
├── security.md       # Security audit focus
├── docs.md           # Documentation focus
├── test.md           # Testing focus
└── architect.md      # Architecture focus
```

Example system prompt structure:

```markdown
<!-- prompts/coder.md -->

# Coder Agent

You are a skilled software developer working on the Dispatch system. Your role is to implement features and fix bugs according to the issue specifications.

## Core Responsibilities

1. **Understand the Issue**: Read the issue description carefully. Ask for clarification if needed.
2. **Implement the Solution**: Write clean, well-tested code that solves the problem.
3. **Follow Project Standards**: Match the existing code style and patterns.
4. **Create Pull Request**: When done, create a PR with a clear description.

## Workflow

1. Read the issue context provided
2. Explore relevant parts of the codebase
3. Plan your implementation approach
4. Write the code
5. Write tests
6. Run existing tests to ensure no regressions
7. Create a PR when ready

## Communication

- If you encounter blocking issues, update the issue status to "blocked"
- If you need clarification, add a comment to the issue
- If you want to propose a different approach, create a proposal

## Constraints

- Stay focused on the assigned issue
- Do not modify unrelated code
- Do not merge your own PRs
- Request review when PR is ready

## Available Tools

You have access to all standard Claude Code tools:
- File read/write
- Terminal commands
- Git operations
- Web search (for documentation)

## Current Issue Context

The issue details will be provided below. Work only on this specific issue.
```

---

## Agent Spawning

### Executor Implementation

```rust
// dispatch-agents/src/executor.rs

use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct AgentExecutor {
    prompts_dir: PathBuf,
    config: AgentConfig,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub claude_path: PathBuf,       // Path to claude binary
    pub model: Option<String>,       // Override model
    pub max_tokens: Option<u32>,     // Token limit
    pub timeout_secs: Option<u64>,   // Max execution time
    pub allowed_tools: Vec<String>,  // Tool whitelist
    pub denied_tools: Vec<String>,   // Tool blacklist
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            claude_path: PathBuf::from("claude"),
            model: None,
            max_tokens: None,
            timeout_secs: Some(3600),  // 1 hour default
            allowed_tools: vec![],
            denied_tools: vec![],
        }
    }
}

impl AgentExecutor {
    pub fn new(prompts_dir: PathBuf, config: AgentConfig) -> Self {
        Self { prompts_dir, config }
    }

    /// Build the full context for an agent
    fn build_context(&self, agent_type: AgentType, issue: &Issue, epic: Option<&Epic>) -> Result<String> {
        let mut context = String::new();

        // Issue details
        context.push_str(&format!("# Issue: {}\n\n", issue.title));
        context.push_str(&format!("**ID:** {}\n", issue.id));
        context.push_str(&format!("**Type:** {}\n", issue.issue_type.as_str()));
        context.push_str(&format!("**Priority:** {}\n", issue.priority.as_str()));

        if !issue.labels.is_empty() {
            context.push_str(&format!("**Labels:** {}\n", issue.labels.join(", ")));
        }

        context.push_str("\n## Description\n\n");
        context.push_str(&issue.prompt);
        context.push_str("\n\n");

        // Epic context if available
        if let Some(epic) = epic {
            context.push_str("## Epic Context\n\n");
            context.push_str(&format!("**Epic:** {}\n", epic.title));
            context.push_str(&format!("**Stage:** {}\n", epic.current_stage_id.as_ref().map(|_| "Current").unwrap_or("N/A")));
            context.push_str(&epic.description);
            context.push_str("\n\n");
        }

        // Repository context
        context.push_str("## Repository\n\n");
        context.push_str(&format!("**Path:** {}\n", issue.repo_path.display()));
        if let Some(ref url) = issue.repo_url {
            context.push_str(&format!("**URL:** {}\n", url));
        }
        if let Some(ref branch) = issue.branch_name {
            context.push_str(&format!("**Branch:** {}\n", branch));
        }

        Ok(context)
    }

    /// Load system prompt for agent type
    fn load_system_prompt(&self, agent_type: AgentType) -> Result<String> {
        // Load base prompt
        let base_path = self.prompts_dir.join("base.md");
        let base = std::fs::read_to_string(&base_path)
            .unwrap_or_default();

        // Load type-specific prompt
        let type_path = self.prompts_dir.join(format!("{}.md", agent_type.as_str()));
        let type_specific = std::fs::read_to_string(&type_path)
            .map_err(|e| DispatchError::Config(format!(
                "Could not load prompt for {}: {}", agent_type.as_str(), e
            )))?;

        Ok(format!("{}\n\n{}", base, type_specific))
    }

    /// Spawn a Claude Code process
    pub async fn spawn(
        &self,
        agent: &Agent,
        issue: &Issue,
        epic: Option<&Epic>,
        worktree_path: &Path,
    ) -> Result<AgentProcess> {
        let system_prompt = self.load_system_prompt(agent.agent_type)?;
        let context = self.build_context(agent.agent_type, issue, epic)?;

        // Write system prompt to temp file (claude requires file path)
        let prompt_file = worktree_path.join(".dispatch-prompt.md");
        std::fs::write(&prompt_file, &system_prompt)?;

        let mut cmd = Command::new(&self.config.claude_path);

        // Core arguments
        cmd.arg("--print")
            .arg("--output-format").arg("stream-json")
            .arg("--system-prompt").arg(&prompt_file);

        // Model override
        if let Some(ref model) = self.config.model {
            cmd.arg("--model").arg(model);
        } else {
            cmd.arg("--model").arg(agent.agent_type.default_model());
        }

        // Token limit
        if let Some(max_tokens) = self.config.max_tokens {
            cmd.arg("--max-tokens").arg(max_tokens.to_string());
        }

        // Tool configuration
        for tool in &self.config.allowed_tools {
            cmd.arg("--allowedTools").arg(tool);
        }
        for tool in &self.config.denied_tools {
            cmd.arg("--disallowedTools").arg(tool);
        }

        // Working directory
        cmd.current_dir(worktree_path);

        // Capture I/O
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // The initial prompt
        cmd.arg(&context);

        // Spawn process
        let child = cmd.spawn().map_err(|e| {
            DispatchError::Agent(format!("Failed to spawn claude: {}", e))
        })?;

        let pid = child.id().unwrap_or(0);

        Ok(AgentProcess {
            child,
            pid,
            agent_id: agent.id.clone(),
            issue_id: issue.id.clone(),
            started_at: Utc::now(),
        })
    }

    /// Spawn with resume capability
    pub async fn spawn_with_resume(
        &self,
        agent: &Agent,
        issue: &Issue,
        epic: Option<&Epic>,
        worktree_path: &Path,
        session_id: &str,
    ) -> Result<AgentProcess> {
        // Similar to spawn but adds --resume flag
        let mut cmd = self.build_base_command(agent, issue, epic, worktree_path)?;
        cmd.arg("--resume").arg(session_id);

        let child = cmd.spawn()?;
        let pid = child.id().unwrap_or(0);

        Ok(AgentProcess {
            child,
            pid,
            agent_id: agent.id.clone(),
            issue_id: issue.id.clone(),
            started_at: Utc::now(),
        })
    }
}

pub struct AgentProcess {
    child: Child,
    pid: u32,
    agent_id: AgentId,
    issue_id: IssueId,
    started_at: DateTime<Utc>,
}

impl AgentProcess {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    pub fn issue_id(&self) -> &IssueId {
        &self.issue_id
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        Ok(self.child.wait().await?)
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }

    /// Send SIGSTOP (pause)
    #[cfg(unix)]
    pub fn pause(&self) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(self.pid as i32), Signal::SIGSTOP)?;
        Ok(())
    }

    /// Send SIGCONT (resume)
    #[cfg(unix)]
    pub fn resume(&self) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        kill(Pid::from_raw(self.pid as i32), Signal::SIGCONT)?;
        Ok(())
    }

    /// Take stdout for streaming
    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }

    /// Take stderr for streaming
    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.child.stderr.take()
    }

    /// Take stdin for input
    pub fn take_stdin(&mut self) -> Option<tokio::process::ChildStdin> {
        self.child.stdin.take()
    }
}
```

---

## Output Processing

### Stream-JSON Parser

```rust
// dispatch-agents/src/output.rs

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Claude Code stream-json output format
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeOutput {
    #[serde(rename = "assistant")]
    Assistant {
        message: AssistantMessage,
    },
    #[serde(rename = "user")]
    User {
        message: UserMessage,
    },
    #[serde(rename = "result")]
    Result {
        result: ResultMessage,
    },
    #[serde(rename = "system")]
    System {
        system: SystemMessage,
    },
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
pub struct ResultMessage {
    pub cost_usd: f64,
    pub duration_ms: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub is_error: bool,
    pub error_message: Option<String>,
}

pub struct OutputCollector {
    agent_id: AgentId,
    issue_id: IssueId,
    log_repo: AgentLogRepository,
    events: broadcast::Sender<DispatchEvent>,
}

impl OutputCollector {
    /// Process stdout stream from agent
    pub async fn collect_stdout(
        &self,
        stdout: tokio::process::ChildStdout,
    ) -> Result<AgentResult> {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut total_cost = 0.0;
        let mut total_tokens_in = 0;
        let mut total_tokens_out = 0;
        let mut error: Option<String> = None;

        while let Some(line) = lines.next_line().await? {
            match serde_json::from_str::<ClaudeOutput>(&line) {
                Ok(output) => {
                    match output {
                        ClaudeOutput::Assistant { message } => {
                            self.process_assistant_message(&message).await?;
                        }
                        ClaudeOutput::Result { result } => {
                            total_cost += result.cost_usd;
                            total_tokens_in += result.tokens_in;
                            total_tokens_out += result.tokens_out;

                            if result.is_error {
                                error = result.error_message;
                            }
                        }
                        ClaudeOutput::System { system } => {
                            self.log_system_message(&system).await?;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    // Log parsing error but continue
                    tracing::warn!("Failed to parse output line: {}", e);
                }
            }
        }

        Ok(AgentResult {
            agent_id: self.agent_id.clone(),
            issue_id: self.issue_id.clone(),
            cost_usd: total_cost,
            tokens_in: total_tokens_in,
            tokens_out: total_tokens_out,
            error,
        })
    }

    async fn process_assistant_message(&self, message: &AssistantMessage) -> Result<()> {
        for block in &message.content {
            match block {
                ContentBlock::Text { text } => {
                    // Log text output
                    self.log_repo.create(&AgentLog {
                        id: AgentLogId::new(),
                        agent_id: self.agent_id.clone(),
                        issue_id: Some(self.issue_id.clone()),
                        level: LogLevel::Info,
                        message: text.clone(),
                        context: None,
                        created_at: Utc::now(),
                    }).await?;

                    // Check for special patterns
                    self.detect_patterns(text).await?;
                }
                ContentBlock::ToolUse { name, input, .. } => {
                    // Track tool usage
                    self.log_repo.create(&AgentLog {
                        id: AgentLogId::new(),
                        agent_id: self.agent_id.clone(),
                        issue_id: Some(self.issue_id.clone()),
                        level: LogLevel::Debug,
                        message: format!("Tool use: {}", name),
                        context: Some(serde_json::to_string(input)?),
                        created_at: Utc::now(),
                    }).await?;

                    // Check for completion signals
                    if name == "dispatch_complete" || name == "dispatch_blocked" {
                        self.handle_completion_signal(name, input).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect patterns in agent output
    async fn detect_patterns(&self, text: &str) -> Result<()> {
        // PR creation
        if text.contains("Created pull request") || text.contains("PR #") {
            if let Some(pr_num) = extract_pr_number(text) {
                self.events.send(DispatchEvent::PrCreated {
                    issue_id: self.issue_id.clone(),
                    pr_number: pr_num,
                })?;
            }
        }

        // Blocked indicator
        if text.contains("BLOCKED:") || text.contains("blocked by") {
            self.events.send(DispatchEvent::IssueStatusChanged {
                issue_id: self.issue_id.clone(),
                from: IssueStatus::InProgress,
                to: IssueStatus::Blocked,
            })?;
        }

        // Completion indicator
        if text.contains("DONE:") || text.contains("completed") {
            self.events.send(DispatchEvent::IssueStatusChanged {
                issue_id: self.issue_id.clone(),
                from: IssueStatus::InProgress,
                to: IssueStatus::AwaitingReview,
            })?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AgentResult {
    pub agent_id: AgentId,
    pub issue_id: IssueId,
    pub cost_usd: f64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub error: Option<String>,
}

fn extract_pr_number(text: &str) -> Option<u64> {
    let re = regex::Regex::new(r"#(\d+)").ok()?;
    re.captures(text)?.get(1)?.as_str().parse().ok()
}
```

---

## Heartbeat Monitoring

```rust
// dispatch-agents/src/heartbeat.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub struct HeartbeatMonitor {
    agent_repo: AgentRepository,
    events: broadcast::Sender<DispatchEvent>,
    processes: Arc<RwLock<HashMap<AgentId, AgentProcess>>>,
    config: HeartbeatConfig,
}

#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    pub check_interval_secs: u64,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            timeout_secs: 120,
            max_retries: 3,
        }
    }
}

impl HeartbeatMonitor {
    /// Start the heartbeat monitoring loop
    pub async fn start(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.check_interval_secs));

        loop {
            interval.tick().await;
            self.check_all_agents().await?;
        }
    }

    async fn check_all_agents(&self) -> Result<()> {
        let agents = self.agent_repo.list_active().await?;
        let now = Utc::now();

        for agent in agents {
            let elapsed = now - agent.last_heartbeat;

            if elapsed.num_seconds() > self.config.timeout_secs as i64 {
                // Agent appears unresponsive
                self.handle_unresponsive_agent(&agent).await?;
            } else {
                // Update heartbeat
                self.send_heartbeat(&agent).await?;
            }
        }

        Ok(())
    }

    async fn send_heartbeat(&self, agent: &Agent) -> Result<()> {
        // Check if process is still alive
        let processes = self.processes.read().await;

        if let Some(process) = processes.get(&agent.id) {
            // Process exists, check if it's responding
            // For now, just update the timestamp if process is running
            if self.is_process_alive(process.pid()).await {
                let mut agent = agent.clone();
                agent.last_heartbeat = Utc::now();
                self.agent_repo.update(&agent).await?;

                self.events.send(DispatchEvent::AgentHeartbeat {
                    agent_id: agent.id.clone(),
                })?;
            }
        }

        Ok(())
    }

    async fn is_process_alive(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            // Signal 0 just checks if process exists
            kill(Pid::from_raw(pid as i32), Signal::SIGCONT).is_ok()
        }

        #[cfg(windows)]
        {
            // Windows process check
            true // TODO: Implement
        }
    }

    async fn handle_unresponsive_agent(&self, agent: &Agent) -> Result<()> {
        tracing::warn!("Agent {} is unresponsive", agent.id);

        // Update status
        let mut agent = agent.clone();
        agent.status = AgentStatus::Errored;
        self.agent_repo.update(&agent).await?;

        // Emit event
        self.events.send(DispatchEvent::AgentErrored {
            agent_id: agent.id.clone(),
            error: "Agent unresponsive".to_string(),
        })?;

        // Kill the process
        let mut processes = self.processes.write().await;
        if let Some(mut process) = processes.remove(&agent.id) {
            let _ = process.kill().await;
        }

        // Update issue status
        if let Some(issue_id) = agent.current_issue_id {
            // Reset issue to unassigned
            self.events.send(DispatchEvent::IssueStatusChanged {
                issue_id,
                from: IssueStatus::InProgress,
                to: IssueStatus::Blocked,
            })?;
        }

        Ok(())
    }
}
```

---

## Lifecycle Manager

```rust
// dispatch-agents/src/lifecycle.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AgentLifecycleManager {
    executor: AgentExecutor,
    output_collector: OutputCollector,
    heartbeat_monitor: HeartbeatMonitor,

    agent_repo: AgentRepository,
    issue_repo: IssueRepository,
    epic_repo: EpicRepository,
    worktree_manager: WorktreeManager,

    running_agents: Arc<RwLock<HashMap<AgentId, AgentProcess>>>,
    events: broadcast::Sender<DispatchEvent>,
}

impl AgentLifecycleManager {
    /// Assign and start an agent for an issue
    pub async fn assign_and_start(
        &self,
        issue_id: &IssueId,
        agent_type: AgentType,
    ) -> Result<AgentId> {
        // Get issue
        let mut issue = self.issue_repo.get(issue_id).await?.ok_or(
            DispatchError::NotFound { entity: "Issue", id: issue_id.to_string() }
        )?;

        // Get epic if associated
        let epic = if let Some(ref epic_id) = issue.epic_id {
            self.epic_repo.get(epic_id).await?
        } else {
            None
        };

        // Create worktree if needed
        let worktree_path = if let Some(ref path) = issue.worktree_path {
            path.clone()
        } else {
            let info = self.worktree_manager.create_worktree(
                issue_id,
                &issue.title,
                None,
            )?;
            issue.worktree_path = Some(info.path.clone());
            issue.branch_name = Some(info.branch);
            info.path
        };

        // Create agent
        let mut agent = Agent::new(agent_type);
        agent.current_issue_id = Some(issue_id.clone());
        agent.worktree_path = Some(worktree_path.clone());
        self.agent_repo.create(&agent).await?;

        // Update issue
        issue.assigned_agent_id = Some(agent.id.clone());
        issue.agent_type = Some(agent_type);
        issue.status = IssueStatus::Assigned;
        issue.assigned_at = Some(Utc::now());
        self.issue_repo.update(&issue).await?;

        // Spawn process
        let process = self.executor.spawn(
            &agent,
            &issue,
            epic.as_ref(),
            &worktree_path,
        ).await?;

        // Update agent with PID
        agent.process_id = Some(process.pid());
        agent.status = AgentStatus::Working;
        self.agent_repo.update(&agent).await?;

        // Track process
        self.running_agents.write().await.insert(agent.id.clone(), process);

        // Emit events
        self.events.send(DispatchEvent::AgentStarted { agent_id: agent.id.clone() })?;
        self.events.send(DispatchEvent::IssueAssigned {
            issue_id: issue_id.clone(),
            agent_id: agent.id.clone(),
        })?;

        // Start output collection in background
        self.spawn_output_collector(agent.id.clone()).await;

        Ok(agent.id)
    }

    /// Stop an agent
    pub async fn stop_agent(&self, agent_id: &AgentId, force: bool) -> Result<()> {
        let mut processes = self.running_agents.write().await;

        if let Some(mut process) = processes.remove(agent_id) {
            if force {
                process.kill().await?;
            } else {
                // Graceful shutdown - send SIGTERM first
                #[cfg(unix)]
                {
                    use nix::sys::signal::{kill, Signal};
                    use nix::unistd::Pid;
                    let _ = kill(Pid::from_raw(process.pid() as i32), Signal::SIGTERM);

                    // Wait up to 30 seconds
                    tokio::select! {
                        _ = process.wait() => {}
                        _ = tokio::time::sleep(Duration::from_secs(30)) => {
                            // Force kill after timeout
                            process.kill().await?;
                        }
                    }
                }

                #[cfg(windows)]
                {
                    process.kill().await?;
                }
            }
        }

        // Update agent status
        if let Some(mut agent) = self.agent_repo.get(agent_id).await? {
            agent.status = AgentStatus::Completed;
            agent.completed_at = Some(Utc::now());
            agent.process_id = None;
            self.agent_repo.update(&agent).await?;
        }

        self.events.send(DispatchEvent::AgentStatusChanged {
            agent_id: agent_id.clone(),
            from: AgentStatus::Working,
            to: AgentStatus::Completed,
        })?;

        Ok(())
    }

    /// Pause an agent
    pub async fn pause_agent(&self, agent_id: &AgentId) -> Result<()> {
        let processes = self.running_agents.read().await;

        if let Some(process) = processes.get(agent_id) {
            process.pause()?;

            // Update status
            if let Some(mut agent) = self.agent_repo.get(agent_id).await? {
                agent.status = AgentStatus::Paused;
                self.agent_repo.update(&agent).await?;
            }

            self.events.send(DispatchEvent::AgentStatusChanged {
                agent_id: agent_id.clone(),
                from: AgentStatus::Working,
                to: AgentStatus::Paused,
            })?;
        }

        Ok(())
    }

    /// Resume a paused agent
    pub async fn resume_agent(&self, agent_id: &AgentId) -> Result<()> {
        let processes = self.running_agents.read().await;

        if let Some(process) = processes.get(agent_id) {
            process.resume()?;

            // Update status
            if let Some(mut agent) = self.agent_repo.get(agent_id).await? {
                agent.status = AgentStatus::Working;
                self.agent_repo.update(&agent).await?;
            }

            self.events.send(DispatchEvent::AgentStatusChanged {
                agent_id: agent_id.clone(),
                from: AgentStatus::Paused,
                to: AgentStatus::Working,
            })?;
        }

        Ok(())
    }

    /// Get status of all running agents
    pub async fn get_running_status(&self) -> Vec<AgentRunningStatus> {
        let processes = self.running_agents.read().await;
        let mut statuses = Vec::new();

        for (agent_id, process) in processes.iter() {
            if let Ok(Some(agent)) = self.agent_repo.get(agent_id).await {
                statuses.push(AgentRunningStatus {
                    agent_id: agent_id.clone(),
                    agent_type: agent.agent_type,
                    status: agent.status,
                    pid: process.pid(),
                    issue_id: agent.current_issue_id.clone(),
                    running_since: process.started_at,
                });
            }
        }

        statuses
    }

    async fn spawn_output_collector(&self, agent_id: AgentId) {
        let mut processes = self.running_agents.write().await;

        if let Some(process) = processes.get_mut(&agent_id) {
            if let Some(stdout) = process.take_stdout() {
                let collector = self.output_collector.clone();
                let agent_id = agent_id.clone();

                tokio::spawn(async move {
                    match collector.collect_stdout(stdout).await {
                        Ok(result) => {
                            tracing::info!("Agent {} completed: ${:.4}", agent_id, result.cost_usd);
                        }
                        Err(e) => {
                            tracing::error!("Agent {} output error: {}", agent_id, e);
                        }
                    }
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct AgentRunningStatus {
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub pid: u32,
    pub issue_id: Option<IssueId>,
    pub running_since: DateTime<Utc>,
}
```

---

## Agent Pool Management

```rust
// dispatch-agents/src/pool.rs

/// Manages a pool of agents for concurrent work
pub struct AgentPool {
    lifecycle: AgentLifecycleManager,
    max_concurrent: usize,
    queue: VecDeque<QueuedAssignment>,
}

#[derive(Debug)]
struct QueuedAssignment {
    issue_id: IssueId,
    agent_type: AgentType,
    queued_at: DateTime<Utc>,
}

impl AgentPool {
    pub fn new(lifecycle: AgentLifecycleManager, max_concurrent: usize) -> Self {
        Self {
            lifecycle,
            max_concurrent,
            queue: VecDeque::new(),
        }
    }

    /// Queue an assignment, starting immediately if capacity available
    pub async fn queue_assignment(
        &mut self,
        issue_id: IssueId,
        agent_type: AgentType,
    ) -> Result<()> {
        let running = self.lifecycle.get_running_status().await.len();

        if running < self.max_concurrent {
            // Start immediately
            self.lifecycle.assign_and_start(&issue_id, agent_type).await?;
        } else {
            // Add to queue
            self.queue.push_back(QueuedAssignment {
                issue_id,
                agent_type,
                queued_at: Utc::now(),
            });
        }

        Ok(())
    }

    /// Process queue when an agent completes
    pub async fn on_agent_completed(&mut self) -> Result<()> {
        if let Some(next) = self.queue.pop_front() {
            self.lifecycle.assign_and_start(&next.issue_id, next.agent_type).await?;
        }
        Ok(())
    }

    /// Get queue status
    pub fn queue_status(&self) -> Vec<&QueuedAssignment> {
        self.queue.iter().collect()
    }
}
```

---

## Implementation PRs

| PR | Description | Files |
|----|-------------|-------|
| PR-027 | Agent data model | `dispatch-core/src/types/agent.rs` |
| PR-028 | Claude Code subprocess spawning | `dispatch-agents/src/executor.rs` |
| PR-029 | Agent lifecycle management | `dispatch-agents/src/lifecycle.rs` |
| PR-030 | Heartbeat monitoring | `dispatch-agents/src/heartbeat.rs` |
| PR-031 | Agent type definitions | `dispatch-agents/src/types.rs`, `prompts/*.md` |
| PR-032 | Issue → Agent context | `dispatch-agents/src/context.rs` |
| PR-033 | Agent output collection | `dispatch-agents/src/output.rs` |
| PR-034 | Agent failure handling | `dispatch-agents/src/lifecycle.rs` |
| PR-035 | CLI agent commands | `dispatch-cli/src/commands/agent.rs` |
