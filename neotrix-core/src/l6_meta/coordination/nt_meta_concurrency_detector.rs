//! Concurrency Conflict Detector — 并发冲突检测器
//!
//! 吸收 KB 经验:
//! - OpenHands 自主循环并发改写共享文件
//! - 共享文件 (cortex_sync.rs/nt_core_llm.rs/CONTEXT.md) 瞬时破坏编译
//! - 应对策略: 本地修复循环文件, 提交时只 stage 自己的文件
//! - 真正回归防线是 CI + required status checks

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 并发冲突检测器
#[allow(dead_code)]
pub struct ConcurrencyConflictDetector {
    monitored_files: Vec<MonitoredFile>,
    conflicts: Vec<Conflict>,
    lock_states: HashMap<String, LockState>,
    config: ConflictDetectorConfig,
    stats: ConflictDetectorStats,
}

/// 冲突检测器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectorConfig {
    pub enable_file_locking: bool,
    pub lock_timeout_ms: u64,
    pub max_conflict_history: usize,
    pub enable_auto_resolution: bool,
}

impl Default for ConflictDetectorConfig {
    fn default() -> Self {
        Self {
            enable_file_locking: true,
            lock_timeout_ms: 5000,
            max_conflict_history: 100,
            enable_auto_resolution: true,
        }
    }
}

/// 监控文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredFile {
    pub file_path: String,
    pub file_type: FileType,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub locked_by: Option<String>,
    pub conflict_count: u64,
}

/// 文件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    SharedConfig,
    SharedState,
    PublicAPI,
    Documentation,
}

/// 冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub conflict_id: String,
    pub file_path: String,
    pub session_a: String,
    pub session_b: String,
    pub conflict_type: ConflictType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    ConcurrentWrite,
    ReadWriteConflict,
    VersionMismatch,
    LockTimeout,
}

/// 锁状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LockState {
    Unlocked,
    Locked { holder: String, since: chrono::DateTime<chrono::Utc> },
}

/// 冲突检测器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetectorStats {
    pub total_files_monitored: u64,
    pub total_conflicts: u64,
    pub resolved_conflicts: u64,
    pub active_locks: u64,
    pub avg_resolution_time: f64,
}

/// 冲突解决结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub resolution_type: String,
    pub success: bool,
    pub message: String,
}

impl ConcurrencyConflictDetector {
    /// 创建新的并发冲突检测器
    pub fn new() -> Self {
        Self {
            monitored_files: Vec::new(),
            conflicts: Vec::new(),
            lock_states: HashMap::new(),
            config: ConflictDetectorConfig::default(),
            stats: ConflictDetectorStats {
                total_files_monitored: 0,
                total_conflicts: 0,
                resolved_conflicts: 0,
                active_locks: 0,
                avg_resolution_time: 0.0,
            },
        }
    }

    /// 监控文件
    pub fn monitor_file(&mut self, file_path: &str, file_type: FileType) {
        let file = MonitoredFile {
            file_path: file_path.to_string(),
            file_type,
            last_modified: chrono::Utc::now(),
            locked_by: None,
            conflict_count: 0,
        };

        self.monitored_files.push(file);
        self.lock_states.insert(file_path.to_string(), LockState::Unlocked);
        self.stats.total_files_monitored += 1;
    }

    /// 尝试获取锁
    pub fn try_lock(&mut self, file_path: &str, session_id: &str) -> bool {
        if let Some(state) = self.lock_states.get(file_path) {
            if *state == LockState::Unlocked {
                *self.lock_states.get_mut(file_path).unwrap() = LockState::Locked {
                    holder: session_id.to_string(),
                    since: chrono::Utc::now(),
                };
                self.stats.active_locks += 1;
                return true;
            }
        }
        false
    }

    /// 释放锁
    pub fn release_lock(&mut self, file_path: &str) -> bool {
        if let Some(state) = self.lock_states.get_mut(file_path) {
            if *state != LockState::Unlocked {
                *state = LockState::Unlocked;
                self.stats.active_locks -= 1;
                return true;
            }
        }
        false
    }

    /// 检测冲突
    pub fn detect_conflict(&mut self, file_path: &str, session_a: &str, session_b: &str) -> Option<Conflict> {
        // 检查是否有锁冲突
        if let Some(state) = self.lock_states.get(file_path) {
            if let LockState::Locked { holder, .. } = state {
                if holder != session_a && holder != session_b {
                    let conflict = Conflict {
                        conflict_id: uuid::Uuid::new_v4().to_string(),
                        file_path: file_path.to_string(),
                        session_a: session_a.to_string(),
                        session_b: session_b.to_string(),
                        conflict_type: ConflictType::ConcurrentWrite,
                        timestamp: chrono::Utc::now(),
                        resolved: false,
                    };

                    self.conflicts.push(conflict.clone());
                    self.stats.total_conflicts += 1;

                    return Some(conflict);
                }
            }
        }
        None
    }

    /// 获取所有监控文件
    pub fn monitored_files(&self) -> &[MonitoredFile] {
        &self.monitored_files
    }

    /// 获取所有冲突
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ConflictDetectorStats {
        &self.stats
    }
}
