pub mod knowledge_gap_detector;
pub mod metacognition_loop;
pub mod monitor;
pub mod nt_core_arch_lint;
pub mod nt_core_meta_auditor;
pub mod planner;
pub mod scanner;
pub mod self_model;
pub mod weakness;

pub use knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
};
pub use metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert, MetaMonitor};
pub use nt_core_arch_lint::ArchLint;
pub use nt_core_meta_auditor::MetaAuditor;
pub use planner::{
    weakness_to_goals, ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal,
    MetaGoalBridge, PlannedEvolution, RiskLevel,
};
pub use scanner::CodeScanner;
pub use self_model::{
    CompilationHealth, ComponentMap, ComponentNode, DebtSeverity, DepEdge, DepGraph, DepKind,
    EventKind, EvolutionEvent, FileInfo, ModuleInfo, SelfModel, TechDebtInventory, TechDebtItem,
    TechDebtKind, TestCoverage,
};
pub use weakness::{Weakness, WeaknessAnalyzer, WeaknessReport, WeaknessSummary};
