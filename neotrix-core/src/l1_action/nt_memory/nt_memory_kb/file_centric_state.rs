//! File-Centric State Abstraction — 吸收自 InfiAgent + arcagent-state
//!
//! 核心原则: 持久状态外化到文件系统，与推理上下文完全解耦。
//! 任务获取专用工作空间目录，每步从工作空间快照+固定窗口重建上下文。
//! O(1) 上下文大小，与任务持续时间无关。

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 工作空间状态快照 — 固定大小，用于每步上下文重建
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub task_id: String,
    pub plan_summary: String,
    pub file_descriptions: Vec<String>,
    pub pinned_state: Vec<String>,
    pub next_steps: Vec<String>,
    pub recent_actions: Vec<ActionRecord>,
    pub timestamp: u64,
}

/// 动作记录 — 固定窗口大小
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub action_type: String,
    pub input: String,
    pub output: String,
    pub success: bool,
    pub timestamp: u64,
}

/// 文件中心状态管理器
pub struct FileCentricState {
    workspace_dir: PathBuf,
    snapshots: HashMap<String, WorkspaceSnapshot>,
    action_buffer_size: usize,
}

impl FileCentricState {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self {
            workspace_dir,
            snapshots: HashMap::new(),
            action_buffer_size: 10, // 固定窗口大小
        }
    }

    /// 从工作空间重建上下文 — O(1) 大小
    pub fn reconstruct_context(&self, task_id: &str) -> Option<WorkspaceSnapshot> {
        self.snapshots.get(task_id).cloned()
    }

    /// 更新工作空间快照
    pub fn update_snapshot(&mut self, snapshot: WorkspaceSnapshot) {
        self.snapshots.insert(snapshot.task_id.clone(), snapshot);
    }

    /// 添加动作到缓冲区，自动裁剪到固定窗口
    pub fn add_action(&mut self, task_id: &str, action: ActionRecord) {
        if let Some(snapshot) = self.snapshots.get_mut(task_id) {
            snapshot.recent_actions.push(action);
            let len = snapshot.recent_actions.len();
            if len > self.action_buffer_size {
                snapshot.recent_actions.drain(0..len - self.action_buffer_size);
            }
        }
    }

    /// 持久化快照到文件系统
    pub fn persist(&self, task_id: &str) -> std::io::Result<()> {
        if let Some(snapshot) = self.snapshots.get(task_id) {
            let path = self.workspace_dir.join(format!("{}.json", task_id));
            let json = serde_json::to_string_pretty(snapshot)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }

    /// 从文件系统恢复快照
    pub fn restore(&mut self, task_id: &str) -> std::io::Result<()> {
        let path = self.workspace_dir.join(format!("{}.json", task_id));
        if path.exists() {
            let json = std::fs::read_to_string(&path)?;
            let snapshot: WorkspaceSnapshot = serde_json::from_str(&json)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            self.snapshots.insert(task_id.to_string(), snapshot);
        }
        Ok(())
    }

    /// 获取所有任务ID
    pub fn list_tasks(&self) -> Vec<String> {
        self.snapshots.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    #[test]
    fn test_file_centric_state_basic() {
        let tmp = std::env::temp_dir().join("nt_file_centric_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        
        let mut state = FileCentricState::new(tmp.clone());
        let snapshot = WorkspaceSnapshot {
            task_id: "test_task".to_string(),
            plan_summary: "Test plan".to_string(),
            file_descriptions: vec!["file1.rs".to_string()],
            pinned_state: vec!["state1".to_string()],
            next_steps: vec!["step1".to_string()],
            recent_actions: vec![],
            timestamp: now(),
        };
        state.update_snapshot(snapshot);
        
        // Context reconstruction is O(1)
        let ctx = state.reconstruct_context("test_task");
        assert!(ctx.is_some());
        assert_eq!(ctx.unwrap().task_id, "test_task");
        
        // Persist and restore
        state.persist("test_task").unwrap();
        let mut state2 = FileCentricState::new(tmp.clone());
        state2.restore("test_task").unwrap();
        assert!(state2.reconstruct_context("test_task").is_some());
        
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_action_buffer_fixed_size() {
        let tmp = std::env::temp_dir().join("nt_action_buffer_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        
        let mut state = FileCentricState::new(tmp.clone());
        state.update_snapshot(WorkspaceSnapshot {
            task_id: "t".to_string(),
            plan_summary: "".to_string(),
            file_descriptions: vec![],
            pinned_state: vec![],
            next_steps: vec![],
            recent_actions: vec![],
            timestamp: now(),
        });
        
        // Add 15 actions, buffer should keep only 10
        for i in 0..15 {
            state.add_action("t", ActionRecord {
                action_type: "test".to_string(),
                input: format!("in_{}", i),
                output: format!("out_{}", i),
                success: true,
                timestamp: now(),
            });
        }
        
        let ctx = state.reconstruct_context("t").unwrap();
        assert!(ctx.recent_actions.len() <= 10);
        
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
