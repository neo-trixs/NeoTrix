//! # NeoTrix Core — 纯理论/数据模型层
//!
//! 零外部依赖层，仅包含核心数据结构和 trait 定义。

pub mod nt_core_cap;
pub mod nt_core_knowledge;
pub mod nt_core_edit;
pub mod nt_core_bank;
pub mod nt_core_event;
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
pub mod file_parser;
pub mod layered_memory;
#[cfg(feature = "rkyv-storage")]
pub mod nt_core_rkyv;

// Re-export 主要类型到 core 层顶层
pub use nt_core_cap::CapabilityVector;
pub use nt_core_knowledge::{KnowledgeSource, KnowledgeProvider, TaskType, RewardSource, SourceAccessTracker, SourceAccessRecord};
pub use nt_core_accessor::{Accessor, AccessionReport, SourceType, UrlAccessor};
pub use nt_core_edit::{SelfEdit, MicroEdit, ToolCall};
pub use nt_core_bank::{ReasoningBank, ReasoningMemory, TemporalContext, MemoryTier, MemoryLifecycle, ReasoningBankStats};
pub use nt_core_absorb::AbsorbValidator;
pub use nt_core_iter::SelfIteration;
pub use nt_core_traits::{MemoryProvider, RichMemoryProvider, AgentExecutor, ToolProvider, ToolDef, ToolOutput, SessionProvider, BrainProvider, EngineProvider, SealResult};
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
pub use context::{ToolSandbox, SandboxError, SessionStore, SessionRecord, SessionMessage, TruncationStrategy, HookRegistry, LifecycleEvent};

// Re-export +1 observer types
pub use nt_core_observer::{OneObserver, ObserverReport, TrajectoryPattern, StepQuality};

// Re-export Walsh memory index
pub use nt_core_walsh::WalshMemoryIndex;

// 2026-09-30: nt_core_self 冻结镜像已删（真身在 neotrix-core l6_meta/nt_core_self）。
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
};  // 2026-09-30: MetaCognitiveLoop/MetaCycleResult 随冻结镜像移除

pub mod governance;
pub mod meta_rules;
pub mod self_measure;
pub mod self_model;
pub mod llm_timeout;
pub mod context_strategy;
/// CJK 字符判定的**唯一事实源**（2026-09-29）：`is_cjk_han`（分词）/ `is_cjk_wide`（计量）。
/// 此前全仓 8 个私有副本、4 种口径，且两个「单一事实源」互指一个不存在的模块。
pub mod nt_cjk;
pub mod shared_types;
pub mod nt_core_approval;

// Re-export unified SelfModel types
pub use nt_core_meta::unified_self_model::{
    StaticIdentityModel, DynamicPerformanceModel, ValueFunctionModel, SelfState, ValueWeight,
    ModuleInfo as UnifiedModuleInfo, FileInfo as UnifiedFileInfo,
    DepGraph as UnifiedDepGraph, DepEdge as UnifiedDepEdge, DepKind as UnifiedDepKind,
    ComponentMap as UnifiedComponentMap, ComponentNode as UnifiedComponentNode,
    TestCoverage as UnifiedTestCoverage, CompilationHealth as UnifiedCompilationHealth,
    TechDebtInventory as UnifiedTechDebtInventory, TechDebtItem as UnifiedTechDebtItem,
    TechDebtKind as UnifiedTechDebtKind, DebtSeverity as UnifiedDebtSeverity,
    EvolutionEvent as UnifiedEvolutionEvent, EventKind as UnifiedEventKind,
    SELF_HISTORY,
};

// Re-export unified ConsciousnessState types
pub use nt_core_gwt::unified_consciousness::{
    ConsciousnessPhase, EmotionalState, VadEmotionalState,
    ConsciousnessObserverState, BranchState, Alert,
    ConsciousnessSnapshot, CapabilityConsciousnessState,
    LegacyConsciousnessPhase, CrystalConsciousnessState, EvolutionPhase,
    Layer,
};
