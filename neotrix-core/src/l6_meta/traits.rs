//! L6 Meta-Cognition Layer Traits
//!
//! 元认知层合约: 元认知协调 (nt_meta) + 自愈修复 (nt_repair) + 跨会话记忆 (nt_nexus)
//!
//! 注: 原 MetaLayer trait 已移除 — 无模块实现，保留在 APPENDIX_SIMULATION_PLATFORM.md 作为架构参考。

use serde::{Deserialize, Serialize};

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
