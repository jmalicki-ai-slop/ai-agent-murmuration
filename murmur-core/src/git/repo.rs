//! Git repository detection and operations

use std::path::{Path, PathBuf};

use git2::Repository;

use crate::{Error, Result};

/// Information about a git remote
#[derive(Debug, Clone)]
pub struct RemoteInfo {
    /// Name of the remote (e.g., "origin")
    pub name: String,
    /// URL of the remote
    pub url: String,
}

/// A git repository wrapper providing murmuration-specific operations
pub struct GitRepo {
    /// The underlying git2 repository
    repo: Repository,
    /// Path to the repository root
    root: PathBuf,
}

impl std::fmt::Debug for GitRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepo")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl GitRepo {
    /// Open a git repository at the given path
    ///
    /// This will search upward from the given path to find the repository root.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let repo = Repository::discover(path).map_err(|e| {
            if e.code() == git2::ErrorCode::NotFound {
                Error::Config(format!(
                    "Not a git repository: {}. Run 'git init' first or navigate to a git repository.",
                    path.display()
                ))
            } else {
                Error::Other(format!("Git error: {}", e))
            }
        })?;

        let root = repo
            .workdir()
            .ok_or_else(|| Error::Config("Bare repositories are not supported".to_string()))?
            .to_path_buf();

        Ok(Self { repo, root })
    }

    /// Get the repository root path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if the given path is inside a git repository
    pub fn is_git_repo(path: impl AsRef<Path>) -> bool {
        Repository::discover(path.as_ref()).is_ok()
    }

    /// Get the default remote (usually "origin")
    pub fn default_remote(&self) -> Result<RemoteInfo> {
        // Try origin first
        if let Ok(remote) = self.repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                return Ok(RemoteInfo {
                    name: "origin".to_string(),
                    url: url.to_string(),
                });
            }
        }

        // Fall back to first available remote
        let remotes = self.repo.remotes().map_err(|e| Error::Other(format!("Failed to list remotes: {}", e)))?;

        for remote_name in remotes.iter().flatten() {
            if let Ok(remote) = self.repo.find_remote(remote_name) {
                if let Some(url) = remote.url() {
                    return Ok(RemoteInfo {
                        name: remote_name.to_string(),
                        url: url.to_string(),
                    });
                }
            }
        }

        Err(Error::Config(
            "No remotes configured. Add a remote with 'git remote add origin <url>'".to_string(),
        ))
    }

    /// List all remotes
    pub fn list_remotes(&self) -> Result<Vec<RemoteInfo>> {
        let remotes = self.repo.remotes().map_err(|e| Error::Other(format!("Failed to list remotes: {}", e)))?;

        let mut result = Vec::new();
        for remote_name in remotes.iter().flatten() {
            if let Ok(remote) = self.repo.find_remote(remote_name) {
                if let Some(url) = remote.url() {
                    result.push(RemoteInfo {
                        name: remote_name.to_string(),
                        url: url.to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<Option<String>> {
        let head = match self.repo.head() {
            Ok(h) => h,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => return Ok(None),
            Err(e) => return Err(Error::Other(format!("Failed to get HEAD: {}", e))),
        };

        if head.is_branch() {
            Ok(head.shorthand().map(|s| s.to_string()))
        } else {
            // Detached HEAD
            Ok(None)
        }
    }

    /// Get the default branch name (main or master)
    pub fn default_branch(&self) -> Result<String> {
        // Check if origin/main exists
        if self.repo.find_reference("refs/remotes/origin/main").is_ok() {
            return Ok("main".to_string());
        }

        // Check if origin/master exists
        if self.repo.find_reference("refs/remotes/origin/master").is_ok() {
            return Ok("master".to_string());
        }

        // Check local main
        if self.repo.find_reference("refs/heads/main").is_ok() {
            return Ok("main".to_string());
        }

        // Check local master
        if self.repo.find_reference("refs/heads/master").is_ok() {
            return Ok("master".to_string());
        }

        // Default to main
        Ok("main".to_string())
    }

    /// Get access to the underlying git2 repository
    pub fn inner(&self) -> &Repository {
        &self.repo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_open_current_dir() {
        // This test assumes we're running in a git repo
        let cwd = env::current_dir().unwrap();
        if GitRepo::is_git_repo(&cwd) {
            let repo = GitRepo::open(&cwd).unwrap();
            assert!(repo.root().exists());
        }
    }

    #[test]
    fn test_is_git_repo_negative() {
        assert!(!GitRepo::is_git_repo("/tmp"));
    }

    #[test]
    fn test_open_non_git_dir() {
        let result = GitRepo::open("/tmp");
        assert!(result.is_err());
    }
}
