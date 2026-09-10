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
