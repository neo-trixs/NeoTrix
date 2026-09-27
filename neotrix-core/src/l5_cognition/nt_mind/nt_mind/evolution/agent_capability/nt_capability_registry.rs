//! 派单注册表 (路由学习 + 拓扑边集) — 有界自进化。

use super::nt_capability_types::{RouteLearnerConfig, TopologyRepair};
use crate::l5_cognition::l1_facade::KnowledgeBase;
use crate::l5_cognition::l1_facade::attention_head::AttentionDomain;

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
    outcomes: std::collections::HashMap<
        AttentionDomain,
        std::collections::HashMap<&'static str, (u32, u32)>,
    >,
    /// 学习策略配置 (min_evidence 可调, 不再硬编码 3)
    pub config: RouteLearnerConfig,
}

impl Default for RouteLearner {
    fn default() -> Self {
        Self::new()
    }
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
        self.outcomes
            .get(&domain)
            .map(|m| {
                m.values()
                    .any(|(_, attempts)| *attempts >= self.config.min_evidence)
            })
            .unwrap_or(false)
    }

    /// 学习后的路由: 若有足够证据, 返回静态档案里在域上成功率最高的档案;
    /// 否则沿用静态映射。
    ///
    /// P5 (自进化验证发现): 纯利用会饿死备选档案 — 一旦某档案达到证据阈值,
    /// 路由只从达标档案里选最优, 未达阈值的备选再无机会被观察, 形成死锁。
    /// 修复: 冷启动覆盖 (MAGE curriculum coverage, 与 P4 任务级搜索 bandit 对齐) —
    /// 只要存在未达证据阈值的已见候选, 就探索尝试最少的候选, 保证每臂都被观察;
    /// 全部已见臂都达标后才纯利用。
    ///
    /// 2026-09-27 复核: 覆盖集只含已见臂是**既定契约** (tests.rs
    /// `route_learner_below_evidence_keeps_static` / `..._config_is_calibratable`
    /// 要求证据不足时保持 static 防冷启动噪声), 未观察臂由提示路由层保证被派单。
    pub fn route(&self, domain: AttentionDomain, static_agent: &'static str) -> &'static str {
        if !self.has_enough_evidence(domain) {
            return static_agent;
        }
        let empty = std::collections::HashMap::new();
        let map = self.outcomes.get(&domain).unwrap_or(&empty);
        let rate = |s: &u32, t: &u32| *s as f64 / (*t).max(1) as f64;
        // 冷启动覆盖: 存在未达阈值的已见臂 → 探索尝试最少的臂。
        let under_evidence: Vec<(&str, u32)> = map
            .iter()
            .filter(|(_, (_, attempts))| *attempts < self.config.min_evidence)
            .map(|(agent, (_, attempts))| (*agent, *attempts))
            .collect();
        if !under_evidence.is_empty() {
            if let Some((least, _)) = under_evidence.iter().min_by_key(|(_, attempts)| *attempts) {
                return least;
            }
        }
        // 全部已见臂都达标 → 利用成功率最高的档案。
        map.iter()
            .filter(|(_, (_, attempts))| *attempts >= self.config.min_evidence)
            .max_by(|(_, (sa, ta)), (_, (sb, tb))| {
                rate(sa, ta)
                    .partial_cmp(&rate(sb, tb))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(agent, _)| *agent)
            .unwrap_or(static_agent)
    }

    /// 该域当前各档案成功率一览 (诊断/审计用)。
    pub fn rates(&self, domain: AttentionDomain) -> Vec<(&'static str, f64, u32)> {
        self.outcomes
            .get(&domain)
            .map(|m| {
                m.iter()
                    .map(|(a, (s, t))| (*a, *s as f64 / (*t).max(1) as f64, *t))
                    .collect()
            })
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
        let parsed: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("route_learner deserialize: {}", e))?;
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

/// 派单拓扑 (P3, MANTA 式) — 域→档案 的边集合, 推理期可自进化。
///
/// MANTA (arXiv 2607.28527) 三个机制落到派单层:
///   1. **任务条件化初始化** `for_task_type`: 不同任务类型给不同初始拓扑
///      (如 research 任务初始把 PatternMatch 指向 researcher)。
///   2. **trace 审计 + 有界结构修复** `audit`/`apply_repair`: 当某域当前档案
///      长期成功率低于候选档案时, 改这条边 (改 agent 角色/链路), 保持 agent
///      预算与任务接口不变 — 拓扑自进化, 而非永远固定静态映射。
///   3. **跨轮 playbook** `persist`/`load`: 拓扑修复经验经 KB kv_store 跨会话
///      存活, 让"什么样的组织更好"沉淀为可复用知识。
///
/// 修复是有界的: 仅接受规范档案名 (researcher/explorer/planner/generalist/
/// verifier/watcher), 不改 agent 总数、不改派单接口 — MANTA "preserving the
/// task interface and agent budget"。
#[derive(Debug, Clone)]
pub struct DispatchTopology {
    /// 域 → 档案 的通信边 (派单拓扑的边集)
    pub edges: std::collections::HashMap<AttentionDomain, &'static str>,
    /// 已应用的修复次数 (拓扑修订号, 自进化履历)
    pub revision: u64,
    /// 最近一次修复记录
    pub last_repair: Option<TopologyRepair>,
}

impl DispatchTopology {
    /// 任务条件化初始化 — 依据任务类型给出初始边集 (MANTA: task-conditioned init)。
    pub fn for_task_type(task_type: &str) -> Self {
        let mut edges = std::collections::HashMap::new();
        let lower = task_type.to_lowercase();
        if lower.contains("research") || lower.contains("study") {
            // research 任务: 初始把检索域指向 researcher (网络研究) 而非 explorer
            edges.insert(AttentionDomain::PatternMatch, "researcher");
        } else {
            edges.insert(AttentionDomain::PatternMatch, "explorer");
        }
        edges.insert(AttentionDomain::SelfReflection, "verifier");
        edges.insert(AttentionDomain::RiskAssessment, "verifier");
        edges.insert(AttentionDomain::Planning, "planner");
        edges.insert(AttentionDomain::GoalAlignment, "planner");
        edges.insert(AttentionDomain::Code, "generalist");
        edges.insert(AttentionDomain::ToolUse, "generalist");
        edges.insert(AttentionDomain::Temporal, "generalist");
        edges.insert(AttentionDomain::Semantic, "watcher");
        edges.insert(AttentionDomain::Creativity, "planner");
        Self {
            edges,
            revision: 0,
            last_repair: None,
        }
    }

    /// 当前某域的档案 (若边不存在回退 explorer)。
    pub fn agent_for(&self, domain: AttentionDomain) -> &'static str {
        self.edges.get(&domain).copied().unwrap_or("explorer")
    }

    /// trace 审计 — 对每个域, 若当前档案尝试 ≥ min_evidence 且候选档案
    /// 成功率显著更高 (> 15pp), 提议结构修复 (MANTA: bounded structural update)。
    pub fn audit(&self, learner: &RouteLearner) -> Vec<TopologyRepair> {
        let mut repairs = Vec::new();
        for (domain, current) in &self.edges {
            let rates = learner.rates(*domain);
            let current_rate = rates
                .iter()
                .find(|(a, _, _)| *a == *current)
                .map(|(_, r, _)| *r)
                .unwrap_or(0.0);
            let current_attempts = rates
                .iter()
                .find(|(a, _, _)| *a == *current)
                .map(|(_, _, t)| *t)
                .unwrap_or(0);
            if current_attempts < learner.config.min_evidence {
                continue;
            }
            // 候选: 尝试 ≥ min_evidence 且成功率 > 当前 + 15pp
            let best_alt = rates
                .iter()
                .filter(|(a, _, t)| *a != *current && *t >= learner.config.min_evidence)
                .max_by(|(_, ra, _), (_, rb, _)| {
                    ra.partial_cmp(rb).unwrap_or(std::cmp::Ordering::Equal)
                });
            if let Some((candidate, candidate_rate, candidate_attempts)) = best_alt {
                if *candidate_rate > current_rate + 0.15 {
                    repairs.push(TopologyRepair {
                        domain: *domain,
                        from_agent: current,
                        to_agent: candidate,
                        from_success_rate: current_rate,
                        to_success_rate: *candidate_rate,
                        evidence_attempts: *candidate_attempts,
                    });
                }
            }
        }
        repairs
    }

    /// 跨域能量流审计 (P7, D45) — 在 learner 统计之上, 叠加 coevo 经验子图证据:
    /// 当前档案在该任务近期有失败警示, 且候选档案掌握度显著更高时, 提议结构修复。
    /// 有界: 候选仅限规范档案; 证据不足 (任务 reward < min_evidence) 不动结构。
    pub fn audit_with_experience(
        &self,
        learner: &RouteLearner,
        coevo: &crate::l5_cognition::nt_mind::nt_mind::evolution::co_evolution::CoEvolutionLoop,
        task_type: &str,
    ) -> Vec<TopologyRepair> {
        let mut repairs = self.audit(learner);
        if coevo.task_rewards(task_type) < coevo.config.min_evidence as usize {
            return repairs;
        }
        let failed = coevo.failure_warnings(task_type, 3);
        if failed.is_empty() {
            return repairs;
        }
        let candidates = [
            "researcher",
            "explorer",
            "planner",
            "generalist",
            "verifier",
            "watcher",
        ];
        for (domain, current) in &self.edges {
            let current_failures = failed.iter().filter(|m| m.agent == *current).count();
            if current_failures == 0 {
                continue;
            }
            let mut best: Option<(&'static str, f64, u32)> = None;
            for agent in candidates {
                if agent == *current {
                    continue;
                }
                let mastery = coevo.mastery(*domain, agent);
                if mastery < coevo.config.mastery_gate {
                    continue;
                }
                if best.map(|(_, b, _)| mastery > b).unwrap_or(true) {
                    best = Some((agent, mastery, current_failures as u32));
                }
            }
            if let Some((candidate, candidate_mastery, evidence)) = best {
                let current_mastery = coevo.mastery(*domain, current);
                if candidate_mastery > current_mastery + 0.15 {
                    repairs.push(TopologyRepair {
                        domain: *domain,
                        from_agent: current,
                        to_agent: candidate,
                        from_success_rate: current_mastery,
                        to_success_rate: candidate_mastery,
                        evidence_attempts: evidence,
                    });
                }
            }
        }
        repairs
    }

    /// 有界结构修复 — 仅接受规范档案名, 应用后 revision+1, 记录 last_repair。
    pub fn apply_repair(&mut self, repair: &TopologyRepair) -> bool {
        let Some(canonical) = Self::canonical_agent(repair.to_agent) else {
            return false;
        };
        self.edges.insert(repair.domain, canonical);
        self.revision += 1;
        self.last_repair = Some(repair.clone());
        true
    }

    /// 拓扑边数量 (审计用)。
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 持久化到 KB kv_store (MANTA cross-run playbook)。
    pub fn persist(&self, kb: &KnowledgeBase) -> Result<(), String> {
        let edges: std::collections::HashMap<String, String> = self
            .edges
            .iter()
            .map(|(d, a)| (format!("{:?}", d), a.to_string()))
            .collect();
        let payload = serde_json::json!({
            "edges": edges,
            "revision": self.revision,
        });
        let json =
            serde_json::to_string(&payload).map_err(|e| format!("topology serialize: {}", e))?;
        kb.save_dispatch_topology(&json)
    }

    /// 从 KB 恢复拓扑 (无存档则保持当前, 冷启动安全)。
    pub fn load(&mut self, kb: &KnowledgeBase) -> Result<(), String> {
        let Some(json) = kb.load_dispatch_topology()? else {
            return Ok(());
        };
        let parsed: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("topology deserialize: {}", e))?;
        self.revision = parsed.get("revision").and_then(|r| r.as_u64()).unwrap_or(0);
        if let Some(edges) = parsed.get("edges").and_then(|e| e.as_object()) {
            for (domain_key, agent) in edges {
                let Some(domain) = Self::parse_domain(domain_key) else {
                    continue;
                };
                if let Some(agent_str) = agent.as_str() {
                    if let Some(canonical) = Self::canonical_agent(agent_str) {
                        self.edges.insert(domain, canonical);
                    }
                }
            }
        }
        Ok(())
    }

    /// 从字符串还原注意力域 (与 RouteLearner::parse_domain 一致, 复用持久化格式)。
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

    /// 规范档案名白名单 (保持 agent 预算, 防注入)。
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
