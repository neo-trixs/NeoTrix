pub mod api;
pub mod keyframe_motion;
pub mod server;
pub mod share;
pub mod tiles;

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

// Re-export our types
pub use api::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub message_count: usize,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainStats {
    pub iteration: u64,
    pub absorb_count: u64,
    pub capability_sum: f64,
    pub memory_count: usize,
    pub engine_active: bool,
    pub capability_vector: Vec<f64>,
    pub dimension_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffBlock {
    pub r#type: String,
    pub content: String,
    pub line_start: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub running: bool,
    pub current_task: Option<String>,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub action: String,
    pub target: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigPayload {
    pub id: String,
    pub name: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub learning_rate: f64,
}

/// 全局固定窗口限流器 (F2: HTTP API rate limiting, release-checklist 8.6)。
/// `per_minute` = 每分钟允许请求数, 0 表示不限制。
#[derive(Debug, Clone)]
pub struct RateWindow {
    per_minute: u64,
    window_start: std::time::Instant,
    count: u64,
}

impl RateWindow {
    pub fn new(per_minute: u64) -> Self {
        Self {
            per_minute,
            window_start: std::time::Instant::now(),
            count: 0,
        }
    }

    /// 若请求数未超限则计数并放行; 否则拒绝。
    /// 窗口每 60s 重置 (固定窗口, 简单全局限流)。
    pub fn allow(&mut self) -> bool {
        if self.per_minute == 0 {
            return true;
        }
        if self.window_start.elapsed().as_secs() >= 60 {
            self.window_start = std::time::Instant::now();
            self.count = 0;
        }
        if self.count < self.per_minute {
            self.count += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub brain: Arc<Mutex<Box<dyn crate::core::nt_core_traits::BrainProvider>>>,
    pub bank: Arc<Mutex<crate::core::ReasoningBank>>,
    pub sessions: Arc<Mutex<Vec<SessionInfo>>>,
    pub permission_counter: Arc<AtomicU64>,
    pub pending_permissions: Arc<Mutex<Vec<PermissionRequest>>>,
    pub agent_running: Arc<Mutex<AgentStatus>>,
    pub agent_start_time: Arc<Mutex<Option<std::time::Instant>>>,
    pub api_token: Option<String>,
    pub rate_limiter: Arc<Mutex<RateWindow>>,
}
