//! Pull request status checking

use crate::{Error, GitHubClient, Issue, IssueState, Result};
use chrono::{DateTime, Utc};
use octocrab::models::pulls::PullRequest as OctocrabPR;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Pull request representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR number
    pub number: u64,
    /// PR title
    pub title: String,
    /// PR body
    pub body: String,
    /// Current state (open, closed)
    pub state: PrState,
    /// Whether the PR has been merged (derived from merged_at)
    pub merged: bool,
    /// Merge commit SHA (if merged)
    pub merge_commit_sha: Option<String>,
    /// When the PR was created
    pub created_at: DateTime<Utc>,
    /// When the PR was last updated
    pub updated_at: DateTime<Utc>,
    /// When the PR was merged (if merged)
    pub merged_at: Option<DateTime<Utc>>,
    /// Head branch name
    pub head_branch: String,
    /// Base branch name
    pub base_branch: String,
}

/// PR state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Closed,
}

impl From<octocrab::models::IssueState> for PrState {
    fn from(state: octocrab::models::IssueState) -> Self {
        match state {
            octocrab::models::IssueState::Open => PrState::Open,
            octocrab::models::IssueState::Closed => PrState::Closed,
            _ => PrState::Open, // Default for unknown states
        }
    }
}

impl From<OctocrabPR> for PullRequest {
    fn from(pr: OctocrabPR) -> Self {
        // Determine merged status from merged_at field
        let merged = pr.merged_at.is_some();

        PullRequest {
            number: pr.number,
            title: pr.title.unwrap_or_default(),
            body: pr.body.unwrap_or_default(),
            state: pr.state.map(|s| s.into()).unwrap_or(PrState::Open),
            merged,
            merge_commit_sha: pr.merge_commit_sha,
            created_at: pr.created_at.unwrap_or_else(Utc::now),
            updated_at: pr.updated_at.unwrap_or_else(Utc::now),
            merged_at: pr.merged_at,
            head_branch: pr.head.ref_field,
            base_branch: pr.base.ref_field,
        }
    }
}

/// Status of a dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyStatus {
    /// Issue is still open, no linked PR merged
    Pending,
    /// PR exists but not merged yet
    InProgress {
        /// The PR number
        pr_number: u64,
    },
    /// PR merged or issue closed - dependency is satisfied
    Complete,
}

impl DependencyStatus {
    /// Check if the dependency is satisfied (complete)
    pub fn is_complete(&self) -> bool {
        matches!(self, DependencyStatus::Complete)
    }

    /// Check if work is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self, DependencyStatus::InProgress { .. })
    }
}

impl GitHubClient {
    /// Get a pull request by number
    pub async fn get_pr(&self, number: u64) -> Result<PullRequest> {
        debug!(number, "Fetching pull request");

        let pr = self
            .client()
            .pulls(self.owner(), self.repo())
            .get(number)
            .await
            .map_err(|e| match &e {
                octocrab::Error::GitHub { source, .. }
                    if source.message.contains("Not Found") =>
                {
                    Error::PrNotFound(number)
                }
                _ => Error::Api(e),
            })?;

        Ok(pr.into())
    }

    /// List PRs with optional state filter
    pub async fn list_prs(&self, state: Option<PrState>) -> Result<Vec<PullRequest>> {
        debug!(?state, "Listing pull requests");

        let pulls_handler = self.client().pulls(self.owner(), self.repo());
        let mut builder = pulls_handler.list();

        if let Some(s) = state {
            builder = builder.state(match s {
                PrState::Open => octocrab::params::State::Open,
                PrState::Closed => octocrab::params::State::Closed,
            });
        }

        let prs = builder.send().await.map_err(Error::Api)?;
        let result: Vec<PullRequest> = prs.items.into_iter().map(PullRequest::from).collect();

        info!(count = result.len(), "Fetched pull requests");

        Ok(result)
    }

    /// Find PRs that reference an issue (via "Fixes #X", "Closes #X", etc.)
    pub async fn find_prs_for_issue(&self, issue_number: u64) -> Result<Vec<PullRequest>> {
        debug!(issue_number, "Finding PRs that reference issue");

        // Get all PRs (both open and closed)
        let all_prs = self.list_prs(None).await?;

        // Filter to those that reference this issue
        let patterns = [
            format!("fixes #{}", issue_number),
            format!("closes #{}", issue_number),
            format!("resolves #{}", issue_number),
            format!("fix #{}", issue_number),
            format!("close #{}", issue_number),
            format!("resolve #{}", issue_number),
        ];

        let matching: Vec<PullRequest> = all_prs
            .into_iter()
            .filter(|pr| {
                let body_lower = pr.body.to_lowercase();
                let title_lower = pr.title.to_lowercase();
                patterns
                    .iter()
                    .any(|p| body_lower.contains(p) || title_lower.contains(p))
            })
            .collect();

        info!(
            issue_number,
            count = matching.len(),
            "Found PRs for issue"
        );

        Ok(matching)
    }

    /// Check the dependency status of an issue
    ///
    /// Returns whether the issue's work is complete (PR merged or issue closed)
    pub async fn check_dependency_status(&self, issue_number: u64) -> Result<DependencyStatus> {
        debug!(issue_number, "Checking dependency status");

        // First, check if the issue is closed
        let issue = self.get_issue(issue_number).await?;

        if issue.state == IssueState::Closed {
            // Issue is closed - check if it was closed via a merged PR
            let prs = self.find_prs_for_issue(issue_number).await?;
            if prs.iter().any(|pr| pr.merged) {
                return Ok(DependencyStatus::Complete);
            }
            // Issue closed but no merged PR - still consider complete
            return Ok(DependencyStatus::Complete);
        }

        // Issue is open - check for linked PRs
        let prs = self.find_prs_for_issue(issue_number).await?;

        // Check if any PR is merged
        for pr in &prs {
            if pr.merged {
                return Ok(DependencyStatus::Complete);
            }
        }

        // Check if any PR is open (in progress)
        for pr in &prs {
            if pr.state == PrState::Open {
                return Ok(DependencyStatus::InProgress { pr_number: pr.number });
            }
        }

        // No linked PRs or all PRs closed without merge
        Ok(DependencyStatus::Pending)
    }

    /// Check if all dependencies for an issue are satisfied
    pub async fn are_dependencies_met(&self, issue: &Issue) -> Result<(bool, Vec<u64>)> {
        let deps = crate::IssueDependencies::parse(&issue.body);
        let mut unmet = Vec::new();

        for dep_ref in &deps.depends_on {
            if !dep_ref.is_local() {
                // Skip cross-repo dependencies for now
                continue;
            }

            let status = self.check_dependency_status(dep_ref.number).await?;
            if !status.is_complete() {
                unmet.push(dep_ref.number);
            }
        }

        for dep_ref in &deps.blocked_by {
            if !dep_ref.is_local() {
                continue;
            }

            let status = self.check_dependency_status(dep_ref.number).await?;
            if !status.is_complete() {
                unmet.push(dep_ref.number);
            }
        }

        Ok((unmet.is_empty(), unmet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_status_is_complete() {
        assert!(DependencyStatus::Complete.is_complete());
        assert!(!DependencyStatus::Pending.is_complete());
        assert!(!DependencyStatus::InProgress { pr_number: 1 }.is_complete());
    }

    #[test]
    fn test_dependency_status_is_in_progress() {
        assert!(!DependencyStatus::Complete.is_in_progress());
        assert!(!DependencyStatus::Pending.is_in_progress());
        assert!(DependencyStatus::InProgress { pr_number: 1 }.is_in_progress());
    }

    #[test]
    fn test_pr_state_conversion() {
        assert_eq!(
            PrState::from(octocrab::models::IssueState::Open),
            PrState::Open
        );
        assert_eq!(
            PrState::from(octocrab::models::IssueState::Closed),
            PrState::Closed
        );
    }
}
