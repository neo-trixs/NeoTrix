pub mod auto_inspector;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod arch_optimizer;

pub mod meta_cognition;

pub mod whale;

// 从 l5_cognition/nt_core/nt_meta 迁移到正确 L6 层的模块
pub mod self_model;
pub mod weakness;
pub mod nt_core_meta_auditor;
pub mod nt_core_arch_lint;
pub mod metacognition_loop;
pub mod planner;
pub mod scanner;
pub mod knowledge_gap_detector;
pub mod monitor;

pub use arch_optimizer::SelfArchitectureOptimizer;
pub use whale::{AdaptiveSwitching, OptimizationPhase, WHALEConfig, WhaleCycleDetector, WhaleCycleResult};

// Re-exports from l6_meta for nt_meta namespace
pub use crate::l6_meta::runtime_monitor::RuntimeMonitor;
pub use crate::l6_meta::evolving_evaluator::EvolvingEvaluator;

// Re-export moved modules for backward compatibility
pub use self_model::{
    CompilationHealth, ComponentMap, ComponentNode, DebtSeverity, DepEdge, DepGraph, DepKind,
    EventKind, EvolutionEvent, FileInfo, ModuleInfo, SelfModel, TechDebtInventory, TechDebtItem,
    TechDebtKind, TestCoverage,
};
pub use weakness::{Weakness, WeaknessAnalyzer, WeaknessReport, WeaknessSummary};
pub use nt_core_meta_auditor::MetaAuditor;
pub use nt_core_arch_lint::ArchLint;
pub use metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use planner::{
    weakness_to_goals, ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal,
    MetaGoalBridge, PlannedEvolution, RiskLevel,
};
pub use scanner::CodeScanner;
pub use knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
};
pub use monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert, MetaMonitor};
