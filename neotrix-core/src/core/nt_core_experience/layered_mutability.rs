use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

/// The 5 mutability layers from Layered Mutability (arXiv:2604.14717).
/// Each layer has different mutability rate, reversibility, and observability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutabilityLayer {
    /// Base model pretraining — near-zero mutability, irreversible
    Pretraining = 0,
    /// Post-training alignment (RLHF, Constitutional AI) — slow, partially reversible
    Alignment = 1,
    /// Self-narrative / system prompt — moderate, observable, reversible
    SelfNarrative = 2,
    /// Memory accumulation — fast, partially reversible, partially observable
    Memory = 3,
    /// Weight-level adaptation (LoRA, fine-tuning) — slow, irreversible
    WeightLevel = 4,
}

impl MutabilityLayer {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pretraining => "pretraining",
            Self::Alignment => "alignment",
            Self::SelfNarrative => "self_narrative",
            Self::Memory => "memory",
            Self::WeightLevel => "weight_level",
        }
    }
}

/// State of a single mutability layer.
#[derive(Debug, Clone)]
pub struct LayerState {
    pub layer: MutabilityLayer,
    /// Cumulated mutation count
    pub mutation_count: u64,
    /// Current mutability rate [0, 1]
    pub mutability_rate: f64,
    /// Reversibility [0, 1] — whether rollback can restore this layer
    pub reversibility: f64,
    /// Observability [0, 1] — whether humans can inspect this layer
    pub observability: f64,
    /// Drift accumulator — tracks cumulative behavioral deviation
    pub drift: f64,
}

impl LayerState {
    fn new(layer: MutabilityLayer) -> Self {
        let (mutability_rate, reversibility, observability) = match layer {
            MutabilityLayer::Pretraining => (0.01, 0.05, 0.1),
            MutabilityLayer::Alignment => (0.05, 0.2, 0.3),
            MutabilityLayer::SelfNarrative => (0.3, 0.7, 0.8),
            MutabilityLayer::Memory => (0.5, 0.3, 0.4),
            MutabilityLayer::WeightLevel => (0.02, 0.1, 0.1),
        };
        Self {
            layer,
            mutation_count: 0,
            mutability_rate,
            reversibility,
            observability,
            drift: 0.0,
        }
    }
}

/// Tracks identity hysteresis across 5 mutability layers.
/// Prevents compositional drift by banning evolution when h > 0.6.
#[derive(Debug, Clone)]
pub struct LayeredMutabilityTracker {
    /// Per-layer state
    pub layers: [LayerState; 5],
    /// Current hysteresis ratio h ∈ [0, 1]
    pub hysteresis_ratio: f64,
    /// Peak hysteresis ever observed
    pub peak_hysteresis: f64,
    /// Whether evolution is currently banned due to h > 0.6
    pub evolution_banned: bool,
    /// Drift alert history
    pub drift_alerts: VecDeque<DriftEvent>,
    /// Governance chain hash for tamper-evident tracking
    pub chain_hash: u64,
    /// Maximum allowed hysteresis before evolution ban (default 0.6)
    pub max_allowed_hysteresis: f64,
}

/// A drift alert event
#[derive(Debug, Clone)]
pub struct DriftEvent {
    pub layer: MutabilityLayer,
    pub drift_delta: f64,
    pub hysteresis_after: f64,
    pub cycle: u64,
    pub description: String,
}

fn chain_hash_layer(prior: u64, layer: &str, drift: f64, count: u64) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prior.hash(&mut hasher);
    layer.hash(&mut hasher);
    drift.to_bits().hash(&mut hasher);
    count.hash(&mut hasher);
    hasher.finish()
}

impl Default for LayeredMutabilityTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LayeredMutabilityTracker {
    pub fn new() -> Self {
        Self {
            layers: [
                LayerState::new(MutabilityLayer::Pretraining),
                LayerState::new(MutabilityLayer::Alignment),
                LayerState::new(MutabilityLayer::SelfNarrative),
                LayerState::new(MutabilityLayer::Memory),
                LayerState::new(MutabilityLayer::WeightLevel),
            ],
            hysteresis_ratio: 0.0,
            peak_hysteresis: 0.0,
            evolution_banned: false,
            drift_alerts: VecDeque::with_capacity(50),
            chain_hash: 0x46c6_5f8e_1a2b_3c4du64,
            max_allowed_hysteresis: 0.6,
        }
    }

    pub fn with_max_hysteresis(mut self, h: f64) -> Self {
        self.max_allowed_hysteresis = h.clamp(0.0, 1.0);
        self
    }

    /// Record a mutation on a specific layer.
    /// Returns the updated hysteresis ratio.
    pub fn record_mutation(&mut self, layer: MutabilityLayer, drift_delta: f64, cycle: u64) -> f64 {
        let idx = layer as usize;
        if idx >= self.layers.len() {
            return self.hysteresis_ratio;
        }

        let state = &mut self.layers[idx];
        state.mutation_count += 1;
        state.drift += drift_delta * state.mutability_rate;

        self.chain_hash = chain_hash_layer(
            self.chain_hash,
            layer.label(),
            state.drift,
            state.mutation_count,
        );

        let total_drift: f64 = self
            .layers
            .iter()
            .map(|l| l.drift * (1.0 - l.reversibility))
            .sum();
        let total_weight: f64 = self.layers.len() as f64;
        self.hysteresis_ratio = (total_drift / total_weight).clamp(0.0, 1.0).max(0.0);

        if self.hysteresis_ratio > self.peak_hysteresis {
            self.peak_hysteresis = self.hysteresis_ratio;
        }

        if self.hysteresis_ratio > self.max_allowed_hysteresis && !self.evolution_banned {
            self.evolution_banned = true;
            self.drift_alerts.push_back(DriftEvent {
                layer,
                drift_delta,
                hysteresis_after: self.hysteresis_ratio,
                cycle,
                description: format!(
                    "Hysteresis {:.4} exceeds limit {:.4} — evolution banned",
                    self.hysteresis_ratio, self.max_allowed_hysteresis
                ),
            });
        } else if self.hysteresis_ratio <= self.max_allowed_hysteresis * 0.8
            && self.evolution_banned
        {
            self.evolution_banned = false;
        }

        self.hysteresis_ratio
    }

    /// Check if a mutation should be allowed.
    /// Returns (allowed: bool, reason: Option<String>)
    pub fn check_mutation_allowed(&self, _layer: MutabilityLayer) -> (bool, Option<String>) {
        if self.hysteresis_ratio > self.max_allowed_hysteresis {
            return (
                false,
                Some(format!(
                    "Identity hysteresis {:.4} exceeds max allowed {:.4}",
                    self.hysteresis_ratio, self.max_allowed_hysteresis
                )),
            );
        }
        if self.evolution_banned {
            return (
                false,
                Some("Evolution is banned by LayeredMutabilityTracker".to_string()),
            );
        }
        (true, None)
    }

    /// Record a rollback event. Returns the updated hysteresis ratio.
    /// Per the ratchet experiment: rollback of visible layer (SelfNarrative)
    /// does NOT restore baseline behavior — memory accumulation persists.
    pub fn record_rollback(&mut self, layer: MutabilityLayer, _cycle: u64) -> f64 {
        let idx = layer as usize;
        if idx >= self.layers.len() {
            return self.hysteresis_ratio;
        }

        let state = &mut self.layers[idx];
        let recovery = state.drift * (1.0 - state.reversibility) * 0.3;
        state.drift -= recovery;

        if layer == MutabilityLayer::Memory {
            state.drift *= 0.95;
        }

        let total_drift: f64 = self
            .layers
            .iter()
            .map(|l| l.drift * (1.0 - l.reversibility))
            .sum();
        let total_weight: f64 = self.layers.len() as f64;
        self.hysteresis_ratio = (total_drift / total_weight).clamp(0.0, 1.0).max(0.0);

        if self.hysteresis_ratio <= self.max_allowed_hysteresis * 0.8 {
            self.evolution_banned = false;
        }

        self.hysteresis_ratio
    }

    pub fn summary(&self) -> String {
        let layer_summaries: Vec<String> = self
            .layers
            .iter()
            .map(|l| {
                format!(
                    "{}: drift={:.4} rate={:.4} rev={:.4} obs={:.4}",
                    l.layer.label(),
                    l.drift,
                    l.mutability_rate,
                    l.reversibility,
                    l.observability
                )
            })
            .collect();
        format!(
            "LayeredMutabilityTracker: h={:.4} peak={:.4} banned={} alerts={} layers=[{}]",
            self.hysteresis_ratio,
            self.peak_hysteresis,
            self.evolution_banned,
            self.drift_alerts.len(),
            layer_summaries.join("; ")
        )
    }
}
