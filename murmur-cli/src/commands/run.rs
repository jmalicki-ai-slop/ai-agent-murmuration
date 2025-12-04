//! Run command - Execute a task with Murmuration agents

use clap::Args;

/// Arguments for the run command
#[derive(Args, Debug)]
pub struct RunArgs {
    /// The task prompt describing what to accomplish
    #[arg(required = true)]
    pub prompt: String,

    /// Working directory for the task (defaults to current directory)
    #[arg(short = 'd', long, default_value = ".")]
    pub workdir: String,

    /// Number of parallel agents to use
    #[arg(short = 'n', long, default_value = "1")]
    pub agents: u32,
}

impl RunArgs {
    /// Execute the run command
    pub async fn execute(&self, verbose: bool) -> anyhow::Result<()> {
        if verbose {
            tracing::info!(
                prompt = %self.prompt,
                workdir = %self.workdir,
                agents = %self.agents,
                "Starting murmur run"
            );
        }

        println!("Murmuration Run");
        println!("===============");
        println!();
        println!("Prompt: {}", self.prompt);
        println!("Working directory: {}", self.workdir);
        println!("Agents: {}", self.agents);
        println!();
        println!("(Agent spawning will be implemented in PR-003)");

        Ok(())
    }
}
