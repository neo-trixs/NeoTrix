use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct SalienceSignal {
    pub source_vsa: Vec<u8>,
    pub novelty: f64,
    pub surprise: f64,
    pub goal_relevance: f64,
    pub emotional_intensity: f64,
    pub composite: f64,
    pub timestamp: u64,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct SalienceWeights {
    pub novelty: f64,
    pub surprise: f64,
    pub goal_relevance: f64,
    pub emotional_intensity: f64,
}

impl Default for SalienceWeights {
    fn default() -> Self {
        Self {
            novelty: 0.25,
            surprise: 0.25,
            goal_relevance: 0.25,
            emotional_intensity: 0.25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SalienceDetector {
    pub novelty_trace: VecDeque<(Vec<u8>, u64)>,
    pub goal_prototypes: Vec<(Vec<u8>, String)>,
    pub signals: VecDeque<SalienceSignal>,
    pub weights: SalienceWeights,
    pub decay_rate: f64,
    pub max_trace: usize,
    pub max_signals: usize,
    pub cycle_count: u64,
    pub novelty_threshold: f64,
    pub signal_timeout_ms: u64,
}

impl Default for SalienceDetector {
    fn default() -> Self {
        Self {
            novelty_trace: VecDeque::new(),
            goal_prototypes: Vec::new(),
            signals: VecDeque::new(),
            weights: SalienceWeights::default(),
            decay_rate: 0.05,
            max_trace: 100,
            max_signals: 50,
            cycle_count: 0,
            novelty_threshold: 0.3,
            signal_timeout_ms: 5000,
        }
    }
}

impl SalienceDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(
        &mut self,
        source_vsa: Vec<u8>,
        prediction_error: f64,
        active_goals: &[&[u8]],
        emotion: f64,
        label: &str,
    ) -> SalienceSignal {
        let novelty = self.compute_novelty(&source_vsa);
        let surprise = prediction_error.clamp(0.0, 1.0);
        let goal_relevance = self.compute_goal_relevance(active_goals);
        let emotional_intensity = emotion.clamp(0.0, 1.0);

        let composite = self.weights.novelty * novelty
            + self.weights.surprise * surprise
            + self.weights.goal_relevance * goal_relevance
            + self.weights.emotional_intensity * emotional_intensity;

        let timestamp = self.cycle_count;

        self.update_trace(source_vsa.clone());

        let signal = SalienceSignal {
            source_vsa,
            novelty,
            surprise,
            goal_relevance,
            emotional_intensity,
            composite,
            timestamp,
            label: label.to_string(),
        };

        if self.signals.len() >= self.max_signals {
            self.signals.pop_front();
        }
        self.signals.push_back(signal.clone());
        self.cycle_count += 1;

        signal
    }

    fn compute_novelty(&self, source_vsa: &[u8]) -> f64 {
        if self.novelty_trace.is_empty() {
            return 1.0;
        }
        let max_sim = self
            .novelty_trace
            .iter()
            .map(|(vsa, _)| QuantizedVSA::similarity(vsa, source_vsa))
            .fold(0.0_f64, f64::max);
        (1.0 - max_sim).clamp(0.0, 1.0)
    }

    fn compute_goal_relevance(&self, active_goals: &[&[u8]]) -> f64 {
        let mut max_sim = 0.0;
        for &goal_vsa in active_goals {
            let sim = QuantizedVSA::similarity(goal_vsa, goal_vsa);
            if sim > max_sim {
                max_sim = sim;
            }
        }
        for (proto_vsa, _) in &self.goal_prototypes {
            for &goal_vsa in active_goals {
                let sim = QuantizedVSA::similarity(proto_vsa, goal_vsa);
                if sim > max_sim {
                    max_sim = sim;
                }
            }
        }
        max_sim.clamp(0.0, 1.0)
    }

    pub fn update_trace(&mut self, source_vsa: Vec<u8>) {
        let ts = self.cycle_count;
        if self.novelty_trace.len() >= self.max_trace {
            self.novelty_trace.pop_front();
        }
        self.novelty_trace.push_back((source_vsa, ts));
    }

    pub fn add_goal_prototype(&mut self, prototype: Vec<u8>, label: &str) {
        self.goal_prototypes.push((prototype, label.to_string()));
    }

    pub fn decay_signals(&mut self) {
        let threshold = if self.cycle_count > self.signal_timeout_ms {
            self.cycle_count - self.signal_timeout_ms
        } else {
            0
        };
        self.signals.retain(|s| s.timestamp >= threshold);
    }

    pub fn top_signals(&self, n: usize) -> Vec<&SalienceSignal> {
        let mut sorted: Vec<&SalienceSignal> = self.signals.iter().collect();
        sorted.sort_by(|a, b| {
            b.composite
                .partial_cmp(&a.composite)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(n);
        sorted
    }

    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    pub fn set_weight(&mut self, dimension: &str, weight: f64) {
        let w = weight.clamp(0.0, 1.0);
        match dimension {
            "novelty" => self.weights.novelty = w,
            "surprise" => self.weights.surprise = w,
            "goal_relevance" => self.weights.goal_relevance = w,
            "emotional_intensity" => self.weights.emotional_intensity = w,
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.novelty_trace.clear();
        self.goal_prototypes.clear();
        self.signals.clear();
        self.cycle_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::{QuantizedVSA, VSA_DIM};

    fn make_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    #[test]
    fn test_new_detector_defaults() {
        let sd = SalienceDetector::new();
        assert_eq!(sd.max_trace, 100);
        assert_eq!(sd.max_signals, 50);
        assert_eq!(sd.weights.novelty, 0.25);
        assert_eq!(sd.weights.surprise, 0.25);
        assert_eq!(sd.weights.goal_relevance, 0.25);
        assert_eq!(sd.weights.emotional_intensity, 0.25);
        assert!((sd.decay_rate - 0.05).abs() < 1e-9);
        assert!((sd.novelty_threshold - 0.3).abs() < 1e-9);
        assert_eq!(sd.signal_timeout_ms, 5000);
        assert_eq!(sd.cycle_count, 0);
        assert!(sd.novelty_trace.is_empty());
    }

    #[test]
    fn test_evaluate_basic() {
        let mut sd = SalienceDetector::new();
        let vsa = make_vsa(42);
        let signal = sd.evaluate(vsa, 0.5, &[], 0.3, "test");
        let expected = 0.25 * 1.0 + 0.25 * 0.5 + 0.25 * 0.0 + 0.25 * 0.3;
        assert!((signal.composite - expected).abs() < 1e-6);
        assert_eq!(signal.label, "test");
    }

    #[test]
    fn test_novelty_detection() {
        let mut sd = SalienceDetector::new();
        let vsa_a = make_vsa(1);
        let vsa_b = make_vsa(2);
        let s1 = sd.evaluate(vsa_a.clone(), 0.0, &[], 0.0, "first");
        let s2 = sd.evaluate(vsa_b, 0.0, &[], 0.0, "second");
        let s3 = sd.evaluate(vsa_a, 0.0, &[], 0.0, "repeat");
        assert!(s1.novelty > s3.novelty);
    }

    #[test]
    fn test_surprise_contributes_to_composite() {
        let mut sd = SalienceDetector::new();
        let vsa = make_vsa(10);
        let low = sd.evaluate(vsa.clone(), 0.1, &[], 0.0, "low");
        let high = sd.evaluate(vsa, 0.9, &[], 0.0, "high");
        assert!(high.composite > low.composite);
    }

    #[test]
    fn test_goal_relevance_matching() {
        let mut sd = SalienceDetector::new();
        let goal = make_vsa(99);
        sd.add_goal_prototype(goal.clone(), "primary");
        let vsa = make_vsa(100);
        let signal = sd.evaluate(vsa, 0.0, &[&goal], 0.0, "goal_test");
        assert!(signal.goal_relevance >= 0.0);
        let no_goal = sd.evaluate(make_vsa(200), 0.0, &[], 0.0, "no_goal");
        assert_eq!(no_goal.goal_relevance, 0.0);
    }

    #[test]
    fn test_emotional_intensity_factor() {
        let mut sd = SalienceDetector::new();
        let vsa = make_vsa(50);
        let calm = sd.evaluate(vsa.clone(), 0.0, &[], 0.1, "calm");
        let intense = sd.evaluate(vsa, 0.0, &[], 0.9, "intense");
        assert!(intense.emotional_intensity > calm.emotional_intensity);
        assert!(intense.composite > calm.composite);
    }

    #[test]
    fn test_update_trace() {
        let mut sd = SalienceDetector::new();
        assert_eq!(sd.novelty_trace.len(), 0);
        sd.update_trace(make_vsa(1));
        assert_eq!(sd.novelty_trace.len(), 1);
        sd.update_trace(make_vsa(2));
        assert_eq!(sd.novelty_trace.len(), 2);
    }

    #[test]
    fn test_add_goal_prototype() {
        let mut sd = SalienceDetector::new();
        assert_eq!(sd.goal_prototypes.len(), 0);
        sd.add_goal_prototype(make_vsa(1), "curiosity");
        assert_eq!(sd.goal_prototypes.len(), 1);
        assert_eq!(sd.goal_prototypes[0].1, "curiosity");
    }

    #[test]
    fn test_decay_signals() {
        let mut sd = SalienceDetector::new();
        sd.signal_timeout_ms = 1;
        sd.evaluate(make_vsa(1), 0.0, &[], 0.0, "s1");
        sd.evaluate(make_vsa(2), 0.0, &[], 0.0, "s2");
        assert_eq!(sd.signals.len(), 2);
        sd.cycle_count = 10;
        sd.decay_signals();
        assert_eq!(sd.signals.len(), 0);
    }

    #[test]
    fn test_top_signals() {
        let mut sd = SalienceDetector::new();
        sd.weights.emotional_intensity = 0.0;
        sd.weights.goal_relevance = 0.0;
        sd.weights.surprise = 1.0;
        sd.weights.novelty = 0.0;
        let low = sd.evaluate(make_vsa(1), 0.2, &[], 0.0, "low");
        let mid = sd.evaluate(make_vsa(2), 0.5, &[], 0.0, "mid");
        let high = sd.evaluate(make_vsa(3), 0.9, &[], 0.0, "high");
        let top = sd.top_signals(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].label, "high");
        assert_eq!(top[1].label, "mid");
    }

    #[test]
    fn test_set_weight() {
        let mut sd = SalienceDetector::new();
        assert!((sd.weights.novelty - 0.25).abs() < 1e-6);
        sd.set_weight("novelty", 0.8);
        assert!((sd.weights.novelty - 0.8).abs() < 1e-6);
        sd.set_weight("nonexistent", 0.5);
        assert!((sd.weights.novelty - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut sd = SalienceDetector::new();
        sd.evaluate(make_vsa(1), 0.5, &[], 0.3, "test");
        sd.add_goal_prototype(make_vsa(2), "goal");
        sd.update_trace(make_vsa(3));
        assert!(sd.signal_count() > 0);
        assert!(!sd.goal_prototypes.is_empty());
        assert!(!sd.novelty_trace.is_empty());
        sd.reset();
        assert_eq!(sd.signal_count(), 0);
        assert!(sd.goal_prototypes.is_empty());
        assert!(sd.novelty_trace.is_empty());
        assert_eq!(sd.cycle_count, 0);
    }

    #[test]
    fn test_novelty_decreases_with_repetition() {
        let mut sd = SalienceDetector::new();
        let vsa = make_vsa(77);
        let first = sd.evaluate(vsa.clone(), 0.0, &[], 0.0, "first");
        let second = sd.evaluate(vsa.clone(), 0.0, &[], 0.0, "second");
        let third = sd.evaluate(vsa, 0.0, &[], 0.0, "third");
        assert!(first.novelty > second.novelty);
        assert!(second.novelty > third.novelty || (second.novelty - third.novelty).abs() < 1e-6);
    }

    #[test]
    fn test_signal_count() {
        let mut sd = SalienceDetector::new();
        assert_eq!(sd.signal_count(), 0);
        sd.evaluate(make_vsa(1), 0.0, &[], 0.0, "a");
        assert_eq!(sd.signal_count(), 1);
        sd.evaluate(make_vsa(2), 0.0, &[], 0.0, "b");
        assert_eq!(sd.signal_count(), 2);
    }

    #[test]
    fn test_max_trace_enforced() {
        let mut sd = SalienceDetector::new();
        sd.max_trace = 5;
        for i in 0..10 {
            sd.update_trace(make_vsa(i));
        }
        assert_eq!(sd.novelty_trace.len(), 5);
    }
}
