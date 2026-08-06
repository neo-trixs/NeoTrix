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
use crate::neotrix::l2_world_impl::nt_world_search::{SearchResult, UnifiedSearch};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::core::nt_core_consciousness_tree::{BranchKind, CapabilityBranch, ConsciousnessTree};

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

/// 派单路由学习者 — 用**观察到的行为化结果**校正静态映射 (D3 修复)。
///
/// 缺陷背景: `route_to_catalog` 是硬编码域→档案映射, 无反馈回路 — 批评器
/// 的结果从不反哺路由。`RouteLearner` 对每个注意力域累积 (档案, 成败) 对:
///   1. `record(domain, agent, success)` 由上层在每次 cycle 后喂入结果
///      (EDV 标准: 吸收被批评器接受/拒绝, 规划产出是否有用)。
///   2. 当某域累计证据 ≥ `config.min_evidence` 时, `route()` 覆盖静态映射,
///      选择该域历史成功率最高的档案 — 让派单从结果里学, 而非永远拍脑袋。
///   3. 证据不足时退回静态映射 (冷启动安全)。
///   4. P1: 统计可经 `persist`/`load` 存 KB kv_store, 跨会话存活 — 派单学习
///      不再是一次性运行内生效, 重启后继续累积证据。
#[derive(Debug, Clone)]
pub struct RouteLearner {
    /// domain → (agent → (success, attempts))
    outcomes: std::collections::HashMap<AttentionDomain, std::collections::HashMap<&'static str, (u32, u32)>>,
    /// 学习策略配置 (min_evidence 可调, 不再硬编码 3)
    pub config: RouteLearnerConfig,
}

impl RouteLearner {
    pub fn new() -> Self {
        Self {
            outcomes: std::collections::HashMap::new(),
            config: RouteLearnerConfig::default(),
        }
    }

    /// 以自定义配置构造 — 上层可把 min_evidence 接入 config 系统 (P1)。
    pub fn with_config(config: RouteLearnerConfig) -> Self {
        Self {
            outcomes: std::collections::HashMap::new(),
            config,
        }
    }

    /// 记录一次路由结果。`success=true` → 派给该档案产生预期行为 (有产出/批评通过)。
    pub fn record(&mut self, domain: AttentionDomain, agent: &'static str, success: bool) {
        let entry = self.outcomes.entry(domain).or_default();
        let cur = entry.entry(agent).or_insert((0, 0));
        cur.0 += success as u32;
        cur.1 += 1;
    }

    /// 是否有足够证据覆盖静态映射 (该域某档案试过 ≥ min_evidence 次)。
    pub fn has_enough_evidence(&self, domain: AttentionDomain) -> bool {
        self.outcomes.get(&domain)
            .map(|m| m.values().any(|(_, attempts)| *attempts >= self.config.min_evidence))
            .unwrap_or(false)
    }

    /// 学习后的路由: 若有足够证据, 返回静态档案里在域上成功率最高的档案;
    /// 否则沿用静态映射。
    pub fn route(&self, domain: AttentionDomain, static_agent: &'static str) -> &'static str {
        if !self.has_enough_evidence(domain) {
            return static_agent;
        }
        let map = self.outcomes.get(&domain).expect("evidence implies map");
        let rate = |s: &u32, t: &u32| *s as f64 / (*t).max(1) as f64;
        map.iter()
            .filter(|(_, (_, attempts))| *attempts >= self.config.min_evidence)
            .max_by(|(_, (sa, ta)), (_, (sb, tb))| {
                rate(sa, ta).partial_cmp(&rate(sb, tb)).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(agent, _)| *agent)
            .unwrap_or(static_agent)
    }

    /// 该域当前各档案成功率一览 (诊断/审计用)。
    pub fn rates(&self, domain: AttentionDomain) -> Vec<(&'static str, f64, u32)> {
        self.outcomes.get(&domain)
            .map(|m| m.iter().map(|(a, (s, t))| (*a, *s as f64 / (*t).max(1) as f64, *t)).collect())
            .unwrap_or_default()
    }

    /// 持久化行为统计到 KB kv_store (P1) — 派单学习跨会话存活。
    pub fn persist(&self, kb: &KnowledgeBase) -> Result<(), String> {
        let owned: std::collections::HashMap<
            String,
            std::collections::HashMap<String, (u32, u32)>,
        > = self
            .outcomes
            .iter()
            .map(|(domain, agents)| {
                (
                    format!("{:?}", domain),
                    agents
                        .iter()
                        .map(|(agent, stats)| (agent.to_string(), *stats))
                        .collect(),
                )
            })
            .collect();
        let payload = serde_json::json!({
            "outcomes": owned,
            "config": self.config,
        });
        let json = serde_json::to_string(&payload)
            .map_err(|e| format!("route_learner serialize: {}", e))?;
        kb.save_route_learner(&json)
    }

    /// 从 KB kv_store 恢复行为统计 (P1) — 冷启动时无存档则保持空状态。
    pub fn load(&mut self, kb: &KnowledgeBase) -> Result<(), String> {
        let Some(json) = kb.load_route_learner()? else {
            return Ok(());
        };
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| format!("route_learner deserialize: {}", e))?;
        self.config = parsed
            .get("config")
            .and_then(|c| serde_json::from_value(c.clone()).ok())
            .unwrap_or_default();
        let Some(outcomes) = parsed.get("outcomes") else {
            return Ok(());
        };
        let mut restored: std::collections::HashMap<
            AttentionDomain,
            std::collections::HashMap<&'static str, (u32, u32)>,
        > = std::collections::HashMap::new();
        for (domain_key, agents) in outcomes.as_object().unwrap_or(&serde_json::Map::new()) {
            let Some(domain) = Self::parse_domain(domain_key) else {
                continue;
            };
            let entry = restored.entry(domain).or_default();
            for (agent, stats) in agents.as_object().unwrap_or(&serde_json::Map::new()) {
                let (s, t) = match stats {
                    serde_json::Value::Array(arr) if arr.len() >= 2 => (
                        arr[0].as_u64().unwrap_or(0) as u32,
                        arr[1].as_u64().unwrap_or(0) as u32,
                    ),
                    _ => continue,
                };
                if let Some(static_agent) = Self::canonical_agent(agent) {
                    entry.insert(static_agent, (s, t));
                }
            }
        }
        self.outcomes = restored;
        Ok(())
    }

    /// 从字符串还原注意力域 (持久化用 `{:?}` 序列化)。
    fn parse_domain(s: &str) -> Option<AttentionDomain> {
        match s {
            "PatternMatch" => Some(AttentionDomain::PatternMatch),
            "Code" => Some(AttentionDomain::Code),
            "Semantic" => Some(AttentionDomain::Semantic),
            "Temporal" => Some(AttentionDomain::Temporal),
            "Planning" => Some(AttentionDomain::Planning),
            "SelfReflection" => Some(AttentionDomain::SelfReflection),
            "ToolUse" => Some(AttentionDomain::ToolUse),
            "GoalAlignment" => Some(AttentionDomain::GoalAlignment),
            "RiskAssessment" => Some(AttentionDomain::RiskAssessment),
            "Creativity" => Some(AttentionDomain::Creativity),
            _ => None,
        }
    }

    /// 把持久化的档案名还原为 `&'static str` 规范名 (未知档案丢弃, 防注入)。
    fn canonical_agent(s: &str) -> Option<&'static str> {
        match s {
            "researcher" => Some("researcher"),
            "explorer" => Some("explorer"),
            "planner" => Some("planner"),
            "generalist" => Some("generalist"),
            "verifier" => Some("verifier"),
            "watcher" => Some("watcher"),
            _ => None,
        }
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

/// 派单执行桥 — 把 AgentCatalog 档案映射到真实执行器 (R-P42: 强化现有节点,
/// 不建平行适配器模块)。背景循环持有生产实现, 测试可注入探针实现。
pub trait AgentExecutor {
    /// 按档案名执行任务, 返回真实动作结果。
    fn execute(&self, agent: &str, task: &str) -> AgentExecutionOutcome;
}

/// 生产执行桥 — 把内置 6 档案接到已接线子系统, 让派单真正驱动动作。
///
/// P0 接线 (断点1/2/8 修复): 之前 `MetaAgentShell::decide_and_run` 派单后
/// 只 eprintln 档案名, 无人消费。此桥让:
///   - researcher → UnifiedSearch (DDG→Wikipedia 有序后端)
///   - explorer   → 权限感知 KB 检索
///   - verifier   → KB 证据溯源计数
///   - watcher    → KB 规模监控
///   - planner    → 无副作用规划占位 (规划由 metacog cycle 产出)
///   - generalist → 综合: 检索 + 证据, 反馈脑能力 (兜底通用执行)
pub struct ProductionAgentExecutor {
    /// 记忆大脑外壳 — KB 写/检索/证据/巩固统一能力面。
    pub memory: MemoryAgent,
    /// 统一搜索 — 有序后端路由 (DDG→Wikipedia)。
    pub search: UnifiedSearch,
}

impl ProductionAgentExecutor {
    pub fn new(kb: std::sync::Arc<KnowledgeBase>) -> Self {
        Self {
            memory: MemoryAgent { kb },
            search: UnifiedSearch::new(),
        }
    }
}

impl AgentExecutor for ProductionAgentExecutor {
    fn execute(&self, agent: &str, task: &str) -> AgentExecutionOutcome {
        match agent {
            "researcher" => match self.search.search(task, 5) {
                Ok(results) if !results.is_empty() => {
                    AgentExecutionOutcome::Success(format!(
                        "searched {} results (backend={})",
                        results.len(),
                        self.search.active_backend(),
                    ))
                }
                Ok(_) => AgentExecutionOutcome::NoOp("search returned no results".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("search: {}", e)),
            },
            "explorer" => match self.memory.capability_retrieve(task, 5) {
                Ok(CapabilityOutcome::Hits(n, first)) if n > 0 => {
                    AgentExecutionOutcome::Success(format!("retrieved {} hits (first: {})", n, first))
                }
                Ok(_) => AgentExecutionOutcome::NoOp("no KB hits".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("retrieve: {}", e)),
            },
            "verifier" => match self.memory.capability_evidence() {
                Ok(CapabilityOutcome::Count(n)) => AgentExecutionOutcome::Success(format!(
                    "evidence audit: {} sources traced",
                    n
                )),
                Ok(_) => AgentExecutionOutcome::NoOp("evidence count unavailable".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("evidence: {}", e)),
            },
            "watcher" => match self.memory.capability_consolidate() {
                Ok(CapabilityOutcome::Count(n)) => AgentExecutionOutcome::Success(format!(
                    "health probe: {} KB nodes",
                    n
                )),
                Ok(_) => AgentExecutionOutcome::NoOp("consolidate signal unavailable".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("consolidate: {}", e)),
            },
            // planner: 规划由 metacog cycle 产出, 执行桥不重复做副作用动作。
            "planner" => AgentExecutionOutcome::NoOp("planning handled by metacog cycle".into()),
            // generalist: 综合执行 — 检索 + 证据溯源, 反馈脑能力面。
            "generalist" => {
                let r = self.memory.capability_retrieve(task, 3);
                let e = self.memory.capability_evidence();
                match (r, e) {
                    (Ok(CapabilityOutcome::Hits(n, _)), Ok(CapabilityOutcome::Count(m))) => {
                        AgentExecutionOutcome::Success(format!("combined {} hits, {} evidence", n, m))
                    }
                    (Err(err), _) => AgentExecutionOutcome::Failure(format!("retrieve: {}", err)),
                    _ => AgentExecutionOutcome::NoOp("no combined signal".into()),
                }
            }
            other => AgentExecutionOutcome::NoOp(format!("no executor for agent '{}'", other)),
        }
    }
}

/// 元认知 agent 外壳 — 用 AttentionManager 按任务类型路由到确定性内核。
///
/// 架构评估结论: 元认知的**决策面** (何时跑哪个阶段) agent 化,
/// 而**执行面** (SCAN/ANALYZE/PLAN 各阶段) 保持确定性内核, 可测且无时序抖动。
///
/// 目录派单桥: 决策面与 `AgentCatalog` 对齐 — 注意力主导域会先映射到内置
/// agent 档案 (explorer/planner/researcher/generalist/verifier/watcher),
/// 由档案决定本轮 cycle 的"身份"与工具权限语义, 并经 `AgentExecutor`
/// 把档案接到真实子系统 — 派单是控制面而非仪式。
pub struct MetaAgentShell {
    pub attention: AttentionManager,
    pub metacog: MetaCognitiveLoop,
    pub iterations_run: usize,
    /// 最近一次被派单的内置 agent 档案名 (来自 AgentCatalog)。
    pub last_dispatched: Option<&'static str>,
    /// 行为化路由学习者 — 用结果反馈覆盖静态映射 (D3)。
    pub learner: RouteLearner,
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
            last_dispatched: None,
            learner: RouteLearner::new(),
        }
    }

    /// 以自定义路由学习配置构造 (P1: min_evidence 等经 config 注入)。
    pub fn with_learner_config(task_type: &str, learner_config: RouteLearnerConfig) -> Self {
        let attention = AttentionManager::from_task_type(0.3, task_type);
        let metacog = MetaCognitiveLoop::new(crate::core::nt_core_meta::SelfModel::new());
        Self {
            attention,
            metacog,
            iterations_run: 0,
            last_dispatched: None,
            learner: RouteLearner::with_config(learner_config),
        }
    }

    /// 注意力主导域 → 内置 agent 档案路由 (目录即派单星图)。
    ///
    /// 映射语义 (对应 AgentCatalog 的 6 类):
    ///   - PatternMatch (检索)      → explorer   (只读探索)
    ///   - Planning (目标规划)      → planner    (先研究出方案)
    ///   - GoalAlignment (目标对齐) → planner    (方案校准)
    ///   - Code (执行)              → generalist (旗舰通用, 全权)
    ///   - ToolUse (工具调用)       → generalist (全权执行)
    ///   - SelfReflection (审查)    → verifier   (以判据回滚, 防自确认陷阱)
    ///   - RiskAssessment (风险评估) → verifier  (以判据把关)
    ///   - Semantic (巩固/记忆)     → watcher    (常驻监测/巩固)
    ///   - Temporal (时间/历史)     → generalist (兜底执行)
    ///   - Creativity (创造)        → planner    (方案生成)
    ///   - 其余                     → 不派单 (None)
    pub fn route_to_catalog(&self) -> Option<&'static str> {
        let dominant = self.attention.dominant_domain()?;
        let static_agent = match dominant {
            AttentionDomain::PatternMatch => "explorer",
            AttentionDomain::Planning | AttentionDomain::GoalAlignment => "planner",
            AttentionDomain::Code | AttentionDomain::ToolUse | AttentionDomain::Temporal => "generalist",
            AttentionDomain::SelfReflection | AttentionDomain::RiskAssessment => "verifier",
            AttentionDomain::Semantic => "watcher",
            AttentionDomain::Creativity => "planner",
        };
        // D3: 学习化路由 — 静态映射为冷启动基线, 有足够证据时用结果反馈覆盖。
        Some(self.learner.route(dominant, static_agent))
    }

    /// 任务提示感知的派单 (P0 缺陷 #4 修复) — 融合两套路由体系。
    ///
    /// 当主导域为 PatternMatch (检索) 时, 用 `AgentCatalog::route(task_hint)`
    /// 在 explorer (KB 检索) 与 researcher (网络研究) 之间细分: 目标是
    /// "research/研究/synthesize" → researcher, 否则 explorer。其余域沿用
    /// 注意力静态映射 + RouteLearner 校正。消除"关键词路由零生产调用"死洞。
    pub fn route_with_hint(&self, task_hint: &str) -> Option<&'static str> {
        let dominant = self.attention.dominant_domain()?;
        if dominant == AttentionDomain::PatternMatch && !task_hint.trim().is_empty() {
            let profile = crate::core::l7_capability::nt_core_orch_agent::AgentCatalog::route(task_hint);
            // 仅接受 PatternMatch 语义内的细分 (researcher/explorer); 关键词
            // 路由若越界到其他域档案则退回注意力静态映射, 避免语义漂移。
            if profile.name == "researcher" || profile.name == "explorer" {
                return Some(self.learner.route(dominant, profile.name));
            }
        }
        self.route_to_catalog()
    }

    /// Agent 决策入口: 根据注意力域选择运行内核的哪个阶段。
    ///
    /// 路由语义:
    ///   - 注意力被 Planning/Code 激活 → 跑完整 cycle (SCAN→PLAN)
    ///   - 注意力被 Memory/Reflection 激活 → 跑 cycle 但只消费 report (轻量)
    ///
    /// 派单语义: 运行前先把主导域映射为 AgentCatalog 档案 (learner 校正后)
    /// 并记录到 `last_dispatched`。cycle 产出后按**行为化结果**喂回 learner:
    /// 有 plan/alert 产出 → 派单成功 (record success), 否则记失败 — 让路由
    /// 真正从结果里学, 而不只是硬编码映射 (EDV: R-P30 behavior→weight)。
    pub fn decide_and_run(&mut self) -> Option<MetaCycleResult> {
        self.attention.decay_all();
        let dominant = self.attention.dominant_domain()?;
        let dispatched = self.route_to_catalog();
        self.last_dispatched = dispatched;
        let result = self.metacog.run_cycle();
        self.iterations_run += 1;
        // 行为反馈: 规划/告警有产出 = 该档案对该域成功。
        let produced = !result.plans.is_empty() || !result.alerts.is_empty();
        if let Some(agent) = dispatched {
            self.learner.record(dominant, agent, produced);
        }
        Some(result)
    }

    /// 派单并执行 (P0 断点修复) — 派单结果经 `AgentExecutor` 驱动真实动作,
    /// 并把**实测执行结果**喂回 RouteLearner (取代"有 plan 产出"的启发式)。
    ///
    /// 返回 (档案名, 执行结果) 供上层日志/决策; 无主导域时返回 None (不空转)。
    /// 与 `decide_and_run` 的区别: 后者只记录"是否派单成功", 前者真正激活执行器
    /// 并以动作成败为行为信号 — 让星系派单从仪式变控制面。
    pub fn dispatch_and_execute(
        &mut self,
        executor: &dyn AgentExecutor,
        task: &str,
    ) -> Option<(&'static str, AgentExecutionOutcome)> {
        self.attention.decay_all();
        let dominant = self.attention.dominant_domain()?;
        let agent = self.route_with_hint(task)?;
        self.last_dispatched = Some(agent);
        let outcome = executor.execute(agent, task);
        // 真实行为信号: 执行成功才强化该档案对该域的派单。
        self.learner.record(dominant, agent, outcome.is_success());
        Some((agent, outcome))
    }

    /// 激活特定域 — 供上层 (background_loop) 按事件触发。
    pub fn stimulate(&mut self, domain: AttentionDomain, amount: f64) {
        self.attention.stimulate_domain(domain, amount);
    }
}

/// 依据目标语义推导应刺激的注意力域集 (P0: 多域刺激, 修复缺陷 #3)。
///
/// 背景循环此前永远只刺激 SelfReflection, 10 个注意力域 9 个形同虚设。
/// 此函数把目标文本映射到对应域:
///   - 研究/搜索/分析       → PatternMatch (检索) + SelfReflection (审查)
///   - 编码/修复/实现       → Code + ToolUse (执行)
///   - 架构/设计/方案       → Planning + Creativity (规划/创造)
///   - 监控/心跳/巩固       → Semantic (记忆/巩固)
///   - 审查/校验/回滚       → RiskAssessment + SelfReflection (把关)
///   - 默认                 → SelfReflection (轻量自省, 兜底)
/// 返回 (域, 刺激强度) 列表; 空文本时只给弱自省, 防空转。
pub fn domains_for_goal(goal: &str) -> Vec<(AttentionDomain, f64)> {
    let lower = goal.to_lowercase();
    let has = |kws: &[&str]| kws.iter().any(|k| lower.contains(k));
    if goal.trim().is_empty() {
        return vec![(AttentionDomain::SelfReflection, 0.2)];
    }
    let mut domains = Vec::new();
    if has(&["research", "search", "研究", "搜索", "分析", "find", "aggregate"]) {
        domains.push((AttentionDomain::PatternMatch, 0.8));
        domains.push((AttentionDomain::SelfReflection, 0.4));
    }
    if has(&["code", "implement", "fix", "refactor", "编码", "实现", "修复", "重构"]) {
        domains.push((AttentionDomain::Code, 0.8));
        domains.push((AttentionDomain::ToolUse, 0.6));
    }
    if has(&["design", "architecture", "plan", "方案", "架构", "设计", "规划"]) {
        domains.push((AttentionDomain::Planning, 0.8));
        domains.push((AttentionDomain::Creativity, 0.4));
    }
    if has(&["monitor", "watch", "health", "监控", "心跳", "巩固"]) {
        domains.push((AttentionDomain::Semantic, 0.7));
    }
    if has(&["review", "verify", "audit", "rollback", "审查", "校验", "回滚"]) {
        domains.push((AttentionDomain::RiskAssessment, 0.7));
        domains.push((AttentionDomain::SelfReflection, 0.5));
    }
    if domains.is_empty() {
        // 无关键词匹配 → 轻量自省兜底, 避免 10 域全空。
        domains.push((AttentionDomain::SelfReflection, 0.3));
    }
    domains
}

/// 星系能力网络 → 派单刺激 (P2: 树从观测变控制面)。
///
/// 缺陷背景: `ConsciousnessTree` 的 branch health/fog/constellation 是纯观测
/// 信号 — 只写日志、enqueue goal, 从不驱动派单。此函数把树的薄弱分支映射为
/// 注意力域刺激: 分支越弱 (health 低 / fog 浓 / constellation 低) → 对应域
/// 刺激越强 → 派单该域档案去强化它。让树真正成为星系派单的控制面。
///
/// 返回 (域, 刺激强度) 列表; 健康分支不产生刺激 (强度 0), 防空转。
pub fn tree_branch_stimuli(tree: &ConsciousnessTree) -> Vec<(AttentionDomain, f64)> {
    // 分支薄弱度阈值: 健康分支 (fog≈0.05, health 满, constellation 高) 薄弱度
    // 仅 ~0.02, 会被过滤; 真薄弱分支 (fog 0.85+/health 低/C0) 达 ~0.9, 驱动派单。
    const WEAK_THRESHOLD: f64 = 0.2;
    let mut stimuli = Vec::new();
    for (kind, branch) in &tree.branches {
        let weakness = branch_weakness(branch);
        if weakness < WEAK_THRESHOLD {
            continue;
        }
        for domain in branch_attention_domains(kind) {
            stimuli.push((domain, weakness));
        }
    }
    stimuli
}

/// 分支薄弱度 [0,1] — health 越低 / fog 越浓 / constellation 越低, 越薄弱。
/// 纯函数: (1-health)*0.4 + fog*0.4 + (1-constellation.score())*0.2。
pub fn branch_weakness(branch: &CapabilityBranch) -> f64 {
    let health_weak = (1.0 - branch.health.clamp(0.0, 1.0)) * 0.4;
    let fog_weak = branch.fog.level.clamp(0.0, 1.0) * 0.4;
    let constel_weak = (1.0 - branch.constellation.score().clamp(0.0, 1.0)) * 0.2;
    (health_weak + fog_weak + constel_weak).clamp(0.0, 1.0)
}

/// 分支 → 注意力域映射 (P2 控制面) — 每个星系分支薄弱时应刺激哪些域。
pub fn branch_attention_domains(kind: &BranchKind) -> Vec<AttentionDomain> {
    match kind {
        BranchKind::Core => vec![AttentionDomain::SelfReflection, AttentionDomain::RiskAssessment],
        BranchKind::Mind => vec![AttentionDomain::Creativity, AttentionDomain::Planning],
        BranchKind::Memory => vec![AttentionDomain::Semantic],
        BranchKind::World => vec![AttentionDomain::PatternMatch],
        BranchKind::Act => vec![AttentionDomain::GoalAlignment, AttentionDomain::ToolUse],
        BranchKind::Io => vec![AttentionDomain::Code, AttentionDomain::ToolUse],
        BranchKind::Shield => vec![AttentionDomain::RiskAssessment, AttentionDomain::SelfReflection],
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

/// 对话吸收桥 (DialogueAbsorbBridge) — 让对话经验参与 SelfIteratingBrain 进化 (R-P42)。
///
/// 吸收模式: 从 KB 读取近期 session/experience 节点 → 由正文关键词派生出
/// 内容感知的 CapabilityVector (custom source) → `absorb_from_custom` 落地,
/// 同时以 `KnowledgeSource::DialogueExperience` 身份跑一次受校验的
/// `safe_absorb` (DefaultAbsorbValidator)。两条路径都真实反哺脑能力,
/// 对话经历不再是 experience-tree 一次性写入的死数据。
pub struct DialogueAbsorbBridge {
    pub kb: std::sync::Arc<KnowledgeBase>,
    /// 单次最多吸收的条目数
    pub max_entries: usize,
    /// 重要性下界 — 低于该值的会话不参与能力吸收
    pub min_importance: f64,
    /// 内容感知向量推导参数 (D5 校准入口)
    pub config: DialogueAbsorbConfig,
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

impl DialogueAbsorbBridge {
    pub fn new(kb: std::sync::Arc<KnowledgeBase>) -> Self {
        let config = DialogueAbsorbConfig::default();
        Self {
            kb,
            max_entries: config.max_entries,
            min_importance: config.min_importance,
            config,
        }
    }

    /// 以自定义参数构造 — 供调参与测试校准 (D5)。
    pub fn with_config(kb: std::sync::Arc<KnowledgeBase>, config: DialogueAbsorbConfig) -> Self {
        Self {
            kb,
            max_entries: config.max_entries,
            min_importance: config.min_importance,
            config,
        }
    }

    /// 从 KB 提取近期对话经验 — NodeType::Session 或标题以 "session-" 开头。
    pub fn recent_experiences(&self) -> Vec<DialogueExperience> {
        let Ok(nodes) = self.kb.all_nodes() else {
            return Vec::new();
        };
        let mut experiences: Vec<DialogueExperience> = nodes
            .into_iter()
            .filter(|n| {
                n.node_type == NodeType::Session || n.title.starts_with("session-")
            })
            .filter(|n| n.importance >= self.min_importance)
            .map(|n| {
                let content = n
                    .content
                    .clone()
                    .or_else(|| n.summary.clone())
                    .unwrap_or_else(|| n.title.clone());
                DialogueExperience {
                    title: n.title,
                    content,
                    importance: n.importance,
                }
            })
            .collect();
        experiences.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        experiences.truncate(self.max_entries);
        experiences
    }

    /// 由对话正文派生出内容感知的能力向量 — 关键词命中 23 维字段则提升对应维度。
    ///
    /// 吸收模式: 这是 content-aware 的源头向量, 经 `register_knowledge_source`
    /// 登记后由 `absorb_from_custom` 以对话特有强度反哺能力面。
    pub fn derive_vector(&self, content: &str) -> crate::core::CapabilityVector {
        let text = content.to_lowercase();
        let mut cv = crate::core::CapabilityVector::default();
        let keyword_dims: &[(&[&str], &str)] = &[
            (&["test", "verify", "assert", "check", "unit"], "verification"),
            (&["memory", "kb", "storage", "recall", "retriev"], "semantic_layer"),
            (&["plan", "goal", "strategy", "schedule"], "compound_composition"),
            (&["analy", "trace", "debug", "root cause"], "analysis"),
            (&["synthes", "summar", "distill", "abstract"], "synthesis"),
            (&["seal", "iterate", "self-improve", "evolve", "absorb"], "experimental"),
            (&["attention", "focus", "route", "domain"], "ai_native_states"),
            (&["creative", "novel", "design", "style"], "creativity"),
            (&["document", "comment", "explain", "doc"], "accessibility"),
            (&["conversation", "dialogue", "user", "prompt"], "inference_depth"),
        ];
        for (kws, dim) in keyword_dims {
            let hits = kws.iter().filter(|k| text.contains(**k)).count();
            if hits > 0 {
                let boost = (self.config.boost_base + self.config.boost_per_hit * hits as f64).min(self.config.boost_cap);
                let _ = cv.set_field_by_name(dim, boost);
            }
        }
        cv
    }

    /// 把近期对话经验吸收进 SelfIteratingBrain — 返回**实测的行为化结果**。
    ///
    /// 吸收模式 (grounded 于 ACL 2026/arXiv 2026 经验学习文献):
    ///   1. **实例级 (instance)**: 每条经验派生内容感知向量, `absorb_from_custom` 落地,
    ///      记录标题避免同一会话反复吸收 (ReMe utility-based refinement 去重)。
    ///   2. **批次级 (batch)**: 全部经验向量取分量 max 共振 (Metacognitive Consolidation
    ///      实例→批次层次), 以 DialogueExperience 源身份吸收 — 原则级信号比实例级持久。
    ///   3. **Verify 校验 (EDV 反 Self-Confirmation Trap)**: `absorb_with_critic` 做
    ///      吸收前后 PerformanceEvaluator 对比, 能力下降则回滚 — 不用恒真的弱 validator。
    ///
    /// 反 Self-Confirmation (D1/D2): 返回值不是"吸收了几条"的虚荣计数, 而是
    /// **实测能力差** — 吸收前 vs 吸收后按 `PerformanceEvaluator` 打分的 delta。
    /// 只有当批评器接受 (未回滚) 且能力面确实变好时才计为有效。
    pub fn absorb_pending(&self, brain: &mut SelfIteratingBrain) -> DialogueAbsorbOutcome {
        let experiences = self.recent_experiences();
        if experiences.is_empty() {
            return DialogueAbsorbOutcome::empty();
        }
        // ── 批次级共振向量 (分量取 max, 反映跨会话主题强度) ──
        let mut batch = crate::core::CapabilityVector::default();
        let mut seen = std::collections::HashSet::new();
        for exp in &experiences {
            if !seen.insert(exp.title.clone()) {
                continue; // ReMe: 已吸收过的会话跳过, 避免重复污染
            }
            let v = self.derive_vector(&exp.content);
            for (i, val) in v.arr().iter().enumerate() {
                if *val > batch.arr()[i] {
                    batch.arr_mut()[i] = *val;
                }
            }
        }
        let has_batch_signal = batch.arr().iter().any(|&v| v > 0.0);
        if !has_batch_signal {
            return DialogueAbsorbOutcome::empty();
        }

        // 实测能力差: 吸收前按 PerformanceEvaluator 打分 (D1/D2 行为化指标)。
        let before_score = crate::neotrix::nt_mind::seal_core::core::PerformanceEvaluator::evaluate(
            &crate::neotrix::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        // ── 实例级: 每条经验内容感知吸收 ──
        let mut absorbed = 0usize;
        for (i, exp) in experiences.iter().enumerate() {
            let custom_name = format!("dialogue:{}:{}", i, exp.title.chars().take(32).collect::<String>());
            let vector = self.derive_vector(&exp.content);
            let has_signal = vector.arr().iter().any(|&v| v > 0.0);
            brain.brain.register_knowledge_source(&custom_name, vector.clone());
            if has_signal && brain.brain.absorb_from_custom(&custom_name) {
                absorbed += 1;
            }
        }

        // ── 批次级: 原则级共振向量经 DialogueExperience 源吸收 ──
        brain.brain.register_knowledge_source("dialogue:batch", batch.clone());
        let _ = brain.brain.absorb_from_custom("dialogue:batch");

        // ── Verify (EDV): 吸收前后性能对比, 能力下降则回滚 ──
        // 批评器返回是否接受 (未回滚); 不再丢弃 — 它是行为化成败的真信号。
        let critic_accepted = brain.absorb_with_critic(crate::core::KnowledgeSource::DialogueExperience);

        // 实测后分: 批评器若回滚, after == before, 无增益。
        let after_score = crate::neotrix::nt_mind::seal_core::core::PerformanceEvaluator::evaluate(
            &crate::neotrix::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        DialogueAbsorbOutcome {
            absorbed,
            critic_accepted,
            score_before: before_score,
            score_after: after_score,
            score_delta: after_score - before_score,
        }
    }

    /// 由研究结论正文派生出内容感知向量 — 研究域关键词 23 维提升。
    ///
    /// 与 `derive_vector` (对话域) 平行的研究域映射: 关键词侧重证据/来源/
    /// 方法/聚合/分析, 反映"外部世界知识被吸收"的信号面。复用同一
    /// `DialogueAbsorbConfig` boost 系数 (D5 单一调参入口)。
    pub fn derive_research_vector(&self, content: &str) -> crate::core::CapabilityVector {
        let text = content.to_lowercase();
        let mut cv = crate::core::CapabilityVector::default();
        let research_dims: &[(&[&str], &str)] = &[
            (&["search", "web", "online", "url", "http"], "semantic_layer"),
            (&["paper", "arxiv", "research", "study", "report"], "inference_depth"),
            (&["method", "approach", "technique", "algorithm"], "analysis"),
            (&["synthes", "aggregate", "summary", "distill", "conclusion"], "synthesis"),
            (&["evidence", "source", "cite", "reference", "verify"], "verification"),
            (&["benchmark", "metric", "evaluate", "compare", "result"], "quality_gates"),
            (&["finding", "insight", "discover", "trend", "pattern"], "ai_native_states"),
            (&["domain", "field", "industry", "topic", "expert"], "domain_specificity"),
            (&["collect", "gather", "mine", "scrape", "harvest"], "compound_composition"),
            (&["open", "share", "collaborate", "community", "doc"], "accessibility"),
        ];
        for (kws, dim) in research_dims {
            let hits = kws.iter().filter(|k| text.contains(**k)).count();
            if hits > 0 {
                let boost = (self.config.boost_base + self.config.boost_per_hit * hits as f64).min(self.config.boost_cap);
                let _ = cv.set_field_by_name(dim, boost);
            }
        }
        cv
    }

    /// 研究结论 → KB → 脑能力进化闭环 (R-P79 生产接线)。
    ///
    /// researcher agent / WebSearchTool 产出的搜索结论不再是一次性丢弃的
    /// 死数据: 落 KB (可溯源) + 蒸馏为内容感知能力向量反哺 SelfIteratingBrain。
    /// 同一 EDV 校验 (critic 接受 + 实测能力差) 复用 `DialogueAbsorbOutcome`。
    ///
    /// `absorbed` 语义: 写入 KB 的结论条数 (>= 1 表示有真实世界知识落地);
    /// 能力面增益以 `score_delta` 为准 (批评器回滚时为 0)。
    pub fn absorb_research_findings(
        &self,
        brain: &mut SelfIteratingBrain,
        query: &str,
        results: &[SearchResult],
    ) -> DialogueAbsorbOutcome {
        if results.is_empty() {
            return DialogueAbsorbOutcome::empty();
        }

        // ── 落 KB (可溯源): 每条结果写 Source 节点, 查询主题写 Insight 节点 ──
        let mut absorbed = 0usize;
        let mut distilled = String::with_capacity(1024);
        for (i, r) in results.iter().enumerate() {
            if let Ok(_id) = self.kb.write_memory_entry(
                &r.title,
                NodeType::Source,
                Some(&r.snippet),
                Some(&r.url),
                Some("nt_world_search"),
                None,
            ) {
                absorbed += 1;
            }
            if i < 8 {
                distilled.push_str(&format!("{} {} ", r.title, r.snippet));
            }
        }
        if let Ok(_id) = self.kb.write_memory_entry(
            &format!("research:{}", query),
            NodeType::Insight,
            Some(distilled.trim()),
            None,
            Some("nt_world_search"),
            None,
        ) {
            absorbed += 1;
        }

        // ── 蒸馏向量 (研究域内容感知) → custom source 吸收 ──
        let vector = self.derive_research_vector(distilled.trim());
        let custom_name = format!("research:{}:{}", query.chars().take(24).collect::<String>(), absorbed);
        brain.brain.register_knowledge_source(&custom_name, vector);
        let has_signal = brain.brain.absorb_from_custom(&custom_name);

        // ── Verify (EDV): 以 ResearchFindings 身份受校验吸收, 能力下降则回滚 ──
        let before_score = crate::neotrix::nt_mind::seal_core::core::PerformanceEvaluator::evaluate(
            &crate::neotrix::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );
        let critic_accepted =
            brain.absorb_with_critic(crate::core::KnowledgeSource::ResearchFindings);
        let after_score = crate::neotrix::nt_mind::seal_core::core::PerformanceEvaluator::evaluate(
            &crate::neotrix::nt_world_model::TaskType::General,
            &brain.brain.capability,
        );

        DialogueAbsorbOutcome {
            absorbed,
            critic_accepted,
            score_before: before_score,
            score_after: after_score,
            score_delta: if has_signal { after_score - before_score } else { 0.0 },
        }
    }

    /// 生产路径: 以统一搜索 (DDG→Wikipedia 有序后端) 执行查询并吸收结论。
    ///
    /// 网络失败 / 无结果时返回空 outcome (graceful), 不污染脑状态。
    /// researcher agent 与 background_loop 都经此入口, 保证搜索结论
    /// 唯一路径落 KB + 参与进化 (R-P42 强化现有节点, 无平行适配器)。
    pub fn absorb_research_query(
        &self,
        brain: &mut SelfIteratingBrain,
        query: &str,
        count: usize,
    ) -> DialogueAbsorbOutcome {
        let search = UnifiedSearch::new();
        match search.search(query, count) {
            Ok(results) if !results.is_empty() => {
                self.absorb_research_findings(brain, query, &results)
            }
            _ => DialogueAbsorbOutcome::empty(),
        }
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

    #[test]
    fn meta_shell_dispatches_catalog_profile() {
        // 派单桥回归: decide_and_run 后 last_dispatched 应记录 AgentCatalog 档案名。
        // 刺激 SelfReflection → 应派单 verifier (以判据回滚/防自确认陷阱)。
        let mut shell = MetaAgentShell::new("planning");
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let r = shell.decide_and_run();
        assert!(r.is_some());
        assert_eq!(shell.last_dispatched, Some("verifier"));
    }

    #[test]
    fn meta_shell_route_maps_all_supported_domains() {
        // route_to_catalog 对每个受支持域都返回内置档案名 (星系派单可全映射)。
        let mut shell = MetaAgentShell::new("planning");
        for domain in [
            AttentionDomain::PatternMatch,
            AttentionDomain::Planning,
            AttentionDomain::Code,
            AttentionDomain::Temporal,
            AttentionDomain::SelfReflection,
            AttentionDomain::Semantic,
        ] {
            shell.stimulate(domain, 0.9);
            assert!(
                shell.route_to_catalog().is_some(),
                "domain {:?} should resolve to a catalog agent",
                domain
            );
        }
    }

    #[test]
    fn route_learner_cold_start_uses_static() {
        // 冷启动 (无证据): 学习器不应覆盖静态映射。
        let learner = RouteLearner::new();
        assert_eq!(learner.route(AttentionDomain::SelfReflection, "verifier"), "verifier");
        assert!(!learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert!(learner.rates(AttentionDomain::SelfReflection).is_empty());
    }

    #[test]
    fn route_learner_overrides_static_with_evidence() {
        // D3: 该域历史证据显示 generalist 成功率更高 → 学习器应覆盖静态 verifier。
        let mut learner = RouteLearner::new();
        for _ in 0..5 {
            learner.record(AttentionDomain::SelfReflection, "generalist", true);
        }
        for _ in 0..5 {
            learner.record(AttentionDomain::SelfReflection, "verifier", false);
        }
        assert!(learner.has_enough_evidence(AttentionDomain::SelfReflection));
        let routed = learner.route(AttentionDomain::SelfReflection, "verifier");
        assert_eq!(routed, "generalist", "learner should override static mapping");
        // 诊断视图可用
        let rates = learner.rates(AttentionDomain::SelfReflection);
        assert_eq!(rates.len(), 2);
    }

    #[test]
    fn route_learner_below_evidence_keeps_static() {
        // 证据不足时即使有成功率差异也不覆盖 (防冷启动噪声)。
        let mut learner = RouteLearner::new();
        learner.record(AttentionDomain::SelfReflection, "generalist", true);
        learner.record(AttentionDomain::SelfReflection, "verifier", false);
        assert!(!learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert_eq!(learner.route(AttentionDomain::SelfReflection, "verifier"), "verifier");
    }

    #[test]
    fn route_learner_config_is_calibratable() {
        // P1: min_evidence 进配置, 默认 3 保持既有行为; 调低后证据阈值随之变化。
        let cfg = RouteLearnerConfig::default();
        assert_eq!(cfg.min_evidence, 3);
        // 自定义配置构造生效: min_evidence=1 → 1 次成功即覆盖静态映射。
        let mut learner = RouteLearner::with_config(RouteLearnerConfig { min_evidence: 1 });
        learner.record(AttentionDomain::SelfReflection, "generalist", true);
        assert!(learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert_eq!(learner.route(AttentionDomain::SelfReflection, "verifier"), "generalist");
        // 默认配置下同场景仍不足 (1 < 3)。
        let mut default = RouteLearner::new();
        default.record(AttentionDomain::SelfReflection, "generalist", true);
        assert!(!default.has_enough_evidence(AttentionDomain::SelfReflection));
    }

    #[test]
    fn route_learner_persist_load_round_trip() {
        // P1: 行为统计经 KB kv_store 持久化, 新实例可完整恢复 → 派单学习跨会话存活。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_route_learner_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));

        let mut original = RouteLearner::new();
        for _ in 0..5 {
            original.record(AttentionDomain::SelfReflection, "generalist", true);
        }
        for _ in 0..3 {
            original.record(AttentionDomain::SelfReflection, "verifier", false);
        }
        original.persist(&kb).expect("persist");

        // 全新实例 (冷启动) 加载后应还原证据 → 路由覆盖生效。
        let mut restored = RouteLearner::new();
        restored.load(&kb).expect("load");
        assert!(restored.has_enough_evidence(AttentionDomain::SelfReflection));
        let rates = restored.rates(AttentionDomain::SelfReflection);
        assert_eq!(rates.len(), 2, "both agents restored");
        // generalist 5/5 成功 > verifier 0/3 → 学习后覆盖静态 verifier。
        assert_eq!(restored.route(AttentionDomain::SelfReflection, "verifier"), "generalist");
    }

    #[test]
    fn route_learner_load_missing_archive_stays_cold() {
        // P1: 无存档时 load 为空操作, 不崩溃、不产生证据。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_route_learner_cold_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let mut learner = RouteLearner::new();
        learner.load(&kb).expect("load empty should be Ok");
        assert!(!learner.has_enough_evidence(AttentionDomain::SelfReflection));
        assert_eq!(learner.route(AttentionDomain::PatternMatch, "explorer"), "explorer");
    }

    #[test]
    fn route_learner_persist_drops_unknown_agents() {
        // P1: 持久化仅保留规范档案名 (researcher/explorer/planner/...), 未知名丢弃。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_route_learner_unk_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let mut learner = RouteLearner::new();
        learner.record(AttentionDomain::PatternMatch, "researcher", true);
        learner.record(AttentionDomain::PatternMatch, "not-a-real-agent", true);
        learner.persist(&kb).expect("persist");
        let mut restored = RouteLearner::new();
        restored.load(&kb).expect("load");
        let rates = restored.rates(AttentionDomain::PatternMatch);
        assert_eq!(rates.len(), 1, "unknown agent dropped on restore");
        assert_eq!(rates[0].0, "researcher");
    }

    #[test]
    fn dialogue_absorb_config_is_calibratable() {
        // D5: 参数集中可调, 默认值保持既有行为 (8 / 0.1)。
        let cfg = DialogueAbsorbConfig::default();
        assert_eq!(cfg.max_entries, 8);
        assert!((cfg.min_importance - 0.1).abs() < 1e-9);
        assert!((cfg.boost_base - 0.5).abs() < 1e-9);
        // 自定义参数构造生效
        let custom = DialogueAbsorbConfig {
            max_entries: 3,
            min_importance: 0.5,
            ..Default::default()
        };
        assert_eq!(custom.max_entries, 3);
    }

    #[test]
    fn dialogue_absorb_outcome_signals_positive() {
        // D1/D2: is_positive 要求 吸收 + 批评器接受 + 能力面变好 三条件齐备。
        let positive = DialogueAbsorbOutcome {
            absorbed: 2,
            critic_accepted: true,
            score_before: 0.4,
            score_after: 0.6,
            score_delta: 0.2,
        };
        assert!(positive.is_positive());
        // 批评器回滚 → 非正向 (自欺防御)
        let rolled_back = DialogueAbsorbOutcome {
            absorbed: 2,
            critic_accepted: false,
            score_before: 0.5,
            score_after: 0.5,
            score_delta: 0.0,
        };
        assert!(!rolled_back.is_positive());
        // 空结果
        assert!(!DialogueAbsorbOutcome::empty().is_positive());
        assert_eq!(DialogueAbsorbOutcome::empty().absorbed, 0);
    }

    #[test]
    fn dialogue_bridge_derive_vector_content_aware() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let cv = bridge.derive_vector("unit tests assert verify memory recall kb retention");
        assert!(cv.verification() > 0.0, "verification should be boosted");
        assert!(cv.semantic_layer() > 0.0, "semantic_layer should be boosted");
        assert!(cv.analysis() == 0.0, "no analysis keyword → stays 0");
    }

    #[test]
    fn dialogue_bridge_recent_experiences_filters() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_exp_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        kb.write_memory_entry(
            "session-test",
            NodeType::Session,
            Some("user asked about seal loop iteration"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        kb.write_memory_entry(
            "not-a-session",
            NodeType::Concept,
            Some("regular node"),
            None,
            Some("test"),
            None,
        )
        .expect("write concept node");
        let bridge = DialogueAbsorbBridge::new(kb);
        let exps = bridge.recent_experiences();
        assert_eq!(exps.len(), 1, "only Session-type node should be collected");
        assert_eq!(exps[0].title, "session-test");
    }

    #[test]
    fn dialogue_bridge_absorb_pending_moves_capability() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_abs_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        kb.write_memory_entry(
            "session-1",
            NodeType::Session,
            Some("user taught testing verification discipline"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        let bridge = DialogueAbsorbBridge::new(kb);
        let mut brain = SelfIteratingBrain::new();
        let outcome = bridge.absorb_pending(&mut brain);
        assert!(outcome.absorbed >= 1, "at least one dialogue experience should absorb");
        // 能力面应朝对话信号移动: 初始全零, 吸收后 verification 维度被提升。
        assert!(
            brain.brain.capability.verification() > 0.0,
            "verification dimension should move toward dialogue signal"
        );
        assert!(
            brain.brain.total_absorb_count > 0,
            "absorb count should increase"
        );
        // 源身份已记录 — DialogueExperience 通道写入吸收历史。
        let has_dialogue = brain
            .brain
            .absorption_history
            .iter()
            .any(|rec| rec.source == crate::core::KnowledgeSource::DialogueExperience);
        assert!(has_dialogue, "DialogueExperience should be in absorption history");
    }

    #[test]
    fn dialogue_bridge_absorbs_multiple_sessions() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_dialogue_multi_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        kb.write_memory_entry(
            "session-alpha",
            NodeType::Session,
            Some("testing verification discipline"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        kb.write_memory_entry(
            "session-beta",
            NodeType::Session,
            Some("planning and memory retrieval strategy"),
            None,
            Some("test"),
            None,
        )
        .expect("write session node");
        let bridge = DialogueAbsorbBridge::new(kb);
        let mut brain = SelfIteratingBrain::new();
        let outcome = bridge.absorb_pending(&mut brain);
        assert_eq!(outcome.absorbed, 2, "two distinct sessions should both absorb");
        // 原则级共振: 两条会话分别在 verification 与 semantic_layer 留下信号。
        assert!(brain.brain.capability.verification() > 0.0);
        assert!(brain.brain.capability.semantic_layer() > 0.0);
        // 行为化指标: outcome 记录了批评器判决 + 实测能力差, 不再是无意义计数。
        assert!(
            outcome.score_before >= 0.0 && outcome.score_after >= 0.0,
            "behavioral score must be measurable"
        );
    }

    fn research_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                title: "NeoTrix Search Backend Fallback".into(),
                url: "https://example.com/search-fallback".into(),
                snippet: "web search method with evidence source citation for verification and synthesis".into(),
            },
            SearchResult {
                title: "Ordered Backend Routing Research".into(),
                url: "https://example.com/routing".into(),
                snippet: "research paper arxiv study aggregating benchmark metrics and findings".into(),
            },
        ]
    }

    #[test]
    fn research_bridge_derive_research_vector_content_aware() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_research_vec_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let cv = bridge.derive_research_vector(
            "web search method with evidence source citation paper arxiv benchmark synthesis finding",
        );
        assert!(cv.semantic_layer() > 0.0, "search/web keywords should boost semantic_layer");
        assert!(cv.inference_depth() > 0.0, "paper/research keywords should boost inference_depth");
        assert!(cv.verification() > 0.0, "evidence/source keywords should boost verification");
        // 与对话域映射区分: 无 dialogue 关键词时 inference_depth 不应被对话映射污染。
        let dialogue_cv = bridge.derive_vector("just a chat");
        assert_eq!(dialogue_cv.inference_depth(), 0.0);
    }

    #[test]
    fn research_bridge_findings_absorb_moves_capability() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_research_abs_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb.clone());
        let mut brain = SelfIteratingBrain::new();
        let outcome = bridge.absorb_research_findings(&mut brain, "neotrix search", &research_results());
        // 结论落 KB: 2 条 Source + 1 条 Insight = 3 节点。
        assert_eq!(outcome.absorbed, 3, "2 sources + 1 insight should be written to KB");
        // 能力面吸收信号: 研究域关键词应提升 semantic_layer / inference_depth。
        assert!(
            brain.brain.capability.semantic_layer() > 0.0,
            "research absorb should move semantic_layer"
        );
        // 源身份已记录 — ResearchFindings 通道写入吸收历史。
        let has_research = brain
            .brain
            .absorption_history
            .iter()
            .any(|rec| rec.source == crate::core::KnowledgeSource::ResearchFindings);
        assert!(has_research, "ResearchFindings should be in absorption history");
        // KB 落盘可溯源: 结论以 Source 节点存在。
        let nodes = kb.all_nodes().expect("read kb");
        assert!(
            nodes.iter().any(|n| n.node_type == NodeType::Source),
            "research findings should be stored as Source nodes"
        );
    }

    #[test]
    fn research_bridge_empty_results_noop() {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_research_empty_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let bridge = DialogueAbsorbBridge::new(kb);
        let mut brain = SelfIteratingBrain::new();
        // 空结论 → 不落 KB 不吸收, graceful no-op。
        let outcome = bridge.absorb_research_findings(&mut brain, "query", &[]);
        assert_eq!(outcome.absorbed, 0);
        assert!(!outcome.is_positive());
        assert_eq!(brain.brain.total_absorb_count, 0);
    }

    #[test]
    fn research_source_registered_in_catalog() {
        // KnowledgeSource::ResearchFindings 完整登记: name / all / source_weight / 向量。
        let s = crate::core::KnowledgeSource::ResearchFindings;
        assert_eq!(s.name(), "neotrix-research-findings");
        assert!((s.source_weight() - 0.84).abs() < 1e-9);
        assert!(crate::core::KnowledgeSource::all().contains(&s));
        let cv = s.capability_vector();
        assert!(cv.verification() > 0.0, "research source should carry a real vector");
    }

    // ──────────────────────────────────────────────────────────────
    // P0 派单执行桥测试
    // ──────────────────────────────────────────────────────────────

    /// 探针执行器 — 记录被调用的 (agent, task), 返回可控结果。
    struct ProbeExecutor {
        pub calls: std::cell::RefCell<Vec<(String, String)>>,
        pub respond: AgentExecutionOutcome,
    }

    impl AgentExecutor for ProbeExecutor {
        fn execute(&self, agent: &str, task: &str) -> AgentExecutionOutcome {
            self.calls.borrow_mut().push((agent.to_string(), task.to_string()));
            self.respond.clone()
        }
    }

    #[test]
    fn domains_for_goal_multi_domain_stimulation() {
        // P0 缺陷 #3 修复: 不同目标语义刺激不同注意力域, 不再永远 SelfReflection。
        let research = domains_for_goal("research the latest papers on agent evolution");
        assert!(
            research.iter().any(|(d, _)| *d == AttentionDomain::PatternMatch),
            "research goal should stimulate PatternMatch"
        );
        let code = domains_for_goal("implement the refactor for core loop");
        assert!(
            code.iter().any(|(d, _)| *d == AttentionDomain::Code),
            "code goal should stimulate Code"
        );
        let review = domains_for_goal("review the diff and verify changes");
        assert!(
            review.iter().any(|(d, _)| *d == AttentionDomain::RiskAssessment),
            "review goal should stimulate RiskAssessment"
        );
        // 空文本 → 弱自省兜底 (防 10 域全空空转)。
        let empty = domains_for_goal("   ");
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].0, AttentionDomain::SelfReflection);
    }

    #[test]
    fn tree_branch_weakness_derived_from_health_fog_constellation() {
        // P2: 分支薄弱度 = (1-health)*0.4 + fog*0.4 + (1-constellation)*0.2。
        // 健康分支 (满 health, 低 fog, 满 constellation) → 0。
        let mut healthy = CapabilityBranch::new(BranchKind::Memory);
        healthy.health = 1.0;
        healthy.fog = crate::core::nt_core_consciousness_tree::FogLevel {
            wired: true,
            consumer_count: 3,
            has_tests: true,
            level: 0.05,
        };
        healthy.constellation = crate::core::nt_core_consciousness_tree::Constellation {
            level: 5,
            c0_compiles: true,
            c1_unit_tests: true,
            c2_integration: true,
            c3_benchmark: true,
            c4_pipeline: true,
            c5_self_healing: true,
            c6_adaptive: true,
        };
        let w_healthy = branch_weakness(&healthy);
        assert!(w_healthy < 0.1, "healthy branch should be ~0, got {w_healthy}");
        // 薄弱分支 (health 0, 高 fog, C0) → 接近 1。
        let mut weak = CapabilityBranch::new(BranchKind::Memory);
        weak.health = 0.0;
        let w_weak = branch_weakness(&weak);
        assert!(w_weak > 0.7, "weak branch should be high, got {w_weak}");
    }

    #[test]
    fn tree_branch_stimuli_skips_healthy_drives_weak() {
        // P2: 树作为控制面 — 健康分支不产生刺激, 薄弱分支驱动对应注意力域。
        let mut tree = ConsciousnessTree::new();
        // Memory 分支设健康 (低薄弱度) → 不应产生 Semantic 刺激。
        if let Some(mem) = tree.branches.get_mut(&BranchKind::Memory) {
            mem.health = 1.0;
            mem.fog = crate::core::nt_core_consciousness_tree::FogLevel {
                wired: true,
                consumer_count: 3,
                has_tests: true,
                level: 0.05,
            };
            mem.constellation = crate::core::nt_core_consciousness_tree::Constellation {
                level: 5,
                c0_compiles: true,
                c1_unit_tests: true,
                c2_integration: true,
                c3_benchmark: true,
                c4_pipeline: true,
                c5_self_healing: true,
                c6_adaptive: true,
            };
        }
        // Shield 分支保持默认 (health 0, fog 0.85, C0) → 薄弱 → 应刺激 RiskAssessment。
        let stimuli = tree_branch_stimuli(&tree);
        assert!(
            stimuli.iter().any(|(d, _)| *d == AttentionDomain::RiskAssessment),
            "weak Shield branch should stimulate RiskAssessment"
        );
        assert!(
            !stimuli.iter().any(|(d, _)| *d == AttentionDomain::Semantic),
            "healthy Memory branch should not stimulate Semantic"
        );
    }

    #[test]
    fn tree_branch_stimuli_fuses_with_goal_dispatch() {
        // P2: 端到端 — 薄弱分支刺激 + 派单执行闭环。Memory 薄弱 → Semantic 刺激 →
        // dispatch_and_execute 派单 watcher (Semantic→watcher 静态映射)。
        // 其余分支全部健康, 保证 Semantic 是唯一强刺激 → 派单确定性。
        let mut tree = ConsciousnessTree::new();
        for (kind, branch) in tree.branches.iter_mut() {
            if kind == &BranchKind::Memory {
                branch.health = 0.0; // 薄弱
            } else {
                branch.health = 1.0;
                branch.fog = crate::core::nt_core_consciousness_tree::FogLevel {
                    wired: true,
                    consumer_count: 3,
                    has_tests: true,
                    level: 0.05,
                };
            }
        }
        let mut shell = MetaAgentShell::new("dialogue");
        for (domain, amount) in tree_branch_stimuli(&tree) {
            shell.stimulate(domain, amount);
        }
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("consolidate ran".into()),
        };
        let (agent, _) = shell.dispatch_and_execute(&probe, "general_dialogue_tick").expect("dispatch");
        // Semantic 刺激应让 watcher 成为主导派单 (Memory 分支薄弱被树驱动)。
        assert_eq!(agent, "watcher");
    }

    #[test]
    fn branch_attention_domains_covers_all_kinds() {
        // P2: 7 星系分支均有非空注意力域映射 (控制面完整性)。
        for kind in BranchKind::all() {
            let domains = branch_attention_domains(&kind);
            assert!(!domains.is_empty(), "branch {kind:?} should map to >=1 attention domain");
        }
    }

    #[test]
    fn dispatch_and_execute_feeds_learner_with_real_result() {
        // 派单不再只 eprintln — 执行结果真实喂回 RouteLearner。
        let mut shell = MetaAgentShell::new("planning");
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("probe did work".into()),
        };
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let (agent, outcome) = shell.dispatch_and_execute(&probe, "review the codebase").expect("dispatch");
        // SelfReflection → verifier 档案被激活, 且探针确实被调用。
        assert_eq!(agent, "verifier");
        assert!(outcome.is_success());
        assert_eq!(shell.last_dispatched, Some("verifier"));
        // 执行器被调用了一次, 参数正确。
        let calls = probe.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "verifier");
        // learner 已记录: verifier 在 SelfReflection 域上的一次成功。
        assert_eq!(shell.learner.rates(AttentionDomain::SelfReflection).len(), 1);
    }

    #[test]
    fn dispatch_and_execute_failure_does_not_reinforce() {
        // 执行失败 → 不强化该档案 (行为信号防自欺)。
        let mut shell = MetaAgentShell::new("planning");
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Failure("backend down".into()),
        };
        shell.stimulate(AttentionDomain::SelfReflection, 0.9);
        let (agent, outcome) = shell.dispatch_and_execute(&probe, "audit").expect("dispatch");
        assert_eq!(agent, "verifier");
        assert!(!outcome.is_success());
        // 失败记录仍入 learner (attempts+1), 但成功计数不增。
        let rates = shell.learner.rates(AttentionDomain::SelfReflection);
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].2, 1, "one attempt recorded");
        assert_eq!(rates[0].1, 0.0, "zero success rate");
    }

    #[test]
    fn dispatch_research_task_routes_to_researcher() {
        // P0 缺陷 #4 修复: PatternMatch 域 + research 文本 → researcher (网络研究),
        // 而非一律 explorer (KB 检索) — 融合关键词路由与注意力静态映射。
        let mut shell = MetaAgentShell::new("planning");
        let probe = ProbeExecutor {
            calls: std::cell::RefCell::new(Vec::new()),
            respond: AgentExecutionOutcome::Success("research done".into()),
        };
        // 刺激 PatternMatch (检索域), 目标是研究任务。
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let (agent, outcome) = shell
            .dispatch_and_execute(&probe, "research the latest papers on agents")
            .expect("dispatch");
        assert_eq!(agent, "researcher", "research text should route to researcher");
        assert!(outcome.is_success());
        // 纯探索文本 → explorer (KB 检索)。
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let (agent, _) = shell
            .dispatch_and_execute(&probe, "explore the codebase structure")
            .expect("dispatch");
        assert_eq!(agent, "explorer", "explore text should route to explorer");
        // 越界关键词 (非 PatternMatch 语义) → 退回注意力静态映射。
        shell.stimulate(AttentionDomain::PatternMatch, 0.9);
        let (agent, _) = shell
            .dispatch_and_execute(&probe, "implement a refactor for core")
            .expect("dispatch");
        assert_eq!(agent, "explorer", "code text outside PatternMatch semantics falls back to explorer");
    }

    #[test]
    fn production_executor_maps_profiles_to_real_actions() {
        // P0 断点 1/2/8 修复: 档案名映射到真实子系统动作。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_executor_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = std::sync::Arc::new(KnowledgeBase::open(Some(tmp.into())).expect("open kb"));
        let exec = ProductionAgentExecutor::new(kb);
        // watcher → KB 规模监控 (确定性, 无网络依赖)。
        match exec.execute("watcher", "health probe") {
            AgentExecutionOutcome::Success(s) => assert!(s.contains("KB nodes"), "watcher should probe KB: {}", s),
            other => panic!("watcher should succeed offline: {:?}", other),
        }
        // verifier → 证据溯源计数 (确定性)。
        match exec.execute("verifier", "evidence audit") {
            AgentExecutionOutcome::Success(s) => assert!(s.contains("evidence"), "verifier should trace evidence: {}", s),
            other => panic!("verifier should succeed offline: {:?}", other),
        }
        // planner → 无副作用规划占位 (规划由 metacog cycle 产出)。
        match exec.execute("planner", "design plan") {
            AgentExecutionOutcome::NoOp(s) => assert!(s.contains("planning"), "planner should be NoOp: {}", s),
            other => panic!("planner should be NoOp: {:?}", other),
        }
        // 未知档案 → NoOp (不崩溃, 可审计)。
        match exec.execute("ghost", "whatever") {
            AgentExecutionOutcome::NoOp(s) => assert!(s.contains("ghost"), "unknown agent should NoOp: {}", s),
            other => panic!("unknown agent should NoOp: {:?}", other),
        }
    }

    #[test]
    fn execution_outcome_summary_human_readable() {
        assert!(AgentExecutionOutcome::Success("done".into()).summary().contains("success"));
        assert!(AgentExecutionOutcome::NoOp("none".into()).summary().contains("noop"));
        assert!(AgentExecutionOutcome::Failure("boom".into()).summary().contains("failed"));
        assert!(AgentExecutionOutcome::Success("x".into()).is_success());
        assert!(!AgentExecutionOutcome::Failure("x".into()).is_success());
    }
}
