//! Bridge: re-exports epoch types from `neotrix_types::core::epoch`.

pub use neotrix_types::core::epoch::{
    EarthEpoch, DimensionDef, CognitiveFramework, FrameworkRoute, ActivationRecord,
    ontology_for, initial_state_for, default_router_bias,
    create_framework, all_frameworks, evaluate_in_epoch,
};
