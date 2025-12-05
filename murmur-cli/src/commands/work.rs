//! Work command - start working on an issue with dependency checking

use clap::Args;
use murmur_core::{
    AgentSpawner, BranchingOptions, Config, GitRepo, OutputStreamer, Secrets, WorktreeMetadata,
    WorktreeOptions,
};
use murmur_db::{
    models::{AgentRun, ConversationLog, WorktreeRecord},
    repos::{AgentRunRepository, ConversationRepository, WorktreeRepository},
    Database,
};
use murmur_github::{DependencyStatus, GitHubClient, IssueDependencies, IssueState};

/// Work on a GitHub issue
#[derive(Args, Debug)]
pub struct WorkArgs {
    /// Issue number to work on
    pub issue: u64,

    /// Repository (owner/repo format, uses current repo if not specified)
    #[arg(short, long)]
    pub repo: Option<String>,

    /// Skip dependency checking
    #[arg(short, long)]
    pub force: bool,

    /// Custom prompt to send to the agent (uses issue description if not provided)
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Don't start the agent, just create the worktree
    #[arg(long)]
    pub no_agent: bool,

    /// Resume from the last interrupted or failed run for this issue
    #[arg(long)]
    pub resume: bool,
}

impl WorkArgs {
    /// Execute the work command
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
            "Working on issue #{} in {}/{}",
            self.issue,
            client.owner(),
            client.repo()
        );
        println!();

        // Initialize database
        let db = Database::open().map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;

        // Fetch the issue with tracking information
        let issue = client.get_issue_with_tracking(self.issue).await?;

        println!("#{}: {}", issue.number, issue.title);
        println!();

        // Check for resumable runs if --resume flag is set
        if self.resume {
            use murmur_core::{
                build_resume_prompt, find_latest_incomplete_run, reconstruct_conversation,
            };

            println!("Checking for incomplete runs to resume...");
            match find_latest_incomplete_run(&db, self.issue as i64)
                .map_err(|e| anyhow::anyhow!("{}", e))?
            {
                Some(resumable) => {
                    println!("Found incomplete run from {}", resumable.start_time);
                    println!("  Run ID: {}", resumable.run_id);
                    println!("  Messages: {}", resumable.message_count);
                    if let Some(exit_code) = resumable.exit_code {
                        println!("  Exit code: {}", exit_code);
                    } else {
                        println!("  Status: Interrupted (no exit code)");
                    }
                    println!();

                    if resumable.message_count == 0 {
                        println!(
                            "{}  No conversation history found. Starting fresh instead.",
                            emoji(no_emoji, "⚠️", "[WARN]")
                        );
                        println!();
                    } else {
                        // Reconstruct conversation
                        let messages = reconstruct_conversation(&db, resumable.run_id)
                            .map_err(|e| anyhow::anyhow!("{}", e))?;

                        println!(
                            "Reconstructed {} messages from previous session",
                            messages.len()
                        );

                        // Build resume prompt
                        let original_prompt = build_prompt_from_issue(&issue);
                        let reason = if resumable.had_error() {
                            format!(
                                "Previous session exited with error code {}",
                                resumable.exit_code.unwrap()
                            )
                        } else {
                            "Previous session was interrupted".to_string()
                        };

                        let _resume_prompt =
                            build_resume_prompt(&original_prompt, &messages, &reason);

                        println!("Resume prompt prepared. Note: Conversation history is for context only.");
                        println!("The agent will review the current state and continue work.");
                        println!();

                        // TODO: Use resume_prompt instead of original prompt when spawning agent
                        // This will require passing conversation history to Claude Code
                        // For now, just inform the user
                        println!("{}  Resume functionality detected previous session but will start fresh.", emoji(no_emoji, "⚠️", "[WARN]"));
                        println!("Full resume with conversation history will be implemented in a future update.");
                        println!();
                    }
                }
                None => {
                    println!("No incomplete runs found for this issue.");
                    println!("Starting fresh...");
                    println!();
                }
            }
        }

        // Check dependencies unless --force
        if !self.force {
            // Use native GitHub tracking with markdown fallback
            let deps = match IssueDependencies::from_issue(&issue) {
                Ok(deps) => deps,
                Err(murmur_github::Error::InvalidDependencyRefs(refs)) => {
                    println!(
                        "{} Invalid dependency references found:",
                        emoji(no_emoji, "❌", "[ERROR]")
                    );
                    for invalid in &refs {
                        println!(
                            "  - \"{}\" (must be #123 or owner/repo#123 format)",
                            invalid
                        );
                    }
                    println!();
                    println!("Please fix the dependency references in the issue body.");
                    println!("Use --force to proceed anyway.");
                    return Ok(());
                }
                Err(e) => return Err(e.into()),
            };

            if deps.has_dependencies() {
                println!("Checking dependencies...");
                println!();

                let mut blocking = Vec::new();

                for dep_ref in deps.depends_on.iter().chain(deps.blocked_by.iter()) {
                    if !dep_ref.is_local() {
                        println!(
                            "  {}  {} (cross-repo, skipped)",
                            emoji(no_emoji, "⚠️", "[WARN]"),
                            dep_ref
                        );
                        continue;
                    }

                    let status = client.check_dependency_status(dep_ref.number).await?;
                    let dep_issue = client.get_issue(dep_ref.number).await;

                    let title = dep_issue
                        .as_ref()
                        .map(|i| i.title.as_str())
                        .unwrap_or("(unknown)");

                    match status {
                        DependencyStatus::Complete => {
                            println!(
                                "  {} #{}: {} [complete]",
                                emoji(no_emoji, "✅", "[OK]"),
                                dep_ref.number,
                                title
                            );
                        }
                        DependencyStatus::InProgress { pr_number } => {
                            println!(
                                "  {} #{}: {} [PR #{} open]",
                                emoji(no_emoji, "🔄", "[WIP]"),
                                dep_ref.number,
                                title,
                                pr_number
                            );
                            blocking.push((dep_ref.number, title.to_string(), Some(pr_number)));
                        }
                        DependencyStatus::Pending => {
                            println!(
                                "  {} #{}: {} [not started]",
                                emoji(no_emoji, "❌", "[PEND]"),
                                dep_ref.number,
                                title
                            );
                            blocking.push((dep_ref.number, title.to_string(), None));
                        }
                    }
                }

                println!();

                if !blocking.is_empty() {
                    println!(
                        "{} Blocked by {} unmet dependenc{}.",
                        emoji(no_emoji, "❌", "[ERROR]"),
                        blocking.len(),
                        if blocking.len() == 1 { "y" } else { "ies" }
                    );
                    println!();
                    println!("Options:");
                    for (i, (num, _, pr)) in blocking.iter().enumerate() {
                        if let Some(pr_num) = pr {
                            println!("  {}. Wait for PR #{} to merge", i + 1, pr_num);
                        } else {
                            println!(
                                "  {}. Run `murmur work {}` to start the blocking issue",
                                i + 1,
                                num
                            );
                        }
                    }
                    println!(
                        "  {}. Run `murmur work {} --force` to proceed anyway",
                        blocking.len() + 1,
                        self.issue
                    );
                    println!();
                    return Ok(());
                }

                println!(
                    "{} All dependencies satisfied!",
                    emoji(no_emoji, "✅", "[OK]")
                );
                println!();
            }
        } else {
            println!(
                "{}  Skipping dependency check (--force)",
                emoji(no_emoji, "⚠️", "[WARN]")
            );
            println!();
        }

        // Check if issue is already closed
        if issue.state == IssueState::Closed {
            println!(
                "{}  Issue #{} is already closed.",
                emoji(no_emoji, "⚠️", "[WARN]"),
                self.issue
            );
            if !self.force {
                println!("Use --force to work on it anyway.");
                return Ok(());
            }
        }

        // Check for existing worktree in database
        let worktree_repo = WorktreeRepository::new(&db);
        let branch_name = format!("murmur/issue-{}", self.issue);

        if let Ok(Some(existing)) = worktree_repo.find_by_branch(&branch_name) {
            let path = std::path::PathBuf::from(&existing.path);
            let exists_on_disk = path.exists();

            if existing.is_active() && exists_on_disk {
                println!(
                    "{}  Worktree already exists and is active:",
                    emoji(no_emoji, "⚠️", "[WARN]")
                );
                println!("  Path:   {}", existing.path);
                println!("  Branch: {}", existing.branch_name);
                println!();

                if !self.force {
                    println!("The worktree appears to be in use.");
                    println!("Options:");
                    println!("  1. Use --force to recreate it");
                    println!(
                        "  2. Use 'murmur worktree clean --stale-only' to clean up stale worktrees"
                    );
                    return Ok(());
                } else {
                    println!("Force flag detected. Removing existing worktree...");
                    // Delete the old database record
                    if let Err(e) = worktree_repo.delete_by_path(&existing.path) {
                        eprintln!("Warning: Failed to remove old worktree record: {}", e);
                    }
                    // Worktree directory and branch will be cleaned up by create_worktree logic
                }
            } else if !exists_on_disk {
                println!(
                    "{}  Found stale worktree entry in database:",
                    emoji(no_emoji, "⚠️", "[WARN]")
                );
                println!("  Path:   {} (missing)", existing.path);
                println!("  Branch: {}", existing.branch_name);
                println!();
                println!("Cleaning up stale entry...");

                // Mark as stale and continue
                let mut stale_record = existing.clone();
                stale_record.mark_stale();
                if let Err(e) = worktree_repo.update(&stale_record) {
                    eprintln!("Warning: Failed to update stale record: {}", e);
                }
            }
        }

        // Create worktree for the issue
        println!("Creating worktree for #{}...", self.issue);

        let cwd = std::env::current_dir()?;
        let git_repo = GitRepo::open(&cwd)?;

        let branching_options = BranchingOptions {
            base_branch: None,
            fetch: true,
            remote: None,
        };

        let point = git_repo.find_branching_point(&branching_options)?;

        if verbose {
            println!(
                "  Branching from {} ({})",
                point.reference,
                &point.commit[..8]
            );
        }

        let worktree_options = WorktreeOptions {
            branch_name: branch_name.clone(),
            force: self.force,
        };

        let info = git_repo.create_cached_worktree(&point, &worktree_options)?;

        // Save metadata
        let metadata =
            WorktreeMetadata::new(format!("issue-{}", self.issue), &point.commit, &branch_name);
        metadata.save(&info.path)?;

        println!("  Created: {}", info.path.display());
        println!("  Branch:  {}", info.branch);
        println!();

        // Track worktree in database immediately after creation to avoid race conditions
        // This is especially important when using --force, where the old record was deleted earlier
        let worktree_record =
            WorktreeRecord::new(info.path.to_string_lossy().to_string(), branch_name.clone())
                .with_issue_number(self.issue as i64)
                .with_main_repo_path(git_repo.root().to_string_lossy().to_string());

        let worktree_repo = WorktreeRepository::new(&db);
        let worktree_id = worktree_repo
            .insert(&worktree_record)
            .map_err(|e| anyhow::anyhow!("Failed to track worktree in database: {}", e))?;

        if verbose {
            println!("Worktree ID: {}", worktree_id);
        }

        if self.no_agent {
            println!("Worktree ready. Run your agent manually:");
            println!("  cd {}", info.path.display());
            return Ok(());
        }

        // Build prompt from issue
        let prompt = if let Some(ref custom_prompt) = self.prompt {
            custom_prompt.clone()
        } else {
            build_prompt_from_issue(&issue)
        };

        println!("Starting agent...");
        println!();

        // Create agent run record in database
        let config_json = serde_json::to_string(&config.agent).unwrap_or_else(|_| "{}".to_string());

        let mut agent_run = AgentRun::new(
            "implementer",
            &prompt,
            info.path.to_str().unwrap_or(""),
            config_json,
        )
        .with_issue_number(self.issue as i64);

        let agent_repo = AgentRunRepository::new(&db);
        let run_id = agent_repo
            .insert(&agent_run)
            .map_err(|e| anyhow::anyhow!("Failed to create agent run record: {}", e))?;

        agent_run.id = Some(run_id);

        if verbose {
            println!("Agent run ID: {}", run_id);
        }

        // Update worktree record with agent run linkage
        let worktree_repo = WorktreeRepository::new(&db);
        if let Ok(Some(mut wt_record)) = worktree_repo.find_by_path(&info.path.to_string_lossy()) {
            wt_record.agent_run_id = Some(run_id);
            if let Err(e) = worktree_repo.update(&wt_record) {
                eprintln!(
                    "Warning: Failed to update worktree with agent run ID: {}",
                    e
                );
            }
        }

        // Spawn agent with GitHub token if available
        let mut spawner = AgentSpawner::from_config(config.agent.clone());

        // Pass GitHub token to agent via environment variable
        if let Ok(secrets) = Secrets::load() {
            if let Some(token) = secrets.github_token() {
                spawner = spawner.with_env("GITHUB_TOKEN", token);
            }
        }

        let mut handle = spawner.spawn(&prompt, &info.path).await?;

        // Get PID and update the database record immediately to avoid race condition
        if let Some(pid) = handle.pid() {
            agent_run.pid = Some(pid as i32);
            // Update database with PID before doing anything else
            if let Err(e) = agent_repo.update(&agent_run) {
                eprintln!("Warning: Failed to update agent run with PID: {}", e);
            }

            if verbose {
                println!("Agent PID: {}", pid);
            }
        } else {
            eprintln!("Warning: Could not retrieve agent PID");
        }

        // Stream output with database logging
        let stdout = handle
            .child_mut()
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        let mut streamer = OutputStreamer::new(stdout);

        // Create a separate database connection for the handler
        let handler_db = Database::open()
            .map_err(|e| anyhow::anyhow!("Failed to open database for handler: {}", e))?;
        let mut handler = DatabaseLoggingHandler::new(handler_db, run_id, verbose);

        if let Err(e) = streamer.stream(&mut handler).await {
            eprintln!("Stream error: {}", e);
        }

        let status = handle.wait().await?;

        // Update agent run with completion status
        agent_run.complete(status.code().unwrap_or(-1));
        if let Err(e) = agent_repo.update(&agent_run) {
            eprintln!("Warning: Failed to update agent run record: {}", e);
        }

        // Update worktree status based on agent exit code
        if let Ok(Some(mut wt_record)) = worktree_repo.find_by_path(&info.path.to_string_lossy()) {
            if status.success() {
                wt_record.mark_completed();
            } else {
                wt_record.mark_abandoned();
            }
            if let Err(e) = worktree_repo.update(&wt_record) {
                eprintln!("Warning: Failed to update worktree status: {}", e);
            }
        }

        println!();
        if status.success() {
            println!(
                "{} Agent completed successfully",
                emoji(no_emoji, "✅", "[OK]")
            );

            // Auto-push and auto-PR if configured
            if config.workflow.auto_push || config.workflow.auto_pr {
                println!();
                self.handle_post_completion(
                    config,
                    &info,
                    &branch_name,
                    &issue.title,
                    verbose,
                    no_emoji,
                )
                .await?;
            } else {
                println!();
                println!("Next steps:");
                println!("  1. Review changes: cd {}", info.path.display());
                println!("  2. Push branch: git push -u origin {}", branch_name);
                println!(
                    "  3. Create PR: gh pr create --title \"Fixes #{}\"",
                    self.issue
                );
            }
        } else {
            println!(
                "{} Agent exited with code: {}",
                emoji(no_emoji, "❌", "[FAIL]"),
                status.code().unwrap_or(-1)
            );
            println!();
            println!("Next steps:");
            println!("  1. Review changes: cd {}", info.path.display());
            println!("  2. Fix issues and retry");
        }

        Ok(())
    }

    /// Handle post-completion tasks: push and PR creation
    async fn handle_post_completion(
        &self,
        config: &Config,
        info: &murmur_core::WorktreeInfo,
        branch_name: &str,
        issue_title: &str,
        verbose: bool,
        no_emoji: bool,
    ) -> anyhow::Result<()> {
        use std::process::Command;

        // Check if there are any commits to push
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&info.path)
            .output()?;

        let has_changes = !status_output.stdout.is_empty();

        // Get the default branch dynamically
        let default_branch_output = Command::new("git")
            .args(["symbolic-ref", "refs/remotes/origin/HEAD", "--short"])
            .current_dir(&info.path)
            .output()?;

        let default_branch = if default_branch_output.status.success() {
            String::from_utf8_lossy(&default_branch_output.stdout)
                .trim()
                .to_string()
        } else {
            // Fallback to origin/main if we can't determine the default branch
            "origin/main".to_string()
        };

        let log_output = Command::new("git")
            .args([
                "log",
                &format!("{}..{}", default_branch, branch_name),
                "--oneline",
            ])
            .current_dir(&info.path)
            .output()?;

        let has_commits = !log_output.stdout.is_empty();

        if !has_changes && !has_commits {
            println!("{}  No changes to push", emoji(no_emoji, "ℹ️", "[INFO]"));
            return Ok(());
        }

        // Auto-push if configured
        if config.workflow.auto_push {
            println!("Pushing branch to origin...");

            let push_result = Command::new("git")
                .args(["push", "-u", "origin", branch_name])
                .current_dir(&info.path)
                .output()?;

            if !push_result.status.success() {
                let stderr = String::from_utf8_lossy(&push_result.stderr);
                eprintln!(
                    "{}  Failed to push branch:",
                    emoji(no_emoji, "⚠️", "[WARN]")
                );
                eprintln!("{}", stderr);
                eprintln!();
                eprintln!("You can push manually with:");
                eprintln!(
                    "  cd {} && git push -u origin {}",
                    info.path.display(),
                    branch_name
                );

                if !config.workflow.auto_pr {
                    return Ok(());
                }
                eprintln!();
                eprintln!("Skipping PR creation due to push failure.");
                return Ok(());
            }

            println!(
                "{} Branch pushed successfully",
                emoji(no_emoji, "✅", "[OK]")
            );

            if verbose {
                let stdout = String::from_utf8_lossy(&push_result.stdout);
                if !stdout.is_empty() {
                    println!("{}", stdout);
                }
            }
        }

        // Auto-PR if configured
        if config.workflow.auto_pr {
            println!();
            println!("Creating pull request...");

            // Check for PR description file
            let pr_desc_path = info.path.join(".murmur").join("pr-description.md");

            let pr_result = if pr_desc_path.exists() {
                if verbose {
                    println!("Using PR description from: {}", pr_desc_path.display());
                }

                Command::new("gh")
                    .args([
                        "pr",
                        "create",
                        "--body-file",
                        pr_desc_path.to_str().unwrap(),
                    ])
                    .current_dir(&info.path)
                    .output()?
            } else {
                if verbose {
                    println!("No .murmur/pr-description.md found, using default description");
                }

                Command::new("gh")
                    .args([
                        "pr",
                        "create",
                        "--title",
                        issue_title,
                        "--body",
                        &format!("Closes #{}", self.issue),
                    ])
                    .current_dir(&info.path)
                    .output()?
            };

            if !pr_result.status.success() {
                let stderr = String::from_utf8_lossy(&pr_result.stderr);
                eprintln!("{}  Failed to create PR:", emoji(no_emoji, "⚠️", "[WARN]"));
                eprintln!("{}", stderr);
                eprintln!();

                // Check for common permission errors
                if stderr.contains("authentication")
                    || stderr.contains("token")
                    || stderr.contains("permission")
                {
                    eprintln!(
                        "{} This looks like a permission issue. Please ensure:",
                        emoji(no_emoji, "💡", "[TIP]")
                    );
                    eprintln!("  1. You have a valid GITHUB_TOKEN set");
                    eprintln!("  2. The token has 'repo' and 'workflow' scopes");
                    eprintln!("  3. You're authenticated with 'gh auth login'");
                    eprintln!();
                }

                eprintln!("You can create the PR manually with:");
                if pr_desc_path.exists() {
                    eprintln!(
                        "  cd {} && gh pr create --body-file .murmur/pr-description.md",
                        info.path.display()
                    );
                } else {
                    eprintln!(
                        "  cd {} && gh pr create --title \"{}\" --body \"Closes #{}\"",
                        info.path.display(),
                        issue_title,
                        self.issue
                    );
                }
                return Ok(());
            }

            let stdout = String::from_utf8_lossy(&pr_result.stdout);
            println!(
                "{} Pull request created successfully",
                emoji(no_emoji, "✅", "[OK]")
            );
            println!("{}", stdout.trim());
        }

        Ok(())
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

fn build_prompt_from_issue(issue: &murmur_github::Issue) -> String {
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

/// StreamHandler that logs to database and prints to console
struct DatabaseLoggingHandler {
    db: Database,
    run_id: i64,
    sequence: i64,
    verbose: bool,
}

impl DatabaseLoggingHandler {
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
                eprintln!("Warning: Failed to log message to database: {}", e);
            }
        }

        self.sequence += 1;
    }
}

impl murmur_core::StreamHandler for DatabaseLoggingHandler {
    fn on_system(&mut self, subtype: Option<&str>, session_id: Option<&str>) {
        let msg = serde_json::json!({
            "type": "system",
            "subtype": subtype,
            "session_id": session_id,
        });
        self.log_message("system", &msg.to_string());

        if self.verbose {
            if let Some(st) = subtype {
                eprintln!("[system: {}]", st);
            }
        }
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

        print!("{}", text);
    }

    fn on_tool_use(&mut self, tool: &str, input: &serde_json::Value) {
        let msg = serde_json::json!({
            "type": "tool_use",
            "tool": tool,
            "input": input,
        });
        self.log_message("tool_use", &msg.to_string());

        if self.verbose {
            eprintln!("\n[tool: {} with input: {}]", tool, input);
        }
    }

    fn on_tool_result(&mut self, output: &str, is_error: bool) {
        let msg = serde_json::json!({
            "type": "tool_result",
            "output": output,
            "is_error": is_error,
        });
        self.log_message("tool_result", &msg.to_string());

        if self.verbose {
            let prefix = if is_error { "error" } else { "result" };
            let display = if output.len() > 200 {
                format!("{}... ({} chars)", &output[..200], output.len())
            } else {
                output.to_string()
            };
            eprintln!("[{}: {}]", prefix, display);
        }
    }

    fn on_complete(&mut self, cost: Option<&murmur_core::CostInfo>, duration_ms: Option<u64>) {
        let msg = serde_json::json!({
            "type": "result",
            "cost": cost,
            "duration_ms": duration_ms,
        });
        self.log_message("result", &msg.to_string());

        println!();
        if self.verbose {
            if let Some(c) = cost {
                eprintln!("[tokens: {} in, {} out]", c.input_tokens, c.output_tokens);
            }
            if let Some(d) = duration_ms {
                eprintln!("[duration: {}ms]", d);
            }
        }
    }

    fn on_parse_error(&mut self, line: &str, error: &serde_json::Error) {
        if self.verbose {
            eprintln!("[parse error on line '{}': {}]", line, error);
        }
    }
}
