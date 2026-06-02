//! # Sensory Perception & Conversation Awareness — Data Structures
//!
//! CORE layer: only enums, structs, traits. Zero external dependencies (serde is std here).
//!
//! Provides:
//! - `SensoryEventKind` / `SensoryEvent` / `SensoryMemory` — nt_world_sense perception pipeline
//! - `AttentionTrigger` / `TriggerMapping` — attention routing primitives
//! - `ConversationTurn` / `ConversationObserver` / `GodViewReport` — conversation awareness

mod sensory_types;
mod sensory_processing;
pub mod sensor_trait;

pub use sensory_types::*;
pub use sensor_trait::{Sensor, SensorSample};

#[cfg(test)]
mod tests;
