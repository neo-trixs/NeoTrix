// Legacy re-exports — backwards compatibility for consumers that used
// crate::l5_cognition::nt_core_consciousness::* paths.

pub use crate::source_hierarchy::{
    ContextMeta, KnowledgeLayer, MeaningMeta, PerceptionMeta, PerceptionSource, ProvenanceChain,
    SourceHierarchy, UpgradeRule, deduce_layer,
};

pub use crate::cognitive_load::{
    CognitiveLoadConfig, CognitiveLoadMonitor, ThinkingMode, COGNITIVE_LOAD_CONFIG,
};

pub use crate::bubble_wall::{area_law_r2, BubbleWall, TokenBill, estimate_tokens};

pub use crate::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged, VsaWorldCategory};

pub use crate::echo_terminal::{
    ChangeType, EchoBatchReport, EchoController, EchoFeatures, EchoPrmBridge, EchoTrajectory,
    FileChange, TerminalObservation,
};
