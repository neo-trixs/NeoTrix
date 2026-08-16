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
pub mod nt_core_bank;
pub mod nt_core_consensus;
pub mod nt_core_graph;
pub mod nt_core_hcube;
pub mod nt_core_knowledge;
pub mod nt_core_kron;
pub mod nt_core_negentropy;
pub mod nt_core_walsh;

// ═══════════════════════════════════════════════════════════════════
// L4 — 认知层 (Cognition)
// ═══════════════════════════════════════════════════════════════════
pub mod l4_cognition;
pub mod nt_core_aura;
pub mod nt_core_cot_generator;
pub mod nt_core_credit;
pub mod nt_core_crt;
pub mod nt_core_e8;
pub mod nt_core_e8_predictor;
pub mod nt_core_e8_vsa;
pub mod nt_core_forecast;
pub mod nt_core_gate;
pub mod nt_core_hex;
pub mod nt_core_plan;
pub mod nt_core_policy;
pub mod nt_core_prm;
pub mod nt_core_reasoning;
pub mod nt_core_sae;
pub mod nt_core_sae_bridge;
pub mod nt_core_task_dispatcher;
pub mod nt_core_td;
pub mod nt_core_trajectory_compress;
pub mod nt_core_ttc;

// ═══════════════════════════════════════════════════════════════════
// L5 — 意识层 (Consciousness)
// ═══════════════════════════════════════════════════════════════════
pub mod l5_consciousness;
pub mod nt_core_context;
pub mod nt_core_dispatch;
pub mod nt_core_gwt;
// L5 意识组件（过渡期保留原路径，供 l5_consciousness 门面转发）
pub mod nt_core_consciousness;
pub mod nt_core_consciousness_core;
pub mod nt_core_consciousness_review;
pub mod nt_core_consciousness_tree;
pub mod nt_core_echo_terminal;

// ═══════════════════════════════════════════════════════════════════
// L6 — 自我层 (Self)
// ═══════════════════════════════════════════════════════════════════
pub mod l6_self;
pub mod nt_core_aware;
pub mod nt_core_self;
pub mod nt_core_self_constitution;
pub mod nt_core_guard_chain;
pub mod nt_core_kb_primitives;
pub mod nt_core_kb_types;

// ═══════════════════════════════════════════════════════════════════
// L7 — 能力层 (Capability) — *** 核心新增 ***
// ═══════════════════════════════════════════════════════════════════
pub mod l7_capability;
#[cfg(feature = "research")]
pub mod nt_core_agent_patterns;
pub mod nt_core_model_skills;

// ═══════════════════════════════════════════════════════════════════
// L8 — 自主神经层 (Autonomic)
// ═══════════════════════════════════════════════════════════════════
pub mod l8_autonomic;
pub mod nt_core_absorb;
pub mod nt_core_iter;
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
pub mod nt_core_accessor;
pub mod nt_core_axiom_tree;
pub mod nt_core_cache;
pub mod nt_core_cap;
pub mod nt_core_conn;
pub mod nt_core_edit;
pub mod nt_core_embed;
pub mod nt_core_epoch;
pub mod nt_core_error_parse;
pub mod nt_core_error_recovery;
pub mod nt_core_event;
pub mod nt_core_mcp;
pub mod nt_core_retrieval;
pub mod nt_core_router;
pub mod nt_core_self_review;
pub mod nt_core_traits;
pub mod nt_core_wbmem;
pub mod nt_core_ws;
pub mod nt_io_cache;
pub mod nt_io_telemetry;

pub mod nt_core_answer_engine;
pub mod nt_core_arch_fitness;
#[cfg(feature = "research")]
pub mod nt_core_bounded_collections;
pub mod nt_core_data_pipeline;
pub mod nt_core_memory_budget;
pub mod nt_core_qtest;
pub mod nt_core_quantum_fusion;
pub mod nt_core_resource_pool;
pub mod nt_core_schema_watchdog;
pub mod nt_core_scoring_substrate;
pub mod nt_core_second_brain;
pub mod nt_core_self_test;
pub mod nt_core_self_test_integration;
pub mod nt_core_simulate_engine;
#[cfg(feature = "research")]
pub mod nt_core_source_edit;
pub mod nt_core_state_substrate;
pub mod nt_core_subagent;
pub mod nt_core_telemetry;
pub mod nt_core_vector_store;
// Formal verification proof harnesses
#[cfg(test)]
pub mod kani_proofs;

// ═══════════════════════════════════════════════════════════════════
// Re-exports — 按 9 层顺序
// ═══════════════════════════════════════════════════════════════════

// --- L7: Capability (能力层 — 核心新增) ---
pub use l7_capability::nt_core_antidistil::{
    analyze_response_pattern, detect_watermarked_in_corpus, AlertType, AntiDistilStats,
    AntiDistilStore, AntiDistillationSystem, ApostropheVariant, DecomposeSuggestion, DetectorStats,
    DistillationAlert, DistillationDetector, ResponseAnalysis, ResponseTracer, TaskDecomposer,
    TraceRecord, TracerStats, WatermarkBits, WatermarkConfig, WatermarkEngine,
};
pub use l7_capability::{
    capability_id_from_name,
    Capability,
    CapabilityCost,
    CapabilityId,
    CapabilityKind,
    CapabilityRegistry,
    CapabilityStats,
    ContextSlot,
    EvolveResult,
    IllusionReport,
    // 成熟度
    MaturityEngine,
    MaturityFeedback,
    PulseBus,
    PulseKind,
    SlotKind,
    // 调度 (thater stub archived — use real nt_core_scheduler)
    // 协议
    StarPulse,
    // 观察者
    TurkeyScientist,
};
// 6 级成熟度（扩展 4→6）
pub use l7_capability::nt_core_orch_agent::{
    AgentMessage, AgentPoolStats, MessageType, SubagentConfig, SubagentInstance, SubagentManager,
    SubagentStatus,
};
pub use l7_capability::registry::MaturityLevel as CapMaturityLevel;

// --- L0: Substrate ---
#[cfg(feature = "research")]
pub use nt_core_deploy::{
    AWQConfig, AWQQuantization, AneDirectProgramV2, AneProgramCache, AotCompiler, AotResult,
    AotTarget, CacheEntry, CachePolicy, CoreAiAotConfig, CoreAiAotResult, CoreAiDeployPipeline,
    DeployReport, EdgeDeployPipeline, GGUFConfig, GGUFLevel, GGUFQuantization, HardwareDetector,
    HardwarePowerProfile, HardwareProfile, LoraAdapter, OsType, PowerProfile, PowerState,
    PowerThermalModel, Quantization, QuantizationPipeline, QuantizedModel, Quantizer,
};

// --- L4: Cognition ---
pub use nt_core_crt::{CrtPlan, CrtTimeScale};
pub use nt_core_e8_vsa::E8VsaEmbedding;
pub use nt_core_hex::{
    all_reasoning_states, evolve_strategy_entry, intention_from_string, optimal_starting_mode,
    rank_modes_for_task, select_mode_by_intent, select_mode_by_intent_with_effort, strategy_matrix,
    FullReasoningState, IntentionContext, MetaState, ModeFit, MultipleHypothesisEvaluator,
    ProblemDomain, ReasoningApproach, ReasoningEffort, ReasoningHexagram, ReasoningPath,
    MODE_DESCRIPTIONS, MODE_NAMES, MODE_TASKS,
};
pub use nt_core_plan::{E8Plan, PlanGenerator, PlanMetrics, PlanStep, StepStatus};
pub use nt_core_policy::{E8Outcome, E8Policy, E8TransitionLearner, NUM_E8_FACTORS};
pub use nt_core_prm::{
    blended_advantage, compute_step_advantages, compute_step_rewards, lambda_grpo_loss,
    zscore_normalize, AgentTrajectory, Coach, CoachContext, HeuristicCoach, LambdaGrpoConfig,
    LambdaGrpoLearner, LambdaGrpoResult, ProcessRewardLearner, ProcessScore, ScoredCriterion,
    StepAdvantage, StepGrpoConfig, StepGrpoLearner, StepGrpoReport, StepReward,
    TrajectoryCollector, TrajectoryStep,
};
pub use nt_core_sae::{
    LayerSae, MonosemanticFeature, SaeConfig, SaeDecoder, SaeEncoder, SaeFeature, SaeOutput,
    SparseAutoencoder, SteeringController, SteeringTarget, SteeringVector, E8_SAE_LAYERS,
    SAE_INPUT_DIM, SAE_LATENT_DIM,
};
pub use nt_core_sae_bridge::SAEBridge;
pub use nt_core_trajectory_compress::{
    CompressionLevel, TrajectoryCompressionReport, TrajectoryCompressor,
};

pub use nt_core_consensus::{
    AbductiveExplanation, AbductiveSolver, ConsensusConfig, ConsensusReport, ReflectionHead,
    ReflectionPipeline, ReflectionResult,
};
pub use nt_core_e8::abduction::{
    AbductionCycleReport, AbductiveHypothesis, AbductiveReasoningEngine, AbductiveState,
    CausalEdge, CausalGraph, CausalNode,
};
pub use nt_core_e8::e8_abduction_bridge::{AbductiveTransitionReport, E8AbductionBridge};

pub use nt_core_reasoning::{
    default_context_builder, default_method_registry, ContextBuilder, MethodRegistry, MethodSpec,
    ReasoningMethod, ReasoningStep, ReasoningTrace, TraceSource,
};

// --- L5: Consciousness ---
pub use l5_consciousness::resonance::{
    default_specialist_states, resonate_and_select, resonate_cycle, ResonanceMatrix,
    ResonanceReport, MODULE_COUNT, RESONANCE_THRESHOLD,
};
// --- L6: Self ---
pub use nt_core_self::archive::{AttentionSnapshot, SiliconArchive, SiliconSnapshot};
pub use nt_core_self::attention_head::{
    AttentionDomain, AttentionHead, AttentionManager, AttentionProfile,
};
pub use nt_core_self::context_window::{CognitiveUnit, CognitiveUnitKind, ContextWindow};
pub use nt_core_self::intrinsic_motivation::{IntrinsicMotivation, MotivationState};
pub use nt_core_self::metacognitive_evaluator::{
    CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport, FlagCategory, FlagSeverity,
    RepairSuggestion, RepairTarget,
};
pub use nt_core_self::reasoning_strategy::{
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
};
pub use nt_core_self::seal::{
    CurriculumTask, EditType, SealIterationReport, SealPipeline, SelfEdit,
};
pub use nt_core_self::self_referential::{PlanRecord, SelfReferentialMonitor, ThresholdAdjustment};
pub use nt_core_self::silicon_self::{SiliconSelfModel, SiliconSelfState};
pub use nt_core_self::skill_crystal::{CrystalRegistry, SkillCrystal};
pub use nt_core_self::system_identity::{CognitiveCapability, SystemIdentity, ValueConstraint};
pub use nt_core_self::thinking_trace::{ReflectionGrade, ThinkingStep, ThinkingTrace};
pub use nt_core_self_constitution::{
    global_constitution, reload_constitution, ComplianceReport, ComplianceViolation, Constitution,
    ConstitutionLoader, DevRule, ExperienceEntry, RuleCategory,
};
pub use nt_core_self_test::{
    report as selftest_report, ConstitutionComplianceTest, SelfTest, SelfTestRegistry,
    SelfTestResult,
};

// --- L9: Transcendent ---
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
pub use nt_core_observer::{E8Observer, ObserverReport, OneObserver, PrmObserver};
pub use nt_core_observer_error::{
    CircuitBreaker, CircuitState, ErrorRecoveryError, FallbackHandler, ObserverErrorRecovery,
    RetryConfig,
};

// --- 基础设施 ---
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_accessor::{AccessionReport, Accessor, SourceType, UrlAccessor};
pub use nt_core_bank::{
    MemoryLifecycle, MemoryTier, ReasoningBank, ReasoningBankStats, ReasoningMemory,
    TemporalContext,
};
pub use nt_core_cache::{CacheStats, SemanticCache};
pub use nt_core_cap::CapabilityVector;
pub use nt_core_edit::{MicroEdit, ToolCall};
pub use nt_core_graph::{EdgeRelation, HyperEdge, HyperGraph, HyperNode, HyperNodeType};
pub use nt_core_hcube::aif::belief::{
    FactorGraphBeliefPropagation, FactorialPOMDP, POMDPBeliefUpdater,
};
pub use nt_core_hcube::aif::{AiStepReport, FreeEnergyEngine, GenerativeModel, PolicyEvaluator};
pub use nt_core_hcube::fhrr_vsa::{
    bind, bundle, bundle_two, cleanup, cleanup_always, encode_scalar, permute, random_vector,
    random_vector_dim, similarity, FhrrHyperCube, FhrrVector, FHRR_DIM,
};
pub use nt_core_hcube::ghrr_vsa::{
    ghrr_bind_dir, ghrr_bundle, ghrr_bundle_two, ghrr_permute, ghrr_random_vector, ghrr_similarity,
    ghrr_unbind_dir, GhrrHyperCube, GhrrVector, GHRR_DIM, GHRR_ETA,
};
pub use nt_core_hcube::qfhrr_vsa::{
    encode_scalar_qfhrr, fhrr_to_qfhrr, qbind, qbundle, qfhrr_to_fhrr, qsimilarity, qunbind,
    random_qfhrr, QuantizedFhrrHyperCube, QuantizedFhrrVector, QFHRR_DIM, QFHRR_LEVELS,
};
pub use nt_core_hcube::{VSAEngine, VsaBackend};
pub use nt_core_iter::SelfIteration;
pub use nt_core_knowledge::{
    KnowledgeProvider, KnowledgeSource, RewardSource, SourceAccessRecord, SourceAccessTracker,
    TaskType,
};
pub use nt_core_mcp::McpServer;
pub use nt_core_traits::{
    AgentExecutor, BrainProvider, EngineProvider, MemoryProvider, NativeTool, RichMemoryProvider,
    SealResult, SessionProvider, ToolDef, ToolOutput, ToolProvider,
};
pub use nt_core_ws::WORKSPACE_MANAGER;

// --- 基础设施: 统一资源池 ---
pub use nt_core_resource_pool::{
    AnyPool, DiscoveredResource, DiscovererInfo, DiscoveryCache, DiscoveryResult, NormalizedEntry,
    PoolHealthReport, PoolSelectionStrategy, PoolSnapshot, PoolSupervisor, PooledResource,
    ProxyUrlNormalizer, ResourceDiscoverer, ResourceDiscoveryEngine, ResourceKind, ResourceMeta,
    ResourceNormalizer, ResourcePool, ResourceRegistry,
};

// ─── 基础设施: 统一数据管道 ───
pub use nt_core_data_pipeline::{
    DataLineage, LineageEntry, PipelineOrchestrator, PipelineRunReport, PipelineStage, StageResult,
};
