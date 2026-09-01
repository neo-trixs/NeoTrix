//! Concurrency Isolation Tester — 并发隔离测试器
//!
//! 吸收 KB 经验:
//! - git worktree add 隔离树避开并发会话文件编辑
//! - 隔离 CARGO_TARGET_DIR 避开默认目录抢锁/OOM
//! - 判定测试失败是并发竞态还是真 bug
//! - 干净基线获取

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 并发隔离测试器
pub struct ConcurrencyIsolationTester {
    test_sessions: Vec<TestSession>,
    isolation_configs: Vec<IsolationConfig>,
    results: Vec<TestResult>,
    config: ConcurrencyConfig,
    stats: ConcurrencyStats,
}

/// 并发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    pub enable_worktree_isolation: bool,
    pub enable_target_dir_isolation: bool,
    pub max_concurrent_sessions: usize,
    pub timeout_seconds: u64,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            enable_worktree_isolation: true,
            enable_target_dir_isolation: true,
            max_concurrent_sessions: 3,
            timeout_seconds: 300,
        }
    }
}

/// 测试会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub session_id: String,
    pub worktree_path: Option<String>,
    pub target_dir: Option<String>,
    pub status: SessionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Created,
    Running,
    Completed,
    Failed,
    Timeout,
}

/// 隔离配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    pub use_worktree: bool,
    pub use_separate_target_dir: bool,
    pub cleanup_on_complete: bool,
}

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub result_id: String,
    pub session_id: String,
    pub test_name: String,
    pub passed: bool,
    pub is_race_condition: bool,
    pub is_real_bug: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

/// 并发统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyStats {
    pub total_sessions: u64,
    pub completed_sessions: u64,
    pub failed_sessions: u64,
    pub race_conditions_detected: u64,
    pub real_bugs_found: u64,
    pub avg_session_duration: f64,
}

impl ConcurrencyIsolationTester {
    /// 创建新的并发隔离测试器
    pub fn new() -> Self {
        Self {
            test_sessions: Vec::new(),
            isolation_configs: Vec::new(),
            results: Vec::new(),
            config: ConcurrencyConfig::default(),
            stats: ConcurrencyStats {
                total_sessions: 0,
                completed_sessions: 0,
                failed_sessions: 0,
                race_conditions_detected: 0,
                real_bugs_found: 0,
                avg_session_duration: 0.0,
            },
        }
    }

    /// 创建隔离测试会话
    pub fn create_session(&mut self, use_worktree: bool) -> TestSession {
        let session_id = uuid::Uuid::new_v4().to_string();

        let worktree_path = if use_worktree && self.config.enable_worktree_isolation {
            Some(format!("/tmp/nt-worktree-{}", session_id))
        } else {
            None
        };

        let target_dir = if self.config.enable_target_dir_isolation {
            Some(format!("/tmp/nt-target-{}", session_id))
        } else {
            None
        };

        let session = TestSession {
            session_id: session_id.clone(),
            worktree_path,
            target_dir,
            status: SessionStatus::Created,
            created_at: chrono::Utc::now(),
        };

        self.test_sessions.push(session.clone());
        self.stats.total_sessions += 1;

        session
    }

    /// 分析测试失败
    pub fn analyze_failure(&mut self, session_id: &str, test_name: &str, error: &str) -> TestResult {
        // 简化版: 基于错误信息判断是否为竞态条件
        let is_race_condition = error.contains("conflict") || 
                               error.contains("lock") ||
                               error.contains("temporary");

        let is_real_bug = !is_race_condition && 
                         (error.contains("assertion") || 
                          error.contains("expected") ||
                          error.contains("panicked"));

        let result = TestResult {
            result_id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            test_name: test_name.to_string(),
            passed: false,
            is_race_condition,
            is_real_bug,
            duration_ms: 0,
            error_message: Some(error.to_string()),
        };

        if is_race_condition {
            self.stats.race_conditions_detected += 1;
        }
        if is_real_bug {
            self.stats.real_bugs_found += 1;
        }

        self.results.push(result.clone());
        result
    }

    /// 获取干净基线
    pub fn get_clean_baseline(&self) -> HashMap<String, String> {
        let mut baseline = HashMap::new();
        baseline.insert("command".into(), "CARGO_TARGET_DIR=/tmp/nt-target-clean cargo test -p neotrix --lib".into());
        baseline.insert("expected".into(), "8123 passed, 0 failed".into());
        baseline
    }

    /// 获取所有会话
    pub fn sessions(&self) -> &[TestSession] {
        &self.test_sessions
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ConcurrencyStats {
        &self.stats
    }
}
