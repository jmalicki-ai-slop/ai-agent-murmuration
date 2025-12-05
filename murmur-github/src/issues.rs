//! Issue fetching and management

use crate::{Error, GitHubClient, Result};
use chrono::{DateTime, Utc};
use octocrab::models::issues::Issue as OctocrabIssue;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Issue state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

impl From<octocrab::models::IssueState> for IssueState {
    fn from(state: octocrab::models::IssueState) -> Self {
        match state {
            octocrab::models::IssueState::Open => IssueState::Open,
            octocrab::models::IssueState::Closed => IssueState::Closed,
            _ => IssueState::Open, // Default to open for unknown states
        }
    }
}

/// GitHub issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue number
    pub number: u64,
    /// Issue title
    pub title: String,
    /// Issue body/description
    pub body: String,
    /// Current state (open/closed)
    pub state: IssueState,
    /// Labels attached to the issue
    pub labels: Vec<String>,
    /// When the issue was created
    pub created_at: DateTime<Utc>,
    /// When the issue was last updated
    pub updated_at: DateTime<Utc>,
    /// Associated pull request URL (if this issue is a PR)
    pub pull_request_url: Option<String>,
    /// Issues tracked by this issue (from GitHub task lists)
    #[serde(default)]
    pub tracked_issues: Vec<u64>,
    /// Issues that track this issue
    #[serde(default)]
    pub tracked_in_issues: Vec<u64>,
    /// Summary of sub-issues (from GitHub task lists)
    #[serde(default)]
    pub sub_issues_summary: Option<SubIssuesSummary>,
}

/// Summary of sub-issue completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubIssuesSummary {
    /// Total number of sub-issues
    pub total: u32,
    /// Number of completed sub-issues
    pub completed: u32,
    /// Percentage completed (0-100)
    pub percent_completed: u32,
}

impl From<OctocrabIssue> for Issue {
    fn from(issue: OctocrabIssue) -> Self {
        Issue {
            number: issue.number,
            title: issue.title,
            body: issue.body.unwrap_or_default(),
            state: issue.state.into(),
            labels: issue.labels.into_iter().map(|l| l.name).collect(),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            pull_request_url: issue.pull_request.map(|pr| pr.url.to_string()),
            tracked_issues: vec![],
            tracked_in_issues: vec![],
            sub_issues_summary: None,
        }
    }
}

/// Filter options for listing issues
#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    /// Filter by state (default: open)
    pub state: Option<IssueState>,
    /// Filter by labels (all must match)
    pub labels: Vec<String>,
    /// Maximum number of issues to fetch (default: 100)
    pub per_page: Option<u8>,
}

impl GitHubClient {
    /// Fetch a single issue by number
    pub async fn get_issue(&self, number: u64) -> Result<Issue> {
        debug!(number, "Fetching issue");

        let issue = self
            .client()
            .issues(self.owner(), self.repo())
            .get(number)
            .await
            .map_err(|e| match &e {
                octocrab::Error::GitHub { source, .. } if source.message.contains("Not Found") => {
                    Error::IssueNotFound(number)
                }
                _ => Error::Api(e),
            })?;

        Ok(issue.into())
    }

    /// Fetch a single issue by number with tracked issues populated
    ///
    /// This uses GraphQL to fetch GitHub's native issue tracking data
    pub async fn get_issue_with_tracking(&self, number: u64) -> Result<Issue> {
        debug!(number, "Fetching issue with tracking info");

        // First get the basic issue data
        let mut issue = self.get_issue(number).await?;

        // Then fetch tracked issues via GraphQL
        let (tracked_issues, tracked_in_issues, sub_issues_summary) =
            self.get_tracked_issues(number).await?;

        issue.tracked_issues = tracked_issues;
        issue.tracked_in_issues = tracked_in_issues;
        issue.sub_issues_summary = sub_issues_summary;

        Ok(issue)
    }

    /// List issues with optional filters
    pub async fn list_issues(&self, filter: &IssueFilter) -> Result<Vec<Issue>> {
        debug!(?filter, "Listing issues");

        let issues_handler = self.client().issues(self.owner(), self.repo());
        let mut builder = issues_handler.list();

        // Apply state filter
        if let Some(state) = filter.state {
            builder = builder.state(match state {
                IssueState::Open => octocrab::params::State::Open,
                IssueState::Closed => octocrab::params::State::Closed,
            });
        }

        // Apply labels filter
        if !filter.labels.is_empty() {
            builder = builder.labels(&filter.labels);
        }

        // Apply per_page
        if let Some(per_page) = filter.per_page {
            builder = builder.per_page(per_page);
        }

        let issues = builder.send().await.map_err(Error::Api)?;

        let result: Vec<Issue> = issues.items.into_iter().map(Issue::from).collect();

        info!(count = result.len(), "Fetched issues");

        Ok(result)
    }

    /// List all issues (paginating through all pages)
    pub async fn list_all_issues(&self, filter: &IssueFilter) -> Result<Vec<Issue>> {
        debug!(?filter, "Listing all issues with pagination");

        let mut all_issues = Vec::new();
        let per_page = filter.per_page.unwrap_or(100);

        let mut page_num = 1u32;
        loop {
            let issues_handler = self.client().issues(self.owner(), self.repo());
            let mut builder = issues_handler.list().per_page(per_page).page(page_num);

            // Apply state filter
            if let Some(state) = filter.state {
                builder = builder.state(match state {
                    IssueState::Open => octocrab::params::State::Open,
                    IssueState::Closed => octocrab::params::State::Closed,
                });
            }

            // Apply labels filter
            if !filter.labels.is_empty() {
                builder = builder.labels(&filter.labels);
            }

            let issues = builder.send().await.map_err(Error::Api)?;
            let items: Vec<Issue> = issues.items.into_iter().map(Issue::from).collect();

            if items.is_empty() {
                break;
            }

            all_issues.extend(items);
            page_num += 1;
        }

        info!(count = all_issues.len(), "Fetched all issues");

        Ok(all_issues)
    }

    /// List open issues only
    pub async fn list_open_issues(&self) -> Result<Vec<Issue>> {
        self.list_issues(&IssueFilter {
            state: Some(IssueState::Open),
            ..Default::default()
        })
        .await
    }

    /// List issues with a specific label
    pub async fn list_issues_by_label(&self, label: &str) -> Result<Vec<Issue>> {
        self.list_issues(&IssueFilter {
            labels: vec![label.to_string()],
            ..Default::default()
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_state_conversion() {
        assert_eq!(
            IssueState::from(octocrab::models::IssueState::Open),
            IssueState::Open
        );
        assert_eq!(
            IssueState::from(octocrab::models::IssueState::Closed),
            IssueState::Closed
        );
    }

    #[test]
    fn test_issue_filter_default() {
        let filter = IssueFilter::default();
        assert!(filter.state.is_none());
        assert!(filter.labels.is_empty());
        assert!(filter.per_page.is_none());
    }
}
