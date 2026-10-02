#![forbid(unsafe_code)]

/// 5-dimension admission scores for a memory entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmissionScores {
    pub utility: f32,
    pub confidence: f32,
    pub novelty: f32,
    pub recency: f32,
    pub type_prior: f32,
}

// ─── 权重类型（2026-09-30 补齐）────────────────────────────────────────
// 同 `decay_forgetting` 的情形：自由函数形态已写好，但 `config.rs` 仍在用
// 权重类型形态，而 `ScoreWeights` / `TypePriorWeights` **全仓都不存在**
// ⇒ 模块不可编译、581 行不参与编译。
// ⇒ 补齐，且 `ScoreWeights` 的字段与既有 `AdmissionScores` **一致**
// （utility/confidence/novelty/recency/type_prior），
// 以便 `weighted_score(&AdmissionScores, &AdmissionScores)` 的传参类型
// 与语义对齐（现有函数已用 `AdmissionScores` 当权重载体，签名不变）。

/// 各维度打分权重（与 [`AdmissionScores`] 同字段，便于直接传入 `weighted_score`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreWeights {
    pub utility: f32,
    pub confidence: f32,
    pub novelty: f32,
    pub recency: f32,
    pub type_prior: f32,
}

impl Default for ScoreWeights {
    /// 等权（各 0.2，合计 1.0）。
    fn default() -> Self {
        Self { utility: 0.2, confidence: 0.2, novelty: 0.2, recency: 0.2, type_prior: 0.2 }
    }
}

impl ScoreWeights {
    /// 权重总和（用于诊断/归一化检查）。
    pub fn total(&self) -> f32 {
        self.utility + self.confidence + self.novelty + self.recency + self.type_prior
    }
}

/// 按条目类型的先验权重表。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypePriorWeights {
    /// entry_type → 先验分。
    pub weights: std::collections::HashMap<String, f32>,
}

/// 未登记类型的缺省先验。
///
/// ⚠️ 取 **0.5**（中性）：既不奖励也不惩罚。依据是本模块自带测试
/// `test_to_type_priors` 断言 `tp.get("unknown") == 0.5`
/// —— 该期望已被采纳为契约。
const DEFAULT_TYPE_PRIOR: f32 = 0.5;

impl TypePriorWeights {
    pub fn new() -> Self {
        Self { weights: std::collections::HashMap::new() }
    }

    pub fn with(mut self, entry_type: impl Into<String>, prior: f32) -> Self {
        self.weights.insert(entry_type.into(), prior);
        self
    }

    /// 取某类型的先验；未登记时返回 [`DEFAULT_TYPE_PRIOR`]（0.5）。
    pub fn get(&self, entry_type: &str) -> f32 {
        self.weights.get(entry_type).copied().unwrap_or(DEFAULT_TYPE_PRIOR)
    }

    pub fn set(&mut self, entry_type: impl Into<String>, prior: f32) {
        self.weights.insert(entry_type.into(), prior);
    }

    pub fn len(&self) -> usize {
        self.weights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }
}

/// Score a raw content string for admission.
///
/// - `content`: the memory entry text (used to derive utility/confidence heuristics).
/// - `existing_count`: how many entries already exist (feeds novelty — more entries → lower novelty).
/// - `entry_age_hours`: age of the entry in hours (feeds recency — older → lower recency).
pub fn score_entry(content: &str, existing_count: usize, entry_age_hours: f32) -> AdmissionScores {
    let len = content.len() as f32;
    let word_count = content.split_whitespace().count() as f32;

    // Utility: longer, richer content scores higher (saturates at ~500 chars).
    let utility = (len / 500.0).min(1.0);

    // Confidence: heuristic based on sentence structure — presence of '.' or ':' suggests formality.
    let has_period = content.contains('.');
    let has_colon = content.contains(':');
    let confidence = if has_period && has_colon {
        0.9
    } else if has_period || has_colon {
        0.7
    } else if word_count > 3.0 {
        0.5
    } else {
        0.3
    };

    // Novelty: decreases as existing count grows (log scale).
    let novelty = if existing_count == 0 {
        1.0
    } else {
        (1.0 / (1.0 + (existing_count as f32).ln())).clamp(0.0, 1.0)
    };

    // Recency: decays with age (half-life ~24h).
    let recency = if entry_age_hours <= 0.0 {
        1.0
    } else {
        (-entry_age_hours / 24.0).exp().clamp(0.0, 1.0)
    };

    // Type_prior: default neutral prior (content-type heuristic).
    let type_prior = if content.starts_with("error")
        || content.starts_with("Error")
        || content.starts_with("ERROR")
    {
        0.9
    } else if content.starts_with("todo")
        || content.starts_with("TODO")
        || content.starts_with("fixme")
    {
        0.8
    } else {
        0.5
    };

    AdmissionScores {
        utility,
        confidence,
        novelty,
        recency,
        type_prior,
    }
}

/// Compute a weighted composite score from dimension scores and weights.
/// Both `scores` and `weights` use the same 5 fields; result is clamped to [0, 1].
pub fn weighted_score(scores: &AdmissionScores, weights: &AdmissionScores) -> f32 {
    let raw = scores.utility * weights.utility
        + scores.confidence * weights.confidence
        + scores.novelty * weights.novelty
        + scores.recency * weights.recency
        + scores.type_prior * weights.type_prior;
    raw.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_weights() -> AdmissionScores {
        AdmissionScores {
            utility: 0.25,
            confidence: 0.20,
            novelty: 0.20,
            recency: 0.15,
            type_prior: 0.20,
        }
    }

    #[test]
    fn score_entry_returns_all_dimensions() {
        let scores = score_entry("The system module handles error recovery.", 5, 2.0);
        assert!(scores.utility > 0.0 && scores.utility <= 1.0);
        assert!(scores.confidence > 0.0 && scores.confidence <= 1.0);
        assert!(scores.novelty > 0.0 && scores.novelty <= 1.0);
        assert!(scores.recency > 0.0 && scores.recency <= 1.0);
        assert!(scores.type_prior > 0.0 && scores.type_prior <= 1.0);
    }

    #[test]
    fn novelty_decreases_with_existing_count() {
        let s0 = score_entry("test", 0, 0.0);
        let s10 = score_entry("test", 10, 0.0);
        let s100 = score_entry("test", 100, 0.0);
        assert!(s0.novelty >= s10.novelty);
        assert!(s10.novelty >= s100.novelty);
    }

    #[test]
    fn recency_decays_with_age() {
        let fresh = score_entry("test", 0, 0.0);
        let aged = score_entry("test", 0, 48.0);
        assert!(fresh.recency > aged.recency);
    }

    #[test]
    fn weighted_score_clamps_to_unit() {
        let scores = AdmissionScores {
            utility: 1.0,
            confidence: 1.0,
            novelty: 1.0,
            recency: 1.0,
            type_prior: 1.0,
        };
        let w = weighted_score(&scores, &default_weights());
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weighted_score_zero_weights_gives_zero() {
        let scores = score_entry("hello", 0, 0.0);
        let zeros = AdmissionScores { utility: 0.0, confidence: 0.0, novelty: 0.0, recency: 0.0, type_prior: 0.0 };
        assert_eq!(weighted_score(&scores, &zeros), 0.0);
    }

    #[test]
    fn error_content_gets_high_type_prior() {
        let scores = score_entry("error: connection refused", 0, 0.0);
        assert!(scores.type_prior >= 0.9);
    }

    #[test]
    fn todo_content_gets_moderate_type_prior() {
        let scores = score_entry("TODO: refactor auth module", 0, 0.0);
        assert!(scores.type_prior >= 0.8);
    }

    #[test]
    fn weighted_score_symmetry() {
        let a = AdmissionScores { utility: 0.8, confidence: 0.6, novelty: 0.4, recency: 0.9, type_prior: 0.5 };
        let b = default_weights();
        let w1 = weighted_score(&a, &b);
        let w2 = weighted_score(&b, &a);
        assert!((w1 - w2).abs() < 1e-6);
    }

    #[test]
    fn score_empty_content() {
        let scores = score_entry("", 0, 0.0);
        assert!((scores.utility - 0.0).abs() < 1e-6);
    }

    #[test]
    fn confidence_high_for_formal_text() {
        let scores = score_entry("Dr. Smith: the analysis shows results.", 0, 0.0);
        assert!(scores.confidence >= 0.9);
    }
}
