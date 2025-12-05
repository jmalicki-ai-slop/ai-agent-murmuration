//! Agent command - Start a typed agent with specialized behavior

use std::path::PathBuf;

use clap::{Args, Subcommand};
use murmur_core::{AgentFactory, AgentType, Config, OutputStreamer, PrintHandler};

/// Arguments for the agent command
#[derive(Args, Debug)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommands,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// Start an agent with a specific type
    Start(StartArgs),
}

/// Arguments for the start subcommand
#[derive(Args, Debug)]
pub struct StartArgs {
    /// The type of agent to start (implement, test, review, coordinator)
    #[arg(long = "type", short = 't', value_parser = parse_agent_type)]
    pub agent_type: AgentType,

    /// The task prompt describing what to accomplish
    #[arg(required = true)]
    pub prompt: String,

    /// Working directory for the task (defaults to current directory)
    #[arg(short = 'd', long, default_value = ".")]
    pub workdir: PathBuf,

    /// Dry run - show what would be executed without running
    #[arg(long)]
    pub dry_run: bool,
}

/// Parse agent type from string
fn parse_agent_type(s: &str) -> Result<AgentType, String> {
    s.parse::<AgentType>()
}

impl AgentArgs {
    /// Execute the agent command
    pub async fn execute(&self, verbose: bool, config: &Config) -> anyhow::Result<()> {
        match &self.command {
            AgentCommands::Start(args) => args.execute(verbose, config).await,
        }
    }
}

impl StartArgs {
    /// Execute the start command
    pub async fn execute(&self, verbose: bool, config: &Config) -> anyhow::Result<()> {
        // Resolve to absolute path
        let workdir = if self.workdir.is_absolute() {
            self.workdir.clone()
        } else {
            std::env::current_dir()?.join(&self.workdir)
        };

        if verbose {
            tracing::info!(
                agent_type = %self.agent_type,
                prompt = %self.prompt,
                workdir = %workdir.display(),
                claude_path = %config.agent.claude_path,
                model = ?config.agent.model,
                "Starting typed agent"
            );
        }

        println!("Murmuration Typed Agent");
        println!("========================");
        println!();
        println!(
            "Agent type: {} ({})",
            self.agent_type,
            self.agent_type.description()
        );
        println!("Prompt: {}", self.prompt);
        println!("Working directory: {}", workdir.display());
        if let Some(ref model) = config.agent.model {
            println!("Model: {}", model);
        }
        println!();

        if self.dry_run {
            println!(
                "[Dry run] Would spawn {} agent with the above configuration",
                self.agent_type
            );
            println!("[Dry run] Claude path: {}", config.agent.claude_path);
            return Ok(());
        }

        // Create factory with config
        let factory = AgentFactory::with_config(config.agent.clone());

        // Create the typed agent
        let typed_agent = factory.create(self.agent_type);

        println!("Spawning {} agent...", self.agent_type);

        // Spawn the agent with the task
        let mut handle = typed_agent.spawn_with_task(&self.prompt, &workdir).await?;

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
