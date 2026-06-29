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
//! - `nt_core_identity` — **Layer 0.6**: 自身特性记忆与LLM定位剥离
//!   - `identity_core` — IdentityCore: 持久身份（self_vsa、人格特质、价值观）
//!   - `self_reasoner` — SelfReasoner: VSA空间内部推理（无需LLM）
//!   - `coproc_bridge` — CoprocessorBridge: LLM外挂管理 + 经验反哺蒸馏
//! - `nt_core_self` — **Layer 0.5**: 硅基生命思维模型 — LLM 认知架构自我建模
//!   - `silicon_self` — SiliconSelfModel: 思维自我全景模型
//!   - `context_window` — ContextWindow: 上下文窗口
//!   - `attention_head` — AttentionHead: 注意力头
//!   - `system_identity` — SystemIdentity: 自我身份/价值观
//!   - `reasoning_strategy` — ReasoningStrategy: 推理策略注册表
//!   - `thinking_trace` — ThinkingTrace: 思维轨迹记录

pub mod nt_core_absorb;
pub mod nt_core_aware;
pub mod nt_core_bank;
pub mod nt_core_cap;
pub mod nt_core_e8;
pub mod nt_core_edit;
pub mod nt_core_embed;
pub mod nt_core_error;
pub mod nt_core_graph;
pub mod nt_core_gwt;
pub mod nt_core_hcube;
pub mod nt_core_hdlib;
pub mod nt_core_iter;
pub mod nt_core_knowledge;
pub mod nt_core_meta;
pub mod nt_core_self;
pub mod nt_core_sense;
pub mod nt_core_shared_types;
pub mod nt_core_ssm;
pub mod nt_core_traits;
pub mod nt_core_util;
pub use nt_core_error::CoreError;
pub mod nt_core_crt;
pub mod nt_core_e8_model;
pub mod nt_core_economic;
pub mod nt_core_hex;
pub mod nt_core_identity;
pub mod nt_core_infer;
pub mod nt_core_inference;
pub mod nt_core_kron;
pub mod nt_core_observer;
pub mod nt_core_policy;
pub mod nt_core_spatial;
pub mod nt_core_walsh;
pub use crate::neotrix::nt_io_router as nt_core_router;
pub use crate::neotrix::nt_memory_ws as nt_core_ws;
pub mod nt_core_sigreg;
pub mod nt_core_td;
pub mod nt_core_time;
pub mod nt_core_wbmem;
pub use crate::neotrix::nt_agent_arch as nt_core_arch;
pub use crate::neotrix::nt_io_conn as nt_core_conn;
pub mod nt_core_epoch;
pub mod nt_core_audio;
pub mod nt_core_aura;
pub mod nt_core_graceful;
pub mod nt_core_consciousness;
pub mod nt_core_context;
pub mod nt_core_negentropy;
pub mod nt_core_prm;
pub use crate::neotrix::nt_io_design_token as nt_core_design_token;
pub use crate::neotrix::nt_memory_vector_store as nt_core_vector_store;
pub mod nt_core_codegen;
pub mod nt_core_experience;
pub mod nt_core_input;
pub mod nt_core_self_evolution;
pub mod nt_core_self_modify;
pub use crate::neotrix::nt_io_network as nt_core_network;
pub mod nt_core_ctm;
pub mod nt_core_reasoning;
pub use crate::neotrix::nt_shield_protect as nt_core_protect;
pub mod nt_core_loop;
pub mod nt_core_scheduler;
pub use crate::neotrix::nt_agent_core as nt_core_agent;
pub use crate::neotrix::nt_memory_storage as nt_core_storage;
pub mod nt_core_language;
pub mod nt_core_source_cognition;
pub use crate::neotrix::nt_agent_hive as nt_core_hive;
pub use crate::neotrix::nt_io_llm as nt_core_llm;
pub use crate::neotrix::nt_io_shutdown as nt_core_shutdown;
pub use crate::neotrix::nt_memory_session as nt_core_session;
pub use crate::neotrix::nt_world_search as nt_core_search;
pub use crate::neotrix::nt_world_translate as nt_core_translate;
pub use crate::neotrix::nt_world_vision as nt_core_vision;
pub use crate::neotrix::nt_world_document as nt_core_document;
pub mod nt_core_health;
pub mod nt_core_idempotency;
pub use crate::neotrix::nt_io_output as nt_core_output;
pub mod nt_core_data_types;
pub mod nt_core_metering;
pub mod nt_core_ratelimit;
pub mod nt_core_value_system;
pub use crate::neotrix::nt_io_llm_provider as nt_core_llm_provider;
pub mod nt_core_self_org;
pub use crate::neotrix::nt_memory_wal as wal;
pub mod self_measure;
pub mod self_model;
pub mod skill;
pub use crate::neotrix::nt_agent_plugin as nt_core_plugin;
pub use crate::neotrix::nt_io_llm_router as nt_core_llm_router;
pub mod nt_core_adversarial;
pub mod nt_core_avsad;
pub mod nt_core_discovery;
pub mod nt_core_emotional_memory;
pub mod nt_core_file_index;
pub mod nt_core_governance;
pub mod nt_core_prediction;
pub mod nt_core_truth;
pub use crate::neotrix::nt_act_trading as nt_core_trading;
pub use crate::neotrix::nt_io_tokenopt as nt_core_tokenopt;
pub mod nt_core_fep_iit;
pub mod nt_core_iit_phi;
pub mod nt_core_sandbox;
pub mod nt_core_world_model;

// ── F1 dedup generics ──
pub use nt_core_data_types::ToolResult;
pub use nt_core_util::{
    unix_now_ms, unix_now_nanos, unix_now_secs, TOR_CONTROL_ADDR, TOR_CONTROL_PORT, TOR_SOCKS_ADDR,
    TOR_SOCKS_PORT,
};

// Re-export consciousness types to core layer
pub use nt_core_consciousness::{
    AwakeningReport, CognitiveLoadMonitor, ConsciousnessAwakening, ConsciousnessStream,
    CritiqueResult, FirstPersonRef, InnerCritic, SpeciousPresent, ThinkingMode, VsaOrigin,
    VsaSelfCategory, VsaTagged, VsaWorldCategory,
};

// Re-export 主要类型到 core 层顶层
pub use nt_core_cap::CapabilityVector;
pub use nt_core_knowledge::{
    AngleSelector, Audience, Modality, MultimodalStoryteller, Story, StoryPlan, StoryRenderer,
};
pub use nt_core_knowledge::{
    Claim, EvidenceInspector, EvidenceVerificationResult, VerifiabilityGate, VerificationStatus,
};
pub use nt_core_knowledge::{
    KnowledgeProvider, KnowledgeSource, RewardSource, SourceAccessRecord, SourceAccessTracker,
    TaskType,
};
// pub use nt_core_accessor::{Accessor, AccessionReport, UrlAccessor}; // dead module
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_bank::{
    MemoryLifecycle, MemoryTier, ReasoningBank, ReasoningBankStats, ReasoningMemory,
    TemporalContext,
};
pub use nt_core_edit::{MicroEdit, SelfEdit, ToolCall};
pub use nt_core_graph::{EdgeRelation, HyperEdge, HyperGraph, HyperNode, HyperNodeType};
pub use nt_core_iter::SelfIteration;
pub use nt_core_ssm::{
    ConsciousnessTier, SelectableOperator, SelectiveState, SemanticBlock, SemanticType,
    SparseMatrix,
};
pub use nt_core_traits::{
    AgentExecutor, BrainProvider, EngineProvider, MemoryProvider, RichMemoryProvider, SealResult,
    SessionProvider,
};

// Re-export resonance types
pub use nt_core_gwt::resonance::{
    compute_semantic_entropy, default_specialist_states, diversity_inject, resonate_and_select,
    resonate_cycle, resonate_cycle_with_diversity, ResonanceMatrix, ResonanceReport,
    DIVERSITY_MIN_ENTROPY, DIVERSITY_NOISE_AMPLITUDE, MODULE_COUNT, RESONANCE_THRESHOLD,
};

// Re-export E8 reasoning types
pub use nt_core_hex::{
    all_reasoning_states, evolve_strategy_entry, optimal_starting_mode, rank_modes_for_task,
    strategy_matrix, FullReasoningState, MetaState, ModeFit, ProblemDomain, ReasoningApproach,
    ReasoningHexagram, ReasoningPath, MODE_DESCRIPTIONS, MODE_NAMES, MODE_TASKS,
};
pub use nt_core_observer::{ObserverReport, OneObserver};
pub use nt_core_policy::{E8Outcome, E8Policy, E8TransitionLearner, NUM_E8_FACTORS};
pub use nt_core_prm::ProcessRewardLearner;
pub use nt_core_prm::{
    AgentTrajectory, Coach, CoachContext, HeuristicCoach, ProcessScore, ScoredCriterion,
    TrajectoryCollector, TrajectoryStep,
};

pub use nt_core_crt::{CrtPlan, CrtTimeScale};
pub use nt_core_ws::WORKSPACE_MANAGER;

// Re-export metacognition types
pub use nt_core_meta::audit::{
    AuditEngine, AuditFinding, AuditPhase, AuditReport, AuditStatus, FindingSeverity,
};
pub use nt_core_meta::knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
};
pub use nt_core_meta::metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use nt_core_meta::monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert, MetaMonitor};
pub use nt_core_meta::planner::{
    ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal, MetaGoalBridge,
    PlannedEvolution, RiskLevel,
};
pub use nt_core_meta::scanner::CodeScanner;
pub use nt_core_meta::self_model::{
    CompilationHealth, ComponentMap, ComponentNode, DebtSeverity, DepEdge, DepGraph, DepKind,
    EventKind, EvolutionEvent, FileInfo, ModuleInfo, SelfModel, TechDebtInventory, TechDebtItem,
    TechDebtKind, TestCoverage,
};
pub use nt_core_meta::weakness::{Weakness, WeaknessAnalyzer, WeaknessReport, WeaknessSummary};

// Re-export thinking_model types
pub use nt_core_self::archive::{AttentionSnapshot, SiliconArchive, SiliconSnapshot};
pub use nt_core_self::attention_head::{
    AttentionDomain, AttentionHead, AttentionManager, AttentionProfile,
};
pub use nt_core_self::context_window::{CognitiveUnit, CognitiveUnitKind, ContextWindow};
pub use nt_core_self::intra_reflection::{
    IntraReflection, IntraReflectionReport, PreActionIntrospector, PredictedOutcome,
};
pub use nt_core_self::intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use nt_core_self::metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport, FlagCategory, FlagSeverity,
    RepairSuggestion, RepairTarget,
};
pub use nt_core_self::reasoning_strategy::{
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
};
pub use nt_core_self::self_referential::{PlanRecord, SelfReferentialMonitor, ThresholdAdjustment};
pub use nt_core_self::silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use nt_core_self::skill_crystal::{CrystalRegistry, SkillCrystal};
pub use nt_core_self::system_identity::{CognitiveCapability, SystemIdentity, ValueConstraint};
pub use nt_core_self::thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};

pub use nt_core_arch::ArchitectAgent;


// pub use nt_core_abstr::{ContrastiveAbstraction, AbstractState, AbstractTransitionMatrix}; // dead module
// pub use nt_core_cdwm::{CDWM, EnvironmentPathway, InterventionPathway}; // dead module

// Re-export context types
pub use nt_core_context::context_gatherer::{
    ContextFragment, ContextGatherer, ContextSource, ContextSourceMeta, GatheredContext,
};
pub use nt_core_context::context_os::{ContextOS, ContextOSStats};
pub use nt_core_context::working_memory::{BindingOp, WorkingMemory, WorkingMemoryItem};
pub use nt_core_context::{
    AllocatedSlice, AssembledContext, BudgetSourceType, CompactionIntent, CompactionPriority,
    ContextBudget,
};

// Re-export experience (Phase 5) types
pub use nt_core_experience::{
    BridgeConfig,
    BridgeV2Stats,
    ConceptNode,
    ConsolidatedMemory,
    // consolidation_bridge
    ConsolidationBridgeV2,
    CurriculumGenerator,
    DifficultyLevel,
    DomainConfidence,
    EpistemicConfig,
    EpistemicSelfModel,
    // epistemic
    EpistemicState,
    ExperienceReflector,
    // policy_repair
    FailurePattern,
    FailureType,
    GeneratorConfig,
    // reflector
    Heuristic,
    HeuristicCategory,
    HeuristicFilter,
    PolicyRepairEngine,
    ReflectorConfig,
    RepairMode,
    RepairPolicy,
    SkillAccumulator,
    SkillComposition,
    SkillFilter,
    // curriculum
    TaskTemplate,
    // skill_acc
    VSASkill,
};

// Re-export self_harness (Self-Harness pattern)
pub use nt_core_experience::{
    HarnessProposal, HarnessProposer, HarnessWeakness, ProposalValidator, SelfHarnessEngine,
    WeaknessMiner,
};

// Re-export Phase 8 — Self-Referential Consciousness Core (SRCC) types
// Memory Physics
// Re-export Spectral VSA, Trigram Index, and E8 Cortical types
pub use nt_core_hcube::HrrBackend;
pub use nt_core_hcube::SpectralVSA;
pub use nt_core_hcube::TrigramInvertedIndex;
pub use nt_core_hcube::{
    e8_cortical_vsa_transform, CorticalCoord, E8CorticalMapping, CORTICAL_NEURON_COUNT,
};
pub use nt_core_hcube::{
    BandPassExpert, FrequencyBand, GraphLaplacian, HighPassExpert, LowPassExpert,
    MoSpectralExperts, SpectralExpert, SpectralFilter, SpectralNSR, SpectralRule,
};
pub use nt_core_hcube::{CerebellumResonator, CortexAdaptive, ResonanceMode, CBRNN};
pub use nt_core_hcube::{
    DefectConfig, E8ParticleSpectrum, E8TopologicalDefects, ForceType, HalfIntegerSpin,
    TopologicalCharge, WeylOrbit,
};
pub use nt_core_hcube::{E8FieldIntegrator, E8Lagrangian, Lattice3D, PDESolver};
pub use nt_core_hcube::{E8FieldSolver, FieldSolverConfig};
pub use nt_core_hcube::{SpatialAttentionGate, VSASpatialEncoder, Vec3D};
pub use nt_core_hcube::{SpectralDenoiser, WaveGeometricEmbed, WaveGeometricVSA};

// Re-export Agentic Search types
// Re-export Semantic File Index types
pub use nt_core_file_index::{
    classify_intent, compute_file_hash, fuse_results, ContentIndex, FileIndexState, FileQuery,
    MerkleNode, MerkleWatch, PathIndex, QueryEngine, QueryIntent, ScanResult, ScoredFile,
    StructureIndex,
};

pub use nt_core_search::{
    AgenticSearcher, HybridRetriever, RRFFuser, SearchBudget, SearchEvaluator, SearchPlan,
    SearchPlanner, SearchResultItem, SearchStrategy, SearchVerdict, SearchedDocument,
    SearchedDocumentCollection,
};

pub use nt_core_hcube::attractor_basin::{
    AttractorBasin, AttractorBasinDynamics, BasinStats, BasinType,
};
pub use nt_core_hcube::dream_consolidation::{
    ConsolidationPhase, DreamConfig, DreamConsolidation, DreamEvent, DreamPhase, DreamReport,
    NremConfig, RemConfig,
};
pub use nt_core_hcube::ebbinghaus_decay::{DecayConfig, EbbinghausDecay, MemoryTrace};
// Self-Referential Processing
pub use nt_core_consciousness::{
    CalibrationBin, CognitiveState, CognitiveStateIngestion, ConsciousnessEvolution,
    ConsciousnessMetrics, DimSnapshot, EmergentReasoningConfig, EmergentReasoningMode,
    EpistemicHonesty, EpistemicReport, HonestyConfig, IngestionConfig, MasterConsciousness,
    MasterConsciousnessConfig, ModeTransition, ModificationProposal, NarrativeThread, OCEANTrait,
    PersonalityConfig, PersonalityMatrix, ProofSearchConfig, ProofSearchSelfModification,
    ReasoningMode, ReconstructedNarrative, ReconstructiveConfig, ReconstructiveNarrative,
    ReflexiveConfig, ReflexiveUnit, SafetyLevel, SafetyVerificationResult, SelfModificationProof,
    ThreadType, TraitState,
};

// Re-export loop engineering types
pub use nt_core_loop::{
    CoverageReport, GoalRegistry, GraphStats, HandlerDiscovery, LoopEngine, LoopGoal,
    LoopGoalStatus, LoopPhase, LoopState, LoopStats, LoopVerifier, NodeGroup, PipelineConditions,
    PipelineGraph, PipelineNodeData, Verdict,
};

// Re-export scheduler types
pub use nt_core_scheduler::{
    default_scheduler, ContextGate, JobRunHistory, JobRunRecord, ScheduleType, ScheduledJob,
    SchedulerEngine, SchedulerStats,
};

// Re-export agent types
pub use nt_core_agent::{
    AgentCommunicationBus, AgentMessage, ByzantineConsensusLayer, CDPSessionManager,
    FactorMiner, QuantDataIngestion, RemoteAgentHost, TeamOrchestrator,
};

// Re-export translation types
pub use nt_core_translate::{
    BilingualEntry, BilingualLexicon, CleanupRule, Language, TextAnalysis, TextType,
    TranslationOutput, TranslationPipeline, TranslationResult, TranslationStrategy,
    TranslationeseDiagnosis, TranslationeseSymptom, VerificationScore, VsaTranslationEngine,
};

// Re-export self-protection types
pub use nt_core_protect::{
    install_panic_filter, obfuscate_str, reveal_bytes, sanitize_panic, strip_source_path,
    AttackAttempt, AttackSurface, AttackSurfaceScanner, DrillResult, EnvironmentValidator,
    FablePacket, FableRouterStats, FableRoutingDecision, FableTier, HoneypotCategory,
    HoneypotForest, HoneypotNode, ImprovementTracker, IntegrityGuard, IntegrityReport, Obfuscated,
    ProtectionStats, RedTeamSeverity, RedTeamingEngine, SafeError, SecurityDrillScheduler,
    SelfProtection, ThinkingBlock, VsaFableRouter,
};

// Re-export production infrastructure types
pub use nt_core_health::{
    CommandProbe, ConsciousnessDashboard, DashboardWeights, DependencyRegistry, HealthProbe,
    HealthReport, HealthStatus, SystemHealth, ToolProbe,
};
pub use nt_core_idempotency::{IdempotencyGuard, IdempotencyKey, SeenSet, SimpleBloom};
pub use nt_core_output::{
    JsonFormatter, MarkdownFormatter, OutputEntry, OutputFormat, OutputFormatter, OutputRouter,
    OutputTarget, TextFormatter,
};
pub use nt_core_ratelimit::{DomainRateLimiter, SlidingWindowRateLimiter, TokenBucket};
pub use nt_core_shutdown::{DropGuard, GracefulShutdown, ShutdownPhase, ShutdownSignal};

pub use nt_core_sandbox::{
    SandboxConfig as KernelSandboxConfig, SandboxLevel as KernelSandboxLevel,
};

// Re-export world model types
pub use nt_core_world_model::{
    ActionEmbedding, AdaptiveExitGate, DynamicsError, LoopedDynamics, SpectralConstraint,
    VsaLatentState,
};

// Re-export trading types
pub use nt_core_trading::{
    AssetClass, DrawdownMonitor, FusedMarketSignal, KellyCalculator, MarketRegime, OHLCVBar,
    OnchainSignal, OrderType, PortfolioSummary, Position, PositionSizer, RiskConfig, RiskManager,
    RiskSnapshot, SentimentSignal, SignalFusion, SignalGenerator, SignalSource,
    TechnicalIndicators, Ticker, Timeframe, TradeSide, TradingEngine, TradingSignal,
    TrendDirection, VarCalculator, VolatilityLevel,
};
