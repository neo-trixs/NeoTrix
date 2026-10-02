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
/// —— 原为「**硬编码满分，从不调 LLM**」，与它自己的文件名不符。
///
/// ✅ **该问题已于 2026-10-02 修复**（P0，见
/// `docs/architecture/EMERGENCE-PLAN-2026-10-02.md`）：
/// `llm_judge::evaluate_response` 曾用 `let _ = (prompt, response);`
/// 丢弃入参、恒给 `config.max_score`、理由硬编码 `"Full score"`
/// ⇒ **任何输入都返回满分**，等于自动为「已涌现」提供证据。
///
/// 现在它**显式拒绝评分**：签名返回 `Option<JudgeResult>`，
/// 有判据但无 rubric 信号时返回 `None` + `REFUSAL_REASON`，
/// 只有「无判据」/「总权重为 0」两个退化配置才给出良定义的 0 分。
/// 由 `test_no_input_can_ever_score_max` **反向锁**保证：
/// 36 组（判据数 × max × 响应）**无一能拿到 max**。
///
/// ⚠️ 本模块仍**未接入生产调用方**（`evaluate_response` 零外部引用）。
/// 接入前需注意：它现在**拒绝**评分，所以任何依赖它给分的上游都会拿到 `None`。
pub mod eval_engine;

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
    outcome_from_regression, ExperimentOutcome, ExperimentRunner,
};

/// 单元测试（`nt_evolution_runner` 的非空门证明）。
#[cfg(test)]
#[path = "tests/nt_evolution_runner_tests.rs"]
mod nt_evolution_runner_tests;

// 2026-09-30 逐个实测后的状态（原先是两行笼统注释「已声明但内部编译错误」）：
// · `otel_bridge`（8 文件 / 1,781 行 / **19 个测试**）实测 **0 编译错误**
//   ⇒ 属笼统注释的**误伤**，已恢复声明。⚠️ 它目前**零生产消费者**
//   （仅本文件提及），恢复是为了让那 19 个测试真正参与验证，
//   而不是宣称它已被业务使用 —— 后者是另一个判断。
// · ~~`session_replay` → 7 错误~~ ✅ **已修并恢复**（2026-09-30：
//   `BudgetAlert` 含 f64 却 derive(Eq)；`.into` 少括号；`metadata` 误当 Option；
//   另有时间戳重复、主导者选取不确定、游标起点三处真缺陷）
// · 上方的 `eval_engine` 注释掉了**另一个**模块（`case_level_regressions`
//   等），它有**明确前置条件**（`llm_judge.rs` 硬编码满分，属假测量）
//   ⇒ 那是**有意的**停用，不是误伤，两者不要混为一谈。
pub mod session_replay;
pub mod otel_bridge; // 实测 0 错误 ⇒ 恢复（2026-09-30）
