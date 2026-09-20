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
