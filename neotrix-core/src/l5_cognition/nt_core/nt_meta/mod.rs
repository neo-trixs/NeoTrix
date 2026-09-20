//! Backward-compatibility re-exports.
//!
//! All modules have been moved to their correct L6 layer:
//! `crate::l5_cognition::l1_facade::*`
//!
//! This file re-exports everything for backward compatibility.
//! New code should import from `crate::l5_cognition::l1_facade` directly.

pub use crate::l5_cognition::l1_facade::knowledge_gap_detector;
pub use crate::l5_cognition::l1_facade::metacognition_loop;
pub use crate::l5_cognition::l1_facade::monitor;
pub use crate::l5_cognition::l1_facade::nt_core_arch_lint;
pub use crate::l5_cognition::l1_facade::nt_core_meta_auditor;
pub use crate::l5_cognition::l1_facade::planner;
pub use crate::l5_cognition::l1_facade::scanner;
pub use crate::l5_cognition::l1_facade::self_model;
pub use crate::l5_cognition::l1_facade::weakness;

pub use crate::l5_cognition::l1_facade::knowledge_gap_detector::{
    GapCategory, GapCluster, GapReport, KnowledgeGap, KnowledgeGapDetector,
};
pub use crate::l5_cognition::l1_facade::metacognition_loop::{MetaCognitiveLoop, MetaCycleResult};
pub use crate::l5_cognition::l1_facade::monitor::{AlertSeverity, HealthCheck, HealthTrend, MetaAlert, MetaMonitor};
pub use crate::l5_cognition::l1_facade::nt_core_arch_lint::ArchLint;
pub use crate::l5_cognition::l1_facade::nt_core_meta_auditor::MetaAuditor;
pub use crate::l5_cognition::l1_facade::planner::{
    weakness_to_goals, ActionStatus, EvolutionAction, EvolutionPlanner, ImpactEstimate, MetaGoal,
    MetaGoalBridge, PlannedEvolution, RiskLevel,
};
pub use crate::l5_cognition::l1_facade::scanner::CodeScanner;
pub use crate::l5_cognition::l1_facade::self_model::{
    CompilationHealth, ComponentMap, ComponentNode, DebtSeverity, DepEdge, DepGraph, DepKind,
    EventKind, EvolutionEvent, FileInfo, ModuleInfo, SelfModel, TechDebtInventory, TechDebtItem,
    TechDebtKind, TestCoverage,
};
pub use crate::l5_cognition::l1_facade::weakness::{Weakness, WeaknessAnalyzer, WeaknessReport, WeaknessSummary};
