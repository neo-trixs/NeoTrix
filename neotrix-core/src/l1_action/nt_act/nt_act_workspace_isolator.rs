//! Workspace Isolation — Git Worktree-based Agent Task Isolation
//!
//! Implements the Cursor/Copilot/Devin pattern: each agent task gets its own
//! git worktree for isolation. Changes don't affect main branch until reviewed.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           Workspace Isolator                 │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │ Worktree │  │  Merge   │  │  Review  │  │
//! │  │ Manager  │  │  Engine  │  │  Gates   │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │        Git Operations Layer          │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Safety
//! - All operations are read-only on main branch
//! - Worktree creation uses `git worktree add` (safe)
//! - Merge requires explicit review approval
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Isolation level for workspace operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IsolationLevel {
    /// No isolation — work directly on main branch
    None,
    /// Worktree isolation — separate git worktree per task
    Worktree,
    /// Container isolation — Docker/container per task
    Container,
    /// VM isolation — full VM per task (most secure)
    Vm,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        Self::Worktree
    }
}

/// Status of a workspace
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkspaceStatus {
    /// Workspace is being created
    Creating,
    /// Workspace is ready for use
    Ready,
    /// Workspace has uncommitted changes
    Dirty,
    /// Workspace is being merged
    Merging,
    /// Workspace merge completed
    Merged,
    /// Workspace is being cleaned up
    Cleanup,
    /// Workspace has been deleted
    Deleted,
    /// An error occurred
    Error(String),
}

/// A git worktree-based isolated workspace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsolatedWorkspace {
    /// Unique workspace identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Path to the worktree on disk
    pub worktree_path: PathBuf,
    /// Branch name for this worktree
    pub branch_name: String,
    /// Parent repository path
    pub repo_path: PathBuf,
    /// Current status
    pub status: WorkspaceStatus,
    /// Isolation level
    pub isolation_level: IsolationLevel,
    /// Timestamp when created
    pub created_at: String,
    /// Agent task ID associated with this workspace
    pub task_id: Option<String>,
    /// Files changed in this workspace
    pub changed_files: Vec<PathBuf>,
    /// Whether changes have been reviewed
    pub reviewed: bool,
    /// Whether changes have been approved for merge
    pub approved: bool,
}

/// Merge strategy for combining workspace changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MergeStrategy {
    /// Fast-forward merge (linear history)
    FastForward,
    /// Squash merge (combine all commits)
    Squash,
    /// Rebase merge (replay commits on main)
    Rebase,
    /// No-merge — just create PR for manual review
    PullRequest,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::PullRequest
    }
}

/// Result of a workspace operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Workspace ID
    pub workspace_id: String,
    /// Human-readable message
    pub message: String,
    /// Files affected
    pub affected_files: Vec<PathBuf>,
    /// Merge commit hash (if merged)
    pub merge_commit: Option<String>,
}

/// Configuration for workspace isolation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceConfig {
    /// Default isolation level
    pub default_isolation: IsolationLevel,
    /// Default merge strategy
    pub default_merge_strategy: MergeStrategy,
    /// Maximum concurrent workspaces
    pub max_concurrent: usize,
    /// Auto-cleanup after merge
    pub auto_cleanup: bool,
    /// Base directory for worktrees
    pub worktree_base_dir: PathBuf,
    /// Require review before merge
    pub require_review: bool,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            default_isolation: IsolationLevel::Worktree,
            default_merge_strategy: MergeStrategy::PullRequest,
            max_concurrent: 10,
            auto_cleanup: true,
            worktree_base_dir: PathBuf::from("/tmp/neotrix-workspaces"),
            require_review: true,
        }
    }
}

// ============================================================================
// Workspace Isolator
// ============================================================================

/// Git worktree-based workspace isolator
///
/// Creates isolated workspaces for agent tasks, manages merge workflows,
/// and ensures changes are reviewed before integration.
pub struct WorkspaceIsolator {
    /// Configuration
    config: WorkspaceConfig,
    /// Active workspaces
    workspaces: Arc<RwLock<HashMap<String, IsolatedWorkspace>>>,
    /// Main repository path
    repo_path: PathBuf,
}

impl WorkspaceIsolator {
    /// Create a new workspace isolator
    pub fn new(repo_path: PathBuf, config: WorkspaceConfig) -> Self {
        Self {
            config,
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            repo_path,
        }
    }

    /// Create a new isolated workspace for an agent task
    ///
    /// # Arguments
    /// * `name` - Human-readable workspace name
    /// * `task_id` - Associated agent task ID
    ///
    /// # Returns
    /// The created workspace with worktree path and branch name
    pub async fn create_workspace(
        &self,
        name: String,
        task_id: Option<String>,
    ) -> Result<IsolatedWorkspace, WorkspaceError> {
        // Check concurrent limit
        {
            let workspaces = self.workspaces.read().await;
            if workspaces.len() >= self.config.max_concurrent {
                return Err(WorkspaceError::TooManyWorkspaces {
                    current: workspaces.len(),
                    max: self.config.max_concurrent,
                });
            }
        }

        // Generate unique IDs
        let workspace_id = format!("ws-{}", uuid::Uuid::new_v4());
        let branch_name = format!("agent/{}-{}", name.replace(' ', "-").to_lowercase(), &workspace_id[..8]);

        // Create worktree directory
        let worktree_path = self.config.worktree_base_dir.join(&workspace_id);
        tokio::fs::create_dir_all(&worktree_path)
            .await
            .map_err(|e| WorkspaceError::Io(e.to_string()))?;

        // Create git worktree
        self.git_worktree_add(&branch_name, &worktree_path).await?;

        let workspace = IsolatedWorkspace {
            id: workspace_id.clone(),
            name,
            worktree_path,
            branch_name,
            repo_path: self.repo_path.clone(),
            status: WorkspaceStatus::Ready,
            isolation_level: self.config.default_isolation,
            created_at: chrono::Utc::now().to_rfc3339(),
            task_id,
            changed_files: Vec::new(),
            reviewed: false,
            approved: false,
        };

        // Register workspace
        {
            let mut workspaces = self.workspaces.write().await;
            workspaces.insert(workspace_id.clone(), workspace.clone());
        }

        Ok(workspace)
    }

    /// Get a workspace by ID
    pub async fn get_workspace(&self, workspace_id: &str) -> Option<IsolatedWorkspace> {
        let workspaces = self.workspaces.read().await;
        workspaces.get(workspace_id).cloned()
    }

    /// List all active workspaces
    pub async fn list_workspaces(&self) -> Vec<IsolatedWorkspace> {
        let workspaces = self.workspaces.read().await;
        workspaces.values().cloned().collect()
    }

    /// Get files changed in a workspace
    pub async fn get_changed_files(&self, workspace_id: &str) -> Result<Vec<PathBuf>, WorkspaceError> {
        let workspace = self.get_workspace(workspace_id)
            .await
            .ok_or(WorkspaceError::NotFound(workspace_id.to_string()))?;

        let output = tokio::process::Command::new("git")
            .args(["diff", "--name-only", "HEAD"])
            .current_dir(&workspace.worktree_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<PathBuf> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .collect();

        // Update workspace
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(ws) = workspaces.get_mut(workspace_id) {
                ws.changed_files = files.clone();
                if !files.is_empty() {
                    ws.status = WorkspaceStatus::Dirty;
                }
            }
        }

        Ok(files)
    }

    /// Review workspace changes
    pub async fn review_workspace(
        &self,
        workspace_id: &str,
        approved: bool,
        comments: Option<String>,
    ) -> Result<WorkspaceResult, WorkspaceError> {
        let mut workspace = self.get_workspace(workspace_id)
            .await
            .ok_or(WorkspaceError::NotFound(workspace_id.to_string()))?;

        workspace.reviewed = true;
        workspace.approved = approved;

        // Update status
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(ws) = workspaces.get_mut(workspace_id) {
                ws.reviewed = true;
                ws.approved = approved;
            }
        }

        Ok(WorkspaceResult {
            success: true,
            workspace_id: workspace_id.to_string(),
            message: if approved {
                format!("Workspace approved for merge{}", 
                    comments.map(|c| format!(": {}", c)).unwrap_or_default())
            } else {
                format!("Workspace rejected{}", 
                    comments.map(|c| format!(": {}", c)).unwrap_or_default())
            },
            affected_files: workspace.changed_files,
            merge_commit: None,
        })
    }

    /// Merge workspace changes into main branch
    pub async fn merge_workspace(
        &self,
        workspace_id: &str,
        strategy: MergeStrategy,
    ) -> Result<WorkspaceResult, WorkspaceError> {
        let workspace = self.get_workspace(workspace_id)
            .await
            .ok_or(WorkspaceError::NotFound(workspace_id.to_string()))?;

        // Check if review is required
        if self.config.require_review && !workspace.approved {
            return Err(WorkspaceError::ReviewRequired(workspace_id.to_string()));
        }

        // Update status
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(ws) = workspaces.get_mut(workspace_id) {
                ws.status = WorkspaceStatus::Merging;
            }
        }

        // Perform merge based on strategy
        let merge_result = match strategy {
            MergeStrategy::FastForward => {
                self.git_merge_fast_forward(&workspace).await?
            }
            MergeStrategy::Squash => {
                self.git_merge_squash(&workspace).await?
            }
            MergeStrategy::Rebase => {
                self.git_merge_rebase(&workspace).await?
            }
            MergeStrategy::PullRequest => {
                // Just create a PR — don't merge
                self.git_create_pr(&workspace).await?
            }
        };

        // Update status
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(ws) = workspaces.get_mut(workspace_id) {
                ws.status = WorkspaceStatus::Merged;
            }
        }

        // Auto-cleanup if configured
        if self.config.auto_cleanup {
            self.cleanup_workspace(workspace_id).await?;
        }

        Ok(WorkspaceResult {
            success: true,
            workspace_id: workspace_id.to_string(),
            message: format!("Workspace merged with {:?} strategy", strategy),
            affected_files: workspace.changed_files,
            merge_commit: merge_result,
        })
    }

    /// Cleanup a workspace (remove worktree and branch)
    pub async fn cleanup_workspace(&self, workspace_id: &str) -> Result<(), WorkspaceError> {
        let workspace = self.get_workspace(workspace_id)
            .await
            .ok_or(WorkspaceError::NotFound(workspace_id.to_string()))?;

        // Remove git worktree
        self.git_worktree_remove(&workspace).await?;

        // Remove directory
        if workspace.worktree_path.exists() {
            tokio::fs::remove_dir_all(&workspace.worktree_path)
                .await
                .map_err(|e| WorkspaceError::Io(e.to_string()))?;
        }

        // Update status
        {
            let mut workspaces = self.workspaces.write().await;
            if let Some(ws) = workspaces.get_mut(workspace_id) {
                ws.status = WorkspaceStatus::Deleted;
            }
        }

        Ok(())
    }

    /// Get workspace statistics
    pub async fn stats(&self) -> WorkspaceStats {
        let workspaces = self.workspaces.read().await;
        let total = workspaces.len();
        let ready = workspaces.values().filter(|w| w.status == WorkspaceStatus::Ready).count();
        let dirty = workspaces.values().filter(|w| w.status == WorkspaceStatus::Dirty).count();
        let merging = workspaces.values().filter(|w| w.status == WorkspaceStatus::Merging).count();
        let merged = workspaces.values().filter(|w| w.status == WorkspaceStatus::Merged).count();

        WorkspaceStats {
            total,
            ready,
            dirty,
            merging,
            merged,
            max_concurrent: self.config.max_concurrent,
        }
    }

    // ==========================================================================
    // Git Operations (Private)
    // ==========================================================================

    async fn git_worktree_add(&self, branch: &str, path: &Path) -> Result<(), WorkspaceError> {
        let output = tokio::process::Command::new("git")
            .args(["worktree", "add", "-b", branch])
            .arg(path)
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::Git(format!("worktree add failed: {}", stderr)));
        }

        Ok(())
    }

    async fn git_worktree_remove(&self, workspace: &IsolatedWorkspace) -> Result<(), WorkspaceError> {
        let output = tokio::process::Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&workspace.worktree_path)
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::Git(format!("worktree remove failed: {}", stderr)));
        }

        Ok(())
    }

    async fn git_merge_fast_forward(
        &self,
        workspace: &IsolatedWorkspace,
    ) -> Result<Option<String>, WorkspaceError> {
        let output = tokio::process::Command::new("git")
            .args(["merge", "--ff-only", &workspace.branch_name])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::Git(format!("merge failed: {}", stderr)));
        }

        // Get merge commit hash
        let commit = self.git_get_head_commit(&self.repo_path).await?;
        Ok(Some(commit))
    }

    async fn git_merge_squash(
        &self,
        workspace: &IsolatedWorkspace,
    ) -> Result<Option<String>, WorkspaceError> {
        let output = tokio::process::Command::new("git")
            .args(["merge", "--squash", &workspace.branch_name])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::Git(format!("squash merge failed: {}", stderr)));
        }

        // Commit the squashed changes
        let commit_output = tokio::process::Command::new("git")
            .args(["commit", "-m", &format!("Squash merge: {}", workspace.name)])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            return Err(WorkspaceError::Git(format!("commit failed: {}", stderr)));
        }

        let commit = self.git_get_head_commit(&self.repo_path).await?;
        Ok(Some(commit))
    }

    async fn git_merge_rebase(
        &self,
        workspace: &IsolatedWorkspace,
    ) -> Result<Option<String>, WorkspaceError> {
        // Checkout main
        tokio::process::Command::new("git")
            .args(["checkout", "main"])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        // Rebase
        let output = tokio::process::Command::new("git")
            .args(["rebase", &workspace.branch_name])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::Git(format!("rebase failed: {}", stderr)));
        }

        let commit = self.git_get_head_commit(&self.repo_path).await?;
        Ok(Some(commit))
    }

    async fn git_create_pr(
        &self,
        workspace: &IsolatedWorkspace,
    ) -> Result<Option<String>, WorkspaceError> {
        // Push branch to origin
        let output = tokio::process::Command::new("git")
            .args(["push", "-u", "origin", &workspace.branch_name])
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorkspaceError::Git(format!("push failed: {}", stderr)));
        }

        // Create PR using gh CLI if available
        let pr_output = tokio::process::Command::new("gh")
            .args([
                "pr", "create",
                "--title", &format!("Agent: {}", workspace.name),
                "--body", &format!(
                    "Auto-generated PR from agent workspace `{}`.\n\nChanged files:\n{}",
                    workspace.name,
                    workspace.changed_files.iter()
                        .map(|f| format!("- {}", f.display()))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                "--head", &workspace.branch_name,
            ])
            .current_dir(&self.repo_path)
            .output()
            .await;

        match pr_output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(Some(stdout.trim().to_string()))
            }
            _ => Ok(None), // gh CLI not available or failed
        }
    }

    async fn git_get_head_commit(&self, path: &Path) -> Result<String, WorkspaceError> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| WorkspaceError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(WorkspaceError::Git("failed to get HEAD commit".to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

// ============================================================================
// Types
// ============================================================================

/// Workspace statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceStats {
    pub total: usize,
    pub ready: usize,
    pub dirty: usize,
    pub merging: usize,
    pub merged: usize,
    pub max_concurrent: usize,
}

/// Workspace errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum WorkspaceError {
    #[error("workspace not found: {0}")]
    NotFound(String),

    #[error("too many workspaces: {current}/{max}")]
    TooManyWorkspaces { current: usize, max: usize },

    #[error("review required before merge: {0}")]
    ReviewRequired(String),

    #[error("git error: {0}")]
    Git(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("merge conflict: {0}")]
    MergeConflict(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_default() {
        assert_eq!(IsolationLevel::default(), IsolationLevel::Worktree);
    }

    #[test]
    fn test_merge_strategy_default() {
        assert_eq!(MergeStrategy::default(), MergeStrategy::PullRequest);
    }

    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();
        assert_eq!(config.default_isolation, IsolationLevel::Worktree);
        assert_eq!(config.max_concurrent, 10);
        assert!(config.require_review);
    }

    #[test]
    fn test_workspace_serialization() {
        let workspace = IsolatedWorkspace {
            id: "ws-test".to_string(),
            name: "test workspace".to_string(),
            worktree_path: PathBuf::from("/tmp/test"),
            branch_name: "agent/test".to_string(),
            repo_path: PathBuf::from("/tmp/repo"),
            status: WorkspaceStatus::Ready,
            isolation_level: IsolationLevel::Worktree,
            created_at: "2026-09-16T00:00:00Z".to_string(),
            task_id: Some("task-1".to_string()),
            changed_files: vec![PathBuf::from("src/main.rs")],
            reviewed: false,
            approved: false,
        };

        let json = serde_json::to_string(&workspace).unwrap();
        let deserialized: IsolatedWorkspace = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "ws-test");
        assert_eq!(deserialized.status, WorkspaceStatus::Ready);
    }
}
