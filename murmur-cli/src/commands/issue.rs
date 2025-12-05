//! Issue management commands

use clap::{Args, Subcommand, ValueEnum};
use murmur_github::{
    DependencyGraph, DependencyStatus, GitHubClient, IssueDependencies, IssueFilter, IssueMetadata,
    IssueState,
};

/// Issue management commands
#[derive(Args, Debug)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Subcommand, Debug)]
pub enum IssueCommand {
    /// List issues from repository
    List {
        /// Filter by state
        #[arg(short, long, default_value = "open")]
        state: StateFilter,

        /// Filter by label
        #[arg(short, long)]
        label: Option<String>,

        /// Repository (owner/repo format, uses current repo if not specified)
        #[arg(short, long)]
        repo: Option<String>,
    },

    /// Show issue details
    Show {
        /// Issue number
        number: u64,

        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,
    },

    /// Show issue dependency tree
    Deps {
        /// Issue number (or 'all' for complete graph)
        number: Option<u64>,

        /// Repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StateFilter {
    Open,
    Closed,
    All,
}

impl From<StateFilter> for Option<IssueState> {
    fn from(filter: StateFilter) -> Self {
        match filter {
            StateFilter::Open => Some(IssueState::Open),
            StateFilter::Closed => Some(IssueState::Closed),
            StateFilter::All => None,
        }
    }
}

impl IssueArgs {
    /// Execute the issue command
    pub async fn execute(
        &self,
        verbose: bool,
        no_emoji: bool,
        repo: Option<&str>,
    ) -> anyhow::Result<()> {
        match &self.command {
            IssueCommand::List {
                state,
                label,
                repo: cmd_repo,
            } => {
                let repo_ref = cmd_repo.as_deref().or(repo);
                list_issues(*state, label.as_deref(), repo_ref, verbose).await
            }
            IssueCommand::Show {
                number,
                repo: cmd_repo,
            } => {
                let repo_ref = cmd_repo.as_deref().or(repo);
                show_issue(*number, repo_ref, verbose).await
            }
            IssueCommand::Deps {
                number,
                repo: cmd_repo,
            } => {
                let repo_ref = cmd_repo.as_deref().or(repo);
                show_deps(*number, repo_ref, verbose, no_emoji).await
            }
        }
    }
}

/// Get emoji or ASCII alternative based on no_emoji flag
fn emoji<'a>(no_emoji: bool, emoji_char: &'a str, ascii_alt: &'a str) -> &'a str {
    if no_emoji {
        ascii_alt
    } else {
        emoji_char
    }
}

fn get_client(repo: Option<&str>) -> anyhow::Result<GitHubClient> {
    let repo_str = repo.ok_or_else(|| {
        anyhow::anyhow!(
            "No repository specified. Use --repo owner/repo or run from a git repository"
        )
    })?;

    GitHubClient::from_url(repo_str).map_err(|e| anyhow::anyhow!("{}", e))
}

async fn list_issues(
    state: StateFilter,
    label: Option<&str>,
    repo: Option<&str>,
    verbose: bool,
) -> anyhow::Result<()> {
    let client = get_client(repo)?;

    if verbose {
        println!(
            "Fetching issues from {}/{}...",
            client.owner(),
            client.repo()
        );
    }

    let filter = IssueFilter {
        state: state.into(),
        labels: label.map(|l| vec![l.to_string()]).unwrap_or_default(),
        per_page: Some(100),
    };

    let issues = client.list_issues(&filter).await?;

    if issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }

    println!(
        "Issues in {}/{} ({} found)",
        client.owner(),
        client.repo(),
        issues.len()
    );
    println!();

    for issue in issues {
        let state_icon = match issue.state {
            IssueState::Open => "○",
            IssueState::Closed => "●",
        };

        let labels = if !issue.labels.is_empty() {
            format!(" [{}]", issue.labels.join(", "))
        } else {
            String::new()
        };

        println!(
            "{} #{}: {}{}",
            state_icon, issue.number, issue.title, labels
        );
    }

    Ok(())
}

async fn show_issue(number: u64, repo: Option<&str>, verbose: bool) -> anyhow::Result<()> {
    let client = get_client(repo)?;

    if verbose {
        println!(
            "Fetching issue #{} from {}/{}...",
            number,
            client.owner(),
            client.repo()
        );
    }

    // Fetch issue with tracking information
    let issue = client.get_issue_with_tracking(number).await?;

    println!();
    println!("#{}: {}", issue.number, issue.title);
    println!(
        "{}",
        "=".repeat(issue.title.len() + format!("#{}: ", issue.number).len())
    );
    println!();

    let state_str = match issue.state {
        IssueState::Open => "open",
        IssueState::Closed => "closed",
    };
    println!("Status: {}", state_str);

    if !issue.labels.is_empty() {
        println!("Labels: {}", issue.labels.join(", "));
    }

    println!("Created: {}", issue.created_at.format("%Y-%m-%d %H:%M UTC"));
    println!("Updated: {}", issue.updated_at.format("%Y-%m-%d %H:%M UTC"));

    // Parse and show metadata
    if let Some(metadata) = IssueMetadata::parse(&issue.body) {
        println!();
        println!("Metadata:");
        if let Some(phase) = metadata.phase {
            println!("  Phase: {}", phase);
        }
        if let Some(pr) = &metadata.pr {
            println!("  PR: {}", pr);
        }
        if let Some(status) = &metadata.status {
            println!("  Status: {}", status);
        }
    }

    // Show native GitHub tracking
    if !issue.tracked_issues.is_empty() {
        println!();
        println!("Tracked Issues (GitHub Native):");
        for dep_num in &issue.tracked_issues {
            let status = match client.check_dependency_status(*dep_num).await {
                Ok(DependencyStatus::Complete) => "✅",
                Ok(DependencyStatus::InProgress { .. }) => "🔄",
                Ok(DependencyStatus::Pending) => "❌",
                Err(_) => "?",
            };
            println!("  {} #{}", status, dep_num);
        }
    }

    if !issue.tracked_in_issues.is_empty() {
        println!();
        println!("Tracked In (GitHub Native):");
        for parent_num in &issue.tracked_in_issues {
            println!("  ← #{}", parent_num);
        }
    }

    if let Some(ref summary) = issue.sub_issues_summary {
        if summary.total > 0 {
            println!();
            println!(
                "Sub-issues: {}/{} completed ({}%)",
                summary.completed, summary.total, summary.percent_completed
            );
        }
    }

    // Parse and show dependencies (fallback for markdown-based deps)
    match IssueDependencies::from_issue(&issue) {
        Ok(deps) if deps.has_dependencies() => {
            println!();
            println!("Dependencies (Markdown):");

            for dep in &deps.depends_on {
                let status = if dep.is_local() {
                    match client.check_dependency_status(dep.number).await {
                        Ok(DependencyStatus::Complete) => "✅",
                        Ok(DependencyStatus::InProgress { .. }) => "🔄",
                        Ok(DependencyStatus::Pending) => "❌",
                        Err(_) => "?",
                    }
                } else {
                    "?"
                };

                println!("  {} {} (depends on)", status, dep);
            }

            for dep in &deps.blocked_by {
                let status = if dep.is_local() {
                    match client.check_dependency_status(dep.number).await {
                        Ok(DependencyStatus::Complete) => "✅",
                        Ok(DependencyStatus::InProgress { .. }) => "🔄",
                        Ok(DependencyStatus::Pending) => "❌",
                        Err(_) => "?",
                    }
                } else {
                    "?"
                };

                println!("  {} {} (blocked by)", status, dep);
            }
        }
        Err(murmur_github::Error::InvalidDependencyRefs(refs)) => {
            println!();
            println!("⚠️  Invalid dependency references:");
            for r in refs {
                println!("  - \"{}\"", r);
            }
        }
        _ => {}
    }

    // Show description
    let body = issue
        .body
        .lines()
        .take_while(|line| !line.starts_with("<!-- murmur:metadata"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    if !body.is_empty() {
        println!();
        println!("Description:");
        for line in body.lines().take(20) {
            println!("  {}", line);
        }
        if body.lines().count() > 20 {
            println!("  ...(truncated)");
        }
    }

    Ok(())
}

async fn show_deps(
    number: Option<u64>,
    repo: Option<&str>,
    verbose: bool,
    no_emoji: bool,
) -> anyhow::Result<()> {
    let client = get_client(repo)?;

    if verbose {
        println!(
            "Building dependency graph for {}/{}...",
            client.owner(),
            client.repo()
        );
    }

    // Fetch all open issues
    let issues = client.list_open_issues().await?;

    if issues.is_empty() {
        println!("No open issues found.");
        return Ok(());
    }

    let graph = match DependencyGraph::from_issues(&issues) {
        Ok(g) => g,
        Err(murmur_github::Error::InvalidDependencyRefs(refs)) => {
            println!(
                "{} Invalid dependency references found in issues:",
                emoji(no_emoji, "❌", "[ERROR]")
            );
            for r in refs {
                println!("  - \"{}\"", r);
            }
            println!();
            println!("Please fix the dependency references in the issue bodies.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    // Check for cycles
    let cycles = graph.find_cycles();
    if !cycles.is_empty() {
        println!(
            "{}  Circular dependencies detected:",
            emoji(no_emoji, "⚠️", "[WARN]")
        );
        for cycle in &cycles {
            let cycle_str = cycle
                .iter()
                .map(|n| format!("#{}", n))
                .collect::<Vec<_>>()
                .join(" → ");
            println!("  {} → #{}", cycle_str, cycle[0]);
        }
        println!();
    }

    match number {
        Some(n) => {
            // Show deps for specific issue
            let issue = issues.iter().find(|i| i.number == n);
            if issue.is_none() {
                println!("Issue #{} not found in open issues.", n);
                return Ok(());
            }

            println!("Dependencies for #{}:", n);
            println!();

            if let Some(deps) = graph.dependencies.get(&n) {
                println!("  Depends on:");
                for dep in deps {
                    let issue_title = issues
                        .iter()
                        .find(|i| i.number == *dep)
                        .map(|i| i.title.as_str())
                        .unwrap_or("(unknown)");
                    let status = if graph.ready.contains(dep) {
                        emoji(no_emoji, "✅", "[OK]")
                    } else {
                        emoji(no_emoji, "❌", "[PEND]")
                    };
                    println!("    {} #{}: {}", status, dep, issue_title);
                }
            } else {
                println!("  No dependencies");
            }

            if let Some(dependents) = graph.dependents.get(&n) {
                println!();
                println!("  Depended on by:");
                for dep in dependents {
                    let issue_title = issues
                        .iter()
                        .find(|i| i.number == *dep)
                        .map(|i| i.title.as_str())
                        .unwrap_or("(unknown)");
                    println!("    #{}: {}", dep, issue_title);
                }
            }
        }
        None => {
            // Show full dependency graph
            println!("Dependency Graph ({} open issues)", issues.len());
            println!();

            // Show ready issues
            let ready: Vec<_> = graph.ready_issues();
            if !ready.is_empty() {
                println!("✅ Ready ({}):", ready.len());
                for n in &ready {
                    let issue = issues.iter().find(|i| i.number == *n);
                    if let Some(i) = issue {
                        println!("  #{}: {}", i.number, i.title);
                    }
                }
                println!();
            }

            // Show blocked issues
            let blocked: Vec<_> = graph.blocked_issues();
            if !blocked.is_empty() {
                println!("❌ Blocked ({}):", blocked.len());
                for n in &blocked {
                    let issue = issues.iter().find(|i| i.number == *n);
                    if let Some(i) = issue {
                        let deps = graph.dependencies.get(n).map(|d| {
                            d.iter()
                                .map(|n| format!("#{}", n))
                                .collect::<Vec<_>>()
                                .join(", ")
                        });
                        println!(
                            "  #{}: {} (waiting on {})",
                            i.number,
                            i.title,
                            deps.unwrap_or_default()
                        );
                    }
                }
                println!();
            }

            // Show topological order if available
            if let Some(order) = graph.topological_order() {
                println!("Execution order:");
                for (i, n) in order.iter().enumerate() {
                    let issue = issues.iter().find(|i| i.number == *n);
                    if let Some(iss) = issue {
                        let status = if graph.ready.contains(n) {
                            "✅"
                        } else {
                            "❌"
                        };
                        println!("  {}. {} #{}: {}", i + 1, status, n, iss.title);
                    }
                }
            }
        }
    }

    Ok(())
}
