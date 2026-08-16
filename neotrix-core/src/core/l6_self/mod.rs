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

pub use crate::core::nt_core_self::archive::{AttentionSnapshot, SiliconArchive, SiliconSnapshot};
pub use crate::core::nt_core_self::attention_head::{
    AttentionDomain, AttentionHead, AttentionManager, AttentionProfile,
};
pub use crate::core::nt_core_self::context_window::{
    CognitiveUnit, CognitiveUnitKind, ContextWindow,
};
pub use crate::core::nt_core_self::emotion_state::{
    EmotionConfig, EmotionDimension, EmotionEngine, EmotionObservation, EmotionReport, EmotionState,
};
pub use crate::core::nt_core_self::intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use crate::core::nt_core_self::metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport, FlagCategory, FlagSeverity,
    RepairSuggestion, RepairTarget,
};
pub use crate::core::nt_core_self::reasoning_strategy::{
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
};
pub use crate::core::nt_core_self::self_referential::{
    PlanRecord, SelfReferentialMonitor, ThresholdAdjustment,
};
pub use crate::core::nt_core_self::silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use crate::core::nt_core_self::skill_crystal::{CrystalRegistry, SkillCrystal};
pub use crate::core::nt_core_self::system_identity::{
    CognitiveCapability, SystemIdentity, ValueConstraint,
};
pub use crate::core::nt_core_self::thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};

// 从 consciousness 移入 L6（自我组件）
pub use crate::core::l5_consciousness::conscious::awakening::{
    AwakeningReport, ConsciousnessAwakening,
};
pub use crate::core::l5_consciousness::conscious::first_person_ref::FirstPersonRef;
pub use crate::core::l5_consciousness::conscious::inner_critic::{CritiqueResult, InnerCritic};
pub use crate::core::l5_consciousness::conscious::volition::{ActionCandidate, VolitionEngine};
