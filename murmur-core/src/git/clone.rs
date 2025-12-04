//! Git repository cloning and URL parsing

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{Error, Result};

/// Parsed repository information
#[derive(Debug, Clone)]
pub struct RepoUrl {
    /// Repository owner/organization
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Full clone URL
    pub clone_url: String,
    /// Host (e.g., "github.com")
    pub host: String,
}

impl RepoUrl {
    /// Parse a repository URL or shorthand
    ///
    /// Supports:
    /// - `https://github.com/owner/repo`
    /// - `https://github.com/owner/repo.git`
    /// - `git@github.com:owner/repo.git`
    /// - `owner/repo` (assumes GitHub)
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        // Handle owner/repo shorthand (assumes GitHub)
        if !input.contains("://") && !input.contains('@') && input.contains('/') {
            let parts: Vec<&str> = input.split('/').collect();
            if parts.len() == 2 {
                let owner = parts[0].to_string();
                let repo = parts[1].trim_end_matches(".git").to_string();
                return Ok(Self {
                    owner: owner.clone(),
                    repo: repo.clone(),
                    clone_url: format!("https://github.com/{}/{}.git", owner, repo),
                    host: "github.com".to_string(),
                });
            }
        }

        // Handle git@ URLs (e.g., git@github.com:owner/repo.git)
        if input.starts_with("git@") {
            if let Some(rest) = input.strip_prefix("git@") {
                if let Some((host, path)) = rest.split_once(':') {
                    let path = path.trim_end_matches(".git");
                    let parts: Vec<&str> = path.split('/').collect();
                    if parts.len() >= 2 {
                        let owner = parts[0].to_string();
                        let repo = parts[1].to_string();
                        return Ok(Self {
                            owner: owner.clone(),
                            repo: repo.clone(),
                            clone_url: input.to_string(),
                            host: host.to_string(),
                        });
                    }
                }
            }
        }

        // Handle https:// URLs
        if input.starts_with("https://") || input.starts_with("http://") {
            if let Ok(url) = url::Url::parse(input) {
                let host = url.host_str().unwrap_or("").to_string();
                let path = url.path().trim_start_matches('/').trim_end_matches(".git");
                let parts: Vec<&str> = path.split('/').collect();

                if parts.len() >= 2 {
                    let owner = parts[0].to_string();
                    let repo = parts[1].to_string();
                    let clone_url = if input.ends_with(".git") {
                        input.to_string()
                    } else {
                        format!("{}.git", input)
                    };

                    return Ok(Self {
                        owner,
                        repo,
                        clone_url,
                        host,
                    });
                }
            }
        }

        Err(Error::Config(format!(
            "Invalid repository URL: {}. Expected format: owner/repo, https://github.com/owner/repo, or git@github.com:owner/repo.git",
            input
        )))
    }

    /// Get the directory name for caching (owner-repo)
    pub fn cache_name(&self) -> String {
        format!("{}-{}", self.owner, self.repo)
    }
}

/// Get the default repos cache directory
///
/// Returns `~/.cache/murmur/repos`
pub fn default_repos_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| Error::Config("Could not determine cache directory".to_string()))?;

    Ok(cache_dir.join("murmur").join("repos"))
}

/// Clone a repository to the cache
pub fn clone_repo(repo_url: &RepoUrl, cache_dir: Option<&Path>) -> Result<PathBuf> {
    let base_dir = match cache_dir {
        Some(dir) => dir.to_path_buf(),
        None => default_repos_cache_dir()?,
    };

    let target_dir = base_dir.join(&repo_url.owner).join(&repo_url.repo);

    // If already exists, just fetch
    if target_dir.exists() {
        fetch_repo(&target_dir)?;
        return Ok(target_dir);
    }

    // Create parent directories
    if let Some(parent) = target_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::Other(format!("Failed to create repos cache directory: {}", e)))?;
    }

    // Clone the repository
    let output = Command::new("git")
        .arg("clone")
        .arg(&repo_url.clone_url)
        .arg(&target_dir)
        .output()
        .map_err(|e| Error::Other(format!("Failed to run git clone: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for common error types
        if stderr.contains("Authentication failed") || stderr.contains("Permission denied") {
            return Err(Error::Config(format!(
                "Authentication failed for {}. Check your credentials or repository access.",
                repo_url.clone_url
            )));
        }

        if stderr.contains("Could not resolve host") || stderr.contains("unable to access") {
            return Err(Error::Config(format!(
                "Network error cloning {}. Check your internet connection.",
                repo_url.clone_url
            )));
        }

        if stderr.contains("not found") || stderr.contains("does not exist") {
            return Err(Error::Config(format!(
                "Repository not found: {}. Check the URL is correct.",
                repo_url.clone_url
            )));
        }

        return Err(Error::Other(format!("git clone failed: {}", stderr)));
    }

    Ok(target_dir)
}

/// Fetch latest from remote for a cached repository
pub fn fetch_repo(repo_path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("fetch")
        .arg("--all")
        .arg("--prune")
        .current_dir(repo_path)
        .output()
        .map_err(|e| Error::Other(format!("Failed to run git fetch: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("git fetch failed for {:?}: {}", repo_path, stderr);
        // Don't fail on fetch errors - repo may still be usable
    }

    Ok(())
}

/// Check if a repository is cached
pub fn is_repo_cached(repo_url: &RepoUrl, cache_dir: Option<&Path>) -> Result<bool> {
    let base_dir = match cache_dir {
        Some(dir) => dir.to_path_buf(),
        None => default_repos_cache_dir()?,
    };

    let target_dir = base_dir.join(&repo_url.owner).join(&repo_url.repo);
    Ok(target_dir.exists() && target_dir.join(".git").exists())
}

/// Get the path to a cached repository
pub fn cached_repo_path(repo_url: &RepoUrl, cache_dir: Option<&Path>) -> Result<PathBuf> {
    let base_dir = match cache_dir {
        Some(dir) => dir.to_path_buf(),
        None => default_repos_cache_dir()?,
    };

    Ok(base_dir.join(&repo_url.owner).join(&repo_url.repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shorthand() {
        let url = RepoUrl::parse("owner/repo").unwrap();
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
        assert_eq!(url.host, "github.com");
        assert_eq!(url.clone_url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_parse_https() {
        let url = RepoUrl::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
        assert_eq!(url.host, "github.com");
    }

    #[test]
    fn test_parse_https_with_git() {
        let url = RepoUrl::parse("https://github.com/owner/repo.git").unwrap();
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
    }

    #[test]
    fn test_parse_git_ssh() {
        let url = RepoUrl::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
        assert_eq!(url.host, "github.com");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(RepoUrl::parse("invalid").is_err());
        assert!(RepoUrl::parse("").is_err());
    }

    #[test]
    fn test_cache_name() {
        let url = RepoUrl::parse("owner/repo").unwrap();
        assert_eq!(url.cache_name(), "owner-repo");
    }
}
