use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::phi_bridge::ConsciousnessState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCoherence {
    pub mean_phi: f64,
    pub max_phi: f64,
    pub min_phi: f64,
    pub mean_coherence: f64,
    pub mean_awareness: f64,
    pub social_connectivity: f64,
    pub diversity_index: f64,
    pub population: usize,
    pub timestamp: u64,
}

impl Default for GlobalCoherence {
    fn default() -> Self {
        Self {
            mean_phi: 0.0,
            max_phi: 0.0,
            min_phi: 1.0,
            mean_coherence: 0.0,
            mean_awareness: 0.0,
            social_connectivity: 0.0,
            diversity_index: 0.0,
            population: 0,
            timestamp: 0,
        }
    }
}

pub struct CoherenceTracker {
    history: Vec<GlobalCoherence>,
    max_history: usize,
}

impl CoherenceTracker {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn compute(
        &mut self,
        states: &HashMap<String, ConsciousnessState>,
        tick: u64,
    ) -> GlobalCoherence {
        if states.is_empty() {
            return GlobalCoherence::default();
        }

        let phi_values: Vec<f64> = states.values().map(|s| s.phi).collect();
        let coherence_values: Vec<f64> = states.values().map(|s| s.coherence).collect();
        let awareness_values: Vec<f64> = states.values().map(|s| s.awareness).collect();
        let social_values: Vec<f64> = states.values().map(|s| s.social_awareness).collect();

        let mean = |v: &[f64]| -> f64 { v.iter().sum::<f64>() / v.len() as f64 };

        let global = GlobalCoherence {
            mean_phi: mean(&phi_values),
            max_phi: phi_values.iter().cloned().fold(f64::MIN, f64::max),
            min_phi: phi_values.iter().cloned().fold(f64::MAX, f64::min),
            mean_coherence: mean(&coherence_values),
            mean_awareness: mean(&awareness_values),
            social_connectivity: mean(&social_values),
            diversity_index: Self::compute_diversity(states),
            population: states.len(),
            timestamp: tick,
        };

        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(global.clone());

        global
    }

    fn compute_diversity(states: &HashMap<String, ConsciousnessState>) -> f64 {
        if states.is_empty() {
            return 0.0;
        }

        let mut buckets = [0u32; 10];
        for state in states.values() {
            let bucket = (state.consciousness_level() * 9.0) as usize;
            buckets[bucket.min(9)] += 1;
        }

        let total = states.len() as f64;
        let mut entropy = 0.0;
        for &count in &buckets {
            if count > 0 {
                let p = count as f64 / total;
                entropy -= p * p.log2();
            }
        }

        entropy / 10.0_f64.log2()
    }

    pub fn trend(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }
        let recent = &self.history[self.history.len() - 1];
        let previous = &self.history[self.history.len() - 2];
        recent.mean_phi - previous.mean_phi
    }

    pub fn history(&self) -> &[GlobalCoherence] {
        &self.history
    }

    pub fn current(&self) -> Option<&GlobalCoherence> {
        self.history.last()
    }
}

impl Default for CoherenceTracker {
    fn default() -> Self {
        Self::new(100)
    }
}
