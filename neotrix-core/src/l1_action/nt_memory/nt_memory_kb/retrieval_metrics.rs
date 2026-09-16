//! Retrieval Quality Metrics — 检索质量评估
//! 跟踪 precision, recall, MRR, NDCG for hybrid retrieval

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RetrievalMetrics {
    pub total_queries: u64,
    pub relevance_scores: Vec<f64>,
    pub rank_positions: Vec<usize>,
    pub source_stats: HashMap<String, SourceStats>,
}

#[derive(Debug, Clone, Default)]
pub struct SourceStats {
    pub queries: u64,
    pub hits: u64,
    pub avg_rank: f64,
}

impl RetrievalMetrics {
    pub fn new() -> Self {
        Self {
            total_queries: 0,
            relevance_scores: Vec::new(),
            rank_positions: Vec::new(),
            source_stats: HashMap::new(),
        }
    }

    pub fn record_query(&mut self, relevance: f64, rank: usize, source: &str) {
        self.total_queries += 1;
        self.relevance_scores.push(relevance);
        self.rank_positions.push(rank);
        
        let stats = self.source_stats.entry(source.to_string()).or_default();
        stats.queries += 1;
        if relevance > 0.5 { stats.hits += 1; }
        stats.avg_rank = (stats.avg_rank * (stats.queries - 1) as f64 + rank as f64) / stats.queries as f64;
    }

    pub fn precision_at_k(&self, k: usize) -> f64 {
        if self.total_queries == 0 { return 0.0; }
        let hits: usize = self.rank_positions.iter().filter(|&&r| r <= k).count();
        hits as f64 / self.total_queries as f64
    }

    pub fn mrr(&self) -> f64 {
        if self.rank_positions.is_empty() { return 0.0; }
        let reciprocal_sum: f64 = self.rank_positions.iter().map(|&r| 1.0 / r as f64).sum();
        reciprocal_sum / self.rank_positions.len() as f64
    }

    pub fn avg_relevance(&self) -> f64 {
        if self.relevance_scores.is_empty() { return 0.0; }
        self.relevance_scores.iter().sum::<f64>() / self.relevance_scores.len() as f64
    }
}

impl Default for RetrievalMetrics {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_basic() {
        let mut m = RetrievalMetrics::new();
        m.record_query(0.9, 1, "vector");
        m.record_query(0.7, 3, "text");
        m.record_query(0.8, 1, "graph");
        
        assert_eq!(m.total_queries, 3);
        assert!(m.precision_at_k(2) > 0.0);
        assert!(m.mrr() > 0.0);
        assert!(m.avg_relevance() > 0.7);
    }
}
