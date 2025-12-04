//! Error types for GitHub operations

use thiserror::Error;

/// Result type for GitHub operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during GitHub operations
#[derive(Error, Debug)]
pub enum Error {
    /// GitHub API error
    #[error("GitHub API error: {0}")]
    Api(#[from] octocrab::Error),

    /// Authentication error
    #[error("GitHub authentication error: {0}")]
    Auth(String),

    /// Missing environment variable
    #[error("Missing environment variable: {0}")]
    MissingEnv(String),

    /// Issue not found
    #[error("Issue #{0} not found")]
    IssueNotFound(u64),

    /// Pull request not found
    #[error("Pull request #{0} not found")]
    PrNotFound(u64),

    /// Rate limit exceeded
    #[error("GitHub rate limit exceeded, resets at {0}")]
    RateLimited(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::MissingEnv(err.to_string())
    }
}
