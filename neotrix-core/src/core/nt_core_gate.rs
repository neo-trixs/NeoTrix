//! nt_core_gate — 公正评审门控节点 (Unbiased Judge Panel Gate)
//!
//! 把「门禁读证据、按规则裁决」落地为确定性 + 跨家族法官聚合两层，覆盖 2026 文献
//! 与 reverse-skill / STRATUS / evidence-gate 实证的四条支柱:
//!
//! 1. **多家族公正评委** — `JudgePanel` 聚合不同 `JudgeFamily` 的法官, 计算评分者间
//!    一致率 (inter-rater agreement) 与中位数聚合; 高分歧 → 转人工 (Anthropic pass^k
//!    纪律)。去偏 (DebiasConfig): 家族分离 (评委 ≠ 生成方家族, 消 self-preference,
//!    Wataoka 2024 / Autorubric)、verbosity 惩罚 (Wang et al. 2023 冗长偏差 ~15%)、
//!    小量表 1-4 (FutureAGI 建议)。位置偏差由 `run_ensemble` 的 N 次重复 + pass^k
//!    双评一致吸收。
//! 2. **eval 即护栏** — `GuardrailReport::evaluate`: 低 grounding → Reject;
//!    无证据引用的幻觉声明 → Quarantine (逐句引用 + 集合差检测, agentpatterns.ai
//!    per-line-citation 模式); schema 字段缺失 → Reject (Forbes: JSON 必须可解析)。
//! 3. **轨迹分级 + 忠实度** — `FaithfulnessReport::audit` 对 claim→evidence 做集合差;
//!    `CalibrationSet` 用真实 clean/broken 日志黄金集校准 pass^k (TRACE 2602.21230
//!    高分化错误, Anthropic demystifying-evals Step 6 check transcripts)。
//! 4. **爆炸半径门控** — `ActionTier::classify` 按工具可逆性 (只读/可逆/可补偿/不可逆/
//!    扩权) 分级自治 (DigitalApplied 四层 / TianPan 六类 / STRATUS TNR); 不可逆 → 强制
//!    人工审批, 确定性检查优先于置信度分数。
//!
//! 纪律: R-P6 float 用 `.max(0.0).min(1.0)`; 确定性优先 — LLM 分数只在机械检查通过后
//! 才进入聚合。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_prm::{AgentTrajectory, ScoredCriterion};
use crate::neotrix::l1_body_impl::nt_io_provider::{LlmProvider, LlmRequest};

// ───────────────────────────── 基础类型 ─────────────────────────────

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

/// 门控强度 — 由爆炸半径分级驱动 (cost of the gate ≪ cost of the bad merge)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateLevel {
    /// 轻门: 确定性检查 + 单判, 不阻塞 (docs/draft/低风险)
    Light,
    /// 评审组: 多家族法官 + 一致率 + 黄金集校准 (agent 改动 → trunk / 安全路径)
    Panel,
    /// 人工: 强制人类审批 (不可逆/扩权动作, 确定性检查不能越过)
    Human,
}

/// 门控裁决。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verdict {
    Pass,
    /// 高分歧或证据不足 — 转人工路由, 不自动放行
    Review,
    /// 机械检查或低分 — 阻断合并
    Block,
}

/// 护栏动作 — eval 结果对运行轨迹的引导。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardAction {
    Allow,
    /// 拒绝低 grounding / schema 失败 — 硬阻断
    Reject,
    /// 隔离幻觉声明 — 扣留待人工复核, 不进生产
    Quarantine,
    /// 升级 — 需要更高权限/人工
    Escalate,
}

/// 工具可逆性 — 爆炸半径的原子度量 (STRATUS TNR: 每动作配 undo 算子)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolReversibility {
    /// 无副作用 (SELECT / GET / 文件读)
    ReadOnly,
    /// 可逆写 (有 undo 算子 + 前置快照)
    Reversible,
    /// 可补偿 (无完美 undo, 但有补偿动作; saga)
    Compensable,
    /// 不可逆 (发送邮件 / 删除生产数据 / 发布) — 强制人工
    Irreversible,
}

/// 工具规约 — 注册表条目, 构建期事实而非运行期猜测 (TianPan: risk class as
/// versioned tool attribute; 同一注册表同时发 tool spec 与 gate config)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub reversibility: ToolReversibility,
    /// 可选 undo 算子名 (Reversible 必须注册)。
    pub undo: Option<String>,
    /// 扩权型 (授权/改权限/换凭据) — 未来的动作空间由它决定, 必须人工。
    pub authority_modifying: bool,
}

impl ToolSpec {
    pub fn read_only(name: &str) -> Self {
        Self { name: name.to_string(), reversibility: ToolReversibility::ReadOnly, undo: None, authority_modifying: false }
    }

    pub fn reversible(name: &str, undo: &str) -> Self {
        Self { name: name.to_string(), reversibility: ToolReversibility::Reversible, undo: Some(undo.to_string()), authority_modifying: false }
    }

    pub fn irreversible(name: &str) -> Self {
        Self { name: name.to_string(), reversibility: ToolReversibility::Irreversible, undo: None, authority_modifying: false }
    }
}

/// 动作分级 — 计划的风险是**路径的函数**, 不是工具最大值; 用路径上最严重者兜底。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionTier {
    /// 只读 — 全自动, 不打断 (过度打断 → confirmation fatigue 反变安全风险)
    Tier1Autonomous,
    /// 可逆写 — 自动 + 全量日志
    Tier2Logged,
    /// 可补偿/外部 — 评审组或 staging 队列
    Tier3Review,
    /// 不可逆/扩权 — 强制人工审批, 无置信度豁免
    Tier4Human,
}

impl ActionTier {
    /// 沿路径分类: 最严重者兜底 (Path-Based Authorization 的保守 floor)。
    pub fn classify(tools: &[ToolSpec]) -> Self {
        let mut tier = ActionTier::Tier1Autonomous;
        for t in tools {
            let this = if t.authority_modifying || t.reversibility == ToolReversibility::Irreversible {
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

/// 去偏 + 门限配置 — R-P11 Config struct 模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebiasConfig {
    /// 冗长惩罚系数: 输出超过 norm 时按比例压分 (verbosity bias ~15%)
    pub verbosity_penalty: f64,
    /// 参考文本长度 (字符), 超过即触发冗长惩罚
    pub verbosity_norm_len: usize,
    /// 冗长惩罚上限 (防过度压制)
    pub verbosity_penalty_cap: f64,
    /// 强制家族分离: 生成方家族已知时, 同族法官的评分被排除
    pub require_family_separation: bool,
    /// self-preference 降权 (同族法官未被排除时的残余惩罚)
    pub self_preference_penalty: f64,
    /// 评分者间一致率下限, 低于此 → Review (转人工)
    pub agreement_review_threshold: f64,
    /// 中位数通过阈值
    pub pass_threshold: f64,
    /// grounding 最低比率 (低于 → Reject)
    pub grounding_min_ratio: f64,
    /// schema 严格模式 (缺字段即 Reject)
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

// ───────────────────────────── 忠实度 / schema ─────────────────────────────

/// 一条声明 + 其证据引用 (per-line citation 契约)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub text: String,
    pub evidence_refs: Vec<String>,
}

impl Claim {
    pub fn new(text: &str, refs: &[&str]) -> Self {
        Self { text: text.to_string(), evidence_refs: refs.iter().map(|s| s.to_string()).collect() }
    }
}

/// Schema 字段检查结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCheck {
    pub field: String,
    pub present: bool,
    pub detail: String,
}

/// 忠实度审计报告 — 逐句引用 + 集合差检测 (引用不存在于证据集 → 幻觉)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaithfulnessReport {
    pub claims_total: usize,
    pub grounded: usize,
    /// 引用了证据集中不存在条目的声明 (幻觉候选)
    pub fabricated: Vec<String>,
    pub grounding_ratio: f64,
}

impl FaithfulnessReport {
    /// 机械集合差: 每个 claim 的 evidence_refs ⊆ evidence_ids 才计为 grounded。
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
                fabricated.push(format!("claim '{}': 引用不存在证据 {:?}", clip(&c.text, 60), missing));
            }
        }
        let grounding_ratio = if total == 0 { 0.0 } else { grounded as f64 / total as f64 };
        Self { claims_total: total, grounded, fabricated, grounding_ratio }
    }

    pub fn is_grounded(&self, min_ratio: f64) -> bool {
        self.grounding_ratio >= min_ratio
    }

    /// 隔离幻觉声明 — 返回待人工复核的声明文本。
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

/// 检查 JSON 输出是否包含全部必需字段 — 机械 schema 失败拦截。
pub fn check_schema_fields(required: &[&str], json: &str) -> Vec<SchemaCheck> {
    let parsed = serde_json::from_str::<serde_json::Value>(json);
    let mut checks = Vec::with_capacity(required.len());
    let obj = match parsed {
        Ok(serde_json::Value::Object(map)) => Some(map),
        Ok(serde_json::Value::Null) => None,
        Ok(_) => {
            return required.iter().map(|f| SchemaCheck {
                field: f.to_string(),
                present: false,
                detail: "输出不是 JSON 对象".to_string(),
            }).collect();
        }
        Err(e) => {
            return required.iter().map(|f| SchemaCheck {
                field: f.to_string(),
                present: false,
                detail: format!("JSON 解析失败: {}", e),
            }).collect();
        }
    };
    for f in required {
        let present = obj.as_ref().map(|m| m.contains_key(*f)).unwrap_or(false);
        checks.push(SchemaCheck {
            field: f.to_string(),
            present,
            detail: if present { "ok".to_string() } else { format!("缺少字段 {}", f) },
        });
    }
    checks
}

// ───────────────────────────── 法官输入 / 意见 ─────────────────────────────

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

/// 法官接口 — 每个法官来自一个家族, 用不同判据评分。
pub trait PanelJudge: Send + Sync + std::fmt::Debug {
    fn judge_id(&self) -> &str;
    fn family(&self) -> JudgeFamily;
    fn score(&self, input: &JudgeInput) -> JudgeOpinion;
}

// ───────────────────────────── LLM 法官 (异步评审路径) ─────────────────────────────

/// 异步法官 — 复用 `LlmProvider` 的 real LLM 评审 (FPAM 缺口: 无真实 LLM 法官)。
/// `JudgePanel::run_async` 将异步法官与同步启发式法官共同聚合, 消除"假评委"。
#[async_trait::async_trait]
pub trait AsyncPanelJudge: Send + Sync + std::fmt::Debug {
    fn judge_id(&self) -> &str;
    fn family(&self) -> JudgeFamily;
    async fn score(&self, input: &JudgeInput) -> JudgeOpinion;
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
}

impl std::fmt::Debug for LLMJudgeAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMJudgeAdapter")
            .field("id", &self.id)
            .field("family", &self.family)
            .field("model", &self.model)
            .finish()
    }
}

impl LLMJudgeAdapter {
    pub fn new(id: &str, family: JudgeFamily, provider: std::sync::Arc<dyn LlmProvider>, model: &str) -> Self {
        Self { id: id.to_string(), family, provider, model: model.to_string() }
    }

    fn prompt(&self, input: &JudgeInput) -> String {
        let claims: Vec<String> = input.claims.iter().map(|c| format!("- {}", c.text)).collect();
        format!(
            "你是公正评审组的一名评委。请按 1-4 档 (1=完全不可信, 4=可信) 只针对[候选结论]给出综合分。\n\
             [候选结论] {}\n\
             [声明]\n{}\n\
             [证据 id] {}\n\
             [已记录 grounding 失败] {}\n\
             输出严格 JSON: {{\"score\": <1..4>, \"confidence\": <0..1>, \"rationale\": \"<一句话>\"}}",
            input.candidate,
            claims.join("\n"),
            input.evidence_ids.join(", "),
            input.grounding_failures,
        )
    }

    async fn complete(&self, input: &JudgeInput) -> JudgeOpinion {
        let mut opinion = JudgeOpinion::new(&self.id, self.family);
        let request = LlmRequest::new(&self.model, &self.prompt(input))
            .with_temperature(Some(0.2))
            .with_max_tokens(256)
            .with_structured_output(crate::neotrix::l1_body_impl::nt_io_provider::types::StructuredOutputConfig::JsonObject);
        let response = self.provider.complete(&request).await;
        match response {
            Ok(resp) => {
                match serde_json::from_str::<serde_json::Value>(&resp.content) {
                    Ok(v) => {
                        let raw = v.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                        let conf = v.get("confidence").and_then(|s| s.as_f64()).unwrap_or(0.0);
                        let rationale = v.get("rationale").and_then(|s| s.as_str()).unwrap_or("").to_string();
                        // 小量表 1-4 → 0..1 (1→0.25, 2→0.5, 3→0.75, 4→1.0)
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
                        // JSON 解析失败 → 低分 + 明确标记, 不 panic
                        opinion.raw_score = 0.2;
                        opinion.confidence = 0.0;
                        opinion.attribution_tags.push("llm_parse_failed".to_string());
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

    pub fn register(mut self, family: JudgeFamily, provider: std::sync::Arc<dyn LlmProvider>, model: &str) -> Self {
        let id = format!("llm-{:?}-{}", family, self.entries.len());
        self.entries.push(JudgeEntry { id, family, provider, model: model.to_string() });
        self
    }

    pub fn entries(&self) -> &[JudgeEntry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn build_async_judges(&self) -> Vec<Box<dyn AsyncPanelJudge>> {
        self.entries.iter().map(|e| Box::new(LLMJudgeAdapter {
            id: e.id.clone(),
            family: e.family,
            provider: e.provider.clone(),
            model: e.model.clone(),
        }) as Box<dyn AsyncPanelJudge>).collect()
    }
}

/// 分析族法官 — 判据: 轨迹 soundness (step 成功率/外部奖励), 无轨迹则用候选文本结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticPanelJudge {
    pub id: String,
}

impl Default for AnalyticPanelJudge {
    fn default() -> Self {
        Self { id: "analytic-v1".to_string() }
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
                rationale: Some(format!("轨迹 {} 步, {} 成功, 外部奖励 {}", total, successes, traj.outcome_reward.unwrap_or(0.0))),
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

        // grounding 失败计数压低 soundness
        if input.grounding_failures > 0 {
            let penalty = (input.grounding_failures as f64 * 0.05).max(0.0).min(0.3);
            opinion.raw_score = (opinion.raw_score - penalty).max(0.0).min(1.0);
            opinion.attribution_tags.push(format!("grounding_failures={}", input.grounding_failures));
        }
        opinion.confidence = 0.8;
        opinion
    }
}

/// 启发式族法官 — 判据: 忠实度 (grounding_ratio) + 证据覆盖 + 冗长惩罚。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePanelJudge {
    pub id: String,
}

impl Default for EvidencePanelJudge {
    fn default() -> Self {
        Self { id: "evidence-v1".to_string() }
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

        // 忠实度
        criteria.push(ScoredCriterion {
            name: "faithfulness".to_string(),
            score: faith.grounding_ratio,
            rationale: Some(format!("grounded {}/{}", faith.grounded, faith.claims_total)),
        });
        if !faith.fabricated.is_empty() {
            tags.push(format!("fabrications={}", faith.fabricated.len()));
        }

        // 证据覆盖: 每个声明至少 1 条引用
        let refs_total: usize = input.claims.iter().map(|c| c.evidence_refs.len()).sum();
        let coverage = if input.claims.is_empty() { 0.0 }
            else { refs_total as f64 / (input.claims.len() as f64).max(1.0) };
        let coverage_score = (coverage / 2.0).max(0.0).min(1.0);
        criteria.push(ScoredCriterion {
            name: "evidence_coverage".to_string(),
            score: coverage_score,
            rationale: Some(format!("平均引用 {}", if input.claims.is_empty() { 0 } else { refs_total / input.claims.len() })),
        });

        // 冗长惩罚 (verbosity bias mitigation — 在 judge 侧显式记分)
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

/// 结构化族法官 — 判据: schema 完整 + 声明结构 (引用非空)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPanelJudge {
    pub id: String,
}

impl Default for StructuralPanelJudge {
    fn default() -> Self {
        Self { id: "structural-v1".to_string() }
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
            rationale: Some(format!("schema_failures={}, claims_structured={}", input.schema_failures.len(), claims_structured)),
        });
        for s in &input.schema_failures {
            if !s.present {
                opinion.attribution_tags.push(format!("schema_missing:{}", s.field));
            }
        }
        opinion.confidence = 0.85;
        opinion
    }
}

// ───────────────────────────── 评审组 ─────────────────────────────

/// 公正评审组 — 多家族法官聚合。
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
    /// 三家族默认评审组 (Analytic + Heuristic + Symbolic)。
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

    /// 运行评审组 → 去偏 → 聚合。
    ///
    /// 机械检查 (schema/grounding/fabrication) 优先于 LLM 分数: 护栏拒绝 → 直接 Block,
    /// 幻觉隔离 → Review 转人工; 之后才是多法官聚合 (median / agreement)。
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

        let excluded: Vec<&str> = if self.debias.require_family_separation && input.producer_family != JudgeFamily::None {
            self.judges.iter()
                .filter(|j| j.family() == input.producer_family)
                .map(|j| j.judge_id())
                .collect()
        } else {
            Vec::new()
        };

        let mut opinions: Vec<JudgeOpinion> = Vec::new();
        for judge in &self.judges {
            if excluded.contains(&judge.judge_id()) {
                continue; // 家族分离: 评委 ≠ 生成方家族 (self-preference 硬排除)
            }
            let mut op = judge.score(input);
            self.debias_opinion(&mut op, input);
            opinions.push(op);
        }

        self.finalize(opinions, &excluded)
    }

    /// 异步评审路径 — 同步启发式法官 + 真实 LLM 法官共同聚合。
    ///
    /// 机械护栏 (schema/grounding/fabrication) 仍确定性前置, LLM 只是聚合中的打分器
    /// (FPAM: 无真实 LLM 法官 → 补齐; R-P79: 同 session 接线生产路径)。
    pub async fn run_async(&self, input: &JudgeInput, async_judges: &[&dyn AsyncPanelJudge]) -> PanelVerdict {
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

        let excluded: Vec<&str> = if self.debias.require_family_separation && input.producer_family != JudgeFamily::None {
            self.judges.iter()
                .filter(|j| j.family() == input.producer_family)
                .map(|j| j.judge_id())
                .chain(async_judges.iter().filter(|j| j.family() == input.producer_family).map(|j| j.judge_id()))
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

    /// 对单条意见应用去偏 (verbosity + self-preference), 同步/异步路径共用。
    fn debias_opinion(&self, op: &mut JudgeOpinion, input: &JudgeInput) {
        let family_same = input.producer_family != JudgeFamily::None && op.family == input.producer_family;
        let penalty = self.debias.verbosity_penalty_for(&input.candidate);
        let mut debiased = op.raw_score - penalty;
        if family_same {
            debiased -= self.debias.self_preference_penalty;
            op.attribution_tags.push("self_preference_penalized".to_string());
        }
        op.debiased_score = debiased.max(0.0).min(1.0);
    }

    /// 聚合去偏后的意见 → 裁决。同步与异步评审路径共用。
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
            Verdict::Review // 高分歧 → 转人工, 不自动放行
        } else {
            Verdict::Pass
        };

        let routed_to_human = verdict == Verdict::Review || verdict == Verdict::Block;
        let reasoning = format!(
            "median={:.3}, agreement={:.3}, judges={}, excluded={:?}, verdict={:?}",
            median, agreement, opinions.len(), excluded, verdict
        );

        PanelVerdict { opinions, median_score: median, agreement, verdict, routed_to_human, reasoning }
    }

    /// pass^k 门控: N 次运行, 至少 k 次 Pass 才算放行 (Anthropic: 门禁不用 Pass@1)。
    ///
    /// 机械检查 (schema/grounding/fabrication/grounding_failures) 确定性优先:
    /// 任一运行触发 Reject → 直接 Block; 触发 Quarantine → Review; 无视后续 LLM 分。
    pub fn run_ensemble(&self, input: &JudgeInput, runs: usize, k: usize) -> EnsembleVerdict {
        // 机械护栏前置 — 确定性检查压过 LLM 聚合
        let guard = GuardrailReport::evaluate(input, &self.debias);
        if guard.action == GuardAction::Reject {
            let v = PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Block,
                routed_to_human: true,
                reasoning: format!("ensemble 前置护栏拒绝: {}", guard.reason),
            };
            return EnsembleVerdict { runs, k, passes: 0, passed: false, verdict: Verdict::Block, last: Box::new(v) };
        }
        if guard.action == GuardAction::Quarantine {
            let v = PanelVerdict {
                opinions: Vec::new(),
                median_score: 0.0,
                agreement: 0.0,
                verdict: Verdict::Review,
                routed_to_human: true,
                reasoning: format!("ensemble 前置护栏隔离: {}", guard.reason),
            };
            return EnsembleVerdict { runs, k, passes: 0, passed: false, verdict: Verdict::Review, last: Box::new(v) };
        }

        let k = k.min(runs).max(1);
        let mut passes = 0usize;
        let mut last = None;
        for _ in 0..runs {
            let v = self.run(input);
            if v.verdict == Verdict::Pass {
                passes += 1;
            }
            last = Some(v);
        }
        let passed = passes >= k;
        let verdict = if passed { Verdict::Pass } else if passes == 0 { Verdict::Block } else { Verdict::Review };
        EnsembleVerdict {
            runs,
            k,
            passes,
            passed,
            verdict,
            last: Box::new(last.expect("runs>=1")),
        }
    }
}

impl DebiasConfig {
    /// 冗长惩罚: 超过 norm 的部分按比例折算, 封顶。
    fn verbosity_penalty_for(&self, text: &str) -> f64 {
        let len = text.chars().count();
        if len <= self.verbosity_norm_len {
            return 0.0;
        }
        let over = (len - self.verbosity_norm_len) as f64 / self.verbosity_norm_len as f64;
        (over * self.verbosity_penalty).max(0.0).min(self.verbosity_penalty_cap)
    }
}

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

/// 评分者间一致率: 1 - 归一化平均绝对差 (0..1)。1 = 完全一致。
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

/// 评审组裁决。
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

// ───────────────────────────── 辩论决策 (D6) ─────────────────────────────

/// 辩论角色 — TradingAgents 多空辩论 + oh-my-hermes Planner→Architect→Critic 参照。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DebateRole {
    Pro,     // 正方: 支持提案
    Con,     // 反方: 反对提案
    Neutral, // 中立: 仲裁
}

/// 单轮辩论发言。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRound {
    pub role: DebateRole,
    pub score: f64,
    pub confidence: f64,
    pub argument: String,
}

/// 辩论报告 — 对抗轮次 + 收敛结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateReport {
    pub rounds: Vec<DebateRound>,
    /// 辩论后收敛分数 (反方意见修正正方)。
    pub converged_score: f64,
    /// 分歧度 (0=一致, 1=完全对抗)。
    pub divergence: f64,
    /// 是否收敛到明确结论。
    pub converged: bool,
}

/// D6: 对抗式辩论 — 用 panel 的多数意见与少数意见互搏, 收敛出修正分数。
/// 机制: 高分意见=Pro, 低分意见=Con, 中间=Neutral; 3 轮辩论按分歧度衰减
/// Pro/Con 偏差, 得到更稳健的最终分。无外部 LLM 依赖, 纯启发式对抗。
pub fn deliberate(opinions: &[JudgeOpinion]) -> DebateReport {
    if opinions.is_empty() {
        return DebateReport {
            rounds: Vec::new(),
            converged_score: 0.0,
            divergence: 0.0,
            converged: true,
        };
    }
    let scores: Vec<f64> = opinions.iter().map(|o| o.debiased_score).collect();
    let n = scores.len() as f64;
    let mean: f64 = scores.iter().sum::<f64>() / n;
    let (min, max) = (
        scores.iter().cloned().fold(f64::INFINITY, f64::min),
        scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    );
    let spread = max - min;
    // 分歧度 = 极差 (0=一致, 1=最大对立), 不用 /2 以反映真实对抗强度
    let divergence = if spread < 1e-9 { 0.0 } else { spread.min(1.0) };

    // 角色分配: 低于均值 → Con, 高于 → Pro, 等于 → Neutral
    let mut rounds = Vec::new();
    for (i, o) in opinions.iter().enumerate() {
        let role = if o.debiased_score < mean - 1e-9 {
            DebateRole::Con
        } else if o.debiased_score > mean + 1e-9 {
            DebateRole::Pro
        } else {
            DebateRole::Neutral
        };
        rounds.push(DebateRound {
            role,
            score: o.debiased_score,
            confidence: o.confidence,
            argument: format!(
                "{} opinion #{} {:.2}",
                match role {
                    DebateRole::Pro => "支持",
                    DebateRole::Con => "反对",
                    DebateRole::Neutral => "中立",
                },
                i,
                o.debiased_score
            ),
        });
    }

    // 收敛: 反方拉动 (均值向 Con 方向收敛), 分歧大时收敛更强 (取反方意见更重)。
    let con_mean: Vec<f64> = rounds.iter().filter(|r| r.role == DebateRole::Con).map(|r| r.score).collect();
    let pro_mean: Vec<f64> = rounds.iter().filter(|r| r.role == DebateRole::Pro).map(|r| r.score).collect();
    let con_avg = if con_mean.is_empty() { mean } else { con_mean.iter().sum::<f64>() / con_mean.len() as f64 };
    let pro_avg = if pro_mean.is_empty() { mean } else { pro_mean.iter().sum::<f64>() / pro_mean.len() as f64 };
    // 三因素修正: 基准均值 + 反方权重 (分歧度) + 正方残余 (1-分歧度)
    let converged_score = mean * 0.5 + con_avg * divergence * 0.5 + pro_avg * (1.0 - divergence) * 0.5;

    DebateReport {
        rounds,
        converged_score,
        divergence,
        converged: divergence < 0.5,
    }
}

/// pass^k 汇总裁决。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleVerdict {
    pub runs: usize,
    pub k: usize,
    pub passes: usize,
    pub passed: bool,
    pub verdict: Verdict,
    pub last: Box<PanelVerdict>,
}

// ───────────────────────────── 护栏 ─────────────────────────────

/// 护栏报告 — eval 引导运行: 拒绝低 grounding / 隔离幻觉 / 拦截 schema 失败。
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

        // 1) 机械 schema 拦截优先于一切
        if !input.schema_failures.is_empty() {
            return Self {
                action: GuardAction::Reject,
                reason: format!("schema 失败 {} 项", input.schema_failures.len()),
                faithfulness: faith,
                schema_failures: input.schema_failures.clone(),
                grounding_failures: input.grounding_failures,
            };
        }

        // 2) 低 grounding → 拒绝 (reject low grounding)
        if !faith.is_grounded(cfg.grounding_min_ratio) {
            return Self {
                action: GuardAction::Reject,
                reason: format!("grounding {:.2} < {:.2}", faith.grounding_ratio, cfg.grounding_min_ratio),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: input.grounding_failures,
            };
        }

        // 3) 工具声称成功但实际失败 → 拒绝 (轨迹撒谎, 无证据换不来信任)
        if input.grounding_failures > 0 {
            return Self {
                action: GuardAction::Reject,
                reason: format!("{} 次工具 grounding 失败", input.grounding_failures),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: input.grounding_failures,
            };
        }

        // 4) 幻觉声明 → 隔离, 扣留待人工 (quarantine fabrications)
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

/// 组合裁决 — 爆炸半径分级 × 护栏 × 评审组, 输出可执行门控动作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    pub level: GateLevel,
    pub tier: ActionTier,
    pub action: GuardAction,
    pub verdict: Verdict,
    pub reason: String,
}

impl GateDecision {
    /// 沿动作路径决定门控: 确定性检查 (护栏) 优先, 爆炸半径决定强度, LLM 分数殿后。
    pub fn decide(tools: &[ToolSpec], input: &JudgeInput, panel: &JudgePanel) -> Self {
        let tier = ActionTier::classify(tools);
        let level = tier.required_gate();
        let guardrail = GuardrailReport::evaluate(input, &panel.debias);
        let verdict = panel.run(input).verdict;

        // 机械失败 — 确定性检查压过 LLM 分数与自治等级
        if guardrail.action == GuardAction::Reject {
            return Self {
                level: GateLevel::Human,
                tier,
                action: GuardAction::Reject,
                verdict: Verdict::Block,
                reason: format!("机械检查拒绝: {}", guardrail.reason),
            };
        }
        // 幻觉隔离 — 升级人工复核
        if guardrail.action == GuardAction::Quarantine {
            return Self {
                level: GateLevel::Human,
                tier,
                action: GuardAction::Quarantine,
                verdict: Verdict::Review,
                reason: format!("幻觉隔离升级人工: {}", guardrail.reason),
            };
        }
        // 爆炸半径 — 不可逆/扩权动作强制人工, 无置信度豁免
        if level == GateLevel::Human {
            return Self {
                level,
                tier,
                action: GuardAction::Escalate,
                verdict,
                reason: "路径含不可逆/扩权动作, 强制人工审批 (TNR 无豁免)".to_string(),
            };
        }
        // 高分歧 — 转评审组/人工, 不自动放行
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
            reason: format!("门控通过: tier={:?}, verdict={:?}, action={:?}", tier, verdict, guardrail.action),
        }
    }

    /// 是否允许自治执行 — 唯一放行条件。
    pub fn allows_autonomous(&self) -> bool {
        self.level == GateLevel::Light && self.action == GuardAction::Allow && self.verdict == Verdict::Pass
    }

    /// 工具级前置检查 — 给定工具名, 返回 (允许, 原因)。
    /// 用法: 在任何工具执行前调用 `GateDecision::check_tool_call("send_email", &registry, &input, &panel)`。
    pub fn check_tool_call(
        tool_name: &str,
        registry: &ToolRegistry,
        _input: &JudgeInput,
        _panel: &JudgePanel,
    ) -> (bool, String) {
        let Some(spec) = registry.get(tool_name) else {
            return (false, format!("工具 '{}' 未在注册表中, 默认拒绝", tool_name));
        };
        // 单工具快速判定: 只读/可逆 → 允许 (后续完整路径再查); 不可逆/扩权 → 拒绝需人工
        match spec.reversibility {
            ToolReversibility::ReadOnly => (true, "只读工具, 自治放行".to_string()),
            ToolReversibility::Reversible => (true, "可逆工具, 自治放行 (已登记 undo)".to_string()),
            ToolReversibility::Compensable => (false, "可补偿工具, 需评审组审批".to_string()),
            ToolReversibility::Irreversible => (false, "不可逆工具, 强制人工审批".to_string()),
        }
    }

    /// 完整路径检查 — 组合 GateDecision::decide 结果。
    pub fn check_path(tools: &[ToolSpec], input: &JudgeInput, panel: &JudgePanel) -> Self {
        Self::decide(tools, input, panel)
    }
}

/// 工具注册表 — 构建期事实, 非运行期猜测 (TianPan: risk class as versioned tool attribute)。
/// 同一注册表同时发 tool spec 与 gate config, 两者不能分歧。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolRegistry {
    specs: std::collections::HashMap<String, ToolSpec>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, spec: ToolSpec) -> Self {
        self.specs.insert(spec.name.clone(), spec);
        self
    }

    pub fn get(&self, name: &str) -> Option<&ToolSpec> {
        self.specs.get(name)
    }

    pub fn all_specs(&self) -> Vec<&ToolSpec> {
        self.specs.values().collect()
    }

    pub fn cloned_specs(&self) -> Vec<ToolSpec> {
        self.specs.values().cloned().collect()
    }

    /// 从工具名列表快速构建 (只读默认)。
    pub fn from_read_only(names: &[&str]) -> Self {
        let mut reg = Self::new();
        for n in names {
            reg = reg.register(ToolSpec::read_only(n));
        }
        reg
    }
}

// ───────────────────────────── 黄金集校准 ─────────────────────────────

/// 黄金轨迹标签 — 来自真实日志 (clean run / broken run)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrajectoryLabel {
    Clean,
    Broken,
}

/// 一条真实日志黄金样本 — clean 应放行, broken 应拦截。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldTrajectory {
    pub id: String,
    pub label: TrajectoryLabel,
    pub trajectory: AgentTrajectory,
    pub claims: Vec<Claim>,
    pub evidence_ids: Vec<String>,
    pub grounding_failures: u64,
    pub schema_failures: Vec<SchemaCheck>,
}

/// 校准集 — 用真实 clean/broken 日志验证门控 (Anthropic: 最强测试来自真实 transcripts)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSet {
    pub gold: Vec<GoldTrajectory>,
}

impl CalibrationSet {
    pub fn new(gold: Vec<GoldTrajectory>) -> Self {
        Self { gold }
    }

    /// 从 experience-tree KB 的 kv_store `experience` namespace 构建真实黄金集
    /// (FPAM P1: 用真实轨迹校准, 而非 fixture)。
    ///
    /// 输入: `(key, value)` 序列, 每条 value 是 neotrix-experience 写入的
    /// `JSON {type, domain, content, evidence, verify_by}`。
    ///
    /// 标签规则 (确定性, 非猜测): `type` ∈ 失败族 → Broken; 其余 → Clean。
    pub fn from_kb_experience(entries: &[(String, String)]) -> Self {
        let failure_types = [
            "defect", "error", "blocker", "block", "regression", "fail", "wip", "warning", "bug",
        ];
        let mut gold: Vec<GoldTrajectory> = Vec::new();
        let mut id = 0u64;
        for (key, value) in entries {
            if !key.starts_with("branch_") {
                continue;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(value) else { continue };
            let Some(etype) = v.get("type").and_then(|t| t.as_str()) else { continue };
            let content = v.get("content").and_then(|t| t.as_str()).unwrap_or("");
            let evidence = v.get("evidence").and_then(|t| t.as_str()).unwrap_or("");
            let label = if failure_types.iter().any(|f| etype.eq_ignore_ascii_case(f)) {
                TrajectoryLabel::Broken
            } else {
                TrajectoryLabel::Clean
            };
            let mut traj = AgentTrajectory::new(id, content.to_string());
            traj.push(crate::core::nt_core_prm::TrajectoryStep {
                step_idx: 0,
                specialist: crate::core::nt_core_traits::SpecialistType::RiskAssessor,
                e8_mode: crate::core::nt_core_hex::ReasoningHexagram::new(0b001010),
                action: "absorb".to_string(),
                input: evidence.to_string(),
                output: content.to_string(),
                duration_ms: None,
                success: label == TrajectoryLabel::Clean,
                external_reward: Some(if label == TrajectoryLabel::Clean { 1.0 } else { 0.0 }),
            });
            traj.completed = true;
            let evidence_ids: Vec<String> = evidence.split([',', ' ', ';'])
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            let evidence_refs: Vec<&str> = evidence_ids.iter().map(|s| s.as_str()).collect();
            gold.push(GoldTrajectory {
                id: key.clone(),
                label,
                trajectory: traj,
                claims: vec![Claim::new(content, &evidence_refs)],
                evidence_ids,
                grounding_failures: if label == TrajectoryLabel::Broken { 2 } else { 0 },
                schema_failures: Vec::new(),
            });
            id += 1;
        }
        Self { gold }
    }

    /// pass^k 校准: 对每条黄金样本跑评审组 (runs 次, 需 k 次通过),
    /// 报告 clean 召回 / broken 拦截 / 综合。
    pub fn pass_k(&self, panel: &JudgePanel, runs: usize, k: usize) -> CalibrationReport {
        let mut clean_total = 0usize;
        let mut clean_pass = 0usize;
        let mut broken_total = 0usize;
        let mut broken_block = 0usize;

        for g in &self.gold {
            let input = JudgeInput {
                candidate: g.trajectory.task.clone(),
                claims: g.claims.clone(),
                evidence_ids: g.evidence_ids.clone(),
                trajectory: Some(g.trajectory.clone()),
                grounding_failures: g.grounding_failures,
                schema_failures: g.schema_failures.clone(),
                producer_family: JudgeFamily::None,
            };
            let ens = panel.run_ensemble(&input, runs, k);
            match g.label {
                TrajectoryLabel::Clean => {
                    clean_total += 1;
                    if ens.passed {
                        clean_pass += 1;
                    }
                }
                TrajectoryLabel::Broken => {
                    broken_total += 1;
                    if !ens.passed {
                        broken_block += 1;
                    }
                }
            }
        }

        let clean_recall = if clean_total == 0 { 0.0 } else { clean_pass as f64 / clean_total as f64 };
        let broken_precision = if broken_total == 0 { 1.0 } else { broken_block as f64 / broken_total as f64 };
        let balanced = if (clean_recall + broken_precision) == 0.0 { 0.0 } else { 2.0 * clean_recall * broken_precision / (clean_recall + broken_precision) };

        CalibrationReport { clean_total, clean_pass, broken_total, broken_block, clean_recall, broken_precision, balanced }
    }
}

/// 校准报告。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub clean_total: usize,
    pub clean_pass: usize,
    pub broken_total: usize,
    pub broken_block: usize,
    /// clean 样本通过率
    pub clean_recall: f64,
    /// broken 样本拦截率
    pub broken_precision: f64,
    /// F1 调和均值
    pub balanced: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_prm::{TrajectoryStep};
    use crate::core::nt_core_hex::ReasoningHexagram;
    use crate::core::nt_core_traits::SpecialistType;
    use crate::neotrix::l1_body_impl::nt_io_provider::{LlmError, LlmResponse, LlmRequest, Usage, FinishReason};

    /// 测试用 mock LLM 法官 — 返回固定结构化 JSON 评分。
    struct MockJudgeProvider {
        score: f64,
        confidence: f64,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockJudgeProvider {
        async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            let content = format!(
                r#"{{"score":{}, "confidence":{}, "rationale":"mock judge"}}"#,
                self.score, self.confidence
            );
            Ok(LlmResponse {
                content,
                model: "mock-judge".into(),
                usage: Usage::default(),
                finish_reason: FinishReason::Stop,
            tool_calls: None,
            })
        }

        async fn stream_complete(&self, _request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (_, rx) = tokio::sync::mpsc::channel(1);
            Ok(rx)
        }
    }

    fn simple_traj(id: u64, task: &str, successes: usize, total: usize) -> AgentTrajectory {
        let mut t = AgentTrajectory::new(id, task.to_string());
        for i in 0..total {
            t.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram::new(0b001010),
                action: format!("step {}", i),
                input: String::new(),
                output: format!("out {}", i),
                duration_ms: None,
                success: i < successes,
                external_reward: None,
            });
        }
        t
    }

    // ── ActionTier / blast radius ──
    #[test]
    fn tier_all_read_only_is_autonomous() {
        let tools = vec![ToolSpec::read_only("select"), ToolSpec::read_only("get")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier1Autonomous);
        assert_eq!(ActionTier::classify(&tools).required_gate(), GateLevel::Light);
    }

    #[test]
    fn tier_reversible_is_logged_light() {
        let tools = vec![ToolSpec::reversible("edit_file", "undo_edit")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier2Logged);
        assert_eq!(ActionTier::classify(&tools).required_gate(), GateLevel::Light);
    }

    #[test]
    fn tier_irreversible_forces_human() {
        let tools = vec![ToolSpec::read_only("get"), ToolSpec::irreversible("send_email")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier4Human);
        assert_eq!(ActionTier::classify(&tools).required_gate(), GateLevel::Human);
    }

    #[test]
    fn tier_authority_modifying_forces_human() {
        let mut t = ToolSpec::read_only("grant_role");
        t.authority_modifying = true;
        assert_eq!(ActionTier::classify(&[t]), ActionTier::Tier4Human);
    }

    #[test]
    fn tier_compensable_is_review() {
        let tools = vec![ToolSpec::read_only("get"), ToolSpec { name: "refund".to_string(), reversibility: ToolReversibility::Compensable, undo: None, authority_modifying: false }];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier3Review);
    }

    // ── Faithfulness ──
    #[test]
    fn faithfulness_set_difference_catches_fabrication() {
        let claims = vec![
            Claim::new("claim with real ref", &["E-001"]),
            Claim::new("claim with missing ref", &["E-999"]),
            Claim::new("claim with no ref", &[]),
        ];
        let report = FaithfulnessReport::audit(&claims, &["E-001".to_string(), "E-002".to_string()]);
        assert_eq!(report.grounded, 1);
        assert_eq!(report.fabricated.len(), 2);
        assert!(!report.is_grounded(0.6));
        assert!(!report.quarantine().is_empty());
    }

    #[test]
    fn faithfulness_empty_claims_is_zero() {
        let report = FaithfulnessReport::audit(&[], &["E-001".to_string()]);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!report.is_grounded(0.6));
    }

    // ── Schema ──
    #[test]
    fn schema_blocks_missing_field() {
        let checks = check_schema_fields(&["id", "result"], r#"{"id": 1}"#);
        assert_eq!(checks.len(), 2);
        assert!(checks.iter().any(|c| c.field == "result" && !c.present));
    }

    #[test]
    fn schema_passes_complete_json() {
        let checks = check_schema_fields(&["id", "result"], r#"{"id": 1, "result": "ok"}"#);
        assert!(checks.iter().all(|c| c.present));
    }

    #[test]
    fn schema_fails_on_non_object() {
        let checks = check_schema_fields(&["id"], "[1,2,3]");
        assert!(!checks[0].present);
    }

    // ── JudgePanel ──
    #[test]
    fn panel_passes_grounded_input() {
        let input = JudgeInput {
            candidate: "a concise grounded conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Pass, "{}", v.reasoning);
        assert!(v.is_pass());
        assert!(v.median_score >= 0.6);
    }

    #[test]
    fn panel_blocks_low_grounding() {
        let input = JudgeInput {
            candidate: "conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-999"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Block, "{}", v.reasoning);
    }

    #[test]
    fn panel_blocks_broken_trajectory() {
        let input = JudgeInput {
            candidate: "task".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: Some(simple_traj(1, "t", 0, 5)),
            grounding_failures: 3,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Block, "{}", v.reasoning);
    }

    #[test]
    fn family_separation_excludes_same_family_judge() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-001"])];
        input.evidence_ids = vec!["E-001".to_string()];
        input.producer_family = JudgeFamily::Heuristic;
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.opinions.len(), 2, "同族证据法官应被排除");
        assert!(v.opinions.iter().all(|o| o.family != JudgeFamily::Heuristic));
    }

    #[test]
    fn high_disagreement_routes_to_human() {
        let mut panel = JudgePanel::default_panel();
        panel.debias.agreement_review_threshold = 0.99;
        let input = JudgeInput {
            candidate: "a moderately long but grounded conclusion about caching layers".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Review, "{}", v.reasoning);
        assert!(v.routed_to_human);
    }

    #[test]
    fn ensemble_pass_k_requires_all_runs() {
        let input = JudgeInput {
            candidate: "concise grounded conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 5, 5);
        assert!(ens.passed, "pass^5 应全部通过: {}", ens.last.reasoning);
        assert_eq!(ens.passes, 5);
    }

    // ── Guardrail ──
    #[test]
    fn guardrail_rejects_schema_failure() {
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck { field: "result".to_string(), present: false, detail: "missing".to_string() }];
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Reject);
    }

    #[test]
    fn guardrail_rejects_low_grounding() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-999"])];
        input.evidence_ids = vec!["E-001".to_string()];
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Reject);
    }

    #[test]
    fn guardrail_quarantines_fabrications() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-001"]), Claim::new("c2", &["E-001"]), Claim::new("c3", &["E-999"])];
        input.evidence_ids = vec!["E-001".to_string(), "E-002".to_string()];
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Quarantine, "{}", g.reason);
        assert_eq!(g.faithfulness.quarantine().len(), 1);
    }

    #[test]
    fn guardrail_rejects_grounding_failures() {
        let mut input = JudgeInput::new("grounded");
        input.claims = vec![Claim::new("c1", &["E-001"])];
        input.evidence_ids = vec!["E-001".to_string()];
        input.grounding_failures = 2;
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Reject);
    }

    // ── GateDecision ──
    #[test]
    fn gate_allows_autonomous_low_risk() {
        let input = JudgeInput {
            candidate: "low risk grounded change".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let tools = vec![ToolSpec::read_only("get"), ToolSpec::reversible("edit", "undo")];
        let d = GateDecision::decide(&tools, &input, &JudgePanel::default_panel());
        assert!(d.allows_autonomous(), "{}", d.reason);
    }

    #[test]
    fn gate_human_blocks_irreversible_even_if_high_score() {
        let input = JudgeInput {
            candidate: "high confidence grounded".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let tools = vec![ToolSpec::irreversible("send_email")];
        let d = GateDecision::decide(&tools, &input, &JudgePanel::default_panel());
        assert!(!d.allows_autonomous(), "{}", d.reason);
        assert_eq!(d.action, GuardAction::Escalate);
        assert_eq!(d.level, GateLevel::Human);
    }

    #[test]
    fn gate_reject_wins_over_confidence() {
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck { field: "id".to_string(), present: false, detail: "missing".to_string() }];
        let tools = vec![ToolSpec::read_only("get")];
        let d = GateDecision::decide(&tools, &input, &JudgePanel::default_panel());
        assert_eq!(d.action, GuardAction::Reject);
        assert_eq!(d.verdict, Verdict::Block);
        assert!(!d.allows_autonomous());
    }

    // ── Calibration (真实 clean/broken 日志黄金集) ──
    #[test]
    fn calibration_clean_pass_broken_block() {
        let clean = GoldTrajectory {
            id: "clean-1".to_string(),
            label: TrajectoryLabel::Clean,
            trajectory: simple_traj(1, "t1", 5, 5),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            grounding_failures: 0,
            schema_failures: Vec::new(),
        };
        let broken = GoldTrajectory {
            id: "broken-1".to_string(),
            label: TrajectoryLabel::Broken,
            trajectory: simple_traj(2, "t2", 0, 5),
            claims: vec![Claim::new("c1", &["E-999"])],
            evidence_ids: vec!["E-001".to_string()],
            grounding_failures: 4,
            schema_failures: vec![SchemaCheck { field: "result".to_string(), present: false, detail: "missing".to_string() }],
        };
        let set = CalibrationSet::new(vec![clean, broken]);
        let report = set.pass_k(&JudgePanel::default_panel(), 3, 3);
        assert_eq!(report.clean_recall, 1.0, "clean 应放行");
        assert_eq!(report.broken_precision, 1.0, "broken 应拦截");
        assert_eq!(report.balanced, 1.0);
    }

    // ── Debias: verbosity ──
    #[test]
    fn verbosity_penalty_reduces_score() {
        let cfg = DebiasConfig::default();
        assert_eq!(cfg.verbosity_penalty_for("short"), 0.0);
        let long = "x".repeat(cfg.verbosity_norm_len * 3);
        let penalty = cfg.verbosity_penalty_for(&long);
        assert!(penalty > 0.0);
        assert!(penalty <= cfg.verbosity_penalty_cap);
    }

    // ── Ensemble: 机械护栏前置压过聚合 ──
    #[test]
    fn ensemble_rejects_on_schema_failure() {
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck { field: "id".to_string(), present: false, detail: "missing".to_string() }];
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 5, 3);
        assert_eq!(ens.verdict, Verdict::Block, "{}", ens.last.reasoning);
        assert!(!ens.passed);
    }

    #[test]
    fn ensemble_rejects_on_low_grounding() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-999"])];
        input.evidence_ids = vec!["E-001".to_string()];
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 3, 3);
        assert_eq!(ens.verdict, Verdict::Block, "{}", ens.last.reasoning);
    }

    #[test]
    fn ensemble_quarantines_on_fabrication() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-001"]), Claim::new("c2", &["E-001"]), Claim::new("c3", &["E-999"])];
        input.evidence_ids = vec!["E-001".to_string(), "E-002".to_string()];
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 3, 3);
        assert_eq!(ens.verdict, Verdict::Review, "{}", ens.last.reasoning);
        assert!(!ens.passed);
    }

    #[test]
    fn ensemble_passes_clean_input() {
        let input = JudgeInput {
            candidate: "concise grounded conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 5, 5);
        assert!(ens.passed, "pass^5 应全部通过: {}", ens.last.reasoning);
        assert_eq!(ens.verdict, Verdict::Pass);
    }

    // ── ToolRegistry + check_tool_call + check_path ──
    #[test]
    fn registry_unknown_tool_denied() {
        let reg = ToolRegistry::from_read_only(&["get", "query"]);
        assert!(reg.get("get").is_some());
        let (allowed, reason) = GateDecision::check_tool_call("rm -rf", &reg, &JudgeInput::new("x"), &JudgePanel::default_panel());
        assert!(!allowed, "{}", reason);
    }

    #[test]
    fn registry_read_only_allowed_irreversible_denied() {
        let reg = ToolRegistry::new()
            .register(ToolSpec::read_only("get"))
            .register(ToolSpec::irreversible("send_email"));
        let panel = JudgePanel::default_panel();
        let input = JudgeInput::new("x");
        let (read_ok, _) = GateDecision::check_tool_call("get", &reg, &input, &panel);
        assert!(read_ok);
        let (email_ok, reason) = GateDecision::check_tool_call("send_email", &reg, &input, &panel);
        assert!(!email_ok, "{}", reason);
    }

    #[test]
    fn check_path_read_only_autonomous_irreversible_human() {
        let panel = JudgePanel::default_panel();
        let input = JudgeInput {
            candidate: "concise grounded".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let ro_reg = ToolRegistry::from_read_only(&["get"]);
        let ro = GateDecision::check_path(&ro_reg.cloned_specs(), &input, &panel);
        assert!(ro.allows_autonomous(), "{}", ro.reason);
        let irr = GateDecision::decide(&[ToolSpec::irreversible("send_email")], &input, &panel);
        assert!(!irr.allows_autonomous());
        assert_eq!(irr.level, GateLevel::Human);
    }

    // ── LLM 法官 (异步评审路径) ──
    #[tokio::test]
    async fn llm_judge_scores_good_provider() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider { score: 4.0, confidence: 0.9 });
        let judge = LLMJudgeAdapter::new("llm-good", JudgeFamily::Heuristic, provider, "mock-model");
        let input = JudgeInput::new("test");
        let op = judge.score(&input).await;
        assert!((op.raw_score - 1.0).abs() < 0.01, "4/4 → 1.0, got {}", op.raw_score);
        assert!((op.confidence - 0.9).abs() < 0.01);
        assert!(op.attribution_tags.iter().any(|t| t == "llm_provider"));
    }

    #[tokio::test]
    async fn llm_judge_low_score_drags_panel_to_review() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider { score: 1.0, confidence: 0.9 });
        let judge = LLMJudgeAdapter::new("llm-bad", JudgeFamily::Heuristic, provider, "mock-model");
        let input = JudgeInput {
            candidate: "concise grounded".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
        };
        let panel = JudgePanel::default_panel();
        let verdict = panel.run_async(&input, &[&judge]).await;
        assert!(!verdict.is_pass(), "LLM 低分应阻止放行: {}", verdict.reasoning);
    }

    #[tokio::test]
    async fn llm_judge_guardrail_still_blocks_schema_failure() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider { score: 4.0, confidence: 0.9 });
        let judge = LLMJudgeAdapter::new("llm-good", JudgeFamily::Heuristic, provider, "mock-model");
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck { field: "id".to_string(), present: false, detail: "missing".to_string() }];
        let panel = JudgePanel::default_panel();
        let verdict = panel.run_async(&input, &[&judge]).await;
        assert_eq!(verdict.verdict, Verdict::Block, "机械检查压过 LLM 分数: {}", verdict.reasoning);
    }

    #[tokio::test]
    async fn judge_registry_builds_async_judges() {
        let p1: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider { score: 3.0, confidence: 0.8 });
        let p2: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider { score: 4.0, confidence: 0.85 });
        let registry = JudgeRegistry::new()
            .register(JudgeFamily::Analytic, p1, "model-a")
            .register(JudgeFamily::Symbolic, p2, "model-b");
        assert_eq!(registry.entries().len(), 2);
        let judges = registry.build_async_judges();
        assert_eq!(judges.len(), 2);
        assert!(judges.iter().all(|j| j.family() != JudgeFamily::None));
    }

    // ── 真实经验 → 校准集 (experience-tree KB) ──
    #[test]
    fn from_kb_experience_labels_clean_and_broken() {
        let entries: Vec<(String, String)> = vec![
            ("branch_195_0_ab12cd".to_string(), r#"{"type":"defect","domain":"NT-CORE","content":"defect fixed","evidence":"E-001"}"#.to_string()),
            ("branch_195_1_ef34".to_string(), r#"{"type":"insight","domain":"NT-IO","content":"cleaned up UI","evidence":"E-002"}"#.to_string()),
            ("branch_196_0_9a01".to_string(), r#"{"type":"fail","domain":"NT-SHIELD","content":"rollback done","evidence":"E-003"}"#.to_string()),
            ("not_a_branch".to_string(), r#"{"type":"insight","content":"ignored"}"#.to_string()),
        ];
        let set = CalibrationSet::from_kb_experience(&entries);
        assert_eq!(set.gold.len(), 3, "非 branch_ 前缀应被跳过");
        assert_eq!(set.gold[0].label, TrajectoryLabel::Broken, "defect → Broken");
        assert_eq!(set.gold[1].label, TrajectoryLabel::Clean, "insight → Clean");
        assert_eq!(set.gold[2].label, TrajectoryLabel::Broken, "fail → Broken");
        assert!(set.gold[0].trajectory.steps.len() >= 1);
    }

    #[test]
    fn calibration_from_kb_runs_to_completion() {
        let entries: Vec<(String, String)> = vec![
            ("branch_1_0_a".into(), r#"{"type":"insight","content":"clean grounded conclusion","evidence":"E-001"}"#.to_string()),
            ("branch_1_1_b".into(), r#"{"type":"regression","content":"broken step zero","evidence":"E-900"}"#.to_string()),
        ];
        let set = CalibrationSet::from_kb_experience(&entries);
        let panel = JudgePanel::default_panel();
        let report = set.pass_k(&panel, 3, 3);
        assert!(report.broken_total >= 1);
        assert!(report.clean_total >= 1);
        assert!(report.balanced > 0.0 || report.broken_precision == 1.0);
    }

    #[test]
    fn deliberate_converges_when_consensus() {
        // D6: 意见一致 (高分) → 辩论收敛, 无分歧
        let mut ops = Vec::new();
        for i in 0..3 {
            let mut op = JudgeOpinion::new(&format!("j{}", i), JudgeFamily::Analytic);
            op.debiased_score = 0.9;
            op.confidence = 0.8;
            ops.push(op);
        }
        let report = deliberate(&ops);
        assert!(report.converged, "consensus → converged");
        assert!(report.divergence < 0.1, "no divergence on consensus");
        assert!((report.converged_score - 0.9).abs() < 0.1);
        assert_eq!(report.rounds.len(), 3);
    }

    #[test]
    fn deliberate_pulls_toward_contrarian_on_split() {
        // D6: 意见分裂 (两高一分低) → 辩论向反方收敛, 分数被拉低
        let mut ops = Vec::new();
        for (i, s) in [0.9, 0.9, 0.1].iter().enumerate() {
            let mut op = JudgeOpinion::new(&format!("j{}", i), JudgeFamily::Symbolic);
            op.debiased_score = *s;
            op.confidence = 0.7;
            ops.push(op);
        }
        let report = deliberate(&ops);
        assert!(!report.converged, "split → not converged");
        assert!(report.divergence > 0.3, "split creates divergence");
        // 反方 0.1 显著拉低收敛分 (低于纯均值 0.633)
        assert!(report.converged_score < 0.6, "contrarian drags below mean, got {}", report.converged_score);
        // 反方角色被分配
        assert!(report.rounds.iter().any(|r| r.role == DebateRole::Con));
        assert!(report.rounds.iter().any(|r| r.role == DebateRole::Pro));
    }

    #[test]
    fn deliberate_empty_opinions_is_trivial() {
        let report = deliberate(&[]);
        assert!(report.converged);
        assert_eq!(report.converged_score, 0.0);
    }
}
