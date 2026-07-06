pub mod api;
pub mod server;

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

#[derive(Clone)]
pub struct AppState {
    pub brain: Arc<Mutex<crate::neotrix::nt_mind::ReasoningBrain>>,
    pub bank: Arc<Mutex<crate::neotrix::nt_mind::ReasoningBank>>,
    pub sessions: Arc<Mutex<Vec<SessionInfo>>>,
    pub permission_counter: Arc<AtomicU64>,
    pub pending_permissions: Arc<Mutex<Vec<PermissionRequest>>>,
    pub agent_running: Arc<Mutex<AgentStatus>>,
    pub agent_start_time: Arc<Mutex<Option<std::time::Instant>>>,
    pub api_token: Option<String>,
}
