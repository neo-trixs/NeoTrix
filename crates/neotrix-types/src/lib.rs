#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unexpected_cfgs)]
#![deny(warnings)]
#![deny(dead_code)]

pub mod core;

// Re-export sub-modules so external code can use neotrix_types::nt_core_bank::X etc.
pub use core::nt_core_bank as memory;
pub use core::knowledge;
pub use core::edit;
pub use core::nt_core_event as event;
pub use core::nt_core_ssm as signal;
pub use core::context;
pub use core::nt_core_gwt as consciousness;
pub use core::nt_core_hcube as hypercube;
pub use core::nt_core_meta as metacognition;
pub use core::nt_core_self as thinking_model;
pub use core::skills;
pub use core::tools;
pub use core::fs_util;
pub use core::skill;
pub use core::nt_core_cap as capability;
pub use core::nt_core_e8 as e8;
pub use core::nt_core_hex as e8_reasoning;
pub use core::nt_core_observer as e8_observer;
pub use core::nt_core_graph as hypergraph;
pub use core::nt_core_accessor as accessor;
pub use core::nt_core_absorb as absorb;
pub use core::nt_core_self_org as self_org;
pub use core::nt_core_traits as traits;

pub use core::nt_core_self_org::{AgentMetadata, AgentStatus, DeadEndRecord, DeadEndRegistry, Heartbeat, SharedState, SelfOrgProtocol};

pub use core::{
    SkillTier, SkillDefinition, SkillRegistry,
    ToolRisk, ToolClassification,
    AbsorbValidator, Accessor, AccessionReport, SourceType, UrlAccessor,
    CapabilityVector,
    SelfEdit, MicroEdit, ToolCall,
    KnowledgeSource, KnowledgeProvider, TaskType, RewardSource, SourceAccessTracker, SourceAccessRecord,
    ReasoningBank, ReasoningMemory, TemporalContext, MemoryTier, MemoryLifecycle, ReasoningBankStats,
    SelectiveState, SelectableOperator, SparseMatrix, ConsciousnessTier, SemanticType, SemanticBlock,
    SelfIteration,
    MemoryProvider, RichMemoryProvider, AgentExecutor, ToolProvider, ToolDef, ToolOutput, SessionProvider, BrainProvider, EngineProvider, SealResult,
    HyperGraph, HyperNode, HyperEdge, HyperNodeType, EdgeRelation,
    ToolSandbox, SandboxError, SessionStore, SessionRecord, SessionMessage, TruncationStrategy, HookRegistry, LifecycleEvent,
    ConsciousnessState, ConsciousnessLoop, RecurrentCell, CellDecision, LoopExit, TickMetrics, PanoramaCell,
    PanoramicInventory, ModuleEntry, CodeLocation, SymbolKind,
    ResonanceMatrix, ResonanceReport, MODULE_COUNT,
    resonate_and_select, resonate_cycle, default_specialist_states,
    RESONANCE_THRESHOLD,
    ReasoningHexagram, MetaState, FullReasoningState, ModeFit, ReasoningPath,
    ReasoningApproach, ProblemDomain,
    all_reasoning_states, optimal_starting_mode, rank_modes_for_task, strategy_matrix,
    evolve_strategy_entry,
    MODE_NAMES, MODE_DESCRIPTIONS, MODE_TASKS,
    E8HexagramHomology, Hexagram, FermionState, E8Weight,
    verify_e8_dimension, verify_three_generations, verify_total_fermions,
    verify_all_identities, verify_dayan_identity, verify_lo_shu, verify_he_tu_sum,
    shao_yong_sequence, king_wen_sequence, hexagram_matrix,
    e8_root_system, fermion_states_for_generation, all_sm_fermions,
    hadamard_matrix, hexagram_hadamard,
    E8_DIM, E8_RANK, E8_ROOTS, HEXAGRAM_COUNT, LINES_PER_HEXAGRAM, TOTAL_LINES,
    FERMION_GENERATIONS, FERMIONS_PER_GENERATION, TOTAL_SM_FERMIONS, REMAINING_E8_GENERATORS,
    TRIGRAM_COUNT, DAYAN_NUMBER, OBSERVABLE_DOF, OBSERVER_DOF, LO_SHU_CONSTANT, HE_TU_SUM,
    TRIGRAM_NAMES, WEN_SEQUENCE,
    CrtTimeScale, CrtPlan, CrtTimeline, CrtGoal,
    OneObserver, ObserverReport, TrajectoryPattern, StepQuality,
    WalshMemoryIndex,
    SiliconSelfModel, SiliconSelfState, ContextWindow, CognitiveUnit, CognitiveUnitKind,
    AttentionHead, AttentionDomain, AttentionProfile, AttentionManager,
    SystemIdentity, CognitiveCapability, ValueConstraint,
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
    ThinkingTrace, ThinkingStep, ReflectionGrade,
    SelfModel, ModuleInfo, FileInfo, DepGraph, DepEdge, DepKind,
    TechDebtInventory, TechDebtItem, TechDebtKind, DebtSeverity,
    EvolutionEvent, EventKind, ComponentMap, ComponentNode,
    TestCoverage, CompilationHealth,
    CodeScanner, MetaMonitor, MetaAlert, AlertSeverity, HealthCheck, HealthTrend,
    WeaknessAnalyzer, Weakness, WeaknessReport, WeaknessSummary,
    EvolutionPlanner, PlannedEvolution, ImpactEstimate, RiskLevel, EvolutionAction, ActionStatus,
    MetaCognitiveLoop, MetaCycleResult,
};
