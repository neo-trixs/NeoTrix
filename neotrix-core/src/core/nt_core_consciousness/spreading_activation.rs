// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

type NodeId = usize;

/// Spreading activation engine — propagates activation through a graph
#[derive(Debug, Clone)]
pub struct SpreadingActivation {
    pub nodes: HashMap<NodeId, f64>,
    pub edges: Vec<(NodeId, NodeId, f64)>,
    pub decay: f64,
    pub threshold: f64,
    pub max_iterations: usize,
}

impl SpreadingActivation {
    pub fn new() -> Self {
        SpreadingActivation {
            nodes: HashMap::new(),
            edges: Vec::new(),
            decay: 0.1,
            threshold: 0.01,
            max_iterations: 10,
        }
    }

    pub fn add_node(&mut self, id: NodeId, initial_activation: f64) {
        self.nodes.insert(id, initial_activation);
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, weight: f64) {
        self.edges.push((from, to, weight));
    }

    pub fn spread(&mut self, seed: NodeId, amount: f64) {
        self.nodes
            .entry(seed)
            .and_modify(|v| *v += amount)
            .or_insert(amount);

        for _ in 0..self.max_iterations {
            let mut delta = HashMap::new();
            for &(from, to, weight) in &self.edges {
                let from_val = *self.nodes.get(&from).unwrap_or(&0.0);
                if from_val > self.threshold {
                    *delta.entry(to).or_insert(0.0) += from_val * weight;
                }
            }
            let mut changed = false;
            for (id, d) in &delta {
                let entry = self.nodes.entry(*id).or_insert(0.0);
                let old = *entry;
                *entry += d;
                *entry -= self.decay * (*entry);
                if (*entry - old).abs() > 0.001 {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn activation(&self, id: NodeId) -> f64 {
        *self.nodes.get(&id).unwrap_or(&0.0)
    }

    pub fn most_active(&self, n: usize) -> Vec<(NodeId, f64)> {
        let mut pairs: Vec<(NodeId, f64)> = self.nodes.iter().map(|(&k, &v)| (k, v)).collect();
        pairs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(n).collect()
    }

    pub fn normalize(&mut self) {
        let max = self.nodes.values().cloned().fold(0.0_f64, f64::max);
        if max > 0.0 {
            for v in self.nodes.values_mut() {
                *v /= max;
            }
        }
    }

    pub fn reset(&mut self) {
        for v in self.nodes.values_mut() {
            *v = 0.0;
        }
    }

    pub fn hebbian_writeback(&mut self) {
        for (from, to, weight) in &mut self.edges {
            let f_act = *self.nodes.get(from).unwrap_or(&0.0);
            let t_act = *self.nodes.get(to).unwrap_or(&0.0);
            let hebb = f_act * t_act;
            *weight = (*weight + hebb * 0.01).clamp(0.0, 1.0);
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_spread() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 0.0);
        sa.add_node(1, 0.0);
        sa.add_edge(0, 1, 0.8);
        sa.spread(0, 1.0);
        assert!(sa.activation(1) > 0.0);
    }

    #[test]
    fn test_most_active_ordering() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 0.1);
        sa.add_node(1, 0.5);
        sa.add_node(2, 0.3);
        let top = sa.most_active(2);
        assert_eq!(top[0].0, 1);
        assert_eq!(top[1].0, 2);
    }

    #[test]
    fn test_normalize() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 2.0);
        sa.add_node(1, 4.0);
        sa.normalize();
        assert!((sa.activation(1) - 1.0).abs() < 0.01);
        assert!((sa.activation(0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 0.8);
        sa.reset();
        assert!((sa.activation(0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_hebbian_writeback() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 1.0);
        sa.add_node(1, 1.0);
        sa.add_edge(0, 1, 0.5);
        sa.hebbian_writeback();
        assert!(sa.edges[0].2 >= 0.5);
    }

    #[test]
    fn test_spread_through_chain() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 0.0);
        sa.add_node(1, 0.0);
        sa.add_node(2, 0.0);
        sa.add_edge(0, 1, 0.5);
        sa.add_edge(1, 2, 0.5);
        sa.spread(0, 1.0);
        assert!(
            sa.activation(2) > 0.0,
            "activation should propagate through chain"
        );
    }

    #[test]
    fn test_isolated_node_no_spread() {
        let mut sa = SpreadingActivation::new();
        sa.add_node(0, 0.5);
        sa.add_node(1, 0.0);
        sa.spread(0, 0.0);
        assert!((sa.activation(1) - 0.0).abs() < 0.01);
    }
}

/// Activation function enum for non-linear transformations
#[derive(Debug, Clone, PartialEq)]
pub enum ActivationFn {
    Sigmoid,
    Relu,
    Tanh,
    Linear,
}

/// Fine-tuned activation parameters with per-iteration decay schedule
pub struct ActivationConfig {
    pub decay_schedule: Vec<f64>,
    pub activation_function: ActivationFn,
    pub global_inhibition: f64,
    pub temperature: f64,
}

impl ActivationConfig {
    pub fn default() -> Self {
        ActivationConfig {
            decay_schedule: vec![0.1; 10],
            activation_function: ActivationFn::Sigmoid,
            global_inhibition: 0.0,
            temperature: 1.0,
        }
    }

    pub fn with_decay_schedule(schedule: Vec<f64>) -> Self {
        ActivationConfig {
            decay_schedule: schedule,
            ..Self::default()
        }
    }
}

fn apply_activation_fn(x: f64, f: &ActivationFn) -> f64 {
    match f {
        ActivationFn::Sigmoid => 1.0 / (1.0 + (-x).exp()),
        ActivationFn::Relu => x.max(0.0),
        ActivationFn::Tanh => x.tanh(),
        ActivationFn::Linear => x,
    }
}

/// VSA-native spreading activation with node vectors
#[derive(Clone)]
pub struct VsaSpreadingActivation {
    pub vsa_nodes: Vec<Vec<f64>>,
    pub vsa_edges: Vec<(usize, usize, f64)>,
    pub vsa_activation: Vec<f64>,
}

impl VsaSpreadingActivation {
    pub fn new(node_count: usize, dim: usize) -> Self {
        let vsa_nodes = (0..node_count).map(|_| vec![0.0; dim]).collect();
        let vsa_activation = vec![0.0; node_count];
        VsaSpreadingActivation {
            vsa_nodes,
            vsa_edges: Vec::new(),
            vsa_activation,
        }
    }

    pub fn add_vsa_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.vsa_edges.push((from, to, weight));
    }

    pub fn spread_vsa(&mut self, seed: usize, amount: f64, config: &ActivationConfig) {
        if seed >= self.vsa_activation.len() {
            return;
        }
        self.vsa_activation[seed] += amount;

        let iterations = config.decay_schedule.len();
        for i in 0..iterations {
            let mut delta = vec![0.0; self.vsa_activation.len()];
            for &(from, to, weight) in &self.vsa_edges {
                if from < self.vsa_activation.len() && to < self.vsa_activation.len() {
                    let from_val = self.vsa_activation[from];
                    if from_val > 0.01 {
                        delta[to] += from_val * weight;
                    }
                }
            }
            let decay = if i < config.decay_schedule.len() {
                config.decay_schedule[i]
            } else {
                0.1
            };
            for j in 0..self.vsa_activation.len() {
                self.vsa_activation[j] += delta[j];
                self.vsa_activation[j] -= decay * self.vsa_activation[j];
                self.vsa_activation[j] =
                    apply_activation_fn(self.vsa_activation[j], &config.activation_function);
            }
            if config.global_inhibition > 0.0 {
                let total: f64 = self.vsa_activation.iter().sum();
                let avg = total / self.vsa_activation.len() as f64;
                for v in self.vsa_activation.iter_mut() {
                    *v -= config.global_inhibition * avg;
                    if *v < 0.0 {
                        *v = 0.0;
                    }
                }
            }
        }
    }

    pub fn vsa_most_active(&self, n: usize) -> Vec<(usize, f64)> {
        let mut pairs: Vec<(usize, f64)> = self
            .vsa_activation
            .iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
        pairs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(n).collect()
    }

    pub fn vsa_hebbian_update(&mut self, rate: f64) {
        for (from, to, weight) in &mut self.vsa_edges {
            if *from < self.vsa_activation.len() && *to < self.vsa_activation.len() {
                let hebb = self.vsa_activation[*from] * self.vsa_activation[*to];
                *weight = (*weight + hebb * rate).clamp(0.0, 1.0);
            }
        }
    }

    pub fn vsa_inhibition(&mut self, factor: f64) {
        if self.vsa_activation.is_empty() {
            return;
        }
        let total: f64 = self.vsa_activation.iter().sum();
        let avg = total / self.vsa_activation.len() as f64;
        for v in self.vsa_activation.iter_mut() {
            *v -= factor * avg;
            if *v < 0.0 {
                *v = 0.0;
            }
        }
    }

    pub fn vsa_node_count(&self) -> usize {
        self.vsa_nodes.len()
    }
}

/// Winner-take-all competition layer with softmax and entropy
pub struct CompetitionLayer {
    pub nodes: Vec<f64>,
    pub k: usize,
}

impl CompetitionLayer {
    pub fn new(k: usize) -> Self {
        CompetitionLayer {
            nodes: Vec::new(),
            k,
        }
    }

    pub fn load(&mut self, activations: &[f64]) {
        self.nodes = activations.to_vec();
    }

    pub fn winners(&self) -> Vec<(usize, f64)> {
        let mut pairs: Vec<(usize, f64)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
        pairs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pairs.into_iter().take(self.k).collect()
    }

    pub fn softmax(&self) -> Vec<f64> {
        if self.nodes.is_empty() {
            return vec![];
        }
        let max = self.nodes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = self.nodes.iter().map(|&v| (v - max).exp()).collect();
        let sum: f64 = exps.iter().sum();
        if sum == 0.0 {
            return vec![0.0; self.nodes.len()];
        }
        exps.iter().map(|e| e / sum).collect()
    }

    pub fn entropy(&self) -> f64 {
        let p = self.softmax();
        if p.is_empty() {
            return 0.0;
        }
        -p.iter()
            .filter(|&&v| v > 0.0)
            .map(|&v| v * v.ln())
            .sum::<f64>()
    }
}

#[cfg(test)]
mod spreading_activation_tests {
    use super::*;

    #[test]
    fn test_activation_config_defaults() {
        let cfg = ActivationConfig::default();
        assert_eq!(cfg.decay_schedule.len(), 10);
        assert!(cfg.decay_schedule.iter().all(|&d| (d - 0.1).abs() < 1e-10));
        assert_eq!(cfg.activation_function, ActivationFn::Sigmoid);
        assert_eq!(cfg.global_inhibition, 0.0);
        assert_eq!(cfg.temperature, 1.0);
    }

    #[test]
    fn test_vsa_network_creation() {
        let vsa = VsaSpreadingActivation::new(5, 8);
        assert_eq!(vsa.vsa_node_count(), 5);
        assert_eq!(vsa.vsa_nodes[0].len(), 8);
        assert!(vsa.vsa_edges.is_empty());
        assert!(vsa.vsa_activation.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_vsa_spread_propagation() {
        let cfg = ActivationConfig::default();
        let mut vsa = VsaSpreadingActivation::new(3, 4);
        vsa.add_vsa_edge(0, 1, 0.8);
        vsa.add_vsa_edge(1, 2, 0.6);
        vsa.spread_vsa(0, 1.0, &cfg);
        assert!(vsa.vsa_activation[0] > 0.0);
        assert!(vsa.vsa_activation[1] > 0.0);
        assert!(vsa.vsa_activation[2] > 0.0);
    }

    #[test]
    fn test_vsa_hebbian_increases_weight() {
        let cfg = ActivationConfig::default();
        let mut vsa = VsaSpreadingActivation::new(2, 4);
        vsa.add_vsa_edge(0, 1, 0.3);
        vsa.spread_vsa(0, 1.0, &cfg);
        let before = vsa.vsa_edges[0].2;
        vsa.vsa_hebbian_update(0.1);
        assert!(
            vsa.vsa_edges[0].2 >= before,
            "hebbian should not decrease weight"
        );
    }

    #[test]
    fn test_vsa_inhibition_reduces_all() {
        let cfg = ActivationConfig::default();
        let mut vsa = VsaSpreadingActivation::new(3, 4);
        vsa.add_vsa_edge(0, 1, 0.5);
        vsa.add_vsa_edge(1, 2, 0.5);
        vsa.spread_vsa(0, 1.0, &cfg);
        let before: Vec<f64> = vsa.vsa_activation.clone();
        vsa.vsa_inhibition(0.5);
        for (i, &b) in before.iter().enumerate() {
            assert!(
                vsa.vsa_activation[i] <= b,
                "node {} activation should not increase after inhibition",
                i
            );
        }
    }

    #[test]
    fn test_competition_winners_count() {
        let mut cl = CompetitionLayer::new(3);
        cl.load(&[0.1, 0.5, 0.3, 0.7, 0.2]);
        let w = cl.winners();
        assert_eq!(w.len(), 3);
        assert_eq!(w[0].0, 3);
        assert_eq!(w[1].0, 1);
        assert_eq!(w[2].0, 2);
    }

    #[test]
    fn test_competition_softmax_sum() {
        let mut cl = CompetitionLayer::new(2);
        cl.load(&[1.0, 2.0, 3.0]);
        let sm = cl.softmax();
        let sum: f64 = sm.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
        assert!(sm[2] > sm[1]);
        assert!(sm[1] > sm[0]);
    }

    #[test]
    fn test_competition_entropy_uniform() {
        let mut cl = CompetitionLayer::new(3);
        cl.load(&[1.0, 1.0, 1.0, 1.0]);
        let h = cl.entropy();
        let expected = (4.0_f64).ln();
        assert!((h - expected).abs() < 1e-10);
    }

    #[test]
    fn test_decay_schedule_vs_constant() {
        let default = ActivationConfig::default();
        let custom = ActivationConfig::with_decay_schedule(vec![0.5; 10]);

        let mut vsa1 = VsaSpreadingActivation::new(2, 4);
        vsa1.add_vsa_edge(0, 1, 0.8);
        vsa1.spread_vsa(0, 1.0, &default);

        let mut vsa2 = VsaSpreadingActivation::new(2, 4);
        vsa2.add_vsa_edge(0, 1, 0.8);
        vsa2.spread_vsa(0, 1.0, &custom);

        assert!(
            (vsa1.vsa_activation[1] - vsa2.vsa_activation[1]).abs() > 0.001,
            "different decay schedules should produce different activations"
        );
    }

    #[test]
    fn test_vsa_most_active_ordering() {
        let mut vsa = VsaSpreadingActivation::new(4, 4);
        vsa.vsa_activation = vec![0.1, 0.7, 0.3, 0.5];
        let top = vsa.vsa_most_active(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, 1);
        assert_eq!(top[1].0, 3);
    }
}
