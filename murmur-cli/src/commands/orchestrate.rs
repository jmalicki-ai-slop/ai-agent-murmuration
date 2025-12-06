//! Orchestrate command - DAG-based multi-issue orchestration
//!
//! This command fetches an epic issue, parses its child issues,
//! builds a dependency graph, and executes issues in topological order.

use clap::Args;
use murmur_core::{
    AgentSpawner, BranchingOptions, Config, GitRepo, OutputStreamer, Secrets, WorktreeOptions,
};
use murmur_db::{
    models::{AgentRun, ConversationLog, WorktreeRecord},
    repos::{AgentRunRepository, ConversationRepository, WorktreeRepository},
    Database,
};
use murmur_github::{DependencyGraph, EpicChildren, GitHubClient, Issue, IssueState};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Orchestrate work on multiple issues from an epic
#[derive(Args, Debug)]
pub struct OrchestrateArgs {
    /// Epic issue number containing child issues
    pub epic: u64,

    /// Repository (owner/repo format, uses current repo if not specified)
    #[arg(short, long)]
    pub repo: Option<String>,

    /// Show execution plan without running agents
    #[arg(long)]
    pub dry_run: bool,

    /// Force recreation of worktrees even if they exist
    #[arg(short, long)]
    pub force: bool,

    /// Maximum number of parallel agents (default: 2)
    #[arg(long, default_value = "2")]
    pub parallelism: usize,

    /// Only run issues with this label (can be specified multiple times)
    #[arg(long)]
    pub label: Vec<String>,
}

/// Execution configuration for orchestration
struct ExecutionConfig<'a> {
    config: &'a Config,
    verbose: bool,
    no_emoji: bool,
}

/// Result of working on a single issue
#[derive(Debug)]
struct IssueResult {
    issue_number: u64,
    success: bool,
    error: Option<String>,
}

impl OrchestrateArgs {
    /// Execute the orchestrate command
    pub async fn execute(
        &self,
        verbose: bool,
        no_emoji: bool,
        config: &Config,
        repo: Option<&str>,
    ) -> anyhow::Result<()> {
        let repo_str = self.repo.as_deref().or(repo).ok_or_else(|| {
            anyhow::anyhow!(
                "No repository specified. Use --repo owner/repo or run from a git repository"
            )
        })?;

        let client = GitHubClient::from_url(repo_str).map_err(|e| anyhow::anyhow!("{}", e))?;

        println!(
            "Orchestrating epic #{} in {}/{}",
            self.epic,
            client.owner(),
            client.repo()
        );
        println!();

        // Fetch the epic issue
        let epic = client.get_issue_with_tracking(self.epic).await?;

        println!(
            "{} Epic: #{} - {}",
            emoji(no_emoji, "📋", "[EPIC]"),
            epic.number,
            epic.title
        );
        println!();

        // Parse child issues from epic body
        let epic_children = EpicChildren::from_body(&epic.body);

        if epic_children.children.is_empty() {
            println!(
                "{}  No child issues found in epic body.",
                emoji(no_emoji, "⚠️", "[WARN]")
            );
            println!("Expected format: `- [ ] #123` or `- [x] #123`");
            return Ok(());
        }

        println!(
            "Found {} child issues ({} pending, {} completed)",
            epic_children.children.len(),
            epic_children.pending.len(),
            epic_children.completed.len()
        );
        println!();

        // Fetch all child issues
        println!("Fetching child issues...");
        let mut issues: HashMap<u64, Issue> = HashMap::new();
        let mut fetch_errors = Vec::new();

        for (issue_num, _) in &epic_children.children {
            match client.get_issue_with_tracking(*issue_num).await {
                Ok(issue) => {
                    issues.insert(*issue_num, issue);
                }
                Err(e) => {
                    fetch_errors.push((*issue_num, e.to_string()));
                }
            }
        }

        if !fetch_errors.is_empty() {
            println!(
                "{}  Failed to fetch {} issues:",
                emoji(no_emoji, "⚠️", "[WARN]"),
                fetch_errors.len()
            );
            for (num, err) in &fetch_errors {
                println!("  #{}: {}", num, err);
            }
            println!();
        }

        // Filter by label if specified
        let issues_to_process: Vec<&Issue> = if !self.label.is_empty() {
            issues
                .values()
                .filter(|issue| self.label.iter().all(|label| issue.labels.contains(label)))
                .collect()
        } else {
            issues.values().collect()
        };

        if issues_to_process.is_empty() {
            println!(
                "{}  No issues match the specified criteria.",
                emoji(no_emoji, "⚠️", "[WARN]")
            );
            return Ok(());
        }

        // Build dependency graph
        println!("Building dependency graph...");
        let issue_vec: Vec<Issue> = issues_to_process.iter().map(|i| (*i).clone()).collect();
        let graph = match DependencyGraph::from_issues(&issue_vec) {
            Ok(g) => g,
            Err(e) => {
                println!(
                    "{}  Failed to build dependency graph: {}",
                    emoji(no_emoji, "❌", "[ERROR]"),
                    e
                );
                return Ok(());
            }
        };

        // Check for cycles
        let cycles = graph.find_cycles();
        if !cycles.is_empty() {
            println!(
                "{}  Circular dependencies detected:",
                emoji(no_emoji, "❌", "[ERROR]")
            );
            for cycle in &cycles {
                let cycle_str: Vec<String> = cycle.iter().map(|n| format!("#{}", n)).collect();
                println!("  {}", cycle_str.join(" -> "));
            }
            println!();
            println!("Please resolve circular dependencies before orchestrating.");
            return Ok(());
        }

        // Get topological order
        let topo_order = match graph.topological_order() {
            Some(order) => order,
            None => {
                println!(
                    "{}  Failed to determine execution order (cycle detected).",
                    emoji(no_emoji, "❌", "[ERROR]")
                );
                return Ok(());
            }
        };

        // Filter out already completed issues
        let pending_order: Vec<u64> = topo_order
            .into_iter()
            .filter(|n| epic_children.pending.contains(n))
            .collect();

        // Display execution plan
        println!();
        println!("Execution Plan:");
        println!("===============");
        println!();

        // Group issues by their dependency depth for display
        let mut depth_map: HashMap<u64, usize> = HashMap::new();
        for &issue_num in &pending_order {
            let deps = graph
                .dependencies
                .get(&issue_num)
                .cloned()
                .unwrap_or_default();
            let depth = if deps.is_empty() {
                0
            } else {
                deps.iter()
                    .filter_map(|d| depth_map.get(d))
                    .max()
                    .unwrap_or(&0)
                    + 1
            };
            depth_map.insert(issue_num, depth);
        }

        // Display by layer
        let max_depth = depth_map.values().max().copied().unwrap_or(0);
        for depth in 0..=max_depth {
            let layer: Vec<u64> = pending_order
                .iter()
                .filter(|n| depth_map.get(n) == Some(&depth))
                .copied()
                .collect();

            if layer.is_empty() {
                continue;
            }

            println!("Layer {} (parallel):", depth + 1);
            for issue_num in &layer {
                if let Some(issue) = issues.get(issue_num) {
                    let deps = graph
                        .dependencies
                        .get(issue_num)
                        .cloned()
                        .unwrap_or_default();
                    let deps_str = if deps.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " (depends on {})",
                            deps.iter()
                                .map(|d| format!("#{}", d))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    let state_icon = if issue.state == IssueState::Closed {
                        emoji(no_emoji, "✅", "[DONE]")
                    } else {
                        emoji(no_emoji, "⏳", "[PEND]")
                    };
                    println!(
                        "  {} #{}: {}{}",
                        state_icon, issue_num, issue.title, deps_str
                    );
                }
            }
            println!();
        }

        // Summary
        println!("Summary:");
        println!("  {} issues to process", pending_order.len());
        println!("  {} layers of execution", max_depth + 1);
        println!("  Max parallelism: {}", self.parallelism);
        println!();

        if self.dry_run {
            println!(
                "{} Dry run complete. Use without --dry-run to execute.",
                emoji(no_emoji, "ℹ️", "[INFO]")
            );
            return Ok(());
        }

        // Execute the orchestration
        println!("Starting orchestration...");
        println!();

        let exec_config = ExecutionConfig {
            config,
            verbose,
            no_emoji,
        };

        let execution_result = self
            .execute_orchestration(
                &client,
                &issues,
                &graph,
                &pending_order,
                &depth_map,
                &exec_config,
            )
            .await;

        match execution_result {
            Ok(results) => {
                self.print_summary(&results, no_emoji);
            }
            Err(e) => {
                println!(
                    "{}  Orchestration failed: {}",
                    emoji(no_emoji, "❌", "[ERROR]"),
                    e
                );
            }
        }

        Ok(())
    }

    /// Execute the orchestration, processing issues in dependency order
    async fn execute_orchestration(
        &self,
        _client: &GitHubClient,
        issues: &HashMap<u64, Issue>,
        graph: &DependencyGraph,
        pending_order: &[u64],
        depth_map: &HashMap<u64, usize>,
        exec_config: &ExecutionConfig<'_>,
    ) -> anyhow::Result<Vec<IssueResult>> {
        let mut results: Vec<IssueResult> = Vec::new();
        let completed: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
        let failed: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));

        // Process layer by layer
        let max_depth = depth_map.values().max().copied().unwrap_or(0);

        for depth in 0..=max_depth {
            let layer: Vec<u64> = pending_order
                .iter()
                .filter(|n| depth_map.get(n) == Some(&depth))
                .copied()
                .collect();

            if layer.is_empty() {
                continue;
            }

            println!(
                "{} Processing layer {} ({} issues)...",
                emoji(exec_config.no_emoji, "🔄", "[LAYER]"),
                depth + 1,
                layer.len()
            );
            println!();

            // Check which issues can proceed (all dependencies completed)
            let mut ready_issues = Vec::new();
            let mut skipped_issues = Vec::new();

            for issue_num in &layer {
                let deps = graph
                    .dependencies
                    .get(issue_num)
                    .cloned()
                    .unwrap_or_default();
                let failed_guard = failed.lock().await;

                // Check if any dependency failed
                let failed_deps: Vec<u64> = deps
                    .iter()
                    .filter(|d| failed_guard.contains(d))
                    .copied()
                    .collect();

                if !failed_deps.is_empty() {
                    skipped_issues.push((
                        *issue_num,
                        format!(
                            "dependency failed: {}",
                            failed_deps
                                .iter()
                                .map(|d| format!("#{}", d))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    ));
                } else {
                    ready_issues.push(*issue_num);
                }
            }

            // Record skipped issues
            for (issue_num, reason) in skipped_issues {
                println!(
                    "  {} #{}: Skipped ({})",
                    emoji(exec_config.no_emoji, "⏭️", "[SKIP]"),
                    issue_num,
                    reason
                );
                results.push(IssueResult {
                    issue_number: issue_num,
                    success: false,
                    error: Some(reason),
                });
            }

            // Process ready issues in parallel (up to parallelism limit)
            for chunk in ready_issues.chunks(self.parallelism) {
                let mut handles = Vec::new();

                for &issue_num in chunk {
                    if let Some(issue) = issues.get(&issue_num) {
                        let issue = issue.clone();
                        let config = exec_config.config.clone();
                        let verbose = exec_config.verbose;
                        let force = self.force;
                        let completed = Arc::clone(&completed);
                        let failed = Arc::clone(&failed);

                        println!(
                            "  {} Starting #{}: {}",
                            emoji(exec_config.no_emoji, "▶️", "[START]"),
                            issue_num,
                            issue.title
                        );

                        let handle = tokio::spawn(async move {
                            let result =
                                execute_single_issue(&issue, &config, verbose, force).await;

                            match &result {
                                Ok(_) => {
                                    completed.lock().await.insert(issue_num);
                                }
                                Err(_) => {
                                    failed.lock().await.insert(issue_num);
                                }
                            }

                            (issue_num, result)
                        });

                        handles.push(handle);
                    }
                }

                // Wait for this chunk to complete
                // We need to track which handle belongs to which issue for panic recovery
                let chunk_issues: Vec<u64> = chunk.to_vec();

                for (idx, handle) in handles.into_iter().enumerate() {
                    let issue_num = chunk_issues[idx];

                    match handle.await {
                        Ok((returned_issue_num, Ok(_))) => {
                            println!(
                                "  {} #{} completed successfully",
                                emoji(exec_config.no_emoji, "✅", "[OK]"),
                                returned_issue_num
                            );
                            results.push(IssueResult {
                                issue_number: returned_issue_num,
                                success: true,
                                error: None,
                            });
                        }
                        Ok((returned_issue_num, Err(e))) => {
                            println!(
                                "  {} #{} failed: {}",
                                emoji(exec_config.no_emoji, "❌", "[FAIL]"),
                                returned_issue_num,
                                e
                            );
                            results.push(IssueResult {
                                issue_number: returned_issue_num,
                                success: false,
                                error: Some(e.to_string()),
                            });
                        }
                        Err(e) => {
                            // Task panicked - we know which issue it was from the index
                            println!(
                                "  {} #{} panicked: {}",
                                emoji(exec_config.no_emoji, "💥", "[PANIC]"),
                                issue_num,
                                e
                            );
                            // Mark this issue as failed so dependent issues are skipped
                            failed.lock().await.insert(issue_num);
                            results.push(IssueResult {
                                issue_number: issue_num,
                                success: false,
                                error: Some(format!("Task panicked: {}", e)),
                            });
                        }
                    }
                }
            }

            println!();
        }

        Ok(results)
    }

    /// Print final summary of orchestration
    fn print_summary(&self, results: &[IssueResult], no_emoji: bool) {
        println!();
        println!("Orchestration Complete");
        println!("======================");
        println!();

        let succeeded: Vec<_> = results.iter().filter(|r| r.success).collect();
        let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

        println!(
            "{} {} issues completed successfully",
            emoji(no_emoji, "✅", "[OK]"),
            succeeded.len()
        );

        if !failed.is_empty() {
            println!(
                "{} {} issues failed or skipped:",
                emoji(no_emoji, "❌", "[FAIL]"),
                failed.len()
            );
            for result in &failed {
                let reason = result.error.as_deref().unwrap_or("unknown error");
                println!("  #{}: {}", result.issue_number, reason);
            }
        }
    }
}

/// Execute work on a single issue
async fn execute_single_issue(
    issue: &Issue,
    config: &Config,
    verbose: bool,
    force: bool,
) -> anyhow::Result<()> {
    // Skip already closed issues
    if issue.state == IssueState::Closed {
        return Ok(());
    }

    // Create worktree for the issue (non-async, database work is done before spawning)
    let cwd = std::env::current_dir()?;
    let git_repo = GitRepo::open(&cwd)?;

    let branching_options = BranchingOptions {
        base_branch: None,
        fetch: true,
        remote: None,
    };

    let point = git_repo.find_branching_point(&branching_options)?;
    let branch_name = format!("murmur/issue-{}", issue.number);

    let worktree_options = WorktreeOptions {
        branch_name: branch_name.clone(),
        force,
    };

    // Create worktree with proper error handling
    let info = git_repo
        .create_cached_worktree(&point, &worktree_options)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to create worktree for issue #{}: {}. {}",
                issue.number,
                e,
                if !force {
                    "Try using --force to recreate existing worktrees"
                } else {
                    "Check that the repository and filesystem are accessible"
                }
            )
        })?;

    // Build prompt from issue
    let prompt = build_prompt_from_issue(issue);

    // All database operations happen in a block that doesn't cross await points
    let (run_id, worktree_path) = {
        // Initialize database
        let db = Database::open().map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;

        // Track worktree in database
        let worktree_record =
            WorktreeRecord::new(info.path.to_string_lossy().to_string(), branch_name.clone())
                .with_issue_number(issue.number as i64)
                .with_main_repo_path(git_repo.root().to_string_lossy().to_string())
                .with_base_commit(&point.commit);

        let worktree_repo = WorktreeRepository::new(&db);
        worktree_repo
            .insert(&worktree_record)
            .map_err(|e| anyhow::anyhow!("Failed to track worktree in database: {}", e))?;

        // Create agent run record
        let config_json = serde_json::to_string(&config.agent).unwrap_or_else(|_| "{}".to_string());

        let agent_run = AgentRun::new(
            "implementer",
            &prompt,
            info.path.to_str().unwrap_or(""),
            config_json,
        )
        .with_issue_number(issue.number as i64);

        let agent_repo = AgentRunRepository::new(&db);
        let run_id = agent_repo
            .insert(&agent_run)
            .map_err(|e| anyhow::anyhow!("Failed to create agent run record: {}", e))?;

        // Update worktree record with agent run linkage
        if let Ok(Some(mut wt_record)) = worktree_repo.find_by_path(&info.path.to_string_lossy()) {
            wt_record.agent_run_id = Some(run_id);
            let _ = worktree_repo.update(&wt_record);
        }

        (run_id, info.path.to_string_lossy().to_string())
    };

    // Spawn agent (async operations happen here)
    let mut spawner = AgentSpawner::from_config(
        config.agent.clone(),
        murmur_core::agent::AgentType::default(),
    );

    // Pass GitHub token to agent
    if let Ok(secrets) = Secrets::load() {
        if let Some(token) = secrets.github_token() {
            spawner = spawner.with_env("GITHUB_TOKEN", token);
        }
    }

    let mut handle = spawner.spawn(&prompt, &info.path).await?;

    // Update PID in database (separate block)
    if let Some(pid) = handle.pid() {
        let db = Database::open().map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let agent_repo = AgentRunRepository::new(&db);
        if let Ok(mut agent_run) = agent_repo.find_by_id(run_id) {
            agent_run.pid = Some(pid as i32);
            let _ = agent_repo.update(&agent_run);
        }
    }

    // Stream output with logging
    let stdout = handle
        .child_mut()
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

    let mut streamer = OutputStreamer::new(stdout);

    let handler_db = Database::open()
        .map_err(|e| anyhow::anyhow!("Failed to open database for handler: {}", e))?;
    let mut handler = MinimalLoggingHandler::new(handler_db, run_id, verbose);

    if let Err(e) = streamer.stream(&mut handler).await {
        tracing::warn!("Stream error for #{}: {}", issue.number, e);
    }

    let status = handle.wait().await?;

    // Update completion status in database (separate block)
    {
        let db = Database::open().map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;

        // Update agent run
        let agent_repo = AgentRunRepository::new(&db);
        if let Ok(mut agent_run) = agent_repo.find_by_id(run_id) {
            agent_run.complete(status.code().unwrap_or(-1));
            let _ = agent_repo.update(&agent_run);
        }

        // Update worktree status
        let worktree_repo = WorktreeRepository::new(&db);
        if let Ok(Some(mut wt_record)) = worktree_repo.find_by_path(&worktree_path) {
            if status.success() {
                wt_record.mark_completed();
            } else {
                wt_record.mark_abandoned();
            }
            let _ = worktree_repo.update(&wt_record);
        }
    }

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Agent exited with code {}",
            status.code().unwrap_or(-1)
        ))
    }
}

fn build_prompt_from_issue(issue: &Issue) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!(
        "Work on GitHub issue #{}: {}\n\n",
        issue.number, issue.title
    ));

    // Extract description (before metadata block)
    let body = issue
        .body
        .lines()
        .take_while(|line| !line.starts_with("<!-- murmur:metadata"))
        .collect::<Vec<_>>()
        .join("\n");

    if !body.is_empty() {
        prompt.push_str("Issue description:\n");
        prompt.push_str(&body);
        prompt.push_str("\n\n");
    }

    prompt.push_str("Please implement this issue. When done, provide a summary of changes made.");

    prompt
}

/// Minimal logging handler for orchestrated agents
struct MinimalLoggingHandler {
    db: Database,
    run_id: i64,
    sequence: i64,
    verbose: bool,
}

impl MinimalLoggingHandler {
    fn new(db: Database, run_id: i64, verbose: bool) -> Self {
        Self {
            db,
            run_id,
            sequence: 0,
            verbose,
        }
    }

    fn log_message(&mut self, message_type: &str, message_json: &str) {
        let log = ConversationLog::new(self.run_id, self.sequence, message_type, message_json);

        let repo = ConversationRepository::new(&self.db);
        if let Err(e) = repo.insert(&log) {
            if self.verbose {
                tracing::warn!("Failed to log message to database: {}", e);
            }
        }

        self.sequence += 1;
    }
}

impl murmur_core::StreamHandler for MinimalLoggingHandler {
    fn on_system(&mut self, subtype: Option<&str>, session_id: Option<&str>) {
        let msg = serde_json::json!({
            "type": "system",
            "subtype": subtype,
            "session_id": session_id,
        });
        self.log_message("system", &msg.to_string());
    }

    fn on_user(&mut self, message: &serde_json::Value) {
        let msg = serde_json::json!({
            "type": "user",
            "message": message,
        });
        self.log_message("user", &msg.to_string());
    }

    fn on_assistant_text(&mut self, text: &str) {
        let msg = serde_json::json!({
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": text}]
            }
        });
        self.log_message("assistant", &msg.to_string());

        // Don't print to stdout for orchestrated agents - too noisy
        if self.verbose {
            print!("{}", text);
        }
    }

    fn on_tool_use(&mut self, tool: &str, input: &serde_json::Value) {
        let msg = serde_json::json!({
            "type": "tool_use",
            "tool": tool,
            "input": input,
        });
        self.log_message("tool_use", &msg.to_string());
    }

    fn on_tool_result(&mut self, output: &str, is_error: bool) {
        let msg = serde_json::json!({
            "type": "tool_result",
            "output": output,
            "is_error": is_error,
        });
        self.log_message("tool_result", &msg.to_string());
    }

    fn on_complete(&mut self, cost: Option<&murmur_core::CostInfo>, duration_ms: Option<u64>) {
        let msg = serde_json::json!({
            "type": "result",
            "cost": cost,
            "duration_ms": duration_ms,
        });
        self.log_message("result", &msg.to_string());
    }

    fn on_parse_error(&mut self, line: &str, error: &serde_json::Error) {
        if self.verbose {
            tracing::warn!("Parse error on line '{}': {}", line, error);
        }
    }
}

/// Get emoji or ASCII alternative based on no_emoji flag
fn emoji<'a>(no_emoji: bool, emoji_char: &'a str, ascii_alt: &'a str) -> &'a str {
    if no_emoji {
        ascii_alt
    } else {
        emoji_char
    }
}
