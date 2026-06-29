pub mod architecture_governor;
pub mod archive;
pub mod attention_head;
pub mod autonomy_harness;
pub mod cognitive_dashboard;
pub mod context_window;
pub mod evolution_trace;
pub mod experimentation;
pub mod intra_reflection;
pub mod intrinsic_motivation;
pub mod metacognitive_evaluator;
pub mod reasoning_strategy;
pub mod self_referential;
pub mod silicon_self;
pub mod skill_crystal;
pub mod skill_registry;
pub mod system_identity;
pub use experimentation::{Experiment, ExperimentDesigner, Hypothesis, Intervention};
pub mod learning_mechanics;
pub use learning_mechanics::{
    LearningMechanicsObservatory, LearningMechanicsReport, ObservableType,
};
pub mod temporal_attention_engine;
pub mod thinking_trace;

pub use archive::{AttentionSnapshot, SiliconArchive, SiliconSnapshot};
pub use attention_head::{AttentionDomain, AttentionHead, AttentionManager, AttentionProfile};
pub use context_window::{CognitiveUnit, CognitiveUnitKind, ContextWindow};
pub use evolution_trace::{
    EvolutionCausalGraph, EvolutionEvent, EvolutionEventType, EvolutionPredictor, EvolutionTrace,
};
pub use intra_reflection::{
    IntraReflection, IntraReflectionReport, PreActionIntrospector, PredictedOutcome,
};
pub use intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport, FlagCategory, FlagSeverity,
    RepairSuggestion, RepairTarget,
};
pub use reasoning_strategy::{ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind};
pub use self_referential::{PlanRecord, SelfReferentialMonitor, ThresholdAdjustment};
pub use silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use skill_crystal::{CrystalRegistry, SkillCrystal};
pub use system_identity::{CognitiveCapability, SystemIdentity, ValueConstraint};
pub use temporal_attention_engine::{
    TemporalAttentionBias, TemporalAttentionConfig, TemporalAttentionState, TemporalEntry,
};
pub use thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};
pub mod research_intuition;
pub use research_intuition::{IntuitionSignal, PastExperience, ResearchIntuition};
pub mod vibe_trainer;
pub use vibe_trainer::{DynamicsPrediction, TrainingConfig, TrainingOutcome, VibeTrainer};
pub mod toy_model_gen;
pub use toy_model_gen::{
    ToyConfig, ToyDataset, ToyModel, ToyModelGenerator, ToyModelResult, ToyModelType,
};
pub mod observables;
pub use observables::{ObservableCategory, ObservableId, ObservablesRegistry};
pub mod config_space;
pub use config_space::{
    ConfigParam, ConfigPoint, ConfigSpaceExplorer, ExplorationResult, SpaceReport,
};
pub mod intervention_hypothesis;
pub use intervention_hypothesis::{
    CausalLink, HypothesisType, InterventionHypothesis, InterventionHypothesisGenerator,
    InterventionPlan, InterventionScope,
};
pub mod verified_rsi;
pub use verified_rsi::{
    ProofStatus, ProposalId, RsiLog, RsiLogEntry, RsiVerifier, Specification, VerificationResult,
    VerifiedProposal, VerifiedRsiPipeline,
};
