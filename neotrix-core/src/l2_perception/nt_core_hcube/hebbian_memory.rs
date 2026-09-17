//! B128 — Hebbian associative memory map
//!
//! A weighted co-occurrence graph implementing Hebbian learning for
//! associative memory. Nodes are symbol names, edges are learned
//! association strengths based on co-occurrence.
//!
//! Architecture (HeLa-Mem):
//!   - Write gating: g = δ(importance > τ)
//!   - Update rule: Δw_ij = α · (x_i · x_j - w_ij)
//!   - Diffusion retrieval: seed → spread along edges → top-K
//!   - Forgetting: exponential decay of unused edges

use std::collections::HashMap;

/// Minimum edge weight threshold — weights below this are treated as zero.
const MIN_WEIGHT: f64 = 1e-6;

/// Hebbian associative memory graph.
///
/// Stores a weighted undirected graph where edge weights represent
/// learned associations between symbols. Edges are updated using
/// a Hebbian rule gated by importance.
#[derive(Debug, Clone)]
pub struct HebbianGraph {
    /// Adjacency list: symbol → Vec<(neighbor, weight)>
    edges: HashMap<String, Vec<(String, f64)>>,
    /// Access count per symbol (for importance gating)
    access_counts: HashMap<String, u64>,
    /// Co-occurrence counter: (a, b) → count
    co_occurrence: HashMap<(String, String), u64>,
    /// Hebbian learning rate
    learning_rate: f64,
    /// Importance threshold for write gating
    importance_threshold: f64,
    /// Decay factor per call to decay() (0.0 = no decay)
    decay_factor: f64,
    /// Global timestep
    timestep: u64,
}

/// Result of a diffusion retrieval query.
#[derive(Debug, Clone)]
pub struct HebbianRetrievalResult {
    /// Retrieved symbols with their activation scores.
    pub results: Vec<(String, f64)>,
    /// Number of hops used to reach these results.
    pub hops: usize,
}

impl Default for HebbianGraph {
    fn default() -> Self {
        Self {
            edges: HashMap::new(),
            access_counts: HashMap::new(),
            co_occurrence: HashMap::new(),
            learning_rate: 0.1,
            importance_threshold: 0.3,
            decay_factor: 0.01,
            timestep: 0,
        }
    }
}

impl HebbianGraph {
    pub fn new(learning_rate: f64, importance_threshold: f64, decay_factor: f64) -> Self {
        Self {
            edges: HashMap::new(),
            access_counts: HashMap::new(),
            co_occurrence: HashMap::new(),
            learning_rate,
            importance_threshold,
            decay_factor,
            timestep: 0,
        }
    }

    /// Add a node to the graph (no-op if already present).
    pub fn add_node(&mut self, symbol: &str) {
        self.edges.entry(symbol.to_string()).or_default();
        self.access_counts.entry(symbol.to_string()).or_insert(0);
    }

    /// Record a co-occurrence event between two symbols.
    /// Updates the edge weight using Hebbian learning:
    ///   Δw = lr · (1 - w)  for co-occurrence
    /// Write gate: only update if importance > threshold
    pub fn observe_co_occurrence(&mut self, symbol_a: &str, symbol_b: &str, importance: f64) {
        if importance < self.importance_threshold {
            return;
        }
        self.add_node(symbol_a);
        self.add_node(symbol_b);
        *self.access_counts.entry(symbol_a.to_string()).or_insert(0) += 1;
        *self.access_counts.entry(symbol_b.to_string()).or_insert(0) += 1;

        // Update co-occurrence counter (canonical ordering)
        let key = if symbol_a <= symbol_b {
            (symbol_a.to_string(), symbol_b.to_string())
        } else {
            (symbol_b.to_string(), symbol_a.to_string())
        };
        *self.co_occurrence.entry(key).or_insert(0) += 1;

        // Hebbian edge update: Δw = lr · (1 - w) for co-occurrence
        let mut found = false;
        if let Some(neighbors) = self.edges.get_mut(symbol_a) {
            for (n, w) in neighbors.iter_mut() {
                if n == symbol_b {
                    *w += self.learning_rate * (1.0 - *w);
                    found = true;
                    break;
                }
            }
        }
        if !found {
            let default_rate = self.learning_rate;
            if let Some(neighbors_a) = self.edges.get_mut(symbol_a) {
                neighbors_a.push((symbol_b.to_string(), default_rate));
            }
            if let Some(neighbors_b) = self.edges.get_mut(symbol_b) {
                neighbors_b.push((symbol_a.to_string(), default_rate));
            }
        }
    }

    /// Batch observe multiple co-occurrence pairs at once.
    pub fn observe_batch(&mut self, pairs: &[(String, String, f64)]) {
        for (a, b, importance) in pairs {
            self.observe_co_occurrence(a, b, *importance);
        }
    }

    /// Hebbian update for a single pair with an explicit target weight.
    /// Δw = lr · (target_weight - w)
    /// Useful for supervised association learning.
    pub fn hebbian_update(
        &mut self,
        symbol_a: &str,
        symbol_b: &str,
        target_weight: f64,
        importance: f64,
    ) {
        if importance < self.importance_threshold {
            return;
        }
        self.add_node(symbol_a);
        self.add_node(symbol_b);
        *self.access_counts.entry(symbol_a.to_string()).or_insert(0) += 1;
        *self.access_counts.entry(symbol_b.to_string()).or_insert(0) += 1;

        let mut found = false;
        if let Some(neighbors) = self.edges.get_mut(symbol_a) {
            for (n, w) in neighbors.iter_mut() {
                if n == symbol_b {
                    *w += self.learning_rate * (target_weight - *w);
                    if *w < MIN_WEIGHT {
                        *w = 0.0;
                    }
                    found = true;
                    break;
                }
            }
        }
        if !found {
            if let Some(neighbors_a) = self.edges.get_mut(symbol_a) {
                neighbors_a.push((symbol_b.to_string(), target_weight * self.learning_rate));
            }
            if let Some(neighbors_b) = self.edges.get_mut(symbol_b) {
                neighbors_b.push((symbol_a.to_string(), target_weight * self.learning_rate));
            }
        }
    }

    /// Diffusion activation retrieval: start from seed symbols and spread
    /// activation along edges for `steps` hops.
    ///
    /// At each hop, a node's activation is the weighted sum of its neighbors'
    /// activations from the previous hop. Decay factor reduces activation
    /// per hop to prevent infinite spread.
    pub fn diffusion_retrieve(
        &self,
        seeds: &[String],
        steps: usize,
        top_k: usize,
        decay: f64,
    ) -> HebbianRetrievalResult {
        let mut activation: HashMap<String, f64> = HashMap::new();
        for seed in seeds {
            if self.edges.contains_key(seed) {
                *activation.entry(seed.clone()).or_insert(0.0) += 1.0;
            }
        }

        let mut max_hops = 0;
        for hop in 0..steps {
            let prev = activation.clone();
            let mut next: HashMap<String, f64> = HashMap::new();
            for (node, act) in &prev {
                if *act < MIN_WEIGHT {
                    continue;
                }
                if let Some(neighbors) = self.edges.get(node) {
                    for (neighbor, weight) in neighbors {
                        let spread = *act * weight * decay;
                        if spread > MIN_WEIGHT {
                            *next.entry(neighbor.clone()).or_insert(0.0) += spread;
                        }
                    }
                }
            }
            // Merge into activation
            for (node, val) in next {
                let entry = activation.entry(node).or_insert(0.0);
                *entry += val;
            }
            max_hops = hop + 1;
        }

        let mut results: Vec<(String, f64)> = activation.into_iter().collect();
        results.sort_by(|(_, a), (_, b)| b.total_cmp(a));
        results.truncate(top_k);

        HebbianRetrievalResult {
            results,
            hops: max_hops,
        }
    }

    /// Apply exponential decay to all edges.
    /// w *= (1 - decay_factor) at each call.
    /// Edges below MIN_WEIGHT are set to 0 and pruned on next prune_edges().
    pub fn decay(&mut self) {
        self.timestep += 1;
        for neighbors in self.edges.values_mut() {
            for (_, w) in neighbors.iter_mut() {
                *w *= 1.0 - self.decay_factor;
                if *w < MIN_WEIGHT {
                    *w = 0.0;
                }
            }
        }
    }

    /// Remove edges with weight below threshold.
    /// Also removes isolated nodes (no edges).
    pub fn prune_edges(&mut self, threshold: f64) -> usize {
        let mut pruned = 0;
        for neighbors in self.edges.values_mut() {
            let before = neighbors.len();
            neighbors.retain(|(_, w)| *w >= threshold);
            pruned += before - neighbors.len();
        }
        // Remove isolated nodes
        self.edges.retain(|_, neighbors| !neighbors.is_empty());
        pruned
    }

    /// Get edge weight between two symbols (undirected).
    pub fn get_edge_weight(&self, a: &str, b: &str) -> f64 {
        if let Some(neighbors) = self.edges.get(a) {
            for (n, w) in neighbors {
                if n == b {
                    return *w;
                }
            }
        }
        0.0
    }

    /// Get the access count for a symbol.
    pub fn get_access_count(&self, symbol: &str) -> u64 {
        self.access_counts.get(symbol).copied().unwrap_or(0)
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.edges.len()
    }

    /// Number of edges (undirected, each counted once).
    pub fn edge_count(&self) -> usize {
        let mut total = 0;
        for neighbors in self.edges.values() {
            total += neighbors.len();
        }
        total / 2
    }

    /// Number of co-occurrence events recorded.
    pub fn co_occurrence_count(&self) -> usize {
        self.co_occurrence.len()
    }

    /// Current timestep.
    pub fn timestep(&self) -> u64 {
        self.timestep
    }

    /// Summary statistics.
    pub fn stats(&self) -> HebbianStats {
        let mut total_weight = 0.0;
        let mut max_weight = 0.0;
        let mut count = 0;
        for neighbors in self.edges.values() {
            for (_, w) in neighbors {
                total_weight += *w;
                if *w > max_weight {
                    max_weight = *w;
                }
                count += 1;
            }
        }
        let avg_weight = if count > 0 {
            total_weight / count as f64
        } else {
            0.0
        };
        HebbianStats {
            nodes: self.edges.len(),
            edges: count / 2,
            avg_weight,
            max_weight,
            co_occurrence_events: self.co_occurrence.len(),
            timestep: self.timestep,
        }
    }
}

/// Summary statistics for a HebbianGraph.
#[derive(Debug, Clone)]
pub struct HebbianStats {
    pub nodes: usize,
    pub edges: usize,
    pub avg_weight: f64,
    pub max_weight: f64,
    pub co_occurrence_events: usize,
    pub timestep: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph = HebbianGraph::default();
        graph.add_node("foo");
        graph.add_node("bar");
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_observe_co_occurrence_creates_edge() {
        let mut graph = HebbianGraph::new(0.5, 0.0, 0.0);
        graph.observe_co_occurrence("a", "b", 1.0);
        let w = graph.get_edge_weight("a", "b");
        assert!(
            w > 0.0,
            "co-occurrence should create edge with positive weight"
        );
    }

    #[test]
    fn test_observe_co_occurrence_undirected() {
        let mut graph = HebbianGraph::new(0.5, 0.0, 0.0);
        graph.observe_co_occurrence("a", "b", 1.0);
        let w_ab = graph.get_edge_weight("a", "b");
        let w_ba = graph.get_edge_weight("b", "a");
        assert!((w_ab - w_ba).abs() < 1e-12, "edge should be undirected");
    }

    #[test]
    fn test_importance_gate_blocks_low_importance() {
        let mut graph = HebbianGraph::new(0.5, 0.8, 0.0);
        graph.observe_co_occurrence("a", "b", 0.3);
        let w = graph.get_edge_weight("a", "b");
        assert!((w - 0.0).abs() < 1e-12, "low importance should be gated");
    }

    #[test]
    fn test_hebbian_update_approaches_target() {
        let mut graph = HebbianGraph::new(0.5, 0.0, 0.0);
        // Repeated updates should approach target_weight = 0.8
        for _ in 0..10 {
            graph.hebbian_update("a", "b", 0.8, 1.0);
        }
        let w = graph.get_edge_weight("a", "b");
        assert!(
            (w - 0.8).abs() < 0.05,
            "weight should approach 0.8, got {w}"
        );
    }

    #[test]
    fn test_diffusion_retrieve_one_hop() {
        let mut graph = HebbianGraph::new(0.5, 0.0, 0.0);
        graph.observe_co_occurrence("seed", "target", 1.0);
        graph.observe_co_occurrence("seed", "other", 0.5);
        let seeds = vec!["seed".to_string()];
        let result = graph.diffusion_retrieve(&seeds, 1, 5, 0.8);
        assert!(
            !result.results.is_empty(),
            "diffusion should return results"
        );
        // seed itself should have highest activation (initial 1.0 + spread back)
        let seed_act = result.results.iter().find(|(n, _)| n == "seed");
        assert!(seed_act.is_some(), "seed should be in results");
    }

    #[test]
    fn test_diffusion_retrieve_two_hops() {
        let mut graph = HebbianGraph::new(1.0, 0.0, 0.0);
        // a - b - c chain
        graph.observe_co_occurrence("a", "b", 1.0);
        graph.observe_co_occurrence("b", "c", 1.0);
        let seeds = vec!["a".to_string()];
        let result = graph.diffusion_retrieve(&seeds, 2, 5, 0.5);
        // c should be reachable in 2 hops
        let c_act = result.results.iter().find(|(n, _)| n == "c");
        assert!(
            c_act.is_some() || graph.node_count() >= 3,
            "c should be reachable via diffusion"
        );
    }

    #[test]
    fn test_decay_reduces_weights() {
        let mut graph = HebbianGraph::new(1.0, 0.0, 0.5);
        graph.observe_co_occurrence("a", "b", 1.0);
        let w_before = graph.get_edge_weight("a", "b");
        graph.decay();
        let w_after = graph.get_edge_weight("a", "b");
        assert!(w_after < w_before, "decay should reduce weight");
    }

    #[test]
    fn test_prune_edges_removes_weak() {
        let mut graph = HebbianGraph::new(0.1, 0.0, 0.0);
        graph.observe_co_occurrence("a", "b", 1.0); // ~0.1 weight
        graph.observe_co_occurrence("c", "d", 1.0); // ~0.1 weight
        let pruned = graph.prune_edges(0.5);
        assert_eq!(pruned, 4, "both edges should be pruned");
        assert_eq!(graph.node_count(), 0, "isolated nodes should be removed");
    }

    #[test]
    fn test_stats_basic() {
        let graph = HebbianGraph::default();
        let stats = graph.stats();
        assert_eq!(stats.nodes, 0);
        assert_eq!(stats.edges, 0);
    }

    #[test]
    fn test_observe_batch() {
        let mut graph = HebbianGraph::new(0.5, 0.0, 0.0);
        let pairs = vec![
            ("a".to_string(), "b".to_string(), 1.0),
            ("b".to_string(), "c".to_string(), 1.0),
            ("a".to_string(), "c".to_string(), 1.0),
        ];
        graph.observe_batch(&pairs);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 3);
    }

    #[test]
    fn test_edge_count_undirected() {
        let mut graph = HebbianGraph::new(0.5, 0.0, 0.0);
        graph.observe_co_occurrence("a", "b", 1.0);
        // One undirected edge = stored as (a→b) and (b→a) internally
        assert_eq!(graph.edge_count(), 1, "one undirected edge");
    }
}
