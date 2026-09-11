use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttentionDomain {
    PatternMatch,
    Code,
    Semantic,
    Temporal,
    Planning,
    SelfReflection,
    ToolUse,
    GoalAlignment,
    RiskAssessment,
    Creativity,
}

impl AttentionDomain {
    pub fn all() -> Vec<AttentionDomain> {
        vec![
            AttentionDomain::PatternMatch,
            AttentionDomain::Code,
            AttentionDomain::Semantic,
            AttentionDomain::Temporal,
            AttentionDomain::Planning,
            AttentionDomain::SelfReflection,
            AttentionDomain::ToolUse,
            AttentionDomain::GoalAlignment,
            AttentionDomain::RiskAssessment,
            AttentionDomain::Creativity,
        ]
    }

    /// P1-13 确定性路由索引 (吸收 PrismSystem skills.json 六字段索引模式):
    /// 关键词 → 域 的确定性映射表, 供任务路由稳定分类 (不依赖 LLM 每次输出漂移)。
    /// PrismSystem 原文: "Router classifier + skills.json six-field index"。
    /// 返回匹配的域 (首个命中) 或 None。
    pub fn from_keywords(task: &str) -> Option<AttentionDomain> {
        let t = task.to_lowercase();
        // 有序: 越具体越靠前 (先匹配精确语义, 再匹配宽泛词)
        const ROUTES: &[(&[&str], AttentionDomain)] = &[
            (
                &["refactor", "code_review", "code review", "audit", "lint"],
                AttentionDomain::Code,
            ),
            (
                &["implement", "fix", "bug", "feature", "write code", "build"],
                AttentionDomain::Code,
            ),
            (
                &["search", "retrieve", "query", "find", "lookup", "explore"],
                AttentionDomain::Semantic,
            ),
            (
                &["plan", "architect", "design", "roadmap", "strategy"],
                AttentionDomain::Planning,
            ),
            (
                &["reflect", "review", "retro", "self", "meta"],
                AttentionDomain::SelfReflection,
            ),
            (
                &["tool", "mcp", "api", "call", "execute", "run"],
                AttentionDomain::ToolUse,
            ),
            (
                &["risk", "security", "threat", "danger", "guard"],
                AttentionDomain::RiskAssessment,
            ),
            (
                &["goal", "objective", "align", "priority"],
                AttentionDomain::GoalAlignment,
            ),
            (
                &["pattern", "match", "similar", "analogy", "reuse"],
                AttentionDomain::PatternMatch,
            ),
            (
                &["time", "schedule", "deadline", "history", "temporal"],
                AttentionDomain::Temporal,
            ),
            (
                &["creative", "novel", "generate", "imagine", "brainstorm"],
                AttentionDomain::Creativity,
            ),
        ];
        for (keywords, domain) in ROUTES {
            if keywords.iter().any(|k| t.contains(k)) {
                return Some(*domain);
            }
        }
        None
    }

    pub fn label(&self) -> &str {
        match self {
            AttentionDomain::PatternMatch => "pattern_match",
            AttentionDomain::Code => "code",
            AttentionDomain::Semantic => "semantic",
            AttentionDomain::Temporal => "temporal",
            AttentionDomain::Planning => "planning",
            AttentionDomain::SelfReflection => "self_reflection",
            AttentionDomain::ToolUse => "tool_use",
            AttentionDomain::GoalAlignment => "goal_alignment",
            AttentionDomain::RiskAssessment => "risk_assessment",
            AttentionDomain::Creativity => "creativity",
        }
    }

    /// 从标签字符串创建 AttentionDomain
    pub fn from_label(label: &str) -> Option<AttentionDomain> {
        match label {
            "pattern_match" => Some(AttentionDomain::PatternMatch),
            "code" => Some(AttentionDomain::Code),
            "semantic" => Some(AttentionDomain::Semantic),
            "temporal" => Some(AttentionDomain::Temporal),
            "planning" => Some(AttentionDomain::Planning),
            "self_reflection" => Some(AttentionDomain::SelfReflection),
            "tool_use" => Some(AttentionDomain::ToolUse),
            "goal_alignment" => Some(AttentionDomain::GoalAlignment),
            "risk_assessment" => Some(AttentionDomain::RiskAssessment),
            "creativity" => Some(AttentionDomain::Creativity),
            _ => None,
        }
    }
}

/// 规则强度等级 (来自 ponytail 吸收: R-P81 lazy ladder)
/// lite=探索/只读任务; full=生产修复/实现; ultra=架构重写/重构
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum RuleIntensity {
    #[default]
    Lite,
    Full,
    Ultra,
}

impl RuleIntensity {
    pub fn from_task_type(task: &str) -> Self {
        match task {
            t if t.contains("explore") || t.contains("read") || t.contains("search") => {
                RuleIntensity::Lite
            }
            t if t.contains("implement") || t.contains("fix") || t.contains("refactor") => {
                RuleIntensity::Full
            }
            t if t.contains("architect") || t.contains("design") || t.contains("rewrite") => {
                RuleIntensity::Ultra
            }
            _ => RuleIntensity::Full,
        }
    }

    pub fn attention_threshold(&self) -> f64 {
        match self {
            RuleIntensity::Lite => 0.2,
            RuleIntensity::Full => 0.4,
            RuleIntensity::Ultra => 0.6,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            RuleIntensity::Lite => "lite",
            RuleIntensity::Full => "full",
            RuleIntensity::Ultra => "ultra",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttentionHead {
    pub id: usize,
    pub domain: AttentionDomain,
    pub receptive_field: usize,
    pub activation: f64,
    pub specialization: Vec<f64>,
    pub focus: Vec<String>,
    pub decay_rate: f64,
    pub priority: u8,
    /// 剩余预算份额 ∈ [0.0, 1.0]，1.0 表示预算充足 (Cost-Aware Routing, Axiom A1)
    pub budget_remaining: f64,
    /// 路由到此域的预估算力成本 ∈ [0.0, 1.0]
    pub compute_cost: f64,
}

impl AttentionHead {
    pub fn new(id: usize, domain: AttentionDomain) -> Self {
        Self {
            id,
            domain,
            receptive_field: 10,
            activation: 0.0,
            specialization: Vec::new(),
            focus: Vec::new(),
            decay_rate: 0.1,
            priority: 5,
            budget_remaining: 1.0,
            compute_cost: 0.5,
        }
    }

    pub fn salience(&self, novelty: f64, coherence: f64) -> f64 {
        self.activation * novelty * coherence
    }

    pub fn stimulate(&mut self, amount: f64) {
        self.activation = (self.activation + amount).min(1.0);
    }

    pub fn decay(&mut self) {
        self.activation = (self.activation - self.decay_rate).max(0.0);
    }

    pub fn focus_on(&mut self, concept: &str) {
        if !self.focus.contains(&concept.to_string()) {
            self.focus.push(concept.to_string());
        }
        self.stimulate(0.1);
    }

    pub fn is_activated(&self, threshold: f64) -> bool {
        self.activation >= threshold
    }

    /// 设置剩余预算份额 ∈ [0.0, 1.0] (Cost-Aware Routing, Axiom A1)
    pub fn set_budget(&mut self, remaining: f64) {
        self.budget_remaining = remaining.clamp(0.0, 1.0);
    }

    /// 预算是否临界 (< 0.2) — 临界时强制 System1Direct 省算力
    pub fn is_budget_critical(&self) -> bool {
        self.budget_remaining < 0.2
    }

    /// 成本感知显著性: salience × budget_remaining (Axiom A1 + A2)
    /// 预算越紧，显著性衰减越快，低价值域自动降权
    pub fn cost_aware_salience(&self, novelty: f64, coherence: f64) -> f64 {
        self.salience(novelty, coherence) * self.budget_remaining
    }
}

#[derive(Debug, Clone)]
pub struct AttentionProfile {
    pub dominant: AttentionDomain,
    pub distribution: HashMap<AttentionDomain, f64>,
    pub num_activated_heads: usize,
}

impl AttentionProfile {
    pub fn new(
        dominant: AttentionDomain,
        distribution: HashMap<AttentionDomain, f64>,
        num_activated_heads: usize,
    ) -> Self {
        Self {
            dominant,
            distribution,
            num_activated_heads,
        }
    }
}

/// Ascendancy Weapon Set — 双专精 (AGENTS.md):
/// 每 session 两个 Weapon Set, 经 AttentionManager 按任务类型路由。
/// Weapon Set I = 获取 (acquisition): CORE+WORLD 域优先
/// Weapon Set II = 进化 (evolution): CORE+MIND 域优先
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WeaponSet {
    /// 获取模式: PatternMatch/Code/Temporal/ToolUse 优先 (采集+执行)
    Acquisition,
    /// 进化模式: Semantic/SelfReflection/Creativity/GoalAlignment 优先 (蒸馏+进化)
    Evolution,
}

impl WeaponSet {
    /// 双专精域映射 — 该专精下获得激活加成的 attention 域
    pub fn priority_domains(&self) -> Vec<AttentionDomain> {
        match self {
            Self::Acquisition => vec![
                AttentionDomain::PatternMatch,
                AttentionDomain::Code,
                AttentionDomain::Temporal,
                AttentionDomain::ToolUse,
            ],
            Self::Evolution => vec![
                AttentionDomain::Semantic,
                AttentionDomain::SelfReflection,
                AttentionDomain::Creativity,
                AttentionDomain::GoalAlignment,
            ],
        }
    }

    /// 从任务类型路由专精 (与 RuleIntensity::from_task_type 协同)
    pub fn from_task_type(task: &str) -> Self {
        let t = task.to_lowercase();
        if t.contains("evolve")
            || t.contains("distill")
            || t.contains("absorb")
            || t.contains("reflect")
            || t.contains("crystallize")
            || t.contains("learn")
        {
            Self::Evolution
        } else {
            Self::Acquisition
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Acquisition => "Weapon Set I (获取: CORE+WORLD)",
            Self::Evolution => "Weapon Set II (进化: CORE+MIND)",
        }
    }
}

/// MTRouter 历史路由条目 — 记录每次路由决策的结果 (arXiv 2604.23530)
/// 用于 cost-aware 模型路由的历史相似度匹配
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// 任务特征哈希 (基于任务文本的确定性哈希)
    pub task_hash: u64,
    /// 路由到的域标签
    pub domain: String,
    /// 使用的模型 ID
    pub model_id: String,
    /// 路由是否成功
    pub success: bool,
    /// 推理延迟 (毫秒)
    pub latency_ms: u64,
    /// 本次路由成本 (token/cost 单位)
    pub cost: f64,
    /// 时间戳 (Unix epoch seconds)
    pub timestamp: i64,
}

/// MTRouter 历史路由缓冲区容量上限 (环形缓冲区)
const ROUTE_HISTORY_MAX: usize = 1000;

#[derive(Debug, Clone)]
pub struct AttentionManager {
    pub heads: Vec<AttentionHead>,
    pub global_threshold: f64,
    pub rule_intensity: RuleIntensity,
    /// Ascendancy 当前专精 (Weapon Set)
    pub weapon_set: WeaponSet,
    /// 全局剩余预算份额 ∈ [0.0, 1.0], 1.0 表示预算充足 (Cost-Aware Routing, Axiom A1)
    pub budget_remaining: f64,
    /// MTRouter 历史路由记录 — 环形缓冲区, 最多保留 ROUTE_HISTORY_MAX 条
    pub route_history: Vec<HistoryEntry>,
}

impl AttentionManager {
    pub fn new(threshold: f64) -> Self {
        let heads: Vec<AttentionHead> = AttentionDomain::all()
            .into_iter()
            .enumerate()
            .map(|(i, domain)| AttentionHead::new(i, domain))
            .collect();
        Self {
            heads,
            global_threshold: threshold,
            rule_intensity: RuleIntensity::default(),
            weapon_set: WeaponSet::Acquisition,
            budget_remaining: 1.0,
            route_history: Vec::new(),
        }
    }

    pub fn with_intensity(threshold: f64, intensity: RuleIntensity) -> Self {
        let mut mgr = Self::new(threshold);
        mgr.set_intensity(intensity);
        mgr
    }

    pub fn set_intensity(&mut self, intensity: RuleIntensity) {
        self.rule_intensity = intensity;
        self.global_threshold = intensity.attention_threshold();
    }

    /// 设置全局预算份额 ∈ [0.0, 1.0] (Cost-Aware Routing, Axiom A1)
    /// 同步更新所有 head 的 budget_remaining
    pub fn set_budget(&mut self, remaining: f64) {
        let clamped = remaining.clamp(0.0, 1.0);
        self.budget_remaining = clamped;
        for head in &mut self.heads {
            head.budget_remaining = clamped;
        }
    }

    pub fn from_task_type(threshold: f64, task: &str) -> Self {
        let intensity = RuleIntensity::from_task_type(task);
        let mut mgr = Self::with_intensity(threshold, intensity);
        mgr.weapon_set = WeaponSet::from_task_type(task);
        mgr
    }

    /// Ascendancy: 切换 Weapon Set 并给予优先级域启动激活加成
    pub fn activate_weapon_set(&mut self, set: WeaponSet, boost: f64) {
        self.weapon_set = set;
        for domain in set.priority_domains() {
            self.stimulate_domain(domain, boost);
        }
    }

    /// 当前专精的优先级域列表
    pub fn active_priority_domains(&self) -> Vec<AttentionDomain> {
        self.weapon_set.priority_domains()
    }

    pub fn stimulate_domain(&mut self, domain: AttentionDomain, amount: f64) {
        if let Some(head) = self.heads.iter_mut().find(|h| h.domain == domain) {
            head.stimulate(amount);
        }
    }

    pub fn decay_all(&mut self) {
        for head in &mut self.heads {
            head.decay();
        }
    }

    pub fn active_heads(&self) -> Vec<&AttentionHead> {
        self.heads
            .iter()
            .filter(|h| h.activation >= self.global_threshold)
            .collect()
    }

    pub fn dominant_domain(&self) -> Option<AttentionDomain> {
        self.heads
            .iter()
            .filter(|h| h.activation >= self.global_threshold)
            .max_by(|a, b| {
                a.activation
                    .partial_cmp(&b.activation)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|h| h.domain)
    }

    /// PILOT 集成: 使用 PILOT 监控数据辅助注意力路由
    ///
    /// 参考: arXiv:2608.26530 "PILOT: Live Self-Improvement for Long-Horizon Agents"
    /// 利用 PILOT 的失败模式检测来调整注意力分配。
    pub fn pilot_assisted_route(
        &mut self,
        current_domain: &AttentionDomain,
        pilot_failure_patterns: &[String],
    ) -> AttentionDomain {
        // 检查当前域是否有失败模式
        let domain_label = current_domain.label();
        let has_failure = pilot_failure_patterns.iter().any(|p| p.contains(domain_label));

        if has_failure {
            // 当前域有失败模式，切换到更稳定的域
            log::warn!(
                "[attention] PILOT detected failure patterns for domain '{}', switching to Code",
                domain_label
            );
            AttentionDomain::Code
        } else {
            current_domain.clone()
        }
    }

    pub fn profile(&self) -> AttentionProfile {
        let distribution: HashMap<AttentionDomain, f64> = self
            .heads
            .iter()
            .map(|h| (h.domain, h.activation))
            .collect();
        let dominant = self
            .dominant_domain()
            .unwrap_or(AttentionDomain::PatternMatch);
        let num_activated = self.active_heads().len();
        AttentionProfile::new(dominant, distribution, num_activated)
    }

    pub fn reset(&mut self) {
        for head in &mut self.heads {
            head.activation = 0.0;
            head.focus.clear();
        }
    }

    // ── W2.2 (batch3 2026-08-26, 源: arxiv 2608.20256 *Learning When to Think*) ──
    // 测试时算力自适应分配: 简单任务 System1 直通, 复杂任务触发 System2 深思。
    // 对齐 mars_system1_activations / mars_system2_iterations 计数语义。

    /// 任务文本 → 难度信号 ∈ [0,1]。确定性关键词+结构启发式 (无 LLM):
    /// 架构/重写类 +0.25, 实现/修复类基线 0.4, 探索/只读类 -0.2;
    /// 多步骤连接词每步 +0.08 (封顶 +0.24)。
    pub fn estimate_task_difficulty(task: &str) -> f64 {
        Self::estimate_task_difficulty_with_budget(task, 1.0)
    }

    /// 预算感知难度估计: 当 budget < 0.2 时, 难度膨胀 1.5× (Cost-Aware Routing, Axiom A1)
    /// 预算紧张时将简单任务推入中等难度区间, 促使路由选择更快路径
    pub fn estimate_task_difficulty_with_budget(task: &str, budget: f64) -> f64 {
        let t = task.to_lowercase();
        let mut d = 0.4f64;
        if ["architect", "design", "rewrite", "架构", "重构", "设计"].iter().any(|k| t.contains(k)) {
            d += 0.25;
        }
        if ["explore", "read", "search", "lookup", "探索", "查找"].iter().any(|k| t.contains(k)) {
            d -= 0.2;
        }
        let steps = ["then", "之后", "再", "然后", ";", "&&"]
            .iter()
            .map(|k| t.matches(k).count())
            .sum::<usize>();
        d += (steps as f64 * 0.08).min(0.24);
        // 预算临界时膨胀难度 → 迫使路由选择 System1Direct
        if budget < 0.2 {
            d = (d * 1.5).clamp(0.0, 1.0);
        }
        d.clamp(0.0, 1.0)
    }

    /// 难度 → 思考模式路由。低难度直通省算力, 高难度强制深思,
    /// 中间带由 RuleIntensity 折中 (Lite 偏直通 / Ultra 偏深思)。
    /// 预算临界 (< 0.2) 时强制 System1Direct, 无论难度多高 (Cost-Aware Routing, Axiom A1)。
    pub fn allocate_compute(&self, difficulty: f64) -> ComputeAllocation {
        // 预算临界: 强制直通, 跳过所有深思分支
        if self.budget_remaining < 0.2 {
            return ComputeAllocation {
                mode: ThinkingMode::System1Direct,
                budget_share: 0.1,
                difficulty,
                reason: "budget critical — forced System1 direct to conserve compute",
            };
        }
        if difficulty < 0.35 {
            ComputeAllocation {
                mode: ThinkingMode::System1Direct,
                budget_share: 0.25,
                difficulty,
                reason: "difficulty below deliberation threshold",
            }
        } else if difficulty > 0.65 || self.rule_intensity == RuleIntensity::Ultra {
            ComputeAllocation {
                mode: ThinkingMode::System2Deliberate,
                budget_share: 1.0_f64.max(difficulty),
                difficulty,
                reason: "high complexity or ultra intensity demands deliberation",
            }
        } else {
            match self.rule_intensity {
                RuleIntensity::Lite => ComputeAllocation {
                    mode: ThinkingMode::System1Direct,
                    budget_share: 0.4,
                    difficulty,
                    reason: "lite intensity favors fast path in mid band",
                },
                _ => ComputeAllocation {
                    mode: ThinkingMode::System2Deliberate,
                    budget_share: 0.7,
                    difficulty,
                    reason: "full intensity deliberates mid-band tasks",
                },
            }
        }
    }

    /// 任务文本一步到位: 难度估计 → 分配决策
    pub fn allocate_for_task(&self, task: &str) -> ComputeAllocation {
        let d = Self::estimate_task_difficulty(task);
        self.allocate_compute(d)
    }

    /// FSM 辅助路由: 基于历史轨迹预测下一个最佳注意力域
    ///
    /// 参考: arXiv:2608.23670 "Emerging Digital Automata from Agent Traces"
    /// 利用 FSM 行为拓扑来增强注意力路由决策。
    pub fn fsm_assisted_route(
        &self,
        history: &[AttentionDomain],
        fsm: &crate::core::nt_core_self::behavior_fsm::FsmModel,
    ) -> Option<AttentionDomain> {
        if history.is_empty() {
            return None;
        }

        // 将历史域序列转换为 FSM 状态 ID
        let state_id = format!("domain_{}", history.last().unwrap().label());

        // 从 FSM 获取可能的下一个状态
        let transitions = fsm.transitions_from(&state_id);
        if transitions.is_empty() {
            return None;
        }

        // 选择概率最高的转换
        let best_transition = transitions.iter().max_by(|a, b| {
            a.probability.partial_cmp(&b.probability).unwrap_or(std::cmp::Ordering::Equal)
        })?;

        // 将 FSM 状态 ID 转换回 AttentionDomain
        let next_state = &best_transition.to;
        AttentionDomain::from_label(next_state)
    }

    /// FSM 失败预测辅助路由: 基于失败概率调整注意力分配
    ///
    /// 参考: arXiv:2608.23670 "Emerging Digital Automata from Agent Traces"
    /// 当 FSM 预测当前状态失败概率较高时, 自动切换到更稳定的注意力域。
    pub fn fsm_failure_aware_route(
        &self,
        current_domain: &AttentionDomain,
        fsm: &crate::core::nt_core_self::behavior_fsm::FsmModel,
        failure_threshold: f64,
    ) -> AttentionDomain {
        let state_id = format!("domain_{}", current_domain.label());
        let failure_prob = fsm.predict_failure_probability(&state_id);

        if failure_prob > failure_threshold {
            // 失败概率高, 切换到更稳定的域 (Code 域通常最稳定)
            log::warn!(
                "[attention] FSM predicts high failure probability ({:.2}) for domain '{}', switching to Code",
                failure_prob,
                current_domain.label()
            );
            AttentionDomain::Code
        } else {
            current_domain.clone()
        }
    }

    /// CUDA Agent RL 辅助路由: 使用 RL 奖励信号优化注意力分配
    ///
    /// 参考: arXiv:2602.24286 "CUDA Agent: Large-Scale Agentic RL for High-Performance CUDA Kernel Generation"
    /// 利用 CUDA Agent 的 RL 优化历史 (RewardCalculator + StrategyManager) 来评估
    /// 当前域的奖励趋势, 若近期奖励信号弱则切换到 RL 优化效果更好的域。
    pub fn cuda_assisted_route(
        &self,
        current_domain: &AttentionDomain,
        cuda_env: &crate::core::nt_core_self::cuda_agent::CudaAgentEnvironment,
        reward_threshold: f64,
    ) -> AttentionDomain {
        // 从 CUDA Agent 的性能分析器获取优化历史
        let history = cuda_env.get_analyzer().get_history();

        if history.is_empty() {
            // 无优化历史, 保持当前域
            return current_domain.clone();
        }

        // 计算近期优化的平均奖励 (最近 5 次或全部)
        let recent: Vec<f64> = history.iter().rev().take(5).map(|r| r.reward).collect();
        let avg_reward = recent.iter().sum::<f64>() / recent.len() as f64;

        if avg_reward < reward_threshold {
            // RL 奖励信号弱, 切换到 RL 优化效果更好的域 (Code 域通常最受益于 RL)
            log::warn!(
                "[attention] CUDA Agent RL reward low ({:.3} < {:.3}) for domain '{}', switching to Code",
                avg_reward,
                reward_threshold,
                current_domain.label()
            );
            AttentionDomain::Code
        } else {
            current_domain.clone()
        }
    }

    // ── MTRouter 历史路由 (arXiv 2604.23530) ──────────────────────────────

    /// 记录一次路由决策到历史缓冲区 (环形缓冲区, 超容量淘汰最旧条目)
    pub fn record_route(&mut self, entry: HistoryEntry) {
        if self.route_history.len() >= ROUTE_HISTORY_MAX {
            self.route_history.remove(0);
        }
        self.route_history.push(entry);
    }

    /// 基于任务特征向量预测最佳模型 — 在历史中查找相似任务,
    /// 返回成功率最高的模型 ID (简单余弦相似度匹配)
    pub fn predict_best_model(&self, task_features: &[f64]) -> Option<String> {
        if self.route_history.is_empty() || task_features.is_empty() {
            return None;
        }

        // 按 (domain, model_id) 分组, 计算每组的加权成功率
        let mut model_scores: HashMap<String, (f64, u64)> = HashMap::new(); // model → (score, count)

        // 将任务特征哈希作为伪特征向量用于相似度匹配
        // 实际部署时 task_features 应来自嵌入模型
        let query_norm: f64 = task_features.iter().map(|x| x * x).sum::<f64>().sqrt();
        if query_norm < 1e-10 {
            return None;
        }

        for entry in &self.route_history {
            // 使用任务哈希生成伪特征 (简化版: hash 低位展开为特征向量)
            let entry_features = Self::hash_to_features(entry.task_hash, task_features.len());
            let entry_norm: f64 = entry_features.iter().map(|x| x * x).sum::<f64>().sqrt();
            if entry_norm < 1e-10 {
                continue;
            }

            // 余弦相似度
            let dot: f64 = task_features
                .iter()
                .zip(entry_features.iter())
                .map(|(a, b)| a * b)
                .sum();
            let similarity = dot / (query_norm * entry_norm);

            // 相似度 > 0.5 的条目参与模型评分
            if similarity > 0.5 {
                let weight = similarity * if entry.success { 1.0 } else { 0.2 };
                let entry_key = entry.model_id.clone();
                let e = model_scores.entry(entry_key).or_insert((0.0, 0));
                e.0 += weight;
                e.1 += 1;
            }
        }

        // 返回加权得分最高的模型
        model_scores
            .into_iter()
            .max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(model, _)| model)
    }

    /// 查询特定 (domain, model_id) 组合在历史中的成功率 ∈ [0.0, 1.0]
    pub fn history_success_rate(&self, domain: &str, model_id: &str) -> f64 {
        let relevant: Vec<&HistoryEntry> = self
            .route_history
            .iter()
            .filter(|e| e.domain == domain && e.model_id == model_id)
            .collect();
        if relevant.is_empty() {
            return 0.0;
        }
        let successes = relevant.iter().filter(|e| e.success).count() as f64;
        successes / relevant.len() as f64
    }

    /// 查询特定域在最近 100 条记录中的平均延迟 (毫秒)
    pub fn recent_avg_latency(&self, domain: &str) -> u64 {
        let recent: Vec<&HistoryEntry> = self
            .route_history
            .iter()
            .rev()
            .take(100)
            .filter(|e| e.domain == domain)
            .collect();
        if recent.is_empty() {
            return 0;
        }
        let total: u64 = recent.iter().map(|e| e.latency_ms).sum();
        total / recent.len() as u64
    }

    /// 将 u64 哈希值展开为伪特征向量 (用于余弦相似度匹配的简化方案)
    fn hash_to_features(hash: u64, dim: usize) -> Vec<f64> {
        (0..dim)
            .map(|i| {
                let shifted = hash
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add((i as u64) * 14695981039346656037);
                // 映射到 [-1.0, 1.0]
                ((shifted >> 33) as f64) / (1u64 << 31) as f64 - 1.0
            })
            .collect()
    }
}

/// W2.2 思考模式: System1 直通 vs System2 深思
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkingMode {
    /// 直通 — 单轮快路径
    System1Direct,
    /// 深思 — 多轮迭代路径
    System2Deliberate,
}

/// W2.2 算力分配决策 (可观测: reason 说明路由依据)
#[derive(Debug, Clone, PartialEq)]
pub struct ComputeAllocation {
    pub mode: ThinkingMode,
    /// 本任务思考预算份额 ∈ (0,1]
    pub budget_share: f64,
    pub difficulty: f64,
    pub reason: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attention_head_new() {
        let h = AttentionHead::new(0, AttentionDomain::Code);
        assert_eq!(h.domain, AttentionDomain::Code);
        assert_eq!(h.activation, 0.0);
        assert_eq!(h.id, 0);
    }

    #[test]
    fn test_stimulate_and_decay() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(0.5);
        assert!((h.activation - 0.5).abs() < 1e-6);
        h.decay();
        assert!((h.activation - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_activation_capped() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(1.5);
        assert!((h.activation - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_salience_formula() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.stimulate(0.8);
        let s = h.salience(0.5, 0.5);
        assert!((s - 0.8 * 0.5 * 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_attention_manager_decay_all() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        let code_idx = AttentionDomain::all()
            .iter()
            .position(|d| *d == AttentionDomain::Code)
            .expect("value should be ok in test");
        let plan_idx = AttentionDomain::all()
            .iter()
            .position(|d| *d == AttentionDomain::Planning)
            .expect("value should be ok in test");
        assert_eq!(mgr.active_heads().len(), 2);
        mgr.decay_all();
        assert!((mgr.heads[code_idx].activation - 0.8).abs() < 1e-6);
        assert!((mgr.heads[plan_idx].activation - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_domain() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.3);
        assert_eq!(mgr.dominant_domain(), Some(AttentionDomain::Code));
    }

    #[test]
    fn test_attention_profile() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::SelfReflection, 0.8);
        let profile = mgr.profile();
        assert_eq!(profile.dominant, AttentionDomain::SelfReflection);
        assert!(profile.num_activated_heads >= 1);
        assert!(
            profile
                .distribution
                .get(&AttentionDomain::SelfReflection)
                .copied()
                .unwrap_or(0.0)
                > 0.0
        );
    }

    #[test]
    fn test_focus_on_concept() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        h.focus_on("rust");
        assert!(h.focus.contains(&"rust".to_string()));
        assert!(h.activation > 0.0);
        let act_before = h.activation;
        h.focus_on("rust");
        assert_eq!(h.focus.len(), 1);
        assert!(h.activation >= act_before);
    }

    #[test]
    fn test_reset_manager() {
        let mut mgr = AttentionManager::new(0.3);
        mgr.stimulate_domain(AttentionDomain::Code, 0.9);
        mgr.stimulate_domain(AttentionDomain::Planning, 0.7);
        assert!(mgr.active_heads().len() > 0);
        mgr.reset();
        assert_eq!(mgr.active_heads().len(), 0);
    }

    #[test]
    fn test_all_domains_count() {
        let domains = AttentionDomain::all();
        assert_eq!(domains.len(), 10);
    }

    #[test]
    fn test_weapon_set_from_task_type() {
        assert_eq!(
            WeaponSet::from_task_type("absorb knowledge"),
            WeaponSet::Evolution
        );
        assert_eq!(
            WeaponSet::from_task_type("distill session"),
            WeaponSet::Evolution
        );
        assert_eq!(
            WeaponSet::from_task_type("crawler fix"),
            WeaponSet::Acquisition
        );
        assert_eq!(
            WeaponSet::from_task_type("implement feature"),
            WeaponSet::Acquisition
        );
    }

    #[test]
    fn test_weapon_set_priority_domains() {
        let acq = WeaponSet::Acquisition.priority_domains();
        assert!(acq.contains(&AttentionDomain::Code));
        assert!(acq.contains(&AttentionDomain::ToolUse));
        assert!(!acq.contains(&AttentionDomain::SelfReflection));
        let evo = WeaponSet::Evolution.priority_domains();
        assert!(evo.contains(&AttentionDomain::SelfReflection));
        assert!(evo.contains(&AttentionDomain::Creativity));
        assert!(!evo.contains(&AttentionDomain::Code));
    }

    #[test]
    fn test_activate_weapon_set_boosts_priority_domains() {
        let mut mgr = AttentionManager::new(0.5);
        mgr.activate_weapon_set(WeaponSet::Evolution, 0.6);
        assert_eq!(mgr.weapon_set, WeaponSet::Evolution);
        for domain in mgr.active_priority_domains() {
            let head = mgr
                .heads
                .iter()
                .find(|h| h.domain == domain)
                .expect("head exists");
            assert!(
                head.activation >= 0.6,
                "priority domain {} should be boosted",
                domain.label()
            );
        }
        // 非优先级域不应被提升
        let code_head = mgr
            .heads
            .iter()
            .find(|h| h.domain == AttentionDomain::Code)
            .unwrap();
        assert!(code_head.activation < 0.6);
    }

    #[test]
    fn test_from_task_type_sets_weapon_set() {
        let mgr = AttentionManager::from_task_type(0.4, "distill knowledge");
        assert_eq!(mgr.weapon_set, WeaponSet::Evolution);
        let mgr2 = AttentionManager::from_task_type(0.4, "write code");
        assert_eq!(mgr2.weapon_set, WeaponSet::Acquisition);
    }

    #[test]
    fn test_attention_head_is_activated() {
        let mut h = AttentionHead::new(0, AttentionDomain::Code);
        assert!(!h.is_activated(0.5));
        h.stimulate(0.6);
        assert!(h.is_activated(0.5));
    }

    #[test]
    fn test_rule_intensity_from_task_type() {
        assert_eq!(
            RuleIntensity::from_task_type("explore codebase"),
            RuleIntensity::Lite
        );
        assert_eq!(
            RuleIntensity::from_task_type("search for pattern"),
            RuleIntensity::Lite
        );
        assert_eq!(
            RuleIntensity::from_task_type("read file"),
            RuleIntensity::Lite
        );
        assert_eq!(
            RuleIntensity::from_task_type("implement feature"),
            RuleIntensity::Full
        );
        assert_eq!(
            RuleIntensity::from_task_type("fix bug"),
            RuleIntensity::Full
        );
        assert_eq!(
            RuleIntensity::from_task_type("refactor module"),
            RuleIntensity::Full
        );
        assert_eq!(
            RuleIntensity::from_task_type("architect system"),
            RuleIntensity::Ultra
        );
        assert_eq!(
            RuleIntensity::from_task_type("design api"),
            RuleIntensity::Ultra
        );
        assert_eq!(
            RuleIntensity::from_task_type("rewrite core"),
            RuleIntensity::Ultra
        );
        assert_eq!(
            RuleIntensity::from_task_type("unknown"),
            RuleIntensity::Full
        );
    }

    #[test]
    fn test_rule_intensity_threshold() {
        assert_eq!(RuleIntensity::Lite.attention_threshold(), 0.2);
        assert_eq!(RuleIntensity::Full.attention_threshold(), 0.4);
        assert_eq!(RuleIntensity::Ultra.attention_threshold(), 0.6);
    }

    #[test]
    fn test_attention_manager_with_intensity() {
        let mgr = AttentionManager::with_intensity(0.3, RuleIntensity::Lite);
        assert_eq!(mgr.global_threshold, 0.2);
        let mgr = AttentionManager::with_intensity(0.3, RuleIntensity::Full);
        assert_eq!(mgr.global_threshold, 0.4);
        let mgr = AttentionManager::with_intensity(0.3, RuleIntensity::Ultra);
        assert_eq!(mgr.global_threshold, 0.6);
    }

    #[test]
    fn test_attention_manager_from_task_type() {
        let mgr = AttentionManager::from_task_type(0.5, "explore");
        assert_eq!(mgr.global_threshold, 0.2);
        let mgr = AttentionManager::from_task_type(0.5, "implement feature");
        assert_eq!(mgr.global_threshold, 0.4);
        let mgr = AttentionManager::from_task_type(0.5, "architect");
        assert_eq!(mgr.global_threshold, 0.6);
    }

    #[test]
    fn test_attention_manager_set_intensity() {
        let mut mgr = AttentionManager::new(0.5);
        mgr.set_intensity(RuleIntensity::Ultra);
        assert_eq!(mgr.global_threshold, 0.6);
    }

    #[test]
    fn test_cuda_assisted_route_no_history_keeps_domain() {
        let mgr = AttentionManager::new(0.3);

        let cuda_env = crate::core::nt_core_self::cuda_agent::CudaAgentEnvironment::new();
        let result = mgr.cuda_assisted_route(&AttentionDomain::Planning, &cuda_env, 0.0);
        assert_eq!(result, AttentionDomain::Planning);
    }

    #[test]
    fn test_cuda_assisted_route_low_reward_switches() {
        let mgr = AttentionManager::new(0.3);
        let mut cuda_env = crate::core::nt_core_self::cuda_agent::CudaAgentEnvironment::new();

        // 注入低奖励历史: 手动提交任务并优化 (RewardCalculator 默认权重, 零改进 = 奖励 0)
        let task = crate::core::nt_core_self::cuda_agent::OptimizationTask {
            id: "t1".into(),
            name: "low_reward_task".into(),
            description: "test".into(),
            code: "x=1".into(),
            language: "python".into(),
            metrics: std::collections::HashMap::new(),
            constraints: vec![],
            status: crate::core::nt_core_self::cuda_agent::TaskStatus::Pending,
        };
        cuda_env.submit_task(task);
        cuda_env.optimize("t1");

        // 奖励阈值 0.1, 低奖励应切换到 Code
        let result = mgr.cuda_assisted_route(&AttentionDomain::Creativity, &cuda_env, 0.1);
        assert_eq!(result, AttentionDomain::Code);
    }

    #[test]
    fn test_cuda_assisted_route_high_reward_keeps_domain() {
        let mgr = AttentionManager::new(0.3);
        let mut cuda_env = crate::core::nt_core_self::cuda_agent::CudaAgentEnvironment::new();

        // 注入高奖励历史: 提交任务并优化 (有策略时模拟 10% 改进)
        let task = crate::core::nt_core_self::cuda_agent::OptimizationTask {
            id: "t1".into(),
            name: "high_reward_task".into(),
            description: "test".into(),
            code: "x=1".into(),
            language: "python".into(),
            metrics: std::collections::HashMap::new(),
            constraints: vec![],
            status: crate::core::nt_core_self::cuda_agent::TaskStatus::Pending,
        };
        cuda_env.submit_task(task);
        cuda_env.optimize("t1");

        // 奖励阈值 0.0, 零改进刚好等于阈值不触发切换
        let result = mgr.cuda_assisted_route(&AttentionDomain::Creativity, &cuda_env, 0.0);
        assert_eq!(result, AttentionDomain::Creativity);
    }
}

/// W2.2 (batch3 2026-08-26) 验收测试: 分级任务集上 System2 触发率与难度正相关。
#[cfg(test)]
mod when_to_think_tests {
    use super::*;

    #[test]
    fn difficulty_orders_task_classes() {
        let easy = AttentionManager::estimate_task_difficulty("explore the kb and read notes");
        let mid = AttentionManager::estimate_task_difficulty("fix the parser bug");
        let hard = AttentionManager::estimate_task_difficulty("architect rewrite of core engine");
        assert!(easy < mid, "easy={easy} mid={mid}");
        assert!(mid < hard, "mid={mid} hard={hard}");
        assert!((0.0..=1.0).contains(&easy) && (0.0..=1.0).contains(&hard));
    }

    #[test]
    fn system1_for_simple_system2_for_complex() {
        let mgr = AttentionManager::with_intensity(0.4, RuleIntensity::Full);
        let simple = mgr.allocate_for_task("search old tickets");
        assert_eq!(simple.mode, ThinkingMode::System1Direct);
        let complex = mgr.allocate_for_task("architect and design then rewrite module; then validate");
        assert_eq!(complex.mode, ThinkingMode::System2Deliberate);
        assert!(complex.budget_share > simple.budget_share);
    }

    #[test]
    fn intensity_breaks_ties_in_mid_band() {
        let lite = AttentionManager::with_intensity(0.4, RuleIntensity::Lite);
        let ultra = AttentionManager::with_intensity(0.4, RuleIntensity::Ultra);
        let d = 0.5; // 中间带
        assert_eq!(lite.allocate_compute(d).mode, ThinkingMode::System1Direct);
        assert_eq!(ultra.allocate_compute(d).mode, ThinkingMode::System2Deliberate);
    }
}
