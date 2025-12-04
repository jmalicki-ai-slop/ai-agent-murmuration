//! Issue repository for CRUD operations on GitHub issues

use crate::error::{DbError, Result};
use crate::models::{
    CreateIssue, IssueState, IssueStatusHistory, UpdateIssue,
};
use chrono::Utc;
use sqlx::SqlitePool;

/// Repository for managing GitHub issue state
pub struct IssueRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> IssueRepository<'a> {
    /// Create a new issue repository
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new issue record
    pub async fn create(&self, issue: CreateIssue) -> Result<IssueState> {
        let now = Utc::now();
        let labels_json = issue.labels.map(|l| serde_json::to_string(&l)).transpose()?;
        let assignees_json = issue.assignees.map(|a| serde_json::to_string(&a)).transpose()?;

        let result = sqlx::query(
            r#"
            INSERT INTO issues (
                github_issue_number, repository, title, body, state,
                status, phase, pr_number, labels, assignees,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(issue.github_issue_number)
        .bind(&issue.repository)
        .bind(&issue.title)
        .bind(&issue.body)
        .bind(&issue.state)
        .bind(&issue.status)
        .bind(&issue.phase)
        .bind(&issue.pr_number)
        .bind(&labels_json)
        .bind(&assignees_json)
        .bind(now)
        .bind(now)
        .execute(self.pool)
        .await?;

        let id = result.last_insert_rowid();

        // Record initial status if provided
        if let Some(status) = &issue.status {
            self.record_status_change(id, None, status, Some("Initial status")).await?;
        }

        self.get_by_id(id).await
    }

    /// Get an issue by its internal ID
    pub async fn get_by_id(&self, id: i64) -> Result<IssueState> {
        sqlx::query_as::<_, IssueState>(
            "SELECT * FROM issues WHERE id = ?"
        )
        .bind(id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DbError::IssueNotFound("unknown".to_string(), id),
            e => e.into(),
        })
    }

    /// Get an issue by repository and GitHub issue number
    pub async fn get_by_number(&self, repository: &str, issue_number: i64) -> Result<IssueState> {
        sqlx::query_as::<_, IssueState>(
            "SELECT * FROM issues WHERE repository = ? AND github_issue_number = ?"
        )
        .bind(repository)
        .bind(issue_number)
        .fetch_one(self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DbError::IssueNotFound(repository.to_string(), issue_number),
            e => e.into(),
        })
    }

    /// List all issues for a repository
    pub async fn list_by_repository(&self, repository: &str) -> Result<Vec<IssueState>> {
        sqlx::query_as::<_, IssueState>(
            "SELECT * FROM issues WHERE repository = ? ORDER BY github_issue_number DESC"
        )
        .bind(repository)
        .fetch_all(self.pool)
        .await
        .map_err(Into::into)
    }

    /// List issues by state (open/closed)
    pub async fn list_by_state(&self, repository: &str, state: &str) -> Result<Vec<IssueState>> {
        sqlx::query_as::<_, IssueState>(
            "SELECT * FROM issues WHERE repository = ? AND state = ? ORDER BY github_issue_number DESC"
        )
        .bind(repository)
        .bind(state)
        .fetch_all(self.pool)
        .await
        .map_err(Into::into)
    }

    /// List issues by status
    pub async fn list_by_status(&self, repository: &str, status: &str) -> Result<Vec<IssueState>> {
        sqlx::query_as::<_, IssueState>(
            "SELECT * FROM issues WHERE repository = ? AND status = ? ORDER BY github_issue_number DESC"
        )
        .bind(repository)
        .bind(status)
        .fetch_all(self.pool)
        .await
        .map_err(Into::into)
    }

    /// Update an existing issue
    pub async fn update(&self, id: i64, update: UpdateIssue) -> Result<IssueState> {
        let now = Utc::now();

        // Fetch current state to check for status changes
        let current = self.get_by_id(id).await?;

        // Build dynamic update query
        let mut query = String::from("UPDATE issues SET updated_at = ?");
        let mut bindings: Vec<String> = vec![now.to_rfc3339()];

        if let Some(title) = &update.title {
            query.push_str(", title = ?");
            bindings.push(title.clone());
        }
        if let Some(body) = &update.body {
            query.push_str(", body = ?");
            bindings.push(body.clone());
        }
        if let Some(state) = &update.state {
            query.push_str(", state = ?");
            bindings.push(state.clone());
        }
        if let Some(status) = &update.status {
            query.push_str(", status = ?");
            bindings.push(status.clone());
        }
        if let Some(phase) = &update.phase {
            query.push_str(", phase = ?");
            bindings.push(phase.clone());
        }
        if let Some(pr_number) = &update.pr_number {
            query.push_str(", pr_number = ?");
            bindings.push(pr_number.clone());
        }
        if let Some(labels) = &update.labels {
            let labels_json = serde_json::to_string(labels)?;
            query.push_str(", labels = ?");
            bindings.push(labels_json);
        }
        if let Some(assignees) = &update.assignees {
            let assignees_json = serde_json::to_string(assignees)?;
            query.push_str(", assignees = ?");
            bindings.push(assignees_json);
        }
        if let Some(closed_at) = &update.closed_at {
            query.push_str(", closed_at = ?");
            bindings.push(closed_at.to_rfc3339());
        }

        query.push_str(" WHERE id = ?");
        bindings.push(id.to_string());

        // Execute update
        let mut q = sqlx::query(&query);
        for binding in bindings {
            q = q.bind(binding);
        }
        q.execute(self.pool).await?;

        // Record status change if status was updated
        if let Some(new_status) = &update.status {
            if current.status.as_ref() != Some(new_status) {
                self.record_status_change(
                    id,
                    current.status.as_deref(),
                    new_status,
                    Some("Status updated"),
                ).await?;
            }
        }

        self.get_by_id(id).await
    }

    /// Delete an issue
    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM issues WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Record a status change in history
    pub async fn record_status_change(
        &self,
        issue_id: i64,
        old_status: Option<&str>,
        new_status: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO issue_status_history (issue_id, old_status, new_status, changed_at, reason)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(issue_id)
        .bind(old_status)
        .bind(new_status)
        .bind(now)
        .bind(reason)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// Get status change history for an issue
    pub async fn get_status_history(&self, issue_id: i64) -> Result<Vec<IssueStatusHistory>> {
        sqlx::query_as::<_, IssueStatusHistory>(
            "SELECT * FROM issue_status_history WHERE issue_id = ? ORDER BY changed_at ASC"
        )
        .bind(issue_id)
        .fetch_all(self.pool)
        .await
        .map_err(Into::into)
    }

    /// Get issues linked to a specific agent run
    pub async fn get_by_agent_run(&self, agent_run_id: i64) -> Result<Option<IssueState>> {
        sqlx::query_as::<_, IssueState>(
            r#"
            SELECT i.* FROM issues i
            INNER JOIN agent_runs ar ON i.id = ar.issue_id
            WHERE ar.id = ?
            "#
        )
        .bind(agent_run_id)
        .fetch_optional(self.pool)
        .await
        .map_err(Into::into)
    }

    /// Count issues by status
    pub async fn count_by_status(&self, repository: &str) -> Result<Vec<(String, i64)>> {
        let rows = sqlx::query_as::<_, (Option<String>, i64)>(
            "SELECT status, COUNT(*) as count FROM issues WHERE repository = ? GROUP BY status"
        )
        .bind(repository)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter()
            .map(|(status, count)| (status.unwrap_or_else(|| "none".to_string()), count))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Database, DatabaseConfig};
    use tempfile::TempDir;

    async fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::new(&db_path);
        let db = Database::connect(config).await.unwrap();
        db.migrate().await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_create_and_get_issue() {
        let (db, _temp) = setup_test_db().await;
        let repo = IssueRepository::new(db.pool());

        let create = CreateIssue {
            github_issue_number: 42,
            repository: "owner/repo".to_string(),
            title: "Test Issue".to_string(),
            body: Some("Test body".to_string()),
            state: "open".to_string(),
            status: Some("ready".to_string()),
            phase: Some("3.5".to_string()),
            pr_number: None,
            labels: Some(vec!["bug".to_string()]),
            assignees: None,
        };

        let issue = repo.create(create).await.unwrap();
        assert_eq!(issue.github_issue_number, 42);
        assert_eq!(issue.title, "Test Issue");
        assert_eq!(issue.status, Some("ready".to_string()));

        let fetched = repo.get_by_number("owner/repo", 42).await.unwrap();
        assert_eq!(fetched.id, issue.id);
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        let (db, _temp) = setup_test_db().await;
        let repo = IssueRepository::new(db.pool());

        let create = CreateIssue {
            github_issue_number: 43,
            repository: "owner/repo".to_string(),
            title: "Test Issue".to_string(),
            body: None,
            state: "open".to_string(),
            status: Some("ready".to_string()),
            phase: None,
            pr_number: None,
            labels: None,
            assignees: None,
        };

        let issue = repo.create(create).await.unwrap();

        let update = UpdateIssue {
            status: Some("in_progress".to_string()),
            ..Default::default()
        };

        let updated = repo.update(issue.id, update).await.unwrap();
        assert_eq!(updated.status, Some("in_progress".to_string()));

        let history = repo.get_status_history(issue.id).await.unwrap();
        assert_eq!(history.len(), 2); // Initial + update
        assert_eq!(history[0].new_status, "ready");
        assert_eq!(history[1].new_status, "in_progress");
    }

    #[tokio::test]
    async fn test_list_by_status() {
        let (db, _temp) = setup_test_db().await;
        let repo = IssueRepository::new(db.pool());

        // Create multiple issues
        for i in 1..=3 {
            let create = CreateIssue {
                github_issue_number: i,
                repository: "owner/repo".to_string(),
                title: format!("Issue {}", i),
                body: None,
                state: "open".to_string(),
                status: Some(if i == 1 { "ready" } else { "blocked" }.to_string()),
                phase: None,
                pr_number: None,
                labels: None,
                assignees: None,
            };
            repo.create(create).await.unwrap();
        }

        let ready_issues = repo.list_by_status("owner/repo", "ready").await.unwrap();
        assert_eq!(ready_issues.len(), 1);

        let blocked_issues = repo.list_by_status("owner/repo", "blocked").await.unwrap();
        assert_eq!(blocked_issues.len(), 2);
    }

    #[tokio::test]
    async fn test_count_by_status() {
        let (db, _temp) = setup_test_db().await;
        let repo = IssueRepository::new(db.pool());

        // Create issues with different statuses
        for (i, status) in [("ready", 2), ("blocked", 3), ("complete", 1)].iter() {
            for j in 0..*status {
                let create = CreateIssue {
                    github_issue_number: (i.len() * 10 + j) as i64,
                    repository: "owner/repo".to_string(),
                    title: format!("{} {}", i, j),
                    body: None,
                    state: "open".to_string(),
                    status: Some(i.to_string()),
                    phase: None,
                    pr_number: None,
                    labels: None,
                    assignees: None,
                };
                repo.create(create).await.unwrap();
            }
        }

        let counts = repo.count_by_status("owner/repo").await.unwrap();
        assert_eq!(counts.len(), 3);

        let ready_count = counts.iter().find(|(s, _)| s == "ready").map(|(_, c)| *c);
        assert_eq!(ready_count, Some(2));
    }
}
