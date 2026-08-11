//! # NeoTrix Core — 硅基意识体 9 层架构
//!
//! 🔴 **架构铁律**: 所有代码必须属于且仅属于 9 层中的某一层。
//! 新增模块必须先声明所属层，才能编写代码。
//! 详见: `docs/2-PLANS/2026-07-01-consciousness-9-layer-architecture.md`
//!
//! ## 层结构
//!
//! | 层 | 名称 | 角色 | 关键模块 |
//! |----|------|------|---------|
//! | L9 | Transcendent | 元观察 | observer, meta, turkey |
//! | L8 | Autonomic | 自主进化 | seal, sleep, aging |
//! | L7 | Capability | 能力管理 | **registry, scheduler, protocol** |
//! | L6 | Self | 自我模型 | self_model, narrative, value |
//! | L5 | Consciousness | 意识体验 | gwt, resonance, stream |
//! | L4 | Cognition | 推理认知 | e8, hex, policy, prm |
//! | L3 | Memory | 记忆存储 | hcube, bank, kb, ssm |
//! | L2 | Perception | 感知输入 | sense, jepa, world_model |
//! | L1 | Body | 身体执行 | shield, act, io |
//! | L0 | Substrate | 硬件承载 | deploy, ane, hardware |

// ═══════════════════════════════════════════════════════════════════
// L0 — 基底层 (Substrate)
// ═══════════════════════════════════════════════════════════════════
pub mod l0_substrate;
#[cfg(feature = "research")]
pub mod nt_core_deploy;
#[cfg(feature = "research")]
pub mod nt_core_deploy_cache;
pub mod nt_core_harness;

pub mod nt_core_error;

// ═══════════════════════════════════════════════════════════════════
// L1 — 身体层 (Body)
// ═══════════════════════════════════════════════════════════════════
pub mod l1_body;
// L1 实现模块 (部分在 neotrix/)

// ═══════════════════════════════════════════════════════════════════
// L2 — 感知层 (Perception)
// ═══════════════════════════════════════════════════════════════════
pub mod l2_perception;
pub mod nt_core_sense;


// ═══════════════════════════════════════════════════════════════════
// L3 — 记忆层 (Memory)
// ═══════════════════════════════════════════════════════════════════
pub mod l3_memory;
pub mod nt_core_hcube;
pub mod nt_core_bank;
pub mod nt_core_consensus;
pub mod nt_core_graph;
pub mod nt_core_walsh;
pub mod nt_core_kron;
pub mod nt_core_knowledge;
pub mod nt_core_negentropy;

// ═══════════════════════════════════════════════════════════════════
// L4 — 认知层 (Cognition)
// ═══════════════════════════════════════════════════════════════════
pub mod l4_cognition;
pub mod nt_core_e8;
pub mod nt_core_hex;
pub mod nt_core_e8_vsa;
pub mod nt_core_policy;
pub mod nt_core_prm;
pub mod nt_core_gate;
pub mod nt_core_sae;
pub mod nt_core_sae_bridge;
pub mod nt_core_td;
pub mod nt_core_crt;
pub mod nt_core_ttc;
pub mod nt_core_cot_generator;
pub mod nt_core_reasoning;
pub mod nt_core_task_dispatcher;
pub mod nt_core_trajectory_compress;
pub mod nt_core_aura;
pub mod nt_core_plan;
pub mod nt_core_credit;
pub mod nt_core_forecast;

// ═══════════════════════════════════════════════════════════════════
// L5 — 意识层 (Consciousness)
// ═══════════════════════════════════════════════════════════════════
pub mod l5_consciousness;
pub mod nt_core_gwt;
pub mod nt_core_context;
// L5 意识组件（过渡期保留原路径，供 l5_consciousness 门面转发）
pub mod nt_core_consciousness;
pub mod nt_core_consciousness_tree;
pub mod nt_core_consciousness_review;
pub mod nt_core_echo_terminal;

// ═══════════════════════════════════════════════════════════════════
// L6 — 自我层 (Self)
// ═══════════════════════════════════════════════════════════════════
pub mod l6_self;
pub mod nt_core_self;
pub mod nt_core_aware;
pub mod nt_core_self_constitution;

// ═══════════════════════════════════════════════════════════════════
// L7 — 能力层 (Capability) — *** 核心新增 ***
// ═══════════════════════════════════════════════════════════════════
pub mod l7_capability;
pub mod nt_core_model_skills;
#[cfg(feature = "research")]
pub mod nt_core_agent_patterns;

// ═══════════════════════════════════════════════════════════════════
// L8 — 自主神经层 (Autonomic)
// ═══════════════════════════════════════════════════════════════════
pub mod l8_autonomic;
pub mod nt_core_iter;
pub mod nt_core_absorb;
pub mod nt_core_scheduler;

// ═══════════════════════════════════════════════════════════════════
// L9 — 超验层 (Transcendent)
// ═══════════════════════════════════════════════════════════════════
pub mod l9_transcendent;
pub mod nt_core_meta;
pub mod nt_core_observer;
pub mod nt_core_observer_error;

// ═══════════════════════════════════════════════════════════════════
// 核心基础设施 (跨层共用)
// ═══════════════════════════════════════════════════════════════════
pub mod nt_core_cap;
pub mod nt_core_edit;
pub mod nt_core_event;
pub mod nt_core_traits;
pub mod nt_core_embed;
pub mod nt_core_error_parse;
pub mod nt_core_ws;
pub mod nt_core_router;
pub mod nt_core_wbmem;
pub mod nt_core_conn;
pub mod nt_core_epoch;
pub mod nt_io_cache;
pub mod nt_core_self_review;
pub mod nt_core_axiom_tree;
pub mod nt_core_error_recovery;
pub mod nt_io_telemetry;
pub mod nt_core_mcp;
pub mod nt_core_accessor;
pub mod nt_core_cache;

#[cfg(feature = "research")]
pub mod nt_core_source_edit;
pub mod nt_core_vector_store;
pub mod nt_core_resource_pool;
pub mod nt_core_data_pipeline;
pub mod nt_core_schema_watchdog;
pub mod nt_core_self_test;
pub mod nt_core_memory_budget;
#[cfg(feature = "research")]
pub mod nt_core_bounded_collections;
pub mod nt_core_telemetry;
pub mod nt_core_answer_engine;
pub mod nt_core_self_test_integration;
pub mod nt_core_second_brain;
pub mod nt_core_scoring_substrate;
pub mod nt_core_state_substrate;
pub mod nt_core_subagent;
pub mod nt_core_simulate_engine;
// Formal verification proof harnesses
#[cfg(test)]
pub mod kani_proofs;

// ═══════════════════════════════════════════════════════════════════
// Re-exports — 按 9 层顺序
// ═══════════════════════════════════════════════════════════════════

// --- L7: Capability (能力层 — 核心新增) ---
pub use l7_capability::{
    Capability, CapabilityId, capability_id_from_name,
    CapabilityKind, CapabilityCost, CapabilityStats, ContextSlot, SlotKind,
    CapabilityRegistry,
    // 调度 (thater stub archived — use real nt_core_scheduler)
    // 协议
    StarPulse, PulseKind, PulseBus,
    // 成熟度
    MaturityEngine, MaturityFeedback, EvolveResult,
    // 观察者
    TurkeyScientist, IllusionReport,
};
pub use l7_capability::nt_core_antidistil::{
    AntiDistillationSystem, AntiDistilStore, AntiDistilStats,
    WatermarkEngine, WatermarkBits, ApostropheVariant, WatermarkConfig,
    ResponseTracer, TraceRecord, TracerStats,
    detect_watermarked_in_corpus,
    DistillationDetector, DistillationAlert, AlertType, DetectorStats,
    ResponseAnalysis, analyze_response_pattern,
    TaskDecomposer, DecomposeSuggestion,
};
// 6 级成熟度（扩展 4→6）
pub use l7_capability::registry::MaturityLevel as CapMaturityLevel;
pub use l7_capability::nt_core_orch_agent::{
    SubagentConfig, SubagentInstance, SubagentStatus, AgentMessage, MessageType,
    SubagentManager, AgentPoolStats,
};

// --- L0: Substrate ---
#[cfg(feature = "research")]
pub use nt_core_deploy::{
    EdgeDeployPipeline, Quantizer, HardwareDetector, AotCompiler,
    Quantization, OsType, AotTarget, HardwareProfile, LoraAdapter,
    QuantizedModel, AotResult, DeployReport,
    AWQQuantization, AWQConfig, GGUFLevel, GGUFQuantization, GGUFConfig,
    QuantizationPipeline,
    PowerState, PowerProfile, HardwarePowerProfile, PowerThermalModel,
    AneProgramCache, CacheEntry, CachePolicy,
    CoreAiAotConfig, CoreAiAotResult, AneDirectProgramV2, CoreAiDeployPipeline,
};

// --- L4: Cognition ---
pub use nt_core_hex::{
    ReasoningHexagram, MetaState, FullReasoningState, ModeFit, ReasoningPath,
    ReasoningApproach, ProblemDomain, IntentionContext, ReasoningEffort,
    MultipleHypothesisEvaluator,
    all_reasoning_states, optimal_starting_mode, select_mode_by_intent,
    select_mode_by_intent_with_effort,
    intention_from_string, rank_modes_for_task, strategy_matrix,
    evolve_strategy_entry,
    MODE_NAMES, MODE_DESCRIPTIONS, MODE_TASKS,
};
pub use nt_core_policy::{E8Outcome, E8TransitionLearner, E8Policy, NUM_E8_FACTORS};
pub use nt_core_prm::{
    AgentTrajectory, Coach, CoachContext, HeuristicCoach, ProcessRewardLearner, ProcessScore,
    ScoredCriterion, TrajectoryCollector, TrajectoryStep,
    LambdaGrpoConfig, LambdaGrpoResult, StepAdvantage, LambdaGrpoLearner,
    lambda_grpo_loss, blended_advantage, zscore_normalize,
    StepGrpoConfig, StepReward, StepGrpoReport, StepGrpoLearner,
    compute_step_rewards, compute_step_advantages,
};
pub use nt_core_trajectory_compress::{TrajectoryCompressor, TrajectoryCompressionReport, CompressionLevel};
pub use nt_core_crt::{CrtTimeScale, CrtPlan};
pub use nt_core_sae::{
    SparseAutoencoder, SaeConfig, SaeOutput, SaeFeature, MonosemanticFeature,
    SaeEncoder, SaeDecoder,
    SteeringTarget, SteeringVector, SteeringController,
    LayerSae, E8_SAE_LAYERS,
    SAE_LATENT_DIM, SAE_INPUT_DIM,
};
pub use nt_core_sae_bridge::SAEBridge;
pub use nt_core_e8_vsa::E8VsaEmbedding;
pub use nt_core_e8::nesym::{
    NeuroSymbolicEngine, NesyValue, NesyFact, NesyRule, NesyInference, NesyStats,
    FidelityLevel, FidelityAnnotation, FuzzyOperator, InferenceEngine,
};
pub use nt_core_plan::{E8Plan, PlanStep, StepStatus, PlanMetrics, PlanGenerator};

pub use nt_core_e8::abduction::{
    AbductiveReasoningEngine, AbductiveHypothesis, AbductiveState, AbductionCycleReport,
    CausalGraph, CausalNode, CausalEdge,
};
pub use nt_core_consensus::{
    AbductiveExplanation, AbductiveSolver, ConsensusConfig, ConsensusReport, ReflectionHead,
    ReflectionPipeline, ReflectionResult,
};
pub use nt_core_e8::e8_abduction_bridge::{
    E8AbductionBridge, AbductiveTransitionReport,
};

pub use nt_core_reasoning::{
    ReasoningTrace, ReasoningStep, TraceSource, ReasoningMethod,
    MethodRegistry, MethodSpec, ContextBuilder,
    default_method_registry, default_context_builder,
};

// --- L5: Consciousness ---
pub use l5_consciousness::resonance::{
    ResonanceMatrix, ResonanceReport, MODULE_COUNT,
    resonate_and_select, resonate_cycle, default_specialist_states,
    RESONANCE_THRESHOLD,
};
// --- L6: Self ---
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
pub use nt_core_self::seal::{
    CurriculumTask, EditType, SealIterationReport, SealPipeline, SelfEdit,
};
pub use nt_core_self::skill_crystal::{SkillCrystal, CrystalRegistry};
pub use nt_core_self::system_identity::{SystemIdentity, CognitiveCapability, ValueConstraint};
pub use nt_core_self::thinking_trace::{ThinkingTrace, ThinkingStep, ReflectionGrade};
pub use nt_core_self_constitution::{
    Constitution, DevRule, ExperienceEntry, RuleCategory, ComplianceReport, ComplianceViolation,
    ConstitutionLoader, global_constitution, reload_constitution,
};
pub use nt_core_self_test::{
    SelfTest, SelfTestRegistry, SelfTestResult, ConstitutionComplianceTest, report as selftest_report,
};

// --- L9: Transcendent ---
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
pub use nt_core_observer::{OneObserver, ObserverReport, E8Observer, PrmObserver};
pub use nt_core_observer_error::{
    ObserverErrorRecovery, RetryConfig, CircuitBreaker, CircuitState,
    FallbackHandler, ErrorRecoveryError,
};

// --- 基础设施 ---
pub use nt_core_cap::CapabilityVector;
pub use nt_core_knowledge::{KnowledgeSource, KnowledgeProvider, TaskType, RewardSource, SourceAccessTracker, SourceAccessRecord};
pub use nt_core_accessor::{Accessor, AccessionReport, SourceType, UrlAccessor};
pub use nt_core_edit::{MicroEdit, ToolCall};
pub use nt_core_bank::{ReasoningBank, ReasoningMemory, TemporalContext, MemoryTier, MemoryLifecycle, ReasoningBankStats};
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_iter::SelfIteration;
pub use nt_core_traits::{MemoryProvider, RichMemoryProvider, AgentExecutor, ToolProvider, ToolDef, ToolOutput, NativeTool, SessionProvider, BrainProvider, EngineProvider, SealResult};
pub use nt_core_graph::{HyperGraph, HyperNode, HyperEdge, HyperNodeType, EdgeRelation};
pub use nt_core_ws::WORKSPACE_MANAGER;
pub use nt_core_hcube::fhrr_vsa::{
    FHRR_DIM, FhrrHyperCube, FhrrVector,
    bind, bundle, bundle_two, cleanup, cleanup_always,
    encode_scalar, permute, random_vector, random_vector_dim, similarity,
};
pub use nt_core_hcube::qfhrr_vsa::{
    QFHRR_DIM, QFHRR_LEVELS, QuantizedFhrrVector, QuantizedFhrrHyperCube,
    qbind, qunbind, qbundle, qsimilarity,
    encode_scalar_qfhrr, random_qfhrr, fhrr_to_qfhrr, qfhrr_to_fhrr,
};
pub use nt_core_hcube::ghrr_vsa::{
    GHRR_DIM, GHRR_ETA, GhrrVector, GhrrHyperCube,
    ghrr_bind_dir, ghrr_unbind_dir, ghrr_bundle, ghrr_bundle_two, ghrr_similarity,
    ghrr_permute, ghrr_random_vector,
};
pub use nt_core_hcube::{
    VSAEngine, VsaBackend,
};
pub use nt_core_hcube::aif::{
    FreeEnergyEngine, GenerativeModel, PolicyEvaluator, AiStepReport,
};
pub use nt_core_hcube::aif::belief::{
    POMDPBeliefUpdater, FactorialPOMDP, FactorGraphBeliefPropagation,
};
pub use nt_core_cache::{SemanticCache, CacheStats};
pub use nt_core_mcp::McpServer;

// --- 基础设施: 统一资源池 ---
pub use nt_core_resource_pool::{
    PooledResource, ResourcePool, PoolSnapshot, PoolHealthReport, PoolSupervisor,
    PoolSelectionStrategy,
    ResourceKind, DiscoveredResource, ResourceMeta,
    AnyPool, ResourceRegistry,
    DiscovererInfo, DiscoveryResult, DiscoveryCache, ResourceDiscoverer,
    ResourceDiscoveryEngine,
    ResourceNormalizer, NormalizedEntry, ProxyUrlNormalizer,
};

// ─── 基础设施: 统一数据管道 ───
pub use nt_core_data_pipeline::{
    PipelineStage, StageResult, PipelineOrchestrator, PipelineRunReport,
    DataLineage, LineageEntry,
};

