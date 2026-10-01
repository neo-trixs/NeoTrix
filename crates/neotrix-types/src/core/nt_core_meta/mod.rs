pub mod self_model;
pub mod unified_self_model;
pub mod scanner;
pub mod monitor;
pub mod weakness;
pub mod planner;

pub use self_model::{
    SelfModel, ModuleInfo, FileInfo, DepGraph, DepEdge, DepKind,
    TestCoverage, CompilationHealth, TechDebtInventory, TechDebtItem,
    TechDebtKind, DebtSeverity, EvolutionEvent, EventKind,
    ComponentMap, ComponentNode,
};
pub use scanner::CodeScanner;
pub use monitor::{MetaMonitor, MetaAlert, AlertSeverity, HealthCheck, HealthTrend};
pub use weakness::{WeaknessAnalyzer, Weakness, WeaknessReport, WeaknessSummary};
pub use planner::{EvolutionPlanner, PlannedEvolution, ImpactEstimate, RiskLevel, EvolutionAction, ActionStatus};
// 2026-09-30: metacognition_loop 冻结镜像已删（真身在 neotrix-core l6_meta/nt_core_meta，
// 且 types 侧为严格子集：独有 pub 名 0 个）。

// Unified self-model types
pub use unified_self_model::{
    StaticIdentityModel, DynamicPerformanceModel, ValueFunctionModel,
    SelfState, SelfModel as UnifiedSelfModel,
    ModuleInfo as UnifiedModuleInfo, FileInfo as UnifiedFileInfo,
    DepGraph as UnifiedDepGraph, DepEdge as UnifiedDepEdge, DepKind as UnifiedDepKind,
    ComponentMap as UnifiedComponentMap, ComponentNode as UnifiedComponentNode,
    TestCoverage as UnifiedTestCoverage, CompilationHealth as UnifiedCompilationHealth,
    TechDebtInventory as UnifiedTechDebtInventory, TechDebtItem as UnifiedTechDebtItem,
    TechDebtKind as UnifiedTechDebtKind, DebtSeverity as UnifiedDebtSeverity,
    EvolutionEvent as UnifiedEvolutionEvent, EventKind as UnifiedEventKind,
    ValueWeight, SELF_HISTORY,
};
