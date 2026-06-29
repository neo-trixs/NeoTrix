//! # SelfReasoner — VSA-Only Internal Reasoning
//!
//! Pure VSA-space reasoning without LLM coprocessor.
//! Uses HRR bind/bundle/similarity operations to think,
//! E8 hexagram states for structured reasoning paths,
//! and metacognitive confidence calibration.

use std::collections::VecDeque;

use crate::memory::hypercube::HyperCube;
use crate::scheduler::e8::{E8Machine, Hexagram};

const MAX_REASONING_TRACE: usize = 64;
const NOVELTY_WINDOW: usize = 3;

#[derive(Debug, Clone)]
pub enum ReasonSource {
    Internal,
    E8Structured,
    Coprocessor,
    HyperCubeRetrieval,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub thought_vsa: Vec<f64>,
    pub confidence: f64,
    pub source: ReasonSource,
    pub timestamp_ms: u64,
    pub novelty: f64,
    pub hexagram: Option<Hexagram>,
}

#[derive(Debug, Clone)]
pub struct SelfReasoner {
    pub reasoning_trace: VecDeque<ReasoningStep>,
    pub max_trace: usize,
    pub total_internal_cycles: u64,
    pub last_confidence: f64,
    pub internal_failure_count: u64,
    pub meta_accuracy: f64,

    self_state: Vec<f64>,
    coherence_threshold: f64,
    e8: E8Machine,

    latent_outbox: VecDeque<Vec<f64>>,
    latent_inbox: VecDeque<Vec<f64>>,
    latent_channel_open: bool,
}

impl SelfReasoner {
    pub fn new() -> Self {
        let self_state = HyperCube.seeded_vector(42);

        Self {
            reasoning_trace: VecDeque::with_capacity(MAX_REASONING_TRACE),
            max_trace: MAX_REASONING_TRACE,
            total_internal_cycles: 0,
            last_confidence: 1.0,
            internal_failure_count: 0,
            meta_accuracy: 1.0,
            self_state,
            coherence_threshold: 0.6,
            e8: E8Machine::new(Hexagram::new(0b101010)),
            latent_outbox: VecDeque::with_capacity(16),
            latent_inbox: VecDeque::with_capacity(16),
            latent_channel_open: false,
        }
    }

    pub fn think(&mut self) -> f64 {
        self.total_internal_cycles += 1;

        // Generate thought from current E8 state
        let thought = HyperCube.seeded_vector(self.e8.current.bits as u64 * 37 + self.total_internal_cycles);

        // Compute confidence as similarity to self_state
        let conviction = HyperCube::similarity(&thought, &self.self_state);
        let raw_confidence = if self.coherence_threshold > 0.0 {
            (conviction / self.coherence_threshold).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let confidence = self.calibrate_confidence(raw_confidence);

        // Novelty: dissimilarity to recent thoughts
        let recent: Vec<&[f64]> = self
            .reasoning_trace
            .iter()
            .rev()
            .take(NOVELTY_WINDOW)
            .map(|s| s.thought_vsa.as_slice())
            .collect();
        let novelty = if recent.is_empty() {
            1.0
        } else {
            let max_sim = recent
                .iter()
                .map(|r| HyperCube::similarity(&thought, r))
                .fold(f64::NEG_INFINITY, |a: f64, b: f64| a.max(b));
            1.0 - max_sim.clamp(0.0, 1.0)
        };

        // Assimilate into self_state
        if confidence > 0.5 {
            self.self_state = HyperCube::bundle(&self.self_state, &thought);
        }

        // E8 transition
        let neighbors = self.e8.neighbors();
        if !neighbors.is_empty() {
            let next = neighbors[self.total_internal_cycles as usize % neighbors.len()];
            self.e8.transition(next);
        }

        let step = ReasoningStep {
            thought_vsa: thought,
            confidence,
            source: ReasonSource::E8Structured,
            timestamp_ms: now_ms(),
            novelty,
            hexagram: Some(self.e8.current),
        };
        self.push_trace(step);
        self.last_confidence = confidence;

        if confidence < 0.2 {
            self.internal_failure_count += 1;
        }

        confidence
    }

    pub fn needs_coprocessor(&self) -> bool {
        let adjusted = 0.4 * self.meta_accuracy;
        if self.last_confidence < adjusted {
            return true;
        }
        if self.internal_failure_count > 3 && self.last_confidence < 0.5 {
            return true;
        }
        if self.e8.stuck_detection(5) {
            return true;
        }
        false
    }

    pub fn record_outcome(&mut self, predicted: f64, actual: f64) {
        let error = (predicted - actual).abs();
        self.meta_accuracy = (self.meta_accuracy * 0.9 + (1.0 - error) * 0.1).clamp(0.0, 1.0);
    }

    pub fn calibrate_confidence(&self, raw: f64) -> f64 {
        (raw * (0.5 + 0.5 * self.meta_accuracy)).clamp(0.0, 1.0)
    }

    pub fn push_trace(&mut self, step: ReasoningStep) {
        if self.reasoning_trace.len() >= self.max_trace {
            self.reasoning_trace.pop_front();
        }
        self.reasoning_trace.push_back(step);
    }

    pub fn trace_summary(&self) -> String {
        format!(
            "reasoner:steps_{}_conf_{:.3}_ma_{:.3}_fail_{}",
            self.reasoning_trace.len(),
            self.last_confidence,
            self.meta_accuracy,
            self.internal_failure_count,
        )
    }

    pub fn open_latent_channel(&mut self) {
        self.latent_channel_open = true;
    }

    pub fn close_latent_channel(&mut self) {
        self.latent_channel_open = false;
    }

    pub fn is_channel_open(&self) -> bool {
        self.latent_channel_open
    }

    pub fn send_latent(&mut self, thought: Vec<f64>) {
        if !self.latent_channel_open {
            return;
        }
        if self.latent_outbox.len() >= 16 {
            self.latent_outbox.pop_front();
        }
        self.latent_outbox.push_back(thought);
    }

    pub fn receive_latent(&mut self) -> Option<Vec<f64>> {
        self.latent_inbox.pop_front()
    }

    pub fn relay_latent(&mut self, incoming: Vec<f64>) {
        if !self.latent_channel_open {
            return;
        }
        if self.latent_inbox.len() >= 16 {
            self.latent_inbox.pop_front();
        }
        self.latent_inbox.push_back(incoming);
    }

    pub fn flush_latent(&mut self) -> Vec<Vec<f64>> {
        let out: Vec<Vec<f64>> = self.latent_outbox.drain(..).collect();
        out
    }

    pub fn latent_outbox_len(&self) -> usize {
        self.latent_outbox.len()
    }

    pub fn latent_inbox_len(&self) -> usize {
        self.latent_inbox.len()
    }
}

impl Default for SelfReasoner {
    fn default() -> Self {
        Self::new()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_reasoner_think() {
        let mut sr = SelfReasoner::new();
        let confidence = sr.think();
        assert!(confidence >= 0.0 && confidence <= 1.0);
        assert_eq!(sr.total_internal_cycles, 1);
    }

    #[test]
    fn test_reasoning_trace_accumulates() {
        let mut sr = SelfReasoner::new();
        for _ in 0..10 {
            sr.think();
        }
        assert_eq!(sr.reasoning_trace.len(), 10);
    }

    #[test]
    fn test_needs_coprocessor_low_confidence() {
        let mut sr = SelfReasoner::new();
        sr.last_confidence = 0.1;
        sr.meta_accuracy = 0.5;
        assert!(sr.needs_coprocessor());
    }

    #[test]
    fn test_record_outcome_updates_accuracy() {
        let mut sr = SelfReasoner::new();
        sr.record_outcome(0.8, 0.7);
        assert!(sr.meta_accuracy > 0.9);
    }

    #[test]
    fn test_calibrate_confidence() {
        let sr = SelfReasoner::new();
        let calibrated = sr.calibrate_confidence(0.8);
        assert!(calibrated >= 0.0 && calibrated <= 1.0);
    }
}
