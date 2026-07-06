use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningBankStats {
    pub total_memories: usize,
    pub success_count: usize,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDetailedStats {
    pub total: usize,
    pub tier_working: usize,
    pub tier_episodic: usize,
    pub tier_semantic: usize,
    pub tier_procedural: usize,
    pub avg_confidence: f64,
    pub avg_importance: f64,
}
