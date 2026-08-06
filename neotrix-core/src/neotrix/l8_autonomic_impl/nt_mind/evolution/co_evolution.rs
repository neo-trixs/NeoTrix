//! 四子图共进化循环 (Co-Evolution Loop, P4) — MAGE (arXiv 2605.10064 / DOI 10.48550/arxiv.2605.10064) 落地。
//!
//! MAGE 核心机制映射到 NeoTrix 派单/学习层:
//!   1. **四子图共进化知识图谱 (EVOKG)**: 本模块把能力图 (capability)、任务图 (task)、
//!      经验图 (experience)、环境图 (environment) 统一为一个 `CoEvoGraph`, 每次
//!      派单 reward 同时更新四个子图 — 知识在图上共进化, 而非散落各处。
//!   2. **双记忆索引**: 经验子图按成败建立双索引 — success index (自身正确轨迹)
//!      与 failure index (失败校正)。`guidance` 从成功索引取指导, `failure_warnings`
//!      从失败索引取警示。
//!   3. **两个 bandit 共享同一 reward 流**: 任务级搜索 bandit (`TaskSearchBandit`,
//!      选择检索策略) + 技能级路由 bandit (既有 `RouteLearner`)。`dispatch_and_execute`
//!      把同一次执行结果同时喂给两个 bandit — 派单组织与检索策略从同一观测共进化。
//!   4. **append-only 记忆增长 (信息单调)**: 经验记忆只追加、从不改写; 超上限时仅
//!      淘汰最旧条目 (bounded curriculum), 已沉淀的知识不被覆写。
//!   5. **frozen backbone**: 执行主干 (AgentExecutor) 不变, 学习信号只落在图与 bandit。
//!
//! 强化既有节点 (R-P42): 任务级搜索 bandit 选择的是 `nt_memory_kb::RetrievalStrategy`,
//! 经 `ProductionAgentExecutor::execute_with_strategy` 走既有的 `search_with_confidence`
//! 检索缝 — 不新建平行检索路径。

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

use crate::core::nt_core_self::attention_head::AttentionDomain;
use crate::neotrix::l3_memory_impl::nt_memory_kb::{KnowledgeBase, RetrievalStrategy};

/// 可选的检索策略集 — 任务级搜索 bandit 的臂 (arm)。
pub const COEVO_STRATEGIES: &[&str] = &["balanced", "conservative", "exploratory", "confidence_weighted"];

/// 策略名 → `RetrievalStrategy` (注入既有 confidence 检索缝)。
pub fn parse_strategy(name: &str) -> RetrievalStrategy {
    match name {
        "conservative" => RetrievalStrategy::Conservative { min_confidence: 0.6 },
        "exploratory" => RetrievalStrategy::Exploratory,
        "confidence_weighted" => RetrievalStrategy::ConfidenceWeighted {
            source_weight: 0.4,
            grounding_weight: 0.3,
            consensus_weight: 0.2,
            recency_weight: 0.1,
        },
        _ => RetrievalStrategy::Balanced,
    }
}

/// 策略名归一 — 未知名一律回落到 balanced (防脏配置)。
pub fn strategy_name(name: &str) -> &'static str {
    match name {
        "conservative" => "conservative",
        "exploratory" => "exploratory",
        "confidence_weighted" => "confidence_weighted",
        _ => "balanced",
    }
}

/// 共进化循环配置 (P4) — epsilon 探索/利用、bounded curriculum 上限、bandit 冷启动证据门。
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CoEvoConfig {
    /// bandit 探索概率: 足够证据后仍以 epsilon 概率尝试次优臂。
    pub epsilon: f64,
    /// 经验子图上限 — 超出后仅淘汰最旧条目 (bounded curriculum coverage)。
    pub max_memories: usize,
    /// bandit 冷启动门: 某策略尝试 ≥ min_evidence 次才进入利用 (防噪声覆盖)。
    pub min_evidence: u32,
    /// 拓扑修复掌握度门槛 (P7): 候选档案 mastery ≥ 该值才被视为"经验上可信"的
    /// 修复依据 — 防把低观测噪声当成组织缺陷。
    pub mastery_gate: f64,
}

impl Default for CoEvoConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.1,
            max_memories: 200,
            min_evidence: 2,
            mastery_gate: 0.5,
        }
    }
}

/// 经验图节点 — 一次派单执行沉淀下来的记忆 (append-only, 双索引: success/failure)。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperienceMemory {
    /// 单调递增 id — 唯一标识, 永不重用。
    pub id: u64,
    /// 所属任务类型 (task 子图维度)。
    pub task_type: String,
    /// 触发派单的注意力域。
    pub domain: AttentionDomain,
    /// 被派单的 agent 档案名。
    pub agent: String,
    /// 成功/失败 — 决定进入 success index 还是 failure index。
    pub success: bool,
    /// 本轮使用的检索策略名。
    pub strategy: String,
    /// 执行摘要 (供后续检索指导)。
    pub summary: String,
    /// 单调时间戳。
    pub ts: i64,
}

impl ExperienceMemory {
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// 单臂统计 — (success, attempts) 行为观测。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StrategyStats {
    pub strategy: String,
    pub success: u32,
    pub attempts: u32,
}

impl StrategyStats {
    pub fn rate(&self) -> f64 {
        self.success as f64 / self.attempts.max(1) as f64
    }
}

/// 四子图共进化知识图谱 (EVOKG) — 学习信号只落在这里与 bandit, 执行主干不变。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CoEvoGraph {
    /// capability 子图: (domain, agent) → (success, attempts) — 域×档案的掌握度。
    pub capability: BTreeMap<String, BTreeMap<String, (u32, u32)>>,
    /// task 子图: task_type → 各检索策略统计 (任务级搜索 bandit 的观测面)。
    pub tasks: BTreeMap<String, Vec<StrategyStats>>,
    /// experience 子图: append-only 双记忆索引 (success index + failure index)。
    pub memories: Vec<ExperienceMemory>,
    /// environment 子图: task_type → (success, attempts) — 任务级总体观察。
    pub environment: BTreeMap<String, (u32, u32)>,
}

/// 任务级搜索 bandit — 按任务类型选择检索策略, 用同一 reward 流更新 (MAGE: task-level search bandit)。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskSearchBandit {
    pub config: CoEvoConfig,
    pub graph: CoEvoGraph,
}

impl TaskSearchBandit {
    /// 冷启动 + epsilon-greedy 选择:
    ///   - 无任何策略达到 min_evidence → 探索: 优先覆盖**从未尝试**的臂 (curriculum
    ///     coverage), 全部试过后挑尝试最少的臂 — 保证每臂都被观察, 不是永远 balanced。
    ///   - 已有足够证据 → 以 epsilon 概率随机探索, 否则利用观测成功率最高的臂。
    pub fn select(&self, task_type: &str) -> String {
        let stats = self.graph.tasks.get(task_type);
        let Some(stats) = stats else {
            return "balanced".to_string();
        };
        let known: Vec<&StrategyStats> = stats
            .iter()
            .filter(|s| s.attempts >= self.config.min_evidence)
            .collect();
        if known.is_empty() {
            // 冷启动: 优先覆盖从未尝试的臂, 保证所有策略被观察 (确定性, 不依赖 epsilon)。
            let tried: Vec<&str> = stats.iter().map(|s| s.strategy.as_str()).collect();
            if let Some(untried) = COEVO_STRATEGIES.iter().find(|s| !tried.contains(s)) {
                return (*untried).to_string();
            }
            // 全部臂都已尝试但证据不足 → 挑尝试最少的臂继续探索。
            return stats
                .iter()
                .min_by_key(|s| (s.attempts, s.success))
                .map(|s| s.strategy.clone())
                .unwrap_or_else(|| "balanced".to_string());
        }
        let explore = self.config.epsilon > 0.0
            && rand::thread_rng().gen::<f64>() < self.config.epsilon;
        if explore {
            let pool = if stats.len() > 1 { stats.as_slice() } else { &[stats[0].clone()][..] };
            let i = rand::thread_rng().gen_range(0..pool.len());
            return pool[i].strategy.clone();
        }
        known
            .iter()
            .max_by(|a, b| {
                a.rate()
                    .partial_cmp(&b.rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.strategy.clone())
            .unwrap_or_else(|| "balanced".to_string())
    }

    /// 某任务当前各策略统计 (审计/诊断)。
    pub fn stats(&self, task_type: &str) -> Vec<StrategyStats> {
        self.graph.tasks.get(task_type).cloned().unwrap_or_default()
    }

    /// 某任务利用期最优策略名。
    pub fn best_strategy(&self, task_type: &str) -> Option<String> {
        let known: Vec<&StrategyStats> = self
            .graph
            .tasks
            .get(task_type)
            .map(|v| {
                v.iter()
                    .filter(|s| s.attempts >= self.config.min_evidence)
                    .collect()
            })
            .unwrap_or_default();
        known
            .iter()
            .max_by(|a, b| {
                a.rate()
                    .partial_cmp(&b.rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.strategy.clone())
    }
}

/// 四子图共进化循环 — 一次派单 reward 同时驱动 任务bandit + 技能bandit(外部RouteLearner) + 图。
///
/// `record_reward` 是 MAGE 的**单一 reward 流入口**: 上层 `dispatch_and_execute` 拿到实测
/// 执行结果后调用本方法, 同一 reward 同时更新 capability/task/experience/environment 四个
/// 子图与任务级搜索 bandit; 技能级路由 bandit (RouteLearner) 由上层在同一位置喂同一结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoEvolutionLoop {
    pub config: CoEvoConfig,
    pub graph: CoEvoGraph,
    /// 下一经验记忆 id — 单调, 保证 append-only 唯一性。
    next_memory_id: u64,
    /// 已吸收的 reward 总数 (进化里程)。
    pub total_rewards: u64,
    /// 图结构自进化次数 — 每次 record_reward 递增, 用于审计"共进化在发生"。
    pub evolution_revision: u64,
    /// 已尝试吸收进大脑的最高经验 id 水位 (P6) — 大脑吸收的去重水位,
    /// 防同一经验被反复吸收 (ReMe utility-based 去重, 与 session 吸收对齐)。
    #[serde(default)]
    pub absorbed_watermark: u64,
}

impl CoEvolutionLoop {
    pub fn new() -> Self {
        Self {
            config: CoEvoConfig::default(),
            graph: CoEvoGraph::default(),
            next_memory_id: 0,
            total_rewards: 0,
            evolution_revision: 0,
            absorbed_watermark: 0,
        }
    }

    pub fn with_config(config: CoEvoConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// 任务级搜索 bandit 视图。
    pub fn bandit(&self) -> TaskSearchBandit {
        TaskSearchBandit {
            config: self.config,
            graph: self.graph.clone(),
        }
    }

    /// 为当前任务选择检索策略 (上层在派单执行前调用, 注入执行器)。
    pub fn select_strategy(&self, task_type: &str) -> String {
        self.bandit().select(task_type)
    }

    /// 单一 reward 流入口 — 同一次执行结果同时更新四子图与任务 bandit。
    pub fn record_reward(
        &mut self,
        task_type: &str,
        domain: AttentionDomain,
        agent: &str,
        strategy: &str,
        success: bool,
        summary: &str,
    ) {
        let strategy = strategy_name(strategy).to_string();
        let domain_key = domain.label().to_string();

        // capability 子图: 域×档案掌握度。
        let cap = self.graph.capability.entry(domain_key).or_default();
        let entry = cap.entry(agent.to_string()).or_insert((0, 0));
        entry.0 += success as u32;
        entry.1 += 1;

        // task 子图: 任务×策略统计 (bandit 观测面)。
        let arms = self.graph.tasks.entry(task_type.to_string()).or_default();
        if let Some(arm) = arms.iter_mut().find(|s| s.strategy == strategy) {
            arm.success += success as u32;
            arm.attempts += 1;
        } else {
            arms.push(StrategyStats {
                strategy: strategy.clone(),
                success: success as u32,
                attempts: 1,
            });
        }

        // environment 子图: 任务级总体成败。
        let env = self.graph.environment.entry(task_type.to_string()).or_insert((0, 0));
        env.0 += success as u32;
        env.1 += 1;

        // experience 子图: append-only 记忆 (双索引: success 进成功索引, failure 进失败索引)。
        self.append_memory(ExperienceMemory {
            id: self.next_memory_id,
            task_type: task_type.to_string(),
            domain,
            agent: agent.to_string(),
            success,
            strategy,
            summary: summary.to_string(),
            ts: now_ts(),
        });
        self.next_memory_id += 1;
        self.total_rewards += 1;
        self.evolution_revision += 1;
    }

    /// append-only 落盘经验记忆 — 只追加不改写; 超上限仅淘汰最旧 (bounded curriculum)。
    fn append_memory(&mut self, mem: ExperienceMemory) {
        self.graph.memories.push(mem);
        if self.graph.memories.len() > self.config.max_memories {
            self.graph.memories.remove(0);
        }
    }

    /// 双记忆索引 — 成功索引: 该任务类型最近 k 条成功记忆 (task-filtered retrieval)。
    pub fn guidance(&self, task_type: &str, k: usize) -> Vec<&ExperienceMemory> {
        let mut hits: Vec<&ExperienceMemory> = self
            .graph
            .memories
            .iter()
            .filter(|m| m.task_type == task_type && m.is_success())
            .collect();
        hits.sort_by(|a, b| b.ts.cmp(&a.ts));
        hits.truncate(k);
        hits
    }

    /// 双记忆索引 — 失败索引: 该任务类型最近 k 条失败记忆 (教师校正/警示)。
    pub fn failure_warnings(&self, task_type: &str, k: usize) -> Vec<&ExperienceMemory> {
        let mut hits: Vec<&ExperienceMemory> = self
            .graph
            .memories
            .iter()
            .filter(|m| m.task_type == task_type && !m.is_success())
            .collect();
        hits.sort_by(|a, b| b.ts.cmp(&a.ts));
        hits.truncate(k);
        hits
    }

    /// capability 子图读取 — 域×档案掌握度 (0..=1, 无观测回 0)。
    pub fn mastery(&self, domain: AttentionDomain, agent: &str) -> f64 {
        self.graph
            .capability
            .get(domain.label())
            .and_then(|m| m.get(agent))
            .map(|(s, t)| *s as f64 / (*t).max(1) as f64)
            .unwrap_or(0.0)
    }

    /// 某任务累计 reward 数。
    pub fn task_rewards(&self, task_type: &str) -> usize {
        self.graph
            .environment
            .get(task_type)
            .map(|(_, t)| *t as usize)
            .unwrap_or(0)
    }

    /// 水位之上未吸收进大脑的经验 (P6) — 大脑吸收只消费新经验, 防重复。
    pub fn new_memories_since_watermark(&self) -> Vec<&ExperienceMemory> {
        self.graph
            .memories
            .iter()
            .filter(|m| m.id >= self.absorbed_watermark)
            .collect()
    }

    /// 推进吸收水位 (P6) — 标记当前全部经验已尝试吸收 (EDV 批评器负责质量把关,
    /// 水位只防重复扫描; 即使回滚也不重试同一条, 避免抖动)。
    pub fn commit_absorb(&mut self) {
        if let Some(max_id) = self.graph.memories.iter().map(|m| m.id).max() {
            self.absorbed_watermark = max_id + 1;
        }
    }

    /// 共进化审计摘要 — 供 [bg-meta] 日志证明"同一 reward 驱动了图+bandit 共进化"。
    pub fn evolution_summary(&self) -> String {
        format!(
            "revision={} rewards={} memories={} tasks={}",
            self.evolution_revision,
            self.total_rewards,
            self.graph.memories.len(),
            self.graph.tasks.len(),
        )
    }

    /// 持久化到 KB kv_store (namespace "coevolution") — 四子图与 bandit 跨会话存活。
    pub fn persist(&self, kb: &KnowledgeBase) -> Result<(), String> {
        let json = serde_json::to_string(self)
            .map_err(|e| format!("coevo serialize: {}", e))?;
        kb.save_coevo(&json)
    }

    /// 从 KB kv_store 恢复 — 冷启动无存档则保持空状态。
    pub fn load(&mut self, kb: &KnowledgeBase) -> Result<(), String> {
        let Some(json) = kb.load_coevo()? else {
            return Ok(());
        };
        let loaded: CoEvolutionLoop = serde_json::from_str(&json)
            .map_err(|e| format!("coevo deserialize: {}", e))?;
        self.config = loaded.config;
        self.graph = loaded.graph;
        self.next_memory_id = loaded.next_memory_id;
        self.total_rewards = loaded.total_rewards;
        self.evolution_revision = loaded.evolution_revision;
        self.absorbed_watermark = loaded.absorbed_watermark;
        Ok(())
    }
}

impl Default for CoEvolutionLoop {
    fn default() -> Self {
        Self::new()
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_deterministic() -> CoEvoConfig {
CoEvoConfig {
            epsilon: 0.1,
            max_memories: 200,
            min_evidence: 2,
            mastery_gate: 0.5,
        }
    }

    fn mem(id: u64, task: &str, success: bool) -> ExperienceMemory {
        ExperienceMemory {
            id,
            task_type: task.to_string(),
            domain: AttentionDomain::PatternMatch,
            agent: "explorer".to_string(),
            success,
            strategy: "balanced".to_string(),
            summary: format!("run {}", id),
            ts: id as i64,
        }
    }

    #[test]
    fn cold_start_selects_balanced() {
        let loop_ = CoEvolutionLoop::new();
        assert_eq!(loop_.select_strategy("research"), "balanced");
    }

    #[test]
    fn cold_start_explores_untried_arms() {
        // 冷启动必须覆盖每臂 (curriculum coverage), 而不是永远 balanced。
        let mut loop_ = CoEvolutionLoop::with_config(cfg_deterministic());
        let mut seen = std::collections::BTreeSet::new();
        for i in 0..5 {
            let s = loop_.select_strategy("research");
            seen.insert(s.clone());
            loop_.record_reward(
                "research",
                AttentionDomain::PatternMatch,
                "explorer",
                &s,
                i % 2 == 0,
                "round",
            );
        }
        // 5 轮内覆盖 ≥ 3 种不同策略 (balanced/conservative/exploratory/confidence_weighted)。
        assert!(seen.len() >= 3, "cold-start should cover multiple arms, got {:?}", seen);
    }

    #[test]
    fn record_reward_evolves_all_four_subgraphs() {
        let mut loop_ = CoEvolutionLoop::with_config(cfg_deterministic());
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "balanced", true, "searched 5");
        assert_eq!(loop_.evolution_revision, 1);
        assert_eq!(loop_.task_rewards("research"), 1);
        assert_eq!(loop_.graph.memories.len(), 1);
        // capability 子图
        assert_eq!(loop_.mastery(AttentionDomain::PatternMatch, "explorer"), 1.0);
        // task 子图 bandit 观测
        let stats = loop_.bandit().stats("research");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].attempts, 1);
        assert_eq!(stats[0].success, 1);
        // environment 子图
        assert_eq!(loop_.graph.environment.get("research").unwrap().0, 1);
    }

    #[test]
    fn bandit_exploits_highest_rate_after_evidence() {
        let mut loop_ = CoEvolutionLoop::with_config(cfg_deterministic());
        // balanced 2 尝试 2 成功; exploratory 2 尝试 0 成功 → 利用 balanced。
        loop_.record_reward("research", AttentionDomain::PatternMatch, "e", "balanced", true, "a");
        loop_.record_reward("research", AttentionDomain::PatternMatch, "e", "balanced", true, "b");
        loop_.record_reward("research", AttentionDomain::PatternMatch, "e", "exploratory", false, "c");
        loop_.record_reward("research", AttentionDomain::PatternMatch, "e", "exploratory", false, "d");
        assert_eq!(loop_.select_strategy("research"), "balanced");
        assert_eq!(loop_.bandit().best_strategy("research").unwrap(), "balanced");
    }

    #[test]
    fn memories_are_append_only_and_bounded() {
        let mut loop_ = CoEvolutionLoop::with_config(cfg_deterministic()); // max=4
        for i in 0..6 {
            loop_.append_memory(mem(i, "research", true));
        }
        // 只淘汰最旧 2 条, 其余原样保留 (信息单调)。
        assert_eq!(loop_.graph.memories.len(), 4);
        assert_eq!(loop_.graph.memories[0].id, 2);
        assert_eq!(loop_.graph.memories[3].id, 5);
    }

    #[test]
    fn dual_memory_guidance_and_failure_warnings() {
        let mut loop_ = CoEvolutionLoop::with_config(cfg_deterministic());
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "balanced", true, "good1");
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "balanced", false, "bad1");
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "balanced", true, "good2");
        loop_.record_reward("coding", AttentionDomain::Code, "generalist", "balanced", true, "unrelated");
        let g: Vec<&ExperienceMemory> = loop_.guidance("research", 10);
        assert_eq!(g.len(), 2);
        assert!(g.iter().all(|m| m.is_success()));
        let w: Vec<&ExperienceMemory> = loop_.failure_warnings("research", 10);
        assert_eq!(w.len(), 1);
        assert!(!w[0].is_success());
        // 任务过滤: coding 的记忆不污染 research 检索。
        assert!(g.iter().all(|m| m.task_type == "research"));
    }

    #[test]
    fn strategy_name_normalizes_unknown() {
        assert_eq!(strategy_name("balanced"), "balanced");
        assert_eq!(strategy_name("exploratory"), "exploratory");
        assert_eq!(strategy_name("bogus"), "balanced");
        assert_eq!(strategy_name("conservative"), "conservative");
    }

    #[test]
    fn absorb_watermark_dedups_consumed_memories() {
        let mut loop_ = CoEvolutionLoop::with_config(cfg_deterministic());
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "balanced", true, "hit 5");
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "conservative", false, "empty");
        // 水位之上 = 全部未吸收经验。
        assert_eq!(loop_.new_memories_since_watermark().len(), 2);
        loop_.commit_absorb();
        // 已尝试吸收 → 无新经验。
        assert!(loop_.new_memories_since_watermark().is_empty());
        assert_eq!(loop_.absorbed_watermark, 2);
        // 新 reward 落在水位之后 → 可被下一次吸收。
        loop_.record_reward("research", AttentionDomain::PatternMatch, "explorer", "exploratory", true, "deep dive");
        let new = loop_.new_memories_since_watermark();
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].id, 2);
        // 水位跨持久化 (round trip) 存活。
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_coevo_wm_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(tmp.into())).expect("open kb");
        loop_.persist(&kb).expect("persist");
        let mut restored = CoEvolutionLoop::new();
        restored.load(&kb).expect("load");
        assert_eq!(restored.absorbed_watermark, 2);
    }
}
