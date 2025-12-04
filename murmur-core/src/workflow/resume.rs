//! Resume interrupted workflows
//!
//! This module provides functionality to detect and resume interrupted agent sessions.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Resume information for an interrupted workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInfo {
    /// Agent run ID
    pub run_id: i64,
    /// Task identifier
    pub task_id: String,
    /// Agent type
    pub agent_type: String,
    /// Worktree path
    pub worktree_path: String,
    /// Original prompt
    pub prompt: String,
    /// Number of conversation messages logged
    pub message_count: usize,
}

impl ResumeInfo {
    /// Create a new resume info
    pub fn new(
        run_id: i64,
        task_id: impl Into<String>,
        agent_type: impl Into<String>,
        worktree_path: impl Into<String>,
        prompt: impl Into<String>,
        message_count: usize,
    ) -> Self {
        Self {
            run_id,
            task_id: task_id.into(),
            agent_type: agent_type.into(),
            worktree_path: worktree_path.into(),
            prompt: prompt.into(),
            message_count,
        }
    }

    /// Check if the worktree still exists
    pub fn worktree_exists(&self) -> bool {
        Path::new(&self.worktree_path).exists()
    }
}

/// Resume detection and management
#[cfg(feature = "database")]
pub struct ResumeManager {
    db: murmur_db::Database,
}

#[cfg(feature = "database")]
impl ResumeManager {
    /// Create a new resume manager
    pub async fn new() -> Result<Self, murmur_db::Error> {
        let db = murmur_db::Database::default().await?;
        Ok(Self { db })
    }

    /// Create a resume manager with a specific database
    pub fn with_database(db: murmur_db::Database) -> Self {
        Self { db }
    }

    /// Find interrupted runs for a specific task
    pub async fn find_interrupted_for_task(
        &self,
        task_id: &str,
    ) -> Result<Option<ResumeInfo>, murmur_db::Error> {
        use murmur_db::AgentRunStatus;

        let run = self.db.agent_runs().find_latest_by_task(task_id).await?;

        if let Some(run) = run {
            if run.status == AgentRunStatus::Interrupted
                || run.status == AgentRunStatus::Running
            {
                let message_count = self
                    .db
                    .conversation_logs()
                    .count_by_run(run.id)
                    .await? as usize;

                return Ok(Some(ResumeInfo::new(
                    run.id,
                    run.task_id,
                    run.agent_type,
                    run.worktree_path,
                    run.prompt,
                    message_count,
                )));
            }
        }

        Ok(None)
    }

    /// Find all interrupted runs
    pub async fn find_all_interrupted(&self) -> Result<Vec<ResumeInfo>, murmur_db::Error> {
        let runs = self.db.agent_runs().find_interrupted().await?;

        let mut results = Vec::new();
        for run in runs {
            let message_count = self
                .db
                .conversation_logs()
                .count_by_run(run.id)
                .await? as usize;

            results.push(ResumeInfo::new(
                run.id,
                run.task_id,
                run.agent_type,
                run.worktree_path,
                run.prompt,
                message_count,
            ));
        }

        Ok(results)
    }

    /// Get conversation history for a run
    pub async fn get_conversation_history(
        &self,
        run_id: i64,
    ) -> Result<Vec<murmur_db::ConversationLog>, murmur_db::Error> {
        self.db.conversation_logs().get_by_run(run_id).await
    }

    /// Mark a run as interrupted
    pub async fn mark_interrupted(&self, run_id: i64) -> Result<(), murmur_db::Error> {
        self.db.agent_runs().mark_interrupted(run_id).await
    }

    /// Create a new agent run
    pub async fn create_run(
        &self,
        task_id: &str,
        agent_type: &str,
        worktree_path: &str,
        prompt: &str,
    ) -> Result<i64, murmur_db::Error> {
        self.db
            .agent_runs()
            .create(task_id, agent_type, worktree_path, prompt)
            .await
    }

    /// Log a conversation message
    pub async fn log_message(
        &self,
        run_id: i64,
        sequence: i64,
        message_type: &str,
        content: &str,
    ) -> Result<i64, murmur_db::Error> {
        self.db
            .conversation_logs()
            .append(run_id, sequence, message_type, content)
            .await
    }

    /// Complete a run
    pub async fn complete_run(
        &self,
        run_id: i64,
        exit_code: i32,
    ) -> Result<(), murmur_db::Error> {
        self.db.agent_runs().mark_completed(run_id, exit_code).await
    }
}

#[cfg(test)]
#[cfg(feature = "database")]
mod tests {
    use super::*;
    use murmur_db::Database;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_find_interrupted_for_task() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();

        let manager = ResumeManager::with_database(db.clone());

        let run_id = manager
            .create_run("issue-42", "coder", "/tmp/worktree", "Test prompt")
            .await
            .unwrap();

        manager.log_message(run_id, 0, "system", "{}").await.unwrap();
        manager.mark_interrupted(run_id).await.unwrap();

        let resume_info = manager
            .find_interrupted_for_task("issue-42")
            .await
            .unwrap();

        assert!(resume_info.is_some());
        let info = resume_info.unwrap();
        assert_eq!(info.task_id, "issue-42");
        assert_eq!(info.message_count, 1);
    }

    #[tokio::test]
    async fn test_find_all_interrupted() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::new(temp_dir.path().join("test.db"))
            .await
            .unwrap();

        let manager = ResumeManager::with_database(db.clone());

        let run1 = manager
            .create_run("issue-1", "coder", "/tmp/wt1", "Test 1")
            .await
            .unwrap();
        let run2 = manager
            .create_run("issue-2", "coder", "/tmp/wt2", "Test 2")
            .await
            .unwrap();

        manager.mark_interrupted(run1).await.unwrap();
        manager.mark_interrupted(run2).await.unwrap();

        let interrupted = manager.find_all_interrupted().await.unwrap();
        assert_eq!(interrupted.len(), 2);
    }
}
