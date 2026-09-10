//! L5 包装器 — 封装 L1 具体实现, 提供 L5 层接口
//!
//! 这些包装器遵循依赖倒置原则: L5 定义接口, L1 实现细节
//! L5 代码通过这些包装器访问 L1 功能, 而不直接依赖 L1 类型

use serde::{Deserialize, Serialize};

/// 会话快照数据 (L5 层定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub e8_state_sequence: Vec<u8>,
    pub message_count: u64,
    pub active_topics: Vec<String>,
    pub created_at: u64,
}

/// 会话恢复管理器 trait — L5 认知层合约
pub trait SessionRecovery: Send + Sync {
    /// 创建快照
    fn create_snapshot(
        &mut self,
        e8_states: &[u8],
        topics: &[String],
        bank_state: &str,
    ) -> Result<SessionSnapshot, String>;

    /// 是否需要创建快照
    fn should_snapshot(&self) -> bool;

    /// 构建会话交接数据
    fn build_handoff(&self) -> Option<String>;
}

/// 会话恢复管理器包装器 — 封装 L1 SessionRecoveryManager
pub struct SessionRecoveryWrapper {
    inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager,
}

impl SessionRecoveryWrapper {
    pub fn new(session_id: &str) -> Self {
        Self {
            inner: crate::l1_action::nt_io::nt_io_session_recovery::SessionRecoveryManager::new(session_id),
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.inner = self.inner.with_interval(interval);
        self
    }

    pub fn with_auto_recover(mut self, recover: bool) -> Self {
        self.inner = self.inner.with_auto_recover(recover);
        self
    }
}

impl SessionRecovery for SessionRecoveryWrapper {
    fn create_snapshot(
        &mut self,
        e8_states: &[u8],
        topics: &[String],
        bank_state: &str,
    ) -> Result<SessionSnapshot, String> {
        self.inner.create_snapshot(e8_states, topics, bank_state).map(|s| SessionSnapshot {
            session_id: s.session_id,
            e8_state_sequence: s.e8_state_sequence,
            message_count: s.message_count,
            active_topics: s.active_topics,
            created_at: s.created_at,
        })
    }

    fn should_snapshot(&self) -> bool {
        self.inner.should_snapshot()
    }

    fn build_handoff(&self) -> Option<String> {
        self.inner.build_handoff()
    }
}

/// 用户画像蒸馏结果 (L5 层定义)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationResult {
    pub nodes_created: usize,
    pub edges_created: usize,
    pub avatar_confidence: f64,
}

/// 用户画像蒸馏引擎 trait — L5 认知层合约
pub trait UserDistillation: Send + Sync {
    /// 自动蒸馏
    fn auto_distill(&mut self) -> String;

    /// 获取用户画像摘要
    fn get_profile_summary(&self) -> String;
}

/// 用户画像蒸馏引擎包装器 — 封装 L1 DistillationEngine
pub struct DistillationEngineWrapper {
    inner: std::sync::Mutex<crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine>,
}

impl DistillationEngineWrapper {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(crate::l1_action::nt_io::nt_io_user_avatar::DistillationEngine::new()),
        }
    }
}

impl Default for DistillationEngineWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl UserDistillation for DistillationEngineWrapper {
    fn auto_distill(&mut self) -> String {
        if let Ok(mut eng) = self.inner.lock() {
            eng.auto_distill()
        } else {
            String::new()
        }
    }

    fn get_profile_summary(&self) -> String {
        if let Ok(eng) = self.inner.lock() {
            format!("edition={}, confidence={:.2}, msgs={}",
                eng.avatar.edition, eng.avatar.confidence, eng.avatar.total_messages_processed)
        } else {
            String::new()
        }
    }
}
