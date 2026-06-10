//! ReasoningEngine 模块 — 从 reasoning_engine.rs 拆分

mod engine_core;
mod reasoning;
mod observer;
mod cognitive_observer;
mod advanced;
mod internal;
mod distill_ext;
mod prd_gen;
pub mod reasoning_distiller;
mod markov_check;
#[cfg(test)]
mod tests;

pub use engine_core::{ReasoningEngine, CostRecord, ReasoningStats, EngineMetrics};
pub use cognitive_observer::{CognitiveEye, CognitiveBlindSpot, BlindSpotKind, CognitiveSnapshot};
pub use reasoning_distiller::{ReasoningDistiller, LlmReasoningPattern, LlmApproachType, ResponseStructure};
pub use markov_check::MarkovCheck;

pub mod context_manager;
pub mod disclosure;
