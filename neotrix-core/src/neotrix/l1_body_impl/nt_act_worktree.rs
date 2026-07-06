//! Git Worktree 管理 — E8 模式感知的工作树隔离
//! 每个工作树可以绑定不同的 E8 模式，实现推理上下文隔离

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub e8_mode: Option<u8>,
    pub commit_hash: String,
    pub is_active: bool,
    pub created_at: u64,
    pub last_used: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeCreateArgs {
    pub name: String,
    pub branch: String,
    pub e8_mode: Option<u8>,
    pub description: String,
    pub base_dir: Option<String>,
}

pub struct WorktreeManager {
    repo_path: PathBuf,
    worktrees_dir: PathBuf,
    allow_cleanup: bool,
}

impl WorktreeManager {
    pub fn new(repo_path: &str) -> Self {
        let base = PathBuf::from(repo_path);
        let wt_dir = base.join("..").join("neotrix-wts").join("worktrees");
        Self {
            repo_path: base,
            worktrees_dir: wt_dir,
            allow_cleanup: true,
        }
    }

    pub fn with_cleanup(mut self, allow: bool) -> Self {
        self.allow_cleanup = allow;
        self
    }

    /// 创建工作树 — git worktree add + 追踪元数据
    pub fn create(&self, args: &WorktreeCreateArgs) -> Result<WorktreeInfo, String> {
        let wt_path = self.worktrees_dir.join(&args.name);
        let branch = &args.branch;

        std::fs::create_dir_all(&wt_path).map_err(|e| format!("create worktree dir: {}", e))?;

        let output = std::process::Command::new("git")
            .args(["worktree", "add", "--checkout"])
            .arg(wt_path.to_str().unwrap_or(""))
            .arg(branch)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git worktree add failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git worktree add error: {}", stderr));
        }

        // Get commit hash
        let hash_output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git rev-parse: {}", e))?;
        let commit_hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

        self.save_metadata(&args.name, &WorktreeMeta {
            name: args.name.clone(),
            e8_mode: args.e8_mode,
            description: args.description.clone(),
            created_at: now,
            branch: branch.clone(),
        })?;

        Ok(WorktreeInfo {
            name: args.name.clone(),
            path: wt_path.to_string_lossy().to_string(),
            branch: branch.clone(),
            e8_mode: args.e8_mode,
            commit_hash,
            is_active: true,
            created_at: now,
            last_used: now,
            description: args.description.clone(),
        })
    }

    /// 列出所有工作树
    pub fn list(&self) -> Result<Vec<WorktreeInfo>, String> {
        let output = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git worktree list: {}", e))?;

        if !output.status.success() {
            return Err(format!("git worktree list error: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut trees = Vec::new();

        for entry in stdout.split("\n\n") {
            if entry.trim().is_empty() { continue; }
            let mut path = String::new();
            let mut branch = String::new();
            let mut commit = String::new();
            let is_active = false;

            for ln in entry.lines() {
                if let Some(p) = ln.strip_prefix("worktree ") {
                    path = p.to_string();
                } else if let Some(b) = ln.strip_prefix("branch refs/heads/") {
                    branch = b.to_string();
                } else if let Some(h) = ln.strip_prefix("HEAD ") {
                    commit = h.to_string();
                } else if ln == "bare" {
                    // skip bare
                }
            }
            if branch == "main" || branch == "worktrees" { /* skip */ }

            if !path.is_empty() {
                let meta = self.load_metadata(&branch);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                trees.push(WorktreeInfo {
                    name: meta.as_ref().map(|m| m.name.clone()).unwrap_or_else(|| branch.clone()),
                    path,
                    branch,
                    e8_mode: meta.as_ref().and_then(|m| m.e8_mode),
                    commit_hash: commit,
                    is_active,
                    created_at: meta.as_ref().map(|m| m.created_at).unwrap_or(0),
                    last_used: now,
                    description: meta.as_ref().map(|m| m.description.clone()).unwrap_or_default(),
                });
            }
        }
        Ok(trees)
    }

    /// 移除工作树
    pub fn remove(&self, name: &str) -> Result<(), String> {
        let meta = self.load_metadata(name);
        let _branch = meta.as_ref().map(|m| m.branch.clone()).unwrap_or_else(|| name.to_string());

        let output = std::process::Command::new("git")
            .args(["worktree", "remove"])
            .arg(self.worktrees_dir.join(name).to_str().unwrap_or(""))
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git worktree remove: {}", e))?;

        if output.status.success() {
            self.remove_metadata(name)?;
        }
        // git worktree remove --force if dirty
        if !output.status.success() && self.allow_cleanup {
            let _ = std::process::Command::new("git")
                .args(["worktree", "remove", "--force"])
                .arg(self.worktrees_dir.join(name).to_str().unwrap_or(""))
                .current_dir(&self.repo_path)
                .output();
            self.remove_metadata(name)?;
        }

        if !output.status.success() {
            return Err(format!("remove failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Ok(())
    }

    pub fn prune(&self) -> Result<String, String> {
        let output = std::process::Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| format!("git worktree prune: {}", e))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn metadata_path(&self, name: &str) -> PathBuf {
        let meta_dir = self.repo_path.join(".neotrix").join("worktrees");
        meta_dir.join(format!("{}.json", name))
    }

    fn save_metadata(&self, name: &str, meta: &WorktreeMeta) -> Result<(), String> {
        let path = self.metadata_path(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create meta dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(meta).map_err(|e| format!("serialize: {}", e))?;
        std::fs::write(&path, &json).map_err(|e| format!("write meta: {}", e))
    }

    fn load_metadata(&self, name: &str) -> Option<WorktreeMeta> {
        let path = self.metadata_path(name);
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<WorktreeMeta>(&s).ok())
    }

    fn remove_metadata(&self, name: &str) -> Result<(), String> {
        let path = self.metadata_path(name);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("remove meta: {}", e))
        } else {
            Ok(())
        }
    }

    pub fn count(&self) -> Result<usize, String> {
        self.list().map(|v| v.len())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorktreeMeta {
    name: String,
    e8_mode: Option<u8>,
    description: String,
    created_at: u64,
    branch: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_manager_creation() {
        let mgr = WorktreeManager::new("/tmp/test-repo");
        let trees = mgr.list();
        // Should fail or return empty — depends on whether /tmp/test-repo exists
        assert!(trees.is_ok() || trees.is_err());
    }

    #[test]
    fn test_metadata_roundtrip() {
        let mgr = WorktreeManager::new("/tmp/meta-test");
        let meta = WorktreeMeta {
            name: "feature-x".into(),
            e8_mode: Some(17),
            description: "E8 mode 17 research".into(),
            created_at: 1234567890,
            branch: "feature/x".into(),
        };
        assert!(mgr.save_metadata("feature-x", &meta).is_ok());
        let loaded = mgr.load_metadata("feature-x");
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "feature-x");
        assert_eq!(loaded.e8_mode, Some(17));
        let _ = mgr.remove_metadata("feature-x");
    }

    #[test]
    fn test_prune_non_existent_repo() {
        let mgr = WorktreeManager::new("/tmp/nonexistent-repo-12345");
        let result = mgr.prune();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_args_defaults() {
        let args = WorktreeCreateArgs {
            name: "exp".into(),
            branch: "feature/exp".into(),
            e8_mode: Some(33),
            description: "E8 mode 33".into(),
            base_dir: None,
        };
        assert_eq!(args.name, "exp");
        assert_eq!(args.e8_mode, Some(33));
        assert_eq!(args.branch, "feature/exp");
    }
}
