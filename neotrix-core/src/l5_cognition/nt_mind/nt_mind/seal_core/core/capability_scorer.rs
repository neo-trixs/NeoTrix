//! Capability Scorer — 能力评分器
//! 根据历史表现、资源消耗、成功率给能力打分

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CapabilityScore {
    pub capability_id: String,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub avg_tokens: f64,
    pub recency_weight: f64,
    pub composite_score: f64,
}

pub struct CapabilityScorer {
    history: HashMap<String, Vec<ExecutionRecord>>,
}

#[derive(Debug, Clone)]
struct ExecutionRecord {
    success: bool,
    latency_ms: u64,
    tokens: u64,
    timestamp: u64,
}

impl CapabilityScorer {
    pub fn new() -> Self {
        Self { history: HashMap::new() }
    }

    pub fn record_execution(&mut self, cap_id: &str, success: bool, latency_ms: u64, tokens: u64) {
        let record = ExecutionRecord {
            success,
            latency_ms,
            tokens,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        };
        self.history.entry(cap_id.to_string()).or_default().push(record);
    }

    pub fn score(&self, cap_id: &str) -> Option<CapabilityScore> {
        let records = self.history.get(cap_id)?;
        if records.is_empty() { return None; }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let total = records.len() as f64;
        let successes = records.iter().filter(|r| r.success).count() as f64;
        let avg_latency = records.iter().map(|r| r.latency_ms as f64).sum::<f64>() / total;
        let avg_tokens = records.iter().map(|r| r.tokens as f64).sum::<f64>() / total;
        
        // Recency weight: recent executions matter more
        let recency_weight = records.iter()
            .map(|r| 1.0 / (1.0 + (now - r.timestamp) as f64 / 86400.0))
            .sum::<f64>() / total;

        let success_rate = successes / total;
        let latency_score = 1.0 / (1.0 + avg_latency / 1000.0); // normalize
        let token_score = 1.0 / (1.0 + avg_tokens / 10000.0); // normalize

        let composite_score = success_rate * 0.5 + latency_score * 0.2 + token_score * 0.2 + recency_weight * 0.1;

        Some(CapabilityScore {
            capability_id: cap_id.to_string(),
            success_rate,
            avg_latency_ms: avg_latency,
            avg_tokens,
            recency_weight,
            composite_score,
        })
    }

    pub fn rank(&self) -> Vec<CapabilityScore> {
        let mut scores: Vec<CapabilityScore> = self.history.keys()
            .filter_map(|id| self.score(id))
            .collect();
        scores.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }
}

impl Default for CapabilityScorer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_basic() {
        let mut scorer = CapabilityScorer::new();
        scorer.record_execution("cap_a", true, 100, 500);
        scorer.record_execution("cap_a", true, 150, 600);
        scorer.record_execution("cap_b", false, 200, 800);
        
        let score_a = scorer.score("cap_a").unwrap();
        assert!(score_a.composite_score > 0.5);
        
        let rank = scorer.rank();
        assert_eq!(rank[0].capability_id, "cap_a");
    }
}
