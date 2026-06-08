//! # NeoTrix Core — 纯理论/数据模型层
//!
//! 零外部依赖层，仅包含核心数据结构和 trait 定义。
//! 不依赖 tokio、reqwest、wgpu 等运行时/IO 库。
//!
//! ## 子模块
//!
//! - `nt_core_cap` — CapabilityVector（23 维能力向量 + 扩展维度 + provenance）
//! - `knowledge` — KnowledgeSource（6 内置来源 + KnowledgeProvider trait）
//! - `edit` — MicroEdit, SelfEdit（能力向量编辑操作）
//! - `nt_core_bank` — ReasoningBank, ReasoningMemory, TemporalContext
//! - `nt_core_ssm` — SelectiveState, SelectableOperator（Mamba SSM 状态空间模型）
//! - `nt_core_absorb` — AbsorbValidator trait（吸收验证）
//! - `nt_core_iter` — SelfIteration trait（自迭代循环抽象）
//! - `nt_core_meta` — **Layer 0**: 项目元认知 — 自知之明系统
//!   - `self_model` — SelfModel: 项目状态全景模型
//!   - `scanner` — CodeScanner: 文件级静态扫描
//!   - `monitor` — MetaMonitor: 持续健康监控
//!   - `weakness` — WeaknessAnalyzer: 弱点/技术债检测
//!   - `planner` — EvolutionPlanner: 进化路径规划
//!   - `metacognition_loop` — MetaCognitiveLoop: 元认知循环
//! - `nt_core_self` — **Layer 0.5**: 硅基生命思维模型 — LLM 认知架构自我建模
//!   - `silicon_self` — SiliconSelfModel: 思维自我全景模型
//!   - `context_window` — ContextWindow: 上下文窗口
//!   - `attention_head` — AttentionHead: 注意力头
//!   - `system_identity` — SystemIdentity: 自我身份/价值观
//!   - `reasoning_strategy` — ReasoningStrategy: 推理策略注册表
//!   - `thinking_trace` — ThinkingTrace: 思维轨迹记录

pub mod nt_core_cap;
pub mod nt_core_knowledge;
pub mod nt_core_edit;
pub mod nt_core_bank;
pub mod nt_core_event;
pub mod nt_core_ssm;
pub mod nt_core_iter_agent;
pub mod nt_core_absorb;
pub mod nt_core_iter;
pub mod nt_core_traits;
pub mod nt_core_hcube;
pub mod nt_core_graph;
pub mod nt_core_gwt;
pub mod nt_core_aware;
pub mod nt_core_accessor;
pub mod nt_core_meta;
pub mod nt_core_self;
pub mod nt_core_sense;
pub mod nt_core_embed;
pub mod nt_core_e8;
pub mod nt_core_hex;
pub mod nt_core_observer;
pub mod nt_core_policy;
pub mod nt_core_walsh;
pub mod nt_core_crt;
pub mod nt_core_kron;
pub mod nt_core_ws;
pub mod nt_core_router;
pub mod nt_core_wbmem;
pub mod nt_core_sigreg;
pub mod nt_core_jepa;
pub mod nt_core_td;
pub mod nt_core_conn;
pub mod nt_core_arch;
pub mod nt_core_epoch;
pub mod nt_core_mcp;
#[cfg(feature = "rkyv-storage")]
pub mod nt_core_rkyv;

pub mod nt_core_abstr;
pub mod nt_core_cdwm;
pub mod nt_core_prm;
pub mod nt_core_iface;
pub mod nt_core_pred;
pub mod nt_core_consciousness;

// Re-export consciousness types to core layer
pub use nt_core_consciousness::{
    FirstPersonRef, SpeciousPresent, ConsciousnessStream,
    InnerCritic, CritiqueResult, CognitiveLoadMonitor, ThinkingMode,
    ConsciousnessAwakening, AwakeningReport,
    VsaOrigin, VsaSelfCategory, VsaWorldCategory, VsaTagged,
};

// Re-export 主要类型到 core 层顶层
pub use nt_core_cap::CapabilityVector;
pub use nt_core_knowledge::{KnowledgeSource, KnowledgeProvider, TaskType, RewardSource, SourceAccessTracker, SourceAccessRecord};
pub use nt_core_accessor::{Accessor, AccessionReport, SourceType, UrlAccessor};
pub use nt_core_edit::{SelfEdit, MicroEdit, ToolCall};
pub use nt_core_bank::{ReasoningBank, ReasoningMemory, TemporalContext, MemoryTier, MemoryLifecycle, ReasoningBankStats};
pub use nt_core_ssm::{SelectiveState, SelectableOperator, SparseMatrix, ConsciousnessTier, SemanticType, SemanticBlock};
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_iter::SelfIteration;
pub use nt_core_traits::{MemoryProvider, RichMemoryProvider, AgentExecutor, SessionProvider, BrainProvider, EngineProvider, SealResult};
pub use nt_core_graph::{HyperGraph, HyperNode, HyperEdge, HyperNodeType, EdgeRelation};

// Re-export resonance types
pub use nt_core_gwt::resonance::{
    ResonanceMatrix, ResonanceReport, MODULE_COUNT,
    resonate_and_select, resonate_cycle, default_specialist_states,
    resonate_cycle_with_diversity, diversity_inject, compute_semantic_entropy,
    DIVERSITY_MIN_ENTROPY, DIVERSITY_NOISE_AMPLITUDE,
    RESONANCE_THRESHOLD,
};

// Re-export E8 reasoning types
pub use nt_core_hex::{
    ReasoningHexagram, MetaState, FullReasoningState, ModeFit, ReasoningPath,
    ReasoningApproach, ProblemDomain,
    all_reasoning_states, optimal_starting_mode, rank_modes_for_task, strategy_matrix,
    evolve_strategy_entry,
    MODE_NAMES, MODE_DESCRIPTIONS, MODE_TASKS,
};
pub use nt_core_policy::{E8Outcome, E8TransitionLearner, E8Policy, NUM_E8_FACTORS};
pub use nt_core_prm::ProcessRewardLearner;
pub use nt_core_observer::{OneObserver, ObserverReport};
pub use nt_core_prm::{
    AgentTrajectory, TrajectoryStep, ProcessScore, ScoredCriterion,
    CoachContext, Coach, TrajectoryCollector, HeuristicCoach,
};
pub use nt_core_gwt::pipeline::{
    PipelineRole, PipelineStage, PipelineSpec, PipelineStepResult, PipelineResult,
    PipelineExecutor, PipelineHandler, CreditArbiter,
};
pub use nt_core_crt::{CrtTimeScale, CrtPlan};
pub use nt_core_ws::WORKSPACE_MANAGER;

// Re-export metacognition types
pub use nt_core_meta::self_model::{
    SelfModel, ModuleInfo, FileInfo, DepGraph, DepEdge, DepKind,
    TestCoverage, CompilationHealth, TechDebtInventory, TechDebtItem,
    TechDebtKind, DebtSeverity, EvolutionEvent, EventKind,
    ComponentMap, ComponentNode,
};
pub use nt_core_meta::scanner::CodeScanner;
pub use nt_core_meta::monitor::{MetaMonitor, MetaAlert, AlertSeverity, HealthCheck, HealthTrend};
pub use nt_core_meta::weakness::{WeaknessAnalyzer, Weakness, WeaknessReport, WeaknessSummary};
pub use nt_core_meta::planner::{EvolutionPlanner, PlannedEvolution, ImpactEstimate, RiskLevel, EvolutionAction, ActionStatus, MetaGoal, MetaGoalBridge};
pub use nt_core_meta::metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use nt_core_meta::knowledge_gap_detector::{KnowledgeGapDetector, KnowledgeGap, GapReport, GapCluster, GapCategory};

// Re-export thinking_model types
pub use nt_core_self::archive::{SiliconArchive, SiliconSnapshot, AttentionSnapshot};
pub use nt_core_self::attention_head::{AttentionHead, AttentionDomain, AttentionProfile, AttentionManager};
pub use nt_core_self::context_window::{ContextWindow, CognitiveUnit, CognitiveUnitKind};
pub use nt_core_self::intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use nt_core_self::metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveHealthReport, CognitiveFlag,
    FlagSeverity, FlagCategory, RepairSuggestion, RepairTarget,
};
pub use nt_core_self::reasoning_strategy::{ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind};
pub use nt_core_self::self_referential::{SelfReferentialMonitor, PlanRecord, ThresholdAdjustment};
pub use nt_core_self::silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use nt_core_self::skill_crystal::{SkillCrystal, CrystalRegistry};
pub use nt_core_self::system_identity::{SystemIdentity, CognitiveCapability, ValueConstraint};
pub use nt_core_self::thinking_trace::{ThinkingTrace, ThinkingStep, ReflectionGrade};
pub use nt_core_self::intra_reflection::{PreActionIntrospector, IntraReflection, IntraReflectionReport, PredictedOutcome};

pub use nt_core_arch::ArchitectAgent;
pub use nt_core_mcp::McpServer;

pub use nt_core_abstr::{ContrastiveAbstraction, AbstractState, AbstractTransitionMatrix};
pub use nt_core_cdwm::{CDWM, EnvironmentPathway, InterventionPathway};
