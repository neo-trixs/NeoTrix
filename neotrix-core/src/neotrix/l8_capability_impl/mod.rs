//! l8_capability_impl — L8 能力实现层
//!
//! 对齐 / 提示管理 / 检索 / 知识图谱 等能力落地实现。

pub mod nt_alignment;
pub mod nt_core_benchmark_suite;
pub mod nt_core_knowledge_graph;
pub mod nt_core_prompting;
pub mod nt_core_retrieval;

pub use nt_alignment::{AlignmentConfig, AlignmentCore, AlignmentStep};
pub use nt_core_knowledge_graph::{KGConfig, KGEdge, KGNode, KnowledgeGraph};
pub use nt_core_prompting::{PromptConfig, PromptManager, PromptTemplate};
pub use nt_core_retrieval::{RetrievalConfig, RetrievalEngine, RetrievalResult};
