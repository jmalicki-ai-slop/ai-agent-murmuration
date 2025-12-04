//! Git worktree creation and management

use std::path::{Path, PathBuf};
use std::process::Command;

use super::branch::BranchingPoint;
use super::repo::GitRepo;
use crate::{Error, Result};

/// Options for creating a worktree
#[derive(Debug, Clone)]
pub struct WorktreeOptions {
    /// Name for the new branch
    pub branch_name: String,
    /// Whether to force-recreate if exists
    pub force: bool,
}

/// Information about a created worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Path to the worktree directory
    pub path: PathBuf,
    /// Name of the branch
    pub branch: String,
    /// Commit SHA at creation
    pub commit: String,
}

/// Get the default cache directory for worktrees
///
/// Returns `~/.cache/murmur/worktrees`
pub fn default_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| Error::Config("Could not determine cache directory".to_string()))?;

    Ok(cache_dir.join("murmur").join("worktrees"))
}

/// Generate a worktree path from repo info and branch name
pub fn worktree_path(cache_dir: &Path, repo_name: &str, branch_name: &str) -> PathBuf {
    // Sanitize branch name for filesystem
    let safe_branch = branch_name
        .replace('/', "-")
        .replace('\\', "-")
        .replace(':', "-");

    cache_dir.join(repo_name).join(safe_branch)
}

impl GitRepo {
    /// Create a new worktree at the specified location
    ///
    /// This creates a new worktree with a new branch based on the given branching point.
    /// Uses `git worktree add` command for reliability.
    pub fn create_worktree(
        &self,
        worktree_dir: &Path,
        branching_point: &BranchingPoint,
        options: &WorktreeOptions,
    ) -> Result<WorktreeInfo> {
        // Check if worktree already exists
        if worktree_dir.exists() {
            if options.force {
                // Remove existing worktree
                self.remove_worktree(worktree_dir)?;
            } else {
                return Err(Error::Config(format!(
                    "Worktree already exists at {}. Use --force to recreate.",
                    worktree_dir.display()
                )));
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = worktree_dir.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Other(format!("Failed to create worktree directory: {}", e))
            })?;
        }

        // Use git worktree add command
        // git worktree add -b <branch> <path> <start-point>
        let mut cmd = Command::new("git");
        cmd.arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(&options.branch_name)
            .arg(worktree_dir)
            .arg(&branching_point.commit)
            .current_dir(self.root());

        let output = cmd.output().map_err(|e| {
            Error::Other(format!("Failed to run git worktree: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check if branch already exists
            if stderr.contains("already exists") {
                return Err(Error::Config(format!(
                    "Branch '{}' already exists. Choose a different name.",
                    options.branch_name
                )));
            }

            return Err(Error::Other(format!(
                "git worktree add failed: {}",
                stderr
            )));
        }

        Ok(WorktreeInfo {
            path: worktree_dir.to_path_buf(),
            branch: options.branch_name.clone(),
            commit: branching_point.commit.clone(),
        })
    }

    /// Remove a worktree
    pub fn remove_worktree(&self, worktree_dir: &Path) -> Result<()> {
        // First try git worktree remove
        let output = Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(worktree_dir)
            .current_dir(self.root())
            .output()
            .map_err(|e| Error::Other(format!("Failed to run git worktree remove: {}", e)))?;

        if !output.status.success() {
            // If git fails, try to remove the directory manually
            // This can happen if the worktree was not properly registered
            if worktree_dir.exists() {
                std::fs::remove_dir_all(worktree_dir).map_err(|e| {
                    Error::Other(format!("Failed to remove worktree directory: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// List all worktrees for this repository
    pub fn list_worktrees(&self) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .arg("worktree")
            .arg("list")
            .arg("--porcelain")
            .current_dir(self.root())
            .output()
            .map_err(|e| Error::Other(format!("Failed to run git worktree list: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Other("git worktree list failed".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();

        for line in stdout.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                worktrees.push(PathBuf::from(path));
            }
        }

        Ok(worktrees)
    }

    /// Generate a repo name for cache directory naming
    ///
    /// Uses the directory name of the repo root
    pub fn repo_name(&self) -> String {
        self.root()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Create a worktree in the default cache location
    pub fn create_cached_worktree(
        &self,
        branching_point: &BranchingPoint,
        options: &WorktreeOptions,
    ) -> Result<WorktreeInfo> {
        let cache_dir = default_cache_dir()?;
        let repo_name = self.repo_name();
        let worktree_dir = worktree_path(&cache_dir, &repo_name, &options.branch_name);

        self.create_worktree(&worktree_dir, branching_point, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_path_sanitization() {
        let cache = PathBuf::from("/tmp/cache");
        let path = worktree_path(&cache, "myrepo", "feature/foo-bar");
        assert_eq!(path, PathBuf::from("/tmp/cache/myrepo/feature-foo-bar"));
    }

    #[test]
    fn test_default_cache_dir() {
        let result = default_cache_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_str().unwrap().contains("murmur"));
        assert!(path.to_str().unwrap().contains("worktrees"));
    }
}
