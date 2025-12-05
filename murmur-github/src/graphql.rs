//! GitHub GraphQL API support for features not available in REST API

use crate::{Error, GitHubClient, Result, SubIssuesSummary};
use serde::Deserialize;
use serde_json::json;
use tracing::debug;

/// GraphQL query response wrapper
#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphQLError {
    message: String,
    #[serde(default)]
    path: Vec<String>,
}

/// Issue with tracked issues (GraphQL)
#[derive(Debug, Deserialize)]
struct IssueTracking {
    repository: Option<RepositoryData>,
}

#[derive(Debug, Deserialize)]
struct RepositoryData {
    issue: Option<IssueData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct IssueData {
    number: u64,
    tracked_issues: IssueConnection,
    tracked_in_issues: IssueConnection,
    sub_issues_summary: SubIssuesSummaryData,
}

#[derive(Debug, Deserialize)]
struct IssueConnection {
    nodes: Vec<IssueNode>,
}

#[derive(Debug, Deserialize)]
struct IssueNode {
    number: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubIssuesSummaryData {
    total: u32,
    completed: u32,
    percent_completed: u32,
}

impl From<SubIssuesSummaryData> for SubIssuesSummary {
    fn from(data: SubIssuesSummaryData) -> Self {
        SubIssuesSummary {
            total: data.total,
            completed: data.completed,
            percent_completed: data.percent_completed,
        }
    }
}

impl GitHubClient {
    /// Fetch tracked issues for a given issue using GraphQL
    ///
    /// Returns (tracked_issues, tracked_in_issues, sub_issues_summary)
    pub async fn get_tracked_issues(
        &self,
        issue_number: u64,
    ) -> Result<(Vec<u64>, Vec<u64>, Option<SubIssuesSummary>)> {
        debug!(issue_number, "Fetching tracked issues via GraphQL");

        let query = r#"
            query($owner: String!, $repo: String!, $number: Int!) {
                repository(owner: $owner, name: $repo) {
                    issue(number: $number) {
                        number
                        trackedIssues(first: 100) {
                            nodes {
                                number
                            }
                        }
                        trackedInIssues(first: 100) {
                            nodes {
                                number
                            }
                        }
                        subIssuesSummary {
                            total
                            completed
                            percentCompleted
                        }
                    }
                }
            }
        "#;

        let variables = json!({
            "owner": self.owner(),
            "repo": self.repo(),
            "number": issue_number,
        });

        let response = self
            .graphql_query::<IssueTracking>(query, &variables)
            .await?;

        let issue_data = response
            .repository
            .and_then(|r| r.issue)
            .ok_or_else(|| Error::IssueNotFound(issue_number))?;

        let tracked_issues: Vec<u64> = issue_data
            .tracked_issues
            .nodes
            .into_iter()
            .map(|n| n.number)
            .collect();

        let tracked_in_issues: Vec<u64> = issue_data
            .tracked_in_issues
            .nodes
            .into_iter()
            .map(|n| n.number)
            .collect();

        let sub_issues_summary = if issue_data.sub_issues_summary.total > 0 {
            Some(issue_data.sub_issues_summary.into())
        } else {
            None
        };

        Ok((tracked_issues, tracked_in_issues, sub_issues_summary))
    }

    /// Execute a GraphQL query
    async fn graphql_query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: &serde_json::Value,
    ) -> Result<T> {
        use murmur_core::Secrets;

        let url = "https://api.github.com/graphql";

        let request_body = json!({
            "query": query,
            "variables": variables,
        });

        // Get GitHub token for authorization
        let secrets = Secrets::load().map_err(|e| Error::Auth(e.to_string()))?;
        let token = secrets
            .github_token()
            .ok_or_else(|| Error::Auth("GitHub token not found".to_string()))?;

        // Create a reqwest client and make the request
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "murmur-github")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Other(format!("GraphQL request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response".to_string());
            return Err(Error::Other(format!(
                "GraphQL request failed with status {}: {}",
                status, text
            )));
        }

        let graphql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| Error::Parse(format!("Failed to parse GraphQL response: {}", e)))?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            return Err(Error::Other(format!(
                "GraphQL errors: {}",
                error_messages.join(", ")
            )));
        }

        graphql_response
            .data
            .ok_or_else(|| Error::Other("GraphQL response missing data".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_issues_summary_conversion() {
        let data = SubIssuesSummaryData {
            total: 10,
            completed: 5,
            percent_completed: 50,
        };
        let summary: SubIssuesSummary = data.into();
        assert_eq!(summary.total, 10);
        assert_eq!(summary.completed, 5);
        assert_eq!(summary.percent_completed, 50);
    }
}
