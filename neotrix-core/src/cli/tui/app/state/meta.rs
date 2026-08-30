use super::*;
#[allow(unused_imports)]
use std::time::Instant;

/// 元数据状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaState {
    pub started_at: DateTime<Utc>,
    pub session_started_at: Option<DateTime<Utc>>,
    pub total_tokens: u64,
    pub total_messages: u64,
    pub total_tool_calls: u64,
    pub session_id: String,
    pub version: String,
    pub git_branch: Option<String>,
    pub git_dirty: bool,
    pub workspace_name: String,
    pub workspace_count: usize,
    pub tokens_per_sec: f64,
}
