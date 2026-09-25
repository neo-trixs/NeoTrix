//! ReasoningEngine — reason 管线立面 (P2 God-file split)。
//!
//! 原 `engine_core.rs` (2988 行) 按 reason 管线调用阶段拆为 6 模块。
//! 本文件只留 `pub mod` + `pub use`，行为零变更纯搬移。

pub mod nt_builders;
pub mod nt_context_cache;
pub mod nt_finalize_broadcast;
pub mod nt_prepare_call;
pub mod nt_prediction_fusion;
pub mod nt_reason_entry;
#[cfg(test)]
mod nt_engine_tests;

pub use nt_builders::{
    CostRecord, EngineMetrics, ReasoningEngine, ReasoningStats, CONTROL_TRAIN_BATCH,
    MAX_COST_LOG, MAX_KB_CACHE_ENTRIES, MAX_KB_INJECTION_TOKENS, MAX_TRACES,
};
pub use nt_finalize_broadcast::_split_response_into_steps;
pub use nt_prepare_call::_detect_refusal_response;
