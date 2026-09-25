//! Agent 能力共享类型 (唯读类型/trait/配置) — 无 KB/脑依赖。

use crate::l5_cognition::l1_facade::attention_head::AttentionDomain;

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
    fn capability_retrieve(&self, query: &str, limit: usize) -> Result<CapabilityOutcome, String>;

    /// 记忆巩固报告 — 当前库规模 + 结构信号。
    fn capability_consolidate(&self) -> Result<CapabilityOutcome, String>;

    /// 证据链溯源 — 列出证据类节点。
    fn capability_evidence(&self) -> Result<CapabilityOutcome, String>;
}

/// 派单路由学习者配置 (P1: min_evidence 从硬编码进配置)。
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteLearnerConfig {
    /// 覆盖静态映射前所需的最少观察次数 (防冷启动噪声)
    pub min_evidence: u32,
}

impl Default for RouteLearnerConfig {
    fn default() -> Self {
        Self { min_evidence: 3 }
    }
}

/// 派单执行结果 — 派单不再只是身份标注, 而是真实动作的产出 (P0 接线)。
#[derive(Debug, Clone, PartialEq)]
pub enum AgentExecutionOutcome {
    /// 执行器成功产出 (如搜索返回结果、检索命中)。
    Success(String),
    /// 执行器运行但无产出 (如搜索空结果) — 不算失败, 但无增益。
    NoOp(String),
    /// 执行失败 (后端不可用/检索报错)。
    Failure(String),
}

impl AgentExecutionOutcome {
    /// 是否构成对 RouteLearner 的正向行为信号 (成功才强化该档案)。
    pub fn is_success(&self) -> bool {
        matches!(self, AgentExecutionOutcome::Success(_))
    }

    /// 人类可读摘要 (background_loop 日志用)。
    pub fn summary(&self) -> String {
        match self {
            AgentExecutionOutcome::Success(s) => format!("success: {}", s),
            AgentExecutionOutcome::NoOp(s) => format!("noop: {}", s),
            AgentExecutionOutcome::Failure(s) => format!("failed: {}", s),
        }
    }
}

/// MANTA 式拓扑修复提议 (P3) — trace 审计发现"当前组织不足"时的有界结构更新。
#[derive(Debug, Clone, PartialEq)]
pub struct TopologyRepair {
    /// 发生组织不足的注意力域
    pub domain: AttentionDomain,
    /// 当前 (旧) 档案
    pub from_agent: &'static str,
    /// 建议改为的档案
    pub to_agent: &'static str,
    /// 旧档案在该域的实测成功率
    pub from_success_rate: f64,
    /// 新档案在该域的实测成功率
    pub to_success_rate: f64,
    /// 依据的证据样本数
    pub evidence_attempts: u32,
}

impl TopologyRepair {
    /// 人类可读摘要 (background_loop 日志用)。
    pub fn summary(&self) -> String {
        format!(
            "{:?}: {} ({:.0}%) -> {} ({:.0}%), evidence={}",
            self.domain,
            self.from_agent,
            self.from_success_rate * 100.0,
            self.to_agent,
            self.to_success_rate * 100.0,
            self.evidence_attempts,
        )
    }
}

/// 派单执行桥 — 把 AgentCatalog 档案映射到真实执行器 (R-P42: 强化现有节点,
/// 不建平行适配器模块)。背景循环持有生产实现, 测试可注入探针实现。
pub trait AgentExecutor {
    /// 按档案名执行任务, 返回真实动作结果。
    fn execute(&self, agent: &str, task: &str) -> AgentExecutionOutcome;

    /// 策略感知执行 (P4, MAGE task-level search bandit 注入) — 按任务级搜索 bandit
    /// 选出的检索策略执行。默认实现 = 常规执行 (frozen backbone, 不改变既有行为);
    /// 生产执行桥可覆盖以把策略接入 confidence 检索缝。
    fn execute_with_strategy(
        &self,
        agent: &str,
        task: &str,
        _strategy: &str,
    ) -> AgentExecutionOutcome {
        self.execute(agent, task)
    }
}

/// 一条待吸收的对话经验 — 来自 KB 的 session/experience 节点。
#[derive(Debug, Clone)]
pub struct DialogueExperience {
    /// 节点标题 (如 "session-2026-...")
    pub title: String,
    /// 对话正文 / 蒸馏摘要
    pub content: String,
    /// 节点重要性 (0.0–1.0), 用于吸收门控
    pub importance: f64,
}

/// 对话吸收参数 — R-P11/R-P28 提取魔法常量为可配置 Default (D5 修复)。
///
/// 原缺陷: `max_entries: 8` / `min_importance: 0.1` / 关键词 boost 系数都是
/// 拍脑袋字面量, 无校准痕迹。集中到此处后, 调参有单一入口且可被测试校准。
#[derive(Debug, Clone, Copy)]
pub struct DialogueAbsorbConfig {
    /// 单次最多吸收的条目数 (默认 8)
    pub max_entries: usize,
    /// 重要性下界 (默认 0.1) — 低于该值的会话不参与能力吸收
    pub min_importance: f64,
    /// 关键词命中的最低维度提升 (默认 0.5), 每条命中再 +0.1, 封顶 0.95
    pub boost_base: f64,
    /// 关键词单次命中递增系数 (默认 0.1)
    pub boost_per_hit: f64,
    /// 维度提升封顶 (默认 0.95)
    pub boost_cap: f64,
}

impl Default for DialogueAbsorbConfig {
    fn default() -> Self {
        Self {
            max_entries: 8,
            min_importance: 0.1,
            boost_base: 0.5,
            boost_per_hit: 0.1,
            boost_cap: 0.95,
        }
    }
}

/// 对话吸收的**实测行为化结果** (D1/D2 反 Self-Confirmation)。
///
/// 不是虚荣计数: 核心信号是批评器是否接受 + 能力面打分的真实 delta。
/// 生产路径 (handle_goal) 据此判断"这次吸收是真进化还是自欺"。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialogueAbsorbOutcome {
    /// 实例级吸收条目数
    pub absorbed: usize,
    /// EDV 批评器是否接受 (未回滚) — 以前被 `let _` 丢弃的真信号
    pub critic_accepted: bool,
    /// 吸收前 PerformanceEvaluator 打分 (TaskType::General)
    pub score_before: f64,
    /// 吸收后打分
    pub score_after: f64,
    /// 实测能力差 (after - before); 批评器回滚时为 0
    pub score_delta: f64,
}

impl DialogueAbsorbOutcome {
    /// 无经验 / 无信号时的空结果。
    pub fn empty() -> Self {
        Self {
            absorbed: 0,
            critic_accepted: false,
            score_before: 0.0,
            score_after: 0.0,
            score_delta: 0.0,
        }
    }

    /// 是否产生了正向行为信号: 有吸收 + 批评器接受 + 能力面变好。
    pub fn is_positive(&self) -> bool {
        self.absorbed > 0 && self.critic_accepted && self.score_delta > 0.0
    }
}
