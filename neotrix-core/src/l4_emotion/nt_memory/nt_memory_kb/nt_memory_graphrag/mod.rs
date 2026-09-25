//! nt_memory_graphrag — GraphRAG 门面: 子模块重导出, 行为零变更.
//! 拆分: nt_types / nt_entity_graph / nt_lightrag_index / nt_store / nt_extractor.

pub mod nt_entity_graph;
pub mod nt_extractor;
pub mod nt_lightrag_index;
pub mod nt_store;
pub mod nt_types;

pub use nt_entity_graph::EntityGraph;
pub use nt_extractor::{stable_slice_document, ExtractionConfig, GraphExtractor};
pub use nt_lightrag_index::LightRagIndex;
pub use nt_store::GraphRagStore;
pub use nt_types::{
    Community, EntityNode, ExtractionMode, GlobalSummary, GraphQueryMode, GraphRagConfig,
    GraphRagStats, HybridResult, IncrementalChange, RelationEdge, SubgraphResult,
};

#[cfg(test)]
mod tests;
