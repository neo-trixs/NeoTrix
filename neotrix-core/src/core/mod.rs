//! # NeoTrix Core — 10-domain module architecture
//!
//! ## Domains
//!
//! | Domain | Module | Role |
//! |--------|--------|------|
//! | 1. Consciousness | `nt_core_consciousness` | E8/GWT/awareness/self/identity |
//! | 2. HyperCube (VSA) | `nt_core_hcube` | VSA primitives, encoding, spatial |
//! | 3. Knowledge | `nt_core_knowledge` | Graph, evidence, retrieval, OSINT |
//! | 4. Experience | `nt_core_experience` | Evolution, self-modification, learning |
//! | 5. Meta-Cognition | `nt_core_meta` | Self-model, planner, monitor, scanner |
//! | 6. Reasoning | `nt_core_reasoning` | MCTS, E8 reasoning, inference |
//! | 7. Input/Perception | `nt_core_input` | Documents, media, code, files |
//! | 8. Shield/Safety | `nt_core_shield` | Safety gates, sandbox, verification |
//! | 9. Loop/Governance | `nt_core_loop`, `nt_core_governance` | Loop engine, trust, consensus |
//! | 10. Economic | `nt_core_economic` | Wallet, market data, signals |

// ═══════════════════════════════════════════════════════
// 1. CONSCIOUSNESS — Awareness, identity, GWT, affect
// ═══════════════════════════════════════════════════════
pub mod nt_core_consciousness;  // (164 files) — main consciousness engine
            // (12 files) — global workspace + resonance
        // (9 files) — context OS, working memory
       // (11 files) — identity core, self-reasoner
           // (28 files) — self-model, attention head
           // (6 files) — intent buffer/engine
          // (3 files) — embodied awareness
            // (9 files) — competitive thinker model
        // (4 files) — FEP-IIT bridge
pub mod nt_core_iit_phi;        // (1 file) — IIT Φ calculation
pub mod nt_core_time;           // (1 file, re-export from neotrix_mind)

// ═══════════════════════════════════════════════════════
// 2. VSA HYPERCUBE — Vector-symbolic algebra
// ═══════════════════════════════════════════════════════
pub mod nt_core_hcube;          // (86 files) — VSA primitives, encoding
        // (3 files) — spatial VSA encoding
pub mod nt_core_negentropy;     // (6 files) — EFE minimizer, negentropy

// ═══════════════════════════════════════════════════════
// 3. KNOWLEDGE — Graph, evidence, storage, OSINT
// ═══════════════════════════════════════════════════════
pub mod nt_core_knowledge;      // (44 files) — knowledge graph, evidence
pub mod nt_core_truth;          // (9 files) — bias audit, fact tiering
          // (3 files) — VSA architect
       // (9 files) — language evaluation
     // (13 files) — JEPA world model
pub mod nt_core_source_cognition; // (10 files) — sensory modalities

// ═══════════════════════════════════════════════════════
// 4. EXPERIENCE — Evolution, learning, self-modification
// ═══════════════════════════════════════════════════════
pub mod nt_core_experience;     // (263 files) — evolution, learning
pub mod nt_core_edit;           // (7 files) — self-edit operations
pub mod nt_core_self_evolution; // (2 files) — RSI core re-export
    // (4 files) — self-modify agent
      // (2 files) — symbolic discovery
    // (4 files) — adversarial training
pub mod nt_core_codegen;        // (13 files) — Ne compiler bridge
pub mod nt_core_self_audit;     // (7 files) — architecture self-audit, wiring verification, cicada loop

// ═══════════════════════════════════════════════════════
// 5. META-COGNITION — Self-model, monitoring, planning
// ═══════════════════════════════════════════════════════
pub mod nt_core_meta;           // (34 files) — meta-cognition loop
pub mod self_measure;           // (8 files) — subsystem measurement

// ═══════════════════════════════════════════════════════
// 6. REASONING — MCTS, E8 hex, inference pipelines
// ═══════════════════════════════════════════════════════
pub mod nt_core_reasoning;      // (22 files) — MCTS, reasoning
           // (14 files) — reasoning bank
pub mod nt_core_hex;            // (2 files) — E8 reasoning hexagrams
      // (6 files) — inference pipelines
pub mod nt_core_prm;            // (1 file, 741 lines) — PRM learner

// ═══════════════════════════════════════════════════════
// 7. INPUT / PERCEPTION — Documents, media, code
// ═══════════════════════════════════════════════════════
pub mod nt_core_input;          // (39 files) — document/media extraction
pub mod nt_core_sense;          // (5 files) — sensory types
          // (3 files) — audio capture, VAD

// ═══════════════════════════════════════════════════════
// 8. SHIELD — Safety, sandbox, verification
// ═══════════════════════════════════════════════════════
pub mod nt_core_shield;         // (8 files) — safety gates
        // (4 files) — sandbox execution

// ═══════════════════════════════════════════════════════
// 9. GENERATION — IntentVector-driven unified output pipeline
// ═══════════════════════════════════════════════════════
pub mod nt_core_generation;      // (15 files) — IntentVector → text/image/video/audio/html

// ═══════════════════════════════════════════════════════
// 10. LOOP / GOVERNANCE — Engine, trust, consensus
// ═══════════════════════════════════════════════════════
pub mod nt_core_loop;           // (10 files) — loop engine
pub mod nt_core_scheduler;      // Job scheduler
pub mod nt_core_governance;     // (5 files) — trust, consensus
      // (5 files) — job scheduler

// ═══════════════════════════════════════════════════════
// 11. ECONOMIC — Wallet, market data, trading
// ═══════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════
// CORE PRIMITIVES — Foundation data types and traits
// ═══════════════════════════════════════════════════════
pub mod nt_core_self;           // Self-awareness, attention, RSI verification
pub mod nt_core_absorb;         // AbsorbValidator trait
pub mod nt_core_cap;            // CapabilityVector (23-dim)
pub mod nt_core_crt;            // CrtPlan, CrtTimeScale
pub mod nt_core_data_types;     // ToolResult, shared data types
pub mod nt_core_e8;             // (710 lines) — E8 root system
pub mod nt_core_e8_model;       // (53 lines) — E8 world model
pub mod nt_core_error;          // CoreError (re-export from neotrix_mind)
pub mod nt_core_graph;          // HyperGraph, HyperEdge
pub mod nt_core_graceful;       // Graceful degradation
pub mod nt_core_health;         // System health probes
pub mod nt_core_idempotency;    // IdempotencyGuard
pub mod nt_core_infer;          // Inference utilities
pub mod nt_core_iter;           // SelfIteration trait
pub mod nt_core_kron;           // Kronecker operations
pub mod nt_core_metering;       // Usage metering
pub mod nt_core_observer;       // Observer pattern
pub mod nt_core_policy;         // E8 policy transition
pub mod nt_core_ratelimit;      // Rate limiting
pub mod nt_core_sigreg;         // Signal registry
pub mod nt_core_shared_types;   // Shared types (re-export from neotrix_mind)
pub mod nt_core_ssm;            // State-space model
pub mod nt_core_td;             // Temporal difference
pub mod nt_core_traits;         // Core traits
pub mod nt_core_util;           // Utility functions
pub mod nt_core_value_system;   // Value system
pub mod nt_core_walsh;          // Walsh-Hadamard transform
pub mod nt_core_wbmem;          // Working memory
pub mod nt_core_skill_store;    // (4 files) — skills web discovery, fusion, store, evolution dispatch
pub mod self_model;             // (439 lines) — standalone self-model
pub mod skill;                  // (155 lines) — skill primitive

// ═══════════════════════════════════════════════════════
// BRIDGES — neotrix/ → core/ alias re-exports
// These provide `crate::core::nt_core_xxx` access
// to modules that live in `crate::neotrix::nt_xxx`
// ═══════════════════════════════════════════════════════
pub use crate::neotrix::nt_agent_arch as nt_core_arch;
pub use crate::neotrix::nt_agent_core as nt_core_agent;
pub use crate::neotrix::nt_agent_hive as nt_core_hive;
pub use crate::neotrix::nt_agent_plugin as nt_core_plugin;
pub use crate::neotrix::nt_act_trading as nt_core_trading;
pub use crate::neotrix::nt_io_conn as nt_core_conn;
pub use crate::neotrix::nt_io_design_token as nt_core_design_token;
pub use crate::neotrix::nt_io_llm as nt_core_llm;
pub use crate::neotrix::nt_io_llm_provider as nt_core_llm_provider;
pub use crate::neotrix::nt_io_llm_router as nt_core_llm_router;
pub use crate::neotrix::nt_io_network as nt_core_network;
pub use crate::neotrix::nt_io_output as nt_core_output;
pub use crate::neotrix::nt_io_router as nt_core_router;
pub use crate::neotrix::nt_io_shutdown as nt_core_shutdown;
pub use crate::neotrix::nt_io_tokenopt as nt_core_tokenopt;
pub use crate::neotrix::nt_memory_session as nt_core_session;
pub use crate::neotrix::nt_memory_storage as nt_core_storage;
pub use crate::neotrix::nt_memory_vector_store as nt_core_vector_store;
pub use crate::neotrix::nt_memory_wal as wal;
pub use crate::neotrix::nt_memory_ws as nt_core_ws;
pub use crate::neotrix::nt_shield_protect as nt_core_protect;
pub use crate::neotrix::nt_world_document as nt_core_document;
pub use crate::neotrix::nt_world_search as nt_core_search;
pub use crate::neotrix::nt_world_translate as nt_core_translate;
pub mod nt_core_vision;

// ═══════════════════════════════════════════════════════
// RE-EXPORTS — Top-level type aliases
// ═══════════════════════════════════════════════════════

// Foundation
pub use nt_core_error::CoreError;
pub use nt_core_data_types::ToolResult;
pub use nt_core_util::{
    unix_now_ms, unix_now_nanos, unix_now_secs, TOR_CONTROL_ADDR, TOR_CONTROL_PORT, TOR_SOCKS_ADDR,
    TOR_SOCKS_PORT,
};

// Capabilities
pub use nt_core_cap::CapabilityVector;
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_iter::SelfIteration;

// Graph & State
pub use nt_core_graph::{EdgeRelation, HyperEdge, HyperGraph, HyperNode, HyperNodeType};
pub use nt_core_ssm::{
    ConsciousnessTier, SelectableOperator, SelectiveState, SemanticBlock, SemanticType,
    SparseMatrix,
};
pub use nt_core_traits::{
    AgentExecutor, BrainProvider, EngineProvider, MemoryProvider, RichMemoryProvider, SealResult,
    SessionProvider,
};

// Edit
pub use nt_core_edit::{MicroEdit, SelfEdit, ToolCall};

// E8 Reasoning
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

// Consciousness
pub use nt_core_consciousness::{
    AwakeningReport, CognitiveLoadMonitor, ConsciousnessAwakening, ConsciousnessStream,
    CritiqueResult, FirstPersonRef, InnerCritic, SpeciousPresent, ThinkingMode, VsaOrigin,
    VsaSelfCategory, VsaTagged, VsaWorldCategory,
};
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

// GWT Resonance
pub use nt_core_consciousness::gwt::resonance::{
    compute_semantic_entropy, default_specialist_states, diversity_inject, resonate_and_select,
    resonate_cycle, resonate_cycle_with_diversity, ResonanceMatrix, ResonanceReport,
    DIVERSITY_MIN_ENTROPY, DIVERSITY_NOISE_AMPLITUDE, MODULE_COUNT, RESONANCE_THRESHOLD,
};

// Reasoning Bank
pub use nt_core_reasoning::bank::{
    MemoryLifecycle, MemoryTier, ReasoningBank, ReasoningBankStats, ReasoningMemory,
    TemporalContext,
};

// Self model (meta-cognition)
pub use nt_core_meta::{
    AuditEngine, AuditFinding, AuditPhase, AuditReport, AuditStatus, FindingSeverity,
    CodeScanner, GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
    MetaCognitiveLoop, MetaCycleResult, AlertSeverity, HealthCheck, HealthTrend, MetaAlert,
    MetaMonitor, ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal,
    MetaGoalBridge, PlannedEvolution, RiskLevel, CompilationHealth, ComponentMap, ComponentNode,
    DebtSeverity, DepEdge, DepGraph, DepKind, EventKind, EvolutionEvent, FileInfo, ModuleInfo,
    SelfModel, TechDebtInventory, TechDebtItem, TechDebtKind, TestCoverage, Weakness,
    WeaknessAnalyzer, WeaknessReport, WeaknessSummary,
};

// Self model (silicon)
pub use nt_core_consciousness::self_awareness::{
    AttentionSnapshot, SiliconArchive, SiliconSnapshot, AttentionDomain, AttentionHead,
    AttentionManager, AttentionProfile, CognitiveUnit, CognitiveUnitKind, ContextWindow,
    IntraReflection, IntraReflectionReport, PreActionIntrospector, PredictedOutcome,
    IntrinsicMotivation, MotivationState, CognitiveEvaluator, CognitiveFlag, CognitiveHealthReport,
    FlagCategory, FlagSeverity, RepairSuggestion, RepairTarget, ReasoningStrategy,
    ReasoningStrategyRegistry, StrategyKind, PlanRecord, SelfReferentialMonitor,
    ThresholdAdjustment, SiliconSelfModel, SiliconSelfState, CrystalRegistry, SkillCrystal,
    SystemIdentity, CognitiveCapability, ValueConstraint, ReflectionGrade, ThinkingStep,
    ThinkingTrace,
};

// Agent architecture
pub use nt_core_governance::arch::ArchitectAgent;

// Context
pub use nt_core_reasoning::context::context_gatherer::{
    ContextFragment, ContextGatherer, ContextSource, ContextSourceMeta, GatheredContext,
};
pub use nt_core_reasoning::context::context_os::{ContextOS, ContextOSStats};
pub use nt_core_reasoning::context::working_memory::{BindingOp, WorkingMemory, WorkingMemoryItem};
pub use nt_core_reasoning::context::{
    AllocatedSlice, AssembledContext, BudgetSourceType, CompactionIntent, CompactionPriority,
    ContextBudget,
};

// Experience / Evolution
pub use nt_core_experience::{
    BridgeConfig, BridgeV2Stats, ConceptNode, ConsolidatedMemory, ConsolidationBridgeV2,
    CurriculumGenerator, DifficultyLevel, DomainConfidence, EpistemicConfig, EpistemicSelfModel,
    EpistemicState, ExperienceReflector, FailurePattern, FailureType, GeneratorConfig, Heuristic,
    HeuristicCategory, HeuristicFilter, PolicyRepairEngine, ReflectorConfig, RepairMode,
    RepairPolicy, SkillAccumulator, SkillComposition, SkillFilter, TaskTemplate, VSASkill,
};
pub use nt_core_experience::{
    HarnessProposal, HarnessProposer, HarnessWeakness, ProposalValidator, SelfHarnessEngine,
    WeaknessMiner,
};

// Knowledge
pub use nt_core_knowledge::{
    AngleSelector, Audience, Modality, MultimodalStoryteller, Story, StoryPlan, StoryRenderer,
    Claim, EvidenceInspector, EvidenceVerificationResult, VerifiabilityGate, VerificationStatus,
    KnowledgeProvider, KnowledgeSource, RewardSource, SourceAccessRecord, SourceAccessTracker,
    TaskType,
};

// VSA HyperCube
pub use nt_core_hcube::{
    HrrBackend, SpectralVSA, TrigramInvertedIndex, e8_cortical_vsa_transform, CorticalCoord,
    E8CorticalMapping, CORTICAL_NEURON_COUNT, BandPassExpert, FrequencyBand, GraphLaplacian,
    HighPassExpert, LowPassExpert, MoSpectralExperts, SpectralExpert, SpectralFilter, SpectralNSR,
    SpectralRule, CerebellumResonator, CortexAdaptive, ResonanceMode, CBRNN, DefectConfig,
    E8ParticleSpectrum, E8TopologicalDefects, ForceType, HalfIntegerSpin, TopologicalCharge,
    WeylOrbit, E8FieldIntegrator, E8Lagrangian, Lattice3D, PDESolver, E8FieldSolver,
    FieldSolverConfig, SpatialAttentionGate, VSASpatialEncoder, Vec3D, SpectralDenoiser,
    WaveGeometricEmbed, WaveGeometricVSA,
};
pub use nt_core_hcube::attractor_basin::{
    AttractorBasin, AttractorBasinDynamics, BasinStats, BasinType,
};
pub use nt_core_hcube::dream_consolidation::{
    ConsolidationPhase, DreamConfig, DreamConsolidation, DreamEvent, DreamPhase, DreamReport,
    NremConfig, RemConfig,
};
pub use nt_core_hcube::ebbinghaus_decay::{DecayConfig, EbbinghausDecay, MemoryTrace};

// File index
pub use nt_core_knowledge::file_index::{
    classify_intent, compute_file_hash, fuse_results, ContentIndex, FileIndexState, FileQuery,
    MerkleNode, MerkleWatch, PathIndex, QueryEngine, QueryIntent, ScanResult, ScoredFile,
    StructureIndex,
};

// Search & Retrieval
pub use nt_core_search::{
    AgenticSearcher, HybridRetriever, RRFFuser, SearchBudget, SearchEvaluator, SearchPlan,
    SearchPlanner, SearchResultItem, SearchStrategy, SearchVerdict, SearchedDocument,
    SearchedDocumentCollection,
};

// Loop Engineering
pub use nt_core_loop::{
    CoverageReport, GoalRegistry, GraphStats, HandlerDiscovery, LoopEngine, LoopGoal,
    LoopGoalStatus, LoopPhase, LoopState, LoopStats, LoopVerifier, NodeGroup, PipelineConditions,
    PipelineGraph, PipelineNodeData, Verdict, StopConditionConfig, StopConditionState, StopReason,
};

// Scheduler
pub use nt_core_loop::scheduler::{
    default_scheduler, ContextGate, JobRunHistory, JobRunRecord, ScheduleType, ScheduledJob,
    SchedulerEngine, SchedulerStats,
};

// Agent & Hive
pub use nt_core_agent::{
    AgentCommunicationBus, AgentMessage, ByzantineConsensusLayer, CDPSessionManager,
    FactorMiner, QuantDataIngestion, RemoteAgentHost, TeamOrchestrator,
};

// Translation
pub use nt_core_translate::{
    BilingualEntry, BilingualLexicon, CleanupRule, Language, TextAnalysis, TextType,
    TranslationOutput, TranslationPipeline, TranslationResult, TranslationStrategy,
    TranslationeseDiagnosis, TranslationeseSymptom, VerificationScore, VsaTranslationEngine,
};

// Security / Self-Protection
pub use nt_core_protect::{
    install_panic_filter, obfuscate_str, reveal_bytes, sanitize_panic, strip_source_path,
    AttackAttempt, AttackSurface, AttackSurfaceScanner, DrillResult, EnvironmentValidator,
    FablePacket, FableRouterStats, FableRoutingDecision, FableTier, HoneypotCategory,
    HoneypotForest, HoneypotNode, ImprovementTracker, IntegrityGuard, IntegrityReport, Obfuscated,
    ProtectionStats, RedTeamSeverity, RedTeamingEngine, SafeError, SecurityDrillScheduler,
    SelfProtection, ThinkingBlock, VsaFableRouter,
};

// Infrastructure
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
pub use nt_core_shield::sandbox::{
    SandboxConfig as KernelSandboxConfig, SandboxLevel as KernelSandboxLevel,
};

// World Model
pub use nt_core_knowledge::world_model::{
    ActionEmbedding, AdaptiveExitGate, DynamicsError, LoopedDynamics, SpectralConstraint,
    VsaLatentState,
};

// Trading
pub use nt_core_trading::{
    AssetClass, DrawdownMonitor, FusedMarketSignal, KellyCalculator, MarketRegime, OHLCVBar,
    OnchainSignal, OrderType, PortfolioSummary, Position, PositionSizer, RiskConfig, RiskManager,
    RiskSnapshot, SentimentSignal, SignalFusion, SignalGenerator, SignalSource,
    TechnicalIndicators, Ticker, Timeframe, TradeSide, TradingEngine, TradingSignal,
    TrendDirection, VarCalculator, VolatilityLevel,
};
