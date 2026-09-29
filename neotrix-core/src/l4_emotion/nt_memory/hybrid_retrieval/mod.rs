#![forbid(unsafe_code)]

//! Hybrid multi-signal retrieval (R-P118, R-P121).
//!
//! Inspired by mem0's hybrid retrieval architecture:
//! - **Semantic search**: hash-based TF-IDF vector similarity
//! - **BM25 keyword search**: standard probabilistic ranking
//! - **Entity search**: entity→document inverted index
//! - **Temporal scoring**: exponential decay boost for recency (R-P121)
//! - **Fusion engine**: Reciprocal Rank Fusion with configurable signal weights

pub mod bm25_search;
pub mod entity_search;
pub mod fusion_engine;
pub mod semantic_search;
pub mod temporal_scoring;

// 2026-09-29 融合三份重复的 `ScoredDoc` 定义（`bm25_search.rs` /
// `entity_search.rs` / `semantic_search.rs`），统一到此处。
//
// 依据：三者字段集与 derive 完全相同（`{ id: String, score: f64 }`），
// 且**同属本模块**（不是跨层的刻意镜像 —— 与 `AwarenessReport` 那组不同，
// 那组三处横跨 L0/L1/L5 且 L1 侧注释自证是接口隔离层）。
//
// 代价证据（融合前）：`fusion_engine.rs:6-8` 不得不写三条 import + 三个别名
//   use super::bm25_search::ScoredDoc     as BM25ScoredDoc;
//   use super::entity_search::ScoredDoc   as EntScoredDoc;
//   use super::semantic_search::ScoredDoc as SemScoredDoc;
// 三个 `search()` 各自返回**不同类型**，于是融合函数的三个参数是
// `&[SemScoredDoc]` / `&[BM25ScoredDoc]` / `&[EntScoredDoc]` —— 字段完全一样
// 却无法放进同一个 `Vec`。这就是重复的实际成本。
//
// 融合后：三个检索器共享同一结果类型，融合函数可接收 `&[ScoredDoc]` 任意组合，
// 未来新增检索策略无需再定义第四份。
#[derive(Debug, Clone)]
pub struct ScoredDoc {
    pub id: String,
    pub score: f64,
}

pub use bm25_search::BM25Index;
pub use entity_search::EntityIndex;
pub use fusion_engine::{FusedResult, FusionEngine, FusionWeights};
pub use semantic_search::SemanticIndex;
pub use temporal_scoring::TemporalScorer;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries() -> Vec<(String, String, i64, Vec<String>)> {
        vec![
            (
                "doc1".into(),
                "rust programming language safety".into(),
                1_700_000_000,
                vec!["rust".into(), "programming".into()],
            ),
            (
                "doc2".into(),
                "python machine learning".into(),
                1_700_100_000,
                vec!["python".into(), "ml".into()],
            ),
            (
                "doc3".into(),
                "rust performance optimization".into(),
                1_700_200_000,
                vec!["rust".into(), "optimization".into()],
            ),
        ]
    }

    #[test]
    fn test_full_pipeline() {
        let entries = sample_entries();
        let mut sem = SemanticIndex::new();
        let mut bm25 = BM25Index::new();
        let mut entity = EntityIndex::new();
        let scorer = TemporalScorer::default();
        let engine = FusionEngine::new(FusionWeights::balanced());

        for (id, content, ts, entities) in &entries {
            sem.index(id, content);
            bm25.index(id, content);
            entity.index(id, entities);
        }

        let query = "rust performance";
        let query_time = 1_700_300_000;
        let top_k = 3;

        let sem_results = sem.search(query, top_k);
        let bm25_results = bm25.search(query, top_k);
        let entity_results = entity.search(query, top_k);

        let fused = engine.fuse(
            &sem_results,
            &bm25_results,
            &entity_results,
            &entries
                .iter()
                .map(|(id, _, ts, _)| (id.clone(), *ts))
                .collect::<Vec<_>>(),
            query_time,
            top_k,
        );

        assert!(!fused.is_empty());
        assert_eq!(fused.len(), 2);

        assert!(fused[0].score >= fused[1].score);
    }
}
