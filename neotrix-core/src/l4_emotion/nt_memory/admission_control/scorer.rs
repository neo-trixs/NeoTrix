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
