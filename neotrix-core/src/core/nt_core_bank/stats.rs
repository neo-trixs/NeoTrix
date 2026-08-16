use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_bank_stats_default() {
        let s = ReasoningBankStats {
            total_memories: 0,
            success_count: 0,
            success_rate: 0.0,
        };
        assert_eq!(s.total_memories, 0);
        assert_eq!(s.success_rate, 0.0);
    }

    #[test]
    fn test_reasoning_bank_stats_success_rate() {
        let s = ReasoningBankStats {
            total_memories: 10,
            success_count: 7,
            success_rate: 0.7,
        };
        assert_eq!(s.success_count, 7);
    }

    #[test]
    fn test_memory_detailed_stats_fields() {
        let s = MemoryDetailedStats {
            total: 100,
            tier_working: 30,
            tier_episodic: 30,
            tier_semantic: 20,
            tier_procedural: 20,
            avg_confidence: 0.75,
            avg_importance: 0.6,
        };
        assert_eq!(s.total, 100);
        assert_eq!(s.tier_working, 30);
        assert_eq!(s.tier_semantic, 20);
    }

    #[test]
    fn test_memory_detailed_stats_tier_sum() {
        let s = MemoryDetailedStats {
            total: 100,
            tier_working: 25,
            tier_episodic: 25,
            tier_semantic: 25,
            tier_procedural: 25,
            avg_confidence: 0.5,
            avg_importance: 0.5,
        };
        let sum = s.tier_working + s.tier_episodic + s.tier_semantic + s.tier_procedural;
        assert_eq!(sum, 100);
    }
}
