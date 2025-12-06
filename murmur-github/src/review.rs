//! Pull request review management

use crate::{Error, GitHubClient, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A review comment on a pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    /// Comment ID
    pub id: u64,
    /// Comment body/text
    pub body: String,
    /// Author username
    pub author: String,
    /// File path (if this is a code review comment)
    pub path: Option<String>,
    /// Line number (if this is a code review comment)
    pub line: Option<u64>,
    /// When the comment was created
    pub created_at: DateTime<Utc>,
}

/// A pull request review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Review ID
    pub id: u64,
    /// Review state (APPROVED, CHANGES_REQUESTED, COMMENTED, etc.)
    pub state: String,
    /// Review body/summary
    pub body: Option<String>,
    /// Author username
    pub author: String,
    /// When the review was submitted
    pub submitted_at: Option<DateTime<Utc>>,
}

impl GitHubClient {
    /// Get all reviews for a pull request
    pub async fn get_pr_reviews(&self, pr_number: u64) -> Result<Vec<Review>> {
        let reviews = self
            .client()
            .pulls(self.owner(), self.repo())
            .list_reviews(pr_number)
            .send()
            .await
            .map_err(Error::Api)?;

        Ok(reviews
            .items
            .into_iter()
            .map(|r| Review {
                id: r.id.0,
                state: r
                    .state
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                body: r.body,
                author: r.user.map(|u| u.login).unwrap_or_default(),
                submitted_at: r.submitted_at,
            })
            .collect())
    }

    /// Get all review comments for a pull request
    pub async fn get_pr_review_comments(&self, pr_number: u64) -> Result<Vec<ReviewComment>> {
        let comments = self
            .client()
            .pulls(self.owner(), self.repo())
            .list_comments(Some(pr_number))
            .send()
            .await
            .map_err(Error::Api)?;

        Ok(comments
            .items
            .into_iter()
            .map(|c| ReviewComment {
                id: c.id.0,
                body: c.body,
                author: c.user.map(|u| u.login).unwrap_or_default(),
                path: Some(c.path),
                line: c.line,
                created_at: c.created_at,
            })
            .collect())
    }

    /// Check if a PR has pending review feedback that requires changes
    pub async fn has_pending_review_feedback(&self, pr_number: u64) -> Result<bool> {
        let reviews = self.get_pr_reviews(pr_number).await?;

        // Check if the most recent review requests changes
        if let Some(latest) = reviews.last() {
            return Ok(latest.state.contains("CHANGES_REQUESTED"));
        }

        Ok(false)
    }

    /// Build a feedback summary from PR reviews and comments
    pub async fn build_review_feedback_summary(&self, pr_number: u64) -> Result<String> {
        let reviews = self.get_pr_reviews(pr_number).await?;
        let comments = self.get_pr_review_comments(pr_number).await?;

        let mut summary = String::new();
        summary.push_str(&format!("# Review Feedback for PR #{}\n\n", pr_number));

        // Add reviews
        let has_reviews = !reviews.is_empty();
        if has_reviews {
            summary.push_str("## Reviews\n\n");
            for review in &reviews {
                summary.push_str(&format!("### {} by {}\n", review.state, review.author));
                if let Some(body) = &review.body {
                    if !body.is_empty() {
                        summary.push_str(&format!("{}\n\n", body));
                    }
                }
            }
        }

        // Add comments
        let has_comments = !comments.is_empty();
        if has_comments {
            summary.push_str("## Code Review Comments\n\n");
            for comment in &comments {
                if let Some(path) = &comment.path {
                    summary.push_str(&format!(
                        "### {} (line {})\n",
                        path,
                        comment.line.unwrap_or(0)
                    ));
                } else {
                    summary.push_str(&format!("### Comment by {}\n", comment.author));
                }
                summary.push_str(&format!("{}\n\n", comment.body));
            }
        }

        if !has_reviews && !has_comments {
            summary.push_str("No review feedback found.\n");
        }

        Ok(summary)
    }
}
