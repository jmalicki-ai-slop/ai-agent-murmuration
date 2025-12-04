//! Run command - Execute a task with Murmuration agents

use std::path::PathBuf;

use clap::Args;
use murmur_core::AgentSpawner;

/// Arguments for the run command
#[derive(Args, Debug)]
pub struct RunArgs {
    /// The task prompt describing what to accomplish
    #[arg(required = true)]
    pub prompt: String,

    /// Working directory for the task (defaults to current directory)
    #[arg(short = 'd', long, default_value = ".")]
    pub workdir: PathBuf,

    /// Number of parallel agents to use
    #[arg(short = 'n', long, default_value = "1")]
    pub agents: u32,

    /// Dry run - show what would be executed without running
    #[arg(long)]
    pub dry_run: bool,
}

impl RunArgs {
    /// Execute the run command
    pub async fn execute(&self, verbose: bool) -> anyhow::Result<()> {
        // Resolve to absolute path
        let workdir = if self.workdir.is_absolute() {
            self.workdir.clone()
        } else {
            std::env::current_dir()?.join(&self.workdir)
        };

        if verbose {
            tracing::info!(
                prompt = %self.prompt,
                workdir = %workdir.display(),
                agents = %self.agents,
                "Starting murmur run"
            );
        }

        println!("Murmuration Run");
        println!("===============");
        println!();
        println!("Prompt: {}", self.prompt);
        println!("Working directory: {}", workdir.display());
        println!("Agents: {}", self.agents);
        println!();

        if self.dry_run {
            println!("[Dry run] Would spawn {} agent(s) with the above configuration", self.agents);
            return Ok(());
        }

        // For now, spawn a single agent (multi-agent orchestration comes later)
        let spawner = AgentSpawner::new();

        println!("Spawning Claude Code agent...");
        let mut handle = spawner.spawn(&self.prompt, &workdir).await?;

        println!("Agent started, waiting for completion...");
        println!("(Output streaming will be implemented in PR-004)");
        println!();

        // Wait for the process to complete
        let status = handle.wait().await?;

        if status.success() {
            println!("Agent completed successfully");
        } else {
            println!("Agent exited with status: {}", status);
        }

        Ok(())
    }
}
