//! Murmur CLI - Command line interface for Murmuration
//!
//! Multi-agent orchestration for software development with Claude Code.

mod commands;

use clap::{Parser, Subcommand};
use murmur_core::{Config, GitRepo, Secrets};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use commands::{
    AgentArgs, IssueArgs, OrchestrateArgs, RunArgs, StatusArgs, WorkArgs, WorktreeArgs,
};

/// Try to detect the GitHub repo from the current directory
fn detect_repo() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let git_repo = GitRepo::open(&cwd).ok()?;
    let remote = git_repo.default_remote().ok()?;

    // Parse owner/repo from remote URL
    let url = &remote.url;
    if let Some(owner_repo) = parse_github_url(url) {
        return Some(owner_repo);
    }

    None
}

fn parse_github_url(url: &str) -> Option<String> {
    // Handle SSH: git@github.com:owner/repo.git
    if url.starts_with("git@github.com:") {
        let path = url.strip_prefix("git@github.com:")?;
        let path = path.strip_suffix(".git").unwrap_or(path);
        return Some(path.to_string());
    }

    // Handle HTTPS: https://github.com/owner/repo.git
    if let Some(path) = url.strip_prefix("https://github.com/") {
        let path = path.strip_suffix(".git").unwrap_or(path);
        return Some(path.to_string());
    }

    None
}

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

    /// Backend to use: claude or cursor (overrides config and env)
    #[arg(long, global = true, env = "MURMUR_BACKEND")]
    backend: Option<String>,

    /// Disable emoji output (use ASCII alternatives)
    #[arg(long, global = true)]
    no_emoji: bool,

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

    /// Start a typed agent with specialized behavior
    #[command(visible_alias = "a")]
    Agent(AgentArgs),

    /// Manage git worktrees
    #[command(visible_alias = "wt")]
    Worktree(WorktreeArgs),

    /// Manage GitHub issues
    #[command(visible_alias = "i")]
    Issue(IssueArgs),

    /// Work on a GitHub issue
    #[command(visible_alias = "w")]
    Work(WorkArgs),

    /// Orchestrate multiple issues from an epic
    #[command(visible_alias = "o")]
    Orchestrate(OrchestrateArgs),

    /// Show status of running agents and worktrees
    #[command(visible_alias = "s")]
    Status(StatusArgs),

    /// Show current configuration
    Config,

    /// Initialize secrets file
    #[command(visible_alias = "init")]
    SecretsInit,
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
    let config = Config::load_with_overrides(
        cli.claude_path.clone(),
        cli.model.clone(),
        cli.backend.clone(),
    )?;

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
        Some(Commands::Agent(args)) => {
            args.execute(cli.verbose, &config).await?;
        }
        Some(Commands::Worktree(args)) => {
            args.execute(cli.verbose).await?;
        }
        Some(Commands::Issue(args)) => {
            // Try to detect repo from current directory
            let repo = detect_repo();
            args.execute(cli.verbose, cli.no_emoji, repo.as_deref())
                .await?;
        }
        Some(Commands::Work(args)) => {
            // Try to detect repo from current directory
            let repo = detect_repo();
            args.execute(cli.verbose, cli.no_emoji, &config, repo.as_deref())
                .await?;
        }
        Some(Commands::Orchestrate(args)) => {
            // Try to detect repo from current directory
            let repo = detect_repo();
            args.execute(cli.verbose, cli.no_emoji, &config, repo.as_deref())
                .await?;
        }
        Some(Commands::Status(args)) => {
            args.execute(cli.verbose, cli.no_emoji).await?;
        }
        Some(Commands::Config) => {
            println!("Murmur Configuration");
            println!("====================");
            println!();
            println!("Agent Settings:");
            println!("  backend: {}", config.agent.backend);
            println!("  claude_path: {}", config.agent.claude_path);
            println!(
                "  model: {}",
                config.agent.model.as_deref().unwrap_or("(default)")
            );
            println!();
            if let Some(path) = Config::default_config_path() {
                println!("Config file: {}", path.display());
                if path.exists() {
                    println!("  (exists)");
                } else {
                    println!("  (not found - using defaults)");
                }
            }
            println!();
            if let Some(path) = Secrets::default_secrets_path() {
                println!("Secrets file: {}", path.display());
                if path.exists() {
                    println!("  (exists)");
                } else {
                    println!("  (not found - run 'murmur secrets-init' to create)");
                }
            }
        }
        Some(Commands::SecretsInit) => match Secrets::create_template() {
            Ok(path) => {
                println!("Created secrets file: {}", path.display());
                println!();
                println!("Please edit the file and add your GitHub token.");
                println!("Get a token at: https://github.com/settings/tokens");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        None => {
            println!("Murmuration - Multi-agent orchestration for software development");
            println!();
            println!("Use --help for usage information");
        }
    }

    Ok(())
}
