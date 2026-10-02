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

/// Configuration for an LLM-as-Judge evaluation.
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub criteria: Vec<Criterion>,
    pub max_score: f32,
}

impl JudgeConfig {
    pub fn new(criteria: Vec<Criterion>, max_score: f32) -> Self {
        Self { criteria, max_score }
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
        });
    }

    let total_weight = config.total_weight();
    if total_weight == 0.0 {
        return Some(JudgeResult {
            total_score: 0.0,
            max_possible: config.max_score,
            criterion_scores: Vec::new(),
            summary: "Total weight is zero".into(),
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
}
