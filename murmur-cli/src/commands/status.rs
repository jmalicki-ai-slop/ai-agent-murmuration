//! Status command - show running agents and worktree states

use chrono::Utc;
use clap::Args;
use murmur_core::{WorktreeMetadata, WorktreePool};
use murmur_db::{repos::AgentRunRepository, Database};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Show status of running agents and worktrees
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show verbose output including completed runs
    #[arg(short, long)]
    verbose: bool,
}

impl StatusArgs {
    /// Execute the status command
    pub async fn execute(&self, verbose: bool, no_emoji: bool) -> anyhow::Result<()> {
        // Open database
        let db = Database::open().map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let repo = AgentRunRepository::new(&db);

        // Find running agents
        let running = repo
            .find_running()
            .map_err(|e| anyhow::anyhow!("Failed to query running agents: {}", e))?;

        // Get last activity timestamps from conversation logs
        let mut last_activity: HashMap<i64, chrono::DateTime<Utc>> = HashMap::new();
        for run in &running {
            if let Some(run_id) = run.id {
                match murmur_db::ConversationRepository::new(&db).find_by_agent_run(run_id) {
                    Ok(conn) => {
                        if let Some(last_log) = conn.last() {
                            last_activity.insert(run_id, last_log.timestamp);
                        }
                    }
                    Err(e) => {
                        if self.verbose || verbose {
                            eprintln!(
                                "Warning: Failed to retrieve conversation log for run {}: {}",
                                run_id, e
                            );
                        }
                    }
                }
            }
        }

        println!();
        if running.is_empty() {
            println!("No running agents.");
        } else {
            println!("Running Agents:");
            println!();

            for run in &running {
                // Check if process is actually still running
                let pid = run.pid.unwrap_or(0);
                let is_alive = check_process_alive(pid);

                // Format issue number
                let issue_str = if let Some(issue) = run.issue_number {
                    format!("#{}", issue)
                } else {
                    "N/A".to_string()
                };

                // Extract branch from workdir metadata
                let branch = extract_branch_from_workdir(&run.workdir);

                // Calculate time ago
                let now = Utc::now();
                let started_ago = format_duration((now - run.start_time).num_seconds());

                println!("  {} [{}]", issue_str, run.agent_type);

                if let Some(branch_name) = branch {
                    println!("      Branch: {}", branch_name);
                }

                println!("      Workdir: {}", run.workdir);
                println!("      PID: {}", pid);
                println!("      Started: {} ago", started_ago);

                // Show last activity
                if let Some(run_id) = run.id {
                    if let Some(last_time) = last_activity.get(&run_id) {
                        let activity_ago = format_duration((now - *last_time).num_seconds());
                        println!("      Last activity: {} ago", activity_ago);
                    }
                }

                // Status indicator
                if is_alive {
                    println!("      Status: Active");
                } else {
                    println!("      Status: Stale (process not found)");
                }

                println!();
            }
        }

        // List worktrees
        let pool = WorktreePool::new()
            .map_err(|e| anyhow::anyhow!("Failed to create worktree pool: {}", e))?;
        let cache_dir = pool.cache_dir();

        // Find all worktrees across all repos
        let mut all_worktrees = Vec::new();
        if cache_dir.exists() {
            for entry in fs::read_dir(cache_dir)? {
                let entry = entry?;
                let repo_path = entry.path();
                if repo_path.is_dir() {
                    let repo_name = repo_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if let Ok(worktrees) = pool.list_worktrees(repo_name) {
                        for wt in worktrees {
                            all_worktrees.push((repo_name.to_string(), wt));
                        }
                    }
                }
            }
        }

        // Build a set of worktrees in use by running agents
        let mut active_worktrees: HashMap<PathBuf, i64> = HashMap::new();
        for run in &running {
            let workdir = PathBuf::from(&run.workdir);
            if let Some(issue) = run.issue_number {
                active_worktrees.insert(workdir, issue);
            }
        }

        // Find stale/orphaned worktrees
        let mut stale_worktrees = Vec::new();
        for (_repo_name, wt) in &all_worktrees {
            if !active_worktrees.contains_key(&wt.path) {
                stale_worktrees.push(wt);
            }
        }

        if !stale_worktrees.is_empty() {
            println!("Stale Worktrees:");
            println!();

            for wt in &stale_worktrees {
                if let Some(ref meta) = wt.metadata {
                    println!(
                        "  {} {} - orphaned (no agent running)",
                        meta.task_id, meta.branch
                    );
                    println!("      Path: {}", wt.path.display());

                    // Check if worktree is dirty
                    if let Ok(is_dirty) = pool.is_dirty(&wt.path) {
                        if is_dirty {
                            println!("      Status: Has uncommitted changes");
                        } else {
                            println!("      Status: Clean");
                        }
                    }

                    println!();
                } else {
                    println!("  {} - orphaned (no metadata)", wt.path.display());
                    println!();
                }
            }

            println!("Use 'murmur worktree clean' to remove stale worktrees.");
            println!();
        }

        // Show completed runs if verbose
        if self.verbose || verbose {
            let recent_completed = repo
                .find_all(Some(5))
                .map_err(|e| anyhow::anyhow!("Failed to query recent runs: {}", e))?;

            let completed: Vec<_> = recent_completed
                .into_iter()
                .filter(|r| r.is_completed())
                .collect();

            if !completed.is_empty() {
                println!("Recent Completed Runs:");
                println!();

                for run in completed {
                    let issue_str = if let Some(issue) = run.issue_number {
                        format!("#{}", issue)
                    } else {
                        "N/A".to_string()
                    };

                    let status_icon = if no_emoji {
                        if run.is_successful() {
                            "[OK]"
                        } else {
                            "[FAIL]"
                        }
                    } else if run.is_successful() {
                        "✅"
                    } else {
                        "❌"
                    };
                    let exit_code = run.exit_code.unwrap_or(-1);

                    println!(
                        "  {} {} [{}] - exit code {}",
                        status_icon, issue_str, run.agent_type, exit_code
                    );

                    if let Some(duration) = run.duration_seconds {
                        println!("      Duration: {:.1}s", duration);
                    }

                    println!();
                }
            }
        }

        Ok(())
    }
}

/// Check if a process with the given PID is still running
fn check_process_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        // Check if /proc/PID exists on Linux
        std::path::Path::new(&format!("/proc/{}", pid)).exists()
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use `kill -0` to check if process exists
        use std::process::Command;

        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use tasklist to check if process exists
        use std::process::Command;

        Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // For other systems, assume it's running
        true
    }
}

/// Format duration in seconds to human-readable string
fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}

/// Extract branch name from worktree path
fn extract_branch_from_workdir(workdir: &str) -> Option<String> {
    let path = PathBuf::from(workdir);
    if let Ok(meta) = WorktreeMetadata::load(&path) {
        Some(meta.branch)
    } else {
        None
    }
}
