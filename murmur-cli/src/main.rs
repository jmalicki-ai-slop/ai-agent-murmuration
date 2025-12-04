//! Murmur CLI - Command line interface for Murmuration
//!
//! Multi-agent orchestration for software development with Claude Code.

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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

#[derive(Parser, Debug)]
enum Commands {
    /// Show version information
    Version,
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
        None => {
            println!("Murmuration - Multi-agent orchestration for software development");
            println!();
            println!("Use --help for usage information");
        }
    }

    Ok(())
}
