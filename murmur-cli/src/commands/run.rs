//! Run command - Execute a task with Murmuration agents

use std::path::PathBuf;

use clap::Args;
use murmur_core::{AgentSpawner, Config, OutputStreamer, PrintHandler};

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
    pub async fn execute(&self, verbose: bool, config: &Config) -> anyhow::Result<()> {
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
                claude_path = %config.agent.claude_path,
                model = ?config.agent.model,
                "Starting murmur run"
            );
        }

        println!("Murmuration Run");
        println!("===============");
        println!();
        println!("Prompt: {}", self.prompt);
        println!("Working directory: {}", workdir.display());
        println!("Agents: {}", self.agents);
        if let Some(ref model) = config.agent.model {
            println!("Model: {}", model);
        }
        println!();

        if self.dry_run {
            println!(
                "[Dry run] Would spawn {} agent(s) with the above configuration",
                self.agents
            );
            println!("[Dry run] Claude path: {}", config.agent.claude_path);
            return Ok(());
        }

        // Create spawner from config (using default Implement agent type)
        let spawner = AgentSpawner::from_config(
            config.agent.clone(),
            murmur_core::agent::AgentType::default(),
        );

        println!("Spawning Claude Code agent...");
        let mut handle = spawner.spawn(&self.prompt, &workdir).await?;

        // Get stdout for streaming
        let stdout = handle
            .child_mut()
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture agent stdout"))?;

        println!("Agent started, streaming output...");
        println!();

        // Stream the output
        let mut streamer = OutputStreamer::new(stdout);
        let mut handler = PrintHandler::new(verbose);
        streamer.stream(&mut handler).await?;

        // Wait for the process to complete
        let status = handle.wait().await?;

        println!();
        if status.success() {
            println!("Agent completed successfully");
        } else {
            println!("Agent exited with status: {}", status);
        }

        Ok(())
    }
}
