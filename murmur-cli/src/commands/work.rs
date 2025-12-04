//! Work command - start working on an issue with dependency checking

use clap::Args;
use murmur_core::{
    AgentSpawner, BranchingOptions, Config, GitRepo, OutputStreamer, PrintHandler, ResumeManager,
    WorktreeMetadata, WorktreeOptions,
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

    /// Resume an interrupted workflow for this issue
    #[arg(long)]
    pub resume: bool,
}

impl WorkArgs {
    /// Execute the work command
    pub async fn execute(
        &self,
        verbose: bool,
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

        // Check for interrupted workflow if --resume is specified
        if self.resume {
            return self.execute_resume(verbose, config, &client).await;
        }

        // Fetch the issue
        let issue = client.get_issue(self.issue).await?;

        println!("#{}: {}", issue.number, issue.title);
        println!();

        // Check dependencies unless --force
        if !self.force {
            let deps = IssueDependencies::parse(&issue.body);

            if deps.has_dependencies() {
                println!("Checking dependencies...");
                println!();

                let mut blocking = Vec::new();

                for dep_ref in deps.depends_on.iter().chain(deps.blocked_by.iter()) {
                    if !dep_ref.is_local() {
                        println!("  ⚠️  {} (cross-repo, skipped)", dep_ref);
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
                            println!("  ✅ #{}: {} [complete]", dep_ref.number, title);
                        }
                        DependencyStatus::InProgress { pr_number } => {
                            println!(
                                "  🔄 #{}: {} [PR #{} open]",
                                dep_ref.number, title, pr_number
                            );
                            blocking.push((dep_ref.number, title.to_string(), Some(pr_number)));
                        }
                        DependencyStatus::Pending => {
                            println!("  ❌ #{}: {} [not started]", dep_ref.number, title);
                            blocking.push((dep_ref.number, title.to_string(), None));
                        }
                    }
                }

                println!();

                if !blocking.is_empty() {
                    println!(
                        "❌ Blocked by {} unmet dependenc{}.",
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

                println!("✅ All dependencies satisfied!");
                println!();
            }
        } else {
            println!("⚠️  Skipping dependency check (--force)");
            println!();
        }

        // Check if issue is already closed
        if issue.state == IssueState::Closed {
            println!("⚠️  Issue #{} is already closed.", self.issue);
            if !self.force {
                println!("Use --force to work on it anyway.");
                return Ok(());
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
        let branch_name = format!("murmur/issue-{}", self.issue);

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

        // Spawn agent
        let spawner = AgentSpawner::from_config(config.agent.clone());
        let mut handle = spawner.spawn(&prompt, &info.path).await?;

        // Stream output
        let stdout = handle
            .child_mut()
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

        let mut streamer = OutputStreamer::new(stdout);
        let mut handler = PrintHandler::new(verbose);

        if let Err(e) = streamer.stream(&mut handler).await {
            eprintln!("Stream error: {}", e);
        }

        let status = handle.wait().await?;

        println!();
        if status.success() {
            println!("✅ Agent completed successfully");
        } else {
            println!("❌ Agent exited with code: {}", status.code().unwrap_or(-1));
        }

        println!();
        println!("Next steps:");
        println!("  1. Review changes: cd {}", info.path.display());
        println!(
            "  2. Create PR: gh pr create --title \"Fixes #{}\"",
            self.issue
        );

        Ok(())
    }

    /// Execute resume workflow
    async fn execute_resume(
        &self,
        verbose: bool,
        config: &Config,
        client: &GitHubClient,
    ) -> anyhow::Result<()> {
        println!("Checking for interrupted workflow...");
        println!();

        let resume_mgr = ResumeManager::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize resume manager: {}", e))?;

        let task_id = format!("issue-{}", self.issue);
        let resume_info = resume_mgr
            .find_interrupted_for_task(&task_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check for interrupted runs: {}", e))?;

        let resume_info = match resume_info {
            Some(info) => info,
            None => {
                println!("❌ No interrupted workflow found for issue #{}.", self.issue);
                println!();
                println!("Run without --resume to start a new workflow.");
                return Ok(());
            }
        };

        println!("✅ Found interrupted workflow:");
        println!("  Run ID:       {}", resume_info.run_id);
        println!("  Agent Type:   {}", resume_info.agent_type);
        println!("  Worktree:     {}", resume_info.worktree_path);
        println!("  Messages:     {}", resume_info.message_count);
        println!();

        if !resume_info.worktree_exists() {
            println!(
                "⚠️  Worktree no longer exists: {}",
                resume_info.worktree_path
            );
            println!();
            println!("Cannot resume - worktree has been removed.");
            return Ok(());
        }

        // Fetch conversation history
        let conversation = resume_mgr
            .get_conversation_history(resume_info.run_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch conversation history: {}", e))?;

        if verbose {
            println!("Conversation history:");
            for log in &conversation {
                println!(
                    "  [{}] {}: {} chars",
                    log.sequence,
                    log.message_type,
                    log.content.len()
                );
            }
            println!();
        }

        println!(
            "⚠️  Resume functionality is not yet fully implemented in the agent spawner."
        );
        println!("   This feature requires passing conversation history to Claude Code,");
        println!("   which is planned for a future update.");
        println!();
        println!("Worktree location: {}", resume_info.worktree_path);
        println!(
            "You can manually continue work in the worktree or start a new workflow."
        );

        Ok(())
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
