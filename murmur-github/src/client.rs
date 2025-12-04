//! GitHub API client using octocrab

use crate::{Error, Result};
use murmur_core::Secrets;
use octocrab::Octocrab;
use tracing::{debug, info};

/// GitHub API client for repository operations
pub struct GitHubClient {
    client: Octocrab,
    owner: String,
    repo: String,
}

impl GitHubClient {
    /// Create a new GitHub client for the specified repository
    ///
    /// Token is loaded from (in priority order):
    /// 1. GITHUB_TOKEN environment variable
    /// 2. ~/.config/murmur/secrets.toml
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Result<Self> {
        let owner = owner.into();
        let repo = repo.into();

        // Load secrets (handles env var and secrets file)
        let secrets = Secrets::load().map_err(|e| Error::Auth(e.to_string()))?;

        let token = secrets.github_token().ok_or_else(|| {
            Error::Auth(
                "GitHub token not found. Set GITHUB_TOKEN environment variable \
                 or add token to ~/.config/murmur/secrets.toml"
                    .to_string(),
            )
        })?;

        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| Error::Auth(format!("Failed to create GitHub client: {}", e)))?;

        info!(owner = %owner, repo = %repo, "Created GitHub client");

        Ok(Self {
            client,
            owner,
            repo,
        })
    }

    /// Create a GitHub client from a repository URL
    ///
    /// Supports formats:
    /// - owner/repo
    /// - https://github.com/owner/repo
    /// - git@github.com:owner/repo.git
    pub fn from_url(url: &str) -> Result<Self> {
        let (owner, repo) = parse_github_url(url)?;
        Self::new(owner, repo)
    }

    /// Get the repository owner
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Get the repository name
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// Get the underlying octocrab client
    pub fn client(&self) -> &Octocrab {
        &self.client
    }

    /// Test the connection by fetching repository info
    pub async fn test_connection(&self) -> Result<()> {
        debug!(
            owner = %self.owner,
            repo = %self.repo,
            "Testing GitHub connection"
        );

        self.client
            .repos(&self.owner, &self.repo)
            .get()
            .await
            .map_err(|e| match e {
                octocrab::Error::GitHub { source, .. } => {
                    if source.message.contains("Not Found") {
                        Error::Other(format!(
                            "Repository {}/{} not found or not accessible",
                            self.owner, self.repo
                        ))
                    } else if source.message.contains("Bad credentials") {
                        Error::Auth("Invalid GitHub token".to_string())
                    } else {
                        Error::Api(octocrab::Error::GitHub {
                            source,
                            backtrace: std::backtrace::Backtrace::capture(),
                        })
                    }
                }
                other => Error::Api(other),
            })?;

        info!("GitHub connection successful");
        Ok(())
    }
}

impl std::fmt::Debug for GitHubClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHubClient")
            .field("owner", &self.owner)
            .field("repo", &self.repo)
            .finish_non_exhaustive()
    }
}

/// Parse a GitHub URL into owner and repo
fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Handle shorthand: owner/repo
    if !url.contains(':') && !url.contains('/') {
        return Err(Error::Parse(format!(
            "Invalid repository format: {}. Expected owner/repo",
            url
        )));
    }

    if !url.contains("://") && !url.contains('@') {
        // Simple owner/repo format
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() == 2 {
            return Ok((
                parts[0].to_string(),
                parts[1].trim_end_matches(".git").to_string(),
            ));
        }
        return Err(Error::Parse(format!(
            "Invalid repository format: {}. Expected owner/repo",
            url
        )));
    }

    // Handle HTTPS URL: https://github.com/owner/repo
    if url.starts_with("https://") || url.starts_with("http://") {
        let url = url::Url::parse(url).map_err(|e| Error::Parse(e.to_string()))?;
        let path = url.path().trim_start_matches('/').trim_end_matches(".git");
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
        return Err(Error::Parse(format!("Invalid GitHub URL path: {}", path)));
    }

    // Handle SSH URL: git@github.com:owner/repo.git
    if url.starts_with("git@") {
        if let Some(path) = url.split(':').nth(1) {
            let path = path.trim_end_matches(".git");
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return Ok((parts[0].to_string(), parts[1].to_string()));
            }
        }
        return Err(Error::Parse(format!("Invalid SSH URL: {}", url)));
    }

    Err(Error::Parse(format!("Unrecognized URL format: {}", url)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shorthand() {
        let (owner, repo) = parse_github_url("owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_https_url() {
        let (owner, repo) = parse_github_url("https://github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_https_url_with_git_suffix() {
        let (owner, repo) = parse_github_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_ssh_url() {
        let (owner, repo) = parse_github_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_github_url("invalid").is_err());
    }
}
