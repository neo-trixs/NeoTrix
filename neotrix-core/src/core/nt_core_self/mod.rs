pub mod affective_interface;
pub mod archive;
pub mod attention_head;
pub mod behavior_fsm;
pub mod context_window;
pub mod cuda_agent;
pub mod dynamic_params;
pub mod emotion_state;
pub mod evolution_analysis;
pub mod human_approval;
pub mod hyperframes;
pub mod intrinsic_motivation;
pub mod metacognitive_evaluator;
pub mod pilot_steering;
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
pub mod trace_evaluation;
pub mod wiki_skill;

pub use affective_interface::{
    au_to_emotion, AffectiveInterface, AffectiveReadout, EmpathyConfig, EmpathyStrategy, GuideMode,
    RelationshipConfig, RelationshipStage, RelationshipState, ResponseIntent, RhythmProfile,
    UserAffectConfig, UserAffectModel, UserAffectSnapshot, UserEmotion, VisualAffect,
    VisualAffectSource,
};
pub use archive::{AttentionSnapshot, SiliconArchive, SiliconSnapshot};
pub use attention_head::{AttentionDomain, AttentionHead, AttentionManager, AttentionProfile};
pub use behavior_fsm::{
    BehaviorFsmAnalyzer, FsmAnalysis, FsmBuilder, FsmModel, FsmState, FsmTransition, FailurePredictor,
    NextStepPredictor, TraceEntry, TraceResult,
};
pub use context_window::{CognitiveUnit, CognitiveUnitKind, ContextWindow};
pub use cuda_agent::{
    CudaAgentEnvironment, OptimizationConstraint, OptimizationResult, OptimizationStrategy,
    OptimizationTask, ConstraintType, PerformanceAnalyzer, RewardCalculator, StrategyManager,
    TaskStatus,
};
pub use emotion_state::{
    EmotionConfig, EmotionDimension, EmotionEngine, EmotionLabel, EmotionObservation, EmotionReport,
    EmotionState,
};
pub use evolution_analysis::{
    analyze_kb_health, print_report, record_meta_cognition_defect, store_report_to_kb, KbDefect,
    KbHealthReport,
};
pub use human_approval::{
    ApprovalDecision, ApprovalManager, ApprovalPolicy, ApprovalRequest, ApprovalRequestBuilder,
    ApprovalStats, ApprovalStatus, RiskLevel,
};
pub use hyperframes::{
    OutputFormat, ProjectStatus, RenderConfig, RenderEngine, RenderQuality, RenderResult,
    RenderStatus, TemplateCategory, TemplateSystem, VideoCodec, VideoProject, VideoScene,
    VideoTemplate,
};
pub use intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport, FlagCategory, FlagSeverity,
    RepairSuggestion, RepairTarget,
};
pub use pilot_steering::{
    FailurePattern, PilotSnapshot, PilotSupervisor, PilotSystem, PilotWorker, SharedMemory,
    SupervisorConfig, SupervisorDecision, TracePoint, WorkerState,
};
pub use reasoning_strategy::{ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind};
pub use self_model::{SelfModel, SelfState};
pub use self_referential::{PlanRecord, SelfReferentialMonitor, ThresholdAdjustment};
pub use session_log_antipattern::{Antipattern, AntipatternMatch, ContextHealth, SessionLogScanner};
pub use silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use skill_crystal::{CrystalRegistry, SkillCrystal};
pub use system_identity::{CognitiveCapability, SystemIdentity, ValueConstraint};
pub use thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};
pub use trace_evaluation::{
    AgentTrace, BenchmarkCase, BenchmarkResult, BenchmarkRunner, EvaluationDimension, EvaluationGrade,
    EvaluationReport, EvaluationResult, TraceEvaluator, TraceStep,
};
pub use wiki_skill::{
    AccumulatedKnowledge, CoEvolutionEvent, CoEvolutionRecord, ExecutableSkill, KnowledgeSkillLink,
    LinkType, RawExperience, SkillOutput, SkillParameter, WikiSkillConfig, WikiSkillKB,
};
