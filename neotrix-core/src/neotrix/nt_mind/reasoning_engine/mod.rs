//! ReasoningEngine 模块 — 从 reasoning_engine.rs 拆分

mod advanced;
mod cognitive_observer;
mod distill_ext;
mod engine_core;
mod internal;
mod markov_check;
mod observer;
mod prd_gen;
mod reasoning;
pub mod reasoning_distiller;
#[cfg(test)]
mod tests;

pub use cognitive_observer::{BlindSpotKind, CognitiveBlindSpot, CognitiveEye, CognitiveSnapshot};
pub use engine_core::{
    CostRecord, EngineMetrics, ReasoningEngine, ReasoningStats, MAX_COST_LOG, MAX_STATE_TRAJECTORY,
    MAX_TRACES,
};
pub use markov_check::MarkovCheck;
pub use reasoning_distiller::{
    LlmApproachType, LlmReasoningPattern, ReasoningDistiller, ResponseStructure,
};

pub mod context_manager;
pub mod disclosure;
