//! nt_lightrag_index — LightRAG 增量索引 (LightRagIndex).
//! 从 `nt_memory_graphrag/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};

use super::nt_types::{GlobalSummary, IncrementalChange};

// ─── LightRag Index ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightRagIndex {
    pub global_summaries: Vec<GlobalSummary>,
    pub change_log: Vec<IncrementalChange>,
    pub last_community_update: u64,
    pub query_count: u64,
}

impl Default for LightRagIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl LightRagIndex {
    pub fn new() -> Self {
        LightRagIndex {
            global_summaries: Vec::new(),
            change_log: Vec::new(),
            last_community_update: 0,
            query_count: 0,
        }
    }
}
