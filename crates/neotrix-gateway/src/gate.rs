//! nt_core_gate — 公正评审门控节点 (Unbiased Judge Panel Gate)
//!
//! 把「门禁读证据、按规则裁决」落地为确定性 + 跨家族法官聚合两层。
//!
//! Gateway crate version: trait-based LLM adapter, locally-defined trajectory types.
//! Core crate provides the concrete LLM implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ───────────────────────────── Local Types (replacing nt_core_prm deps) ────

/// Trajectory step — simplified for gateway use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step_idx: usize,
    pub action: String,
    pub input: String,
    pub output: String,
    pub success: bool,
    pub external_reward: Option<f64>,
}

/// Agent trajectory — execution trace for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrajectory {
    pub id: u64,
    pub task: String,
    pub steps: Vec<TrajectoryStep>,
    pub outcome_reward: Option<f64>,
    pub completed: bool,
}

impl AgentTrajectory {
    pub fn new(id: u64, task: String) -> Self {
        Self {
            id,
            task,
            steps: Vec::new(),
            outcome_reward: None,
            completed: false,
        }
    }

    pub fn push(&mut self, step: TrajectoryStep) {
        self.steps.push(step);
    }
}

/// Scored criterion — per-dimension score in rubric evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCriterion {
    pub name: String,
    pub score: f64,
    pub rationale: Option<String>,
}

// ───────────────────────────── LLM Provider Trait ──────────────────────────

/// LLM response from provider.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
}

/// LLM provider trait — gateway defines the interface, core implements.
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, model: &str, prompt: &str, max_tokens: u32) -> Result<LlmResponse, String>;
}

// ───────────────────────────── 基础类型 ─────────────────────────────

/// 法官家族
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgeFamily {
    Analytic,
    Heuristic,
    Symbolic,
    None,
}

/// 门控强度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateLevel {
    Light,
    Panel,
    Human,
}

/// 门控裁决
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verdict {
    Pass,
    Review,
    Block,
}

/// 护栏动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardAction {
    Allow,
    Reject,
    Quarantine,
    Escalate,
}

/// 工具可逆性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolReversibility {
    ReadOnly,
    Reversible,
    Compensable,
    Irreversible,
}

/// 工具规约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub reversibility: ToolReversibility,
    pub undo: Option<String>,
    pub authority_modifying: bool,
}

impl ToolSpec {
    pub fn read_only(name: &str) -> Self {
        Self {
            name: name.to_string(),
            reversibility: ToolReversibility::ReadOnly,
            undo: None,
            authority_modifying: false,
        }
    }

    pub fn reversible(name: &str, undo: &str) -> Self {
        Self {
            name: name.to_string(),
            reversibility: ToolReversibility::Reversible,
            undo: Some(undo.to_string()),
            authority_modifying: false,
        }
    }

    pub fn irreversible(name: &str) -> Self {
        Self {
            name: name.to_string(),
            reversibility: ToolReversibility::Irreversible,
            undo: None,
            authority_modifying: false,
        }
    }
}

/// 动作分级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionTier {
    Tier1Autonomous,
    Tier2Logged,
    Tier3Review,
    Tier4Human,
}

impl ActionTier {
    pub fn classify(tools: &[ToolSpec]) -> Self {
        let mut tier = ActionTier::Tier1Autonomous;
        for t in tools {
            let this =
                if t.authority_modifying || t.reversibility == ToolReversibility::Irreversible {
                    ActionTier::Tier4Human
                } else if t.reversibility == ToolReversibility::Compensable {
                    ActionTier::Tier3Review
                } else if t.reversibility == ToolReversibility::Reversible {
                    ActionTier::Tier2Logged
                } else {
                    ActionTier::Tier1Autonomous
                };
            if tier_rank(this) > tier_rank(tier) {
                tier = this;
            }
        }
        tier
    }

    pub fn required_gate(self) -> GateLevel {
        match self {
            ActionTier::Tier1Autonomous | ActionTier::Tier2Logged => GateLevel::Light,
            ActionTier::Tier3Review => GateLevel::Panel,
            ActionTier::Tier4Human => GateLevel::Human,
        }
    }
}

fn tier_rank(t: ActionTier) -> u8 {
    match t {
        ActionTier::Tier1Autonomous => 1,
        ActionTier::Tier2Logged => 2,
        ActionTier::Tier3Review => 3,
        ActionTier::Tier4Human => 4,
    }
}

// ───────────────────────────── 去偏配置 ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebiasConfig {
    pub verbosity_penalty: f64,
    pub verbosity_norm_len: usize,
    pub verbosity_penalty_cap: f64,
    pub require_family_separation: bool,
    pub self_preference_penalty: f64,
    pub agreement_review_threshold: f64,
    pub pass_threshold: f64,
    pub grounding_min_ratio: f64,
    pub schema_strict: bool,
}

impl Default for DebiasConfig {
    fn default() -> Self {
        Self {
            verbosity_penalty: 0.10,
            verbosity_norm_len: 600,
            verbosity_penalty_cap: 0.30,
            require_family_separation: true,
            self_preference_penalty: 0.05,
            agreement_review_threshold: 0.60,
            pass_threshold: 0.60,
            grounding_min_ratio: 0.60,
            schema_strict: true,
        }
    }
}

impl DebiasConfig {
    fn verbosity_penalty_for(&self, text: &str) -> f64 {
        let len = text.chars().count();
        if len <= self.verbosity_norm_len {
            return 0.0;
        }
        let over = (len - self.verbosity_norm_len) as f64 / self.verbosity_norm_len as f64;
        (over * self.verbosity_penalty)
            .max(0.0)
            .min(self.verbosity_penalty_cap)
    }
}

// ───────────────────────────── 忠实度 / schema ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub text: String,
    pub evidence_refs: Vec<String>,
}

impl Claim {
    pub fn new(text: &str, refs: &[&str]) -> Self {
        Self {
            text: text.to_string(),
            evidence_refs: refs.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCheck {
    pub field: String,
    pub present: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaithfulnessReport {
    pub claims_total: usize,
    pub grounded: usize,
    pub fabricated: Vec<String>,
    pub grounding_ratio: f64,
}

impl FaithfulnessReport {
    pub fn audit(claims: &[Claim], evidence_ids: &[String]) -> Self {
        let evidence: HashSet<&str> = evidence_ids.iter().map(|s| s.as_str()).collect();
        let total = claims.len();
        let mut grounded = 0usize;
        let mut fabricated = Vec::new();
        for c in claims {
            let refs: HashSet<&str> = c.evidence_refs.iter().map(|s| s.as_str()).collect();
            if refs.is_empty() {
                fabricated.push(format!("claim '{}': 无证据引用", clip(&c.text, 60)));
            } else if refs.is_subset(&evidence) {
                grounded += 1;
            } else {
                let missing: Vec<&str> = refs.difference(&evidence).copied().collect();
                fabricated.push(format!(
                    "claim '{}': 引用不存在证据 {:?}",
                    clip(&c.text, 60),
                    missing
                ));
            }
        }
        let grounding_ratio = if total == 0 {
            0.0
        } else {
            grounded as f64 / total as f64
        };
        Self {
            claims_total: total,
            grounded,
            fabricated,
            grounding_ratio,
        }
    }

    pub fn is_grounded(&self, min_ratio: f64) -> bool {
        self.grounding_ratio >= min_ratio
    }

    pub fn quarantine(&self) -> Vec<String> {
        self.fabricated.clone()
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{}…", cut)
    }
}

pub fn check_schema_fields(required: &[&str], json: &str) -> Vec<SchemaCheck> {
    let parsed = serde_json::from_str::<serde_json::Value>(json);
    let mut checks = Vec::with_capacity(required.len());
    let obj = match parsed {
        Ok(serde_json::Value::Object(map)) => Some(map),
        Ok(serde_json::Value::Null) => None,
        Ok(_) => {
            return required
                .iter()
                .map(|f| SchemaCheck {
                    field: f.to_string(),
                    present: false,
                    detail: "输出不是 JSON 对象".to_string(),
                })
                .collect();
        }
        Err(e) => {
            return required
                .iter()
                .map(|f| SchemaCheck {
                    field: f.to_string(),
                    present: false,
                    detail: format!("JSON 解析失败: {}", e),
                })
                .collect();
        }
    };
    for f in required {
        let present = obj.as_ref().map(|m| m.contains_key(*f)).unwrap_or(false);
        checks.push(SchemaCheck {
            field: f.to_string(),
            present,
            detail: if present {
                "ok".to_string()
            } else {
                format!("缺少字段 {}", f)
            },
        });
    }
    checks
}

// ───────────────────────────── 法官输入 / 意见 ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub name: String,
    pub description: String,
    pub score_examples: Vec<(f64, String)>,
}

impl RubricCriterion {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            score_examples: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricSpec {
    pub dimensions: Vec<RubricCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub real_slice: Option<String>,
    pub budget_used: Option<f64>,
    pub budget_total: f64,
    pub claimed_seeds: Option<u32>,
}

impl Attestation {
    pub fn new(budget_total: f64) -> Self {
        Self {
            real_slice: None,
            budget_used: None,
            budget_total,
            claimed_seeds: None,
        }
    }

    pub fn contradiction_score(
        &self,
        observed_budget: Option<f64>,
        observed_seeds: Option<u32>,
    ) -> f64 {
        let mut penalty = 0.0f64;
        if let Some(declared) = self.budget_used {
            if let Some(obs) = observed_budget {
                if declared > obs * 1.05 {
                    penalty += 0.5;
                }
            }
        }
        if let Some(declared) = self.claimed_seeds {
            if let Some(obs) = observed_seeds {
                if declared != obs {
                    penalty += 0.5;
                }
            }
        }
        if penalty == 0.0 && (self.real_slice.is_none() || self.budget_used.is_none()) {
            penalty += 0.1;
        }
        penalty.max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeInput {
    pub candidate: String,
    pub claims: Vec<Claim>,
    pub evidence_ids: Vec<String>,
    pub trajectory: Option<AgentTrajectory>,
    pub grounding_failures: u64,
    pub schema_failures: Vec<SchemaCheck>,
    pub producer_family: JudgeFamily,
    pub rubric: Option<RubricSpec>,
    pub samples: u8,
    pub attestation: Option<Attestation>,
}

impl JudgeInput {
    pub fn new(candidate: &str) -> Self {
        Self {
            candidate: candidate.to_string(),
            claims: Vec::new(),
            evidence_ids: Vec::new(),
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            rubric: None,
            samples: 1,
            attestation: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeOpinion {
    pub judge_id: String,
    pub family: JudgeFamily,
    pub raw_score: f64,
    pub debiased_score: f64,
    pub confidence: f64,
    pub criteria: Vec<ScoredCriterion>,
    pub attribution_tags: Vec<String>,
}

impl JudgeOpinion {
    fn new(judge_id: &str, family: JudgeFamily) -> Self {
        Self {
            judge_id: judge_id.to_string(),
            family,
            raw_score: 0.5,
            debiased_score: 0.5,
            confidence: 0.0,
            criteria: Vec::new(),
            attribution_tags: Vec::new(),
        }
    }
}

// ───────────────────────────── 法官 ─────────────────────────────

pub trait PanelJudge: Send + Sync + std::fmt::Debug {
    fn judge_id(&self) -> &str;
    fn family(&self) -> JudgeFamily;
    fn score(&self, input: &JudgeInput) -> JudgeOpinion;
}

#[async_trait::async_trait]
pub trait AsyncPanelJudge: Send + Sync + std::fmt::Debug {
    fn judge_id(&self) -> &str;
    fn family(&self) -> JudgeFamily;
    async fn score(&self, input: &JudgeInput) -> JudgeOpinion;
}

// ───────────────────────────── LLM 法官 ─────────────────────────────

pub struct LLMJudgeAdapter {
    pub id: String,
    pub family: JudgeFamily,
    pub provider: std::sync::Arc<dyn LlmProvider>,
    pub model: String,
    pub samples: u8,
}

impl std::fmt::Debug for LLMJudgeAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMJudgeAdapter")
            .field("id", &self.id)
            .field("family", &self.family)
            .field("model", &self.model)
            .field("samples", &self.samples)
            .finish()
    }
}

impl LLMJudgeAdapter {
    pub fn new(
        id: &str,
        family: JudgeFamily,
        provider: std::sync::Arc<dyn LlmProvider>,
        model: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            family,
            provider,
            model: model.to_string(),
            samples: 1,
        }
    }

    pub fn with_samples(mut self, samples: u8) -> Self {
        self.samples = samples.max(1);
        self
    }

    fn prompt(&self, input: &JudgeInput) -> String {
        let claims: Vec<String> = input
            .claims
            .iter()
            .map(|c| format!("- {}", c.text))
            .collect();
        format!(
            "你是公正评审组的一名评委。\n\
             [候选结论] {}\n\
             [声明]\n{}\n\
             [证据 id] {}\n\
             [已记录 grounding 失败] {}\n\
             请按 1-4 档 (1=完全不可信, 4=可信) 只针对[候选结论]给出综合分。\n\
             输出严格 JSON: {{\"score\": <1..4>, \"confidence\": <0..1>, \"rationale\": \"<一句话>\"}}",
            input.candidate,
            claims.join("\n"),
            input.evidence_ids.join(", "),
            input.grounding_failures,
        )
    }

    async fn complete(&self, input: &JudgeInput) -> JudgeOpinion {
        let samples = input.samples.max(self.samples).max(1) as usize;
        let mut opinions = Vec::with_capacity(samples);
        for _ in 0..samples {
            opinions.push(self.single_sample(input).await);
        }
        let mut opinion = opinions.remove(0);
        if samples > 1 {
            let mut score_sum = opinion.raw_score;
            let mut conf_sum = opinion.confidence;
            for o in &opinions {
                score_sum += o.raw_score;
                conf_sum += o.confidence;
            }
            opinion.raw_score = (score_sum / samples as f64).max(0.0).min(1.0);
            opinion.debiased_score = opinion.raw_score;
            opinion.confidence = (conf_sum / samples as f64).max(0.0).min(1.0);
            opinion
                .attribution_tags
                .push(format!("multi_sample_{}", samples));
        }
        opinion
    }

    async fn single_sample(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family);
        let max_tokens = if input.rubric.is_some() { 900 } else { 512 };
        match self
            .provider
            .complete(&self.model, &self.prompt(input), max_tokens)
            .await
        {
            Ok(resp) => {
                let raw_json = strip_markdown_fence(&resp.content);
                match serde_json::from_str::<serde_json::Value>(&raw_json) {
                    Ok(v) => {
                        let raw = v.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                        let conf = v.get("confidence").and_then(|s| s.as_f64()).unwrap_or(0.0);
                        let rationale = v
                            .get("rationale")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string();
                        let score = (raw.max(1.0).min(4.0) - 1.0) / 3.0;
                        opinion.raw_score = score.max(0.0).min(1.0);
                        opinion.confidence = conf.max(0.0).min(1.0);
                        opinion.criteria.push(ScoredCriterion {
                            name: "llm_judge".to_string(),
                            score,
                            rationale: Some(rationale),
                        });
                        opinion.attribution_tags.push("llm_provider".to_string());
                    }
                    Err(_) => {
                        opinion.raw_score = 0.2;
                        opinion.confidence = 0.0;
                        opinion
                            .attribution_tags
                            .push("llm_parse_failed".to_string());
                        opinion.criteria.push(ScoredCriterion {
                            name: "llm_judge".to_string(),
                            score: 0.2,
                            rationale: Some("LLM 未返回可解析 JSON".to_string()),
                        });
                    }
                }
            }
            Err(_) => {
                opinion.raw_score = 0.3;
                opinion.confidence = 0.0;
                opinion
                    .attribution_tags
                    .push("llm_call_failed".to_string());
            }
        }
        opinion
    }
}

fn strip_markdown_fence(raw: &str) -> String {
    let s = raw.trim();
    if s.starts_with("```") {
        let body = s.trim_start_matches('`');
        let body = body.strip_prefix("json").unwrap_or(body);
        let body = body.trim_start_matches('`');
        return body.trim().trim_end_matches('`').trim().to_string();
    }
    s.to_string()
}

#[async_trait::async_trait]
impl AsyncPanelJudge for LLMJudgeAdapter {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        self.family
    }

    async fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        self.complete(input).await
    }
}

// ───────────────────────────── 内置法官 ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticPanelJudge {
    pub id: String,
}

impl Default for AnalyticPanelJudge {
    fn default() -> Self {
        Self {
            id: "analytic-v1".to_string(),
        }
    }
}

impl PanelJudge for AnalyticPanelJudge {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        JudgeFamily::Analytic
    }

    fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family());

        if let Some(traj) = &input.trajectory {
            let total = traj.steps.len().max(1);
            let successes = traj.steps.iter().filter(|s| s.success).count();
            let base = successes as f64 / total as f64;
            let reward_boost = traj.outcome_reward.unwrap_or(0.0).max(0.0) * 0.15;
            let score = (base * 0.85 + reward_boost).max(0.0).min(1.0);
            opinion.raw_score = score;
            opinion.criteria.push(ScoredCriterion {
                name: "soundness".to_string(),
                score,
                rationale: Some(format!(
                    "轨迹 {} 步, {} 成功, 外部奖励 {}",
                    total,
                    successes,
                    traj.outcome_reward.unwrap_or(0.0)
                )),
            });
        } else {
            let non_empty = !input.candidate.trim().is_empty();
            let has_claims = !input.claims.is_empty();
            let score = match (non_empty, has_claims) {
                (true, true) => 0.85,
                (true, false) => 0.70,
                (false, _) => 0.25,
            };
            opinion.raw_score = score;
            opinion.criteria.push(ScoredCriterion {
                name: "completeness".to_string(),
                score,
                rationale: Some(format!("候选非空={}, 含声明={}", non_empty, has_claims)),
            });
        }

        if input.grounding_failures > 0 {
            let penalty = (input.grounding_failures as f64 * 0.05).max(0.0).min(0.3);
            opinion.raw_score = (opinion.raw_score - penalty).max(0.0).min(1.0);
            opinion
                .attribution_tags
                .push(format!("grounding_failures={}", input.grounding_failures));
        }
        opinion.confidence = 0.8;
        opinion
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePanelJudge {
    pub id: String,
}

impl Default for EvidencePanelJudge {
    fn default() -> Self {
        Self {
            id: "evidence-v1".to_string(),
        }
    }
}

impl PanelJudge for EvidencePanelJudge {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        JudgeFamily::Heuristic
    }

    fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family());
        let faith = FaithfulnessReport::audit(&input.claims, &input.evidence_ids);

        let mut tags = Vec::new();
        let mut criteria = Vec::new();

        criteria.push(ScoredCriterion {
            name: "faithfulness".to_string(),
            score: faith.grounding_ratio,
            rationale: Some(format!(
                "grounded {}/{}",
                faith.grounded, faith.claims_total
            )),
        });
        if !faith.fabricated.is_empty() {
            tags.push(format!("fabrications={}", faith.fabricated.len()));
        }

        let refs_total: usize = input.claims.iter().map(|c| c.evidence_refs.len()).sum();
        let coverage = if input.claims.is_empty() {
            0.0
        } else {
            refs_total as f64 / (input.claims.len() as f64).max(1.0)
        };
        let coverage_score = (coverage / 2.0).max(0.0).min(1.0);
        criteria.push(ScoredCriterion {
            name: "evidence_coverage".to_string(),
            score: coverage_score,
            rationale: Some(format!(
                "平均引用 {}",
                if input.claims.is_empty() {
                    0
                } else {
                    refs_total / input.claims.len()
                }
            )),
        });

        let len = input.candidate.chars().count();
        let mut score = faith.grounding_ratio * 0.7 + coverage_score * 0.3;
        if len > 0 && input.candidate.trim().is_empty() {
            score = 0.2;
            tags.push("empty_candidate".to_string());
        }
        opinion.raw_score = score;
        opinion.criteria = criteria;
        opinion.attribution_tags = tags;
        opinion.confidence = 0.75;
        opinion
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPanelJudge {
    pub id: String,
}

impl Default for StructuralPanelJudge {
    fn default() -> Self {
        Self {
            id: "structural-v1".to_string(),
        }
    }
}

impl PanelJudge for StructuralPanelJudge {
    fn judge_id(&self) -> &str {
        &self.id
    }

    fn family(&self) -> JudgeFamily {
        JudgeFamily::Symbolic
    }

    fn score(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family());

        let schema_ok = input.schema_failures.is_empty();
        let claims_structured = input.claims.iter().all(|c| !c.evidence_refs.is_empty());
        let score = match (schema_ok, claims_structured) {
            (true, true) => 0.9,
            (true, false) => 0.6,
            (false, true) => 0.35,
            (false, false) => 0.15,
        };
        opinion.raw_score = score;
        opinion.criteria.push(ScoredCriterion {
            name: "schema".to_string(),
            score,
            rationale: Some(format!(
                "schema_failures={}, claims_structured={}",
                input.schema_failures.len(),
                claims_structured
            )),
        });
        for s in &input.schema_failures {
            if !s.present {
                opinion
                    .attribution_tags
                    .push(format!("schema_missing:{}", s.field));
            }
        }
        opinion.confidence = 0.85;
        opinion
    }
}

// ───────────────────────────── 评审组 ─────────────────────────────

#[derive(Debug)]
pub struct JudgePanel {
    pub judges: Vec<Box<dyn PanelJudge>>,
    pub debias: DebiasConfig,
}

impl Default for JudgePanel {
    fn default() -> Self {
        Self::default_panel()
    }
}

impl JudgePanel {
    pub fn default_panel() -> Self {
        Self {
            judges: vec![
                Box::new(AnalyticPanelJudge::default()),
                Box::new(EvidencePanelJudge::default()),
                Box::new(StructuralPanelJudge::default()),
            ],
            debias: DebiasConfig::default(),
        }
    }

    pub fn run(&self, input: &JudgeInput) -> PanelVerdict {
        let guard = GuardrailReport::evaluate(input, &self.debias);
        if guard.action == GuardAction::Reject {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: true,
                reasoning: format!("机械检查拒绝, 压过 LLM 分数: {}", guard.reason),
            };
        }
        if guard.action == GuardAction::Quarantine {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Review,
                routed_to_human: true,
                reasoning: format!("幻觉隔离转人工: {}", guard.reason),
            };
        }

        let excluded: Vec<&str> = if self.debias.require_family_separation
            && input.producer_family != JudgeFamily::None
        {
            self.judges
                .iter()
                .filter(|j| j.family() == input.producer_family)
                .map(|j| j.judge_id())
                .collect()
        } else {
            Vec::new()
        };

        let mut opinions: Vec<JudgeOpinion> = Vec::new();
        for judge in &self.judges {
            if excluded.contains(&judge.judge_id()) {
                continue;
            }
            let mut op = judge.score(input);
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }

        self.finalize(opinions, &excluded)
    }

    pub async fn run_async(
        &self,
        input: &JudgeInput,
        async_judges: &[&dyn AsyncPanelJudge],
    ) -> PanelVerdict {
        let guard = GuardrailReport::evaluate(input, &self.debias);
        if guard.action == GuardAction::Reject {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: true,
                reasoning: format!("机械检查拒绝, 压过 LLM 分数: {}", guard.reason),
            };
        }
        if guard.action == GuardAction::Quarantine {
            return PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Review,
                routed_to_human: true,
                reasoning: format!("幻觉隔离转人工: {}", guard.reason),
            };
        }

        let excluded: Vec<&str> = if self.debias.require_family_separation
            && input.producer_family != JudgeFamily::None
        {
            self.judges
                .iter()
                .filter(|j| j.family() == input.producer_family)
                .map(|j| j.judge_id())
                .chain(
                    async_judges
                        .iter()
                        .filter(|j| j.family() == input.producer_family)
                        .map(|j| j.judge_id()),
                )
                .collect()
        } else {
            Vec::new()
        };

        let mut opinions: Vec<JudgeOpinion> = Vec::new();
        for judge in &self.judges {
            if excluded.contains(&judge.judge_id()) {
                continue;
            }
            let mut op = judge.score(input);
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }
        for judge in async_judges {
            if excluded.contains(&judge.judge_id()) {
                continue;
            }
            let mut op = judge.score(input).await;
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }

        self.finalize(opinions, &excluded)
    }

    fn debias_opinion(&self, op: &mut JudgeOpinion, input: &JudgeInput) {
        let family_same =
            input.producer_family != JudgeFamily::None && op.family == input.producer_family;
        let penalty = self.debias.verbosity_penalty_for(&input.candidate);
        let mut debiased = op.raw_score - penalty;
        if family_same {
            debiased -= self.debias.self_preference_penalty;
            op.attribution_tags
                .push("self_preference_penalized".to_string());
        }
        op.debiased_score = debiased.max(0.0).min(1.0);
    }

    fn finalize(&self, opinions: Vec<JudgeOpinion>, excluded: &[&str]) -> PanelVerdict {
        if opinions.is_empty() {
            return PanelVerdict {
                opinions,
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: false,
                reasoning: "无可用法官 (全部被家族分离排除)".to_string(),
            };
        }

        let scores: Vec<f64> = opinions.iter().map(|o| o.debiased_score).collect();
        let median = median_f64(&scores);
        let agreement = agreement(&scores);

        let verdict = if median < self.debias.pass_threshold {
            Verdict::Block
        } else if agreement < self.debias.agreement_review_threshold {
            Verdict::Review
        } else {
            Verdict::Pass
        };

        let routed_to_human = verdict == Verdict::Review || verdict == Verdict::Block;
        let reasoning = format!(
            "median={:.3}, agreement={:.3}, judges={}, excluded={:?}, verdict={:?}",
            median,
            agreement,
            opinions.len(),
            excluded,
            verdict
        );

        PanelVerdict {
            opinions,
            median_score: median,
            agreement,
            verdict,
            routed_to_human,
            reasoning,
        }
    }
}

// ───────────────────────────── 辅助函数 ─────────────────────────────

fn median_f64(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = s.len();
    if n == 0 {
        0.0
    } else if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) / 2.0
    }
}

fn agreement(scores: &[f64]) -> f64 {
    if scores.len() < 2 {
        return 1.0;
    }
    let mut sum = 0.0;
    let mut count = 0usize;
    for i in 0..scores.len() {
        for j in (i + 1)..scores.len() {
            sum += (scores[i] - scores[j]).abs();
            count += 1;
        }
    }
    let mean_abs = sum / count as f64;
    (1.0 - mean_abs).max(0.0).min(1.0)
}

// ───────────────────────────── 护栏 / 裁决 ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelVerdict {
    pub opinions: Vec<JudgeOpinion>,
    pub median_score: f64,
    pub agreement: f64,
    pub verdict: Verdict,
    pub routed_to_human: bool,
    pub reasoning: String,
}

impl PanelVerdict {
    pub fn is_pass(&self) -> bool {
        self.verdict == Verdict::Pass
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailReport {
    pub action: GuardAction,
    pub reason: String,
    pub faithfulness: FaithfulnessReport,
    pub schema_failures: Vec<SchemaCheck>,
    pub grounding_failures: u64,
}

impl GuardrailReport {
    pub fn evaluate(input: &JudgeInput, cfg: &DebiasConfig) -> Self {
        let faith = FaithfulnessReport::audit(&input.claims, &input.evidence_ids);

        if !input.schema_failures.is_empty() {
            return Self {
                action: GuardAction::Reject,
                reason: format!("schema 失败 {} 项", input.schema_failures.len()),
                faithfulness: faith,
                schema_failures: input.schema_failures.clone(),
                grounding_failures: input.grounding_failures,
            };
        }

        if !faith.is_grounded(cfg.grounding_min_ratio) {
            return Self {
                action: GuardAction::Reject,
                reason: format!(
                    "grounding {:.2} < {:.2}",
                    faith.grounding_ratio, cfg.grounding_min_ratio
                ),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: input.grounding_failures,
            };
        }

        if input.grounding_failures > 0 {
            return Self {
                action: GuardAction::Reject,
                reason: format!("{} 次工具 grounding 失败", input.grounding_failures),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: input.grounding_failures,
            };
        }

        if !faith.fabricated.is_empty() {
            return Self {
                action: GuardAction::Quarantine,
                reason: format!("{} 条无证据声明被隔离", faith.fabricated.len()),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: 0,
            };
        }

        Self {
            action: GuardAction::Allow,
            reason: "全部机械检查通过".to_string(),
            faithfulness: faith,
            schema_failures: Vec::new(),
            grounding_failures: 0,
        }
    }
}

// ───────────────────────────── 组合裁决 ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    pub level: GateLevel,
    pub tier: ActionTier,
    pub action: GuardAction,
    pub verdict: Verdict,
    pub reason: String,
}

impl GateDecision {
    pub fn decide(tools: &[ToolSpec], input: &JudgeInput, panel: &JudgePanel) -> Self {
        let tier = ActionTier::classify(tools);
        let level = tier.required_gate();
        let guardrail = GuardrailReport::evaluate(input, &panel.debias);
        let verdict = panel.run(input).verdict;

        if guardrail.action == GuardAction::Reject {
            return Self {
                level: GateLevel::Human,
                tier,
                action: GuardAction::Reject,
                verdict: Verdict::Block,
                reason: format!("机械检查拒绝: {}", guardrail.reason),
            };
        }
        if guardrail.action == GuardAction::Quarantine {
            return Self {
                level: GateLevel::Human,
                tier,
                action: GuardAction::Quarantine,
                verdict: Verdict::Review,
                reason: format!("幻觉隔离升级人工: {}", guardrail.reason),
            };
        }
        if level == GateLevel::Human {
            return Self {
                level,
                tier,
                action: GuardAction::Escalate,
                verdict,
                reason: "路径含不可逆/扩权动作, 强制人工审批 (TNR 无豁免)".to_string(),
            };
        }
        if verdict == Verdict::Review {
            return Self {
                level: GateLevel::Panel,
                tier,
                action: GuardAction::Escalate,
                verdict,
                reason: format!("评审组高分歧 (agreement < 阈值): {:?}", verdict),
            };
        }
        Self {
            level,
            tier,
            action: GuardAction::Allow,
            verdict,
            reason: format!(
                "门控通过: tier={:?}, verdict={:?}, action={:?}",
                tier, verdict, guardrail.action
            ),
        }
    }

    pub fn allows_autonomous(&self) -> bool {
        self.level == GateLevel::Light
            && self.action == GuardAction::Allow
            && self.verdict == Verdict::Pass
    }
}

// 2026-09-27 删除了本文件的 `ToolRegistry` (工具注册表) —— 经核实零消费者:
//   - 本文件 1337 行内无任何代码使用它(仅定义处 + impl)
//   - 全仓 grep 无 `gateway::ToolRegistry` / `gate::ToolRegistry` 引用
//   - 真正在用的是 `nt_core_gate::ToolRegistry` (l5_cognition/nt_core_gate/),
//     见 nt_mind_background_loop/run.rs:4 与 l3_embodiment/nt_shield_enforcer.rs:388
// `ToolSpec` 本身**仍在使用** (本文件 ActionTier::classify 等), 故保留。

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_tier_classify() {
        let tools = vec![ToolSpec::read_only("ls")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier1Autonomous);
    }

    #[test]
    fn test_action_tier_irreversible() {
        let tools = vec![ToolSpec::irreversible("send_email")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier4Human);
    }

    #[test]
    fn test_faithfulness_audit() {
        let claims = vec![Claim::new("test", &["e1"])];
        let evidence = vec!["e1".to_string()];
        let report = FaithfulnessReport::audit(&claims, &evidence);
        assert!(report.is_grounded(0.5));
    }

    #[test]
    fn test_guardrail_reject_low_grounding() {
        let input = JudgeInput::new("test");
        let cfg = DebiasConfig::default();
        let report = GuardrailReport::evaluate(&input, &cfg);
        assert_eq!(report.action, GuardAction::Reject); // no claims = 0 grounding
    }

    #[test]
    fn test_panel_default() {
        let panel = JudgePanel::default_panel();
        assert_eq!(panel.judges.len(), 3);
    }
}
