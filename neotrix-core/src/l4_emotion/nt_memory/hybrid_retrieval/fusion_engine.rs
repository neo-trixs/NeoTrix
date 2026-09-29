#![forbid(unsafe_code)]

//! Reciprocal Rank Fusion (RRF) engine combining semantic, BM25, entity,
//! and temporal signals with configurable weights.

// 2026-09-29: 三份重复的 ScoredDoc 已统一到 `super::ScoredDoc`，
// 别名不再需要。此前这三行迫使融合函数的三个参数是三个**不同类型**
// （`&[SemScoredDoc]` / `&[BM25ScoredDoc]` / `&[EntScoredDoc]`），
// 字段完全一样却无法放进同一个 Vec。
use super::ScoredDoc;
use super::temporal_scoring::TemporalScorer;
use std::collections::HashMap;

const RRF_K: f64 = 60.0;

#[derive(Debug, Clone)]
pub struct FusedResult {
    pub id: String,
    pub score: f64,
}

/// Signal weights for fusion. Each weight ∈ [0, 1], scaled to sum to 1.0.
#[derive(Debug, Clone)]
pub struct FusionWeights {
    pub semantic: f32,
    pub bm25: f32,
    pub entity: f32,
    pub temporal: f32,
}

impl FusionWeights {
    pub fn new(semantic: f32, bm25: f32, entity: f32, temporal: f32) -> Self {
        let total = semantic + bm25 + entity + temporal;
        if total == 0.0 {
            return Self::balanced();
        }
        Self {
            semantic: semantic / total,
            bm25: bm25 / total,
            entity: entity / total,
            temporal: temporal / total,
        }
    }

    /// Equal weight to all four signals.
    pub fn balanced() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    /// Semantic-primary: higher weight on semantic similarity.
    pub fn semantic_heavy() -> Self {
        Self::new(2.0, 1.0, 0.5, 1.0)
    }

    /// Keyword-primary: higher weight on BM25.
    pub fn keyword_heavy() -> Self {
        Self::new(0.5, 2.0, 1.0, 1.0)
    }

    /// Temporal-primary: recency-focused.
    pub fn recency_focused() -> Self {
        Self::new(0.5, 0.5, 0.5, 3.0)
    }
}

pub struct FusionEngine {
    weights: FusionWeights,
    scorer: TemporalScorer,
}

impl FusionEngine {
    pub fn new(weights: FusionWeights) -> Self {
        Self {
            weights,
            scorer: TemporalScorer::default(),
        }
    }

    pub fn with_temporal(weights: FusionWeights, scorer: TemporalScorer) -> Self {
        Self { weights, scorer }
    }

    /// Fuse results from semantic, BM25, and entity signals using RRF.
    ///
    /// - `sem_results`: results from SemanticIndex::search
    /// - `bm25_results`: results from BM25Index::search
    /// - `entity_results`: results from EntityIndex::search
    /// - `timestamps`: (doc_id, timestamp) pairs for temporal scoring
    /// - `query_time`: current time for temporal decay
    /// - `top_k`: max results to return
    pub fn fuse(
        &self,
        sem_results: &[ScoredDoc],
        bm25_results: &[ScoredDoc],
        entity_results: &[ScoredDoc],
        timestamps: &[(String, i64)],
        query_time: i64,
        top_k: usize,
    ) -> Vec<FusedResult> {
        let ts_map: HashMap<&str, i64> = timestamps
            .iter()
            .map(|(id, ts)| (id.as_str(), *ts))
            .collect();

        let mut rrf_scores: HashMap<String, f64> = HashMap::new();

        self.accumulate_rrf(
            &mut rrf_scores,
            &sem_results
                .iter()
                .map(|r| (r.id.as_str(), r.score))
                .collect::<Vec<_>>(),
            self.weights.semantic as f64,
        );
        self.accumulate_rrf(
            &mut rrf_scores,
            &bm25_results
                .iter()
                .map(|r| (r.id.as_str(), r.score))
                .collect::<Vec<_>>(),
            self.weights.bm25 as f64,
        );
        self.accumulate_rrf(
            &mut rrf_scores,
            &entity_results
                .iter()
                .map(|r| (r.id.as_str(), r.score))
                .collect::<Vec<_>>(),
            self.weights.entity as f64,
        );

        if self.weights.temporal > 0.0 {
            for (doc_id, raw_rrf) in rrf_scores.iter_mut() {
                if let Some(&ts) = ts_map.get(doc_id.as_str()) {
                    let temporal_weight = self.scorer.temporal_boost(1.0, ts, query_time);
                    let blended = *raw_rrf * (1.0 - self.weights.temporal as f64)
                        + *raw_rrf * temporal_weight * self.weights.temporal as f64;
                    *raw_rrf = blended;
                }
            }
        }

        let mut results: Vec<FusedResult> = rrf_scores
            .into_iter()
            .map(|(id, score)| FusedResult { id, score })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(top_k);
        results
    }

    fn accumulate_rrf(
        &self,
        scores: &mut HashMap<String, f64>,
        ranked: &[(&str, f64)],
        weight: f64,
    ) {
        for (rank, (doc_id, _)) in ranked.iter().enumerate() {
            let rrf = 1.0 / (RRF_K + (rank + 1) as f64);
            *scores.entry(doc_id.to_string()).or_insert(0.0) += rrf * weight;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_weights_sum_to_one() {
        let w = FusionWeights::balanced();
        let sum = w.semantic + w.bm25 + w.entity + w.temporal;
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fuse_combines_signals() {
        let engine = FusionEngine::new(FusionWeights::balanced());
        let sem = vec![ScoredDoc {
            id: "a".into(),
            score: 0.9,
        }];
        let bm25 = vec![ScoredDoc {
            id: "b".into(),
            score: 0.8,
        }];
        let ent = vec![ScoredDoc {
            id: "a".into(),
            score: 0.7,
        }];
        let ts = vec![("a".into(), 1_000_000), ("b".into(), 1_000_000)];

        let results = engine.fuse(&sem, &bm25, &ent, &ts, 1_000_100, 5);
        assert!(!results.is_empty());
        let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    #[test]
    fn test_top_k_limits_output() {
        let engine = FusionEngine::new(FusionWeights::balanced());
        let sem: Vec<ScoredDoc> = (0..10)
            .map(|i| ScoredDoc {
                id: format!("d{}", i),
                score: 1.0 - i as f64 * 0.1,
            })
            .collect();
        let bm25: Vec<ScoredDoc> = (0..10)
            .map(|i| ScoredDoc {
                id: format!("d{}", i),
                score: 1.0 - i as f64 * 0.1,
            })
            .collect();
        let ent: Vec<ScoredDoc> = vec![];
        let ts: Vec<(String, i64)> = (0..10)
            .map(|i| (format!("d{}", i), 1_000_000 + i * 1000))
            .collect();

        let results = engine.fuse(&sem, &bm25, &ent, &ts, 1_000_100, 3);
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_rrf_k_constant() {
        assert!((RRF_K - 60.0).abs() < 1e-10);
    }

    #[test]
    fn test_recency_focused_increases_temporal_weight() {
        let rf = FusionWeights::recency_focused();
        assert!(rf.temporal > rf.semantic);
        assert!(rf.temporal > rf.bm25);
    }
}
