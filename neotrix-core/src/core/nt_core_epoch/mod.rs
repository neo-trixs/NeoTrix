//! Bridge: re-exports epoch types from `neotrix_types::core::epoch`.

pub use neotrix_types::core::epoch::{
    all_frameworks, create_framework, default_router_bias, evaluate_in_epoch, initial_state_for,
    ontology_for, ActivationRecord, CognitiveFramework, DimensionDef, EarthEpoch, FrameworkRoute,
};
