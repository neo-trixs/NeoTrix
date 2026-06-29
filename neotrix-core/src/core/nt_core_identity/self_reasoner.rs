use std::collections::VecDeque;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::core::nt_core_hcube::HrrBackend;
use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_reasoning::{Hypothesis, ReasonerConfig, VsaBlackboard, VsaReasoner};

const MAX_REASONING_TRACE: usize = 64;
const META_ACCURACY_WINDOW: usize = 100;
const E8_HISTORY_LEN: usize = 10;
const NOVELTY_WINDOW: usize = 3;

#[derive(Debug, Clone)]
pub enum ReasonSource {
    Internal,
    E8Structured,
    Coprocessor,
    HyperCubeRetrieval,
    E8Resonance,
}

impl std::fmt::Display for ReasonSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonSource::Internal => write!(f, "internal"),
            ReasonSource::E8Structured => write!(f, "e8"),
            ReasonSource::Coprocessor => write!(f, "coprocessor"),
            ReasonSource::HyperCubeRetrieval => write!(f, "hcube"),
            ReasonSource::E8Resonance => write!(f, "e8_resonance"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub thought_vsa: Vec<u8>,
    pub confidence: f64,
    pub source: ReasonSource,
    pub timestamp_ms: u64,
    pub novelty: f64,
    pub hexagram: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct PredictionRecord {
    pub predicted_confidence: f64,
    pub actual_outcome: f64,
    #[allow(dead_code)]
    pub cycle: u64,
}

#[derive(Debug, Clone)]
pub struct SelfReasoner {
    pub reasoning_trace: VecDeque<ReasoningStep>,
    pub max_trace: usize,
    pub max_thoughts_per_cycle: usize,
    pub internal_thoughts_produced: u64,
    pub total_internal_cycles: u64,
    pub cycle_thoughts: Vec<Vec<u8>>,
    pub last_confidence: f64,
    pub internal_failure_count: u64,
    pub meta_accuracy: f64,
    pub current_hexagram: ReasoningHexagram,

    prediction_history: VecDeque<PredictionRecord>,
    meta_accuracy_window: usize,

    // ── Real VSA internals ──
    vsa_backend: HrrBackend,
    self_state: Vec<f64>,
    coherence_threshold: f64,
    e8_state_history: VecDeque<ReasoningHexagram>,

    /// 6 role vectors, one per hexagram axis (bit 5 → bit 0)
    role_vectors: Vec<Vec<f64>>,
    /// 6 filler pairs [zero_val, one_val] per axis
    filler_pairs: Vec<[Vec<f64>; 2]>,

    /// PRISM-style blackboard for VSA-only reasoning
    pub blackboard: Option<VsaBlackboard>,
    /// VSA reasoning engine operating on the blackboard
    pub vsa_reasoner: Option<VsaReasoner>,
}

impl SelfReasoner {
    /// Create a new SelfReasoner with the given seed and configuration.
    pub fn new() -> Self {
        let vsa_backend = HrrBackend::new(4096);
        let mut rng = StdRng::seed_from_u64(42);
        let self_state = vsa_backend.random_vector(&mut rng);

        let role_vectors: Vec<Vec<f64>> = (0..6)
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(1000 + i as u64);
                vsa_backend.random_vector(&mut rng)
            })
            .collect();

        let filler_pairs: Vec<[Vec<f64>; 2]> = (0..6)
            .map(|i| {
                let mut rng0 = StdRng::seed_from_u64(2000 + i as u64 * 2);
                let mut rng1 = StdRng::seed_from_u64(2000 + i as u64 * 2 + 1);
                [
                    vsa_backend.random_vector(&mut rng0),
                    vsa_backend.random_vector(&mut rng1),
                ]
            })
            .collect();

        Self {
            reasoning_trace: VecDeque::with_capacity(MAX_REASONING_TRACE),
            max_trace: MAX_REASONING_TRACE,
            max_thoughts_per_cycle: 6,
            internal_thoughts_produced: 0,
            total_internal_cycles: 0,
            cycle_thoughts: Vec::with_capacity(6),
            last_confidence: 1.0,
            internal_failure_count: 0,
            meta_accuracy: 1.0,
            current_hexagram: ReasoningHexagram::new(0b101010),

            prediction_history: VecDeque::with_capacity(META_ACCURACY_WINDOW),
            meta_accuracy_window: META_ACCURACY_WINDOW,

            vsa_backend,
            self_state,
            coherence_threshold: 0.6,
            e8_state_history: {
                let mut h = VecDeque::with_capacity(E8_HISTORY_LEN);
                h.push_back(ReasoningHexagram::new(0b101010));
                h
            },
            role_vectors,
            filler_pairs,
            blackboard: Some(VsaBlackboard::new(256)),
            vsa_reasoner: Some(VsaReasoner::new(ReasonerConfig::default())),
        }
    }

    pub fn think_internal(&mut self, _context_vsa: &[u8], _identity_vsa: &[u8]) -> f64 {
        self.total_internal_cycles += 1;
        self.cycle_thoughts.clear();

        let hexagram = self.current_hexagram;

        // 1. Generate thought vector from hexagram via role/filler binding
        let thought = self.hexagram_to_vsa(&hexagram);

        // 2. Compute conviction (similarity between thought and self_state)
        let conviction = HrrBackend::similarity(&thought, &self.self_state);

        // 3. Compute confidence
        let raw_confidence = if self.coherence_threshold > 0.0 {
            (conviction / self.coherence_threshold).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let confidence = self.calibrate_confidence(raw_confidence);

        // 4. Compute novelty: similarity to recent thoughts
        let recent: Vec<Vec<f64>> = self
            .reasoning_trace
            .iter()
            .rev()
            .take(NOVELTY_WINDOW)
            .map(|s| {
                s.thought_vsa
                    .iter()
                    .map(|&b| b as f64 / 127.5 - 1.0)
                    .collect()
            })
            .collect();
        let novelty = if recent.is_empty() {
            1.0
        } else {
            let max_sim = recent
                .iter()
                .map(|r| HrrBackend::similarity(&thought, r))
                .fold(f64::NEG_INFINITY, |a, b| a.max(b));
            1.0 - max_sim.clamp(0.0, 1.0)
        };

        // 5. Assimilation: if confidence > 0.5, bundle thought into self_state
        if confidence > 0.5 {
            let refs = [self.self_state.as_slice(), thought.as_slice()];
            self.self_state = HrrBackend::bundle(&refs);
        }

        // 6. E8 state transition: move toward neighbor best matching the thought
        let neighbors = hexagram.neighbors();
        let best_neighbor = neighbors
            .iter()
            .max_by(|a, b| {
                let va = self.hexagram_to_vsa(a);
                let vb = self.hexagram_to_vsa(b);
                let sim_a = HrrBackend::similarity(&va, &thought);
                let sim_b = HrrBackend::similarity(&vb, &thought);
                sim_a
                    .partial_cmp(&sim_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(hexagram);

        // 20% exploration chance
        let explore = {
            let mut rng = StdRng::seed_from_u64(self.total_internal_cycles.wrapping_mul(37));
            rand::Rng::gen_bool(&mut rng, 0.2)
        };

        let next_hexagram = if explore && !neighbors.is_empty() {
            let mut rng =
                StdRng::seed_from_u64(self.total_internal_cycles.wrapping_mul(73).wrapping_add(17));
            neighbors[rand::Rng::gen_range(&mut rng, 0..neighbors.len())]
        } else {
            best_neighbor
        };

        self.current_hexagram = next_hexagram;
        self.e8_state_history.push_back(next_hexagram);
        if self.e8_state_history.len() > E8_HISTORY_LEN {
            self.e8_state_history.pop_front();
        }

        // 7. Store thought
        let thought_u8: Vec<u8> = thought.iter().map(|&x| f64_to_u8(x)).collect();
        self.cycle_thoughts.push(thought_u8.clone());

        // 8. Record reasoning step
        let step = ReasoningStep {
            thought_vsa: thought_u8,
            confidence,
            source: ReasonSource::E8Structured,
            timestamp_ms: now_ms(),
            novelty,
            hexagram: Some(hexagram.0),
        };
        self.push_trace(step);

        self.internal_thoughts_produced += 1;
        self.last_confidence = confidence;

        if self.cycle_thoughts.is_empty() {
            self.internal_failure_count += 1;
            self.last_confidence = 0.0;
            return 0.0;
        }

        // 9. Optional blackboard reasoning path
        if let Some(ref mut reasoner) = self.vsa_reasoner {
            let context = if !_context_vsa.is_empty() {
                vec![_context_vsa.to_vec()]
            } else {
                let recent: Vec<Vec<u8>> = self
                    .reasoning_trace
                    .iter()
                    .rev()
                    .take(3)
                    .map(|s| s.thought_vsa.clone())
                    .collect();
                recent
            };
            if let Some(best) = reasoner.reason_cycle(_identity_vsa, &context) {
                let blackboard_step = ReasoningStep {
                    thought_vsa: best.content,
                    confidence: best.confidence * 0.85,
                    source: ReasonSource::Internal,
                    timestamp_ms: now_ms(),
                    novelty: 0.7,
                    hexagram: None,
                };
                self.push_trace(blackboard_step);
                self.last_confidence = self.last_confidence.max(best.confidence * 0.85);
            }
        }

        confidence
    }

    pub fn push_trace(&mut self, step: ReasoningStep) {
        if self.reasoning_trace.len() >= self.max_trace {
            self.reasoning_trace.pop_front();
        }
        self.reasoning_trace.push_back(step);
    }

    pub fn needs_coprocessor(&self) -> bool {
        let adjusted_threshold = 0.4 * self.meta_accuracy;
        if self.last_confidence < adjusted_threshold {
            return true;
        }
        if self.internal_failure_count > 3 && self.last_confidence < 0.5 {
            return true;
        }
        // Stuck detection: E8 state unchanged for 5+ steps
        if self.e8_state_history.len() >= 5 {
            let recent: Vec<ReasoningHexagram> = self
                .e8_state_history
                .iter()
                .rev()
                .take(5)
                .copied()
                .collect();
            if recent.windows(2).all(|w| w[0] == w[1]) {
                return true;
            }
        }
        // Oscillation detection
        if self.reasoning_trace.len() >= 4 {
            let recent_steps: Vec<&ReasoningStep> =
                self.reasoning_trace.iter().rev().take(4).collect();
            let mut oscillating = true;
            for w in recent_steps.windows(2) {
                let delta = (w[0].confidence - w[1].confidence).abs();
                if delta <= 0.3 {
                    oscillating = false;
                    break;
                }
            }
            if oscillating {
                return true;
            }
        }
        false
    }

    pub fn record_outcome(&mut self, predicted_confidence: f64, actual_outcome: f64) {
        if self.prediction_history.len() >= self.meta_accuracy_window {
            self.prediction_history.pop_front();
        }
        self.prediction_history.push_back(PredictionRecord {
            predicted_confidence,
            actual_outcome,
            cycle: self.total_internal_cycles,
        });

        if self.prediction_history.len() >= 10 {
            let total_abs_error: f64 = self
                .prediction_history
                .iter()
                .map(|r| (r.predicted_confidence - r.actual_outcome).abs())
                .sum();
            let n = self.prediction_history.len() as f64;
            self.meta_accuracy = (1.0 - total_abs_error / n).clamp(0.0, 1.0);
        }
    }

    pub fn meta_accuracy(&self) -> f64 {
        self.meta_accuracy
    }

    pub fn calibrate_confidence(&self, raw_confidence: f64) -> f64 {
        let ma = self.meta_accuracy;
        let adjusted = raw_confidence * (0.5 + 0.5 * ma);
        adjusted.clamp(0.0, 1.0)
    }

    pub fn trace_summary(&self) -> String {
        let total = self.reasoning_trace.len();
        let by_source = ["internal", "e8", "coprocessor", "hcube", "e8_resonance"]
            .iter()
            .map(|s| {
                let count = self
                    .reasoning_trace
                    .iter()
                    .filter(|t| &t.source.to_string() == s)
                    .count();
                format!("{}:{}", s, count)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "trace:{}_steps_by:{}_conf:{:.2}_ma:{:.3}",
            total, by_source, self.last_confidence, self.meta_accuracy
        )
    }

    pub fn integrate_coprocessor_result(&mut self, thought_vsa: Vec<u8>, confidence: f64) {
        // Convert u8 → f64 VSA vector and merge into self_state
        let insight_vsa: Vec<f64> = thought_vsa
            .iter()
            .map(|&b| b as f64 / 127.5 - 1.0)
            .collect();
        let insight_norm = HrrBackend::normalize(&insight_vsa);

        // Weighted bundle: (self_state + weight * insight) / (1 + weight)
        let weight = confidence * 2.0; // coprocessor results get extra weight
        let mut merged = self.self_state.clone();
        for (m, i) in merged.iter_mut().zip(insight_norm.iter()) {
            *m = (*m + weight * i) / (1.0 + weight);
        }
        self.self_state = HrrBackend::normalize(&merged);

        let step = ReasoningStep {
            thought_vsa,
            confidence,
            source: ReasonSource::Coprocessor,
            timestamp_ms: now_ms(),
            novelty: 0.8,
            hexagram: None,
        };
        self.push_trace(step);
        self.last_confidence = confidence;
        self.internal_failure_count = 0;
    }

    pub fn recent_thoughts(&self, count: usize) -> Vec<&[u8]> {
        self.reasoning_trace
            .iter()
            .rev()
            .take(count)
            .map(|s| s.thought_vsa.as_slice())
            .collect()
    }

    pub fn reset_cycle(&mut self) {
        self.cycle_thoughts.clear();
    }

    // ─── Private helpers ───

    /// Convert a hexagram into a VSA vector by binding role and filler vectors
    /// for each axis, then bundling all 6 axis bindings.
    fn hexagram_to_vsa(&self, hex: &ReasoningHexagram) -> Vec<f64> {
        let mut bindings = Vec::with_capacity(6);

        for axis in 0..6 {
            let role = &self.role_vectors[axis];
            let filler = &self.filler_pairs[axis][hex.axis(axis) as usize];
            let bound = HrrBackend::bind(role, filler);
            bindings.push(bound);
        }

        let refs: Vec<&[f64]> = bindings.iter().map(|v| v.as_slice()).collect();
        if refs.is_empty() {
            return self
                .vsa_backend
                .random_vector(&mut StdRng::seed_from_u64(hex.0 as u64));
        }
        HrrBackend::bundle(&refs)
    }
}

impl SelfReasoner {
    pub fn reason_with_blackboard(&mut self, input: &[u8]) -> Option<Hypothesis> {
        let context: Vec<Vec<u8>> = self
            .recent_thoughts(3)
            .into_iter()
            .map(|s| s.to_vec())
            .collect();
        let reasoner = self.vsa_reasoner.as_mut()?;
        let hypothesis = reasoner.reason_cycle(input, &context);
        if let Some(ref h) = hypothesis {
            let step = ReasoningStep {
                thought_vsa: h.content.clone(),
                confidence: h.confidence * 0.85,
                source: ReasonSource::Internal,
                timestamp_ms: now_ms(),
                novelty: 0.7,
                hexagram: None,
            };
            self.push_trace(step);
            self.last_confidence = self.last_confidence.max(h.confidence * 0.85);
        }
        hypothesis
    }
}

impl Default for SelfReasoner {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Conversion helpers ───

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn f64_to_u8(x: f64) -> u8 {
    ((x.max(-1.0).min(1.0) + 1.0) * 127.5) as u8
}
