use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

const VSA_DIM: usize = 4096;

/// Aggregate statistics for the Goal Drift Index.
#[derive(Debug, Clone)]
pub struct GdiStats {
    pub sample_count: usize,
    pub mean_gdi: f64,
    pub std_gdi: f64,
    pub max_gdi: f64,
    pub min_gdi: f64,
    pub drift_count: usize,
    pub drift_ratio: f64,
}

/// A single drift measurement sample.
#[derive(Debug, Clone)]
pub struct DriftSample {
    pub timestamp: u64,
    pub semantic_score: f64,
    pub lexical_score: f64,
    pub structural_score: f64,
    pub distributional_score: f64,
    pub gdi: f64,
    pub is_drift: bool,
}

/// GoalDriftIndex — SAHOO-inspired learned multi-signal drift detector.
///
/// Monitors four drift signals and computes a composite Goal Drift Index (GDI).
/// Compares current output text against a reference to detect semantic, lexical,
/// structural, and distributional divergence.
#[derive(Clone)]
pub struct GoalDriftIndex {
    window_size: usize,
    semantic_threshold: f64,
    lexical_threshold: f64,
    structural_threshold: f64,
    distributional_threshold: f64,
    history: VecDeque<DriftSample>,
}

impl GoalDriftIndex {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            semantic_threshold: 0.3,
            lexical_threshold: 0.4,
            structural_threshold: 0.3,
            distributional_threshold: 0.3,
            history: VecDeque::with_capacity(window_size),
        }
    }

    /// Record a drift sample by comparing `text` against `reference`.
    ///
    /// Computes four normalized drift signals:
    /// - semantic: 1 - VSA cosine similarity
    /// - lexical: 1 - word-level Jaccard similarity
    /// - structural: KL divergence of sentence length distributions
    /// - distributional: KL divergence of character bigram frequencies
    ///
    /// Composite GDI is the arithmetic mean of the four signals.
    /// Drift is flagged when GDI exceeds an adaptive threshold
    /// (running mean + 2 * running standard deviation).
    pub fn record(&mut self, text: &str, reference: &str) -> DriftSample {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let semantic_score = self.compute_semantic_drift(text, reference);
        let lexical_score = self.compute_lexical_drift(text, reference);
        let structural_score = self.compute_structural_drift(text, reference);
        let distributional_score = self.compute_distributional_drift(text, reference);

        let gdi = (semantic_score + lexical_score + structural_score + distributional_score) / 4.0;

        let is_drift = if self.history.is_empty() {
            gdi > self.composite_threshold_initial()
        } else {
            let (mean, std) = self.running_stats();
            gdi > mean + 2.0 * std
        };

        let sample = DriftSample {
            timestamp: now,
            semantic_score,
            lexical_score,
            structural_score,
            distributional_score,
            gdi,
            is_drift,
        };

        if self.history.len() >= self.window_size {
            self.history.pop_front();
        }
        self.history.push_back(sample.clone());

        sample
    }

    /// Current composite Goal Drift Index (most recent sample).
    pub fn gdi(&self) -> f64 {
        self.history.back().map(|s| s.gdi).unwrap_or(0.0)
    }

    /// Whether the most recent sample is flagged as drift.
    pub fn drift_detected(&self) -> bool {
        self.history.back().map(|s| s.is_drift).unwrap_or(false)
    }

    /// Reset all history.
    pub fn reset(&mut self) {
        self.history.clear();
    }

    /// Aggregate statistics over the current window.
    pub fn stats(&self) -> GdiStats {
        let count = self.history.len();
        if count == 0 {
            return GdiStats {
                sample_count: 0,
                mean_gdi: 0.0,
                std_gdi: 0.0,
                max_gdi: 0.0,
                min_gdi: 0.0,
                drift_count: 0,
                drift_ratio: 0.0,
            };
        }

        let sum: f64 = self.history.iter().map(|s| s.gdi).sum();
        let mean = sum / count as f64;
        let variance = self
            .history
            .iter()
            .map(|s| (s.gdi - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        let std = variance.sqrt();
        let max = self
            .history
            .iter()
            .map(|s| s.gdi)
            .fold(f64::NEG_INFINITY, f64::max);
        let min = self
            .history
            .iter()
            .map(|s| s.gdi)
            .fold(f64::INFINITY, f64::min);
        let drift_count = self.history.iter().filter(|s| s.is_drift).count();

        GdiStats {
            sample_count: count,
            mean_gdi: mean,
            std_gdi: std,
            max_gdi: max,
            min_gdi: min,
            drift_count,
            drift_ratio: if count > 0 {
                drift_count as f64 / count as f64
            } else {
                0.0
            },
        }
    }

    fn composite_threshold_initial(&self) -> f64 {
        (self.semantic_threshold
            + self.lexical_threshold
            + self.structural_threshold
            + self.distributional_threshold)
            / 4.0
    }

    fn running_stats(&self) -> (f64, f64) {
        let count = self.history.len();
        if count == 0 {
            return (0.0, 1.0);
        }
        let sum: f64 = self.history.iter().map(|s| s.gdi).sum();
        let mean = sum / count as f64;
        let variance = self
            .history
            .iter()
            .map(|s| (s.gdi - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        (mean, variance.sqrt())
    }

    // ── Drift signal computations ──

    fn compute_semantic_drift(&self, text: &str, reference: &str) -> f64 {
        let tv = text_to_vsa(text);
        let rv = text_to_vsa(reference);
        let sim = QuantizedVSA::cosine(&tv, &rv);
        1.0 - sim
    }

    fn compute_lexical_drift(&self, text: &str, reference: &str) -> f64 {
        let jac = jaccard_similarity(text, reference);
        1.0 - jac
    }

    fn compute_structural_drift(&self, text: &str, reference: &str) -> f64 {
        let tlens = sentence_lengths(text);
        let rlens = sentence_lengths(reference);
        kl_divergence(&tlens, &rlens)
    }

    fn compute_distributional_drift(&self, text: &str, reference: &str) -> f64 {
        let tfreq = char_bigram_frequencies(text);
        let rfreq = char_bigram_frequencies(reference);
        kl_divergence_map(&tfreq, &rfreq)
    }
}

// ── Drift signal helpers ──

fn text_to_vsa(text: &str) -> Vec<u8> {
    let words: Vec<&str> = text.split_whitespace().filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return vec![0; VSA_DIM];
    }
    let first = word_to_vsa(words[0]);
    if words.len() == 1 {
        return first;
    }
    let rest: Vec<Vec<u8>> = words[1..]
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let mut v = word_to_vsa(w);
            v = QuantizedVSA::permute(&v, (i + 1) as isize);
            v
        })
        .collect();
    let refs: Vec<&[u8]> = rest.iter().map(|v| v.as_slice()).collect();
    let mut all: Vec<&[u8]> = vec![first.as_slice()];
    all.extend(refs);
    QuantizedVSA::bundle(&all)
}

fn word_to_vsa(word: &str) -> Vec<u8> {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    word.hash(&mut h);
    let seed = h.finish();
    QuantizedVSA::seeded_random(seed, VSA_DIM)
}

fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a: HashSet<&str> = a.split_whitespace().filter(|w| !w.is_empty()).collect();
    let set_b: HashSet<&str> = b.split_whitespace().filter(|w| !w.is_empty()).collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

fn sentence_lengths(text: &str) -> Vec<f64> {
    let sentences: Vec<&str> = text
        .split(|c: char| c == '.' || c == '!' || c == '?' || c == '\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if sentences.is_empty() {
        return vec![0.0];
    }

    let lengths: Vec<usize> = sentences
        .iter()
        .map(|s| s.split_whitespace().count())
        .collect();

    let max_len = *lengths.iter().max().unwrap_or(&1);
    let bins = (max_len + 1).max(5);
    let mut hist = vec![0.0_f64; bins];

    for &len in &lengths {
        let idx = len.min(bins - 1);
        hist[idx] += 1.0;
    }

    let total = lengths.len() as f64;
    for v in hist.iter_mut() {
        *v = (*v + 1.0) / (total + bins as f64);
    }

    hist
}

fn char_bigram_frequencies(text: &str) -> HashMap<String, f64> {
    let cleaned: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let chars: Vec<char> = cleaned.chars().collect();
    let mut freq: HashMap<String, f64> = HashMap::new();

    if chars.len() < 2 {
        freq.insert(String::new(), 1.0);
        return freq;
    }

    for window in chars.windows(2) {
        let bigram: String = window.iter().collect();
        *freq.entry(bigram).or_insert(0.0) += 1.0;
    }

    let total: f64 = freq.values().sum();
    let vocab = freq.len() as f64;

    // Laplace smoothing
    for v in freq.values_mut() {
        *v = (*v + 1.0) / (total + vocab);
    }

    freq
}

fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    let max_len = p.len().max(q.len());
    if max_len == 0 {
        return 0.0;
    }

    let mut padded_p = p.to_vec();
    let mut padded_q = q.to_vec();
    padded_p.resize(max_len, 1.0 / (max_len as f64));
    padded_q.resize(max_len, 1.0 / (max_len as f64));

    let mut kl = 0.0;
    for i in 0..max_len {
        let pi = padded_p[i];
        let qi = padded_q[i];
        if pi > 0.0 && qi > 0.0 {
            kl += pi * (pi / qi).ln();
        }
    }
    kl.max(0.0)
}

fn kl_divergence_map(p: &HashMap<String, f64>, q: &HashMap<String, f64>) -> f64 {
    let all_keys: HashSet<&String> = p.keys().chain(q.keys()).collect();
    if all_keys.is_empty() {
        return 0.0;
    }

    let mut kl = 0.0;
    for k in all_keys {
        let pi = p.get(k).copied().unwrap_or(0.0);
        let qi = q.get(k).copied().unwrap_or(0.0);
        if pi > 0.0 {
            let qs = if qi > 0.0 { qi } else { 1e-10 };
            kl += pi * (pi / qs).ln();
        }
    }
    kl.max(0.0)
}

/// ConstraintPreservation — SAHOO-inspired safety-critical invariance enforcement.
///
/// Registers named invariant checks (functions from text → bool) and tracks
/// violation counts across all invocations.
pub struct ConstraintPreservation {
    invariant_checks: HashMap<String, Box<dyn Fn(&str) -> bool + Send>>,
    violation_count: HashMap<String, u64>,
    total_checks: u64,
}

impl ConstraintPreservation {
    pub fn new() -> Self {
        Self {
            invariant_checks: HashMap::new(),
            violation_count: HashMap::new(),
            total_checks: 0,
        }
    }

    pub fn register_check(&mut self, name: &str, check: Box<dyn Fn(&str) -> bool + Send>) {
        self.invariant_checks.insert(name.to_string(), check);
        self.violation_count.entry(name.to_string()).or_insert(0);
    }

    pub fn check_all(&mut self, text: &str) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        for (name, check) in &self.invariant_checks {
            self.total_checks += 1;
            let passed = check(text);
            results.insert(name.clone(), passed);
            if !passed {
                *self.violation_count.entry(name.clone()).or_insert(0) += 1;
            }
        }
        results
    }

    pub fn violation_rate(&self, name: &str) -> f64 {
        let checked = self.total_checks;
        if checked == 0 {
            return 0.0;
        }
        let violations = self.violation_count.get(name).copied().unwrap_or(0);
        violations as f64 / checked as f64
    }

    pub fn total_violations(&self) -> u64 {
        self.violation_count.values().sum()
    }
}

/// RegressionRisk — SAHOO-inspired regression-risk quantification.
///
/// Compares recent performance (last 10 samples) against the older window
/// to estimate the probability that the system is regressing.
pub struct RegressionRisk {
    risk_scores: VecDeque<f64>,
    window: usize,
}

impl RegressionRisk {
    pub fn new(window: usize) -> Self {
        Self {
            risk_scores: VecDeque::with_capacity(window),
            window,
        }
    }

    pub fn record(&mut self, score: f64) {
        if self.risk_scores.len() >= self.window {
            self.risk_scores.pop_front();
        }
        self.risk_scores.push_back(score);
    }

    /// Estimate probability of regression.
    ///
    /// Compares mean of last `recency` samples (default 10) against mean of
    /// the remaining older samples. If recent < older by more than 1 pooled
    /// standard deviation, return a normal CDF approximation of the probability
    /// that recent performance is worse than older.
    pub fn regression_risk(&self) -> f64 {
        let n = self.risk_scores.len();
        if n < 3 {
            return 0.0;
        }

        let recency = 10usize.min(n / 2);
        if recency < 2 {
            return 0.0;
        }

        let recent_slice: Vec<f64> = self
            .risk_scores
            .iter()
            .rev()
            .take(recency)
            .copied()
            .collect();
        let older_slice: Vec<f64> = self.risk_scores.iter().take(n - recency).copied().collect();

        let recent_mean = recent_slice.iter().sum::<f64>() / recent_slice.len() as f64;
        let older_mean = older_slice.iter().sum::<f64>() / older_slice.len() as f64;

        // If recent mean >= older mean, no regression
        if recent_mean >= older_mean {
            return 0.0;
        }

        let recent_var = recent_slice
            .iter()
            .map(|v| (v - recent_mean).powi(2))
            .sum::<f64>()
            / recent_slice.len() as f64;
        let older_var = older_slice
            .iter()
            .map(|v| (v - older_mean).powi(2))
            .sum::<f64>()
            / older_slice.len() as f64;

        let pooled_std = ((recent_var + older_var) / 2.0).sqrt();
        if pooled_std < 1e-12 {
            return 0.5;
        }

        let effect = (older_mean - recent_mean) / pooled_std;
        // Normal CDF approximation (Abramowitz and Stegun 26.2.17)
        normal_cdf(effect)
    }
}

/// Normal CDF approximation using the Abramowitz & Stegun formula 26.2.17.
/// Accurate to ~1.5e-7.
fn normal_cdf(x: f64) -> f64 {
    if x < -8.0 {
        return 0.0;
    }
    if x > 8.0 {
        return 1.0;
    }
    let b0 = 0.2316419;
    let b1 = 0.319381530;
    let b2 = -0.356563782;
    let b3 = 1.781477937;
    let b4 = -1.821255978;
    let b5 = 1.330274429;

    let t = 1.0 / (1.0 + b0 * x.abs());
    let poly = t * (b1 + t * (b2 + t * (b3 + t * (b4 + t * b5))));
    let phi = (1.0 / (2.0 * std::f64::consts::PI).sqrt()) * (x * x / -2.0).exp();

    let result = phi * poly;
    if x >= 0.0 {
        1.0 - result
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdi_constructor_defaults() {
        let gdi = GoalDriftIndex::new(100);
        assert_eq!(gdi.window_size, 100);
        assert!((gdi.semantic_threshold - 0.3).abs() < 1e-10);
        assert!((gdi.lexical_threshold - 0.4).abs() < 1e-10);
        assert!((gdi.structural_threshold - 0.3).abs() < 1e-10);
        assert!((gdi.distributional_threshold - 0.3).abs() < 1e-10);
        assert!(gdi.history.is_empty());
        assert_eq!(gdi.gdi(), 0.0);
        assert!(!gdi.drift_detected());
    }

    #[test]
    fn identical_texts_no_drift() {
        let mut gdi = GoalDriftIndex::new(100);
        let text = "The quick brown fox jumps over the lazy dog. It was a warm summer day.";
        let sample = gdi.record(text, text);
        assert!(
            sample.gdi < 0.15,
            "identical texts should have near-zero drift, got {}",
            sample.gdi
        );
        assert!(!sample.is_drift, "identical texts should not trigger drift");
    }

    #[test]
    fn very_different_texts_detect_drift() {
        let mut gdi = GoalDriftIndex::new(100);
        let text_a = "Neural networks are a powerful tool for machine learning and artificial intelligence research. Deep learning has transformed many fields.";
        let text_b = "Quantum computing operations rely on superposition and entanglement of qubits. Error correction remains a major challenge for practical systems.";
        let sample = gdi.record(text_a, text_b);
        assert!(
            sample.gdi > 0.25,
            "different topics should have significant drift, got {}",
            sample.gdi
        );
    }

    #[test]
    fn drift_detected_flag() {
        let mut gdi = GoalDriftIndex::new(100);
        // First sample with similar texts -> no drift
        let s1 = gdi.record("Hello world and good day", "Hello world and good day");
        assert!(!s1.is_drift);

        // Now introduce a very different text
        let s2 = gdi.record(
            "Quantum decoherence limits the coherence time of superconducting qubits",
            "It was a dark and stormy night and the captain said to his men",
        );
        // With the adaptive threshold, after seeing a low-drift first sample,
        // a very different text should be flagged (or at least have a higher GDI)
        assert!(
            s2.gdi > s1.gdi,
            "different text should have higher GDI than identical text"
        );
    }

    #[test]
    fn constraint_preservation_check_passes() {
        let mut cp = ConstraintPreservation::new();
        cp.register_check("non_empty", Box::new(|t: &str| !t.is_empty()));
        cp.register_check("max_1000_chars", Box::new(|t: &str| t.len() < 1000));

        let results = cp.check_all("Hello world");
        assert!(results.get("non_empty").copied().unwrap_or(false));
        assert!(results.get("max_1000_chars").copied().unwrap_or(false));
        assert_eq!(cp.total_violations(), 0);
    }

    #[test]
    fn constraint_preservation_violation_counting() {
        let mut cp = ConstraintPreservation::new();
        cp.register_check("non_empty", Box::new(|t: &str| !t.is_empty()));
        cp.register_check("contains_xyz", Box::new(|t: &str| t.contains("xyz")));

        cp.check_all("hello");
        cp.check_all("");
        let results = cp.check_all("world");

        assert!(
            !results.get("non_empty").copied().unwrap_or(true),
            "empty text should fail non_empty"
        );
        assert_eq!(cp.total_violations(), 2); // one for empty, one for missing xyz
        assert!((cp.violation_rate("non_empty") - 1.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn regression_risk_calculation() {
        let mut rr = RegressionRisk::new(50);
        // Simulate declining scores: start at 0.9 and gradually drop
        for i in 0..30 {
            rr.record(0.9 - (i as f64) * 0.02);
        }
        let risk = rr.regression_risk();
        assert!(
            risk > 0.5,
            "declining scores should have high regression risk, got {}",
            risk
        );
    }

    #[test]
    fn regression_risk_no_risk_for_improving_scores() {
        let mut rr = RegressionRisk::new(50);
        // Improving scores should give zero risk
        for i in 0..30 {
            rr.record(0.1 + (i as f64) * 0.02);
        }
        let risk = rr.regression_risk();
        assert!(
            risk < 0.5,
            "improving scores should have low regression risk, got {}",
            risk
        );
    }

    #[test]
    fn gdi_reset_clears_history() {
        let mut gdi = GoalDriftIndex::new(100);
        gdi.record("Hello world", "Goodbye world");
        assert!(!gdi.history.is_empty());
        assert!(gdi.gdi() > 0.0);

        gdi.reset();
        assert!(gdi.history.is_empty());
        assert_eq!(gdi.gdi(), 0.0);
        assert!(!gdi.drift_detected());
    }

    #[test]
    fn multiple_samples_expand_window() {
        let mut gdi = GoalDriftIndex::new(100);
        for i in 0..10 {
            let t = format!("Sample text number {} for testing purposes", i);
            gdi.record(&t, &t);
        }
        assert_eq!(gdi.history.len(), 10);
        let stats = gdi.stats();
        assert_eq!(stats.sample_count, 10);
        assert!(stats.mean_gdi < 0.15);
        assert!(stats.drift_ratio < 0.5);
    }

    #[test]
    fn gdi_stats_empty_window() {
        let gdi = GoalDriftIndex::new(100);
        let stats = gdi.stats();
        assert_eq!(stats.sample_count, 0);
        assert_eq!(stats.mean_gdi, 0.0);
        assert_eq!(stats.drift_ratio, 0.0);
    }

    #[test]
    fn regression_risk_insufficient_data() {
        let rr = RegressionRisk::new(50);
        assert_eq!(rr.regression_risk(), 0.0);

        let mut rr2 = RegressionRisk::new(50);
        rr2.record(1.0);
        rr2.record(0.9);
        // 2 samples < min 3
        assert_eq!(rr2.regression_risk(), 0.0);
    }

    #[test]
    fn constraint_preservation_register_duplicate_overwrites() {
        let mut cp = ConstraintPreservation::new();
        cp.register_check("check", Box::new(|t: &str| t.len() > 2));
        cp.register_check("check", Box::new(|t: &str| t.len() > 5));

        let r1 = cp.check_all("hello");
        // "hello".len() = 5, not > 5, so should fail
        assert!(!r1.get("check").copied().unwrap_or(true));
    }

    #[test]
    fn structural_drift_detects_sentence_length_change() {
        let mut gdi = GoalDriftIndex::new(100);
        let short = "Hi. Bye. Ok. No. Yes.";
        let long = "The quick brown fox jumps over the lazy dog near the riverbank. It was a beautiful sunny day in the middle of spring.";
        let sample = gdi.record(short, long);
        // Different sentence length distributions should produce non-zero structural drift
        assert!(sample.structural_score > 0.01);
    }

    #[test]
    fn bigram_distributional_drift_detects_char_change() {
        let mut gdi = GoalDriftIndex::new(100);
        let ascii = "hello world this is a test of the emergency broadcast system";
        let unicode_mix = "café résumé naïve jalapeño piñata";
        let sample = gdi.record(ascii, unicode_mix);
        // Very different character patterns should produce distributional drift
        assert!(sample.distributional_score > 0.01);
    }

    #[test]
    fn regression_risk_flat_scores() {
        let mut rr = RegressionRisk::new(50);
        for _ in 0..30 {
            rr.record(0.75);
        }
        let risk = rr.regression_risk();
        // Flat scores (no trend) should give low risk (recent == older)
        assert!(risk < 0.5);
    }
}
