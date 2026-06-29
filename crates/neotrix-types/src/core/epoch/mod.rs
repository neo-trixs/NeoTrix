//! # EarthEpoch Cognitive Frameworks
//!
//! Eight distinct cognitive frameworks, each representing a different
//! paradigm of Earth-perception in human civilization history.
//!
//! The system evolves by switching between these frameworks — not by
//! optimizing within a single one.

pub mod types;
pub mod definitions;

pub use types::{
    EarthEpoch, DimensionDef, CognitiveFramework, FrameworkRoute, ActivationRecord,
};
pub use definitions::{
    ontology_for, initial_state_for, default_router_bias,
    create_framework, all_frameworks, evaluate_in_epoch,
};
