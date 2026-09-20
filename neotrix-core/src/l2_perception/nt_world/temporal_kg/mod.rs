//! NT-WORLD Temporal Knowledge Graph: 带时间维度的知识图谱
//!
//! 支持时间切片查询、关系时效性管理和 PPR 重要性排序。
//! 实体和关系都携带时间戳，支持 "某时刻的知识状态" 快照查询。

pub mod entity;
pub mod fact_extractor;
pub mod graph_traversal;
pub mod knowledge_graph;
pub mod pagerank;
pub mod relation;
pub mod temporal_query;

pub use entity::{extract_entities, Entity, EntityType};
pub use fact_extractor::FactExtractor;
pub use graph_traversal::GraphTraversal;
pub use knowledge_graph::KnowledgeGraph;
pub use pagerank::personalized_page_rank;
pub use relation::TemporalRelation;
pub use temporal_query::{QueryResult, TemporalQueryBuilder};
