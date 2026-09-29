pub mod auto_inspector;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod arch_optimizer;

pub mod gwt_router;

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
pub mod nt_deep_route;

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

/// 进化验证底座（D-3 裁决产物）。
///
/// ⛔ **未接线**：`eval_engine` 三个文件（650 行）仍在磁盘上但**不声明**。
/// 它们零生产消费者，且 `llm_judge.rs:86` 是 `let normalized = 1.0;`
/// —— **硬编码满分，从不调 LLM**，与它自己的文件名不符。
/// ⛔ 接线前必须先修 judge（否则是「永远满分」的假测量）。
/// 见 `docs/architecture/DECISIONS-REQUIRED-2026-09-29.md` D-3。
// pub mod eval_engine;

/// 自进化验证底座 —— 补上「改了一版，怎么知道变好了」这一层。
///
/// 与 `eval_engine` 的区别：那个是**通用评测原语**（数据集/变体/判官算术），
/// 这个是**进化判决**（臂中立 A/B + 噪声地板 + veto + 预注册 + 账本）。
///
/// L5 通过 `l5_cognition::traits::EvalHarnessApi` 消费本层，
/// **不直接 `use crate::l6_meta::*`**（见 traits.rs:133-139 的跨层隔离约定）。
pub mod nt_evolution_eval;

/// 实验运行器 —— 把 `nt_evolution_eval` 的判决能力变成**可调用的能力**。
///
/// ⛔ 2026-09-29 之前 `nt_evolution_eval` 零消费者（754 行 + 19 测试从未被调用）
/// ⇒ 本模块是它的接线点。见 `nt_evolution_runner.rs` 的模块文档 §为什么需要这一层。
pub mod nt_evolution_runner;

pub use nt_evolution_eval::nt_evolution_eval::{
    case_level_regressions, estimate_noise_floor, judge_ab, judge_case, noise_threshold, pass_rate,
    safety_regressions, Arm, CaseOutcome, CaseSpec, ClaimFidelity, EnvFingerprint, Ledger,
    LedgerEntry, NoiseFloor, Preregistration, Unverified, Verdict, Veto,
};

/// 单元测试（`nt_evolution_eval` 的非空门证明）。
#[cfg(test)]
#[path = "tests/nt_evolution_eval_tests.rs"]
mod nt_evolution_eval_tests;

pub use nt_evolution_runner::{
    outcome_from_regression, prereg_from_hypothesis, ExperimentOutcome, ExperimentRunner,
};

/// 单元测试（`nt_evolution_runner` 的非空门证明）。
#[cfg(test)]
#[path = "tests/nt_evolution_runner_tests.rs"]
mod nt_evolution_runner_tests;

// NOTE: 以下模块已声明但内部编译错误待修复，暂时注释
// pub mod otel_bridge;
// pub mod session_replay;
