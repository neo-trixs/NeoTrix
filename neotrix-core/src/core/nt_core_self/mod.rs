pub mod archive;
pub mod attention_head;
pub mod context_window;
pub mod emotion_state;
pub mod evolution_analysis;
pub mod intrinsic_motivation;
pub mod metacognitive_evaluator;
pub mod reasoning_strategy;
pub mod seal;
pub mod self_audit;
pub mod self_model;
pub mod self_referential;
pub mod session_log_antipattern;
pub mod silicon_self;
pub mod skill_crystal;
pub mod system_identity;
pub mod thinking_trace;

pub use archive::{AttentionSnapshot, SiliconArchive, SiliconSnapshot};
pub use attention_head::{AttentionDomain, AttentionHead, AttentionManager, AttentionProfile};
pub use context_window::{CognitiveUnit, CognitiveUnitKind, ContextWindow};
pub use emotion_state::{
    EmotionConfig, EmotionDimension, EmotionEngine, EmotionObservation, EmotionReport, EmotionState,
};
pub use evolution_analysis::{
    analyze_kb_health, print_report, record_meta_cognition_defect, store_report_to_kb, KbDefect,
    KbHealthReport,
};
pub use intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport, FlagCategory, FlagSeverity,
    RepairSuggestion, RepairTarget,
};
pub use reasoning_strategy::{ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind};
pub use self_model::{SelfModel, SelfState};
pub use self_referential::{PlanRecord, SelfReferentialMonitor, ThresholdAdjustment};
pub use session_log_antipattern::{Antipattern, AntipatternMatch, ContextHealth, SessionLogScanner};
pub use silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use skill_crystal::{CrystalRegistry, SkillCrystal};
pub use system_identity::{CognitiveCapability, SystemIdentity, ValueConstraint};
pub use thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};
