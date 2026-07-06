pub mod archive;
pub mod attention_head;
pub mod context_window;
pub mod intrinsic_motivation;
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
