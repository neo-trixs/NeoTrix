//! L6 Meta-Cognition Layer Traits
//!
//! 元认知层合约: 元认知协调 (nt_meta) + 自愈修复 (nt_repair) + 跨会话记忆 (nt_nexus)
//! 吸收来源: PentestCode (持久状态), Git Knowledge Loop (知识版本控制)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 元认知事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaEvent {
    pub event_type: MetaEventType,
    pub source: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 元认知事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetaEventType {
    SelfReflection,
    CrossSessionPattern,
    HealthCheck,
    RepairAction,
    EvolutionStep,
    GovernanceViolation,
}

/// 自愈动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAction {
    pub action_type: String,
    pub target: String,
    pub reason: String,
    pub expected_outcome: String,
    pub actual_outcome: Option<String>,
    pub success: Option<bool>,
}

/// 跨会话记忆 — PentestCode 持久状态 + Git Knowledge Loop 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionMemory {
    pub session_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub findings: Vec<Finding>,
    pub decisions: Vec<Decision>,
    pub learnings: Vec<Learning>,
    pub next_steps: Vec<String>,
}

/// 发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub finding_type: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub decision: String,
    pub rationale: String,
    pub alternatives: Vec<String>,
    pub outcome: Option<String>,
}

/// 学习
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub topic: String,
    pub insight: String,
    pub source: String,
    pub applicability: Vec<String>,
}

/// 治理合规状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatus {
    pub compliant: bool,
    pub violations: Vec<GovernanceViolation>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// 治理违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceViolation {
    pub rule: String,
    pub severity: String,
    pub location: String,
    pub description: String,
}

/// 元认知层核心合约
pub trait MetaLayer: Send + Sync {
    /// 初始化元认知层
    fn initialize(&mut self) -> Result<(), String>;

    /// 自我反思 — 分析系统状态
    fn self_reflect(&self) -> Result<MetaEvent, String>;

    /// 跨会话记忆 — 恢复之前会话状态 (PentestCode 吸收)
    fn restore_session(&mut self, session_id: &str) -> Result<CrossSessionMemory, String>;

    /// 保存会话 — 持久化当前状态
    fn save_session(&self, memory: &CrossSessionMemory) -> Result<(), String>;

    /// 跨会话模式挖掘 — 发现跨会话的重复模式
    fn mine_patterns(
        &self,
        sessions: &[CrossSessionMemory],
    ) -> Result<Vec<serde_json::Value>, String>;

    /// 自愈修复 — 检测问题并执行修复
    fn detect_and_repair(&mut self) -> Result<Vec<RepairAction>, String>;

    /// 治理合规检查
    fn check_governance(&self) -> Result<GovernanceStatus, String>;

    /// 进化步骤 — SEAL pipeline 元认知层
    fn evolution_step(&mut self) -> Result<MetaEvent, String>;

    /// 知识版本控制 — Git Knowledge Loop 吸收
    fn version_knowledge(
        &self,
        knowledge: &serde_json::Value,
        message: &str,
    ) -> Result<String, String>;

    /// 获取元认知状态快照
    fn snapshot(&self) -> MetaSnapshot;
}

/// 元认知状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSnapshot {
    pub active_sessions: usize,
    pub repair_count: u64,
    pub governance_compliant: bool,
    pub evolution_cycle: u64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}
