//! GOD Agent — 中央路由编排器
//!
//! 所有用户消息经 GOD Agent 路由：分类任务 → 选择 agent persona → 调度模型 → 管理熔断 → 人类升级。
//! 不是独立 agent，而是 ConsciousnessTree + GWT + AgentIdentity 的协调层。
//!
//! 设计启发: Munder Difflin "Michael" (GOD agent) + Cumora agent-as-teammate

use serde::{Deserialize, Serialize};
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
    /// Prefer specialist agents over generalists
    pub prefer_specialists: bool,
    /// Max cost per routing decision (USD)
    pub max_cost_per_decision: f64,
    /// Fallback agent ID when no specialist available
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

        // Find best match
        let (task_type, confidence, keywords) = if let Some((ty, (score, kw))) = scores
            .iter()
            .max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap_or(std::cmp::Ordering::Equal))
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

/// Select the best agent from available candidates
fn select_agent(agents: &[AgentInfo], specialty: &str, config: &AgentSelectionConfig) -> Option<AgentInfo> {
    let min_autonomy = autonomy_score(&config.min_autonomy);

    let mut candidates: Vec<&AgentInfo> = agents
        .iter()
        .filter(|a| {
            a.status == "available" && autonomy_score(&a.autonomy_level) >= min_autonomy
        })
        .filter(|a| a.specialty == specialty || a.tags.contains(&specialty.to_string()))
        .collect();

    if candidates.is_empty() {
        // Fallback to any available agent
        candidates = agents
            .iter()
            .filter(|a| {
                a.status == "available" && autonomy_score(&a.autonomy_level) >= min_autonomy
            })
            .collect();
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort by specialty exact match, then cost (cheapest), then least experienced
    candidates.sort_by(|a, b| {
        let a_exact = if a.specialty == specialty { 0 } else { 1 };
        let b_exact = if b.specialty == specialty { 0 } else { 1 };
        a_exact
            .cmp(&b_exact)
            .then_with(|| a.cost_budget.partial_cmp(&b.cost_budget).unwrap_or(std::cmp::Ordering::Equal))
    });

    candidates.into_iter().next().cloned()
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
}
