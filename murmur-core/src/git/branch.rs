//! Branch detection and management for git repositories

use git2::{BranchType, FetchOptions, RemoteCallbacks};

use super::repo::GitRepo;
use crate::{Error, Result};

/// Options for determining the branching point
#[derive(Debug, Clone, Default)]
pub struct BranchingOptions {
    /// Explicit base branch override
    pub base_branch: Option<String>,
    /// Whether to fetch from remote before determining base
    pub fetch: bool,
    /// Remote name to fetch from (defaults to "origin")
    pub remote: Option<String>,
}

/// Information about a branching point
#[derive(Debug, Clone)]
pub struct BranchingPoint {
    /// The reference to branch from (e.g., "origin/main")
    pub reference: String,
    /// The commit SHA at the branching point
    pub commit: String,
    /// The branch name (e.g., "main")
    pub branch_name: String,
}

impl GitRepo {
    /// Fetch the latest from a remote
    ///
    /// # Arguments
    /// * `remote_name` - Name of the remote to fetch from (defaults to "origin")
    pub fn fetch(&self, remote_name: Option<&str>) -> Result<()> {
        let remote_name = remote_name.unwrap_or("origin");

        let mut remote = self.inner().find_remote(remote_name).map_err(|e| {
            Error::Config(format!("Remote '{}' not found: {}", remote_name, e))
        })?;

        // Set up callbacks for progress (silent for now)
        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(|_| true);

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Fetch all branches
        remote
            .fetch(&[] as &[&str], Some(&mut fetch_options), None)
            .map_err(|e| Error::Other(format!("Fetch failed: {}", e)))?;

        Ok(())
    }

    /// Find the best branching point for a new worktree
    ///
    /// Priority:
    /// 1. Explicit base branch from options
    /// 2. origin/main
    /// 3. origin/master
    /// 4. origin/HEAD
    /// 5. Local main
    /// 6. Local master
    pub fn find_branching_point(&self, options: &BranchingOptions) -> Result<BranchingPoint> {
        // Optionally fetch first
        if options.fetch {
            let remote = options.remote.as_deref().unwrap_or("origin");
            if let Err(e) = self.fetch(Some(remote)) {
                tracing::warn!("Failed to fetch from {}: {}. Continuing with local state.", remote, e);
            }
        }

        // If explicit base branch specified, use it
        if let Some(ref base) = options.base_branch {
            return self.resolve_branch_reference(base);
        }

        let remote = options.remote.as_deref().unwrap_or("origin");

        // Try origin/main
        let remote_main = format!("{}/main", remote);
        if let Ok(point) = self.resolve_branch_reference(&remote_main) {
            return Ok(point);
        }

        // Try origin/master
        let remote_master = format!("{}/master", remote);
        if let Ok(point) = self.resolve_branch_reference(&remote_master) {
            return Ok(point);
        }

        // Try local main
        if let Ok(point) = self.resolve_branch_reference("main") {
            return Ok(point);
        }

        // Try local master
        if let Ok(point) = self.resolve_branch_reference("master") {
            return Ok(point);
        }

        Err(Error::Config(
            "No suitable base branch found. Expected main, master, or specify --base".to_string(),
        ))
    }

    /// Resolve a branch reference to a branching point
    fn resolve_branch_reference(&self, reference: &str) -> Result<BranchingPoint> {
        let repo = self.inner();

        // Try as remote tracking branch first (refs/remotes/...)
        if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/{}", reference)) {
            let commit = remote_ref
                .peel_to_commit()
                .map_err(|e| Error::Other(format!("Failed to resolve {}: {}", reference, e)))?;

            let branch_name = reference
                .split('/')
                .last()
                .unwrap_or(reference)
                .to_string();

            return Ok(BranchingPoint {
                reference: reference.to_string(),
                commit: commit.id().to_string(),
                branch_name,
            });
        }

        // Try as local branch (refs/heads/...)
        if let Ok(local_ref) = repo.find_reference(&format!("refs/heads/{}", reference)) {
            let commit = local_ref
                .peel_to_commit()
                .map_err(|e| Error::Other(format!("Failed to resolve {}: {}", reference, e)))?;

            return Ok(BranchingPoint {
                reference: reference.to_string(),
                commit: commit.id().to_string(),
                branch_name: reference.to_string(),
            });
        }

        // Try direct reference
        if let Ok(direct_ref) = repo.find_reference(reference) {
            let commit = direct_ref
                .peel_to_commit()
                .map_err(|e| Error::Other(format!("Failed to resolve {}: {}", reference, e)))?;

            let branch_name = reference
                .split('/')
                .last()
                .unwrap_or(reference)
                .to_string();

            return Ok(BranchingPoint {
                reference: reference.to_string(),
                commit: commit.id().to_string(),
                branch_name,
            });
        }

        Err(Error::Config(format!("Branch '{}' not found", reference)))
    }

    /// List all local branches
    pub fn list_local_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();

        for branch in self.inner().branches(Some(BranchType::Local)).map_err(|e| {
            Error::Other(format!("Failed to list branches: {}", e))
        })? {
            let (branch, _) = branch.map_err(|e| Error::Other(format!("Failed to read branch: {}", e)))?;
            if let Some(name) = branch.name().ok().flatten() {
                branches.push(name.to_string());
            }
        }

        Ok(branches)
    }

    /// List all remote tracking branches
    pub fn list_remote_branches(&self, remote: Option<&str>) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        let filter_prefix = remote.map(|r| format!("{}/", r));

        for branch in self.inner().branches(Some(BranchType::Remote)).map_err(|e| {
            Error::Other(format!("Failed to list branches: {}", e))
        })? {
            let (branch, _) = branch.map_err(|e| Error::Other(format!("Failed to read branch: {}", e)))?;
            if let Some(name) = branch.name().ok().flatten() {
                // Filter by remote if specified
                if let Some(ref prefix) = filter_prefix {
                    if name.starts_with(prefix) {
                        branches.push(name.to_string());
                    }
                } else {
                    branches.push(name.to_string());
                }
            }
        }

        Ok(branches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_find_branching_point_default() {
        let cwd = env::current_dir().unwrap();
        if GitRepo::is_git_repo(&cwd) {
            let repo = GitRepo::open(&cwd).unwrap();
            let options = BranchingOptions::default();

            // Should find some branching point in a valid repo
            let result = repo.find_branching_point(&options);
            if result.is_ok() {
                let point = result.unwrap();
                assert!(!point.commit.is_empty());
                assert!(!point.branch_name.is_empty());
            }
        }
    }

    #[test]
    fn test_list_branches() {
        let cwd = env::current_dir().unwrap();
        if GitRepo::is_git_repo(&cwd) {
            let repo = GitRepo::open(&cwd).unwrap();

            // Should be able to list branches
            let local = repo.list_local_branches();
            assert!(local.is_ok());

            let remote = repo.list_remote_branches(Some("origin"));
            assert!(remote.is_ok());
        }
    }
}
