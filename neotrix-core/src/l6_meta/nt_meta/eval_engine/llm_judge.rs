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

/// 判据来源 —— `EMERGENCE-PLAN` §6.4「自报独立性」的**机器化**。
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

/// **证据层级** —— 判官**输入本身**是什么性质的东西。
///
/// ## 为什么需要它（`CriteriaSource` 不够用）
///
/// [`CriteriaSource`] 回答的是「判据**从哪来**」，本枚举回答的是
/// 「被判官读的那个**产出**是从哪来的」。两者**正交**。
///
/// 【源】判据来自 `morluto/rea` 的 `docs/adr/0003-managed-code-evidence-and-provider-boundary.md`：
/// > "That reconstruction is valuable, but it is not the original source
/// > and cannot replace the underlying metadata and CIL evidence."
///
/// rea 用**四级**区分，本仓实现其中对本仓产出链真正起作用的三级：
/// | 本枚举 | rea 对应 | 含义 |
/// |---|---|---|
/// | `Canonical` | canonical static observation | **直接观测/量出来的**，不是任何解读的产物 |
/// | `Reconstruction` | reconstruction | 由 canonical 经**某个模型的解读**而来 |
/// | `Inference` | analyst inference | 人的/系统的**推断与外推**，无直接观测 |
///
/// ## ⛔ 缺失的那一层才是危险的那一层
///
/// 【推】在 `EvidenceTier` 出现之前，`CriteriaSource::Independent` 的语义是
/// 「判据独立」—— 但如果**被判官读的产出本身是上游模型的重建**，
/// 那么：判官再独立，也只是在**对重建做独立评估**。
/// ⇒ 那**不是**涌现证据，而是「重建的独立评估」。
/// 二级分类看不见这一层 ⇒ 会**静默误判为涌现证据**。
///
/// **失败长什么样**：`seal_loop` 把 L5 的自产解读喂给判官，
/// 判官来源标 `Independent`，于是计数 +1 —— 而底层从未被直接观测过。
///
/// **如何验证**：构造 `EvidenceTier::Reconstruction` +
/// `CriteriaSource::Independent` 的组合，断言 `is_emergence_evidence()`
/// 返回 `false`（而非 `true`）。
///
/// ## 默认取最坏假设
///
/// 与本仓一贯默认收紧同向（`CriteriaSource` 默认 `SelfReported`、
/// 沙箱 fail-closed、cumora 未知引擎默认拒绝）：
/// 不声明时取 `Inference`（最弱），要升级必须**显式** [`JudgeResult::with_evidence_tier`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvidenceTier {
    /// ✅ 直接观测/量出来的。可作涌现证据。
    Canonical,
    /// ⛔ 由 canonical 经某个模型的解读而来。**不是**原始事实。
    Reconstruction,
    /// ⛔⛔ 推断与外推，最弱。本仓默认值。
    #[default]
    Inference,
}

impl EvidenceTier {
    /// ⛔ 只有 `Canonical` 才算直接观测。
    pub fn is_canonical(&self) -> bool {
        matches!(self, EvidenceTier::Canonical)
    }
}

/// Configuration for an LLM-as-Judge evaluation.
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub criteria: Vec<Criterion>,
    pub max_score: f32,
    /// 判据来源。`new()` **默认 `SelfReported`** ⇒ 不可当涌现证据。
    pub criteria_source: CriteriaSource,
}

impl JudgeConfig {
    /// ⛔ 默认 `SelfReported` —— 未声明来源时**取最坏假设**。
    /// 想让它可当涌现证据，必须**显式**调[`with_source`](Self::with_source)。
    pub fn new(criteria: Vec<Criterion>, max_score: f32) -> Self {
        Self { criteria, max_score, criteria_source: CriteriaSource::SelfReported }
    }

    /// 显式声明判据来源。见[`CriteriaSource`]。
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
    /// 本次评分所用的判据来源（随结果一起落盘，供事后审计）。
    pub criteria_source: CriteriaSource,
    /// 被判官读取的**产出**本身的证据层级（2026-10-05 增补）。
    ///
    /// ⛔ **无 `Default` 构造路径**：`JudgeResult` 的每一处构造都必须显式
    ///   指定它，或经 [`JudgeResult::with_evidence_tier`] 显式设置。
    ///   ⇒ 编译器强制「新评分必须声明产出层级」，与
    ///   `CriteriaSource` 的「默认取最坏」不同 —— 这里连默认值都不给，
    ///   因为忘记声明**没有**安全的取值（`Inference` 只是最坏，不是安全）。
    pub evidence_tier: EvidenceTier,
}

impl JudgeResult {
    /// 显式声明产出层级。见[`EvidenceTier`]。
    pub fn with_evidence_tier(mut self, tier: EvidenceTier) -> Self {
        self.evidence_tier = tier;
        self
    }

    /// **可否计入涌现证据** —— `EMERGENCE-PLAN` §6.4 的机器判据。
    ///
    /// ⛔ **两个条件都必须满足**（与 2026-10-05 之前不同）：
    /// 1. `CriteriaSource::Independent` —— 判据独立于产出；
    /// 2. `EvidenceTier::Canonical` —— 产出本身是**直接观测**。
    ///
    /// **为什么必须是「与」而不是「或」**：
    /// 独立判官读**重建产物**时，判官再独立也只是「对重建做独立评估」，
    /// 底层从未被直接观测 ⇒ 那不是涌现证据（依据 rea ADR-0003）。
    /// 只有判据独立**且**产出为 canonical，两条链路才都没有模型的解读环节。
    ///
    /// ⛔ `SelfReported` ⇒ false（判据与产出同源）。
    /// ⛔ `Reconstruction` / `Inference` ⇒ false（产出非直接观测）。
    ///
    /// 之所以做成**方法**而不是注释：注释会被下一个 agent 忽略，
    /// 而`if !result.is_emergence_evidence() { skip }` 会在编译期与评审时暴露。
    pub fn is_emergence_evidence(&self) -> bool {
        matches!(self.criteria_source, CriteriaSource::Independent)
            && self.evidence_tier.is_canonical()
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
            // 退化配置（无判据 / 权重零）⇒ 无「被判读的产出」可言，
            // 取最弱层：这两条路径本就不该产出任何涌现证据。
            evidence_tier: EvidenceTier::Inference,
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
            // 退化配置（无判据 / 权重零）⇒ 无「被判读的产出」可言，
            // 取最弱层：这两条路径本就不该产出任何涌现证据。
            evidence_tier: EvidenceTier::Inference,
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
    // 判据来源 / 自报独立性（`EMERGENCE-PLAN` §6.4）
    // ══════════════════════════════════════════════════════════════

    /// 反向锁：`JudgeConfig::new` **默认 `SelfReported`**
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
            evidence_tier: EvidenceTier::Canonical,
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
            evidence_tier: EvidenceTier::Canonical,
        };
        assert!(manual.is_emergence_evidence());
    }

    /// 边界判据的**失败方向**也要锁：满分同源 ⇒ 仍不可计入。
    /// （防止将来有人用「分数很高」当作「可信」的等价物。）
    #[test]
    fn self_reported_perfect_score_still_not_evidence() {
        let manual = JudgeResult {
            total_score: 10.0,
            max_possible: 10.0,
            criterion_scores: Vec::new(),
            summary: "Full score".into(),
            criteria_source: CriteriaSource::SelfReported,
            evidence_tier: EvidenceTier::Canonical,
        };
        assert!(!manual.is_emergence_evidence());
    }

    // ══════════════════════════════════════════════════════════════
    // 证据层级（2026-10-05，来自 `morluto/rea` ADR-0003）
    // ══════════════════════════════════════════════════════════════

    fn judge(src: CriteriaSource, tier: EvidenceTier) -> JudgeResult {
        JudgeResult {
            total_score: 9.0,
            max_possible: 10.0,
            criterion_scores: Vec::new(),
            summary: String::new(),
            criteria_source: src,
            evidence_tier: tier,
        }
    }

    /// **本轮新增的核心判据**：独立判官读**重建产物** ⇒ 仍不得计入。
    ///
    /// 这正是 `EvidenceTier` 存在的唯一理由。
    /// 二级分类下这个组合会返回 `true`（判据独立 ⇒ 可计入）⇒ **静默误判**。
    #[test]
    fn independent_judge_over_reconstruction_is_NOT_evidence() {
        let r = judge(CriteriaSource::Independent, EvidenceTier::Reconstruction);
        assert!(
            !r.is_emergence_evidence(),
            "独立判官读**重建产物**不得计入涌现证据：底层从未被直接观测过"
        );
    }

    /// 推断层同理（比重建更弱）。
    #[test]
    fn independent_judge_over_inference_is_NOT_evidence() {
        assert!(!judge(CriteriaSource::Independent, EvidenceTier::Inference).is_emergence_evidence());
    }

    /// ✅ 唯一可计入的组合：**判据独立 且 产出为 canonical**。
    #[test]
    fn independent_judge_over_canonical_IS_evidence() {
        assert!(judge(CriteriaSource::Independent, EvidenceTier::Canonical).is_emergence_evidence());
    }

    /// ⛔ 反向锁：canonical 也救不了同源判据（两条件是「与」不是「或」）。
    #[test]
    fn canonical_output_cannot_rescue_self_reported_judge() {
        assert!(
            !judge(CriteriaSource::SelfReported, EvidenceTier::Canonical).is_emergence_evidence(),
            "判据同源时，即便产出是直接观测也**不得**计入"
        );
    }

    /// 全 2×3 真值表 —— 穷举而非抽样，防止将来加变体时漏判。
    #[test]
    fn evidence_truth_table_is_exhaustive() {
        let expected = |s: CriteriaSource, t: EvidenceTier| {
            matches!(s, CriteriaSource::Independent) && matches!(t, EvidenceTier::Canonical)
        };
        for s in [CriteriaSource::SelfReported, CriteriaSource::Independent] {
            for t in [EvidenceTier::Canonical, EvidenceTier::Reconstruction, EvidenceTier::Inference] {
                assert_eq!(
                    judge(s, t).is_emergence_evidence(),
                    expected(s, t),
                    "组合({s:?}, {t:?}) 判定与真值表不符"
                );
            }
        }
    }

    /// ⛔ 默认取最弱层（`Inference`），且**不能**靠 Default 静默升级。
    #[test]
    fn evidence_tier_default_is_weakest() {
        assert_eq!(EvidenceTier::default(), EvidenceTier::Inference);
        assert!(!EvidenceTier::default().is_canonical());
        // 且 Inference 单独就足以否决
        assert!(!judge(CriteriaSource::Independent, EvidenceTier::default()).is_emergence_evidence());
    }

    /// 反向锁：把 `&&` 改回「只看 CriteriaSource」必须失败。
    /// 这条锁直接守住本轮的核心修复。
    /// **未来污染不变性元测试**（2026-10-05，来自 `HKUDS/Vibe-Trading`）
    ///
    /// 【源】`agent/tests/factors/test_lookahead.py` 的判据：
    /// *"Look-ahead guard: factor values at row `t` must not depend on rows > t."*
    /// 方法是「扰动下游未来 ⇒ 断言上游结论不变」，并配**正向对照**
    /// （否则一个恒返回 0 的坏实现也能通过）。
    ///
    /// 【推】为什么这条适配本仓：`evaluate_response` 的
    /// 「判官与被评判产出同源」问题，在金融语境下的等价物就是前视偏差 ——
    /// **判官的结论不得依赖它本不该看到的输入**。
    /// 本仓 `evaluate_response` 当前对非退化配置**恒拒绝**（返回 `None`），
    /// ⇒ 真正可测的不变量在**退化路径**上。
    ///
    /// 形状：扰动 `response`（被判读的产出 = 「未来」）⇒
    /// 断言 `prompt` 侧可观测的量不变。
    #[test]
    fn perturbing_response_must_not_change_prompt_side_observation() {
        // 退化配置（无判据）⇒ 返回 Some，且分数由 config 决定而非输入内容
        let empty = JudgeConfig::new(vec![], 5.0);
        let baseline = evaluate_response(&empty, "PROMPT_A", "RESPONSE_A").expect("退化配置应给出确定分");

        // 扰动「未来」：response 完全换掉
        let perturbed = evaluate_response(&empty, "PROMPT_A", "TOTALLY_DIFFERENT_ZZZZ").expect("退化配置应给出确定分");

        // 不变量：prompt 侧与 config 决定的量不受 response 影响
        assert_eq!(
            (baseline.total_score, baseline.max_possible, baseline.summary.clone()),
            (perturbed.total_score, perturbed.max_possible, perturbed.summary.clone()),
            "扰动 response 不得改变退化路径上的分数/上限/摘要"
        );
        assert!(
            baseline.criterion_scores.is_empty() && perturbed.criterion_scores.is_empty(),
            "无判据 ⇒ 不应有 criterion_scores"
        );
    }

    /// **正向对照**（子代理明确要求的一条）：
    /// 上面的不变性测试**单独存在时会被一个恒返回 0 的坏实现通过**。
    /// 故必须同时证明「输入确实被读到了、确实起作用」。
    #[test]
    fn positive_control_proves_inputs_actually_reach_the_function() {
        // 对照 1：response 确实被读取（由 `prompt` 侧无关，但配置区分度必须真实存在）
        let degenerate = JudgeConfig::new(vec![], 5.0);
        let a = evaluate_response(&degenerate, "p", "r").expect("退化");
        let b = evaluate_response(&JudgeConfig::new(vec![], 9.0), "p", "r").expect("退化");
        assert_eq!(
            a.max_possible, 5.0,
            "max_possible 必须来自 config ⇒ 证明 config 真的被读了"
        );
        assert_ne!(
            a.max_possible, b.max_possible,
            "不同 config 必须给出不同上限（否则上面的不变性是空洞的）"
        );

        // 对照 2：非退化配置必须**拒绝**，且拒绝与输入无关（恒 None）
        let real = JudgeConfig::new(vec![Criterion::new("a", "d", 1.0)], 5.0);
        for resp in ["", "r", "完全不同的回答", "ZZZZ"] {
            assert!(
                evaluate_response(&real, "p", resp).is_none(),
                "非退化配置对任何输入都必须拒绝（输入不得影响拒绝与否）"
            );
        }
    }

    /// **污染方向对照**：证明 `response` 真的进了函数体。
    /// 若实现将来改成忽略 `response`，上面两条不变性测试会同时「通过」——
    /// 本测试就是防这个：它要求**至少存在一个 response 影响可观测量的机制**。
    ///
    /// 当前实现的诚实答案是：`evaluate_response` 对非退化配置拒绝，
    /// 所以**不存在**这样的机制 —— 本测试断言的正是这个**已知边界**，
    /// 而不是编造一个差异。⚠️ 若将来真接入模型判官，本测试必须改为
    /// 「同一 config 下不同 response ⇒ 分数可不同」。
    #[test]
    fn response_is_currently_not_read_because_non_degenerate_is_refused() {
        let real = JudgeConfig::new(vec![Criterion::new("a", "d", 1.0)], 5.0);
        // 全部拒绝 ⇒ response 对结果**没有任何**影响力
        let outs: Vec<Option<f32>> = ["A", "B", "C"]
            .iter()
            .map(|r| evaluate_response(&real, "p", r).map(|x| x.total_score))
            .collect();
        assert!(
            outs.iter().all(|o| o.is_none()),
            "当前实现下非退化配置恒拒绝 ⇒ response 无影响力（这是已知边界，非缺陷断言）"
        );
    }

    #[test]
    fn reverse_lock_tier_must_participate_in_the_verdict() {
        // 若 `is_emergence_evidence` 忽略 tier，则此断言必红
        let sneaky = judge(CriteriaSource::Independent, EvidenceTier::Reconstruction);
        assert_ne!(
            sneaky.is_emergence_evidence(),
            judge(CriteriaSource::Independent, EvidenceTier::Canonical).is_emergence_evidence(),
            "tier 必须参与判定：否则 Reconstruction 与 Canonical 无区别，EvidenceTier 是死字段"
        );
        assert!(
            sneaky.evidence_tier != EvidenceTier::Canonical,
            "本测试自身的前提：Reconstruction 不得等于 Canonical"
        );
    }
}

