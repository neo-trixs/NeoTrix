use serde::{Serialize, Deserialize};

// ===== Feed command types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedRefreshRequest {
    pub tag_filter: Option<String>,
    pub search_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItemResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content_type: String,
    pub source_url: String,
    pub source_name: String,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub author: Option<String>,
    pub published_at: u64,
    pub score: f64,
    pub tags: Vec<String>,
    pub neotrix_insight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedStateResponse {
    pub items: Vec<FeedItemResponse>,
    pub timelines: Vec<EventTimelineResponse>,
    pub tags: Vec<TagResponse>,
    pub last_refresh: u64,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimelineResponse {
    pub id: String,
    pub title: String,
    pub item_ids: Vec<String>,
    pub start_time: u64,
    pub end_time: u64,
    pub key_events: Vec<String>,
    pub neotrix_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResponse {
    pub name: String,
    pub count: u64,
    pub is_active: bool,
}

// ===== End feed types =====

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
pub struct ReasonRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonResponse {
    pub output: String,
    pub success: bool,
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
pub struct ChainStats {
    pub total_entries: usize,
    pub outbound_count: usize,
    pub inbound_count: usize,
    pub genesis_hash: String,
    pub chain_valid: bool,
    pub identity_name: String,
    pub identity_edition: u32,
}
