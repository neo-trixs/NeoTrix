//! # L9 — 超验层 (Transcendent)
//!
//! 观察自身观察过程。不参与决策，只参与反思。
//! 科幻映射: 火鸡科学家 / Stand Alone Complex / Lain 上帝视角
//!
//! ## 规则
//! - L9 只读不写 — 可读取任何层的数据，但不可修改
//! - L9 不参与调度决策
//! - L9 输出是观察报告 → 写入 L3 供反思

pub use crate::core::nt_core_meta as meta;

pub use crate::core::nt_core_meta::self_model::{
    SelfModel, ModuleInfo, FileInfo, DepGraph, DepEdge, DepKind,
    TestCoverage, CompilationHealth, TechDebtInventory,
    TechDebtItem, TechDebtKind, DebtSeverity,
    EvolutionEvent, EventKind, ComponentMap, ComponentNode,
};
pub use crate::core::nt_core_meta::scanner::CodeScanner;
pub use crate::core::nt_core_meta::monitor::{
    MetaMonitor, MetaAlert, AlertSeverity, HealthCheck, HealthTrend,
};
pub use crate::core::nt_core_meta::weakness::{
    WeaknessAnalyzer, Weakness, WeaknessReport, WeaknessSummary,
};
pub use crate::core::nt_core_meta::planner::{
    EvolutionPlanner, PlannedEvolution, ImpactEstimate,
    RiskLevel, EvolutionAction, ActionStatus, MetaGoal, MetaGoalBridge,
};
pub use crate::core::nt_core_meta::metacognition_loop::{
    MetaCognitiveLoop, MetaCycleResult,
};

// +1 Observer（元认知+PRM头合并）
pub use crate::core::nt_core_observer::{
    OneObserver, ObserverReport, E8Observer,
};
pub use crate::core::nt_core_observer_error::{
    ObserverErrorRecovery, RetryConfig, CircuitBreaker, CircuitState,
    FallbackHandler, ErrorRecoveryError,
};

// 评估基准
pub use crate::core::nt_core_meta::eval_bench::{
    Benchmark, BenchmarkMetric, BenchmarkRunner, BenchmarkSuite,
    EvalResult, RunnerStats,
};

// 知识缺口检测
pub use crate::core::nt_core_meta::knowledge_gap_detector::{
    KnowledgeGapDetector, KnowledgeGap, GapReport, GapCluster, GapCategory,
};

// 能力观察者（TurkeyScientist - 在 l7_capability/observer.rs 中）
pub use crate::core::l7_capability::{
    TurkeyScientist, IllusionRisk, IllusionReport,
};
