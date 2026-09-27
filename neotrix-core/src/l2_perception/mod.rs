pub mod nt_judgment;
pub mod nt_world;
pub use crate::l1_action::nt_core_llm;

// 从 core/ 迁移的 L2 模块
pub mod nt_core_e8;
pub mod nt_core_e8_predictor;
pub mod nt_core_e8_vsa;
pub mod nt_core_hcube;
pub mod nt_core_sense;
pub mod nt_core_knowledge;
pub mod nt_core_vector_store;
pub mod nt_core_code_search;
/// EVO-05 有界符号调用图（symbols/calls/impact/explain，全有界 50）
pub mod nt_code_graph;
/// EVO-12 情报profile管线＋配对收益（rubric/去重/配额/PairedGap）
pub mod nt_intel_digest;
pub mod nt_routing;
pub mod nt_web_perception;

