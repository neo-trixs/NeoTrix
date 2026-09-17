//! Backward-compatibility re-exports.
//!
//! All modules have been moved to their correct L6 layer:
//! `crate::l6_meta::nt_meta::*`
//!
//! This file re-exports everything for backward compatibility.
//! New code should import from `crate::l6_meta::nt_meta` directly.

pub use crate::l6_meta::nt_meta::knowledge_gap_detector;
pub use crate::l6_meta::nt_meta::metacognition_loop;
pub use crate::l6_meta::nt_meta::monitor;
pub use crate::l6_meta::nt_meta::nt_core_arch_lint;
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor;
pub use crate::l6_meta::nt_meta::planner;
pub use crate::l6_meta::nt_meta::scanner;
pub use crate::l6_meta::nt_meta::self_model;
pub use crate::l6_meta::nt_meta::weakness;

pub use crate::l6_meta::nt_meta::knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
};
pub use crate::l6_meta::nt_meta::metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use crate::l6_meta::nt_meta::monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert, MetaMonitor};
pub use crate::l6_meta::nt_meta::nt_core_arch_lint::ArchLint;
pub use crate::l6_meta::nt_meta::nt_core_meta_auditor::MetaAuditor;
pub use crate::l6_meta::nt_meta::planner::{
    weakness_to_goals, ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal,
    MetaGoalBridge, PlannedEvolution, RiskLevel,
};
pub use crate::l6_meta::nt_meta::scanner::CodeScanner;
pub use crate::l6_meta::nt_meta::self_model::{
    CompilationHealth, ComponentMap, ComponentNode, DebtSeverity, DepEdge, DepGraph, DepKind,
    EventKind, EvolutionEvent, FileInfo, ModuleInfo, SelfModel, TechDebtInventory, TechDebtItem,
    TechDebtKind, TestCoverage,
};
pub use crate::l6_meta::nt_meta::weakness::{Weakness, WeaknessAnalyzer, WeaknessReport, WeaknessSummary};
