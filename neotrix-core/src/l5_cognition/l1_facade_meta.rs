//! L1 Facade — nt_meta/evolution/governance 子门面（由 `l1_facade.rs` 拆分）
//!
//! 领域：nt_meta / 进化 / 审计 / 治理类 L6 具体类型再导出。
//! 单一事实源仍在 L6，此处仅 re-export；`l1_facade.rs` 以
//! `pub use self::l1_facade_meta::*;` 回导，保持外部 `l1_facade::Xxx` 路径零断裂。

// ─── L6 nt_meta sub-modules ─────────────────────────────────────────────────
pub use crate::l6_meta::nt_meta::arch_optimizer; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::auto_inspector; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::knowledge_gap_detector; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::metacognition_loop; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::monitor; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::nt_core_arch_lint; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::planner; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::scanner; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::self_model; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::weakness; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 nt_meta type re-exports ─────────────────────────────────────────────
pub use crate::l6_meta::nt_meta::knowledge_gap_detector::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    GapCategory,
    GapCluster,
    GapReport,
    KnowledgeGap,
};
pub use crate::l6_meta::nt_meta::monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor::AuditorFinding; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::planner::EvolutionPlanner; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::planner::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    weakness_to_goals,
    ActionStatus,
    EvolutionAction,
    ImpactEstimate,
    MetaGoal,
    MetaGoalBridge,
    PlannedEvolution,
    RiskLevel as MetaRiskLevel,
};
pub use crate::l6_meta::nt_meta::self_model::ModuleInfo; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::self_model::SelfModel as MetaSelfModel; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::self_model::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    CompilationHealth,
    ComponentMap,
    ComponentNode,
    DebtSeverity,
    DepEdge,
    DepGraph,
    DepKind,
    EventKind,
    EvolutionEvent,
    FileInfo,
    TechDebtInventory,
    TechDebtItem,
    TechDebtKind,
    TestCoverage,
};
pub use crate::l6_meta::nt_meta::weakness::Weakness; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::weakness::WeaknessSummary; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::ArchLint; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::CodeScanner; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::KnowledgeGapDetector; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::MetaAuditor; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::MetaCognitiveLoop; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::MetaCycleResult; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::MetaMonitor; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::SelfArchitectureOptimizer; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::WeaknessAnalyzer; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_meta::WeaknessReport; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 coordination evolution/sensor (进化侧) ──────────────────────────────
// 注：同 block 内 watchdog/quality/verifier 属健康/自测侧，已归 `l1_facade_observer`。
pub use crate::l6_meta::coordination::nt_meta_sentrux::SentruxSensor; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::coordination::self_improvement::{SelfImprovementLoop, SystemMetrics}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 memory evolution (进化侧) ───────────────────────────────────────────
// 注：同 block 内 `meta_observer` 属观察侧，已归 `l1_facade_observer`。
pub use crate::l6_meta::memory::evolution_harness::EvolutionHarness; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::memory::transcendent_loop::LoopConfig; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 scheduler types ─────────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_scheduler::{default_scheduler, SchedulerEngine}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 constitution types ──────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_self_constitution::global_constitution; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self_constitution::Constitution; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self_constitution::ConstitutionLoader; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self_constitution::GovernanceConstitutionSelfTest; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self_constitution::RuleCategory; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 nexus types ─────────────────────────────────────────────────────────
pub use crate::l6_meta::nt_nexus::cross_session_memory::{
    CrossSessionMemory, CrossSessionMemorySelfTest, MemoryCategory,
}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 nt_core_kb_types ────────────────────────────────────────────────────
pub use neotrix_types::knowledge_access::{NodeType, RelationType}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 auto-inspector ──────────────────────────────────────────────────────
pub use crate::l6_meta::nt_meta::auto_inspector::AutoInspector; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 nt_core_self modules (L5 → L0 abstraction) ──────────────────────────
// Module re-exports (preserves submodule paths for L5 code)
pub use crate::l6_meta::nt_core_self::affective_interface; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::archive; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::attention_head; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::emotion_state; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::intrinsic_motivation; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::metacognitive_evaluator; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::reasoning_strategy; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::seal; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::self_audit; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::self_referential; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::silicon_self; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::skill_crystal; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::thinking_trace; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// Type re-exports (direct access without submodule)
pub use crate::l6_meta::nt_core_self::affective_interface::AffectiveInterface; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::archive::SiliconArchive; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::attention_head::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    AttentionDomain,
    AttentionManager,
    ThinkingMode,
    WeaponSet,
};
pub use crate::l6_meta::nt_core_self::emotion_state::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    EmotionDimension,
    EmotionEngine,
    EmotionLabel,
    EmotionReport,
};
pub use crate::l6_meta::nt_core_self::intrinsic_motivation::{
    IntrinsicMotivation, MotivationState,
}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::metacognitive_evaluator::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    CognitiveEvaluator,
    CognitiveHealthReport,
    FlagSeverity,
};
pub use crate::l6_meta::nt_core_self::reasoning_strategy::StrategyKind; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::seal::grpo::{GRPOLoop, GrpoConfig}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::seal::ConstitutionGate; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::self_audit::{
    converge_check,
    scan_build_status,
    scan_disk_pressure,
    scan_memory_pressure,
    scan_system_health,
    scan_test_flakiness,
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    AuditFinding,
    AuditReport,
    AuditSeverity,
    ConvergeCheckFn,
    MultiSignalEval,
    ToolGroundingMonitor,
};
pub use crate::l6_meta::nt_core_self::self_model::SelfModel; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::self_referential::SelfReferentialMonitor; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::silicon_self::{SiliconSelfModel, SiliconSelfState}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self::skill_crystal::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    CrystalRegistry,
    SkillCrystal,
    VerificationContract,
    VerificationStatus as CrystalVerificationStatus,
};
pub use crate::l6_meta::nt_core_self::thinking_trace::{
    ReflectionGrade, ThinkingStep, ThinkingTrace,
}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 value-function self-model ───────────────────────────────────────────
pub use crate::l6_meta::nt_core_self_model::SelfModel as ValueSelfModel; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 spec-driven pipeline ────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_absorb::spec_driven::{
    // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    EvolutionSpec,
    SpecDiff,
    SpecDrivenPipeline,
    SpecPipelineConfig,
    SpecPipelineStats,
    SpecStatus,
    SpecVerification,
    SpecVerifier,
};

// ─── L6 self-ref code ───────────────────────────────────────────────────────
pub use crate::l6_meta::nt_core_iter::self_ref_code::{MutationResult, SelfCodeMonitor}; // ALLOW: nt_meta/进化/governance 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
