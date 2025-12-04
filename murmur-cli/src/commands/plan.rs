//! Plan management commands

use clap::{Args, Subcommand};
use murmur_core::parse_plan;
use murmur_github::{GitHubClient, ImportOptions};
use std::path::PathBuf;

/// Plan management commands
#[derive(Args, Debug)]
pub struct PlanArgs {
    #[command(subcommand)]
    pub command: PlanCommand,
}

#[derive(Subcommand, Debug)]
pub enum PlanCommand {
    /// Import PLAN.md as GitHub issues
    Import {
        /// Path to plan file
        #[arg(short, long, default_value = "PLAN.md")]
        file: PathBuf,

        /// Actually create issues (dry-run by default)
        #[arg(long)]
        execute: bool,

        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,

        /// Additional labels to add
        #[arg(short, long)]
        label: Vec<String>,
    },

    /// Show plan status
    Status {
        /// Path to plan file
        #[arg(short, long, default_value = "PLAN.md")]
        file: PathBuf,

        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,
    },
}

impl PlanArgs {
    /// Execute the plan command
    pub async fn execute(&self, verbose: bool, repo: Option<&str>) -> anyhow::Result<()> {
        match &self.command {
            PlanCommand::Import {
                file,
                execute,
                repo: cmd_repo,
                label,
            } => {
                let repo_ref = cmd_repo.as_deref().or(repo);
                import_plan(file, *execute, repo_ref, label, verbose).await
            }
            PlanCommand::Status {
                file,
                repo: cmd_repo,
            } => {
                let repo_ref = cmd_repo.as_deref().or(repo);
                show_status(file, repo_ref, verbose).await
            }
        }
    }
}

async fn import_plan(
    file: &PathBuf,
    execute: bool,
    repo: Option<&str>,
    labels: &[String],
    verbose: bool,
) -> anyhow::Result<()> {
    // Read and parse the plan file
    let content = std::fs::read_to_string(file)?;
    let plan = parse_plan(&content)?;

    let total_prs: usize = plan.phases.iter().map(|p| p.prs.len()).sum();
    println!("Parsed {}: {} phases, {} PRs", file.display(), plan.phases.len(), total_prs);
    println!();

    if !execute {
        // Dry run - show what would be created
        println!("Would create:");
        println!();

        for phase in &plan.phases {
            println!("  📁 Epic: Phase {}: {}", phase.id, phase.name);
            if !phase.goal.is_empty() {
                println!("     Goal: {}", phase.goal);
            }

            for pr in &phase.prs {
                let prefix = if pr.is_sub_pr { "    " } else { "  " };
                println!("{}  📝 {}: {}", prefix, pr.id, pr.description);

                if verbose && !pr.files.is_empty() {
                    for file in &pr.files {
                        println!("{}       └─ {}", prefix, file);
                    }
                }
            }
            println!();
        }

        println!("Run with --execute to create issues.");
        return Ok(());
    }

    // Execute - create issues
    let repo_str = repo.ok_or_else(|| {
        anyhow::anyhow!(
            "No repository specified. Use --repo owner/repo or run from a git repository"
        )
    })?;

    let client =
        GitHubClient::from_url(repo_str).map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Creating issues in {}/{}...", client.owner(), client.repo());
    println!();

    let options = ImportOptions {
        labels: labels.to_vec(),
        dry_run: false,
        skip_existing: true,
    };

    let result = client.import_plan(&plan, &options).await?;

    // Show results
    if result.created > 0 {
        println!();
        println!("Created {} issue(s):", result.created);

        for phase in &plan.phases {
            if let Some(&num) = result.epics.get(&phase.id) {
                println!("  ✅ #{} Phase {}: {}", num, phase.id, phase.name);
            }
            for pr in &phase.prs {
                if let Some(&num) = result.prs.get(&pr.id) {
                    println!("  ✅ #{} {}: {}", num, pr.id, pr.description);
                }
            }
        }
    }

    if result.skipped > 0 {
        println!();
        println!("Skipped {} existing issue(s).", result.skipped);
    }

    if !result.errors.is_empty() {
        println!();
        println!("Errors:");
        for err in &result.errors {
            println!("  ❌ {}", err);
        }
    }

    println!();
    println!(
        "Summary: {} created, {} skipped, {} errors",
        result.created,
        result.skipped,
        result.errors.len()
    );

    Ok(())
}

async fn show_status(file: &PathBuf, repo: Option<&str>, _verbose: bool) -> anyhow::Result<()> {
    // Read and parse the plan file
    let content = std::fs::read_to_string(file)?;
    let plan = parse_plan(&content)?;

    let repo_str = repo.ok_or_else(|| {
        anyhow::anyhow!(
            "No repository specified. Use --repo owner/repo or run from a git repository"
        )
    })?;

    let client =
        GitHubClient::from_url(repo_str).map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("Plan status for {}/{}:", client.owner(), client.repo());
    println!();

    // Fetch all issues to check status
    let issues = client.list_open_issues().await?;
    let closed_issues = client
        .list_issues(&murmur_github::IssueFilter {
            state: Some(murmur_github::IssueState::Closed),
            ..Default::default()
        })
        .await?;

    let all_issues: std::collections::HashMap<String, (u64, bool)> = issues
        .iter()
        .map(|i| (i.title.clone(), (i.number, false)))
        .chain(
            closed_issues
                .iter()
                .map(|i| (i.title.clone(), (i.number, true))),
        )
        .collect();

    let mut completed_prs = 0;
    let mut total_prs = 0;

    for phase in &plan.phases {
        let epic_title = format!("Phase {}: {}", phase.id, phase.name);
        let epic_status = all_issues.get(&epic_title);

        let icon = match epic_status {
            Some((_, true)) => "✅",
            Some((_, false)) => "🔄",
            None => "❌",
        };

        println!("{} Phase {}: {}", icon, phase.id, phase.name);

        for pr in &phase.prs {
            total_prs += 1;
            let pr_title = format!("{}: {}", pr.id, pr.description);
            let pr_status = all_issues.get(&pr_title);

            let icon = match pr_status {
                Some((_, true)) => {
                    completed_prs += 1;
                    "✅"
                }
                Some((_, false)) => "🔄",
                None => "❌",
            };

            let prefix = if pr.is_sub_pr { "    " } else { "  " };
            let num_str = pr_status
                .map(|(n, _)| format!("#{}", n))
                .unwrap_or_default();

            println!(
                "{}  {} {} {} {}",
                prefix,
                icon,
                pr.id,
                num_str,
                pr.description
            );
        }
        println!();
    }

    println!(
        "Progress: {}/{} PRs completed ({:.0}%)",
        completed_prs,
        total_prs,
        if total_prs > 0 {
            completed_prs as f64 / total_prs as f64 * 100.0
        } else {
            0.0
        }
    );

    Ok(())
}
