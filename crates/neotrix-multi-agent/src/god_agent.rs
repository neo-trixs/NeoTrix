//! GOD Agent — 中央路由编排器
//!
//! 所有用户消息经 GOD Agent 路由：分类任务 → 选择 agent persona → 调度模型 → 管理熔断 → 人类升级。
//! 不是独立 agent，而是 ConsciousnessTree + GWT + AgentIdentity 的协调层。
//!
//! 设计启发: Munder Difflin "Michael" (GOD agent) + Cumora agent-as-teammate

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

// ─── GOD Agent ──────────────────────────────────────────────────────────────

/// GOD Agent — central orchestrator for all user-facing interactions
pub struct GodAgent {
    /// Task classifier
    classifier: TaskClassifier,
    /// Agent selection config
    selection: AgentSelectionConfig,
    /// Escalation rules
    escalation: EscalationRules,
    /// Routing history for learning
    history: Vec<RoutingDecision>,
}

/// Config for agent selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSelectionConfig {
    /// Prefer specialist agents over generalists.
    /// `false` ⇒ 完全不按专长筛候选 (只剩治理闸 + 成本闸)。
    pub prefer_specialists: bool,
    /// Max cost per routing decision (USD) —— 成本闸上限。
    ///
    /// 语义本文件按「**每次决策**的预算」写, 但被比较的对象是
    /// [`AgentInfo::cost_budget`], 而后者在 `AgentInfo` 里**没有任何字段
    /// 文档** (唯一可依据的是字段名本身)。因此本文件按「预算 vs 预算」的
    /// 同一量纲比较, 并把这个量纲歧义显式写下来而不是假装它已被解决。
    pub max_cost_per_decision: f64,
    /// Fallback agent ID used when no specialist matches.
    ///
    /// 优先级**高于**「任意合规 agent」, 但**低于**治理闸与成本闸:
    /// fallback 自己不合规 (不可用 / 自治不足 / 超预算) 时不会被选,
    /// 改由「任意合规 agent」兜底。
    pub fallback_agent_id: Option<String>,
    /// Minimum autonomy level required
    pub min_autonomy: String,
}

impl Default for AgentSelectionConfig {
    fn default() -> Self {
        Self {
            prefer_specialists: true,
            max_cost_per_decision: 5.0,
            fallback_agent_id: None,
            min_autonomy: "constrained".to_string(),
        }
    }
}

/// Escalation rules — when to escalate to human
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRules {
    /// Escalate on stop-level circuit breaker
    pub escalate_on_stop: bool,
    /// Escalate when cost exceeds threshold
    pub escalate_on_cost: bool,
    /// Escalate on ambiguous task classification
    pub escalate_on_ambiguity: bool,
    /// Ambiguity threshold (below this = ambiguous)
    pub ambiguity_threshold: f64,
}

impl Default for EscalationRules {
    fn default() -> Self {
        Self {
            escalate_on_stop: true,
            escalate_on_cost: true,
            escalate_on_ambiguity: true,
            ambiguity_threshold: 0.6,
        }
    }
}

// ─── Task Classification ────────────────────────────────────────────────────

/// Task classifier — determines what kind of task the user is asking
pub struct TaskClassifier {
    /// Keyword-based rules
    rules: Vec<ClassificationRule>,
}

/// A single classification rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// Rule name
    pub name: String,
    /// Keywords that trigger this rule
    pub keywords: Vec<String>,
    /// Target task type
    pub task_type: TaskType,
    /// Confidence boost when matched
    pub confidence_boost: f64,
}

/// Classified task type
///
/// ⚠️ **变体声明顺序是 `task_type_rank` 的平手裁决序** —— 分类同分时先声明的
/// 变体胜出。新增变体请追加到**末尾**, 不要插在中间, 否则会静默改变既有
/// 平手分类的结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Code generation / implementation
    CodeGeneration,
    /// Code review / audit
    CodeReview,
    /// Debugging / troubleshooting
    Debugging,
    /// Research / analysis
    Research,
    /// Architecture / design
    Architecture,
    /// Documentation / writing
    Documentation,
    /// Testing
    Testing,
    /// General chat
    GeneralChat,
    /// System administration
    SystemAdmin,
    /// File operations
    FileOperations,
}

impl TaskType {
    /// Preferred agent specialty for this task type
    pub fn preferred_specialty(&self) -> &str {
        match self {
            TaskType::CodeGeneration => "code_generation",
            TaskType::CodeReview => "code_review",
            TaskType::Debugging => "debugging",
            TaskType::Research => "research",
            TaskType::Architecture => "architecture",
            TaskType::Documentation => "documentation",
            TaskType::Testing => "testing",
            TaskType::GeneralChat => "general",
            TaskType::SystemAdmin => "system_admin",
            TaskType::FileOperations => "file_operations",
        }
    }
}

/// 分类平手裁决序号 —— 与 `TaskType` 变体声明顺序严格一致。
///
/// 为什么需要它: `classify` 的分数存在 `HashMap<TaskType, …>` 里, 而
/// `HashMap` 的迭代顺序每个进程随机 ⇒ 只按 `score` 做 `max_by` 时, **同分的
/// 赢家会随运行次数变化**。这里给出一个覆盖全部变体的**全序** (0..=9 两两
/// 不同), 于是「分数相同」总能被它裁决掉, 不再需要任何随机来源。
///
/// 不发明「哪个域更重要」的政策 —— 声明序本身已是仓内既有约定, 且新增变体
/// 时行为可预测 (见 `TaskType` 的文档警告)。
fn task_type_rank(t: &TaskType) -> u8 {
    match t {
        TaskType::CodeGeneration => 0,
        TaskType::CodeReview => 1,
        TaskType::Debugging => 2,
        TaskType::Research => 3,
        TaskType::Architecture => 4,
        TaskType::Documentation => 5,
        TaskType::Testing => 6,
        TaskType::GeneralChat => 7,
        TaskType::SystemAdmin => 8,
        TaskType::FileOperations => 9,
    }
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Detected task type
    pub task_type: TaskType,
    /// Confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Matched keywords
    pub matched_keywords: Vec<String>,
    /// Suggested model tier
    pub suggested_tier: String,
}

impl Default for TaskClassifier {
    fn default() -> Self {
        Self {
            rules: default_classification_rules(),
        }
    }
}

impl TaskClassifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify a user message
    pub fn classify(&self, message: &str) -> ClassificationResult {
        let msg_lower = message.to_lowercase();
        let mut scores: HashMap<TaskType, (f64, Vec<String>)> = HashMap::new();

        for rule in &self.rules {
            let mut matched = Vec::new();
            for keyword in &rule.keywords {
                if msg_lower.contains(keyword.as_str()) {
                    matched.push(keyword.clone());
                }
            }
            if !matched.is_empty() {
                let entry = scores
                    .entry(rule.task_type.clone())
                    .or_insert_with(|| (0.0, Vec::new()));
                entry.0 += rule.confidence_boost * matched.len() as f64;
                entry.1.extend(matched);
            }
        }

        // Find best match.
        //
        // `scores` 是 `HashMap`, 迭代顺序每进程随机 ⇒ 光按分数 `max_by` 在同分
        // 时赢家是不确定的。第二级判据 `task_type_rank` 是覆盖全部变体的全序,
        // 把同分情况完全裁决掉, 结果与运行次数、与 HashMap 的随机种子都无关。
        let (task_type, confidence, keywords) = if let Some((ty, (score, kw))) = scores
            .iter()
            .max_by(|a, b| {
                a.1 .0
                    .partial_cmp(&b.1 .0)
                    .unwrap_or(Ordering::Equal)
                    // 反向比较 ⇒ 序号小的 (先声明的) 变体赢下这轮平手
                    .then_with(|| task_type_rank(&b.0).cmp(&task_type_rank(&a.0)))
            })
        {
            (
                ty.clone(),
                (score / 1.2).min(1.0), // normalize: single-keyword ~0.67, multi-keyword ~0.8+
                kw.clone(),
            )
        } else {
            (TaskType::GeneralChat, 0.5, Vec::new())
        };

        let suggested_tier = match &task_type {
            TaskType::CodeGeneration | TaskType::Architecture => "balanced".to_string(),
            TaskType::CodeReview | TaskType::Debugging => "powerful".to_string(),
            TaskType::Research => "powerful".to_string(),
            TaskType::GeneralChat => "fast".to_string(),
            _ => "balanced".to_string(),
        };

        ClassificationResult {
            task_type,
            confidence,
            matched_keywords: keywords,
            suggested_tier,
        }
    }
}

/// Default classification rules
fn default_classification_rules() -> Vec<ClassificationRule> {
    vec![
        ClassificationRule {
            name: "code_generation".to_string(),
            keywords: vec![
                "实现".to_string(),
                "写代码".to_string(),
                "implement".to_string(),
                "create".to_string(),
                "build".to_string(),
                "add function".to_string(),
                "add feature".to_string(),
                "新建".to_string(),
                "添加".to_string(),
                "开发".to_string(),
            ],
            task_type: TaskType::CodeGeneration,
            confidence_boost: 0.8,
        },
        ClassificationRule {
            name: "code_review".to_string(),
            keywords: vec![
                "审查".to_string(),
                "review".to_string(),
                "audit".to_string(),
                "check code".to_string(),
                "代码审查".to_string(),
                "安全检查".to_string(),
            ],
            task_type: TaskType::CodeReview,
            confidence_boost: 0.9,
        },
        ClassificationRule {
            name: "debugging".to_string(),
            keywords: vec![
                "调试".to_string(),
                "debug".to_string(),
                "fix".to_string(),
                "bug".to_string(),
                "error".to_string(),
                "报错".to_string(),
                "修复".to_string(),
                "异常".to_string(),
            ],
            task_type: TaskType::Debugging,
            confidence_boost: 0.85,
        },
        ClassificationRule {
            name: "research".to_string(),
            keywords: vec![
                "研究".to_string(),
                "调研".to_string(),
                "research".to_string(),
                "分析".to_string(),
                "analyze".to_string(),
                "比较".to_string(),
                "compare".to_string(),
                "调查".to_string(),
            ],
            task_type: TaskType::Research,
            confidence_boost: 0.8,
        },
        ClassificationRule {
            name: "architecture".to_string(),
            keywords: vec![
                "架构".to_string(),
                "设计".to_string(),
                "architecture".to_string(),
                "design".to_string(),
                "重构".to_string(),
                "refactor".to_string(),
                "模块".to_string(),
                "module".to_string(),
            ],
            task_type: TaskType::Architecture,
            confidence_boost: 0.75,
        },
        ClassificationRule {
            name: "documentation".to_string(),
            keywords: vec![
                "文档".to_string(),
                "docs".to_string(),
                "document".to_string(),
                "注释".to_string(),
                "comment".to_string(),
                "README".to_string(),
                "教程".to_string(),
                "tutorial".to_string(),
            ],
            task_type: TaskType::Documentation,
            confidence_boost: 0.85,
        },
        ClassificationRule {
            name: "testing".to_string(),
            keywords: vec![
                "测试".to_string(),
                "test".to_string(),
                "测试用例".to_string(),
                "unit test".to_string(),
                "集成测试".to_string(),
                "integration test".to_string(),
            ],
            task_type: TaskType::Testing,
            confidence_boost: 0.85,
        },
    ]
}

// ─── Routing Decision ───────────────────────────────────────────────────────

/// A routing decision made by the GOD Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Input message (truncated)
    pub input: String,
    /// Classification result
    pub classification: ClassificationResult,
    /// Selected agent persona ID
    pub selected_agent_id: Option<String>,
    /// Selected agent name
    pub selected_agent_name: Option<String>,
    /// Model tier used
    pub model_tier: String,
    /// Was this escalated to human?
    pub escalated: bool,
    /// Escalation reason (if any)
    pub escalation_reason: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

// ─── GOD Agent Implementation ───────────────────────────────────────────────

impl GodAgent {
    pub fn new() -> Self {
        Self {
            classifier: TaskClassifier::new(),
            selection: AgentSelectionConfig::default(),
            escalation: EscalationRules::default(),
            history: Vec::new(),
        }
    }

    pub fn with_config(selection: AgentSelectionConfig, escalation: EscalationRules) -> Self {
        Self {
            classifier: TaskClassifier::new(),
            selection,
            escalation,
            history: Vec::new(),
        }
    }

    /// Route a user message — classify → select agent → decide model → check escalation
    pub fn route(
        &mut self,
        message: &str,
        available_agents: &[AgentInfo],
        circuit_breaker_state: Option<&str>, // "normal" | "steer" | "constrain" | "stop"
    ) -> RoutingDecision {
        // 1. Classify the task
        let classification = self.classifier.classify(message);

        // 2. Check circuit breaker — if stopped, must escalate
        if circuit_breaker_state == Some("stop") {
            let decision = RoutingDecision {
                input: truncate(message, 100),
                classification: classification.clone(),
                selected_agent_id: None,
                selected_agent_name: None,
                model_tier: classification.suggested_tier.clone(),
                escalated: true,
                escalation_reason: Some("Agent circuit breaker is STOP — requires human intervention".to_string()),
                timestamp: now_secs(),
            };
            self.history.push(decision.clone());
            return decision;
        }

        // 3. Check ambiguity — escalate if too ambiguous
        if self.escalation.escalate_on_ambiguity
            && classification.confidence < self.escalation.ambiguity_threshold
        {
            let decision = RoutingDecision {
                input: truncate(message, 100),
                classification: classification.clone(),
                selected_agent_id: None,
                selected_agent_name: None,
                model_tier: classification.suggested_tier.clone(),
                escalated: true,
                escalation_reason: Some(format!(
                    "Task classification ambiguous (confidence {:.2} < {:.1})",
                    classification.confidence, self.escalation.ambiguity_threshold
                )),
                timestamp: now_secs(),
            };
            self.history.push(decision.clone());
            return decision;
        }

        // 4. Select best agent
        let specialty = classification.task_type.preferred_specialty();
        let selected = select_agent(available_agents, specialty, &self.selection);

        let decision = RoutingDecision {
            input: truncate(message, 100),
            classification: classification.clone(),
            selected_agent_id: selected.as_ref().map(|a| a.id.clone()),
            selected_agent_name: selected.as_ref().map(|a| a.name.clone()),
            model_tier: classification.suggested_tier.clone(),
            escalated: false,
            escalation_reason: None,
            timestamp: now_secs(),
        };

        self.history.push(decision.clone());
        decision
    }

    /// Get routing history
    pub fn history(&self) -> &[RoutingDecision] {
        &self.history
    }

    /// Get classification stats
    pub fn stats(&self) -> RoutingStats {
        let total = self.history.len();
        let escalated = self.history.iter().filter(|d| d.escalated).count();
        let by_type: HashMap<String, usize> = self
            .history
            .iter()
            .fold(HashMap::new(), |mut acc, d| {
                *acc.entry(format!("{:?}", d.classification.task_type))
                    .or_insert(0) += 1;
                acc
            });
        RoutingStats {
            total,
            escalated,
            by_type,
        }
    }
}

// ─── Agent Selection ────────────────────────────────────────────────────────

/// Agent info for selection (subset of AgentPersona)
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub specialty: String,
    pub autonomy_level: String, // "readonly" | "constrained" | "supervised" | "autonomous"
    pub status: String,         // "available" | "busy" | "unavailable" | "disabled"
    pub cost_budget: f64,
    pub tags: Vec<String>,
}

/// Select the best agent from available candidates.
///
/// 候选分层, 逐层放宽 (先到先得):
///
/// | 层 | 条件 |
/// |---|---|
/// | T0 | 治理闸 + 成本闸 + 专长匹配 (`prefer_specialists == true` 时才启用) |
/// | T1 | 治理闸 + 成本闸 + `id == fallback_agent_id` |
/// | T2 | 治理闸 + 成本闸 (任意) |
/// | T3 | 治理闸, **不含成本闸** —— 泄压阀, 见下 |
///
/// 「治理闸」= `status == "available"` 且 `autonomy_score(level) >= min_autonomy`。
///
/// # 成本闸 (`max_cost_per_decision`)
///
/// T0~T2 一律排除 `cost_budget > max_cost_per_decision` 的候选; `cost_budget`
/// 为 `NaN` 视为不可用的预算声明 ⇒ 同样排除。
///
/// ## 全员超预算时的规则 = 泄压阀 (fail-open), T3
///
/// 若治理闸内**没有任何**候选落在成本上限内, 则只按「超预算最少」放开一个:
/// 超支量 `cost_budget - max_cost_per_decision` 最小者胜。
///
/// **为什么不用 deny-by-default (返回 `None`)。** 本模块的基调确实是 fail-closed
/// (熔断 `stop` 与分类歧义都升级到人类), 但对 `select_agent` 而言
/// 「全超预算 ⇒ `None`」并不是 fail-closed, 而是**静默丢件**: 上游 `route`
/// 在拿到 `None` 时只把 `selected_agent_id` 置空, 既不置 `escalated` 也不填
/// `escalation_reason` ⇒ 一次决策被丢弃且下游无从得知原因。语义上, 成本超限
/// 属于**资源约束**而非**安全策略** —— 安全策略该停机 (熔断 `stop` 已经覆盖
/// 这条路), 资源约束该降级。泄压阀保住了零停机, 同时「已超预算」这一事实
/// 仍然保留在返回值的 `cost_budget` 上供上游观测。
///
/// **被否决的备选:**
/// * 返回 `None` —— 如上, 在 `route` 里退化成无理由、无升级标记的静默丢件。
/// * 改签名成 `Result` —— `RoutingDecision` 没有承载错误的字段, 加字段是
///   破坏性改动, 且超出本次缺陷修复的范围。
///
/// 泄压阀**不**绕过治理闸: 若无人过得了治理闸 (全不可用 / 自治不足),
/// `select_agent` 仍返回 `None`。
fn select_agent(
    agents: &[AgentInfo],
    specialty: &str,
    config: &AgentSelectionConfig,
) -> Option<AgentInfo> {
    let min_autonomy = autonomy_score(&config.min_autonomy);
    let cap = config.max_cost_per_decision;

    // 治理闸: 可用 + 自治级别达标。与成本闸正交 —— 泄压阀也必须过这一关。
    let governed = |a: &AgentInfo| {
        a.status == "available" && autonomy_score(&a.autonomy_level) >= min_autonomy
    };
    // 成本闸。NaN 预算判为不达标。
    let within_cap = |a: &AgentInfo| !a.cost_budget.is_nan() && a.cost_budget <= cap;
    // 合规 = 治理闸 ∧ 成本闸
    let compliant = |a: &AgentInfo| governed(a) && within_cap(a);

    // T0 —— 专长精确命中或 tag 命中 (仅当 prefer_specialists)。
    // `prefer_specialists == false` ⇒ 这一层的过滤器恒为 false, 专长偏好完全不生效。
    let specialists: Vec<&AgentInfo> = agents
        .iter()
        .filter(|a| compliant(a))
        .filter(|a| {
            config.prefer_specialists
                && (a.specialty == specialty || a.tags.iter().any(|t| t.as_str() == specialty))
        })
        .collect();

    if let Some(best) = pick_best(&specialists, specialty, config) {
        return Some(best.clone());
    }

    // T1 —— 指定 fallback_agent_id。仍须过治理闸 + 成本闸, 故配置错误不会
    // 把一个不合规的 agent 顶上来。
    if let Some(fallback_id) = config.fallback_agent_id.as_deref() {
        let fallback: Vec<&AgentInfo> = agents
            .iter()
            .filter(|a| compliant(a) && a.id == fallback_id)
            .collect();
        if let Some(best) = pick_best(&fallback, specialty, config) {
            return Some(best.clone());
        }
    }

    // T2 —— 任意合规候选。
    let compliants: Vec<&AgentInfo> = agents.iter().filter(|a| compliant(a)).collect();
    if let Some(best) = pick_best(&compliants, specialty, config) {
        return Some(best.clone());
    }

    // T3 —— 泄压阀: 全员超预算 ⇒ 只按「超预算最少」放开 (见函数文档)。
    agents
        .iter()
        .filter(|a| governed(a) && !within_cap(a))
        .min_by(|a, b| {
            let a_over = cost_overshoot(a.cost_budget, cap);
            let b_over = cost_overshoot(b.cost_budget, cap);
            a_over.partial_cmp(&b_over).unwrap_or(Ordering::Equal)
        })
        .cloned()
}

/// 在候选集里挑最优: 专长精确命中优先 (仅当 `prefer_specialists`) → 成本预算低者优先。
///
/// **关于原注释「then least experienced」的更正**: `AgentInfo` 根本没有经验
/// 字段 (无 `experience` / `level` / `success_count` 之类), 仓内也没有任何
/// 可派生的经验量, 因此这一层排序**无法实现**。这里选择**更正注释**而不是
/// 凭空造一个 `u32` 经验分 —— 后者要么恒为 0 (等价于没有这一层, 只是把一个
/// 死注释换成一个死字段), 要么就得伪造一个数据模型里不存在的语义。
///
/// 排序键全部相等时保留输入顺序 (`min_by` 返回第一个最小值), 所以只要
/// 候选集是从调用方切片按序收集的, 选择结果就是确定的。
fn pick_best<'a>(
    candidates: &[&'a AgentInfo],
    specialty: &str,
    config: &AgentSelectionConfig,
) -> Option<&'a AgentInfo> {
    candidates.iter().copied().min_by(|a, b| {
        let specialty_rank = if config.prefer_specialists {
            let a_exact = i32::from(a.specialty != specialty);
            let b_exact = i32::from(b.specialty != specialty);
            a_exact.cmp(&b_exact)
        } else {
            Ordering::Equal
        };
        specialty_rank.then_with(|| {
            a.cost_budget
                .partial_cmp(&b.cost_budget)
                .unwrap_or(Ordering::Equal)
        })
    })
}

/// 相对成本上限的超支量 (>= 0)。
///
/// `NaN` 预算视为最大超支 —— 一条不可用的预算声明永远排在真实超支之后,
/// 免得 `partial_cmp` 返回 `None` 后退化成一个碰巧的输入序。
fn cost_overshoot(cost_budget: f64, max_cost_per_decision: f64) -> f64 {
    if cost_budget.is_nan() {
        return f64::INFINITY;
    }
    let over = cost_budget - max_cost_per_decision;
    if over > 0.0 {
        over
    } else {
        0.0
    }
}

fn autonomy_score(level: &str) -> u8 {
    match level {
        "readonly" => 0,
        "constrained" => 1,
        "supervised" => 2,
        "autonomous" => 3,
        _ => 0,
    }
}

// ─── Stats ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStats {
    pub total: usize,
    pub escalated: usize,
    pub by_type: HashMap<String, usize>,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code_generation() {
        let classifier = TaskClassifier::new();
        let r = classifier.classify("帮我实现一个解析器");
        assert_eq!(r.task_type, TaskType::CodeGeneration);
        assert!(r.confidence > 0.5);
    }

    #[test]
    fn test_classify_debugging() {
        let classifier = TaskClassifier::new();
        let r = classifier.classify("这个函数报错了，帮我调试");
        assert_eq!(r.task_type, TaskType::Debugging);
    }

    #[test]
    fn test_classify_review() {
        let classifier = TaskClassifier::new();
        let r = classifier.classify("帮我审查这段代码的安全性");
        assert_eq!(r.task_type, TaskType::CodeReview);
    }

    #[test]
    fn test_classify_general() {
        let classifier = TaskClassifier::new();
        let r = classifier.classify("今天天气怎么样");
        assert_eq!(r.task_type, TaskType::GeneralChat);
    }

    #[test]
    fn test_god_agent_route() {
        let mut god = GodAgent::new();
        let agents = vec![
            AgentInfo {
                id: "coder-1".to_string(),
                name: "编码员".to_string(),
                specialty: "code_generation".to_string(),
                autonomy_level: "constrained".to_string(),
                status: "available".to_string(),
                cost_budget: 5.0,
                tags: vec!["code".to_string()],
            },
            AgentInfo {
                id: "reviewer-1".to_string(),
                name: "审查员".to_string(),
                specialty: "code_review".to_string(),
                autonomy_level: "readonly".to_string(),
                status: "available".to_string(),
                cost_budget: 3.0,
                tags: vec!["review".to_string()],
            },
        ];

        let decision = god.route("帮我实现一个解析器", &agents, None);
        assert!(!decision.escalated);
        assert_eq!(decision.classification.task_type, TaskType::CodeGeneration);
        assert_eq!(decision.selected_agent_id.as_deref(), Some("coder-1"));
    }

    #[test]
    fn test_god_agent_escalate_on_stop() {
        let mut god = GodAgent::new();
        let agents = vec![];

        let decision = god.route("hello", &agents, Some("stop"));
        assert!(decision.escalated);
        assert!(decision.escalation_reason.unwrap().contains("STOP"));
    }

    #[test]
    fn test_god_agent_escalate_on_ambiguity() {
        let mut god = GodAgent::new();
        let agents = vec![];

        // Very short, ambiguous message
        let decision = god.route("嗯", &agents, None);
        assert!(decision.escalated);
    }

    #[test]
    fn test_agent_selection_prefers_specialist() {
        let agents = vec![
            AgentInfo {
                id: "general".to_string(),
                name: "通用".to_string(),
                specialty: "general".to_string(),
                autonomy_level: "constrained".to_string(),
                status: "available".to_string(),
                cost_budget: 2.0,
                tags: vec![],
            },
            AgentInfo {
                id: "coder".to_string(),
                name: "编码员".to_string(),
                specialty: "code_generation".to_string(),
                autonomy_level: "constrained".to_string(),
                status: "available".to_string(),
                cost_budget: 5.0,
                tags: vec![],
            },
        ];

        let config = AgentSelectionConfig::default();
        let selected = select_agent(&agents, "code_generation", &config);
        assert_eq!(selected.unwrap().id, "coder");
    }

    // ─── 测试辅助 ────────────────────────────────────────────────────────────

    /// 构造测试 agent —— 字段默认值 (name=id, autonomy=constrained, status=available)
    fn agent(id: &str, specialty: &str, autonomy: &str, status: &str, cost: f64, tags: &[&str]) -> AgentInfo {
        AgentInfo {
            id: id.to_string(),
            name: id.to_string(),
            specialty: specialty.to_string(),
            autonomy_level: autonomy.to_string(),
            status: status.to_string(),
            cost_budget: cost,
            tags: tags.iter().map(|t| t.to_string()).collect(),
        }
    }

    fn select_id(agents: &[AgentInfo], specialty: &str, config: &AgentSelectionConfig) -> Option<String> {
        select_agent(agents, specialty, config).map(|a| a.id)
    }

    // ─── 成本闸 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_cost_cap_excludes_over_budget_specialist() {
        // 唯一专长匹配但超预算 ⇒ 必须被成本闸挡下; 放宽上限后它就重新胜出。
        let agents = vec![
            agent("rich-coder", "code_generation", "constrained", "available", 9.9, &[]),
            agent("cheap-general", "general", "constrained", "available", 1.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.max_cost_per_decision = 5.0;

        // 超预算 ⇒ 落到 T2 兜底层, 绝不会选 rich-coder
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("cheap-general".to_string())
        );

        // 上限放宽到 50 ⇒ T0 恢复, 专长匹配者胜出 (差分证明闸确实在起作用)
        config.max_cost_per_decision = 50.0;
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("rich-coder".to_string())
        );
    }

    #[test]
    fn test_cost_cap_excludes_over_budget_tag_match() {
        // tag 命中也必须过成本闸
        let agents = vec![
            agent("tagged-rich", "general", "constrained", "available", 8.0, &["code_generation"]),
            agent("cheap", "general", "constrained", "available", 2.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.max_cost_per_decision = 5.0;
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("cheap".to_string())
        );
    }

    #[test]
    fn test_cost_cap_relief_valve_picks_smallest_overshoot() {
        // 故意不按成本排序输入 ⇒ 证明是按「超支量」而非输入序裁决
        let agents = vec![
            agent("huge", "code_generation", "constrained", "available", 100.0, &[]),
            agent("near-cap", "code_generation", "constrained", "available", 6.0, &[]),
            agent("mid", "code_generation", "constrained", "available", 7.5, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.max_cost_per_decision = 5.0;

        // 全员超预算 ⇒ 泄压阀: 超支量最小者 (near-cap, 超支 1.0) 胜出, 而不是 None
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("near-cap".to_string())
        );
    }

    #[test]
    fn test_cost_cap_relief_valve_still_respects_governance_gate() {
        // 泄压阀只放开成本闸, 不放开治理闸 ⇒ 无人合格时仍返回 None
        let agents = vec![
            agent("readonly-ro", "code_generation", "readonly", "available", 100.0, &[]),
            agent("disabled-ro", "code_generation", "constrained", "disabled", 100.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.max_cost_per_decision = 5.0;
        assert_eq!(select_id(&agents, "code_generation", &config), None);
    }

    #[test]
    fn test_cost_cap_nan_budget_treated_as_over_budget() {
        let agents = vec![
            agent("nan-budget", "code_generation", "constrained", "available", f64::NAN, &[]),
            agent("real-budget", "code_generation", "constrained", "available", 4.0, &[]),
        ];
        let config = AgentSelectionConfig::default();
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("real-budget".to_string())
        );
    }

    // ─── prefer_specialists ──────────────────────────────────────────────────

    #[test]
    fn test_prefer_specialists_false_demotes_specialist() {
        let agents = vec![
            agent("coder", "code_generation", "constrained", "available", 5.0, &[]),
            agent("general", "general", "constrained", "available", 2.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();

        // true ⇒ 专长优先, 即使它更贵
        assert!(config.prefer_specialists);
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("coder".to_string())
        );

        // false ⇒ 专长偏好完全失效, 只剩成本闸 ⇒ 便宜的通用 agent 胜出
        config.prefer_specialists = false;
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("general".to_string())
        );
    }

    // ─── fallback_agent_id ───────────────────────────────────────────────────

    #[test]
    fn test_fallback_agent_id_used_when_no_specialist_matches() {
        // fallback 比最便宜的候选更贵 ⇒ 只有 T1 生效时才会选它
        let agents = vec![
            agent("cheapest", "general", "constrained", "available", 1.0, &[]),
            agent("designated", "general", "constrained", "available", 4.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.fallback_agent_id = Some("designated".to_string());
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("designated".to_string())
        );

        // 取消配置 ⇒ 退回「任意合规候选」= 最便宜的
        config.fallback_agent_id = None;
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("cheapest".to_string())
        );
    }

    #[test]
    fn test_fallback_agent_id_not_used_when_autonomy_too_low() {
        let agents = vec![
            agent("ro-designated", "general", "readonly", "available", 0.5, &[]),
            agent("ok", "general", "constrained", "available", 4.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.fallback_agent_id = Some("ro-designated".to_string());
        // readonly < min_autonomy("constrained") ⇒ 不得被选
        assert_eq!(select_id(&agents, "code_generation", &config), Some("ok".to_string()));
    }

    #[test]
    fn test_fallback_agent_id_not_used_when_over_budget() {
        let agents = vec![
            agent("rich-designated", "general", "constrained", "available", 99.0, &[]),
            agent("ok", "general", "constrained", "available", 4.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.fallback_agent_id = Some("rich-designated".to_string());
        config.max_cost_per_decision = 5.0;
        assert_eq!(select_id(&agents, "code_generation", &config), Some("ok".to_string()));
    }

    #[test]
    fn test_fallback_agent_id_not_used_when_busy() {
        let agents = vec![
            agent("busy-designated", "general", "constrained", "busy", 0.5, &[]),
            agent("ok", "general", "constrained", "available", 4.0, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.fallback_agent_id = Some("busy-designated".to_string());
        assert_eq!(select_id(&agents, "code_generation", &config), Some("ok".to_string()));
    }

    #[test]
    fn test_specialist_still_beats_fallback_agent_id() {
        // T0 命中时不该被 T1 的 fallback 覆盖 (fallback 只在无专长匹配时兜底)
        let agents = vec![
            agent("coder", "code_generation", "constrained", "available", 5.0, &[]),
            agent("designated", "general", "constrained", "available", 0.5, &[]),
        ];
        let mut config = AgentSelectionConfig::default();
        config.fallback_agent_id = Some("designated".to_string());
        assert_eq!(
            select_id(&agents, "code_generation", &config),
            Some("coder".to_string())
        );
    }

    // ─── classify 确定性 ────────────────────────────────────────────────────

    #[test]
    fn test_classify_tie_break_is_deterministic() {
        // 两条规则命中同一个关键词 + 相同 boost ⇒ 分数必然完全平手。
        // 规则顺序故意与 TaskType 声明顺序相反, 若裁决依赖 HashMap 顺序就会漂。
        let classifier = TaskClassifier {
            rules: vec![
                ClassificationRule {
                    name: "research-tie".to_string(),
                    keywords: vec!["平手".to_string()],
                    task_type: TaskType::Research,
                    confidence_boost: 0.5,
                },
                ClassificationRule {
                    name: "codegen-tie".to_string(),
                    keywords: vec!["平手".to_string()],
                    task_type: TaskType::CodeGeneration,
                    confidence_boost: 0.5,
                },
            ],
        };

        // 声明序: CodeGeneration(0) 先于 Research(3) ⇒ 平手时 CodeGeneration 胜
        for _ in 0..64 {
            let r = classifier.classify("这是一个平手的消息");
            assert_eq!(r.task_type, TaskType::CodeGeneration);
            assert_eq!(r.matched_keywords, vec!["平手".to_string()]);
        }
    }

    #[test]
    fn test_task_type_rank_is_total() {
        // 全序是 classify 确定性的前提: 任两个变体序号不同。
        let all = [
            TaskType::CodeGeneration,
            TaskType::CodeReview,
            TaskType::Debugging,
            TaskType::Research,
            TaskType::Architecture,
            TaskType::Documentation,
            TaskType::Testing,
            TaskType::GeneralChat,
            TaskType::SystemAdmin,
            TaskType::FileOperations,
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        task_type_rank(a),
                        task_type_rank(b),
                        "{a:?} 与 {b:?} 序号相同 ⇒ 平手裁决退化"
                    );
                }
            }
        }
    }
}
