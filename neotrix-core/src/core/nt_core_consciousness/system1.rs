use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// Identifies which intuition heuristic produced a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum IntuitionHeuristic {
    Similarity, // Pattern match: "this looks like X"
    Frequency,  // Common occurrence: "this happens often"
    Proximity,  // Temporal proximity: "this just happened"
    Recency,    // Recent use: "we used this recently"
    Emotion,    // Emotional association: "this feels like when..."
}

impl IntuitionHeuristic {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Similarity => "similarity",
            Self::Frequency => "frequency",
            Self::Proximity => "proximity",
            Self::Recency => "recency",
            Self::Emotion => "emotion",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Similarity,
            Self::Frequency,
            Self::Proximity,
            Self::Recency,
            Self::Emotion,
        ]
    }
}

/// A single intuition record: a cached (situation → action) pair
#[derive(Debug, Clone)]
pub struct IntuitionRecord {
    pub situation_vsa: Vec<u8>,
    pub action_label: String,
    pub heuristic: IntuitionHeuristic,
    pub confidence: f64,
    pub timestamp: Instant,
    pub access_count: u64,
    pub success_count: u64,
}

impl IntuitionRecord {
    pub fn new(
        situation_vsa: Vec<u8>,
        action_label: String,
        heuristic: IntuitionHeuristic,
    ) -> Self {
        Self {
            situation_vsa,
            action_label,
            heuristic,
            confidence: 0.5,
            timestamp: Instant::now(),
            access_count: 0,
            success_count: 0,
        }
    }

    pub fn update_confidence(&mut self, outcome_success: bool) {
        self.access_count += 1;
        if outcome_success {
            self.success_count += 1;
        }
        self.confidence = self.success_count as f64 / self.access_count as f64;
        self.timestamp = Instant::now();
    }
}

/// Configuration for the System 1 intuition engine
#[derive(Debug, Clone)]
pub struct System1Config {
    pub buffer_capacity: usize,
    pub match_threshold: f64,
    pub top_k: usize,
    pub latency_budget_ms: u64,
    pub heuristic_weights: HashMap<IntuitionHeuristic, f64>,
    pub frequency_decay_hours: f64,
    pub recency_boost_seconds: u64,
}

impl Default for System1Config {
    fn default() -> Self {
        let mut heuristic_weights = HashMap::new();
        heuristic_weights.insert(IntuitionHeuristic::Similarity, 1.0);
        heuristic_weights.insert(IntuitionHeuristic::Frequency, 0.6);
        heuristic_weights.insert(IntuitionHeuristic::Proximity, 0.4);
        heuristic_weights.insert(IntuitionHeuristic::Recency, 0.5);
        heuristic_weights.insert(IntuitionHeuristic::Emotion, 0.3);

        Self {
            buffer_capacity: 100,
            match_threshold: 0.65,
            top_k: 3,
            latency_budget_ms: 100,
            heuristic_weights,
            frequency_decay_hours: 24.0,
            recency_boost_seconds: 60,
        }
    }
}

/// FastPatternMatcher — sparse VSA cosine similarity matching, <1ms target
#[derive(Debug, Clone)]
pub struct FastPatternMatcher {
    config: System1Config,
}

impl FastPatternMatcher {
    pub fn new(config: System1Config) -> Self {
        Self { config }
    }

    /// Match a query VSA vector against the buffer, returning top-k scored results
    pub fn match_against(&self, query: &[u8], buffer: &[IntuitionRecord]) -> Vec<(usize, f64)> {
        if buffer.is_empty() {
            return Vec::new();
        }

        let now = Instant::now();
        let mut scores: Vec<(usize, f64)> = Vec::with_capacity(buffer.len());

        for (i, record) in buffer.iter().enumerate() {
            let base_sim = QuantizedVSA::similarity(query, &record.situation_vsa);
            if base_sim < self.config.match_threshold {
                continue;
            }

            let weight = self
                .config
                .heuristic_weights
                .get(&record.heuristic)
                .copied()
                .unwrap_or(0.5);

            let freq_boost = if record.access_count > 0 {
                let elapsed = now.duration_since(record.timestamp);
                let decay_hours = self.config.frequency_decay_hours;
                let decay = (-elapsed.as_secs_f64() / (decay_hours * 3600.0)).exp();
                (record.access_count as f64).ln_1p() * decay * 0.1
            } else {
                0.0
            };

            let recency_boost = {
                let elapsed = now.duration_since(record.timestamp);
                if elapsed < Duration::from_secs(self.config.recency_boost_seconds) {
                    0.15
                } else {
                    0.0
                }
            };

            let confidence_factor = record.confidence * 0.2;
            let final_score = base_sim * weight + freq_boost + recency_boost + confidence_factor;

            scores.push((i, final_score));
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(self.config.top_k);
        scores
    }

    /// Batch match: compute similarities for multiple queries at once
    pub fn batch_match(
        &self,
        queries: &[&[u8]],
        buffer: &[IntuitionRecord],
    ) -> Vec<Vec<(usize, f64)>> {
        queries
            .iter()
            .map(|q| self.match_against(q, buffer))
            .collect()
    }

    /// Compute a single similarity score between two VSA vectors
    pub fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b)
    }
}

/// IntuitionBuffer — circular buffer of recent intuition records
#[derive(Debug, Clone)]
pub struct IntuitionBuffer {
    records: Vec<IntuitionRecord>,
    capacity: usize,
}

impl IntuitionBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a new record; evicts oldest if at capacity
    pub fn push(&mut self, record: IntuitionRecord) {
        if self.records.len() >= self.capacity {
            self.records.remove(0);
        }
        self.records.push(record);
    }

    /// Get all records
    pub fn records(&self) -> &[IntuitionRecord] {
        &self.records
    }

    /// Get records matching a specific heuristic
    pub fn records_by_heuristic(&self, heuristic: IntuitionHeuristic) -> Vec<&IntuitionRecord> {
        self.records
            .iter()
            .filter(|r| r.heuristic == heuristic)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Remove records older than the given duration
    pub fn prune_older_than(&mut self, duration: Duration) {
        let cutoff = Instant::now() - duration;
        self.records.retain(|r| r.timestamp > cutoff);
    }

    /// Find index of a record by action label
    pub fn find_by_action(&self, action: &str) -> Option<usize> {
        self.records.iter().position(|r| r.action_label == action)
    }

    /// Update record at index with outcome feedback
    pub fn update_outcome(&mut self, idx: usize, success: bool) {
        if let Some(record) = self.records.get_mut(idx) {
            record.update_confidence(success);
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

/// Report from System 1 after an intuition match
#[derive(Debug, Clone)]
pub struct IntuitionReport {
    pub matched: bool,
    pub top_matches: Vec<(String, f64, IntuitionHeuristic)>,
    pub best_action: Option<String>,
    pub best_confidence: f64,
    pub match_count: usize,
    pub elapsed_ms: f64,
}

/// ConflictDetector — detects contradictions between System 1 intuition and System 2 reasoning
#[derive(Debug, Clone)]
pub struct ConflictDetector {
    pub contradiction_threshold: f64,
    pub total_conflicts: u64,
    pub s1_wins: u64,
    pub s2_wins: u64,
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            contradiction_threshold: 0.7,
            total_conflicts: 0,
            s1_wins: 0,
            s2_wins: 0,
        }
    }

    /// A conflict is detected when S1 and S2 recommend different actions
    /// and both have confidence above the threshold
    pub fn detect_conflict(
        &self,
        s1_action: &str,
        s1_confidence: f64,
        s2_action: &str,
        s2_confidence: f64,
    ) -> bool {
        s1_action != s2_action
            && s1_confidence >= self.contradiction_threshold
            && s2_confidence >= self.contradiction_threshold
    }

    /// Resolve a conflict: choose between S1 and S2 based on context
    pub fn resolve(
        &mut self,
        s1_action: &str,
        s1_confidence: f64,
        s2_action: &str,
        s2_confidence: f64,
        s1_latency_ms: f64,
        uncertainty: f64,
    ) -> ConflictReport {
        let has_conflict = self.detect_conflict(s1_action, s1_confidence, s2_action, s2_confidence);

        if !has_conflict {
            return ConflictReport {
                has_conflict: false,
                chosen_action: s2_action.to_string(),
                chosen_system: "System2",
                confidence: s2_confidence.max(s1_confidence),
                reasoning: "no contradiction".to_string(),
            };
        }

        // Weighted resolution:
        // - S1 wins if: high confidence + low latency + low uncertainty
        // - S2 wins if: high confidence + high uncertainty (needs analysis)
        // - Default to S2 (conservative)
        let s1_score = s1_confidence * (1.0 - s1_latency_ms / 5000.0);
        let s2_score = s2_confidence * (0.5 + uncertainty);

        self.total_conflicts += 1;

        if s1_score > s2_score {
            self.s1_wins += 1;
            ConflictReport {
                has_conflict: true,
                chosen_action: s1_action.to_string(),
                chosen_system: "System1",
                confidence: s1_confidence,
                reasoning: format!(
                    "S1 intuition ({:.1}%) beats S2 analysis ({:.1}%) under uncertainty {:.2}",
                    s1_confidence * 100.0,
                    s2_confidence * 100.0,
                    uncertainty
                ),
            }
        } else {
            self.s2_wins += 1;
            ConflictReport {
                has_conflict: true,
                chosen_action: s2_action.to_string(),
                chosen_system: "System2",
                confidence: s2_confidence,
                reasoning: format!(
                    "S2 analysis ({:.1}%) overrides S1 intuition ({:.1}%) under uncertainty {:.2}",
                    s2_confidence * 100.0,
                    s1_confidence * 100.0,
                    uncertainty
                ),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub has_conflict: bool,
    pub chosen_action: String,
    pub chosen_system: &'static str,
    pub confidence: f64,
    pub reasoning: String,
}

/// System1Intuition — the orchestrator for the System 1 fast intuition pathway
#[derive(Debug, Clone)]
pub struct System1Intuition {
    pub config: System1Config,
    pub buffer: IntuitionBuffer,
    pub matcher: FastPatternMatcher,
    pub conflict_detector: ConflictDetector,
    pub total_matches: u64,
    pub total_missed: u64,
    pub average_latency_ms: f64,
}

impl Default for System1Intuition {
    fn default() -> Self {
        Self::new()
    }
}

impl System1Intuition {
    pub fn new() -> Self {
        let config = System1Config::default();
        let cap = config.buffer_capacity;
        Self {
            config: config.clone(),
            buffer: IntuitionBuffer::new(cap),
            matcher: FastPatternMatcher::new(config),
            conflict_detector: ConflictDetector::new(),
            total_matches: 0,
            total_missed: 0,
            average_latency_ms: 0.0,
        }
    }

    pub fn with_config(config: System1Config) -> Self {
        Self {
            buffer: IntuitionBuffer::new(config.buffer_capacity),
            matcher: FastPatternMatcher::new(config.clone()),
            conflict_detector: ConflictDetector::new(),
            total_matches: 0,
            total_missed: 0,
            average_latency_ms: 0.0,
            config,
        }
    }

    /// Core intuition call: match a query VSA vector against the buffer
    pub fn intuit(&mut self, query_vsa: &[u8]) -> IntuitionReport {
        let start = Instant::now();

        let matches = self.matcher.match_against(query_vsa, self.buffer.records());

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        self.average_latency_ms = self.average_latency_ms * 0.9 + elapsed * 0.1;

        if matches.is_empty() {
            self.total_missed += 1;
            return IntuitionReport {
                matched: false,
                top_matches: Vec::new(),
                best_action: None,
                best_confidence: 0.0,
                match_count: 0,
                elapsed_ms: elapsed,
            };
        }

        self.total_matches += 1;

        let top_matches: Vec<(String, f64, IntuitionHeuristic)> = matches
            .iter()
            .map(|(idx, score)| {
                let record = &self.buffer.records()[*idx];
                (record.action_label.clone(), *score, record.heuristic)
            })
            .collect();

        let (best_idx, _) = matches[0];
        let best = &self.buffer.records()[best_idx];

        IntuitionReport {
            matched: true,
            top_matches,
            best_action: Some(best.action_label.clone()),
            best_confidence: best.confidence,
            match_count: matches.len(),
            elapsed_ms: elapsed,
        }
    }

    /// Learn from experience: store a new (situation → action) pair
    pub fn learn(
        &mut self,
        situation_vsa: Vec<u8>,
        action_label: String,
        heuristic: IntuitionHeuristic,
    ) {
        let record = IntuitionRecord::new(situation_vsa, action_label, heuristic);
        self.buffer.push(record);
    }

    /// Provide outcome feedback to update confidence
    pub fn feedback(&mut self, action: &str, success: bool) {
        if let Some(idx) = self.buffer.find_by_action(action) {
            self.buffer.update_outcome(idx, success);
        }
    }

    /// Resolve a conflict between S1 intuition and S2 reasoning
    pub fn resolve_conflict(
        &mut self,
        s1_action: &str,
        s1_confidence: f64,
        s2_action: &str,
        s2_confidence: f64,
        s1_latency_ms: f64,
        uncertainty: f64,
    ) -> ConflictReport {
        self.conflict_detector.resolve(
            s1_action,
            s1_confidence,
            s2_action,
            s2_confidence,
            s1_latency_ms,
            uncertainty,
        )
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total_matches + self.total_missed;
        if total == 0 {
            return 0.0;
        }
        self.total_matches as f64 / total as f64
    }

    pub fn prune_old(&mut self, duration: Duration) {
        self.buffer.prune_older_than(duration);
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.total_matches = 0;
        self.total_missed = 0;
        self.average_latency_ms = 0.0;
    }
}

/// Trait for reading external situations into VSA vectors for System 1
/// Defined locally to avoid reverse architecture dependency (core/ → neotrix/).
pub trait SituationsReader {
    fn situations(&self) -> Vec<SituationInput>;
}

pub struct SituationInput {
    pub label: String,
    pub vsa_bytes: Vec<u8>,
    pub heuristic: IntuitionHeuristic,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    #[test]
    fn test_intuition_heuristic_names() {
        assert_eq!(IntuitionHeuristic::Similarity.name(), "similarity");
        assert_eq!(IntuitionHeuristic::Emotion.name(), "emotion");
    }

    #[test]
    fn test_intuition_record_confidence_update() {
        let vsa = test_vsa(1);
        let mut record =
            IntuitionRecord::new(vsa, "test_action".into(), IntuitionHeuristic::Similarity);
        assert!((record.confidence - 0.5).abs() < 1e-9);
        record.update_confidence(true);
        assert!((record.confidence - 1.0).abs() < 1e-9);
        record.update_confidence(false);
        assert!((record.confidence - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_intuition_buffer_push_and_evict() {
        let mut buf = IntuitionBuffer::new(3);
        for i in 0..5u64 {
            buf.push(IntuitionRecord::new(
                test_vsa(i),
                format!("action_{}", i),
                IntuitionHeuristic::Recency,
            ));
        }
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.records()[0].action_label, "action_2");
        assert_eq!(buf.records()[2].action_label, "action_4");
    }

    #[test]
    fn test_fast_pattern_matcher_no_match() {
        let config = System1Config::default();
        let matcher = FastPatternMatcher::new(config);
        let buf = IntuitionBuffer::new(10);
        let results = matcher.match_against(&test_vsa(99), buf.records());
        assert!(results.is_empty());
    }

    #[test]
    fn test_fast_pattern_matcher_exact_match() {
        let config = System1Config {
            match_threshold: 0.5,
            ..Default::default()
        };
        let matcher = FastPatternMatcher::new(config);
        let mut buf = IntuitionBuffer::new(10);
        let vsa = test_vsa(42);
        buf.push(IntuitionRecord::new(
            vsa.clone(),
            "exact_match".into(),
            IntuitionHeuristic::Similarity,
        ));
        let results = matcher.match_against(&vsa, buf.records());
        assert!(!results.is_empty(), "should find exact match");
        assert_eq!(results[0].0, 0, "first record should match");
    }

    #[test]
    fn test_intuition_buffer_find_by_action() {
        let mut buf = IntuitionBuffer::new(10);
        buf.push(IntuitionRecord::new(
            test_vsa(1),
            "hello".into(),
            IntuitionHeuristic::Similarity,
        ));
        assert_eq!(buf.find_by_action("hello"), Some(0));
        assert_eq!(buf.find_by_action("missing"), None);
    }

    #[test]
    fn test_intuition_buffer_update_outcome() {
        let mut buf = IntuitionBuffer::new(10);
        buf.push(IntuitionRecord::new(
            test_vsa(1),
            "test".into(),
            IntuitionHeuristic::Similarity,
        ));
        buf.update_outcome(0, true);
        assert!((buf.records()[0].confidence - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_conflict_detector_no_conflict() {
        let detector = ConflictDetector::new();
        assert!(!detector.detect_conflict("a", 0.8, "a", 0.9));
    }

    #[test]
    fn test_conflict_detector_has_conflict() {
        let detector = ConflictDetector::new();
        assert!(detector.detect_conflict("a", 0.8, "b", 0.9));
    }

    #[test]
    fn test_conflict_detector_low_confidence_no_conflict() {
        let detector = ConflictDetector::new();
        assert!(!detector.detect_conflict("a", 0.5, "b", 0.9));
    }

    #[test]
    fn test_conflict_resolve_s1_wins() {
        let mut detector = ConflictDetector::new();
        let report = detector.resolve("intuit", 0.9, "analyze", 0.7, 10.0, 0.2);
        assert!(report.has_conflict);
        assert_eq!(report.chosen_system, "System1");
        assert_eq!(report.chosen_action, "intuit");
    }

    #[test]
    fn test_conflict_resolve_s2_wins() {
        let mut detector = ConflictDetector::new();
        let report = detector.resolve("intuit", 0.7, "analyze", 0.9, 100.0, 0.8);
        assert!(report.has_conflict);
        assert_eq!(report.chosen_system, "System2");
        assert_eq!(report.chosen_action, "analyze");
    }

    #[test]
    fn test_system1_intuit_no_records() {
        let mut sys1 = System1Intuition::new();
        let report = sys1.intuit(&test_vsa(1));
        assert!(!report.matched);
        assert_eq!(report.match_count, 0);
    }

    #[test]
    fn test_system1_intuit_with_match() {
        let mut sys1 = System1Intuition::new();
        let vsa = test_vsa(42);
        sys1.learn(
            vsa.clone(),
            "known_pattern".into(),
            IntuitionHeuristic::Similarity,
        );
        let report = sys1.intuit(&vsa);
        assert!(report.matched);
        assert_eq!(report.best_action.as_deref(), Some("known_pattern"));
    }

    #[test]
    fn test_system1_hit_rate() {
        let mut sys1 = System1Intuition::new();
        assert!((sys1.hit_rate() - 0.0).abs() < 1e-9);
        let vsa = test_vsa(1);
        sys1.learn(vsa.clone(), "a".into(), IntuitionHeuristic::Similarity);
        sys1.intuit(&vsa);
        sys1.intuit(&test_vsa(99));
        assert!((sys1.hit_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_system1_feedback_updates_confidence() {
        let mut sys1 = System1Intuition::new();
        let vsa = test_vsa(1);
        sys1.learn(vsa, "fb_test".into(), IntuitionHeuristic::Frequency);
        sys1.feedback("fb_test", true);
        sys1.feedback("fb_test", true);
        sys1.feedback("fb_test", false);
        if let Some(idx) = sys1.buffer.find_by_action("fb_test") {
            assert!((sys1.buffer.records()[idx].confidence - 2.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_system1_prune_old() {
        let mut sys1 = System1Intuition::new();
        sys1.learn(test_vsa(1), "old".into(), IntuitionHeuristic::Recency);
        // Prune everything older than 0 duration = all records
        sys1.prune_old(Duration::from_secs(0));
        assert!(sys1.buffer.is_empty());
    }

    #[test]
    fn test_heuristic_weights_in_scoring() {
        let mut config = System1Config::default();
        config.match_threshold = 0.0;
        config
            .heuristic_weights
            .insert(IntuitionHeuristic::Similarity, 10.0);
        let matcher = FastPatternMatcher::new(config);
        let mut buf = IntuitionBuffer::new(10);
        let vsa = test_vsa(7);
        buf.push(IntuitionRecord::new(
            vsa.clone(),
            "high_weight".into(),
            IntuitionHeuristic::Similarity,
        ));
        buf.push(IntuitionRecord::new(
            vsa.clone(),
            "low_weight".into(),
            IntuitionHeuristic::Emotion,
        ));
        let results = matcher.match_against(&vsa, buf.records());
        assert_eq!(
            results[0].0, 0,
            "similarity heuristic should rank first due to weight"
        );
    }

    #[test]
    fn test_batch_match() {
        let config = System1Config {
            match_threshold: 0.5,
            ..Default::default()
        };
        let matcher = FastPatternMatcher::new(config);
        let mut buf = IntuitionBuffer::new(10);
        buf.push(IntuitionRecord::new(
            test_vsa(1),
            "a".into(),
            IntuitionHeuristic::Similarity,
        ));
        buf.push(IntuitionRecord::new(
            test_vsa(2),
            "b".into(),
            IntuitionHeuristic::Similarity,
        ));
        let v1 = test_vsa(1);
        let v3 = test_vsa(3);
        let queries = vec![v1.as_slice(), v3.as_slice()];
        let results = matcher.batch_match(&queries, buf.records());
        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty(), "first query should match 'a'");
    }

    #[test]
    fn test_system1_reset() {
        let mut sys1 = System1Intuition::new();
        sys1.learn(test_vsa(1), "x".into(), IntuitionHeuristic::Similarity);
        sys1.intuit(&test_vsa(1));
        assert!(sys1.total_matches > 0);
        sys1.reset();
        assert_eq!(sys1.total_matches, 0);
        assert!(sys1.buffer.is_empty());
    }

    #[test]
    fn test_system1_record_by_heuristic_filter() {
        let mut buf = IntuitionBuffer::new(10);
        buf.push(IntuitionRecord::new(
            test_vsa(1),
            "sim".into(),
            IntuitionHeuristic::Similarity,
        ));
        buf.push(IntuitionRecord::new(
            test_vsa(2),
            "emo".into(),
            IntuitionHeuristic::Emotion,
        ));
        let sims = buf.records_by_heuristic(IntuitionHeuristic::Similarity);
        assert_eq!(sims.len(), 1);
        assert_eq!(sims[0].action_label, "sim");
    }

    #[test]
    fn test_conflict_detector_s1_wins_low_latency() {
        let mut detector = ConflictDetector::new();
        // Very low latency + low uncertainty → S1 should win
        let report = detector.resolve("intuit", 0.85, "analyze", 0.8, 1.0, 0.1);
        assert_eq!(report.chosen_system, "System1");
    }

    #[test]
    fn test_conflict_detector_tracks_stats() {
        let mut detector = ConflictDetector::new();
        let _ = detector.resolve("a", 0.9, "b", 0.8, 10.0, 0.3);
        let _ = detector.resolve("c", 0.8, "d", 0.9, 100.0, 0.7);
        assert_eq!(detector.total_conflicts, 2);
        assert_eq!(detector.s1_wins + detector.s2_wins, 2);
    }
}
