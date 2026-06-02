use std::collections::{HashMap, VecDeque};

/// Semantic entropy for code generation (ConSelf 2026)
/// High entropy → don't generate code, request LLM instead.
#[derive(Debug, Clone)]
pub struct SemanticEntropyGate {
    pub entropy_threshold: f64,
    pub max_tokens: usize,
    pub recent_entropies: VecDeque<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

impl Default for SemanticEntropyGate {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticEntropyGate {
    pub fn new() -> Self {
        Self {
            entropy_threshold: 0.7,
            max_tokens: 4096,
            recent_entropies: VecDeque::with_capacity(50),
        }
    }

    /// Compute semantic entropy of a code generation request using n-gram diversity.
    /// entropy = unique_trigrams / total_trigrams
    /// Higher diversity (more unique trigrams) → higher entropy → more uncertainty → defer to LLM.
    pub fn compute_entropy(prompt: &str, context: &[String]) -> f64 {
        let mut all_text = prompt.to_string();
        for ctx in context {
            all_text.push(' ');
            all_text.push_str(ctx);
        }

        let tokens: Vec<&str> = all_text.split_whitespace().collect();
        if tokens.len() < 3 {
            return 0.0;
        }

        let total = tokens.len().saturating_sub(2);
        let mut unique = HashMap::new();
        for w in tokens.windows(3) {
            let key = format!("{} {} {}", w[0], w[1], w[2]);
            unique.entry(key).or_insert(0u32);
        }

        let unique_count = unique.len();
        unique_count as f64 / total as f64
    }

    /// Should this request be deferred to LLM?
    pub fn should_defer(&self, prompt: &str, context: &[String]) -> bool {
        let entropy = Self::compute_entropy(prompt, context);
        entropy > self.entropy_threshold
    }

    /// Record an entropy measurement for trend tracking.
    pub fn record(&mut self, entropy: f64) {
        if self.recent_entropies.len() >= self.recent_entropies.capacity() {
            self.recent_entropies.pop_front();
        }
        self.recent_entropies.push_back(entropy);
    }

    /// Trend: is entropy increasing?
    pub fn entropy_trend(&self) -> TrendDirection {
        if self.recent_entropies.len() < 3 {
            return TrendDirection::Stable;
        }

        let v: Vec<f64> = self.recent_entropies.iter().copied().collect();
        let mut increasing = 0;
        let mut decreasing = 0;

        for w in v.windows(2) {
            if w[1] > w[0] + 0.01 {
                increasing += 1;
            } else if w[1] < w[0] - 0.01 {
                decreasing += 1;
            }
        }

        let total = v.len() - 1;
        let inc_ratio = increasing as f64 / total as f64;
        let dec_ratio = decreasing as f64 / total as f64;

        if inc_ratio > 0.6 {
            TrendDirection::Increasing
        } else if dec_ratio > 0.6 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
}

// ─── SE‑08 / ConSelf 2026: pairwise candidate‑based semantic entropy ───

/// Action taken based on an entropy measurement.
#[derive(Debug, Clone, PartialEq)]
pub enum EntropyAction {
    AutoApplied,
    DeferredToLLM,
    Rejected,
}

/// A record of one entropy measurement in the detector history.
#[derive(Debug, Clone)]
pub struct EntropyRecord {
    pub edit_hash: String,
    pub entropy: f32,
    pub action: EntropyAction,
}

/// Context surrounding a code edit whose semantic entropy is being measured.
#[derive(Debug, Clone)]
pub struct EditContext {
    pub file_path: String,
    pub current_code: String,
    pub target_change: String,
}

/// Semantic entropy estimated via pairwise edit‑distance between multiple
/// candidate code edits (SE‑08 / ConSelf 2026).
///
/// High entropy → high uncertainty about correctness → defer to LLM.
/// Low entropy → confident → auto‑apply.
#[derive(Debug, Clone)]
pub struct SemanticEntropy {
    /// Number of generation samples used to estimate entropy (default 3).
    pub n_samples: usize,
    /// Temperature range for sampling candidates (default 0.5, 1.5).
    pub temperature_range: (f32, f32),
    /// Entropy above this threshold causes deferral to LLM (default 0.7).
    pub entropy_threshold: f32,
    /// Recent entropy measurements.
    pub history: Vec<EntropyRecord>,
}

impl SemanticEntropy {
    pub fn new(n_samples: usize, entropy_threshold: f32) -> Self {
        Self {
            n_samples,
            temperature_range: (0.5, 1.5),
            entropy_threshold,
            history: Vec::new(),
        }
    }

    /// Estimate semantic entropy from N candidate code strings.
    ///
    /// Computes pairwise normalised Levenshtein similarity between all
    /// candidate pairs, then `entropy = 1.0 − avg_pairwise_similarity`.
    ///
    /// Edge cases:
    /// - `n_samples < 2` or fewer than 2 candidates → 0.0
    /// - All candidates identical → 0.0 (maximum confidence)
    /// - All candidates completely different → 1.0 (maximum uncertainty)
    /// - Empty candidates → 0.0
    pub fn estimate_entropy(&self, candidates: &[String]) -> f32 {
        if candidates.len() < 2 {
            return 0.0;
        }

        let mut total_similarity: f64 = 0.0;
        let mut pairs: usize = 0;

        for i in 0..candidates.len() {
            for j in (i + 1)..candidates.len() {
                let sim = char_similarity(&candidates[i], &candidates[j]);
                total_similarity += sim as f64;
                pairs += 1;
            }
        }

        let avg_similarity = total_similarity / pairs as f64;
        1.0 - avg_similarity as f32
    }

    /// Return `true` when the measured entropy exceeds the threshold.
    pub fn should_defer(&self, entropy: f32) -> bool {
        entropy > self.entropy_threshold
    }

    /// Record an entropy measurement for later inspection / trending.
    pub fn record(&mut self, edit_hash: String, entropy: f32, action: EntropyAction) {
        self.history.push(EntropyRecord { edit_hash, entropy, action });
    }
}

impl Default for SemanticEntropy {
    fn default() -> Self {
        Self::new(3, 0.7)
    }
}

/// Normalised character‑level similarity between two strings.
/// Returns 1.0 for identical strings, 0.0 for completely different.
/// Uses Levenshtein distance normalised by the longer string length.
fn char_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let dist = levenshtein_distance(a, b);
    let max_len = a.len().max(b.len()) as f32;
    1.0 - (dist as f32 / max_len)
}

/// Compute Levenshtein edit distance between two strings (character‑level).
/// Uses two‑row DP for O(min(m,n)) memory.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let (short, long) = if m <= n {
        (&a_chars, &b_chars)
    } else {
        (&b_chars, &a_chars)
    };

    let mut prev_row: Vec<usize> = (0..=short.len()).collect();
    let mut curr_row = vec![0usize; short.len() + 1];

    for (i, lc) in long.iter().enumerate() {
        curr_row[0] = i + 1;
        for (j, sc) in short.iter().enumerate() {
            let cost = if lc == sc { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)          // insertion
                .min(prev_row[j + 1] + 1)                // deletion
                .min(prev_row[j] + cost);                // substitution
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[short.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gate_default_threshold() {
        let gate = SemanticEntropyGate::new();
        assert!((gate.entropy_threshold - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_compute_entropy_low_diversity() {
        let prompt = "let x = 1; let x = 1; let x = 1; let x = 1; let x = 1;";
        let context = vec!["fn main() {}".into()];
        let entropy = SemanticEntropyGate::compute_entropy(prompt, &context);
        // Repeated pattern → low diversity → low entropy (close to 1.0 - high_ratio)
        assert!(entropy < 0.8, "entropy too high: {}", entropy);
    }

    #[test]
    fn test_compute_entropy_high_diversity() {
        let prompt = "xquzl mfbrp kwjdt yhgsa npcve litrb zomax qeruy";
        let context = vec!["abc def ghi jkl mno pqr stu vwx yz".into()];
        let entropy = SemanticEntropyGate::compute_entropy(prompt, &context);
        // Mostly unique tokens → high diversity → low uniqueness ratio → high entropy
        assert!(entropy > 0.5, "entropy too low: {}", entropy);
    }

    #[test]
    fn test_should_defer_high_entropy() {
        let gate = SemanticEntropyGate {
            entropy_threshold: 0.3,
            ..SemanticEntropyGate::new()
        };
        let prompt = "a b c d e f g h i j k l m n o p";
        let context = vec!["q r s t u v w x y z".into()];
        // High diversity → high entropy → should defer
        assert!(gate.should_defer(prompt, &context));
    }

    #[test]
    fn test_should_not_defer_low_entropy() {
        let gate = SemanticEntropyGate {
            entropy_threshold: 0.9,
            ..SemanticEntropyGate::new()
        };
        let prompt = "let x = 1; let x = 1;";
        let context = vec!["fn main() {}".into()];
        // Low diversity → should NOT defer
        assert!(!gate.should_defer(prompt, &context));
    }

    #[test]
    fn test_record_and_trend_tracking() {
        let mut gate = SemanticEntropyGate::new();
        assert_eq!(gate.entropy_trend(), TrendDirection::Stable);
        gate.record(0.8);
        gate.record(0.7);
        gate.record(0.6);
        // Decreasing
        assert_eq!(gate.entropy_trend(), TrendDirection::Decreasing);
    }

    #[test]
    fn test_empty_context_zero_entropy() {
        let entropy = SemanticEntropyGate::compute_entropy("", &[]);
        assert!((entropy - 0.0).abs() < 1e-6);
    }

    // ─── Pairwise SemanticEntropy tests (SE‑08 / ConSelf) ───

    #[test]
    fn test_new_semantic_entropy_defaults() {
        let se = SemanticEntropy::default();
        assert_eq!(se.n_samples, 3);
        assert!((se.entropy_threshold - 0.7).abs() < f32::EPSILON);
        assert_eq!(se.temperature_range, (0.5, 1.5));
        assert!(se.history.is_empty());
    }

    #[test]
    fn test_new_semantic_entropy_custom() {
        let se = SemanticEntropy::new(5, 0.5);
        assert_eq!(se.n_samples, 5);
        assert!((se.entropy_threshold - 0.5).abs() < f32::EPSILON);
        assert!(se.history.is_empty());
    }

    /// 1. Identical candidates → entropy ≈ 0
    #[test]
    fn test_identical_candidates_zero_entropy() {
        let se = SemanticEntropy::default();
        let candidates = vec!["fn main() {}".into(), "fn main() {}".into(), "fn main() {}".into()];
        let entropy = se.estimate_entropy(&candidates);
        assert!(entropy < 0.01, "identical candidates should yield ~0 entropy, got {}", entropy);
    }

    /// 2. All different candidates → entropy ≈ 1
    #[test]
    fn test_all_different_candidates_high_entropy() {
        let se = SemanticEntropy::default();
        let candidates = vec!["aaaa".into(), "bbbb".into(), "cccc".into()];
        let entropy = se.estimate_entropy(&candidates);
        // Each pair has 0 similarity (no overlapping chars), so entropy ≈ 1
        assert!(entropy > 0.9, "all-different candidates should yield ~1 entropy, got {}", entropy);
    }

    /// 3. n_samples = 0 → 0 entropy (always apply)
    #[test]
    fn test_n_samples_zero_entropy() {
        let se = SemanticEntropy::new(0, 0.7);
        let candidates: Vec<String> = vec![];
        let entropy = se.estimate_entropy(&candidates);
        assert!((entropy - 0.0).abs() < f32::EPSILON);
    }

    /// 4. Single candidate → 0 entropy
    #[test]
    fn test_single_candidate_zero_entropy() {
        let se = SemanticEntropy::default();
        let candidates = vec!["fn main() {}".into()];
        let entropy = se.estimate_entropy(&candidates);
        assert!((entropy - 0.0).abs() < f32::EPSILON);
    }

    /// 5. Empty edit → 0 entropy
    #[test]
    fn test_empty_candidates_zero_entropy() {
        let se = SemanticEntropy::default();
        let candidates: Vec<String> = vec![];
        let entropy = se.estimate_entropy(&candidates);
        assert!((entropy - 0.0).abs() < f32::EPSILON);
    }

    /// 6. Default threshold behavior
    #[test]
    fn test_should_defer_default_threshold() {
        let se = SemanticEntropy::default();
        // entropy 0.8 > threshold 0.7 → defer
        assert!(se.should_defer(0.8));
        // entropy 0.6 ≤ threshold 0.7 → do not defer
        assert!(!se.should_defer(0.6));
        // entropy exactly at threshold → do not defer
        assert!(!se.should_defer(0.7));
    }

    #[test]
    fn test_should_defer_custom_threshold() {
        let se = SemanticEntropy::new(3, 0.3);
        assert!(se.should_defer(0.5));
        assert!(!se.should_defer(0.2));
    }

    /// 7. Recording and history management
    #[test]
    fn test_record_entropy() {
        let mut se = SemanticEntropy::default();
        assert!(se.history.is_empty());
        se.record("abc123".into(), 0.2, EntropyAction::AutoApplied);
        assert_eq!(se.history.len(), 1);
        assert_eq!(se.history[0].edit_hash, "abc123");
        assert!((se.history[0].entropy - 0.2).abs() < f32::EPSILON);
        assert_eq!(se.history[0].action, EntropyAction::AutoApplied);
    }

    #[test]
    fn test_record_multiple_entropies() {
        let mut se = SemanticEntropy::default();
        se.record("a".into(), 0.1, EntropyAction::AutoApplied);
        se.record("b".into(), 0.8, EntropyAction::DeferredToLLM);
        se.record("c".into(), 0.9, EntropyAction::Rejected);
        assert_eq!(se.history.len(), 3);
        assert_eq!(se.history[1].action, EntropyAction::DeferredToLLM);
        assert_eq!(se.history[2].action, EntropyAction::Rejected);
    }

    #[test]
    fn test_record_maintains_order() {
        let mut se = SemanticEntropy::default();
        se.record("first".into(), 0.3, EntropyAction::AutoApplied);
        se.record("second".into(), 0.7, EntropyAction::DeferredToLLM);
        assert_eq!(se.history[0].edit_hash, "first");
        assert_eq!(se.history[1].edit_hash, "second");
    }

    /// 8. Integration: entropy measurement affects apply decision
    #[test]
    fn test_semantic_entropy_gate_cycle() {
        let mut se = SemanticEntropy::default();
        // Low-entropy candidates → should NOT defer
        let low_entropy_candidates = vec![
            "fn main() { let x = 1; }".into(),
            "fn main() { let x = 1; }".into(),
        ];
        let entropy1 = se.estimate_entropy(&low_entropy_candidates);
        assert!(!se.should_defer(entropy1));
        se.record("low".into(), entropy1, EntropyAction::AutoApplied);

        // High-entropy candidates → SHOULD defer
        let high_entropy_candidates = vec![
            "aaaa".into(),
            "bbbb".into(),
        ];
        let entropy2 = se.estimate_entropy(&high_entropy_candidates);
        assert!(se.should_defer(entropy2));
        se.record("high".into(), entropy2, EntropyAction::DeferredToLLM);

        // History shows both actions
        assert_eq!(se.history.len(), 2);
        assert_eq!(se.history[0].action, EntropyAction::AutoApplied);
        assert_eq!(se.history[1].action, EntropyAction::DeferredToLLM);
    }

    // ─── Helper function tests ───

    #[test]
    fn test_char_similarity_identical() {
        assert!((char_similarity("hello", "hello") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_char_similarity_completely_different() {
        assert!((char_similarity("abc", "xyz") - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_char_similarity_partial() {
        let sim = char_similarity("kitten", "sitten");
        // 1 char diff out of 6 → similarity ≈ 0.833
        assert!((sim - 0.833).abs() < 0.01);
    }

    #[test]
    fn test_char_similarity_both_empty() {
        assert!((char_similarity("", "") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_char_similarity_one_empty() {
        assert!((char_similarity("abc", "") - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
    }

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("rust", "rust"), 0);
    }

    #[test]
    fn test_levenshtein_single_substitution() {
        assert_eq!(levenshtein_distance("kitten", "sitten"), 1);
    }

    #[test]
    fn test_levenshtein_single_insertion() {
        assert_eq!(levenshtein_distance("kitten", "kittens"), 1);
    }

    #[test]
    fn test_levenshtein_single_deletion() {
        assert_eq!(levenshtein_distance("kitten", "kitte"), 1);
    }

    #[test]
    fn test_levenshtein_complex() {
        // "saturday" → "sunday": s-a-t-u-r-d-a-y vs s-u-n-d-a-y
        // substitutions: a→u (pos 2), t→n (pos 3), r→d (pos 5?) ...
        // well‑known: distance = 3
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }
}
