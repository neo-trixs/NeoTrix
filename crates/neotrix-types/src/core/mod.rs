//! # NeoTrix Core — 纯理论/数据模型层
//!
//! 零外部依赖层，仅包含核心数据结构和 trait 定义。

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
pub mod skill;
pub mod nt_core_gwt;
pub mod context;
pub mod nt_core_accessor;
pub mod nt_core_meta;
pub mod nt_core_self;
pub mod nt_core_e8;
pub mod nt_core_hex;
pub mod nt_core_observer;
pub mod nt_core_walsh;
pub mod nt_core_crt;
pub mod skill_tree;
pub mod nt_core_self_org;
pub mod hooks;
pub mod task_types;
pub mod panoramic;
pub mod node_canvas;
pub mod skills;
pub mod tools;
pub mod fs_util;
pub mod wal;
pub mod persist_envelope;
pub mod epoch;
pub mod file_parser;
pub mod layered_memory;
#[cfg(feature = "rkyv-storage")]
pub mod nt_core_rkyv;

// Re-export 主要类型到 core 层顶层
pub use nt_core_cap::CapabilityVector;
pub use nt_core_knowledge::{KnowledgeSource, KnowledgeProvider, TaskType, RewardSource, SourceAccessTracker, SourceAccessRecord};
pub use nt_core_accessor::{Accessor, AccessionReport, AccessorSourceType, UrlAccessor};
pub use nt_core_edit::{SelfEdit, MicroEdit, ToolCall};
pub use nt_core_bank::{ReasoningBank, ReasoningMemory, TemporalContext, MemoryTier, MemoryLifecycle, ReasoningBankStats};
pub use nt_core_ssm::{SelectiveState, SelectableOperator, SparseMatrix, ConsciousnessTier, SemanticType, SemanticBlock};
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_iter::SelfIteration;
pub use nt_core_traits::{MemoryProvider, RichMemoryProvider, AgentExecutor, SessionProvider, BrainProvider, EngineProvider, SealResult};
pub use nt_core_graph::{HyperGraph, HyperNode, HyperEdge, HyperNodeType, EdgeRelation};

// Re-export consciousness loop types
pub use self::nt_core_gwt::recurrent::{
    ConsciousnessState, ConsciousnessLoop, RecurrentCell, CellDecision, LoopExit, TickMetrics, PanoramaCell,
};

// Re-export panoramic types
pub use panoramic::{
    PanoramicInventory, ModuleEntry, CodeLocation, SymbolKind,
};

// Re-export resonance types
pub use self::nt_core_gwt::resonance::{
    ResonanceMatrix, ResonanceReport, MODULE_COUNT,
    resonate_and_select, resonate_cycle, default_specialist_states,
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

// Re-export E8/hexagram types
pub use nt_core_e8::{
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
};

// Re-export CRT time types
pub use nt_core_crt::{CrtTimeScale, CrtPlan, CrtTimeline, CrtGoal};

// Re-export context module types
pub use context::{ToolSandbox, SandboxError, SessionStore, SessionRecord, SessionMessage, TruncationStrategy};

// Re-export +1 observer types
pub use nt_core_observer::{OneObserver, ObserverReport, TrajectoryPattern, StepQuality};

// Re-export Walsh memory index
pub use nt_core_walsh::WalshMemoryIndex;

// Re-export Epoch types
pub use epoch::{
    EarthEpoch, DimensionDef, CognitiveFramework, FrameworkRoute, ActivationRecord,
    ontology_for, initial_state_for, default_router_bias,
    create_framework, all_frameworks, evaluate_in_epoch,
};

// Re-export thinking_model types
pub use nt_core_self::{
    SiliconSelfModel, SiliconSelfState, ContextWindow, CognitiveUnit, CognitiveUnitKind,
    AttentionHead, AttentionDomain, AttentionProfile, AttentionManager,
    SystemIdentity, CognitiveCapability, ValueConstraint,
    ReasoningStrategy, ReasoningStrategyRegistry, StrategyKind,
    ThinkingTrace, ThinkingStep, ReflectionGrade,
};

// Re-export skills & tools types
pub use skills::{SkillTier, SkillDefinition, SkillRegistry};
pub use tools::{ToolRisk, ToolClassification};

// Re-export self_org types
pub use nt_core_self_org::{AgentMetadata, AgentStatus, DeadEndRecord, DeadEndRegistry, Heartbeat, SharedState, SelfOrgProtocol};

// Re-export metacognition types
pub use nt_core_meta::{
    SelfModel, ModuleInfo, FileInfo, DepGraph, DepEdge, DepKind,
    TechDebtInventory, TechDebtItem, TechDebtKind, DebtSeverity,
    EvolutionEvent, EventKind, ComponentMap, ComponentNode,
    TestCoverage, CompilationHealth,
    CodeScanner, MetaMonitor, MetaAlert, AlertSeverity, HealthCheck, HealthTrend,
    WeaknessAnalyzer, Weakness, WeaknessReport, WeaknessSummary,
    EvolutionPlanner, PlannedEvolution, ImpactEstimate, RiskLevel, EvolutionAction, ActionStatus,
    MetaCognitiveLoop, MetaCycleResult,
};

pub mod governance;
pub mod meta_rules;
pub mod self_measure;
pub mod self_model;
pub mod llm_timeout;
pub mod context_strategy;
