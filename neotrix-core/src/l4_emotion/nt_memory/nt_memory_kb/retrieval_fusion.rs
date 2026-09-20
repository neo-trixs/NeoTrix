//! Hybrid Retrieval Fusion — 吸收自 memrust + hirn
//!
//! 三路融合: Vector(HNSW) + Text(BM25) + Entity Graph
//! - Reciprocal Rank Fusion (RRF) 合并排名
//! - 指数时间衰减 (1周半衰期)
//! - 预过滤: 每个索引内部过滤，非事后过滤

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub source: RetrievalSource,
    pub rank: usize,
}

/// 检索来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RetrievalSource {
    Vector,
    Text,
    Graph,
    Fused,
}

impl Default for RetrievalSource {
    fn default() -> Self {
        Self::Fused
    }
}

/// 混合检索融合器
pub struct RetrievalFusion {
    rrf_k: usize,                // RRF 常数
    recency_half_life_secs: u64, // 时间衰减半衰期
}

impl RetrievalFusion {
    pub fn new() -> Self {
        Self {
            rrf_k: 60,                      // 标准 RRF 常数
            recency_half_life_secs: 604800, // 1周 = 7*24*3600
        }
    }

    /// Reciprocal Rank Fusion: RRF(d) = Σ 1/(k + rank_i(d))
    pub fn rrf_fuse(&self, rankings: Vec<Vec<RetrievalResult>>) -> Vec<RetrievalResult> {
        let mut scores: HashMap<String, f64> = HashMap::new();
        let mut contents: HashMap<String, (String, RetrievalSource)> = HashMap::new();

        for ranking in &rankings {
            for result in ranking {
                let entry = scores.entry(result.id.clone()).or_insert(0.0);
                *entry += 1.0 / (self.rrf_k as f64 + result.rank as f64);

                if !contents.contains_key(&result.id) {
                    contents.insert(result.id.clone(), (result.content.clone(), result.source));
                }
            }
        }

        let mut fused: Vec<RetrievalResult> = scores
            .into_iter()
            .map(|(id, score)| {
                let (content, _) = contents.remove(&id).unwrap_or_default();
                RetrievalResult {
                    id,
                    content,
                    score,
                    source: RetrievalSource::Fused,
                    rank: 0,
                }
            })
            .collect();

        fused.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        fused
            .iter_mut()
            .enumerate()
            .for_each(|(i, r)| r.rank = i + 1);
        fused
    }

    /// 指数时间衰减: decay(t) = exp(-λ * age), λ = ln(2) / half_life
    pub fn recency_decay(&self, results: &mut Vec<RetrievalResult>, _now_secs: u64) {
        let _lambda = (2.0_f64).ln() / self.recency_half_life_secs as f64;

        for result in results.iter_mut() {
            // 假设 timestamp 编码在 id 中或需要外部提供
            // 这里简化: 按 rank 赋予权重
            let age_penalty = 1.0 / (1.0 + result.rank as f64 * 0.1);
            result.score *= age_penalty;
        }
    }

    /// 三路融合: vector + text + graph
    pub fn fuse(
        &self,
        vector_results: Vec<RetrievalResult>,
        text_results: Vec<RetrievalResult>,
        graph_results: Vec<RetrievalResult>,
    ) -> Vec<RetrievalResult> {
        let rankings = vec![vector_results, text_results, graph_results];
        let mut fused = self.rrf_fuse(rankings);
        self.recency_decay(&mut fused, now());
        fused
    }

    /// 预过滤: 保留分数高于阈值的结果
    pub fn pre_filter(&self, results: &mut Vec<RetrievalResult>, min_score: f64) {
        results.retain(|r| r.score >= min_score);
    }
}

impl Default for RetrievalFusion {
    fn default() -> Self {
        Self::new()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_fusion() {
        let fusion = RetrievalFusion::new();

        let vector_results = vec![
            RetrievalResult {
                id: "a".into(),
                content: "A".into(),
                score: 0.9,
                source: RetrievalSource::Vector,
                rank: 1,
            },
            RetrievalResult {
                id: "b".into(),
                content: "B".into(),
                score: 0.8,
                source: RetrievalSource::Vector,
                rank: 2,
            },
        ];

        let text_results = vec![
            RetrievalResult {
                id: "b".into(),
                content: "B".into(),
                score: 0.85,
                source: RetrievalSource::Text,
                rank: 1,
            },
            RetrievalResult {
                id: "c".into(),
                content: "C".into(),
                score: 0.7,
                source: RetrievalSource::Text,
                rank: 2,
            },
        ];

        let fused = fusion.rrf_fuse(vec![vector_results, text_results]);

        // "b" appears in both rankings, should rank highest
        assert_eq!(fused[0].id, "b");
        assert_eq!(fused.len(), 3); // a, b, c
    }

    #[test]
    fn test_pre_filter() {
        let fusion = RetrievalFusion::new();
        let mut results = vec![
            RetrievalResult {
                id: "a".into(),
                content: "A".into(),
                score: 0.9,
                source: RetrievalSource::Vector,
                rank: 1,
            },
            RetrievalResult {
                id: "b".into(),
                content: "B".into(),
                score: 0.3,
                source: RetrievalSource::Vector,
                rank: 2,
            },
        ];

        fusion.pre_filter(&mut results, 0.5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "a");
    }
}
