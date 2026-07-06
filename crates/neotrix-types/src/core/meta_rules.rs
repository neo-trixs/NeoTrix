use serde::{Serialize, Deserialize};

/// Rules that govern how memory operates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRules {
    /// Minimum importance threshold for L4 storage (0.0-1.0)
    pub archive_threshold: f64,
    /// How often to auto-compact L4 (in iterations)
    pub compact_interval: u32,
    /// Maximum tokens to load from L2 for context
    pub max_session_tokens: usize,
    /// Whether to auto-tag memories based on content
    pub auto_tagging: bool,
}

impl Default for MetaRules {
    fn default() -> Self {
        Self {
            archive_threshold: 0.3,
            compact_interval: 50,
            max_session_tokens: 4096,
            auto_tagging: true,
        }
    }
}
