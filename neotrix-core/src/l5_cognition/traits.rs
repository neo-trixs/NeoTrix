//! L5 Cognition Layer Traits
//!
//! 认知层合约: 核心推理 (nt_core) + 自我进化 (nt_mind)
//! 吸收来源: RD-Agent (R&D 自动化), PentestCode (多智能体协调), Sentrux (质量门禁)

use serde::{Deserialize, Serialize};

/// 推理任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTask {
    pub task_type: TaskType,
    pub input: serde_json::Value,
    pub context: Vec<String>,
    pub constraints: Vec<String>,
    pub priority: u8,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Analyze,
    Plan,
    Execute,
    Review,
    Research,
    Synthesize,
}

/// 推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResult {
    pub task: ReasoningTask,
    pub output: serde_json::Value,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub reasoning_chain: Vec<String>,
}

/// 多智能体任务 — PentestCode 13-agent 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub agent_type: String,
    pub instruction: String,
    pub tools: Vec<String>,
    pub context: serde_json::Value,
}

/// 多智能体结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub agent_type: String,
    pub output: serde_json::Value,
    pub confidence: f64,
    pub evidence_chain: Vec<String>,
}

/// 质量信号 — Sentrux 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _QualitySignal {
    pub score: u32, // 0-10000
    pub metrics: QualityMetrics,
    pub violations: Vec<QualityViolation>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub modularity: f64,
    pub acyclicity: f64,
    pub depth: f64,
    pub equality: f64,
    pub redundancy: f64,
}

/// 质量违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityViolation {
    pub rule: String,
    pub severity: String,
    pub location: String,
    pub message: String,
}

/// R&D 自动化任务 — RD-Agent 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RndTask {
    pub hypothesis: String,
    pub experiment_design: String,
    pub expected_outcome: String,
    pub resources_needed: Vec<String>,
}

/// R&D 自动化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RndResult {
    pub hypothesis: String,
    pub outcome: String,
    pub success: bool,
    pub learnings: Vec<String>,
    pub next_steps: Vec<String>,
}

/// 认知状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CognitionSnapshot {
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub quality_score: Option<u32>,
    pub agent_count: usize,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

// ⛔ **2026-10-07 删除两处零消费副本**（roadmap T1-4）。
//
// 本文件原在 `:117` 与 `:127` 各定义一份 `SessionSnapshot` / `DistillationResult`，
// 与 `nt_mind/foundation/l1_wrappers.rs:10` / `:85` 的**活定义逐字相同**
// （字段名、类型、顺序、derive 列表全同）。
//
// 实测零消费（brace-aware 全仓搜索 `l5_cognition::traits::{SessionSnapshot,
// DistillationResult}`，含花括号导入形式，**排除本文件**）：
// ⛔ **零命中**。而 `l1_wrappers` 侧有活的 `SessionRecovery` / `UserDistillation`
// trait 消费者（`:26` / `:64-65` 的 adapter），它们返回的是**活定义**的那个。
//
// ⇒ 本文件**仍是有消费者的**（其他符号 12 个文件在用），
//   所以只删这两个重复类型，不动其余内容。
// ⇒ 未改为 `pub use` 再导出：本文件是 L5 顶层 trait 契约面，
//   向下引到 `nt_mind::foundation` 会给 traits 面引入具体实现路径；
//   零消费者状态下删除比转发更干净。

// ═══════════════════════════════════════════════════════════════════════
// L6 → L5 Trait Abstractions (跨层引用隔离)
//
// L5 认知层通过这些 trait 访问 L6 元认知层提供的能力，避免直接
// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
// 在 L6 模块中提供（impl L5Trait for L6Type）。
// ═══════════════════════════════════════════════════════════════════════

/// L6 EvalHarness 回归测试接口 — L5 SEAL 闭环消费
///
/// ## 为什么加 `Send + Sync`（2026-09-29）
///
/// `SelfIteratingBrain` 实现了 `l0_substrate::nt_core_traits::BrainHandle`，
/// 而后者是 `trait BrainHandle: Send + Sync {}`。当把
/// `Option<Box<dyn EvalHarnessApi>>` 存进 brain 字段时，
/// 编译器报 `cannot be shared between threads safely`。
///
/// ⇒ 约束必须落在 **trait 定义**上，不能靠调用方自觉
/// （`Box<dyn EvalHarnessApi>` 不会因为持有者需要 Send 就自动变 Send）。
///
/// 现有唯一实现 `nt_mind_eval_harness::EvalHarness` 持有
/// `Arc<dyn LlmProvider>` 与 `Arc<ConsciousnessGoldStandard>`，
/// 两者本身已是 `Send + Sync` ⇒ 加约束**不破坏既有实现**。
///
/// ⛔ 若将来出现需要内部可变状态（`Cell`/`RefCell`）的实现，
///   正确做法是 `Mutex`，**不是**去掉这里的约束。
pub trait EvalHarnessApi: Send + Sync {
    fn generate_regression_test(&self, candidate: &str) -> RegressionCase;
    fn run_regression_test(&self, case: &RegressionCase) -> RegressionResult;
}

/// **L6 → L5 评测闸门工厂**（2026-09-29 加，B5b）
///
/// ## 为什么需要它
///
/// A9 已在 `SelfIteratingBrain` 加了 `eval_harness` 注入点，但**没人注入**
/// —— 那正是本轮刚根治的 `check-doc-drift` 同一个病：
/// 「实现了但永远收不到调用，等于没有」。
///
/// 而 L5 不能自己 `new` 一个 `EvalHarness`：
/// ① 它是 L6 具体类型 ⇒ 违反本文件 :133-139 的跨层隔离约定
/// ② `EvalHarness::new_default` 需要 `Vec<ModelSpec>` / `Arc<dyn LlmProvider>` /
///    `judge_model` —— 全部是 L6/L1 的具体类型
///
/// ⇒ 本 trait 是**跨层工厂的既有模式**（与 :183 `GoldStandardApi`、
/// :207 `EvolutionHarnessApi` 同族）：**L5 声明，L6 实现**。
/// 与 `EvolutionHarnessApi::new_harness() -> Self where Self: Sized` 的区别：
/// 那个返回具体类型（L5 仍需知道 L6 类型名），本 trait 返回
/// `Box<dyn EvalHarnessApi>` ⇒ **L5 全程只见 trait，不见 L6 类型**。
pub trait EvalHarnessFactory: Send + Sync {
    /// 构造一个评测闸门。返回 `None` = 无法构造（无 provider / 未配置）。
    ///
    /// ⛔ **不得 panic、不得阻塞**。这是启动路径上的调用点。
    fn make_eval_harness(&self) -> Option<Box<dyn EvalHarnessApi>>;
}

/// 回归测试用例 (L5 侧轻量数据结构, 与 L6 EvalHarness 解耦)
#[derive(Debug, Clone)]
pub struct RegressionCase {
    pub id: String,
    pub candidate: String,
    pub forbidden_tokens: Vec<String>,
    pub required_categories: Vec<String>,
}

/// 回归测试结果
#[derive(Debug, Clone)]
pub struct RegressionResult {
    pub passed: bool,
    pub reasons: Vec<String>,
}

/// L6 ConsciousnessGoldStandard 意识金标接口 — L5 控制蒸馏消费
///
/// L5 通过此 trait 访问 L6 `ConsciousnessGoldStandard`, 不直接依赖具体类型。
pub trait GoldStandardApi: Send + Sync {
    fn new_gold_standard() -> Self where Self: Sized;
}

/// L6 ConsciousnessMonitor 意识监控接口 — L5 后台循环消费
pub trait ConsciousnessMonitorApi {
    fn new_monitor() -> Self where Self: Sized;
    fn observe(&mut self);
    fn get_report(&self) -> ConsciousnessAwarenessReport;
}

/// 意识感知报告 (L5 侧数据结构)
#[derive(Debug, Clone)]
pub struct ConsciousnessAwarenessReport {
    pub consciousness: f64,
    pub coherence: f64,
    pub phi: f64,
}

/// L6 EvolutionHarness 超越层进化接口 — L5 后台循环消费
///
/// 返回 `serde_json::Value` 以避免 L5 依赖 L6 内部类型
/// (MetaObservationReport, ConsonanceReport, LoopReport 等)。
/// 调用方按需 `.get("field")` 提取字段。
pub trait EvolutionHarnessApi {
    fn new_harness() -> Self where Self: Sized;
    fn harness_infos_from_registry_export(json: &str) -> (Vec<RegistryNodeInfo>, Vec<String>) where Self: Sized;
    fn harness_run_cycle(&mut self, snapshot_json: &serde_json::Value, infos: &[RegistryNodeInfo]) -> serde_json::Value;
    fn harness_persist_suggestions(
        &mut self,
        kb: &crate::l5_cognition::l1_facade::KnowledgeBase,
        report: &serde_json::Value,
    ) -> usize;
    fn harness_actionable_suggestions(report: &serde_json::Value, threshold: f64) -> Vec<RegistrySuggestion>;
}

/// 能力网节点信息 (L5 侧, 用于 EvolutionHarness trait)
#[derive(Debug, Clone)]
pub struct RegistryNodeInfo {
    pub node_id: String,
    pub domain: String,
    pub layer: String,
    pub constellation: String,
    pub maturity: f64,
}

/// 超越层进化建议 (L5 侧, 用于 EvolutionHarness trait)
#[derive(Debug, Clone)]
pub struct RegistrySuggestion {
    pub node_id: String,
    pub resonance: f64,
    pub suggestion: String,
}

/// L6 SystemMetrics 系统指标接口 — L5 后台循环自改进消费
pub trait SystemMetricsApi {
    fn new_system_metrics(
        success_rate: f64,
        avg_tokens: f64,
        skill_hit_rate: f64,
        crystallization_rate: f64,
        knowledge_retention: f64,
        error_recovery_rate: f64,
        timestamp: i64,
    ) -> Self where Self: Sized;
}
