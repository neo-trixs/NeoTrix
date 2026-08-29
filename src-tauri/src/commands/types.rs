use serde::{Serialize, Deserialize};

#[derive(Debug, serde::Serialize)]
#[allow(dead_code)]
pub struct ProxyStatus {
    pub running: bool,
    pub mode: String,
    pub pid: u32,
    pub port: u16,
    pub uptime_secs: u64,
    pub active_count: u64,
    pub idle_secs: u64,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self {
            running: false,
            mode: "off".into(),
            pid: 0,
            port: 11080,
            uptime_secs: 0,
            active_count: 0,
            idle_secs: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub message_count: usize,
    pub created: i64,
    pub project: String,
    pub sort_order: i64,
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
pub struct DiffBlock {
    pub r#type: String,
    pub content: String,
    pub line_start: u32,
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

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatus {
    pub running: bool,
    pub current_task: Option<String>,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PermissionRequest {
    pub id: String,
    pub action: String,
    pub target: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_type: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub pinned: bool,
    pub archived: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatFileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub file_count: usize,
}

