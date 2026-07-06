pub mod self_model;
pub mod scanner;
pub mod monitor;
pub mod weakness;
pub mod planner;
pub mod metacognition_loop;

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
pub use metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
