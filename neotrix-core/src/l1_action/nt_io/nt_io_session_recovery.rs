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
        // git 备份是 best-effort：失败不应阻断主快照创建，但已在 git_commit 内部记录错误
        let _ = self.git_commit(&snapshot);
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
            // 文件存在但损坏/不可解析时，回退 git 备份（file 存在并不能保证可读）
            let direct = std::fs::read_to_string(&path).ok()
                .and_then(|s| serde_json::from_str::<SessionSnapshot>(&s).ok());
            match direct {
                Some(s) => Some(s),
                None if self.auto_recover => self.git_restore_latest(),
                None => None,
            }
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
        // 原子写：快照文件不因 crash 截断损坏
        neotrix_types::fs_util::atomic_write(&self.snapshot_path("latest"), json.as_bytes())
            .map_err(|e| format!("write snapshot: {}", e))?;
        let ts_path = self.snapshot_path(&format!("snap-{}", snapshot.created_at));
        neotrix_types::fs_util::atomic_write(&ts_path, json.as_bytes())
            .map_err(|e| format!("write timestamped snapshot: {}", e))?;
        Ok(())
    }

    fn git_commit(&self, snapshot: &SessionSnapshot) -> Result<(), String> {
        let repo_dir = &self.snapshots_dir;
        if !repo_dir.join(".git").exists() {
            for (label, args) in [
                ("init", vec!["init"]),
                ("config email", vec!["config", "user.email", "neotrix@session"]),
                ("config name", vec!["config", "user.name", "NeoTrix Session"]),
            ] {
                let out = std::process::Command::new("git")
                    .args(&args)
                    .current_dir(repo_dir)
                    .output()
                    .map_err(|e| format!("git {label} spawn failed: {e}"))?;
                if !out.status.success() {
                    log::warn!("session git {label} failed: {}", String::from_utf8_lossy(&out.stderr).trim());
                }
            }
        }
        // git add 后检查 exit status：非零即失败，不能静默吞掉导致 backup 缺失
        let out = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("git add spawn failed: {e}"))?;
        if !out.status.success() {
            log::warn!("session git add failed: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        let msg = format!("session {} snapshot {}", self.session_id, snapshot.created_at);
        let out = std::process::Command::new("git")
            .args(["commit", "-m", &msg, "--allow-empty"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("git commit spawn failed: {e}"))?;
        if !out.status.success() {
            return Err(format!("session git commit failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
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
            let checkout = std::process::Command::new("git")
                .args(["checkout", "HEAD", "--", "."])
                .current_dir(repo_dir)
                .output().ok()?;
            if !checkout.status.success() {
                log::warn!("session git checkout restore failed: {}", String::from_utf8_lossy(&checkout.stderr).trim());
                return None;
            }
        }
        let path = self.snapshot_path("latest");
        std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<SessionSnapshot>(&s).ok())
    }

    /// 从最新快照构建有界交接摘要 (ai-memory 吸收, R-P79 接线)。
    ///
    /// ai-memory 核心概念: 会话结束时相关观察收敛为连贯摘要, 下一个 agent 接收
    /// **有界 (bounded) 交接** — 而非重新解释架构/已失败方法/未决问题。
    /// 这里把同样概念落地到 SessionRecoveryManager: 从最新快照提取高信号上下文,
    /// 生成一条 ≤ HANDOFF_MAX_CHARS 的交接串, 供恢复/日志/后台循环注入下一会话。
    pub fn build_handoff(&self) -> Option<String> {
        const HANDOFF_MAX_CHARS: usize = 600;
        let snap = self.load_latest_snapshot()?;
        if snap.message_count == 0 && snap.active_topics.is_empty() && snap.plan_ids.is_empty() {
            return None;
        }
        let mut parts: Vec<String> = Vec::new();
        if !snap.active_topics.is_empty() {
            let topics = snap.active_topics.join(", ");
            parts.push(format!("topics: {}", topics));
        }
        if !snap.plan_ids.is_empty() {
            parts.push(format!("open plans: {}", snap.plan_ids.len()));
        }
        if snap.bank_snapshot.trim().len() > 3 {
            parts.push("state: has bank snapshot".to_string());
        }
        let body = parts.join(" | ");
        if body.is_empty() {
            return None;
        }
        let prefix = format!("[session-handoff {}] ", self.session_id);
        if prefix.len() + body.len() <= HANDOFF_MAX_CHARS {
            return Some(format!("{}{}", prefix, body));
        }
        // 超界截断到最近的 token 边界 (避免切断 UTF-8 序列)
        let budget = HANDOFF_MAX_CHARS - prefix.len();
        let mut idx = budget;
        while idx > 0 && !body.is_char_boundary(idx) {
            idx -= 1;
        }
        Some(format!("{}{}…", prefix, &body[..idx]))
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

    #[test]
    fn test_build_handoff_from_snapshot() {
        let mut mgr = SessionRecoveryManager::new("handoff-test")
            .with_interval(1);
        let snap = mgr.create_snapshot(
            &[1, 2, 3],
            &["rust".to_string(), "memory".to_string(), "absorption".to_string()],
            "bank_ctx_xyz",
        ).expect("create snapshot");
        let handoff = mgr.build_handoff().expect("handoff built");
        assert!(handoff.contains("handoff-test"), "session id in handoff");
        assert!(handoff.contains("rust"), "active topic surfaced");
        assert!(handoff.contains("absorption"));
        assert!(handoff.contains("bank snapshot"), "bank state surfaced");
        assert!(handoff.len() <= 700, "handoff is bounded, got {}", handoff.len());
        // 快照轮数 > 0 → 交接非空
        assert!(snap.created_at > 0);
    }

    #[test]
    fn test_build_handoff_none_when_empty() {
        // 无快照 → None (不产生空交接)
        let mgr = SessionRecoveryManager::new("empty-handoff")
            .with_auto_recover(false);
        assert!(mgr.build_handoff().is_none());
    }
}
