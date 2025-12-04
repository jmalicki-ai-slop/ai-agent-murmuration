//! Agent run history repository

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::{Error, Result};

/// Status of an agent run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRunStatus {
    /// Agent is currently running
    Running,
    /// Agent completed successfully
    Completed,
    /// Agent failed with an error
    Failed,
    /// Agent was interrupted before completion
    Interrupted,
}

impl AgentRunStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "interrupted" => Self::Interrupted,
            _ => Self::Running,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Interrupted => "interrupted",
        }
    }
}

impl std::fmt::Display for AgentRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Agent run record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    /// Unique ID
    pub id: i64,
    /// Task identifier (e.g., "issue-42")
    pub task_id: String,
    /// Type of agent (e.g., "coder", "reviewer")
    pub agent_type: String,
    /// Path to the worktree
    pub worktree_path: String,
    /// When the run started
    pub started_at: DateTime<Utc>,
    /// When the run completed (None if still running or interrupted)
    pub completed_at: Option<DateTime<Utc>>,
    /// Current status
    pub status: AgentRunStatus,
    /// Exit code if completed
    pub exit_code: Option<i32>,
    /// Initial prompt
    pub prompt: String,
    /// When this record was created
    pub created_at: DateTime<Utc>,
}

/// Repository for agent runs
#[derive(Clone)]
pub struct AgentRunsRepo {
    pool: SqlitePool,
}

impl AgentRunsRepo {
    /// Create a new repository
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new agent run
    pub async fn create(
        &self,
        task_id: &str,
        agent_type: &str,
        worktree_path: &str,
        prompt: &str,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        let status = AgentRunStatus::Running.as_str();

        let result = sqlx::query(
            r#"
            INSERT INTO agent_runs (task_id, agent_type, worktree_path, started_at, status, prompt, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(task_id)
        .bind(agent_type)
        .bind(worktree_path)
        .bind(&now)
        .bind(status)
        .bind(prompt)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Mark a run as completed
    pub async fn mark_completed(&self, run_id: i64, exit_code: i32) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let status = if exit_code == 0 {
            AgentRunStatus::Completed
        } else {
            AgentRunStatus::Failed
        };

        sqlx::query(
            r#"
            UPDATE agent_runs
            SET completed_at = ?, status = ?, exit_code = ?
            WHERE id = ?
            "#
        )
        .bind(&now)
        .bind(status.as_str())
        .bind(exit_code)
        .bind(run_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark a run as interrupted
    pub async fn mark_interrupted(&self, run_id: i64) -> Result<()> {
        let status = AgentRunStatus::Interrupted;

        sqlx::query(
            r#"
            UPDATE agent_runs
            SET status = ?
            WHERE id = ?
            "#
        )
        .bind(status.as_str())
        .bind(run_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a run by ID
    pub async fn get(&self, run_id: i64) -> Result<AgentRun> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, agent_type, worktree_path, started_at, completed_at,
                   status, exit_code, prompt, created_at
            FROM agent_runs
            WHERE id = ?
            "#
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NotFound(format!("Agent run {} not found", run_id)))?;

        Ok(AgentRun {
            id: row.get("id"),
            task_id: row.get("task_id"),
            agent_type: row.get("agent_type"),
            worktree_path: row.get("worktree_path"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            status: AgentRunStatus::from_str(row.get("status")),
            exit_code: row.get("exit_code"),
            prompt: row.get("prompt"),
            created_at: row.get("created_at"),
        })
    }

    /// Find the most recent run for a task
    pub async fn find_latest_by_task(&self, task_id: &str) -> Result<Option<AgentRun>> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, agent_type, worktree_path, started_at, completed_at,
                   status, exit_code, prompt, created_at
            FROM agent_runs
            WHERE task_id = ?
            ORDER BY started_at DESC
            LIMIT 1
            "#
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| AgentRun {
            id: row.get("id"),
            task_id: row.get("task_id"),
            agent_type: row.get("agent_type"),
            worktree_path: row.get("worktree_path"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            status: AgentRunStatus::from_str(row.get("status")),
            exit_code: row.get("exit_code"),
            prompt: row.get("prompt"),
            created_at: row.get("created_at"),
        }))
    }

    /// Find interrupted runs
    pub async fn find_interrupted(&self) -> Result<Vec<AgentRun>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, agent_type, worktree_path, started_at, completed_at,
                   status, exit_code, prompt, created_at
            FROM agent_runs
            WHERE status = 'interrupted' OR status = 'running'
            ORDER BY started_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| AgentRun {
                id: row.get("id"),
                task_id: row.get("task_id"),
                agent_type: row.get("agent_type"),
                worktree_path: row.get("worktree_path"),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                status: AgentRunStatus::from_str(row.get("status")),
                exit_code: row.get("exit_code"),
                prompt: row.get("prompt"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    /// List all runs for a task
    pub async fn list_by_task(&self, task_id: &str) -> Result<Vec<AgentRun>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, agent_type, worktree_path, started_at, completed_at,
                   status, exit_code, prompt, created_at
            FROM agent_runs
            WHERE task_id = ?
            ORDER BY started_at DESC
            "#
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| AgentRun {
                id: row.get("id"),
                task_id: row.get("task_id"),
                agent_type: row.get("agent_type"),
                worktree_path: row.get("worktree_path"),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                status: AgentRunStatus::from_str(row.get("status")),
                exit_code: row.get("exit_code"),
                prompt: row.get("prompt"),
                created_at: row.get("created_at"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_and_get_run() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();
        let repo = db.agent_runs();

        let run_id = repo
            .create("issue-42", "coder", "/tmp/worktree", "Test prompt")
            .await
            .unwrap();

        let run = repo.get(run_id).await.unwrap();
        assert_eq!(run.task_id, "issue-42");
        assert_eq!(run.agent_type, "coder");
        assert_eq!(run.status, AgentRunStatus::Running);
    }

    #[tokio::test]
    async fn test_mark_completed() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();
        let repo = db.agent_runs();

        let run_id = repo
            .create("issue-42", "coder", "/tmp/worktree", "Test prompt")
            .await
            .unwrap();

        repo.mark_completed(run_id, 0).await.unwrap();

        let run = repo.get(run_id).await.unwrap();
        assert_eq!(run.status, AgentRunStatus::Completed);
        assert!(run.completed_at.is_some());
        assert_eq!(run.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_find_latest_by_task() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();
        let repo = db.agent_runs();

        repo.create("issue-42", "coder", "/tmp/worktree1", "First")
            .await
            .unwrap();
        let second_id = repo
            .create("issue-42", "coder", "/tmp/worktree2", "Second")
            .await
            .unwrap();

        let latest = repo.find_latest_by_task("issue-42").await.unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().id, second_id);
    }
}
