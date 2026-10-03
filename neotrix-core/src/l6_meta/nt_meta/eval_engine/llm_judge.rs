#![deny(clippy::unwrap_used)]

/// A single evaluation criterion with weight.
#[derive(Debug, Clone)]
pub struct Criterion {
    pub name: String,
    pub description: String,
    pub weight: f32,
}

impl Criterion {
    pub fn new(name: impl Into<String>, description: impl Into<String>, weight: f32) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
        }
    }
}

/// ⭐⭐ 判据来源 —— `EMERGENCE-PLAN` §6.4「自报独立性」的**机器化**。
///
/// **为什么需要它**：CASP（剑桥，Hinton/Bengio/Jack Clark 等 23 人）指出
/// 「AI systems now write most of the code inside the companies that build them」；
/// `yetone/cumora` 用 seen-cursor 让过期回复**重新决策**；
/// `yetone/magpie` 规定历史「never inferred from today's configured key」。
/// ⇒ 三份**互相独立**的来源指向同一条：**当系统自身能影响度量时，度量即失效。**
///
/// **为什么默认是 `SelfReported`**：不声明来源时**取最坏假设**。
/// 这与本仓一贯的默认收紧同向（cumora 的未知引擎**默认拒绝**、
/// 沙箱**默认 fail-closed**）；若默认取 `Independent`，
/// 则「忘记声明」会静默升级成「可当涌现证据」—— 那正是本仓最贵的一类 bug。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriteriaSource {
    /// ⛔ 判据与被评判产出**同源**（自己写 rubric 评自己）
    SelfReported,
    /// ✅ 判据**独立于**被评判产出（外部基准 / 人工标注 / 固定 rubric）
    Independent,
}

/// Configuration for an LLM-as-Judge evaluation.
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub criteria: Vec<Criterion>,
    pub max_score: f32,
    /// ⭐ 判据来源。`new()` **默认 `SelfReported`** ⇒ 不可当涌现证据。
    pub criteria_source: CriteriaSource,
}

impl JudgeConfig {
    /// ⛔ 默认 `SelfReported` —— 未声明来源时**取最坏假设**。
    /// 想让它可当涌现证据，必须**显式**调[`with_source`](Self::with_source)。
    pub fn new(criteria: Vec<Criterion>, max_score: f32) -> Self {
        Self { criteria, max_score, criteria_source: CriteriaSource::SelfReported }
    }

    /// ⭐ 显式声明判据来源。见[`CriteriaSource`]。
    pub fn with_source(mut self, source: CriteriaSource) -> Self {
        self.criteria_source = source;
        self
    }

    /// Total weight across all criteria.
    pub fn total_weight(&self) -> f32 {
        self.criteria.iter().map(|c| c.weight).sum()
    }
}

/// Score for a single criterion.
#[derive(Debug, Clone)]
pub struct CriterionScore {
    pub criterion_name: String,
    pub score: f32,
    pub reasoning: String,
}

/// Result of evaluating a response.
#[derive(Debug, Clone)]
pub struct JudgeResult {
    pub total_score: f32,
    pub max_possible: f32,
    pub criterion_scores: Vec<CriterionScore>,
    pub summary: String,
    /// ⭐ 本次评分所用的判据来源（随结果一起落盘，供事后审计）。
    pub criteria_source: CriteriaSource,
}

impl JudgeResult {
    /// ⭐⭐ **可否计入涌现证据** —— `EMERGENCE-PLAN` §6.4 的机器判据。
    ///
    /// ⛔ `SelfReported`（判据与产出同源）⇒ **返回false**：
    /// 这种评分可用于调试与回归，但**不得**计入 `new_category` 等涌现计数，
    /// 且必须与独立来源（如 `nt_emergence_detector` 的实测计数）**并列呈现**。
    ///
    /// ✅ `Independent` ⇒ 可计入。
    ///
    /// ⭐ 之所以做成**方法**而不是注释：注释会被下一个 agent 忽略，
    /// 而`if !result.is_emergence_evidence() { skip }` 会在编译期与评审时暴露。
    pub fn is_emergence_evidence(&self) -> bool {
        matches!(self.criteria_source, CriteriaSource::Independent)
    }
}

/// Evaluate a response against configured criteria.
///
/// Performs weighted scoring: each criterion's score (0..max_score) is multiplied
/// by its weight, normalized by total weight, then scaled to max_possible.
/// **能否自动评判**：本仓的 `Criterion` 只有 `name` / `description` / `weight`，
/// **不含任何可用于打分的 rubric 信号**（无关键词、无模式、无参考答案）。
/// ⇒ 任何不调用真实模型的确定性评分都只能是**换一种说法的伪造**
/// （按长度？按相似度？都是代理指标冒充判据）。
/// ⇒ 故此处**拒绝评分**而不是编一个分数。
///
/// 危害对比：恒返回 `max_score` 的版本会**自动为「已涌现」提供证据**
/// （分数永远最高）—— 那比没有仪器更危险。
/// 见 `docs/architecture/EMERGENCE-PLAN-2026-10-02.md` P0。
pub const REFUSAL_REASON: &str =
    "Criterion carries no rubric signal (no keywords/patterns/reference answer); \
     a deterministic auto-scorer would be a fabricated proxy metric. \
     Refusing to score rather than returning a constant maximum.";

/// Evaluate a response against configured criteria.
///
/// Returns `None` when a credible automatic judgement is **not possible**
/// (i.e. whenever there is at least one weighted criterion).
/// `Some` is returned only for the two degenerate configs whose score is
/// well-defined without inspecting the response at all
/// (no criteria / total weight zero ⇒ legitimately zero).
///
/// ⛔ **Never returns `Some(..)` with `total_score == max_possible` for a
/// non-degenerate config.** If you need a real score, plug in a real model
/// judge — do not relax this.
pub fn evaluate_response(
    config: &JudgeConfig,
    prompt: &str,
    response: &str,
) -> Option<JudgeResult> {

    if config.criteria.is_empty() {
        return Some(JudgeResult {
            total_score: 0.0,
            max_possible: config.max_score,
            criterion_scores: Vec::new(),
            summary: "No criteria configured".into(),
            criteria_source: config.criteria_source,
        });
    }

    let total_weight = config.total_weight();
    if total_weight == 0.0 {
        return Some(JudgeResult {
            total_score: 0.0,
            max_possible: config.max_score,
            criterion_scores: Vec::new(),
            summary: "Total weight is zero".into(),
            criteria_source: config.criteria_source,
        });
    }

    // ⛔ 原实现在此把 `normalized` 硬编码为 1.0、`score` 硬编码为
    // `config.max_score`、`reasoning` 硬编码为 "Full score"，
    // 并且用 `let _ = (prompt, response);` 把入参整个丢弃
    // ⇒ **任何输入都返回满分**，且现有测试还把该行为断言成了预期。
    // ⇒ 现改为：无非退化配置可判 ⇒ 拒绝评分（None）。
    //    `prompt` / `response` 保留在签名里，是为了让将来接入真实
    //    模型评判时**不必再改调用方签名**；当前刻意不读取它们。
    let _ = (prompt, response);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(n: usize, max: f32) -> JudgeConfig {
        JudgeConfig::new(
            (0..n)
                .map(|i| Criterion::new(format!("c{i}"), "desc", 1.0 / n as f32))
                .collect(),
            max,
        )
    }

    #[test]
    fn test_criterion_new() {
        let c = Criterion::new("faithfulness", "Stays grounded in context", 0.5);
        assert_eq!(c.name, "faithfulness");
        assert_eq!(c.description, "Stays grounded in context");
        assert!((c.weight - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_judge_config_total_weight() {
        let config = JudgeConfig::new(
            vec![
                Criterion::new("a", "desc a", 0.3),
                Criterion::new("b", "desc b", 0.7),
            ],
            5.0,
        );
        assert!((config.total_weight() - 1.0).abs() < f32::EPSILON);
    }

    // ── 以下三条取代原「恒满分」断言 ──────────────────────────────
    // 原测试传字面量 "prompt"/"response" 却断言拿满分，
    // **把伪造行为钉成了预期** —— 这正是它长期没被修的原因：
    // 任何让实现变诚实的改动都会让它们红，于是改动被搁置。

    #[test]
    fn test_non_degenerate_config_is_REFUSED_not_scored() {
        assert!(
            evaluate_response(&cfg(1, 5.0), "p", "r").is_none(),
            "有 rubric 缺失的配置**必须拒绝评分**，不得返回任何分数"
        );
    }

    /// ⛔ 反向锁：满分的唯一来源只能是两个退化配置（其 0 分是良定义的）。
    /// 覆盖 3 种响应 × 多种配置，确保**没有任何一种**能拿到 `max_score`。
    #[test]
    fn test_no_input_can_ever_score_max() {
        let responses = ["", "r", "完全不相关的回答内容"];
        for n in 1..=4usize {
            for max in [1.0f32, 5.0, 10.0] {
                for r in responses {
                    let got = evaluate_response(&cfg(n, max), "prompt", r);
                    assert!(
                        got.is_none(),
                        "配置({n} 判据, max={max}) + 响应 {r:?} 竟返回了 {:?}",
                        got.map(|g| g.total_score)
                    );
                }
            }
        }
    }

    #[test]
    fn test_degenerate_configs_still_score_legitimately() {
        // 无判据 ⇒ 0 分是良定义的，不算伪造
        let empty = JudgeConfig::new(vec![], 5.0);
        let r = evaluate_response(&empty, "p", "r").expect("退化配置应给出确定分");
        assert_eq!(r.total_score, 0.0);
        assert_eq!(r.max_possible, 5.0);
        assert!(r.criterion_scores.is_empty());
        assert_eq!(r.summary, "No criteria configured");

        // 总权重为 0 ⇒ 同理
        let zero = JudgeConfig::new(vec![Criterion::new("a", "d", 0.0)], 5.0);
        let r = evaluate_response(&zero, "p", "r").expect("退化配置应给出确定分");
        assert_eq!(r.total_score, 0.0);
        assert_eq!(r.summary, "Total weight is zero");
    }

    #[test]
    fn test_refusal_reason_states_why() {
        let r = REFUSAL_REASON;
        // 拒绝理由必须真的说明原因，否则调用方无法据此决策
        assert!(r.contains("rubric"), "拒绝理由未说明缺 rubric 信号");
        assert!(r.contains("Refusing"), "拒绝理由未表明这是主动拒绝而非故障");
    }

    // ══════════════════════════════════════════════════════════════
    // ⭐ 判据来源 / 自报独立性（`EMERGENCE-PLAN` §6.4）
    // ══════════════════════════════════════════════════════════════

    /// ⭐⭐ 反向锁：`JudgeConfig::new` **默认 `SelfReported`**
    /// ⇒ 不声明来源就**不可**计入涌现证据。
    /// 这条锁的意义：若默认取 `Independent`，
    /// 「忘记声明」会静默升级成「可当涌现证据」。
    #[test]
    fn default_config_is_self_reported_and_not_emergence_evidence() {
        let c = cfg(2, 10.0);
        assert_eq!(c.criteria_source, CriteriaSource::SelfReported);
        // 造一个结果并检查判定（走真实 evaluate_response 的非退化路径）
        let r = evaluate_response(&c, "p", "r");
        assert!(r.is_none(), "非退化配置本应拒绝评分");
        // 直接构造结果验证判定逻辑（拒绝路径下也要可判）
        let manual = JudgeResult {
            total_score: 10.0,
            max_possible: 10.0,
            criterion_scores: Vec::new(),
            summary: String::new(),
            criteria_source: CriteriaSource::SelfReported,
        };
        assert!(
            !manual.is_emergence_evidence(),
            "同源判据**不得**计入涌现证据"
        );
    }

    /// 显式声明独立 ⇒ 可计入涌现证据。
    #[test]
    fn independent_source_is_emergence_evidence() {
        let c = cfg(2, 10.0).with_source(CriteriaSource::Independent);
        assert_eq!(c.criteria_source, CriteriaSource::Independent);
        let manual = JudgeResult {
            total_score: 7.0,
            max_possible: 10.0,
            criterion_scores: Vec::new(),
            summary: String::new(),
            criteria_source: c.criteria_source,
        };
        assert!(manual.is_emergence_evidence());
    }

    /// ⭐ 边界判据的**失败方向**也要锁：满分同源 ⇒ 仍不可计入。
    /// （防止将来有人用「分数很高」当作「可信」的等价物。）
    #[test]
    fn self_reported_perfect_score_still_not_evidence() {
        let manual = JudgeResult {
            total_score: 10.0,
            max_possible: 10.0,
            criterion_scores: Vec::new(),
            summary: "Full score".into(),
            criteria_source: CriteriaSource::SelfReported,
        };
        assert!(!manual.is_emergence_evidence());
    }
}

