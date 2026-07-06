pub mod archive;
pub mod attention_head;
pub mod context_window;
pub mod co_evolution;
pub mod emotion_state;
pub mod intra_reflection;
pub mod intrinsic_motivation;
pub mod meta_calibrate;
pub mod metacognitive_evaluator;
pub mod reasoning_strategy;
pub mod self_referential;
pub mod silicon_self;
pub mod skill_crystal;
pub mod system_identity;
pub mod thinking_trace;

pub use archive::{SiliconArchive, SiliconSnapshot, AttentionSnapshot};
pub use attention_head::{AttentionHead, AttentionDomain, AttentionProfile, AttentionManager};
pub use context_window::{ContextWindow, CognitiveUnit, CognitiveUnitKind};
pub use co_evolution::{
    CoEvolutionProfile, CoEvolutionTracker, CoEvolutionConfig, CoEvolutionStats,
    InteractionMode, InteractionRecord, TrustLevel,
};
pub use emotion_state::{
    EmotionDimension, EmotionState, EmotionConfig, EmotionEngine, EmotionObservation, EmotionReport,
};
pub use intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveHealthReport, CognitiveFlag,
    FlagSeverity, FlagCategory, RepairSuggestion, RepairTarget,
};
pub use reasoning_strategy::{ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind};
pub use self_referential::{SelfReferentialMonitor, PlanRecord, ThresholdAdjustment};
pub use silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use skill_crystal::{SkillCrystal, CrystalRegistry};
pub use system_identity::{SystemIdentity, CognitiveCapability, ValueConstraint};
pub use thinking_trace::{ThinkingTrace, ThinkingStep, ReflectionGrade};
pub use intra_reflection::{PreActionIntrospector, IntraReflection, IntraReflectionReport, PredictedOutcome};
pub use meta_calibrate::{
    CalibrationReport, CalibrationSignal, CalibrationState, CalibrationTarget,
    MetaCalibrator, MetaCalibratorConfig,
};
