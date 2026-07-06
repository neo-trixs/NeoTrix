//! 会话恢复系统 — Git 备份的 E8 推理状态持久化 + 自动恢复
//! 每 N 轮推理自动创建 Git snapshot，重启时恢复最新状态

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 会话快照 — E8 推理状态 + 上下文 + 计划的完整持久化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub e8_state_sequence: Vec<u8>,
    pub message_count: u64,
    pub active_topics: Vec<String>,
    pub plan_ids: Vec<String>,
    pub subagent_ids: Vec<String>,
    pub bank_snapshot: String,
    pub created_at: u64,
    pub git_commit_hash: Option<String>,
}

/// 会话恢复管理器
pub struct SessionRecoveryManager {
    session_id: String,
    snapshots_dir: PathBuf,
    snapshot_interval: u64,
    snapshot_count: u64,
    auto_recover: bool,
}

impl SessionRecoveryManager {
    pub fn new(session_id: &str) -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix")
            .join("sessions");
        Self {
            session_id: session_id.to_string(),
            snapshots_dir: base,
            snapshot_interval: 5,
            snapshot_count: 0,
            auto_recover: true,
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.snapshot_interval = interval;
        self
    }

    pub fn with_auto_recover(mut self, recover: bool) -> Self {
        self.auto_recover = recover;
        self
    }

    /// 创建快照 → 写入文件 → Git commit
    pub fn create_snapshot(&mut self, e8_states: &[u8], topics: &[String],
                           bank_state: &str) -> Result<SessionSnapshot, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let snapshot = SessionSnapshot {
            session_id: self.session_id.clone(),
            e8_state_sequence: e8_states.to_vec(),
            message_count: self.snapshot_count,
            active_topics: topics.to_vec(),
            plan_ids: vec![],
            subagent_ids: vec![],
            bank_snapshot: bank_state.to_string(),
            created_at: now,
            git_commit_hash: None,
        };
        self.snapshot_count += 1;
        self.persist_snapshot(&snapshot)?;
        self.git_commit(&snapshot)?;
        Ok(snapshot)
    }

    /// 是否需要创建快照 (每 N 次)
    pub fn should_snapshot(&self) -> bool {
        self.snapshot_count > 0 && self.snapshot_count.is_multiple_of(self.snapshot_interval)
    }

    /// 从磁盘加载最新快照
    pub fn load_latest_snapshot(&self) -> Option<SessionSnapshot> {
        let path = self.snapshot_path("latest");
        if path.exists() {
            std::fs::read_to_string(&path).ok()
                .and_then(|s| serde_json::from_str::<SessionSnapshot>(&s).ok())
        } else if self.auto_recover {
            self.git_restore_latest()
        } else {
            None
        }
    }

    fn snapshot_path(&self, name: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{}-{}.json", self.session_id, name))
    }

    fn persist_snapshot(&self, snapshot: &SessionSnapshot) -> Result<(), String> {
        std::fs::create_dir_all(&self.snapshots_dir)
            .map_err(|e| format!("create snapshots dir: {}", e))?;
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| format!("serialize: {}", e))?;
        std::fs::write(self.snapshot_path("latest"), &json)
            .map_err(|e| format!("write snapshot: {}", e))?;
        let ts_path = self.snapshot_path(&format!("snap-{}", snapshot.created_at));
        std::fs::write(ts_path, &json)
            .map_err(|e| format!("write timestamped snapshot: {}", e))?;
        Ok(())
    }

    fn git_commit(&self, snapshot: &SessionSnapshot) -> Result<(), String> {
        let repo_dir = &self.snapshots_dir;
        if !repo_dir.join(".git").exists() {
            if let Err(e) = std::process::Command::new("git")
                .args(["init"])
                .current_dir(repo_dir)
                .output()
                .map(|_| ())
            {
                log::warn!("session git init failed: {e}");
            }
            if let Err(e) = std::process::Command::new("git")
                .args(["config", "user.email", "neotrix@session"])
                .current_dir(repo_dir)
                .output()
                .map(|_| ())
            {
                log::warn!("session git config email failed: {e}");
            }
            if let Err(e) = std::process::Command::new("git")
                .args(["config", "user.name", "NeoTrix Session"])
                .current_dir(repo_dir)
                .output()
                .map(|_| ())
            {
                log::warn!("session git config name failed: {e}");
            }
        }
        if let Err(e) = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_dir)
            .output()
            .map(|_| ())
        {
            log::warn!("session git add failed: {e}");
        }
        let msg = format!("session {} snapshot {}", self.session_id, snapshot.created_at);
        if let Err(e) = std::process::Command::new("git")
            .args(["commit", "-m", &msg, "--allow-empty"])
            .current_dir(repo_dir)
            .output()
            .map(|_| ())
        {
            log::warn!("session git commit failed: {e}");
        }
        Ok(())
    }

    fn git_restore_latest(&self) -> Option<SessionSnapshot> {
        let repo_dir = &self.snapshots_dir;
        if !repo_dir.join(".git").exists() {
            return None;
        }
        let output = std::process::Command::new("git")
            .args(["log", "--oneline", "-1"])
            .current_dir(repo_dir)
            .output().ok()?;
        if output.status.success() {
            let _ = std::process::Command::new("git")
                .args(["checkout", "HEAD", "--", "."])
                .current_dir(repo_dir)
                .output().ok()?;
        }
        let path = self.snapshot_path("latest");
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<SessionSnapshot>(&s).ok())
    }

    pub fn get_recovery_info(&self) -> RecoveryInfo {
        let path = self.snapshot_path("latest");
        let has_snapshot = path.exists();
        let has_git = self.snapshots_dir.join(".git").exists();
        RecoveryInfo {
            session_id: self.session_id.clone(),
            has_snapshot,
            has_git_backup: has_git,
            snapshot_count: self.snapshot_count,
            snapshot_dir: self.snapshots_dir.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryInfo {
    pub session_id: String,
    pub has_snapshot: bool,
    pub has_git_backup: bool,
    pub snapshot_count: u64,
    pub snapshot_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_recovery_manager_creation() {
        let mgr = SessionRecoveryManager::new("test-session");
        let info = mgr.get_recovery_info();
        assert_eq!(info.session_id, "test-session");
        assert!(!info.has_snapshot);
    }

    #[test]
    fn test_create_snapshot() {
        let mut mgr = SessionRecoveryManager::new("snap-test")
            .with_interval(3);
        let snap = mgr.create_snapshot(&[1, 9, 17], &["rust".to_string(), "ai".to_string()], "bank_state_abc")
            .expect("create snapshot");
        assert_eq!(snap.e8_state_sequence, vec![1u8, 9, 17]);
        assert!(snap.created_at > 0);
    }

    #[test]
    fn test_should_snapshot() {
        let mut mgr = SessionRecoveryManager::new("interval-test")
            .with_interval(5);
        assert!(!mgr.should_snapshot());

        mgr.snapshot_count = 5;
        assert!(mgr.should_snapshot());

        mgr.snapshot_count = 6;
        assert!(!mgr.should_snapshot());
    }

    #[test]
    fn test_git_backend_skipped_if_no_git() {
        let mut mgr = SessionRecoveryManager::new("no-git-test")
            .with_interval(2);
        let snap = mgr.create_snapshot(&[0], &[], "data");
        assert!(snap.is_ok());
        assert!(snap.unwrap().git_commit_hash.is_none());
    }

    #[test]
    fn test_load_returns_none_if_no_snapshot() {
        let mgr = SessionRecoveryManager::new("nonexistent")
            .with_auto_recover(false);
        assert!(mgr.load_latest_snapshot().is_none());
    }
}
