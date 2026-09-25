//! nt_memory_search — hub (P3 God-file split).
//! 主文件只留 `pub mod` + `pub use`，行为零变更纯搬移。

pub mod nt_cache;
pub mod nt_engine;
pub mod nt_gate_optimizer;
pub mod nt_pure_fns;
pub mod nt_router;
pub mod nt_scoring;

pub use nt_cache::{
    build_materialized_neighbors, register_nt_memory_search_self_tests,
    MaterializedNeighborCache, MaterializedNeighborCacheSelfTest,
};
pub use nt_engine::KbSearchEngine;
pub use nt_gate_optimizer::{
    precision_gate, staleness_signal, CraniMEMGate, Diagnosis, Fts5OptimizerConfig, GoalContext,
    RetrievalEvalPoint, RetrievalEvolver, RetrievalTuning,
};
pub use nt_pure_fns::{get_related, hybrid_search, search_by_type, search_fts};
pub use nt_router::{SearchBridge, SearchRegistry, SearchRouter};
pub use nt_scoring::{
    confidence_score, cosine_f32, entity_graph_scores, fuse_signals, relational_score,
    temporal_score, SmartVectorScorer, SmartVectorScore, SmartVectorWeights,
};
