//! 执行与评估 (记忆执行器 + 生产执行桥 + 元认知外壳 + 域映射) 。

use super::super::co_evolution::{CoEvoConfig, CoEvolutionLoop};
use super::nt_capability_registry::{DispatchTopology, RouteLearner};
use super::nt_capability_types::{
    AgentExecutionOutcome, AgentExecutor, CapabilityOutcome, MemoryAgentCapability,
    RouteLearnerConfig, TopologyRepair,
};
use crate::l5_cognition::l1_facade::attention_head::{AttentionDomain, AttentionManager};
use crate::l5_cognition::l1_facade::{
    KnowledgeBase, MetaCognitiveLoop, MetaCycleResult, UnifiedSearch,
};
use crate::l5_cognition::nt_core_consciousness_tree::{
    BranchKind, CapabilityBranch, ConsciousnessTree,
};
use neotrix_types::knowledge_access::NodeType;

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

    fn capability_retrieve(&self, query: &str, limit: usize) -> Result<CapabilityOutcome, String> {
        let nodes = self.kb.search_permission_aware(
            query,
            limit,
            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::default(),
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

    /// 深特征路由（D1，deep_route.py 的 Rust 镜像）：只读任务文本。
    /// 不可逆/高风险 → researcher 先取证（保守）；校准/概率 → verifier；
    /// 健康/计数 → watcher；定位/文件 → explorer；默认 generalist。
    pub(crate) fn selfcall_route(task: &str) -> &'static str {
        let high_stakes = [
            "删库",
            "不可逆",
            "irreversible",
            "生产",
            "prod",
            "线上",
            "删除",
        ];
        let calib = ["校准", "概率", "calib", "confidence", "置信"];
        let health = ["健康", "数量", "health", "count", "规模"];
        let locate = ["定位", "文件", "locate", "代码", "file"];
        let hits = |words: &[&str]| words.iter().any(|w| task.contains(w));
        if hits(&high_stakes) {
            "researcher"
        } else if hits(&calib) {
            "verifier"
        } else if hits(&health) {
            "watcher"
        } else if hits(&locate) {
            "explorer"
        } else {
            "generalist"
        }
    }
}

impl AgentExecutor for ProductionAgentExecutor {
    fn execute(&self, agent: &str, task: &str) -> AgentExecutionOutcome {
        match agent {
            "researcher" => match self.search.search(task, 5) {
                Ok(results) if !results.is_empty() => AgentExecutionOutcome::Success(format!(
                    "searched {} results (backend={})",
                    results.len(),
                    self.search.active_backend(),
                )),
                Ok(_) => AgentExecutionOutcome::NoOp("search returned no results".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("search: {}", e)),
            },
            "explorer" => match self.memory.capability_retrieve(task, 5) {
                Ok(CapabilityOutcome::Hits(n, first)) if n > 0 => AgentExecutionOutcome::Success(
                    format!("retrieved {} hits (first: {})", n, first),
                ),
                Ok(_) => AgentExecutionOutcome::NoOp("no KB hits".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("retrieve: {}", e)),
            },
            "verifier" => match self.memory.capability_evidence() {
                Ok(CapabilityOutcome::Count(n)) => {
                    AgentExecutionOutcome::Success(format!("evidence audit: {} sources traced", n))
                }
                Ok(_) => AgentExecutionOutcome::NoOp("evidence count unavailable".into()),
                Err(e) => AgentExecutionOutcome::Failure(format!("evidence: {}", e)),
            },
            "watcher" => match self.memory.capability_consolidate() {
                Ok(CapabilityOutcome::Count(n)) => {
                    AgentExecutionOutcome::Success(format!("health probe: {} KB nodes", n))
                }
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
                        AgentExecutionOutcome::Success(format!(
                            "combined {} hits, {} evidence",
                            n, m
                        ))
                    }
                    (Err(err), _) => AgentExecutionOutcome::Failure(format!("retrieve: {}", err)),
                    _ => AgentExecutionOutcome::NoOp("no combined signal".into()),
                }
            }
            // selfcaller: 能力自调用（D1 融合）— 按深层特征选子臂执行，全程留痕。
            // selfcall_route 永不返回 "selfcaller"，无递归。
            "selfcaller" => {
                let sub = Self::selfcall_route(task);
                match self.execute(sub, task) {
                    AgentExecutionOutcome::Success(s) => {
                        AgentExecutionOutcome::Success(format!("selfcall [{sub}]: {s}"))
                    }
                    AgentExecutionOutcome::NoOp(s) => {
                        AgentExecutionOutcome::NoOp(format!("selfcall [{sub}]: {s}"))
                    }
                    AgentExecutionOutcome::Failure(s) => {
                        AgentExecutionOutcome::Failure(format!("selfcall [{sub}]: {s}"))
                    }
                }
            }
            other => AgentExecutionOutcome::NoOp(format!("no executor for agent '{}'", other)),
        }
    }

    /// 策略感知执行 (P4, MAGE task-level search bandit) — 当任务级搜索 bandit 选出
    /// 非默认策略时, explorer 走既有 confidence 检索缝 `search_with_confidence`
    /// (R-P42: 强化既有检索节点, 非平行路径)。默认/unknown 策略回落常规执行。
    fn execute_with_strategy(
        &self,
        agent: &str,
        task: &str,
        strategy: &str,
    ) -> AgentExecutionOutcome {
        use crate::l5_cognition::nt_mind::nt_mind::evolution::co_evolution::{
            parse_strategy, strategy_name,
        };
        let strategy = strategy_name(strategy);
        match (agent, strategy) {
            ("explorer", "balanced") => self.execute(agent, task),
            ("explorer", parsed) => {
                match self
                    .memory
                    .kb
                    .search_with_confidence(task, parse_strategy(parsed), 5)
                {
                    Ok(results) if !results.is_empty() => AgentExecutionOutcome::Success(format!(
                        "retrieved {} hits (strategy={})",
                        results.len(),
                        parsed
                    )),
                    Ok(_) => AgentExecutionOutcome::NoOp("no KB hits".into()),
                    Err(e) => AgentExecutionOutcome::Failure(format!("retrieve: {}", e)),
                }
            }
            _ => self.execute(agent, task),
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
    /// MANTA 式派单拓扑 (P3) — 域→档案边集合, 推理期可自进化。
    /// 静态映射退居"初始组织", 拓扑经 trace 审计 + 有界修复自进化。
    pub topology: DispatchTopology,
    /// MAGE 四子图共进化循环 (P4) — 同一 reward 驱动任务级搜索 bandit + 图共进化;
    /// 技能级路由 bandit 复用 `learner`。任务类型用于选择检索策略。
    pub coevo: CoEvolutionLoop,
    /// 本外壳服务的任务类型 — 任务级搜索 bandit 的分组键。
    pub task_type: String,
}

impl MetaAgentShell {
    pub fn new(task_type: &str) -> Self {
        // 按任务类型选择强度 + Weapon Set (Ascendancy 双专精路由)
        let attention = AttentionManager::from_task_type(0.3, task_type);
        let metacog = MetaCognitiveLoop::new(crate::l5_cognition::l1_facade::MetaSelfModel::new());
        Self {
            attention,
            metacog,
            iterations_run: 0,
            last_dispatched: None,
            learner: RouteLearner::new(),
            topology: DispatchTopology::for_task_type(task_type),
            coevo: CoEvolutionLoop::new(),
            task_type: task_type.to_string(),
        }
    }

    /// 以自定义路由学习配置构造 (P1: min_evidence 等经 config 注入)。
    pub fn with_learner_config(task_type: &str, learner_config: RouteLearnerConfig) -> Self {
        let attention = AttentionManager::from_task_type(0.3, task_type);
        let metacog = MetaCognitiveLoop::new(crate::l5_cognition::l1_facade::MetaSelfModel::new());
        Self {
            attention,
            metacog,
            iterations_run: 0,
            last_dispatched: None,
            learner: RouteLearner::with_config(learner_config),
            topology: DispatchTopology::for_task_type(task_type),
            coevo: CoEvolutionLoop::new(),
            task_type: task_type.to_string(),
        }
    }

    /// 以自定义共进化配置构造 (P4: epsilon/max_memories/min_evidence 注入)。
    pub fn with_coevo_config(task_type: &str, coevo_config: CoEvoConfig) -> Self {
        let attention = AttentionManager::from_task_type(0.3, task_type);
        let metacog = MetaCognitiveLoop::new(crate::l5_cognition::l1_facade::MetaSelfModel::new());
        Self {
            attention,
            metacog,
            iterations_run: 0,
            last_dispatched: None,
            learner: RouteLearner::new(),
            topology: DispatchTopology::for_task_type(task_type),
            coevo: CoEvolutionLoop::with_config(coevo_config),
            task_type: task_type.to_string(),
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
        // P3: 静态映射退居初始组织 — 派单拓扑的边是当前组织, 可被 trace 审计
        // + 有界修复自进化 (MANTA)。冷启动时拓扑 = 任务条件化初始边。
        let static_agent = self.topology.agent_for(dominant);
        // D3: 学习化路由 — 拓扑边为基线, 有足够证据时用结果反馈覆盖。
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
            // P0#4：关键词路由内联（AgentCatalog 已移除，逻辑落于此，契约见上）。
            // 非研究文本回退静态映射 + learner 校正（原行为不变）。
            let hint_lc = task_hint.to_lowercase();
            if ["research", "研究", "synthesize"]
                .iter()
                .any(|w| hint_lc.contains(w))
            {
                return Some("researcher");
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
        // W2.2 (batch3, arxiv 2608.20256 Learning When to Think):
        // 测试时算力自适应分配 — System2 判定的任务追加一轮深思迭代,
        // System1 直通单轮。行为差异即路由落地 (R-P79)。
        let alloc = self.attention.allocate_for_task(&self.task_type);
        let mut result = self.metacog.run_cycle();
        if alloc.mode == crate::l5_cognition::l1_facade::attention_head::ThinkingMode::System2Deliberate
        {
            result = self.metacog.run_cycle();
        }
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
    ///
    /// P4 (MAGE): 同一次执行结果构成**单一 reward 流**, 同时驱动技能级路由 bandit
    /// (`learner`) 与四子图共进化循环 (`coevo`, 含任务级搜索 bandit)。检索策略由
    /// `coevo.select_strategy` 选出并注入 `execute_with_strategy` — 让任务 bandit 的
    /// reward 与其选择因果绑定, 而非噪声。
    pub fn dispatch_and_execute(
        &mut self,
        executor: &dyn AgentExecutor,
        task: &str,
    ) -> Option<(&'static str, AgentExecutionOutcome)> {
        self.attention.decay_all();
        let dominant = self.attention.dominant_domain()?;
        let agent = self.route_with_hint(task)?;
        self.last_dispatched = Some(agent);
        let strategy = self.coevo.select_strategy(&self.task_type);
        let outcome = executor.execute_with_strategy(agent, task, &strategy);
        // 真实行为信号: 执行成功才强化该档案对该域的派单 (技能级路由 bandit)。
        self.learner.record(dominant, agent, outcome.is_success());
        // P4: 同一 reward 流同时更新四子图 + 任务级搜索 bandit (MAGE 共进化)。
        self.coevo.record_reward(
            &self.task_type,
            dominant,
            agent,
            &strategy,
            outcome.is_success(),
            &outcome.summary(),
        );
        Some((agent, outcome))
    }

    /// 激活特定域 — 供上层 (background_loop) 按事件触发。
    pub fn stimulate(&mut self, domain: AttentionDomain, amount: f64) {
        self.attention.stimulate_domain(domain, amount);
    }

    /// MANTA trace 审计 + 有界结构修复 (P3) — 依据 learner 的 (域, 档案, 成败)
    /// 行为 trace, 发现"当前组织不足"时改派单拓扑的边 (域→档案), 让组织自进化。
    /// 返回实际应用的修复列表 (空 = 当前组织仍足够)。
    pub fn audit_and_repair_topology(&mut self) -> Vec<TopologyRepair> {
        let repairs =
            self.topology
                .audit_with_experience(&self.learner, &self.coevo, &self.task_type);
        let mut applied = Vec::new();
        for repair in repairs {
            if self.topology.apply_repair(&repair) {
                applied.push(repair);
            }
        }
        applied
    }

    /// 拓扑跨轮 playbook — 持久化到 KB (MANTA: cross-run experience)。
    pub fn persist_topology(&self, kb: &KnowledgeBase) -> Result<(), String> {
        self.topology.persist(kb)
    }

    /// 拓扑跨轮 playbook — 从 KB 恢复 (冷启动无存档则保持当前)。
    pub fn load_topology(&mut self, kb: &KnowledgeBase) -> Result<(), String> {
        self.topology.load(kb)
    }

    /// 四子图共进化循环持久化 (P4, MAGE) — 图谱 + 任务级搜索 bandit 跨会话存活。
    pub fn persist_coevo(&self, kb: &KnowledgeBase) -> Result<(), String> {
        self.coevo.persist(kb)
    }

    /// 四子图共进化循环恢复 (P4) — 冷启动无存档则保持空图谱 (从零累积, 不重头再来)。
    pub fn load_coevo(&mut self, kb: &KnowledgeBase) -> Result<(), String> {
        self.coevo.load(kb)
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
///
/// 返回 (域, 刺激强度) 列表; 空文本时只给弱自省, 防空转。
pub fn domains_for_goal(goal: &str) -> Vec<(AttentionDomain, f64)> {
    let lower = goal.to_lowercase();
    let has = |kws: &[&str]| kws.iter().any(|k| lower.contains(k));
    if goal.trim().is_empty() {
        return vec![(AttentionDomain::SelfReflection, 0.2)];
    }
    let mut domains = Vec::new();
    if has(&[
        "research",
        "search",
        "研究",
        "搜索",
        "分析",
        "find",
        "aggregate",
    ]) {
        domains.push((AttentionDomain::PatternMatch, 0.8));
        domains.push((AttentionDomain::SelfReflection, 0.4));
    }
    if has(&[
        "code",
        "implement",
        "fix",
        "refactor",
        "编码",
        "实现",
        "修复",
        "重构",
    ]) {
        domains.push((AttentionDomain::Code, 0.8));
        domains.push((AttentionDomain::ToolUse, 0.6));
    }
    if has(&[
        "design",
        "architecture",
        "plan",
        "方案",
        "架构",
        "设计",
        "规划",
    ]) {
        domains.push((AttentionDomain::Planning, 0.8));
        domains.push((AttentionDomain::Creativity, 0.4));
    }
    if has(&["monitor", "watch", "health", "监控", "心跳", "巩固"]) {
        domains.push((AttentionDomain::Semantic, 0.7));
    }
    if has(&[
        "review", "verify", "audit", "rollback", "审查", "校验", "回滚",
    ]) {
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
    // T3: 优先用元认知校准后的健康分 (calibrated_health) 而非原始 health, 防止过度自信域
    // 在 GWT 路由中被低估为健康 → 注意力错配 (D15)。未校准分支 calibrated_health=0 (哨兵)
    // → 回退到 health, 保持与既有生产路径一致 (set_branch_health_from_self_tests 已填充)。
    let eff_health = if branch.calibrated_health > 0.0 {
        branch.calibrated_health
    } else {
        branch.health
    };
    let health_weak = (1.0 - eff_health.clamp(0.0, 1.0)) * 0.4;
    let fog_weak = branch.fog.level.clamp(0.0, 1.0) * 0.4;
    let constel_weak = (1.0 - branch.constellation.score().clamp(0.0, 1.0)) * 0.2;
    (health_weak + fog_weak + constel_weak).clamp(0.0, 1.0)
}

/// 分支 → 注意力域映射 (P2 控制面) — 每个星系分支薄弱时应刺激哪些域。
pub fn branch_attention_domains(kind: &BranchKind) -> Vec<AttentionDomain> {
    match kind {
        BranchKind::Core => vec![
            AttentionDomain::SelfReflection,
            AttentionDomain::RiskAssessment,
        ],
        BranchKind::Mind => vec![AttentionDomain::Creativity, AttentionDomain::Planning],
        BranchKind::Memory => vec![AttentionDomain::Semantic],
        BranchKind::World => vec![AttentionDomain::PatternMatch],
        BranchKind::Act => vec![AttentionDomain::GoalAlignment, AttentionDomain::ToolUse],
        BranchKind::Io => vec![AttentionDomain::Code, AttentionDomain::ToolUse],
        BranchKind::Shield => vec![
            AttentionDomain::RiskAssessment,
            AttentionDomain::SelfReflection,
        ],
        BranchKind::Meta => vec![AttentionDomain::SelfReflection, AttentionDomain::Planning],
        BranchKind::Repair => vec![AttentionDomain::RiskAssessment, AttentionDomain::ToolUse],
        BranchKind::Governance => vec![
            AttentionDomain::RiskAssessment,
            AttentionDomain::GoalAlignment,
        ],
        BranchKind::Nexus => vec![AttentionDomain::Semantic, AttentionDomain::SelfReflection],
        BranchKind::Game => vec![AttentionDomain::Creativity, AttentionDomain::GoalAlignment],
    }
}

impl MemoryAgent {
    /// Fallible constructor: returns `Err` if KB cannot be opened.
    pub fn try_new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let kb = KnowledgeBase::open(None)?;
        Ok(Self {
            kb: std::sync::Arc::new(kb),
        })
    }
}

impl Default for MemoryAgent {
    fn default() -> Self {
        match Self::try_new() {
            Ok(agent) => agent,
            Err(e) => {
                tracing::warn!(
                    "MemoryAgent default fallback: KB open failed ({e}), returning empty agent"
                );
                Self {
                    kb: std::sync::Arc::new(
                        KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:")))
                            .expect("in-memory KB must always open"),
                    ),
                }
            }
        }
    }
}
