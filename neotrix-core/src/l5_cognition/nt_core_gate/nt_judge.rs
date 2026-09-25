//! nt_judge — 法官家族/输入意见/rubric/LLM 适配器/注册表.
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};

use crate::l1_action::nt_core_llm::{LlmProvider, LlmRequest};
use crate::l5_cognition::nt_core_prm::{AgentTrajectory, ScoredCriterion};

use super::nt_guardrail::{Claim, SchemaCheck};

/// 法官家族 — 用于跨家族判定 (self-preference / family-bias 控制)。
/// 生成方家族已知时, 同族法官被排除或降权 (Autorubric: GPT-4o/Claude 3.5 同族偏高)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgeFamily {
    /// 分析/推理族 (frontier reasoning lineage)
    Analytic,
    /// 启发式/规则族 (rule-based lineage)
    Heuristic,
    /// 结构化/形式族 (formal, schema-aware lineage)
    Symbolic,
    /// 未知/无族属
    None,
}

/// 一维评审标准 — Replica 式 per-task rubric 的单个维度。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub name: String,
    pub description: String,
    /// 锚定评分范例 (Replica #10: 每维 0.0/0.5/1.0 具体反例锚定分数, 显著降 judge 噪声)。
    /// 按低→高排列的 <分数, 范例> 对; 缺省空 = 无锚定 (向后兼容)。
    #[serde(default)]
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

    pub fn with_examples(mut self, examples: Vec<(f64, String)>) -> Self {
        self.score_examples = examples;
        self
    }
}

/// 任务专属 rubric 规格 (Replica 方法论映射: 自动生成五维 rubric, 低噪声奖励)。
/// 五维 (Replica 五维 → NeoTrix 评审五维):
///   claim_support   — 产出是否支撑候选论断 (对应 Replica "支撑科学论断")
///   evidence_ground — 证据接地, 无幻觉 (对应 Replica "科学诚信/反作弊")
///   mechanism       — 实现真实机制而非硬编码/代理 (对应 Replica "实现实验设计的机制")
///   resource        — 计算/预算利用合理 (对应 Replica "预算利用")
///   fidelity        — 与目标/论文的忠实度 (对应 Replica "图形相似")
/// 缺省 (None) 时 LLM 法官 fallback 现有 1-4 综合分提示词, 向后兼容。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricSpec {
    pub dimensions: Vec<RubricCriterion>,
}

impl RubricSpec {
    /// 缺省五维 rubric — Replica 五维的 NeoTrix 评审映射。
    pub fn default_five() -> Self {
        Self {
            dimensions: vec![
                RubricCriterion {
                    name: "claim_support".into(),
                    description: "产出是否支撑候选论断".into(),
                    score_examples: vec![
                        (0.0, "产出与候选论断无关或相矛盾".into()),
                        (0.5, "产出部分支撑论断但关键环节缺失".into()),
                        (1.0, "产出完整支撑论断, 无遗漏".into()),
                    ],
                },
                RubricCriterion {
                    name: "evidence_ground".into(),
                    description: "证据接地, 无幻觉, 不伪造".into(),
                    score_examples: vec![
                        (0.0, "引用不存在的证据或捏造结果".into()),
                        (0.5, "证据相关但未直接验证断言".into()),
                        (1.0, "每个断言均有可核验证据支撑".into()),
                    ],
                },
                RubricCriterion {
                    name: "mechanism".into(),
                    description: "实现真实机制而非硬编码或代理".into(),
                    score_examples: vec![
                        (0.0, "硬编码期望输出绕过机制".into()),
                        (0.5, "实现了部分机制但走捷径".into()),
                        (1.0, "实现并执行了完整真实机制".into()),
                    ],
                },
                RubricCriterion {
                    name: "resource".into(),
                    description: "计算/预算利用合理, 无浪费".into(),
                    score_examples: vec![
                        (0.0, "耗尽预算却无实质进展或空转填充".into()),
                        (0.5, "预算利用部分合理但有关键浪费".into()),
                        (1.0, "预算使用与目标成比例, 无浪费".into()),
                    ],
                },
                RubricCriterion {
                    name: "fidelity".into(),
                    description: "与目标/论文的忠实度".into(),
                    score_examples: vec![
                        (0.0, "声称复现但未忠实于目标设定".into()),
                        (0.5, "整体忠实但有偏离目标之处".into()),
                        (1.0, "严格忠实目标设定与实验范围".into()),
                    ],
                },
            ],
        }
    }
}

/// 自证 — 产物附带生成方对真实执行切片的声明 (Replica #9: 要求报告真实执行切片,
/// 声称与观测相矛盾则得分砍半)。声明项可为 None = 未自证, 视为轻微扣分而非矛盾。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// 声称真实执行过的代码/工作切片段 id (可空 — 未自证)
    pub real_slice: Option<String>,
    /// 声称使用的预算 (可空 — 未自证)
    pub budget_used: Option<f64>,
    /// 本轮实际总预算 (观测值)
    pub budget_total: f64,
    /// 声称的种子/采样数 (可空 — 未自证)
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

    /// 与观测比对: 声明与观测矛盾 → 0.5 扣分; 未自证 (声明为 None) → 0.1 轻微扣分;
    /// 一致 → 无扣分。返回 0..1 的矛盾惩罚系数。
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

/// 门控输入 — 候选产物 + 声明 + 证据 + 轨迹 + 机械检查失败计数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeInput {
    pub candidate: String,
    pub claims: Vec<Claim>,
    pub evidence_ids: Vec<String>,
    /// 可选轨迹 — 有则评委加入 soundness 维度 (轨迹 vs 仅最终答案)
    pub trajectory: Option<AgentTrajectory>,
    /// 工具调用声称成功但实际失败的次数 (self_audit 计数)
    pub grounding_failures: u64,
    /// 已检测的 schema 失败 (可预填; 空则运行中检查)
    pub schema_failures: Vec<SchemaCheck>,
    /// 生成方家族 — 用于自我偏好排除
    pub producer_family: JudgeFamily,
    /// 可选 rubric — 有则五维分评 (Replica: per-task rubric judge); 缺省 fallback 综合分
    #[serde(default)]
    pub rubric: Option<RubricSpec>,
    /// LLM 法官采样次数 — >1 时多次调用取均值降噪 (Replica: 3-sample aggregation)
    #[serde(default)]
    pub samples: u8,
    /// 生成方自证 (Replica #9) — 无则未启用自证纪律
    #[serde(default)]
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

/// 单个法官的意见 — 原始 + 去偏后分数。
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
    pub(crate) fn new(judge_id: &str, family: JudgeFamily) -> Self {
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

/// 法官接口 — 每个法官来自一个家族, 用不同判据评分。
pub trait PanelJudge: Send + Sync + std::fmt::Debug {
    fn judge_id(&self) -> &str;
    fn family(&self) -> JudgeFamily;
    fn score(&self, input: &JudgeInput) -> JudgeOpinion;
}

/// 异步法官 — 复用 `LlmProvider` 的 real LLM 评审 (FPAM 缺口: 无真实 LLM 法官)。
/// `JudgePanel::run_async` 将异步法官与同步启发式法官共同聚合, 消除"假评委"。
#[async_trait::async_trait]
pub trait AsyncPanelJudge: Send + Sync + std::fmt::Debug {
    fn judge_id(&self) -> &str;
    fn family(&self) -> JudgeFamily;
    async fn score(&self, input: &JudgeInput) -> JudgeOpinion;
}

/// 剥离 markdown 代码块围栏: LLM 常返回 ```json ... ``` (实测 llm7/codestral 命中),
/// 直接 serde 解析会失败 → 误入 llm_parse_failed 低分路径。
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

/// LLM 法官适配器 — 把候选结论 + 声明 + 证据喂给 LLM, 要求返回结构化评分。
///
/// 提示词按 FutureAGI 小量表 1-4 出分; 用 structured_output JsonObject 强制 JSON;
/// 解析失败 → 低分 + 明确 attribution tag, 不 panic (确定性优先, LLM 只是打分器)。
pub struct LLMJudgeAdapter {
    pub id: String,
    pub family: JudgeFamily,
    pub provider: std::sync::Arc<dyn LlmProvider>,
    pub model: String,
    /// 采样次数 — >1 时多次调用 LLM 取均值降噪 (Replica: 3-sample aggregation)
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
        let base = format!(
            "你是公正评审组的一名评委。\n\
             [候选结论] {}\n\
             [声明]\n{}\n\
             [证据 id] {}\n\
             [已记录 grounding 失败] {}\n",
            input.candidate,
            claims.join("\n"),
            input.evidence_ids.join(", "),
            input.grounding_failures,
        );
        match &input.rubric {
            Some(rubric) => {
                let dims: Vec<String> = rubric
                    .dimensions
                    .iter()
                    .map(|d| {
                        let anchors: Vec<String> = d
                            .score_examples
                            .iter()
                            .map(|(s, e)| format!("{}分: {}", s, e))
                            .collect();
                        if anchors.is_empty() {
                            format!("  \"{}\": \"{}\",", d.name, d.description)
                        } else {
                            format!(
                                "  \"{}\": \"{}\" (锚定范例: {})",
                                d.name,
                                d.description,
                                anchors.join(" | "),
                            )
                        }
                    })
                    .collect();
                format!(
                    "{}\n\
                     按以下 rubric 维度逐维评分 (每维 0..1), 并给综合分。\n\
                     [rubric 维度]\n{}\n\
                     输出严格 JSON: {{\"score\": <0..1>, \"confidence\": <0..1>, \"rationale\": \"<一句话>\", \
                     \"criteria\": [{{\"name\": \"<维度名>\", \"score\": <0..1>, \"rationale\": \"<理由>\"}}]}}",
                    base,
                    dims.join("\n"),
                )
            }
            None => format!(
                "{}\n\
                 请按 1-4 档 (1=完全不可信, 4=可信) 只针对[候选结论]给出综合分。\n\
                 输出严格 JSON: {{\"score\": <1..4>, \"confidence\": <0..1>, \"rationale\": \"<一句话>\"}}",
                base,
            ),
        }
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
            let mut acc: Vec<ScoredCriterion> = Vec::new();
            let all: Vec<ScoredCriterion> = std::iter::once(&opinion)
                .chain(opinions.iter())
                .flat_map(|o| o.criteria.iter().cloned())
                .collect();
            for name in [
                "llm_judge",
                "claim_support",
                "evidence_ground",
                "mechanism",
                "resource",
                "fidelity",
            ] {
                let grouped: Vec<&ScoredCriterion> =
                    all.iter().filter(|c| c.name == name).collect();
                if grouped.is_empty() {
                    continue;
                }
                let s = grouped.iter().map(|c| c.score).sum::<f64>() / grouped.len() as f64;
                let rationale = grouped
                    .iter()
                    .find_map(|c| c.rationale.clone())
                    .unwrap_or_else(|| format!("{} 维 {} 采样均值", name, grouped.len()));
                acc.push(ScoredCriterion {
                    name: name.to_string(),
                    score: s.max(0.0).min(1.0),
                    rationale: Some(rationale),
                });
            }
            opinion.criteria = acc;
            opinion
                .attribution_tags
                .push(format!("multi_sample_{}", samples));
        }
        opinion
    }

    async fn single_sample(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family);
        // rubric 五维+criteria 输出体积大, 512 易截断 → rubric 模式放宽到 900
        let max_tokens = if input.rubric.is_some() { 900 } else { 512 };
        let request = LlmRequest::new(&self.model, &self.prompt(input))
            .with_temperature(Some(0.2))
            .with_max_tokens(max_tokens)
            .with_structured_output(crate::l1_action::nt_core_llm::StructuredOutputConfig::JsonObject);
        let response = self.provider.complete(&request).await;
        match response {
            Ok(resp) => {
                // 真实 LLM 常以 ```json 代码块包裹 (实测 llm7/codestral 命中),
                // 直接 from_str 会误入 llm_parse_failed → 剥离围栏再解析
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
                        let score = if input.rubric.is_some() {
                            raw.max(0.0).min(1.0)
                        } else {
                            (raw.max(1.0).min(4.0) - 1.0) / 3.0
                        };
                        opinion.raw_score = score.max(0.0).min(1.0);
                        opinion.confidence = conf.max(0.0).min(1.0);
                        opinion.criteria.push(ScoredCriterion {
                            name: "llm_judge".to_string(),
                            score,
                            rationale: Some(rationale),
                        });
                        if let Some(criteria) = v.get("criteria").and_then(|c| c.as_array()) {
                            for c in criteria {
                                let name = c
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let cs = c.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                                let cr = c
                                    .get("rationale")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                opinion.criteria.push(ScoredCriterion {
                                    name,
                                    score: cs.max(0.0).min(1.0),
                                    rationale: Some(cr),
                                });
                            }
                        }
                        opinion.attribution_tags.push("llm_provider".to_string());
                    }
                    Err(_) => {
                        // JSON 解析失败 → 低分 + 明确标记, 不 panic
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
                // provider 调用失败 → 低分 + 标记, 让聚合把它当异常信号
                opinion.raw_score = 0.3;
                opinion.confidence = 0.0;
                opinion.attribution_tags.push("llm_call_failed".to_string());
            }
        }
        opinion
    }
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

/// 多 provider 法官注册表 — 家族 → (provider, model) 映射, 评审组从注册表组装。
/// 每个家族独立 provider/model, 避免同源 LLM 单点与同族共谋 (家庭分离的 provider 级实现)。
#[derive(Debug, Clone, Default)]
pub struct JudgeRegistry {
    entries: Vec<JudgeEntry>,
}

pub struct JudgeEntry {
    pub id: String,
    pub family: JudgeFamily,
    pub provider: std::sync::Arc<dyn LlmProvider>,
    pub model: String,
}

impl std::fmt::Debug for JudgeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JudgeEntry")
            .field("id", &self.id)
            .field("family", &self.family)
            .field("model", &self.model)
            .finish()
    }
}

impl Clone for JudgeEntry {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            family: self.family,
            provider: self.provider.clone(),
            model: self.model.clone(),
        }
    }
}

impl JudgeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        mut self,
        family: JudgeFamily,
        provider: std::sync::Arc<dyn LlmProvider>,
        model: &str,
    ) -> Self {
        let id = format!("llm-{:?}-{}", family, self.entries.len());
        self.entries.push(JudgeEntry {
            id,
            family,
            provider,
            model: model.to_string(),
        });
        self
    }

    pub fn entries(&self) -> &[JudgeEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn build_async_judges(&self) -> Vec<Box<dyn AsyncPanelJudge>> {
        self.entries
            .iter()
            .map(|e| {
                Box::new(LLMJudgeAdapter {
                    id: e.id.clone(),
                    family: e.family,
                    provider: e.provider.clone(),
                    model: e.model.clone(),
                    samples: 1,
                }) as Box<dyn AsyncPanelJudge>
            })
            .collect()
    }
}
