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
pub struct QualitySignal {
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
pub struct RndTask {
    pub hypothesis: String,
    pub experiment_design: String,
    pub expected_outcome: String,
    pub resources_needed: Vec<String>,
}

/// R&D 自动化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RndResult {
    pub hypothesis: String,
    pub outcome: String,
    pub success: bool,
    pub learnings: Vec<String>,
    pub next_steps: Vec<String>,
}

/// 认知状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitionSnapshot {
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub quality_score: Option<u32>,
    pub agent_count: usize,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// 会话快照数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub e8_state_sequence: Vec<u8>,
    pub message_count: u64,
    pub active_topics: Vec<String>,
    pub created_at: u64,
}

/// 蒸馏结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationResult {
    pub nodes_created: usize,
    pub edges_created: usize,
    pub avatar_confidence: f64,
}

// ═══════════════════════════════════════════════════════════════════════
// L6 → L5 Trait Abstractions (跨层引用隔离)
//
// L5 认知层通过这些 trait 访问 L6 元认知层提供的能力，避免直接
// `use crate::l6_meta::*` 造成向上依赖。Concrete implementations
// 在 L6 模块中提供（impl L5Trait for L6Type）。
// ═══════════════════════════════════════════════════════════════════════

/// L6 EvalHarness 回归测试接口 — L5 SEAL 闭环消费
pub trait EvalHarnessApi {
    fn generate_regression_test(&self, candidate: &str) -> RegressionCase;
    fn run_regression_test(&self, case: &RegressionCase) -> RegressionResult;
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
/// Marker trait: L5 仅持有 `Arc<dyn GoldStandardApi>` 引用,
/// 不直接依赖 L6 `ConsciousnessGoldStandard` 具体类型。
/// 构造在 L6 侧完成 (L6 impl 提供 `new_gold_standard()`)。
pub trait GoldStandardApi: Send + Sync {}

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
        kb: &crate::l5_cognition::kb_facade::KnowledgeBase,
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
