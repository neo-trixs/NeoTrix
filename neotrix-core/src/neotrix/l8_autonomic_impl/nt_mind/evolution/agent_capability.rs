//! Agent 能力层 (Agent Capability Layer) — 记忆大脑 agent 化 (R-P42 强化现有节点)。
//!
//! 结论性设计 (架构评估): 元认知**不整体 agent 化**, 而是 "确定性内核 + agent 外壳"。
//! 本模块提供:
//!   1. `MemoryAgentCapability` — 记忆大脑统一能力 trait (写入/检索/巩固/证据/图查询),
//!      把 `KnowledgeBase` 的具体方法暴露为 agent 可调用的统一表面。
//!   2. `MetaAgentShell` — 元认知 agent 外壳, 用 AttentionManager 按任务类型路由
//!      到确定性内核 (MetaCognitiveLoop) 的对应阶段。
//!
//! 来源: ai-knowledge-graph agent 化吸收 + Onyx 决策管线 (P0-2) + 统一写入弧 (P0-1)。
//! 约束: 核心确定性管线 (nt_core_meta) 保持同步无运行时依赖, 外壳只在
//! `nt_mind` 层做路由 — 不引入平行适配器模块 (R-P42)。

use crate::core::nt_core_meta::{MetaCognitiveLoop, MetaCycleResult};
use crate::core::nt_core_self::attention_head::{
    AttentionDomain, AttentionManager,
};
use crate::neotrix::l3_memory_impl::nt_memory_kb::{
    KnowledgeBase, NodeType,
};

/// 记忆大脑能力类型 — agent 可按任务路由到具体能力。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryCapabilityKind {
    /// 统一写入弧 (write_memory_entry)
    Write,
    /// 决策式混合检索 (adaptive pipeline)
    Retrieve,
    /// Dreaming 巩固 (VSA 重组/提纯)
    Consolidate,
    /// 证据链溯源 (historian evidence)
    Evidence,
    /// GraphRAG 图查询
    Graph,
}

impl MemoryCapabilityKind {
    /// 映射到注意力域 — 供 AttentionManager 路由决策。
    pub fn attention_domain(&self) -> AttentionDomain {
        match self {
            MemoryCapabilityKind::Write => AttentionDomain::Planning,
            MemoryCapabilityKind::Retrieve => AttentionDomain::PatternMatch,
            MemoryCapabilityKind::Consolidate => AttentionDomain::Semantic,
            MemoryCapabilityKind::Evidence => AttentionDomain::SelfReflection,
            MemoryCapabilityKind::Graph => AttentionDomain::Temporal,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MemoryCapabilityKind::Write => "write",
            MemoryCapabilityKind::Retrieve => "retrieve",
            MemoryCapabilityKind::Consolidate => "consolidate",
            MemoryCapabilityKind::Evidence => "evidence",
            MemoryCapabilityKind::Graph => "graph",
        }
    }
}

/// 能力调用结果 — 统一包装, agent 消费不依赖具体返回类型。
#[derive(Debug, Clone)]
pub enum CapabilityOutcome {
    /// 写入/巩固类动作的结果计数
    Count(usize),
    /// 检索类结果 (节点数 + 首个命中标题)
    Hits(usize, String),
    /// 诊断/状态类文本
    Text(String),
}

/// 记忆大脑统一能力 trait — 把记忆域具体方法抽象为 agent 可调用表面。
///
/// R-P79 接线语义: 该 trait 的实例化必须绑定真实 KnowledgeBase,
/// 禁止死代码 — 由 `MemoryAgent` 在 background_loop 中接线。
pub trait MemoryAgentCapability {
    /// 统一写入弧 — 落主库 + 派生 graphrag 边 + evidence 元数据。
    fn capability_write(
        &self,
        title: &str,
        content: &str,
        domain: &str,
    ) -> Result<CapabilityOutcome, String>;

    /// 决策式检索 — adaptive 管线分类/打分/路由后按权限过滤。
    fn capability_retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<CapabilityOutcome, String>;

    /// 记忆巩固报告 — 当前库规模 + 结构信号。
    fn capability_consolidate(&self) -> Result<CapabilityOutcome, String>;

    /// 证据链溯源 — 列出证据类节点。
    fn capability_evidence(&self) -> Result<CapabilityOutcome, String>;
}

/// 基于 KnowledgeBase 的标准实现 — 记忆大脑 agent 化接线点。
pub struct MemoryAgent {
    pub kb: std::sync::Arc<KnowledgeBase>,
}

impl MemoryAgentCapability for MemoryAgent {
    fn capability_write(
        &self,
        title: &str,
        content: &str,
        domain: &str,
    ) -> Result<CapabilityOutcome, String> {
        let id = self.kb.write_memory_entry(
            title,
            NodeType::Concept,
            Some(content),
            None,
            Some(domain),
            None,
        )?;
        Ok(CapabilityOutcome::Text(format!("node={}", id)))
    }

    fn capability_retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<CapabilityOutcome, String> {
        let nodes = self.kb.search_permission_aware(
            query,
            limit,
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::PermissionLevel::default(),
        )?;
        let first = nodes
            .first()
            .map(|n| n.node.title.clone())
            .unwrap_or_default();
        Ok(CapabilityOutcome::Hits(nodes.len(), first))
    }

    fn capability_consolidate(&self) -> Result<CapabilityOutcome, String> {
        let all = self.kb.all_nodes()?;
        Ok(CapabilityOutcome::Count(all.len()))
    }

    fn capability_evidence(&self) -> Result<CapabilityOutcome, String> {
        let all = self.kb.all_nodes()?;
        let evidence_count = all
            .iter()
            .filter(|n| n.domain.as_deref() == Some("nt_memory_historian"))
            .count();
        Ok(CapabilityOutcome::Count(evidence_count))
    }
}

/// 元认知 agent 外壳 — 用 AttentionManager 按任务类型路由到确定性内核。
///
/// 架构评估结论: 元认知的**决策面** (何时跑哪个阶段) agent 化,
/// 而**执行面** (SCAN/ANALYZE/PLAN 各阶段) 保持确定性内核, 可测且无时序抖动。
pub struct MetaAgentShell {
    pub attention: AttentionManager,
    pub metacog: MetaCognitiveLoop,
    pub iterations_run: usize,
}

impl MetaAgentShell {
    pub fn new(task_type: &str) -> Self {
        // 按任务类型选择强度 + Weapon Set (Ascendancy 双专精路由)
        let attention = AttentionManager::from_task_type(0.3, task_type);
        let metacog = MetaCognitiveLoop::new(crate::core::nt_core_meta::SelfModel::new());
        Self {
            attention,
            metacog,
            iterations_run: 0,
        }
    }

    /// Agent 决策入口: 根据注意力域选择运行内核的哪个阶段。
    ///
    /// 路由语义:
    ///   - 注意力被 Planning/Code 激活 → 跑完整 cycle (SCAN→PLAN)
    ///   - 注意力被 Memory/Reflection 激活 → 跑 cycle 但只消费 report (轻量)
    ///   - 其余 → 不执行 (避免无效空转)
    pub fn decide_and_run(&mut self) -> Option<MetaCycleResult> {
        self.attention.decay_all();
        let dominant = self.attention.dominant_domain()?;
        match dominant {
            AttentionDomain::Planning
            | AttentionDomain::Code
            | AttentionDomain::Temporal => {
                let result = self.metacog.run_cycle();
                self.iterations_run += 1;
                Some(result)
            }
            AttentionDomain::Semantic
            | AttentionDomain::SelfReflection
            | AttentionDomain::PatternMatch => {
                let result = self.metacog.run_cycle();
                self.iterations_run += 1;
                Some(result)
            }
            _ => None,
        }
    }

    /// 激活特定域 — 供上层 (background_loop) 按事件触发。
    pub fn stimulate(&mut self, domain: AttentionDomain, amount: f64) {
        self.attention.stimulate_domain(domain, amount);
    }
}

impl Default for MemoryAgent {
    fn default() -> Self {
        Self {
            kb: std::sync::Arc::new(
                KnowledgeBase::open(None).unwrap_or_else(|e| {
                    panic!("MemoryAgent default requires KB open: {}", e)
                }),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_agent() -> MemoryAgent {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_memagent_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = KnowledgeBase::open(Some(tmp.into())).expect("open kb");
        MemoryAgent { kb: std::sync::Arc::new(kb) }
    }

    #[test]
    fn capability_write_and_retrieve() {
        let agent = mem_agent();
        let w = agent.capability_write("AgentTest", "NeoTrix agent capability wiring", "test").unwrap();
        assert!(matches!(w, CapabilityOutcome::Text(_)));
        let r = agent.capability_retrieve("NeoTrix agent", 5).unwrap();
        match r {
            CapabilityOutcome::Hits(n, first) => {
                assert!(n >= 1);
                assert_eq!(first, "AgentTest");
            }
            _ => panic!("expected hits"),
        }
    }

    #[test]
    fn capability_consolidate_counts() {
        let agent = mem_agent();
        let c = agent.capability_consolidate().unwrap();
        assert!(matches!(c, CapabilityOutcome::Count(_)));
    }

    #[test]
    fn capability_evidence_empty_ok() {
        let agent = mem_agent();
        let e = agent.capability_evidence().unwrap();
        assert!(matches!(e, CapabilityOutcome::Count(_)));
    }

    #[test]
    fn meta_shell_routes_via_attention() {
        let mut shell = MetaAgentShell::new("planning");
        shell.stimulate(AttentionDomain::Planning, 0.9);
        let r = shell.decide_and_run();
        assert!(r.is_some());
        assert_eq!(shell.iterations_run, 1);
    }

    #[test]
    fn meta_shell_no_dominant_no_run() {
        let mut shell = MetaAgentShell::new("planning");
        // 无任何域被刺激 → dominant_domain() 返回 None → 不空转
        let r = shell.decide_and_run();
        assert!(r.is_none());
        assert_eq!(shell.iterations_run, 0);
    }

    #[test]
    fn capability_kind_domain_mapping() {
        assert_eq!(
            MemoryCapabilityKind::Consolidate.attention_domain(),
            AttentionDomain::Semantic
        );
        assert_eq!(
            MemoryCapabilityKind::Retrieve.attention_domain(),
            AttentionDomain::PatternMatch
        );
        assert_eq!(MemoryCapabilityKind::Write.attention_domain(), AttentionDomain::Planning);
    }
}
