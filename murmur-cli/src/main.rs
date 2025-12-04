//! Murmur CLI - Command line interface for Murmuration
//!
//! Multi-agent orchestration for software development with Claude Code.

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use commands::RunArgs;

/// Murmuration: Multi-agent orchestration for software development
#[derive(Parser, Debug)]
#[command(name = "murmur")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

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

    match cli.command {
        Some(Commands::Version) => {
            println!("murmur {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Run(args)) => {
            args.execute(cli.verbose).await?;
        }
        None => {
            println!("Murmuration - Multi-agent orchestration for software development");
            println!();
            println!("Use --help for usage information");
        }
    }

    Ok(())
}
