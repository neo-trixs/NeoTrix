//! ReasoningEngine — Complete module with all methods needed by submodules

mod cognitive_observer;
pub mod engine_core;

pub use engine_core::{
    CostRecord, EngineMetrics, ReasoningEngine, ReasoningStats, MAX_COST_LOG, MAX_TRACES,
};
pub use cognitive_observer::{BlindSpotKind, CognitiveBlindSpot, CognitiveEye, CognitiveSnapshot};
