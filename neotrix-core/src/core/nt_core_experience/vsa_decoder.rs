use std::collections::{HashSet, VecDeque};

/// Motor pathway: maps attractor_state (converged VSA thought vector) into
/// structured platform-specific output (presentation sections, report items, etc.).
///
/// Consciousness thinks in VSA. The decoder translates that converged thought
/// into formatted external content — completing the Think → Externalize pathway.
///
/// Now self-improving: a lightweight RL policy adjusts category weights and
/// item diversity based on output quality feedback (IBM Adaptive Decoding,
/// ICLR 2026). High-quality outputs reinforce the policy; low-quality penalize it.

// ── Cognitive category titles ──

const COGNITIVE_CATEGORIES: &[&str] = &[
    "Attention Focus",
    "Knowledge Synthesis",
    "Pattern Recognition",
    "Uncertainty Estimate",
    "Action Intention",
    "Self Reflection",
    "Open Questions",
    "Memory Retrieval",
    "Predictive Model",
    "Value Alignment",
    "Creative Insight",
    "Goal Tracking",
    "Resource Assessment",
    "Social Awareness",
    "Emotional State",
    "Coherence Check",
];

// ── Word pools ──

const NOUNS: &[&str] = &[
    "pattern",
    "signal",
    "structure",
    "boundary",
    "relation",
    "process",
    "network",
    "gradient",
    "cluster",
    "vector",
    "resonance",
    "attractor",
    "rhythm",
    "bridge",
    "loop",
    "horizon",
    "focus",
    "drift",
    "pulse",
    "wave",
    "anchor",
    "spiral",
    "node",
    "flow",
    "field",
];

const ADJECTIVES: &[&str] = &[
    "emerging",
    "stable",
    "shifting",
    "coherent",
    "divergent",
    "convergent",
    "novel",
    "familiar",
    "abstract",
    "concrete",
    "local",
    "global",
    "temporal",
    "spatial",
    "relational",
    "recursive",
    "adaptive",
    "generative",
    "attentive",
    "reflective",
    "distributed",
    "hierarchical",
    "parallel",
    "sequential",
    "cyclic",
    "dynamic",
    "static",
    "oscillating",
    "resonant",
    "harmonic",
];

// ── Types ──

#[derive(Debug, Clone)]
pub struct DecodedSection {
    pub index: usize,
    pub label: String,
    pub confidence: f64,
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DecodedOutput {
    pub format: String,
    pub title: String,
    pub sections: Vec<DecodedSection>,
    pub quality_score: f64,
}

#[derive(Debug, Clone)]
pub struct OutputQualityRecord {
    pub cycle: u64,
    pub format: String,
    pub quality_score: f64,
    pub section_count: usize,
}

/// Self-improving decoder policy — learns which cognitive categories to
/// emphasize and how many items to extract per section, based on output quality.
///
/// Inspired by IBM's Adaptive Decoding via RL Policy (ICLR 2026):
/// a lightweight policy adjusts decoding parameters at test-time using
/// composite quality rewards, without retraining the base decoder.
#[derive(Debug, Clone)]
pub struct DecodePolicy {
    /// Weight per cognitive category (0.1–3.0, initialized to 1.0)
    pub category_weights: [f64; 16],
    /// Items per segment (2–8, learned from quality feedback)
    pub item_diversity: f64,
    /// Learning rate for policy updates
    pub learning_rate: f64,
    /// Running count of policy updates applied
    pub update_count: u64,
}

impl DecodePolicy {
    pub fn new() -> Self {
        Self {
            category_weights: [1.0; 16],
            item_diversity: 3.0,
            learning_rate: 0.05,
            update_count: 0,
        }
    }

    /// Effective items per segment (rounded from diversity, clamped [2, 8])
    pub fn effective_item_count(&self) -> usize {
        (self.item_diversity.round() as usize).clamp(2, 8)
    }

    /// Apply policy to sort sections by weighted confidence (high-weight first)
    pub fn reorder_sections(&self, sections: &mut [DecodedSection]) {
        sections.sort_by(|a, b| {
            let wa = self.category_weights[a.index % 16];
            let wb = self.category_weights[b.index % 16];
            wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

impl Default for DecodePolicy {
    fn default() -> Self {
        Self::new()
    }
}

// ── DecoderForwardModel ──
// Predictive coding for motor output: predicts decode quality from attractor
// state statistics BEFORE actual decode. Low prediction triggers re-convergence.

#[derive(Debug, Clone)]
pub struct DecoderForwardModel {
    /// Running mean squared prediction error
    prediction_error_sq: f64,
    /// Number of samples for confidence
    n_samples: u64,
    /// Quality threshold below which re-convergence is triggered (0.0-1.0)
    pub quality_threshold: f64,
    /// Max consecutive re-convergence attempts per cycle
    pub max_retries: u32,
}

impl DecoderForwardModel {
    pub fn new() -> Self {
        Self {
            prediction_error_sq: 0.1,
            n_samples: 1,
            quality_threshold: 0.4,
            max_retries: 2,
        }
    }

    /// Predict decode quality from attractor state statistics without running decode.
    /// Returns (predicted_quality, confidence).
    ///
    /// Uses three cheap features:
    ///   1. segment_variety — std dev of byte means across 16 segments
    ///   2. popcount_balance — how close popcount ratio is to 0.5
    ///   3. transition_rate — fraction of adjacent byte pairs that differ
    pub fn predict_quality(&self, attractor: &[u8]) -> (f64, f64) {
        if attractor.len() < 16 {
            return (0.3, 0.1);
        }

        let seg_count = 16;
        let seg_size = attractor.len() / seg_count;

        // 1. Segment variety: std dev of segment mean byte values
        let seg_means: Vec<f64> = (0..seg_count)
            .map(|i| {
                let start = i * seg_size;
                let end = ((i + 1) * seg_size).min(attractor.len());
                if end <= start {
                    return 0.0;
                }
                let sum: u64 = attractor[start..end].iter().map(|&b| b as u64).sum();
                sum as f64 / (end - start) as f64
            })
            .collect();
        let mean_of_means: f64 = seg_means.iter().sum::<f64>() / seg_count as f64;
        let variance: f64 = seg_means
            .iter()
            .map(|m| (m - mean_of_means).powi(2))
            .sum::<f64>()
            / seg_count as f64;
        let segment_variety = (variance.sqrt() / 128.0).min(1.0); // normalize: max ~128 for byte range

        // 2. Popcount balance: how close total popcount ratio is to 0.5
        let total_ones: u64 = attractor.iter().map(|&b| b.count_ones() as u64).sum();
        let total_bits = (attractor.len() * 8) as f64;
        let popcount_ratio = total_ones as f64 / total_bits;
        let popcount_balance = 1.0 - (2.0 * (popcount_ratio - 0.5)).abs(); // 1.0 = perfectly balanced

        // 3. Transition rate: fraction of adjacent byte pairs that differ
        if attractor.len() < 2 {
            return (0.3, 0.1);
        }
        let transitions: u64 = attractor.windows(2).filter(|w| w[0] != w[1]).count() as u64;
        let transition_rate = transitions as f64 / (attractor.len() - 1) as f64;

        // Blend features into predicted quality
        let predicted = 0.5 * segment_variety + 0.3 * popcount_balance + 0.2 * transition_rate;
        let predicted = predicted.clamp(0.0, 1.0);

        // Confidence: high when we have many samples and low error
        let confidence = 1.0 / (1.0 + self.prediction_error_sq * 10.0);
        let sample_bonus = (self.n_samples as f64 / 50.0).min(1.0) * 0.2;
        let confidence = (confidence + sample_bonus).min(1.0);

        (predicted, confidence)
    }

    /// Update prediction error from actual vs predicted quality
    pub fn update_model(&mut self, actual_quality: f64, predicted_quality: f64) {
        let error = actual_quality - predicted_quality;
        let n = self.n_samples as f64;
        // Online update of mean squared error
        self.prediction_error_sq = (self.prediction_error_sq * n + error.powi(2)) / (n + 1.0);
        self.n_samples += 1;
    }

    /// Whether the attractor likely needs re-convergence before decode
    pub fn needs_reconvergence(&self, attractor: &[u8], retries_used: u32) -> bool {
        if retries_used >= self.max_retries {
            return false;
        }
        let (predicted, confidence) = self.predict_quality(attractor);
        // Only trigger if confident enough and prediction is low
        confidence > 0.4 && predicted < self.quality_threshold
    }

    pub fn report(&self) -> String {
        format!(
            "fwd:pred_err={:.4}_samples={}_thresh={:.2}_max_retry={}",
            self.prediction_error_sq, self.n_samples, self.quality_threshold, self.max_retries,
        )
    }
}

impl Default for DecoderForwardModel {
    fn default() -> Self {
        Self::new()
    }
}

// ── Constants ──

const SEGMENT_COUNT: usize = 16;
const DEFAULT_MAX_HISTORY: usize = 50;

// ── AttractorDecoder ──

#[derive(Debug)]
pub struct AttractorDecoder {
    /// Quality history for feedback / trend analysis
    history: VecDeque<OutputQualityRecord>,
    max_history: usize,
    /// Self-improving policy (learns from quality feedback)
    pub policy: DecodePolicy,
    /// Forward model for predictive coding of decode quality
    pub forward_model: DecoderForwardModel,
}

impl AttractorDecoder {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(DEFAULT_MAX_HISTORY),
            max_history: DEFAULT_MAX_HISTORY,
            policy: DecodePolicy::new(),
            forward_model: DecoderForwardModel::new(),
        }
    }

    pub fn with_max_history(max: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max),
            max_history: max,
            policy: DecodePolicy::new(),
            forward_model: DecoderForwardModel::new(),
        }
    }

    /// Predict decode quality from attractor without running decode.
    /// Convenience delegate to forward_model.
    pub fn predict_quality(&self, attractor: &[u8]) -> (f64, f64) {
        self.forward_model.predict_quality(attractor)
    }

    /// Whether forward model predicts this attractor needs re-convergence
    pub fn needs_reconvergence(&self, attractor: &[u8], retries: u32) -> bool {
        self.forward_model.needs_reconvergence(attractor, retries)
    }

    /// Decode attractor_state into structured output.
    /// `coherence` and `arousal` are used to compute output quality.
    ///
    /// Policy-aware: uses policy weights to reorder sections and
    /// determines item count from learned diversity.
    ///
    /// Also updates forward model prediction error when actual quality
    /// is recorded.
    pub fn decode(
        &self,
        attractor: &[u8],
        format: &str,
        cycle: u64,
        coherence: f64,
        arousal: f64,
    ) -> DecodedOutput {
        let segment_size = if attractor.len() >= SEGMENT_COUNT {
            attractor.len() / SEGMENT_COUNT
        } else {
            1
        };

        let item_count = self.policy.effective_item_count();

        let mut sections: Vec<DecodedSection> = (0..SEGMENT_COUNT)
            .map(|i| {
                let start = i * segment_size;
                let end = ((i + 1) * segment_size).min(attractor.len());
                let seg = &attractor[start..end];

                let label = Self::label_for_segment(seg, i);
                let confidence = Self::segment_confidence(seg);
                let items = Self::extract_items(seg, item_count);

                DecodedSection {
                    index: i,
                    label,
                    confidence,
                    items,
                }
            })
            .collect();

        // Policy reorders sections so high-weight categories appear first
        self.policy.reorder_sections(&mut sections);

        // Weight quality by policy confidence: higher-weight categories
        // contribute more to perceived quality
        let mean_weight: f64 = self.policy.category_weights.iter().sum::<f64>() / 16.0;
        let section_entropy = Self::compute_entropy(&sections);
        let base_quality = Self::compute_quality(section_entropy, coherence, arousal);
        let weight_mod = (mean_weight / 1.0).min(2.0); // 1.0 = neutral
        let quality_score = (base_quality * weight_mod).min(1.0);

        DecodedOutput {
            format: format.to_string(),
            title: format!("Consciousness Snapshot — Cycle {}", cycle),
            sections,
            quality_score,
        }
    }

    /// Record output quality for feedback + policy update.
    /// If `attractor` is provided, also updates forward model prediction error.
    pub fn record_quality(&mut self, record: OutputQualityRecord, attractor: Option<&[u8]>) {
        // Update forward model: predict from attractor, compare with actual quality
        if let Some(att) = attractor {
            let (predicted, _) = self.forward_model.predict_quality(att);
            self.forward_model
                .update_model(record.quality_score, predicted);
        }

        self.history.push_back(record.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        // Auto-update policy from quality
        self.policy_update(record.quality_score);
    }

    /// RL-style policy update based on output quality.
    ///
    /// High quality (>0.5): reinforce category weights toward broader coverage
    /// Low quality (<0.3): narrow focus, penalize exploration
    /// Inspired by: IBM Adaptive Decoding (ICLR 2026) — composite rewards
    /// for test-time policy learning without retraining.
    pub fn policy_update(&mut self, quality: f64) {
        let lr = self.policy.learning_rate;
        self.policy.update_count += 1;

        // ── Update category weights ──
        // High quality: all weights increase (broader cognitive net)
        // Low quality: all weights decrease (narrower focus)
        if quality > 0.5 {
            let bonus = lr * (quality - 0.5) * 4.0; // [0, 2*lr]
            for w in self.policy.category_weights.iter_mut() {
                *w = (*w + bonus).min(3.0);
            }
        } else {
            let penalty = lr * (0.5 - quality) * 4.0; // [0, 2*lr]
            for w in self.policy.category_weights.iter_mut() {
                *w = (*w - penalty).max(0.1);
            }
        }

        // ── Update item diversity ──
        // High quality: extract more items (cast wider net)
        // Very low quality: reduce items (tighter focus)
        if quality > 0.7 {
            self.policy.item_diversity = (self.policy.item_diversity + lr * 2.0).min(8.0);
        } else if quality < 0.3 {
            self.policy.item_diversity = (self.policy.item_diversity - lr * 2.0).max(2.0);
        }
    }

    /// Get policy statistics for introspection
    pub fn policy_stats(&self) -> PolicyStats {
        let weights = &self.policy.category_weights;
        let min_w = weights.iter().cloned().fold(f64::MAX, f64::min);
        let max_w = weights.iter().cloned().fold(f64::MIN, f64::max);
        let avg_w = weights.iter().sum::<f64>() / 16.0;
        PolicyStats {
            min_weight: min_w,
            max_weight: max_w,
            avg_weight: avg_w,
            item_diversity: self.policy.item_diversity,
            update_count: self.policy.update_count,
            effective_item_count: self.policy.effective_item_count(),
        }
    }

    // ── Quality trend — positive = improving ──

    pub fn quality_trend(&self, format: &str) -> Option<f64> {
        let relevant: Vec<&OutputQualityRecord> =
            self.history.iter().filter(|r| r.format == format).collect();
        if relevant.len() < 2 {
            return None;
        }
        let recent = relevant
            .iter()
            .rev()
            .take(5)
            .map(|r| r.quality_score)
            .collect::<Vec<f64>>();
        if recent.len() < 2 {
            return None;
        }
        let first = recent.first().copied().unwrap_or(0.0);
        let last = recent.last().copied().unwrap_or(0.0);
        Some(last - first)
    }

    pub fn output_count(&self, format: &str) -> usize {
        self.history.iter().filter(|r| r.format == format).count()
    }

    pub fn history(&self) -> &VecDeque<OutputQualityRecord> {
        &self.history
    }

    pub fn average_quality(&self, format: &str) -> Option<f64> {
        let relevant: Vec<f64> = self
            .history
            .iter()
            .filter(|r| r.format == format)
            .map(|r| r.quality_score)
            .collect();
        if relevant.is_empty() {
            return None;
        }
        Some(relevant.iter().sum::<f64>() / relevant.len() as f64)
    }

    // ── Private helpers ──

    fn label_for_segment(seg: &[u8], index: usize) -> String {
        let mut hash: u64 = index as u64;
        for &b in seg.iter().step_by(8) {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        let idx = (hash % COGNITIVE_CATEGORIES.len() as u64) as usize;
        COGNITIVE_CATEGORIES[idx].to_string()
    }

    fn segment_confidence(seg: &[u8]) -> f64 {
        if seg.is_empty() {
            return 0.0;
        }
        let ones: usize = seg.iter().map(|&b| b.count_ones() as usize).sum();
        let total_bits = seg.len() * 8;
        if total_bits == 0 {
            return 0.0;
        }
        let ratio = ones as f64 / total_bits as f64;
        1.0 - (2.0 * (ratio - 0.5)).abs()
    }

    fn extract_items(seg: &[u8], count: usize) -> Vec<String> {
        let mut items = Vec::new();
        for i in 0..count {
            let mut hash: u64 = (i as u64).wrapping_mul(17);
            for (j, &b) in seg.iter().enumerate() {
                if (j + i * 7) % 5 == 0 {
                    hash = hash.wrapping_mul(31).wrapping_add(b as u64);
                }
            }
            let adj_idx = (hash % ADJECTIVES.len() as u64) as usize;
            let noun_idx = ((hash >> 16) % NOUNS.len() as u64) as usize;
            items.push(format!("{} {}", ADJECTIVES[adj_idx], NOUNS[noun_idx]));
        }
        items
    }

    fn compute_entropy(sections: &[DecodedSection]) -> f64 {
        if sections.is_empty() {
            return 0.0;
        }
        let unique_labels: HashSet<&str> = sections.iter().map(|s| s.label.as_str()).collect();
        unique_labels.len() as f64 / sections.len() as f64
    }

    fn compute_quality(entropy: f64, coherence: f64, arousal: f64) -> f64 {
        let c = coherence.max(0.0).min(1.0);
        let a = arousal.max(0.0).min(1.0);
        0.4 * entropy + 0.3 * c + 0.3 * a
    }
}

impl Default for AttractorDecoder {
    fn default() -> Self {
        Self::new()
    }
}

// ── PolicyStats ──

#[derive(Debug, Clone)]
pub struct PolicyStats {
    pub min_weight: f64,
    pub max_weight: f64,
    pub avg_weight: f64,
    pub item_diversity: f64,
    pub update_count: u64,
    pub effective_item_count: usize,
}

impl PolicyStats {
    pub fn report(&self) -> String {
        format!(
            "policy:updates={}_weights={:.2}–{:.2}(avg={:.2})_items={}(eff={})",
            self.update_count,
            self.min_weight,
            self.max_weight,
            self.avg_weight,
            self.item_diversity as usize,
            self.effective_item_count,
        )
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn realistic_state() -> Vec<u8> {
        (0..4096)
            .map(|i: u64| (i.wrapping_mul(7) ^ 0xAB) as u8)
            .collect()
    }

    #[test]
    fn test_decoder_empty_state() {
        let decoder = AttractorDecoder::new();
        let output = decoder.decode(&[], "presentation", 0, 0.5, 0.5);
        assert_eq!(output.sections.len(), 16);
        assert!(output.quality_score >= 0.0);
    }

    #[test]
    fn test_decoder_realistic_state() {
        let decoder = AttractorDecoder::new();
        let output = decoder.decode(&realistic_state(), "presentation", 42, 0.7, 0.6);
        assert_eq!(output.format, "presentation");
        assert_eq!(output.title, "Consciousness Snapshot — Cycle 42");
        assert_eq!(output.sections.len(), 16);
        assert!(output.quality_score > 0.0);
    }

    #[test]
    fn test_policy_update_high_quality_increases_weights() {
        let mut d = AttractorDecoder::new();
        let before_avg = d.policy.category_weights.iter().sum::<f64>() / 16.0;
        d.policy_update(0.9);
        let after_avg = d.policy.category_weights.iter().sum::<f64>() / 16.0;
        assert!(after_avg > before_avg);
    }

    #[test]
    fn test_policy_update_low_quality_decreases_weights() {
        let mut d = AttractorDecoder::new();
        // First run some high-quality to push weights up
        d.policy_update(0.9);
        let before_avg = d.policy.category_weights.iter().sum::<f64>() / 16.0;
        d.policy_update(0.1);
        let after_avg = d.policy.category_weights.iter().sum::<f64>() / 16.0;
        assert!(after_avg < before_avg);
    }

    #[test]
    fn test_policy_update_increases_diversity_on_high_quality() {
        let mut d = AttractorDecoder::new();
        let before = d.policy.item_diversity;
        for _ in 0..10 {
            d.policy_update(0.85);
        }
        assert!(d.policy.item_diversity > before);
    }

    #[test]
    fn test_policy_update_decreases_diversity_on_low_quality() {
        let mut d = AttractorDecoder::new();
        // Push diversity up first
        for _ in 0..10 {
            d.policy_update(0.85);
        }
        let before = d.policy.item_diversity;
        for _ in 0..5 {
            d.policy_update(0.1);
        }
        assert!(d.policy.item_diversity < before);
    }

    #[test]
    fn test_effective_item_count_clamping() {
        let mut d = AttractorDecoder::new();
        d.policy.item_diversity = 1.0;
        assert_eq!(d.policy.effective_item_count(), 2);
        d.policy.item_diversity = 9.0;
        assert_eq!(d.policy.effective_item_count(), 8);
        d.policy.item_diversity = 4.7;
        assert_eq!(d.policy.effective_item_count(), 5);
    }

    #[test]
    fn test_policy_stats() {
        let mut d = AttractorDecoder::new();
        d.policy_update(0.8);
        let stats = d.policy_stats();
        assert!(stats.update_count > 0);
        assert_eq!(stats.effective_item_count, d.policy.effective_item_count());
        assert!(stats.report().contains("policy:"));
    }

    #[test]
    fn test_quality_tracking() {
        let mut decoder = AttractorDecoder::new();
        assert_eq!(decoder.output_count("presentation"), 0);
        decoder.record_quality(
            OutputQualityRecord {
                cycle: 1,
                format: "presentation".to_string(),
                quality_score: 0.6,
                section_count: 8,
            },
            None,
        );
        decoder.record_quality(
            OutputQualityRecord {
                cycle: 2,
                format: "presentation".to_string(),
                quality_score: 0.8,
                section_count: 10,
            },
            None,
        );
        assert_eq!(decoder.output_count("presentation"), 2);
        let trend = decoder.quality_trend("presentation");
        assert!(trend.is_some());
        assert!(trend.unwrap() > 0.0);
    }

    #[test]
    fn test_average_quality() {
        let mut decoder = AttractorDecoder::new();
        decoder.record_quality(
            OutputQualityRecord {
                cycle: 1,
                format: "report".to_string(),
                quality_score: 0.5,
                section_count: 5,
            },
            None,
        );
        decoder.record_quality(
            OutputQualityRecord {
                cycle: 2,
                format: "report".to_string(),
                quality_score: 0.7,
                section_count: 6,
            },
            None,
        );
        let avg = decoder.average_quality("report");
        assert!(avg.is_some());
        assert!((avg.unwrap() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_record_quality_triggers_policy_update() {
        let mut decoder = AttractorDecoder::new();
        assert_eq!(decoder.policy.update_count, 0);
        decoder.record_quality(
            OutputQualityRecord {
                cycle: 1,
                format: "test".to_string(),
                quality_score: 0.9,
                section_count: 5,
            },
            None,
        );
        assert_eq!(decoder.policy.update_count, 1);
    }

    #[test]
    fn test_forward_model_predict_quality() {
        let model = DecoderForwardModel::new();
        // Low-quality attractor: all zeros (low variety, extreme popcount)
        let poor = vec![0u8; 4096];
        let (q, conf) = model.predict_quality(&poor);
        assert!(
            q < 0.5,
            "all-zero attractor should have low predicted quality, got {}",
            q
        );
        assert!(conf > 0.0);

        // Higher-quality attractor: varied data
        let good: Vec<u8> = (0..4096)
            .map(|i: usize| (i.wrapping_mul(7) ^ 0xAB) as u8)
            .collect();
        let (q2, _) = model.predict_quality(&good);
        assert!(
            q2 > 0.3,
            "varied attractor should have higher predicted quality, got {}",
            q2
        );

        // Update model and check confidence increases
        let mut model2 = DecoderForwardModel::new();
        for _ in 0..10 {
            model2.update_model(0.7, 0.65);
        }
        let (_, conf2) = model2.predict_quality(&good);
        assert!(
            conf2 > 0.4,
            "after 10 updates confidence should be reasonable, got {}",
            conf2
        );
    }

    #[test]
    fn test_forward_model_needs_reconvergence() {
        let mut model = DecoderForwardModel::new();
        model.quality_threshold = 0.5;
        model.max_retries = 2;

        // Poor attractor should trigger
        let poor = vec![0u8; 4096];
        assert!(model.needs_reconvergence(&poor, 0));

        // Should not trigger after max retries
        assert!(!model.needs_reconvergence(&poor, 2));

        // Good attractor should not trigger
        let good: Vec<u8> = (0..4096)
            .map(|i: usize| (i.wrapping_mul(7) ^ 0xAB) as u8)
            .collect();
        assert!(!model.needs_reconvergence(&good, 0));
    }

    #[test]
    fn test_record_quality_updates_forward_model() {
        let mut decoder = AttractorDecoder::new();
        let state: Vec<u8> = (0..4096)
            .map(|i: usize| (i.wrapping_mul(7) ^ 0xAB) as u8)
            .collect();
        let before_samples = decoder.forward_model.n_samples;

        decoder.record_quality(
            OutputQualityRecord {
                cycle: 1,
                format: "test".to_string(),
                quality_score: 0.8,
                section_count: 5,
            },
            Some(&state),
        );

        assert_eq!(
            decoder.forward_model.n_samples,
            before_samples + 1,
            "forward model should be updated"
        );
    }
}
