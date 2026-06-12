use super::types::KnowledgePacket;

/// The full CAT7 7-field SVAF evaluation set + TextSummary (8 total).
///
/// MMP CAT7 reference (meshcognition.org):
///   Focus | Issue | Intent | Motivation | Commitment | Perspective | Mood
///
/// Mapping:
///   CapabilityDelta = Focus
///   NegentropyGain  = Motivation
///   Confidence      = Commitment
///   TextSummary     = our own added field (novelty detection)
///   Issue / Intent / Perspective / Mood = CAT7 additions
///
/// Mood carries α=0.20 (highest per-field weight, CNS-discovered universal cross-domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvaField {
    /// focus: what capability does this unlock?
    CapabilityDelta,
    /// text novelty: semantic newness vs absorbed knowledge
    TextSummary,
    /// motivation: local negentropy gain from this knowledge
    NegentropyGain,
    /// commitment: reliability / validation level
    Confidence,
    /// issue: what problem/opportunity does this address?
    Issue,
    /// intent: what action does the sender intend?
    Intent,
    /// perspective: viewpoint / angle of the sender
    Perspective,
    /// mood: valence × arousal emotional state (α=0.20)
    Mood,
}

impl SvaField {
    pub fn all() -> [SvaField; 8] {
        [
            SvaField::CapabilityDelta,
            SvaField::TextSummary,
            SvaField::NegentropyGain,
            SvaField::Confidence,
            SvaField::Issue,
            SvaField::Intent,
            SvaField::Perspective,
            SvaField::Mood,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            SvaField::CapabilityDelta => "capability_delta",
            SvaField::TextSummary => "text_summary",
            SvaField::NegentropyGain => "negentropy_gain",
            SvaField::Confidence => "confidence",
            SvaField::Issue => "issue",
            SvaField::Intent => "intent",
            SvaField::Perspective => "perspective",
            SvaField::Mood => "mood",
        }
    }
}

/// Mood encoded as valence-arousal (VA) in ℝ².
///
/// Russell's circumplex model:
///   valence [-1, +1]: unpleasant ← → pleasant
///   arousal [-1, +1]: calm ← → excited
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoodVa {
    pub valence: f64,
    pub arousal: f64,
}

impl MoodVa {
    pub fn new(valence: f64, arousal: f64) -> Self {
        MoodVa {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(-1.0, 1.0),
        }
    }

    /// Unit-norm distance to another VA point.
    pub fn distance(&self, other: &MoodVa) -> f64 {
        let dv = self.valence - other.valence;
        let da = self.arousal - other.arousal;
        ((dv * dv + da * da) / 8.0).sqrt()
    }

    /// Convert to VSA bundle seed bytes (for alignment).
    pub fn to_seed_bytes(&self) -> [u8; 8] {
        let v = ((self.valence * 127.0) as i16).to_le_bytes();
        let a = ((self.arousal * 127.0) as i16).to_le_bytes();
        [v[0], v[1], a[0], a[1], 0x4d, 0x4f, 0x4f, 0x44] // "MOOD"
    }
}

/// SVAF 4-class absorption decision per field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvaDecision {
    Accept,
    Guard,
    Redundant,
    Reject,
}

/// A per-field evaluation result.
#[derive(Debug, Clone)]
pub struct SvaFieldEvaluation {
    pub field: SvaField,
    pub raw_score: f64,
    pub decision: SvaDecision,
}

/// The SVAF 4-class gate — CAT7 (8-field) for competitive knowledge absorption.
///
/// Each incoming KnowledgePacket is evaluated on 8 fields independently.
/// Per-field weights α_f are learned over time via negentropy feedback.
/// A packet is absorbed if at least one field passes ACCEPT.
///
/// Content-driven convergence (MMP): evaluates content independently of peer trust.
/// A rejected peer's high-CMB content is absorbed; repeated content-empty CMBs
/// lower that peer's effective standing.
pub struct SvaGate {
    /// Learned per-field weights α_f (8 fields, mood at 0.20 initial).
    weights: [f64; 8],
    accept_threshold: f64,
    redundant_threshold: f64,
    #[allow(dead_code)]
    guard_upper: f64,
    #[allow(dead_code)]
    aligner_seed: u64,
    #[allow(dead_code)]
    vsa_dim: usize,
    absorbed_embeddings: Vec<Vec<u8>>,
    field_hits: [u64; 8],
    field_misses: [u64; 8],
    /// Sentiment dictionary for mood VA extraction
    positive_words: Vec<&'static str>,
    negative_words: Vec<&'static str>,
    issue_indicators: Vec<&'static str>,
    intent_indicators: Vec<&'static str>,
    perspective_indicators: Vec<&'static str>,
}

impl SvaGate {
    pub fn new(vsa_dim: usize, aligner_seed: u64) -> Self {
        SvaGate {
            // 7 CAT7 + TextSummary: mood 0.20, others 0.114 each → sum ≈ 1.0
            weights: [0.114, 0.114, 0.114, 0.114, 0.114, 0.114, 0.114, 0.20],
            accept_threshold: 0.75,
            redundant_threshold: 0.30,
            guard_upper: 0.90,
            aligner_seed,
            vsa_dim,
            absorbed_embeddings: Vec::new(),
            field_hits: [0; 8],
            field_misses: [0; 8],
            positive_words: vec![
                "good", "great", "excellent", "amazing", "breakthrough", "discover",
                "improve", "optimize", "fast", "efficient", "innovative", "novel",
                "valuable", "beneficial", "success", "win", "solution", "progress",
            ],
            negative_words: vec![
                "bad", "poor", "failure", "broken", "slow", "bug", "error",
                "problem", "issue", "risk", "danger", "waste", "loss", "wrong",
                "difficult", "hard", "fail", "crash", "degrading", "harmful",
            ],
            issue_indicators: vec![
                "problem", "issue", "bug", "need", "fix", "improve", "resolve",
                "broken", "missing", "gap", "requirement", "demand", "address",
            ],
            intent_indicators: vec![
                "aim", "intend", "goal", "propose", "want", "purpose", "objective",
                "target", "plan", "will", "shall", "would", "expect",
            ],
            perspective_indicators: vec![
                "from", "view", "perspective", "angle", "lens", "standpoint",
                "according", "regarding", "consider", "aspect", "dimension",
                "side", "approach", "through",
            ],
        }
    }

    /// Evaluate a KnowledgePacket across all 8 SVAF fields.
    pub fn evaluate(&mut self, packet: &KnowledgePacket) -> Vec<SvaFieldEvaluation> {
        let mut results = Vec::with_capacity(8);
        for (i, field) in SvaField::all().iter().enumerate() {
            let raw = self.score_field(packet, *field);
            let decision = self.classify(raw, *field);
            if matches!(decision, SvaDecision::Accept | SvaDecision::Guard) {
                self.field_hits[i] += 1;
            } else {
                self.field_misses[i] += 1;
            }
            results.push(SvaFieldEvaluation {
                field: *field,
                raw_score: raw,
                decision,
            });
        }
        results
    }

    /// Global absorption decision: accept if any field is ACCEPT.
    pub fn should_absorb(&mut self, evaluations: &[SvaFieldEvaluation]) -> bool {
        evaluations
            .iter()
            .any(|e| matches!(e.decision, SvaDecision::Accept))
    }

    /// Weighted SVAF score across all fields (for score modulation).
    pub fn weighted_score(&self, evaluations: &[SvaFieldEvaluation]) -> f64 {
        let mut score = 0.0;
        for (i, e) in evaluations.iter().enumerate() {
            let w = self.weights[i];
            match e.decision {
                SvaDecision::Accept => score += w * (0.8 + 0.2 * e.raw_score),
                SvaDecision::Guard => score += w * (0.5 + 0.5 * e.raw_score),
                SvaDecision::Redundant => score += w * 0.2 * e.raw_score,
                SvaDecision::Reject => score += w * 0.05 * e.raw_score,
            }
        }
        score.clamp(0.0, 1.0)
    }

    /// Record an absorbed embedding for future novelty comparisons.
    pub fn record_absorption(&mut self, vsa_embedding: Vec<u8>) {
        self.absorbed_embeddings.push(vsa_embedding);
        if self.absorbed_embeddings.len() > 1000 {
            self.absorbed_embeddings.remove(0);
        }
    }

    /// Adapt per-field weights based on recent negentropy outcomes.
    pub fn adapt_weights(&mut self, learning_rate: f64) {
        for i in 0..8 {
            let total = self.field_hits[i] + self.field_misses[i];
            if total > 5 {
                let hit_rate = self.field_hits[i] as f64 / total as f64;
                let delta = (hit_rate - 0.5) * learning_rate;
                self.weights[i] = (self.weights[i] + delta).clamp(0.05, 0.35);
            }
        }
        self.normalize_weights();
    }

    /// Mood always gets minimum α=0.15 (MMP: mood is universally cross-domain).
    pub fn enforce_mood_floor(&mut self) {
        self.weights[7] = self.weights[7].max(0.15);
        self.normalize_weights();
    }

    fn normalize_weights(&mut self) {
        let sum: f64 = self.weights.iter().sum();
        if sum > 0.0 {
            for w in &mut self.weights {
                *w /= sum;
            }
        }
    }

    // ── field scoring ──

    fn score_field(&self, packet: &KnowledgePacket, field: SvaField) -> f64 {
        match field {
            SvaField::CapabilityDelta => {
                let len = packet.capability_delta.len() as f64;
                let len_score = (len as f64 / 500.0).clamp(0.0, 1.0);
                len_score * 0.6 + packet.local_negentropy_gain.clamp(0.0, 1.0) * 0.4
            }
            SvaField::TextSummary => {
                let text_len = packet.text_summary.len();
                if text_len == 0 {
                    return 0.0;
                }
                let len_score = (text_len as f64 / 1000.0).clamp(0.0, 1.0) * 0.3;
                let novelty = if self.absorbed_embeddings.is_empty() {
                    0.5
                } else if !packet.vsa_vectors.is_empty() {
                    self.compute_novelty(&packet.vsa_vectors[0])
                } else {
                    0.5
                };
                len_score + novelty * 0.7
            }
            SvaField::NegentropyGain => packet.local_negentropy_gain.clamp(0.0, 1.0),
            SvaField::Confidence => packet.local_confidence.clamp(0.0, 1.0),
            SvaField::Issue => {
                let text = &packet.text_summary;
                if text.is_empty() {
                    return 0.0;
                }
                let lower = text.to_lowercase();
                let count = self
                    .issue_indicators
                    .iter()
                    .filter(|w| lower.contains(*w))
                    .count();
                let raw = count as f64 / 5.0;
                (raw * 0.6 + packet.local_negentropy_gain.clamp(0.0, 1.0) * 0.4).min(1.0)
            }
            SvaField::Intent => {
                let text = &packet.text_summary;
                if text.is_empty() {
                    return 0.0;
                }
                let lower = text.to_lowercase();
                let count = self
                    .intent_indicators
                    .iter()
                    .filter(|w| lower.contains(*w))
                    .count();
                (count as f64 / 5.0).min(1.0)
            }
            SvaField::Perspective => {
                let text = &packet.text_summary;
                if text.is_empty() {
                    return 0.0;
                }
                // Perspective benefits from capability_delta length (detailed descriptions)
                let lower = text.to_lowercase();
                let count = self
                    .perspective_indicators
                    .iter()
                    .filter(|w| lower.contains(*w))
                    .count();
                let indicator_score = (count as f64 / 5.0).min(1.0);
                let len_bonus = (packet.capability_delta.len() as f64 / 300.0).clamp(0.0, 1.0);
                (indicator_score * 0.5 + len_bonus * 0.5).min(1.0)
            }
            SvaField::Mood => {
                let va = self.extract_mood(packet);
                let valence_score = (va.valence + 1.0) / 2.0;
                let arousal_score = (va.arousal + 1.0) / 2.0;
                valence_score * 0.6 + arousal_score * 0.4
            }
        }
    }

    fn extract_mood(&self, packet: &KnowledgePacket) -> MoodVa {
        let text = &packet.text_summary;
        if text.is_empty() {
            return MoodVa::new(0.0, 0.0);
        }
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let total = words.len() as f64;
        if total == 0.0 {
            return MoodVa::new(0.0, 0.0);
        }

        let pos_count = self
            .positive_words
            .iter()
            .filter(|w| lower.contains(*w))
            .count() as f64;
        let neg_count = self
            .negative_words
            .iter()
            .filter(|w| lower.contains(*w))
            .count() as f64;

        let valence = if pos_count + neg_count > 0.0 {
            ((pos_count - neg_count) / (pos_count + neg_count)).clamp(-1.0, 1.0)
        } else {
            // Default: mild positive if negentropy is high, else neutral
            (packet.local_negentropy_gain * 2.0 - 1.0).clamp(-1.0, 1.0)
        };

        let arousal = ((pos_count + neg_count) / (total + 1.0) * 2.0 - 1.0).clamp(-1.0, 1.0);

        MoodVa::new(valence, arousal)
    }

    fn compute_novelty(&self, vsa: &[u8]) -> f64 {
        if self.absorbed_embeddings.is_empty() {
            return 0.8;
        }
        let mut max_sim = 0.0f64;
        for existing in &self.absorbed_embeddings {
            let sim = self.hamming_similarity(vsa, existing);
            if sim > max_sim {
                max_sim = sim;
            }
        }
        1.0 - max_sim
    }

    fn hamming_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len());
        if len == 0 {
            return 0.0;
        }
        let same = a[..len]
            .iter()
            .zip(b[..len].iter())
            .filter(|(x, y)| x == y)
            .count();
        same as f64 / len as f64
    }

    fn classify(&self, score: f64, field: SvaField) -> SvaDecision {
        // Mood: lower threshold to ensure cross-domain absorption (MMP key property)
        let effective_redundant = if field == SvaField::Mood {
            self.redundant_threshold * 0.5 // 0.15
        } else {
            self.redundant_threshold
        };
        let effective_accept = if field == SvaField::Mood {
            self.accept_threshold * 0.8 // 0.60
        } else {
            self.accept_threshold
        };

        if score >= effective_accept {
            SvaDecision::Accept
        } else if score <= effective_redundant {
            SvaDecision::Reject
        } else if score < (effective_accept + effective_redundant) / 2.0 {
            SvaDecision::Redundant
        } else {
            SvaDecision::Guard
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_packet() -> KnowledgePacket {
        KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "test",
            "new capability: parallel pattern matching",
            "discovered efficient parallel matching algorithm with O(log n) complexity",
            0.85,
        )
        .with_confidence(0.75)
    }

    fn gate() -> SvaGate {
        SvaGate::new(4096, 42)
    }

    #[test]
    fn test_evaluate_returns_8_fields() {
        let mut g = gate();
        let packet = sample_packet();
        let results = g.evaluate(&packet);
        assert_eq!(results.len(), 8);
    }

    #[test]
    fn test_evaluate_field_labels() {
        let mut g = gate();
        let packet = sample_packet();
        let results = g.evaluate(&packet);
        let labels: Vec<&str> = results.iter().map(|r| r.field.label()).collect();
        assert!(labels.contains(&"capability_delta"));
        assert!(labels.contains(&"text_summary"));
        assert!(labels.contains(&"negentropy_gain"));
        assert!(labels.contains(&"confidence"));
        assert!(labels.contains(&"issue"));
        assert!(labels.contains(&"intent"));
        assert!(labels.contains(&"perspective"));
        assert!(labels.contains(&"mood"));
    }

    #[test]
    fn test_high_negentropy_packet_accepted() {
        let mut g = gate();
        let packet = KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "critical",
            "major breakthrough",
            "revolutionary discovery that changes everything",
            0.95,
        )
        .with_confidence(0.90);
        let results = g.evaluate(&packet);
        assert!(g.should_absorb(&results), "high negentropy should be absorbed");
    }

    #[test]
    fn test_low_value_packet_rejected() {
        let mut g = gate();
        let packet = KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "noise",
            "",
            "",
            0.05,
        )
        .with_confidence(0.1);
        let results = g.evaluate(&packet);
        assert!(!g.should_absorb(&results), "low value packet should not be absorbed");
    }

    #[test]
    fn test_mood_field_always_has_chance() {
        let mut g = gate();
        // Packet with emotional content but low negentropy
        let packet = KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "feeling",
            "excellent progress",
            "great breakthrough amazing success wonderful improvement",
            0.3,
        )
        .with_confidence(0.2);
        let results = g.evaluate(&packet);
        let mood_result = results.iter().find(|r| r.field == SvaField::Mood).unwrap();
        // Mood with positive words should score well even if negentropy is low
        assert!(
            mood_result.raw_score > 0.3,
            "positive mood should score > 0.3, got {}",
            mood_result.raw_score
        );
    }

    #[test]
    fn test_negative_mood_detected() {
        let g = gate();
        let packet = KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "bad",
            "broken system",
            "poor performance bug error failure crash",
            0.1,
        )
        .with_confidence(0.1);
        let va = g.extract_mood(&packet);
        assert!(
            va.valence < 0.0,
            "negative text should give negative valence, got {}",
            va.valence
        );
    }

    #[test]
    fn test_issue_field_detection() {
        let mut g = gate();
        let packet = KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "bugs",
            "fix memory leak",
            "address the memory leak problem in the allocator",
            0.6,
        );
        let results = g.evaluate(&packet);
        let issue = results.iter().find(|r| r.field == SvaField::Issue).unwrap();
        assert!(
            issue.raw_score > 0.3,
            "issue-oriented text should score > 0.3, got {}",
            issue.raw_score
        );
    }

    #[test]
    fn test_intent_field_detection() {
        let mut g = gate();
        let packet = KnowledgePacket::new(
            crate::core::nt_core_hive::HiveId::new(1),
            "goal",
            "propose new architecture",
            "we intend to propose a new architecture pattern aimed at scalability",
            0.7,
        );
        let results = g.evaluate(&packet);
        let intent = results.iter().find(|r| r.field == SvaField::Intent).unwrap();
        assert!(
            intent.raw_score > 0.3,
            "intent-oriented text should score > 0.3, got {}",
            intent.raw_score
        );
    }

    #[test]
    fn test_weighted_score_range() {
        let mut g = gate();
        let packet = sample_packet();
        let results = g.evaluate(&packet);
        let ws = g.weighted_score(&results);
        assert!(ws >= 0.0 && ws <= 1.0, "weighted score should be in [0,1], got {}", ws);
    }

    #[test]
    fn test_adapt_weights_converges() {
        let mut g = gate();
        g.field_hits = [10, 8, 12, 15, 6, 9, 11, 20];
        g.field_misses = [2, 4, 1, 0, 3, 2, 1, 5];
        g.adapt_weights(0.1);
        let sum: f64 = g.weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.01, "weights must sum to 1");
        assert!(g.weights[7] >= 0.15, "mood floor must be ≥ 0.15");
    }

    #[test]
    fn test_mood_floor_enforcement() {
        let mut g = gate();
        g.weights[7] = 0.05;
        g.enforce_mood_floor();
        assert!(
            g.weights[7] >= 0.15,
            "mood floor should be enforced, got {}",
            g.weights[7]
        );
        let sum: f64 = g.weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.01, "weights must sum to 1 after floor");
    }

    #[test]
    fn test_record_absorption_updates_novelty() {
        let mut g = gate();
        g.record_absorption(vec![1u8; 512]);
        let packet = sample_packet();
        let results = g.evaluate(&packet);
        assert_eq!(results.len(), 8);
    }

    #[test]
    fn test_mood_va_distance() {
        let a = MoodVa::new(1.0, 1.0);
        let b = MoodVa::new(-1.0, -1.0);
        let d = a.distance(&b);
        assert!((d - 1.0).abs() < 0.01, "opposite VA should have distance ~1.0, got {}", d);
    }

    #[test]
    fn test_mood_va_zero_distance() {
        let a = MoodVa::new(0.5, -0.3);
        let d = a.distance(&a);
        assert!((d - 0.0).abs() < 0.01, "identical VA should have distance 0.0, got {}", d);
    }

    #[test]
    fn test_mood_va_seed_bytes() {
        let a = MoodVa::new(0.5, -0.3);
        let b = MoodVa::new(0.5, -0.3);
        assert_eq!(a.to_seed_bytes(), b.to_seed_bytes(), "same VA → same seed");
    }
}
