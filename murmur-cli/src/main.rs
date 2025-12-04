//! Murmur CLI - Command line interface for Murmuration
//!
//! Multi-agent orchestration for software development with Claude Code.

mod commands;

use clap::{Parser, Subcommand};
use murmur_core::Config;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use commands::{RunArgs, WorktreeArgs};

/// Murmuration: Multi-agent orchestration for software development
#[derive(Parser, Debug)]
#[command(name = "murmur")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Path to claude executable (overrides config and env)
    #[arg(long, global = true, env = "MURMUR_CLAUDE_PATH")]
    claude_path: Option<String>,

    /// Model to use (overrides config and env)
    #[arg(long, global = true, env = "MURMUR_MODEL")]
    model: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show version information
    Version,

    /// Run a task with Murmuration agents
    #[command(visible_alias = "r")]
    Run(RunArgs),

    /// Manage git worktrees
    #[command(visible_alias = "wt")]
    Worktree(WorktreeArgs),

    /// Show current configuration
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if cli.verbose {
        tracing::info!("Verbose mode enabled");
    }

    // Load configuration with overrides
    let config = Config::load_with_overrides(cli.claude_path.clone(), cli.model.clone())?;

    if cli.verbose {
        tracing::info!(
            claude_path = %config.agent.claude_path,
            model = ?config.agent.model,
            "Configuration loaded"
        );
    }

    match cli.command {
        Some(Commands::Version) => {
            println!("murmur {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Run(args)) => {
            args.execute(cli.verbose, &config).await?;
        }
        Some(Commands::Worktree(args)) => {
            args.execute(cli.verbose).await?;
        }
        Some(Commands::Config) => {
            println!("Murmur Configuration");
            println!("====================");
            println!();
            println!("Agent Settings:");
            println!("  claude_path: {}", config.agent.claude_path);
            println!("  model: {}", config.agent.model.as_deref().unwrap_or("(default)"));
            println!();
            if let Some(path) = Config::default_config_path() {
                println!("Config file: {}", path.display());
                if path.exists() {
                    println!("  (exists)");
                } else {
                    println!("  (not found - using defaults)");
                }
            }
        }
        None => {
            println!("Murmuration - Multi-agent orchestration for software development");
            println!();
            println!("Use --help for usage information");
        }
    }

    Ok(())
}
