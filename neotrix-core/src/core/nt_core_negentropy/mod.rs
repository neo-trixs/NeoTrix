pub mod act_planner;
pub mod dysib_layer;
pub mod efe_minimizer;
pub mod jepa_efe_calculator;
pub mod jepa_transition;
pub use act_planner::*;
pub use dysib_layer::DySIBLayer;
pub use efe_minimizer::*;
pub use jepa_efe_calculator::JepaEfeCalculator;
pub use jepa_transition::JepaTransitionModel;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyMetric {
    pub total: f64,
    pub components: NegentropyComponents,
    pub flux: NegentropyFlux,
    pub history: Vec<f64>,
    pub trend: f64,
    pub demon_efficiency: f64,
    pub subsystem_entropy: HashMap<String, f64>,
    pub system_negentropy: f64,
    /// SHA-256 witness: hash of (total || components_hash || flux_hash || cycle)
    pub witness: Option<[u8; 32]>,
    /// Last cycle number when witness was computed
    pub witness_cycle: u64,
    /// Hex-encoded witness for external display/verification
    pub witness_hex: String,
}

impl Default for NegentropyMetric {
    fn default() -> Self {
        Self {
            total: 0.0,
            components: NegentropyComponents::default(),
            flux: NegentropyFlux::default(),
            history: Vec::with_capacity(100),
            trend: 0.0,
            demon_efficiency: 0.0,
            subsystem_entropy: HashMap::new(),
            system_negentropy: 0.0,
            witness: None,
            witness_cycle: 0,
            witness_hex: String::new(),
        }
    }
}

impl NegentropyMetric {
    pub fn record(&mut self, components: NegentropyComponents, flux: NegentropyFlux) {
        self.total = components.weighted_total();
        self.components = components;
        self.flux = flux;
        self.history.push(self.total);
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        self.trend = self.compute_trend();
        self.demon_efficiency = self.compute_demon_efficiency();
        self.record_witness(self.history.len() as u64);
    }

    pub fn compute_trend(&self) -> f64 {
        let n = self.history.len();
        if n < 10 {
            return 0.0;
        }
        let recent: f64 = self.history[n - 10..].iter().sum::<f64>() / 10.0;
        let older: f64 = if n >= 20 {
            self.history[n - 20..n - 10].iter().sum::<f64>() / 10.0
        } else {
            self.history[..n - 10].iter().sum::<f64>() / (n - 10) as f64
        };
        recent - older
    }

    pub fn compute_demon_efficiency(&self) -> f64 {
        let operational_bits = self.flux.operational_cost.max(0.01);
        self.total / operational_bits
    }

    pub fn trend_desc(&self) -> &'static str {
        if self.trend > 0.05 {
            "growing"
        } else if self.trend < -0.05 {
            "declining"
        } else {
            "stable"
        }
    }

    pub fn health(&self) -> &'static str {
        if self.total > 0.7 {
            "excellent"
        } else if self.total > 0.5 {
            "good"
        } else if self.total > 0.3 {
            "fair"
        } else {
            "critical"
        }
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let c = &self.components;
        vec![
            format!(
                "  N_total={:.4} ({}) trend={:+.4}/iter",
                self.total,
                self.health(),
                self.trend
            ),
            format!(
                "  Φ={:.4}  VSA_coh={:.4}  KB_order={:.4}  Pred={:.4}",
                c.phi, c.vsa_coherence, c.kb_order, c.prediction_acc
            ),
            format!(
                "  Attn={:.4}  Strat_diff={:.4}  Temp_coh={:.4}",
                c.attention_focus, c.strategy_diff, c.temporal_coherence
            ),
            format!(
                "  Flux: import={:.2}/s  export={:.2}/s  η_demon={:.4}",
                self.flux.import_rate, self.flux.export_rate, self.demon_efficiency
            ),
        ]
    }

    pub fn history_slice(&self) -> &[f64] {
        &self.history
    }

    pub fn record_subsystem_entropy(&mut self, subsystem: &str, entropy: f64) {
        self.subsystem_entropy
            .insert(subsystem.to_string(), entropy);
        self.system_negentropy = self.total - self.subsystem_entropy.values().sum::<f64>();
    }

    /// Compute a SHA-256 witness over the current metric state.
    /// The witness commits to (total, components_hash, flux_hash, cycle) so that
    /// anyone can later verify that a given N_total value was indeed produced at a given cycle.
    pub fn compute_witness(&self, cycle: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.total.to_le_bytes());
        hasher.update(&self.components.weighted_total().to_le_bytes());
        hasher.update(&self.flux.net_flux.to_le_bytes());
        hasher.update(&self.flux.operational_cost.to_le_bytes());
        hasher.update(&cycle.to_le_bytes());
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }

    /// Record a witness at the given cycle, storing the hash.
    pub fn record_witness(&mut self, cycle: u64) -> [u8; 32] {
        let w = self.compute_witness(cycle);
        self.witness = Some(w);
        self.witness_cycle = cycle;
        self.witness_hex = hex::encode(w);
        w
    }

    /// Verify a witness commitment.
    /// Returns true if the given digest matches a recomputation of the witness at the stored cycle.
    pub fn verify_witness(&self, claimed: &[u8; 32]) -> bool {
        let recomputed = self.compute_witness(self.witness_cycle);
        &recomputed == claimed
    }

    /// Verify the stored witness is valid.
    pub fn verify_self(&self) -> bool {
        match self.witness {
            Some(w) => self.verify_witness(&w),
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyComponents {
    pub phi: f64,
    pub vsa_coherence: f64,
    pub kb_order: f64,
    pub prediction_acc: f64,
    pub attention_focus: f64,
    pub strategy_diff: f64,
    pub temporal_coherence: f64,
}

impl NegentropyComponents {
    pub const WEIGHTS: [f64; 7] = [0.25, 0.15, 0.20, 0.15, 0.10, 0.05, 0.10];

    pub fn weighted_total(&self) -> f64 {
        let vals = [
            self.phi,
            self.vsa_coherence,
            self.kb_order,
            self.prediction_acc,
            self.attention_focus,
            self.strategy_diff,
            self.temporal_coherence,
        ];
        vals.iter()
            .zip(Self::WEIGHTS.iter())
            .map(|(v, w)| v * w)
            .sum()
    }

    pub fn active_mask(&self) -> u32 {
        let mut mask = 0u32;
        if self.phi > 0.0 {
            mask |= 1 << 0;
        }
        if self.vsa_coherence > 0.0 {
            mask |= 1 << 1;
        }
        if self.kb_order > 0.0 {
            mask |= 1 << 2;
        }
        if self.prediction_acc > 0.0 {
            mask |= 1 << 3;
        }
        if self.attention_focus > 0.0 {
            mask |= 1 << 4;
        }
        if self.strategy_diff > 0.0 {
            mask |= 1 << 5;
        }
        if self.temporal_coherence > 0.0 {
            mask |= 1 << 6;
        }
        mask
    }
}

impl Default for NegentropyComponents {
    fn default() -> Self {
        Self {
            phi: 0.0,
            vsa_coherence: 0.0,
            kb_order: 0.0,
            prediction_acc: 0.0,
            attention_focus: 0.0,
            strategy_diff: 0.0,
            temporal_coherence: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyFlux {
    pub import_rate: f64,
    pub export_rate: f64,
    pub net_flux: f64,
    pub efficiency: f64,
    pub operational_cost: f64,
}

impl Default for NegentropyFlux {
    fn default() -> Self {
        Self {
            import_rate: 0.0,
            export_rate: 0.0,
            net_flux: 0.0,
            efficiency: 0.0,
            operational_cost: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyReport {
    pub iteration: u64,
    pub metric: NegentropyMetric,
    pub delta: f64,
    pub regime: NegentropyRegime,
    pub recommendation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NegentropyRegime {
    Growing,
    Stable,
    Plateau,
    Declining,
    Critical,
}

impl NegentropyReport {
    pub fn new(iteration: u64, metric: &NegentropyMetric, prev_total: f64) -> Self {
        let delta = metric.total - prev_total;
        let regime = if metric.total < 0.2 {
            NegentropyRegime::Critical
        } else if metric.trend < -0.05 {
            NegentropyRegime::Declining
        } else if metric.trend.abs() < 0.02 {
            if metric.total > 0.6 {
                NegentropyRegime::Stable
            } else {
                NegentropyRegime::Plateau
            }
        } else {
            NegentropyRegime::Growing
        };

        let recommendation = match regime {
            NegentropyRegime::Growing => "maintain current trajectory",
            NegentropyRegime::Stable => "monitor for drift",
            NegentropyRegime::Plateau => "increase exploration, inject noise",
            NegentropyRegime::Declining => "run consolidation cycle, prune stale knowledge",
            NegentropyRegime::Critical => "emergency: trigger deep consolidation, halt ingestion",
        };

        Self {
            iteration,
            metric: metric.clone(),
            delta,
            regime,
            recommendation,
        }
    }
}
