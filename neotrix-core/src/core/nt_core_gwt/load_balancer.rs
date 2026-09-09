//! Load Balancer — MoE auxiliary load balancing loss (adapted from MiniMind/GShard).
//!
//! Prevents attention collapse by ensuring balanced expert utilization across
//! GWT specialist modules. Computes auxiliary loss: N * Σ(f_i * P_i) * coef
//!
//! - f_i = actual load fraction for expert i (sliding window)
//! - P_i = mean router probability for expert i
//! - N = number of experts (MODULE_COUNT)
//! - coef = auxiliary loss coefficient

use super::resonance::MODULE_COUNT;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancer {
    /// Sliding window of recent selections (for f_i computation)
    selection_history: Vec<[f64; MODULE_COUNT]>,
    /// Maximum window size
    window_size: usize,
    /// Auxiliary loss coefficient
    aux_loss_coef: f64,
    /// Cumulative selection counts
    selection_counts: [f64; MODULE_COUNT],
    /// Total selections
    total_selections: f64,
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            selection_history: Vec::new(),
            window_size: 128,
            aux_loss_coef: 0.01,
            selection_counts: [0.0; MODULE_COUNT],
            total_selections: 0.0,
        }
    }

    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    pub fn with_coef(mut self, coef: f64) -> Self {
        self.aux_loss_coef = coef;
        self
    }

    /// Record a routing step: gate_probs is the softmax distribution, selected is the expert indices chosen.
    pub fn record_step(&mut self, gate_probs: &[f64; MODULE_COUNT], selected: &[usize]) {
        // Update selection counts
        for &s in selected {
            if s < MODULE_COUNT {
                self.selection_counts[s] += 1.0;
                self.total_selections += 1.0;
            }
        }

        // Record this step's distribution
        self.selection_history.push(*gate_probs);
        if self.selection_history.len() > self.window_size {
            self.selection_history.remove(0);
        }
    }

    /// Compute actual load fraction f_i for each expert (sliding window).
    pub fn compute_load_fractions(&self) -> [f64; MODULE_COUNT] {
        let mut counts = [0.0; MODULE_COUNT];
        let total = self.selection_history.len() as f64;
        if total == 0.0 {
            return [1.0 / MODULE_COUNT as f64; MODULE_COUNT];
        }
        // Count selections from history (simplified: use uniform top-1 assumption)
        for step in &self.selection_history {
            // Find the expert with highest gate probability
            let mut best = 0;
            for i in 1..MODULE_COUNT {
                if step[i] > step[best] {
                    best = i;
                }
            }
            counts[best] += 1.0;
        }
        let mut fractions = [0.0; MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            fractions[i] = counts[i] / total;
        }
        fractions
    }

    /// Compute mean router probability P_i for each expert.
    pub fn compute_mean_probs(&self) -> [f64; MODULE_COUNT] {
        let mut sums = [0.0; MODULE_COUNT];
        let total = self.selection_history.len() as f64;
        if total == 0.0 {
            return [1.0 / MODULE_COUNT as f64; MODULE_COUNT];
        }
        for step in &self.selection_history {
            for i in 0..MODULE_COUNT {
                sums[i] += step[i];
            }
        }
        let mut means = [0.0; MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            means[i] = sums[i] / total;
        }
        means
    }

    /// Compute auxiliary load balancing loss: N * Σ(f_i * P_i) * coef
    pub fn compute_loss(&self) -> f64 {
        let load = self.compute_load_fractions();
        let probs = self.compute_mean_probs();
        let n = MODULE_COUNT as f64;
        let mut loss = 0.0;
        for i in 0..MODULE_COUNT {
            loss += load[i] * probs[i];
        }
        n * loss * self.aux_loss_coef
    }

    /// Compute load entropy (higher = more balanced).
    pub fn load_entropy(&self) -> f64 {
        let load = self.compute_load_fractions();
        let mut entropy = 0.0;
        for &f in &load {
            if f > 1e-10 {
                entropy -= *f * f.ln();
            }
        }
        entropy
    }

    /// Compute imbalance ratio (1.0 = perfectly balanced).
    pub fn imbalance_ratio(&self) -> f64 {
        let load = self.compute_load_fractions();
        let max_load = load.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_load = load.iter().cloned().fold(f64::INFINITY, f64::min);
        if min_load < 1e-10 {
            f64::INFINITY
        } else {
            max_load / min_load
        }
    }

    /// Apply balancing biases to gate logits (complementary to auxiliary loss).
    /// Pushes underused experts up and overused experts down.
    pub fn apply_biases(&self, gate_logits: &mut [f64; MODULE_COUNT]) {
        let load = self.compute_load_fractions();
        let target = 1.0 / MODULE_COUNT as f64;
        for i in 0..MODULE_COUNT {
            // Bias = target - actual load (positive for underused, negative for overused)
            let bias = (target - load[i]) * 0.1;
            gate_logits[i] += bias;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancer_default() {
        let lb = LoadBalancer::new();
        let loss = lb.compute_loss();
        assert!((loss - 1.0 / MODULE_COUNT as f64).abs() < 0.01);
    }

    #[test]
    fn test_record_step() {
        let mut lb = LoadBalancer::new();
        let probs = [0.1; MODULE_COUNT];
        lb.record_step(&probs, &[0]);
        assert_eq!(lb.selection_counts[0], 1.0);
        assert_eq!(lb.total_selections, 1.0);
    }

    #[test]
    fn test_load_entropy_balanced() {
        let mut lb = LoadBalancer::new();
        let probs = [1.0 / MODULE_COUNT as f64; MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            lb.record_step(&probs, &[i]);
        }
        let entropy = lb.load_entropy();
        assert!(entropy > 1.0); // Should be high for balanced
    }

    #[test]
    fn test_apply_biases() {
        let mut lb = LoadBalancer::new();
        // Simulate all selections to expert 0
        let probs = [0.1; MODULE_COUNT];
        for _ in 0..100 {
            lb.record_step(&probs, &[0]);
        }
        let mut logits = [0.0; MODULE_COUNT];
        lb.apply_biases(&mut logits);
        // Expert 0 should have negative bias (overused)
        assert!(logits[0] < 0.0);
        // Other experts should have positive bias (underused)
        assert!(logits[1] > 0.0);
    }
}
