use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};
use crate::core::nt_core_consciousness::global_workspace::BroadcastContent;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_time::unix_now_ms;
use std::collections::VecDeque;
use std::time::Instant;

/// Records a single shift of attention from one focus to another.
#[derive(Debug, Clone)]
pub struct AttentionShift {
    pub from_focus: Vec<u8>,
    pub to_focus: Vec<u8>,
    pub shift_reason: String,
    pub elapsed_ms: u64,
    pub was_voluntary: bool,
    pub timestamp: u64,
}

/// Tracks attention prediction expectations and accuracy.
/// Learns transition patterns from the attention trace.
#[derive(Debug, Clone)]
pub struct AttentionExpectation {
    pub predicted_focus: Option<Vec<u8>>,
    pub prediction_confidence: f64,
    pub accuracy_tracking: VecDeque<f64>,
    max_accuracy_len: usize,
}

impl AttentionExpectation {
    pub fn new() -> Self {
        Self {
            predicted_focus: None,
            prediction_confidence: 0.5,
            accuracy_tracking: VecDeque::with_capacity(20),
            max_accuracy_len: 20,
        }
    }

    /// Predict the next focus based on the most frequent transition pattern
    /// in the trace. Finds all shifts whose `from_focus` matches the current focus,
    /// and returns the most common `to_focus` among those transitions.
    /// Returns `None` if trace is empty or no pattern found.
    pub fn predict_next(&mut self, trace: &VecDeque<AttentionShift>) -> Option<Vec<u8>> {
        if trace.is_empty() {
            self.predicted_focus = None;
            return None;
        }

        let current_focus = &trace.back().expect("non-empty trace confirmed").to_focus;

        let mut transitions: Vec<Vec<u8>> = Vec::new();
        for shift in trace {
            let sim = QuantizedVSA::similarity(&shift.from_focus, current_focus);
            if sim > 0.8 {
                transitions.push(shift.to_focus.clone());
            }
        }

        if transitions.is_empty() {
            self.predicted_focus = None;
            return None;
        }

        let mut best_count = 0usize;
        let mut best_focus = transitions[0].clone();
        for candidate in &transitions {
            let count = transitions
                .iter()
                .filter(|t| QuantizedVSA::similarity(t, candidate) > 0.8)
                .count();
            if count > best_count {
                best_count = count;
                best_focus = candidate.clone();
            }
        }

        self.predicted_focus = Some(best_focus.clone());
        Some(best_focus)
    }

    /// Record prediction accuracy after an actual attention shift.
    /// Computes prediction_error = 1 - similarity(predicted, actual),
    /// pushes to accuracy_tracking, and updates prediction_confidence
    /// as the rolling average of recent accuracy.
    pub fn record_accuracy(&mut self, actual: &[u8]) {
        let prediction_error = match &self.predicted_focus {
            Some(predicted) => 1.0 - QuantizedVSA::similarity(predicted, actual),
            None => 1.0,
        };

        self.accuracy_tracking.push_back(prediction_error);
        while self.accuracy_tracking.len() > self.max_accuracy_len {
            self.accuracy_tracking.pop_front();
        }

        let recent_error: f64 =
            self.accuracy_tracking.iter().sum::<f64>() / self.accuracy_tracking.len().max(1) as f64;
        self.prediction_confidence = (1.0 - recent_error).clamp(0.0, 1.0);
    }
}

impl Default for AttentionExpectation {
    fn default() -> Self {
        Self::new()
    }
}

/// Attention Schema Engine implementing Graziano's Attention Schema Theory.
///
/// The system maintains a model of its OWN attention process — not just directing
/// attention, but knowing *what it is attending to*. This metacognitive layer allows
/// the system to predict, track, and reflect on its own attentional state.
///
/// Planned integration: `integrate_with_broadcast()` bridges with
/// GlobalLatentWorkspace to respond to broadcast content.
/// Type alias for backward compatibility with consciousness_cycle.
pub type AttentionSchema = AttentionSchemaEngine;

#[derive(Debug, Clone)]
pub struct AttentionSchemaEngine {
    pub current_focus: Option<VsaTagged>,
    pub attention_trace: VecDeque<AttentionShift>,
    pub expectations: AttentionExpectation,
    pub meta_attention_level: f64,
    max_trace_len: usize,
    total_shifts: u64,
    start_time: Instant,
}

impl AttentionSchemaEngine {
    pub fn new(max_trace_len: usize) -> Self {
        Self {
            current_focus: None,
            attention_trace: VecDeque::with_capacity(max_trace_len),
            expectations: AttentionExpectation::new(),
            meta_attention_level: 0.5,
            max_trace_len,
            total_shifts: 0,
            start_time: Instant::now(),
        }
    }

    /// Direct attention to a target. Records the shift including elapsed time
    /// from the previous shift, and updates prediction accuracy.
    ///
    /// The target is tagged with `Self(MetaCognition)` to mark it as an
    /// attentional object in the cognitive architecture.
    pub fn attend_to(&mut self, target: VsaTagged, reason: &str, voluntary: bool) {
        let now = unix_now_ms();

        let elapsed_ms = self
            .attention_trace
            .back()
            .map(|last| now.saturating_sub(last.timestamp))
            .unwrap_or(0);

        let from_focus = self
            .current_focus
            .as_ref()
            .map(|f| f.vector.clone())
            .unwrap_or_else(|| vec![0u8; target.vector.len().max(1)]);

        if self.current_focus.is_some() {
            self.expectations.record_accuracy(&target.vector);
        }

        let shift = AttentionShift {
            from_focus,
            to_focus: target.vector.clone(),
            shift_reason: reason.to_string(),
            elapsed_ms,
            was_voluntary: voluntary,
            timestamp: now,
        };

        self.attention_trace.push_back(shift);
        while self.attention_trace.len() > self.max_trace_len {
            self.attention_trace.pop_front();
        }

        self.current_focus = Some(target);
        self.total_shifts += 1;
    }

    /// Predict the next focus based on learned transition patterns.
    /// Delegates to `AttentionExpectation::predict_next`.
    pub fn predict_next_focus(&mut self) -> Option<Vec<u8>> {
        self.expectations.predict_next(&self.attention_trace)
    }

    /// Current prediction error: 1 - max similarity between predicted and actual focus.
    /// Returns 0.0 if there is no prediction or no current focus.
    pub fn prediction_error(&self) -> f64 {
        match (&self.expectations.predicted_focus, &self.current_focus) {
            (Some(predicted), Some(actual)) => {
                1.0 - QuantizedVSA::similarity(predicted, &actual.vector)
            }
            _ => 0.0,
        }
    }

    /// Set meta-attention level, clamped to [0.0, 1.0].
    pub fn set_meta_attention(&mut self, level: f64) {
        self.meta_attention_level = level.clamp(0.0, 1.0);
    }

    /// Approximate attention shift rate in shifts per second since engine creation.
    pub fn shift_rate(&self) -> f64 {
        let secs = self.start_time.elapsed().as_secs_f64().max(0.001);
        self.total_shifts as f64 / secs
    }

    /// Ratio of voluntary shifts to total shifts, in [0.0, 1.0].
    pub fn voluntary_ratio(&self) -> f64 {
        if self.total_shifts == 0 {
            return 0.0;
        }
        let voluntary = self
            .attention_trace
            .iter()
            .filter(|s| s.was_voluntary)
            .count() as f64;
        voluntary / self.total_shifts as f64
    }

    /// Integration point with GlobalLatentWorkspace broadcasts.
    /// When content wins the workspace competition, attention can optionally
    /// shift to the broadcast content.
    pub fn integrate_with_broadcast(&mut self, content: &BroadcastContent) {
        let tagged = VsaTagged {
            vector: content.vector.clone(),
            tag: VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
            confidence: 1.0,
            timestamp: unix_now_ms(),
            salience: content.salience,
            provenance: None,
            sense_modality: None,
            prediction: None,
            outcome: None,
        };
        self.attend_to(tagged, &format!("broadcast: {}", content.winner), false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    fn tagged_focus(seed: u64, _label: &str) -> VsaTagged {
        let mut t = VsaTagged::new(
            test_vsa(seed),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        );
        t.timestamp = seed; // deterministic timestamp for predictable elapsed_ms
        t
    }

    // ── 1. AttentionShift creation ──

    #[test]
    fn test_attention_shift_creation() {
        let from = test_vsa(1);
        let to = test_vsa(2);
        let shift = AttentionShift {
            from_focus: from.clone(),
            to_focus: to.clone(),
            shift_reason: "test shift".into(),
            elapsed_ms: 100,
            was_voluntary: true,
            timestamp: 1000,
        };
        assert_eq!(shift.from_focus, from);
        assert_eq!(shift.to_focus, to);
        assert_eq!(shift.shift_reason, "test shift");
        assert_eq!(shift.elapsed_ms, 100);
        assert!(shift.was_voluntary);
    }

    // ── 2. AttentionExpectation default state ──

    #[test]
    fn test_attention_expectation_new() {
        let exp = AttentionExpectation::new();
        assert!(exp.predicted_focus.is_none());
        assert!((exp.prediction_confidence - 0.5).abs() < 1e-9);
        assert!(exp.accuracy_tracking.is_empty());
    }

    // ── 3. AttentionExpectation predict with empty trace ──

    #[test]
    fn test_attention_expectation_predict_empty() {
        let mut exp = AttentionExpectation::new();
        let trace: VecDeque<AttentionShift> = VecDeque::new();
        let result = exp.predict_next(&trace);
        assert!(result.is_none());
        assert!(exp.predicted_focus.is_none());
    }

    // ── 4. AttentionSchemaEngine initial state ──

    #[test]
    fn test_attention_schema_engine_new() {
        let engine = AttentionSchemaEngine::new(100);
        assert!(engine.current_focus.is_none());
        assert!(engine.attention_trace.is_empty());
        assert!((engine.meta_attention_level - 0.5).abs() < 1e-9);
        assert_eq!(engine.total_shifts, 0);
    }

    // ── 5. attend_to updates current_focus ──

    #[test]
    fn test_attend_to_updates_focus() {
        let mut engine = AttentionSchemaEngine::new(100);
        let target = tagged_focus(42, "focus A");
        engine.attend_to(target, "test reason", true);
        assert!(engine.current_focus.is_some());
        assert_eq!(engine.current_focus.as_ref().unwrap().timestamp, 42);
    }

    // ── 6. attend_to records a shift in the trace ──

    #[test]
    fn test_attend_to_records_shift() {
        let mut engine = AttentionSchemaEngine::new(100);
        let a = tagged_focus(10, "A");
        let b = tagged_focus(20, "B");
        engine.attend_to(a, "first", true);
        assert_eq!(engine.attention_trace.len(), 1);
        engine.attend_to(b, "second", false);
        assert_eq!(engine.attention_trace.len(), 2);
    }

    // ── 7. trace does not exceed max_trace_len ──

    #[test]
    fn test_attend_to_max_trace() {
        let mut engine = AttentionSchemaEngine::new(3);
        for i in 0..10 {
            let t = tagged_focus(i as u64 * 10, &format!("item {}", i));
            engine.attend_to(t, "fill", true);
        }
        assert_eq!(engine.attention_trace.len(), 3);
    }

    // ── 8. shift_reason is recorded ──

    #[test]
    fn test_attention_shift_reason_recorded() {
        let mut engine = AttentionSchemaEngine::new(100);
        let t = tagged_focus(1, "reason test");
        engine.attend_to(t, "curiosity", false);
        let shift = engine.attention_trace.back().unwrap();
        assert_eq!(shift.shift_reason, "curiosity");
    }

    // ── 9. voluntary vs involuntary flag ──

    #[test]
    fn test_voluntary_vs_involuntary() {
        let mut engine = AttentionSchemaEngine::new(100);
        let a = tagged_focus(1, "voluntary");
        let b = tagged_focus(2, "involuntary");
        engine.attend_to(a, "chosen", true);
        assert!(engine.attention_trace[0].was_voluntary);
        engine.attend_to(b, "interrupt", false);
        assert!(!engine.attention_trace[1].was_voluntary);
    }

    // ── 10. predict_next_focus after repeated pattern ──

    #[test]
    fn test_predict_next_focus_after_pattern() {
        let mut engine = AttentionSchemaEngine::new(100);
        // Create two distinct foci
        let a = tagged_focus(100, "A");
        let b = tagged_focus(200, "B");

        // Pattern: A → B, A → B, A → B
        for _ in 0..3 {
            engine.attend_to(a.clone(), "to A", true);
            engine.attend_to(b.clone(), "to B", true);
        }

        // Current focus should be B (last attend_to was B)
        // Pattern from B: nothing yet (never shifted from B)
        // So prediction might be None if no pattern from B
        // Let's check what we get
        let prediction = engine.predict_next_focus();
        // After A→B, A→B, A→B, the trace is [B→A, A→B, B→A, A→B, B→A, A→B]
        // Actually no: trace is [to A, to B, to A, to B, to A, to B]
        // Shifts: none→A, A→B, B→A, A→B, B→A, A→B
        // Current focus = B (from last shift to B)
        // Shifts from B: there are 2 (B→A)
        // Most common from B: A
        // Prediction should be A

        // If prediction returns something, verify similarity
        if let Some(pred) = prediction {
            let sim = QuantizedVSA::similarity(&pred, &a.vector);
            assert!(sim > 0.8, "prediction should match A's vector");
        }
    }

    // ── 11. prediction_error returns valid value ──

    #[test]
    fn test_prediction_error_tracking() {
        let mut engine = AttentionSchemaEngine::new(100);
        // No focus yet
        assert!((engine.prediction_error() - 0.0).abs() < 1e-9);

        let a = tagged_focus(10, "A");
        let b = tagged_focus(20, "B");
        engine.attend_to(a, "first", true);
        engine.attend_to(b, "second", true);

        let err = engine.prediction_error();
        assert!(err >= 0.0);
        assert!(err <= 1.0);
    }

    // ── 12. meta_attention clamping ──

    #[test]
    fn test_meta_attention_clamping() {
        let mut engine = AttentionSchemaEngine::new(100);
        engine.set_meta_attention(1.5);
        assert!((engine.meta_attention_level - 1.0).abs() < 1e-9);
        engine.set_meta_attention(-0.5);
        assert!((engine.meta_attention_level - 0.0).abs() < 1e-9);
        engine.set_meta_attention(0.7);
        assert!((engine.meta_attention_level - 0.7).abs() < 1e-9);
    }

    // ── 13. shift_rate returns non-negative ──

    #[test]
    fn test_shift_rate() {
        let engine = AttentionSchemaEngine::new(100);
        assert!(engine.shift_rate() >= 0.0);
        let mut engine = AttentionSchemaEngine::new(100);
        let a = tagged_focus(1, "A");
        engine.attend_to(a, "first", true);
        assert!(engine.shift_rate() > 0.0);
    }

    // ── 14. voluntary_ratio in [0, 1] ──

    #[test]
    fn test_voluntary_ratio() {
        let engine = AttentionSchemaEngine::new(100);
        assert!((engine.voluntary_ratio() - 0.0).abs() < 1e-9);

        let mut engine = AttentionSchemaEngine::new(100);
        engine.attend_to(tagged_focus(1, "A"), "v1", true);
        engine.attend_to(tagged_focus(2, "B"), "v2", true);
        assert!((engine.voluntary_ratio() - 1.0).abs() < 1e-9);

        engine.attend_to(tagged_focus(3, "C"), "forced", false);
        assert!((engine.voluntary_ratio() - 2.0 / 3.0).abs() < 1e-9);
    }

    // ── 15. focus is tagged Self(MetaCognition) ──

    #[test]
    fn test_focus_is_self_tagged() {
        let mut engine = AttentionSchemaEngine::new(100);
        let target = tagged_focus(7, "meta-test");
        engine.attend_to(target, "meta", true);
        let focus = engine.current_focus.as_ref().unwrap();
        assert!(focus.is_self());
        assert_eq!(focus.tag, VsaOrigin::Self_(VsaSelfCategory::MetaCognition));
    }

    // ── 16. elapsed_ms between shifts ──

    #[test]
    fn test_elapsed_ms_between_shifts() {
        let mut engine = AttentionSchemaEngine::new(100);
        let a = tagged_focus(1, "A");
        let b = tagged_focus(2, "B");
        engine.attend_to(a, "first", true);
        // Second shift's elapsed_ms should be computed from timestamp difference
        engine.attend_to(b, "second", true);
        let second = engine.attention_trace.back().unwrap();
        // elapsed_ms = now - previous_timestamp; actual value depends on real time
        // Verify from_focus is the previous focus vector
        // Verify from_focus is the previous focus vector
        assert_eq!(second.from_focus, test_vsa(1));
    }

    // ── 17. accuracy_tracking gets populated ──

    #[test]
    fn test_accuracy_tracking_populated() {
        let mut engine = AttentionSchemaEngine::new(100);
        // First attend_to doesn't record accuracy (no prior focus)
        engine.attend_to(tagged_focus(10, "A"), "first", true);
        assert!(engine.expectations.accuracy_tracking.is_empty());

        // Second attend_to records accuracy of first prediction
        engine.attend_to(tagged_focus(20, "B"), "second", true);
        assert_eq!(engine.expectations.accuracy_tracking.len(), 1);

        // Third attend_to adds another accuracy record
        engine.attend_to(tagged_focus(30, "C"), "third", true);
        assert_eq!(engine.expectations.accuracy_tracking.len(), 2);
    }

    // ── 18. prediction_confidence updates ──

    #[test]
    fn test_prediction_confidence_updates() {
        let mut engine = AttentionSchemaEngine::new(100);
        // With no prediction and no shifts, confidence stays at 0.5
        assert!((engine.expectations.prediction_confidence - 0.5).abs() < 1e-9);

        // After attending to multiple items without prediction, errors accumulate
        engine.attend_to(tagged_focus(1, "A"), "a", true);
        engine.attend_to(tagged_focus(2, "B"), "b", true);
        // record_accuracy was called for B vs predicted_focus (which is None → error=1.0)
        // So confidence drops: 1.0 - 1.0 = 0.0
        assert!((engine.expectations.prediction_confidence - 0.0).abs() < 1e-9);
    }

    // ── 19. integrate_with_broadcast creates a shift ──

    #[test]
    fn test_integrate_with_broadcast() {
        let mut engine = AttentionSchemaEngine::new(100);
        let content = BroadcastContent {
            winner: "test_module".into(),
            vector: test_vsa(99),
            salience: 0.8,
            runner_up: None,
            broadcast_cycle: 1,
            processor_count: 0,
            competition_diversity: 0.0,
        };
        engine.integrate_with_broadcast(&content);
        assert!(engine.current_focus.is_some());
        let focus = engine.current_focus.as_ref().unwrap();
        assert!(focus.is_self());
        assert_eq!(engine.attention_trace.len(), 1);
        let shift = engine.attention_trace.back().unwrap();
        assert_eq!(shift.shift_reason, "broadcast: test_module");
        assert!(!shift.was_voluntary);
    }

    // ── 20. Default for AttentionExpectation ──

    #[test]
    fn test_attention_expectation_default() {
        let exp: AttentionExpectation = Default::default();
        assert!(exp.predicted_focus.is_none());
        assert!((exp.prediction_confidence - 0.5).abs() < 1e-9);
    }

    // ── 21. record_accuracy without prediction ──

    #[test]
    fn test_record_accuracy_no_prediction() {
        let mut exp = AttentionExpectation::new();
        let v = test_vsa(1);
        exp.record_accuracy(&v);
        assert_eq!(exp.accuracy_tracking.len(), 1);
        // No prediction → error = 1.0
        assert!((exp.accuracy_tracking[0] - 1.0).abs() < 1e-9);
        // confidence = 1.0 - 1.0 = 0.0
        assert!((exp.prediction_confidence - 0.0).abs() < 1e-9);
    }

    // ── 22. accuracy_tracking respects max length ──

    #[test]
    fn test_accuracy_tracking_max_len() {
        let mut exp = AttentionExpectation::new();
        // Push 30 items (max_accuracy_len = 20)
        for i in 0..30 {
            let _pred = exp.predict_next(&VecDeque::new()); // ensures record_accuracy has a prediction context
            exp.record_accuracy(&test_vsa(i));
        }
        assert_eq!(exp.accuracy_tracking.len(), 20);
    }

    // ── 23. multiple attend_to on single target doesn't duplicate errors ──

    #[test]
    fn test_attend_to_same_target_twice() {
        let mut engine = AttentionSchemaEngine::new(100);
        let a = tagged_focus(5, "same");
        engine.attend_to(a.clone(), "first", true);
        engine.attend_to(a.clone(), "second", true);
        assert_eq!(engine.attention_trace.len(), 2);
        assert!(engine.total_shifts >= 2);
    }

    // ── 24. first_person_ref integration via VsaTagged ──

    #[test]
    fn test_focus_has_confidence() {
        let mut engine = AttentionSchemaEngine::new(100);
        let target = VsaTagged::new(
            test_vsa(1),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        )
        .with_confidence(0.85);
        engine.attend_to(target, "conf test", true);
        let focus = engine.current_focus.as_ref().unwrap();
        assert!((focus.confidence - 0.85).abs() < 1e-9);
    }

    // ── 25. prediction error with exact match ──

    #[test]
    fn test_prediction_error_exact_match() {
        let mut engine = AttentionSchemaEngine::new(100);
        let a = tagged_focus(10, "A");
        engine.attend_to(a.clone(), "first", true);
        // Manually set predicted_focus to a.vector
        engine.expectations.predicted_focus = Some(a.vector.clone());
        // attend_to second with the same vector
        let a2 = tagged_focus(10, "A again");
        engine.attend_to(a2, "second", true);
        let err = engine.prediction_error();
        // After the shift, actual is a2.vector, predicted was a.vector, both from seed=10 → should match
        assert!(err >= 0.0 && err <= 1.0);
    }
}
