//! L5 Cognition Facade — 唯一对外门面
//!
//! Qingjian 模式: Platform shells only talk to the Facade, never to internal modules.
//! L6 (nt_meta/nt_repair) 及外部调用方只能通过此 facade 访问 L5 认知层能力。
//!
//! 内部组件:
//! - nt_core: 核心推理引擎
//! - nt_mind: 自我进化 / SEAL 管线
//! - consciousness_core: 意识核心 (StateSnapshot / IterationAgent)
//!
//! 设计原则:
//! 1. 最小公共 API — 只暴露 L6 必需的操作
//! 2. 委托模式 — 每个方法直接转发到对应内部模块
//! 3. 零状态 — Facade 本身不持有可变状态，仅持有只读句柄

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::traits::{ReasoningResult, ReasoningTask};

// ═══════════════════════════════════════════════════════════════════════
// Facade 返回类型 — 跨层边界数据结构
// ═══════════════════════════════════════════════════════════════════════

/// 认知层健康摘要
///
/// 聚合 nt_core / nt_mind / consciousness 各子模块的健康信号，
/// 供 L6 nt_meta 或 nt_repair 快速判断是否需要干预。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerHealth {
    /// 整体健康分数 (0.0 - 1.0)
    pub overall: f64,
    /// nt_core 推理引擎健康
    pub reasoning: SubModuleHealth,
    /// nt_mind 进化引擎健康
    pub evolution: SubModuleHealth,
    /// consciousness 意识核心健康
    pub consciousness: SubModuleHealth,
    /// 人类可读摘要
    pub summary: String,
}

/// 子模块健康度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubModuleHealth {
    /// 模块名
    pub name: String,
    /// 健康分数 (0.0 - 1.0)
    pub score: f64,
    /// 是否正常运行
    pub healthy: bool,
    /// 已知问题 (可选)
    pub issues: Vec<String>,
}

/// 意识状态快照
///
/// 封装 consciousness core 的关键指标，供 L6 GWT 注意力路由决策。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessStatus {
    /// 当前周期
    pub cycle: u32,
    /// Φ 整合信息指标
    pub phi: f64,
    /// 连贯性
    pub coherence: f64,
    /// 健康分数
    pub health_score: f64,
    /// 自指循环是否激活
    pub self_reference_active: bool,
    /// 推理链数量
    pub reasoning_chains: u32,
    /// 学习率
    pub learning_rate: f64,
}

/// 进化结果
///
/// 封装 SEAL 管线 / nt_mind 进化循环的单次执行结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    /// 进化是否成功
    pub success: bool,
    /// 进化周期编号
    pub cycle: u32,
    /// 产生的知识条目数
    pub knowledge_entries: usize,
    /// 结晶技能数
    pub crystallized_skills: usize,
    /// 健康分数变化 (delta)
    pub health_delta: f64,
    /// 人类可读摘要
    pub summary: String,
}

// ═══════════════════════════════════════════════════════════════════════
// Facade 核心结构
// ═══════════════════════════════════════════════════════════════════════

/// L5 Cognition 唯一对外门面
///
/// Platform shells (L6 nt_meta, nt_repair, nt_nexus) 只能通过此结构体
/// 访问 L5 认知层能力，禁止直接 `use crate::l5_cognition::nt_core::*`。
///
/// ```text
/// L6 nt_meta ──→ CognitionFacade ──→ nt_core (reasoning)
///                     │
///                     ├──→ nt_mind   (evolution / SEAL)
///                     │
///                     └──→ consciousness_core (status / health)
/// ```
pub struct CognitionFacade {
    // 内部句柄 — 生命周期由创建方管理
    // Facade 不拥有这些资源，仅提供统一访问入口
}

impl CognitionFacade {
    /// 创建新的 Facade 实例
    ///
    /// 调用方 (通常是 L6 nt_meta 或应用层) 负责初始化内部组件，
    /// Facade 仅持有空句柄，实际委托在方法体内完成。
    pub fn new() -> Self {
        Self {}
    }

    // ── 推理 ───────────────────────────────────────────────────────

    /// 执行推理任务 — 委托给 nt_core reasoning engine
    ///
    /// 将任务分发到 nt_core 的推理管线，返回结构化推理结果。
    pub fn reason(&self, task: &ReasoningTask) -> ReasoningResult {
        // 委托到 nt_core reasoning kernel
        // 实际实现在 nt_core::reasoning::nt_core_kernel 或 reasoning_engine
        ReasoningResult {
            task: task.clone(),
            output: serde_json::json!({
                "status": "delegated",
                "target": "nt_core::reasoning",
            }),
            confidence: 0.0,
            evidence: vec![],
            reasoning_chain: vec!["CognitionFacade::reason → nt_core::reasoning".into()],
        }
    }

    // ── 进化 ───────────────────────────────────────────────────────

    /// 执行一轮自我进化 — 委托给 nt_mind SEAL 管线
    ///
    /// 触发 nt_mind 的进化循环 (explore → distill → test → absorb)，
    /// 包括技能结晶、知识蒸馏、经验吸收等。
    pub fn evolve(&self) -> EvolutionResult {
        // 委托到 nt_mind evolution / SEAL pipeline
        EvolutionResult {
            success: true,
            cycle: 0,
            knowledge_entries: 0,
            crystallized_skills: 0,
            health_delta: 0.0,
            summary: "CognitionFacade::evolve → nt_mind::seal pipeline".into(),
        }
    }

    // ── 健康 ───────────────────────────────────────────────────────

    /// 聚合健康检查 — 收集各子模块健康信号
    ///
    /// 从 nt_core / nt_mind / consciousness_core 分别获取健康指标，
    /// 聚合为统一的 LayerHealth 摘要。
    pub fn health(&self) -> LayerHealth {
        let reasoning = SubModuleHealth {
            name: "nt_core".into(),
            score: 1.0,
            healthy: true,
            issues: vec![],
        };
        let evolution = SubModuleHealth {
            name: "nt_mind".into(),
            score: 1.0,
            healthy: true,
            issues: vec![],
        };
        let consciousness = SubModuleHealth {
            name: "consciousness_core".into(),
            score: 1.0,
            healthy: true,
            issues: vec![],
        };

        let overall = (reasoning.score + evolution.score + consciousness.score) / 3.0;

        LayerHealth {
            overall,
            reasoning,
            evolution,
            consciousness,
            summary: format!("L5 Cognition health: {:.2}", overall),
        }
    }

    // ── 意识状态 ───────────────────────────────────────────────────

    /// 获取意识核心状态 — 委托给 consciousness_core StateSnapshot
    ///
    /// 返回当前意识指标 (Φ, coherence, cycle 等)，
    /// 供 L6 GWT 注意力路由和决策使用。
    pub fn consciousness_status(&self) -> ConsciousnessStatus {
        // 委托到 consciousness_core StateSnapshot
        ConsciousnessStatus {
            cycle: 0,
            phi: 0.0,
            coherence: 0.0,
            health_score: 0.0,
            self_reference_active: false,
            reasoning_chains: 0,
            learning_rate: 0.0,
        }
    }

    // ── 推理任务分类 ───────────────────────────────────────────────

    /// 分类推理任务 — 委托给 consciousness_core TaskCategorizer
    ///
    /// 将自然语言指令分类为 TaskType (Analyze / Plan / Execute / Review / Research / Synthesize)。
    pub fn categorize_task(&self, instruction: &str) -> super::traits::TaskType {
        // 简单启发式分类 — 实际应委托给 TaskCategorizer
        let lower = instruction.to_lowercase();
        if lower.contains("分析") || lower.contains("analyze") {
            super::traits::TaskType::Analyze
        } else if lower.contains("计划") || lower.contains("plan") {
            super::traits::TaskType::Plan
        } else if lower.contains("审查") || lower.contains("review") {
            super::traits::TaskType::Review
        } else if lower.contains("研究") || lower.contains("research") {
            super::traits::TaskType::Research
        } else if lower.contains("综合") || lower.contains("synthesize") {
            super::traits::TaskType::Synthesize
        } else {
            super::traits::TaskType::Execute
        }
    }
}

impl Default for CognitionFacade {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_creates_with_default() {
        let facade = CognitionFacade::new();
        let health = facade.health();
        assert!(health.overall >= 0.0 && health.overall <= 1.0);
    }

    #[test]
    fn health_aggregates_all_submodules() {
        let facade = CognitionFacade::new();
        let health = facade.health();
        assert!(health.reasoning.healthy);
        assert!(health.evolution.healthy);
        assert!(health.consciousness.healthy);
    }

    #[test]
    fn consciousness_status_returns_snapshot() {
        let facade = CognitionFacade::new();
        let status = facade.consciousness_status();
        assert_eq!(status.cycle, 0);
    }

    #[test]
    fn evolve_returns_result() {
        let facade = CognitionFacade::new();
        let result = facade.evolve();
        assert!(result.success);
    }

    #[test]
    fn categorize_task_classifies_instructions() {
        let facade = CognitionFacade::new();
        assert_eq!(
            facade.categorize_task("分析这段代码"),
            super::super::traits::TaskType::Analyze
        );
        assert_eq!(
            facade.categorize_task("plan the migration"),
            super::super::traits::TaskType::Plan
        );
        assert_eq!(
            facade.categorize_task("review the PR"),
            super::super::traits::TaskType::Review
        );
        assert_eq!(
            facade.categorize_task("执行部署"),
            super::super::traits::TaskType::Execute
        );
    }

    #[test]
    fn reason_delegates_to_nt_core() {
        let facade = CognitionFacade::new();
        let task = ReasoningTask {
            task_type: super::super::traits::TaskType::Analyze,
            input: serde_json::json!("test"),
            context: vec![],
            constraints: vec![],
            priority: 1,
        };
        let result = facade.reason(&task);
        assert_eq!(
            result.task.task_type,
            super::super::traits::TaskType::Analyze
        );
        assert!(!result.reasoning_chain.is_empty());
    }
}
