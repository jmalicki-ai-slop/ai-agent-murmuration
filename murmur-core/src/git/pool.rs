//! Worktree pool/cache management
//!
//! Manages a cache of worktrees for reuse, with LRU eviction.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::worktree::default_cache_dir;
use crate::{Error, Result};

/// Metadata file name stored in each worktree
const METADATA_FILE: &str = ".murmur-worktree.toml";

/// Status of a cached worktree
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeStatus {
    /// Actively being used by an agent
    Active,
    /// Work completed successfully
    Completed,
    /// Work was abandoned or failed
    Abandoned,
    /// Ready to be reused
    #[default]
    Available,
}

/// Metadata stored with each cached worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeMetadata {
    /// Identifier for the task (e.g., issue number)
    pub task_id: String,

    /// When the worktree was created
    #[serde(with = "humantime_serde")]
    pub created_at: SystemTime,

    /// When the worktree was last used
    #[serde(with = "humantime_serde")]
    pub last_used: SystemTime,

    /// Base commit SHA at creation
    pub base_commit: String,

    /// Current status
    pub status: WorktreeStatus,

    /// Branch name
    pub branch: String,
}

impl WorktreeMetadata {
    /// Create new metadata for a fresh worktree
    pub fn new(
        task_id: impl Into<String>,
        base_commit: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            task_id: task_id.into(),
            created_at: now,
            last_used: now,
            base_commit: base_commit.into(),
            status: WorktreeStatus::Active,
            branch: branch.into(),
        }
    }

    /// Update last used timestamp
    pub fn touch(&mut self) {
        self.last_used = SystemTime::now();
    }

    /// Load metadata from a worktree directory
    pub fn load(worktree_path: &Path) -> Result<Self> {
        let meta_path = worktree_path.join(METADATA_FILE);
        let contents = fs::read_to_string(&meta_path)
            .map_err(|e| Error::Config(format!("Failed to read worktree metadata: {}", e)))?;

        toml::from_str(&contents)
            .map_err(|e| Error::Config(format!("Failed to parse worktree metadata: {}", e)))
    }

    /// Save metadata to a worktree directory
    pub fn save(&self, worktree_path: &Path) -> Result<()> {
        let meta_path = worktree_path.join(METADATA_FILE);
        let contents = toml::to_string_pretty(self)
            .map_err(|e| Error::Other(format!("Failed to serialize worktree metadata: {}", e)))?;

        fs::write(&meta_path, contents)
            .map_err(|e| Error::Other(format!("Failed to write worktree metadata: {}", e)))
    }
}

/// Worktree pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of worktrees to cache per repo
    pub max_per_repo: usize,

    /// Maximum total size in bytes (0 = unlimited)
    pub max_total_size: u64,

    /// Maximum age in seconds before eviction (0 = unlimited)
    pub max_age_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_per_repo: 10,
            max_total_size: 0,           // Unlimited by default
            max_age_secs: 7 * 24 * 3600, // 7 days
        }
    }
}

/// Information about a cached worktree
#[derive(Debug, Clone)]
pub struct CachedWorktree {
    /// Path to the worktree
    pub path: PathBuf,
    /// Metadata if available
    pub metadata: Option<WorktreeMetadata>,
}

/// Worktree pool manager
#[derive(Debug)]
pub struct WorktreePool {
    /// Base cache directory
    cache_dir: PathBuf,
    /// Configuration
    config: PoolConfig,
}

impl WorktreePool {
    /// Create a new pool with default cache directory
    pub fn new() -> Result<Self> {
        Ok(Self::with_cache_dir(default_cache_dir()?))
    }

    /// Create a pool with a specific cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            config: PoolConfig::default(),
        }
    }

    /// Create a pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Result<Self> {
        Ok(Self {
            cache_dir: default_cache_dir()?,
            config,
        })
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// List all cached worktrees for a repository
    pub fn list_worktrees(&self, repo_name: &str) -> Result<Vec<CachedWorktree>> {
        let repo_dir = self.cache_dir.join(repo_name);

        if !repo_dir.exists() {
            return Ok(Vec::new());
        }

        let mut worktrees = Vec::new();

        for entry in fs::read_dir(&repo_dir)
            .map_err(|e| Error::Other(format!("Failed to read cache directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| Error::Other(format!("Failed to read directory entry: {}", e)))?;

            let path = entry.path();
            if path.is_dir() {
                let metadata = WorktreeMetadata::load(&path).ok();
                worktrees.push(CachedWorktree { path, metadata });
            }
        }

        Ok(worktrees)
    }

    /// Find an available worktree for reuse
    pub fn find_available(&self, repo_name: &str, task_id: &str) -> Result<Option<CachedWorktree>> {
        let worktrees = self.list_worktrees(repo_name)?;

        // First, look for an exact match by task_id
        for wt in &worktrees {
            if let Some(ref meta) = wt.metadata {
                if meta.task_id == task_id && meta.status == WorktreeStatus::Available {
                    return Ok(Some(wt.clone()));
                }
            }
        }

        // Then, look for any available worktree
        for wt in worktrees {
            if let Some(ref meta) = wt.metadata {
                if meta.status == WorktreeStatus::Available {
                    return Ok(Some(wt));
                }
            }
        }

        Ok(None)
    }

    /// Check if a worktree is dirty (has uncommitted changes)
    pub fn is_dirty(&self, worktree_path: &Path) -> Result<bool> {
        let output = std::process::Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(worktree_path)
            .output()
            .map_err(|e| Error::Other(format!("Failed to check git status: {}", e)))?;

        if !output.status.success() {
            // If git status fails, assume dirty
            return Ok(true);
        }

        // If output is non-empty, there are changes
        Ok(!output.stdout.is_empty())
    }

    /// Clean up old worktrees based on configuration
    pub fn cleanup(&self, repo_name: &str) -> Result<Vec<PathBuf>> {
        let mut removed = Vec::new();
        let worktrees = self.list_worktrees(repo_name)?;

        let now = SystemTime::now();
        let max_age = std::time::Duration::from_secs(self.config.max_age_secs);

        for wt in worktrees {
            let should_remove = if let Some(ref meta) = wt.metadata {
                // Check if too old
                self.config.max_age_secs > 0
                    && now
                        .duration_since(meta.last_used)
                        .is_ok_and(|age| age > max_age && meta.status != WorktreeStatus::Active)
            } else {
                // No metadata, consider for removal if old
                if let Ok(modified) = fs::metadata(&wt.path).and_then(|m| m.modified()) {
                    if let Ok(age) = now.duration_since(modified) {
                        age > max_age
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if should_remove {
                if let Err(e) = fs::remove_dir_all(&wt.path) {
                    tracing::warn!("Failed to remove old worktree {:?}: {}", wt.path, e);
                } else {
                    removed.push(wt.path);
                }
            }
        }

        // Enforce max_per_repo limit
        let remaining = self.list_worktrees(repo_name)?;
        if remaining.len() > self.config.max_per_repo {
            // Sort by last_used (oldest first)
            let mut sorted: Vec<_> = remaining
                .into_iter()
                .filter(|wt| {
                    wt.metadata
                        .as_ref()
                        .map(|m| m.status != WorktreeStatus::Active)
                        .unwrap_or(true)
                })
                .collect();

            sorted.sort_by(|a, b| {
                let a_time = a
                    .metadata
                    .as_ref()
                    .map(|m| m.last_used)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let b_time = b
                    .metadata
                    .as_ref()
                    .map(|m| m.last_used)
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            });

            // Remove oldest until under limit
            while sorted.len() > self.config.max_per_repo {
                if let Some(wt) = sorted.first() {
                    if let Err(e) = fs::remove_dir_all(&wt.path) {
                        tracing::warn!("Failed to remove excess worktree {:?}: {}", wt.path, e);
                    } else {
                        removed.push(wt.path.clone());
                    }
                }
                sorted.remove(0);
            }
        }

        Ok(removed)
    }
}

impl Default for WorktreePool {
    fn default() -> Self {
        Self::new().expect("Failed to create default worktree pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_metadata_serialization() {
        let meta = WorktreeMetadata::new("42", "abc123", "murmur/42-test");
        let toml = toml::to_string(&meta).unwrap();
        assert!(toml.contains("task_id = \"42\""));
        assert!(toml.contains("base_commit = \"abc123\""));
    }

    #[test]
    fn test_pool_with_temp_dir() {
        let temp = TempDir::new().unwrap();
        let pool = WorktreePool::with_cache_dir(temp.path().to_path_buf());

        let worktrees = pool.list_worktrees("test-repo").unwrap();
        assert!(worktrees.is_empty());
    }
}
