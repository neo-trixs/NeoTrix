//! 代码符号索引 / 混合检索 — D3 兼容层
//!
//! 本体 (纯逻辑, 仅 std + serde + regex) 已下沉至 core `nt_core_code_search`;
//! 本模块 re-export 保持 `neotrix::nt_world_code_search::*` 调用方路径不变。

pub use crate::core::nt_core_code_search::{
    reciprocal_rank_fusion, CodeSearchEngine, CodeSearchResult, ImpactDepth, ImpactResult,
    RankedHit, RrfHit, SymbolIndex, SymbolRecord, RRF_K,
};