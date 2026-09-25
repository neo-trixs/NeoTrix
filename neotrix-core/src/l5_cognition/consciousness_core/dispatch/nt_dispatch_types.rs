//! 子任务类型 — ConsciousTask / Report / Context（含实体路由决议类型）（由 `dispatch.rs` 纯搬移拆分，行为零变更）

use serde::{Deserialize, Serialize};

use super::super::external_closure::ExternalClosureReport;

// ─── 子任务类型 ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousTask {
    pub id: String,
    pub summary: String,
    pub capability_tag: String,
    pub domain: String,
    pub specialist: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAllocation {
    pub task: ConsciousTask,
    pub provider: AllocationProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationProvider {
    Internal {
        node_id: String,
        path: Vec<String>,
        cost: f64,
    },
    External { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskLoopReport {
    pub instruction: String,
    pub routed_skill: String,
    pub allocations: Vec<TaskAllocation>,
    pub internal_count: usize,
    pub external_gap_count: usize,
    pub strengthening_actions: usize,
    pub external_gaps: Vec<String>,
    pub external_closures: Vec<ExternalClosureReport>,
    pub internal_results: Vec<InternalExecutionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessStepProgress {
    pub index: usize,
    pub total: usize,
    pub kind: String,
    pub capability_tag: String,
    pub summary: String,
    pub status: String,
    pub output: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InternalExecutionResult {
    pub task_id: String,
    pub summary: String,
    pub provider_path: Vec<String>,
    pub executed: bool,
    pub output: String,
}

// T27c 42 词复核（skills/index.json 147 triggers vs 静态表关键词，简版）：
// L3 专属词：仅 skill triggers 有、无静态关键词（如 TDD/RAG/MCP/gitleaks/ADR/changelog）。
// 需静态兜底词：仅静态表有（如合并pdf/断点续传/分镜提取）；两者均无时走 DirectLlm。
// 重叠词 21 个一律 L3 优先：安全/审计/架构→shield 系 skill；测试→tdd；漏洞扫描→agentic_scan；
// 审查/设计/诊断/重构/吸收→各自 skill；L3 未命中才落静态表（复用 decompose 既有入口）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityRouteDecision {
    Skill {
        skill_id: String,
        agent_id: Option<String>,
    },
    Agent {
        agent_id: String,
    },
    Static {
        capability: String,
        layer: String,
        role: String,
    },
    DirectLlm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub workspace_id: String,
    pub agent_id: String,
    pub task_id: Option<String>,
    pub skill_id: Option<String>,
    pub shared_memory: serde_json::Value,
    pub available_tools: Vec<String>,
}
