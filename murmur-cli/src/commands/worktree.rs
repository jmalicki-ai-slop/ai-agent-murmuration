//! Worktree management commands

use clap::{Args, Subcommand};
use murmur_core::{
    BranchingOptions, GitRepo, PoolConfig, RepoUrl, WorktreeMetadata, WorktreeOptions,
    WorktreePool, WorktreeStatus,
};

/// Worktree management commands
#[derive(Args, Debug)]
pub struct WorktreeArgs {
    #[command(subcommand)]
    pub command: WorktreeCommand,
}

#[derive(Subcommand, Debug)]
pub enum WorktreeCommand {
    /// Create a worktree for a task
    Create {
        /// Task identifier (e.g., issue number or slug)
        task: String,

        /// Repository URL or shorthand (uses current repo if not specified)
        #[arg(short, long)]
        repo: Option<String>,

        /// Base branch to create from
        #[arg(short, long)]
        base: Option<String>,

        /// Force recreate if exists
        #[arg(short, long)]
        force: bool,
    },

    /// List worktrees
    List {
        /// Repository name filter
        #[arg(short, long)]
        repo: Option<String>,
    },

    /// Clean old worktrees
    Clean {
        /// Clean all non-active worktrees
        #[arg(long)]
        all: bool,

        /// Clean worktrees older than N days
        #[arg(long)]
        older_than: Option<u64>,

        /// Repository name filter
        #[arg(short, long)]
        repo: Option<String>,
    },

    /// Show worktree details
    Show {
        /// Task identifier
        task: String,

        /// Repository name
        #[arg(short, long)]
        repo: Option<String>,
    },
}

impl WorktreeArgs {
    /// Execute the worktree command
    pub async fn execute(&self, verbose: bool) -> anyhow::Result<()> {
        match &self.command {
            WorktreeCommand::Create {
                task,
                repo,
                base,
                force,
            } => {
                create_worktree(task, repo.as_deref(), base.as_deref(), *force, verbose).await
            }
            WorktreeCommand::List { repo } => list_worktrees(repo.as_deref(), verbose).await,
            WorktreeCommand::Clean {
                all,
                older_than,
                repo,
            } => clean_worktrees(*all, *older_than, repo.as_deref(), verbose).await,
            WorktreeCommand::Show { task, repo } => {
                show_worktree(task, repo.as_deref(), verbose).await
            }
        }
    }
}

async fn create_worktree(
    task: &str,
    repo_url: Option<&str>,
    base: Option<&str>,
    force: bool,
    verbose: bool,
) -> anyhow::Result<()> {
    // Determine the repository
    let (git_repo, repo_name) = if let Some(url) = repo_url {
        let parsed = RepoUrl::parse(url)?;
        let repo_path = murmur_core::clone_repo(&parsed, None)?;
        let repo = GitRepo::open(&repo_path)?;
        (repo, parsed.cache_name())
    } else {
        let cwd = std::env::current_dir()?;
        let repo = GitRepo::open(&cwd)?;
        let name = repo.repo_name();
        (repo, name)
    };

    if verbose {
        println!("Repository: {}", repo_name);
    }

    // Find branching point
    let branching_options = BranchingOptions {
        base_branch: base.map(|s| s.to_string()),
        fetch: true,
        remote: None,
    };

    let point = git_repo.find_branching_point(&branching_options)?;

    if verbose {
        println!("Branching from: {} ({})", point.reference, &point.commit[..8]);
    }

    // Create branch name
    let branch_name = format!("murmur/{}", task);

    let worktree_options = WorktreeOptions {
        branch_name: branch_name.clone(),
        force,
    };

    // Create the worktree
    let info = git_repo.create_cached_worktree(&point, &worktree_options)?;

    // Save metadata
    let metadata = WorktreeMetadata::new(task, &point.commit, &branch_name);
    metadata.save(&info.path)?;

    println!("Created worktree:");
    println!("  Path:   {}", info.path.display());
    println!("  Branch: {}", info.branch);
    println!("  Base:   {} ({})", point.reference, &point.commit[..8]);

    Ok(())
}

async fn list_worktrees(repo_filter: Option<&str>, verbose: bool) -> anyhow::Result<()> {
    let pool = WorktreePool::new()?;
    let cache_dir = pool.cache_dir();

    if !cache_dir.exists() {
        println!("No worktrees cached.");
        return Ok(());
    }

    let mut found_any = false;

    for entry in std::fs::read_dir(cache_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let repo_name = entry.file_name().to_string_lossy().to_string();

        // Apply filter
        if let Some(filter) = repo_filter {
            if !repo_name.contains(filter) {
                continue;
            }
        }

        let worktrees = pool.list_worktrees(&repo_name)?;
        if worktrees.is_empty() {
            continue;
        }

        found_any = true;
        println!("Repository: {}", repo_name);

        for wt in worktrees {
            let path_name = wt
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Some(meta) = &wt.metadata {
                let status = match meta.status {
                    WorktreeStatus::Active => "active",
                    WorktreeStatus::Completed => "completed",
                    WorktreeStatus::Abandoned => "abandoned",
                    WorktreeStatus::Available => "available",
                };
                println!(
                    "  {} [{}] - task: {}",
                    path_name, status, meta.task_id
                );

                if verbose {
                    println!("    Branch: {}", meta.branch);
                    println!("    Base:   {}", &meta.base_commit[..8.min(meta.base_commit.len())]);
                }
            } else {
                println!("  {} [unknown]", path_name);
            }
        }
        println!();
    }

    if !found_any {
        println!("No worktrees found.");
    }

    Ok(())
}

async fn clean_worktrees(
    all: bool,
    older_than: Option<u64>,
    repo_filter: Option<&str>,
    verbose: bool,
) -> anyhow::Result<()> {
    let mut config = PoolConfig::default();

    if all {
        config.max_per_repo = 0; // Remove all non-active
    }

    if let Some(days) = older_than {
        config.max_age_secs = days * 24 * 3600;
    }

    let pool = WorktreePool::with_config(config)?;
    let cache_dir = pool.cache_dir();

    if !cache_dir.exists() {
        println!("No worktrees to clean.");
        return Ok(());
    }

    let mut total_removed = 0;

    for entry in std::fs::read_dir(cache_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let repo_name = entry.file_name().to_string_lossy().to_string();

        // Apply filter
        if let Some(filter) = repo_filter {
            if !repo_name.contains(filter) {
                continue;
            }
        }

        let removed = pool.cleanup(&repo_name)?;
        if !removed.is_empty() {
            if verbose {
                println!("Cleaned from {}:", repo_name);
                for path in &removed {
                    println!("  - {}", path.display());
                }
            }
            total_removed += removed.len();
        }
    }

    println!("Cleaned {} worktree(s).", total_removed);

    Ok(())
}

async fn show_worktree(
    task: &str,
    repo_filter: Option<&str>,
    _verbose: bool,
) -> anyhow::Result<()> {
    let pool = WorktreePool::new()?;
    let cache_dir = pool.cache_dir();

    if !cache_dir.exists() {
        println!("Worktree not found.");
        return Ok(());
    }

    for entry in std::fs::read_dir(cache_dir)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }

        let repo_name = entry.file_name().to_string_lossy().to_string();

        // Apply filter
        if let Some(filter) = repo_filter {
            if !repo_name.contains(filter) {
                continue;
            }
        }

        let worktrees = pool.list_worktrees(&repo_name)?;

        for wt in worktrees {
            if let Some(meta) = &wt.metadata {
                if meta.task_id == task || meta.branch.contains(task) {
                    println!("Worktree Details");
                    println!("================");
                    println!();
                    println!("Repository: {}", repo_name);
                    println!("Path:       {}", wt.path.display());
                    println!("Task:       {}", meta.task_id);
                    println!("Branch:     {}", meta.branch);
                    println!("Base:       {}", meta.base_commit);
                    println!(
                        "Status:     {:?}",
                        meta.status
                    );

                    // Check if dirty
                    if let Ok(is_dirty) = pool.is_dirty(&wt.path) {
                        println!("Dirty:      {}", if is_dirty { "yes" } else { "no" });
                    }

                    return Ok(());
                }
            }
        }
    }

    println!("Worktree not found for task: {}", task);

    Ok(())
}
