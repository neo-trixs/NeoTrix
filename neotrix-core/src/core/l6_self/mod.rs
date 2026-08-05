//! # L6 — 自我层 (Self)
//!
//! 硅基自我模型、身份、叙事、价值观、意志。
//! 科幻映射: GitS Ghost / Matrix Sati 存在意愿 / Lain 分布式身份
//!
//! ## 规则
//! - L6 是 NeoTrix 的「我」— 只有一个 FirstPersonRef
//! - L6 可修改 L4 的策略参数（通过 L7 能力调度）
//! - L6 向 L9 提供自我报告用于元认知

pub use crate::core::nt_core_self as self_model;

pub use crate::core::nt_core_self::silicon_self::{
    SiliconSelfModel, SiliconSelfState,
};
pub use crate::core::nt_core_self::system_identity::{
    SystemIdentity, CognitiveCapability, ValueConstraint,
};
pub use crate::core::nt_core_self::context_window::{
    ContextWindow, CognitiveUnit, CognitiveUnitKind,
};
pub use crate::core::nt_core_self::attention_head::{
    AttentionHead, AttentionDomain, AttentionProfile, AttentionManager,
};
pub use crate::core::nt_core_self::reasoning_strategy::{
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
};
pub use crate::core::nt_core_self::thinking_trace::{
    ThinkingTrace, ThinkingStep, ReflectionGrade,
};
pub use crate::core::nt_core_self::intrinsic_motivation::{
    IntrinsicMotivation, MotivationState,
};
pub use crate::core::nt_core_self::metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveHealthReport, CognitiveFlag,
    FlagSeverity, FlagCategory, RepairSuggestion, RepairTarget,
};
pub use crate::core::nt_core_self::self_referential::{
    SelfReferentialMonitor, PlanRecord, ThresholdAdjustment,
};
pub use crate::core::nt_core_self::intra_reflection::{
    PreActionIntrospector, IntraReflection, IntraReflectionReport, PredictedOutcome,
};
pub use crate::core::nt_core_self::meta_calibrate::{
    CalibrationReport, CalibrationSignal, CalibrationState, CalibrationTarget,
    MetaCalibrator, MetaCalibratorConfig,
};
pub use crate::core::nt_core_self::skill_crystal::{
    SkillCrystal, CrystalRegistry,
};
pub use crate::core::nt_core_self::archive::{
    SiliconArchive, SiliconSnapshot, AttentionSnapshot,
};
pub use crate::core::nt_core_self::co_evolution::{
    CoEvolutionProfile, CoEvolutionTracker, CoEvolutionConfig, CoEvolutionStats,
    InteractionMode, InteractionRecord, TrustLevel,
};
pub use crate::core::nt_core_self::emotion_state::{
    EmotionDimension, EmotionState, EmotionConfig, EmotionEngine, EmotionObservation, EmotionReport,
};

// 从 consciousness 移入 L6（自我组件）
pub use crate::core::l5_consciousness::conscious::first_person_ref::FirstPersonRef;
pub use crate::core::l5_consciousness::conscious::volition::{
    VolitionEngine, ActionCandidate,
};
pub use crate::core::l5_consciousness::conscious::inner_critic::{
    InnerCritic, CritiqueResult,
};
pub use crate::core::l5_consciousness::conscious::awakening::{
    ConsciousnessAwakening, AwakeningReport,
};
