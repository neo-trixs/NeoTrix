use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Episodic,
    Semantic,
}

impl NodeKind {
    pub fn name(&self) -> &'static str {
        match self {
            NodeKind::Episodic => "episodic",
            NodeKind::Semantic => "semantic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Temporal,
    Causal,
    Associative,
}

impl EdgeKind {
    pub fn name(&self) -> &'static str {
        match self {
            EdgeKind::Temporal => "temporal",
            EdgeKind::Causal => "causal",
            EdgeKind::Associative => "associative",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryNode {
    pub id: u64,
    pub kind: NodeKind,
    pub vsa_vector: Vec<u8>,
    pub activation: f64,
    pub label: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryEdge {
    pub source: u64,
    pub target: u64,
    pub kind: EdgeKind,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryGraph {
    nodes: Vec<MemoryNode>,
    edges: Vec<MemoryEdge>,
    max_nodes: usize,
    next_id: u64,
    /// SRMU-style relevance gate: tracks per-node temporal decay
    pub node_relevance: HashMap<u64, (f64, Instant)>,
    pub relevance_decay: f64,
}

impl MemoryGraph {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(max_nodes.min(1000)),
            edges: Vec::new(),
            max_nodes: max_nodes.max(1),
            next_id: 1,
            node_relevance: HashMap::new(),
            relevance_decay: 0.95,
        }
    }

    pub fn add_node(&mut self, kind: NodeKind, vsa: Vec<u8>, label: &str) -> u64 {
        if self.nodes.len() >= self.max_nodes {
            let oldest_idx = self
                .nodes
                .iter()
                .enumerate()
                .min_by_key(|(_, n)| n.timestamp)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let removed_id = self.nodes[oldest_idx].id;
            self.nodes.swap_remove(oldest_idx);
            self.edges
                .retain(|e| e.source != removed_id && e.target != removed_id);
        }
        let id = self.next_id;
        self.next_id += 1;
        let now = now_nanos();
        self.nodes.push(MemoryNode {
            id,
            kind,
            vsa_vector: vsa,
            activation: 0.0,
            label: label.to_string(),
            timestamp: now,
        });
        id
    }

    pub fn add_edge(&mut self, source: u64, target: u64, kind: EdgeKind, weight: f64) {
        self.edges.push(MemoryEdge {
            source,
            target,
            kind,
            weight: weight.clamp(0.0, 1.0),
        });
    }

    /// SRMU relevance-gated insert: only stores if relevance >= 0.3 threshold.
    /// Relevance = decay * last_relevance + (1-decay) * base_relevance.
    pub fn relevance_gated_insert(
        &mut self,
        kind: NodeKind,
        vsa: Vec<u8>,
        label: &str,
        base_relevance: f64,
    ) -> Option<u64> {
        let now = Instant::now();
        let id_val = self.next_id;
        let updated_relevance =
            if let Some((prev_rel, prev_time)) = self.node_relevance.get(&id_val) {
                let elapsed = now.duration_since(*prev_time).as_secs_f64();
                let decay_factor = self.relevance_decay.powf(elapsed.max(0.1));
                decay_factor * prev_rel + (1.0 - decay_factor) * base_relevance
            } else {
                base_relevance
            };
        self.node_relevance.insert(id_val, (updated_relevance, now));

        if updated_relevance >= 0.3 {
            Some(self.add_node(kind, vsa, label))
        } else {
            None
        }
    }

    pub fn node_index(&self, id: u64) -> Option<usize> {
        self.nodes.iter().position(|n| n.id == id)
    }

    pub fn all_labels(&self) -> Vec<(u64, String)> {
        self.nodes.iter().map(|n| (n.id, n.label.clone())).collect()
    }

    pub fn activate_from_vsa(&mut self, query: &[u8], top_k: usize) -> Vec<(u64, String, f64)> {
        let mut scores: Vec<(usize, f64)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (i, QuantizedVSA::similarity(&n.vsa_vector, query)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let k = top_k.min(scores.len());
        let mut results = Vec::with_capacity(k);
        for &(idx, sim) in scores.iter().take(k) {
            self.nodes[idx].activation = sim.clamp(0.0, 1.0);
            results.push((self.nodes[idx].id, self.nodes[idx].label.clone(), sim));
        }
        results
    }

    pub fn spread(&mut self, hops: usize, decay: f64, inhibition: f64) -> Vec<(u64, String, f64)> {
        if self.nodes.is_empty() || self.edges.is_empty() {
            return self.top_activated(self.nodes.len());
        }

        let mut current: Vec<f64> = self.nodes.iter().map(|n| n.activation).collect();
        let n = self.nodes.len();

        for _hop in 0..hops {
            let mut incoming = vec![0.0; n];
            let mut contributor_count = vec![0usize; n];

            for edge in &self.edges {
                let src_idx = self.node_index(edge.source);
                let tgt_idx = self.node_index(edge.target);
                if let (Some(si), Some(ti)) = (src_idx, tgt_idx) {
                    if current[si] > 0.0 {
                        let contribution = current[si] * edge.weight * decay;
                        incoming[ti] += contribution;
                        contributor_count[ti] += 1;
                    }
                }
            }

            for i in 0..n {
                if incoming[i] > 0.0 && contributor_count[i] > 0 {
                    let inh_factor = 1.0 / (1.0 + inhibition * (contributor_count[i] as f64 - 1.0));
                    current[i] = (current[i] + incoming[i] * inh_factor).clamp(0.0, 1.0);
                }
            }
        }

        for (i, &act) in current.iter().enumerate() {
            self.nodes[i].activation = act;
        }

        self.top_activated(self.nodes.len())
    }

    pub fn retrieve(
        &mut self,
        query: &[u8],
        top_k: usize,
        max_hops: Option<usize>,
    ) -> Vec<(u64, String, f64)> {
        let hops = max_hops.unwrap_or(3);
        self.retrieve_with_hops(query, top_k, hops)
    }

    pub fn retrieve_with_hops(
        &mut self,
        query: &[u8],
        top_k: usize,
        max_hops: usize,
    ) -> Vec<(u64, String, f64)> {
        self.reset_activation();
        self.activate_from_vsa(query, top_k);
        self.spread(max_hops, 0.85, 0.3);
        let mut results: Vec<(usize, f64)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (i, n.activation))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let k = top_k.min(results.len());
        results
            .into_iter()
            .take(k)
            .map(|(idx, _)| {
                (
                    self.nodes[idx].id,
                    self.nodes[idx].label.clone(),
                    self.nodes[idx].activation,
                )
            })
            .collect()
    }

    pub fn retrieve_with_scorer<F>(
        &mut self,
        query: &[u8],
        top_k: usize,
        max_hops: usize,
        scorer: &F,
    ) -> Vec<(u64, String, f64)>
    where
        F: Fn(u64, f64) -> f64,
    {
        self.reset_activation();
        self.activate_from_vsa(query, top_k * 2);
        self.spread(max_hops, 0.85, 0.3);

        let mut results: Vec<(usize, f64)> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let score = n.activation;
                let adjusted = scorer(n.id, score);
                (i, adjusted)
            })
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let k = top_k.min(results.len());
        results
            .into_iter()
            .take(k)
            .map(|(idx, _)| {
                (
                    self.nodes[idx].id,
                    self.nodes[idx].label.clone(),
                    self.nodes[idx].activation,
                )
            })
            .collect()
    }

    pub fn decay_all(&mut self, rate: f64) {
        let factor = (1.0 - rate.clamp(0.0, 1.0)).max(0.0);
        for node in &mut self.nodes {
            node.activation *= factor;
            if node.activation < 1e-10 {
                node.activation = 0.0;
            }
        }
    }

    pub fn reset_activation(&mut self) {
        for node in &mut self.nodes {
            node.activation = 0.0;
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn top_activated(&self, count: usize) -> Vec<(u64, String, f64)> {
        let mut indices: Vec<usize> = (0..self.nodes.len()).collect();
        indices.sort_by(|&a, &b| {
            let cmp = self.nodes[b]
                .activation
                .partial_cmp(&self.nodes[a].activation)
                .unwrap_or(std::cmp::Ordering::Equal);
            if cmp != std::cmp::Ordering::Equal {
                cmp
            } else {
                self.nodes[a].id.cmp(&self.nodes[b].id)
            }
        });
        let k = count.min(indices.len());
        indices
            .into_iter()
            .take(k)
            .map(|i| {
                (
                    self.nodes[i].id,
                    self.nodes[i].label.clone(),
                    self.nodes[i].activation,
                )
            })
            .collect()
    }
}

/// Creates a scorer closure using CompetitiveScorer style scoring.
/// If no evidence manager is available, returns identity scoring.
pub fn competitive_retrieval_scorer(
    node_relevance: &std::collections::HashMap<u64, (f64, std::time::Instant)>,
    _relevance_decay: f64,
) -> impl Fn(u64, f64) -> f64 + '_ {
    move |node_id: u64, base_score: f64| {
        let relevance_bonus = node_relevance
            .get(&node_id)
            .map(|(rel, _)| *rel * 0.2)
            .unwrap_or(0.0);
        // Combine base activation with competitive score
        // activation weight 0.6, relevance bonus 0.2, confidence from activation 0.2
        base_score * 0.6 + relevance_bonus * 0.2 + (base_score * 0.2)
    }
}

fn now_nanos() -> u64 {
    crate::core::nt_core_time::unix_now_nanos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    fn empty_graph() -> MemoryGraph {
        MemoryGraph::new(100)
    }

    fn small_graph() -> MemoryGraph {
        let mut g = MemoryGraph::new(10);
        g.add_node(NodeKind::Semantic, test_vsa(1), "apple");
        g.add_node(NodeKind::Semantic, test_vsa(2), "banana");
        g.add_node(NodeKind::Semantic, test_vsa(3), "cherry");
        g
    }

    fn linked_graph() -> MemoryGraph {
        let mut g = MemoryGraph::new(20);
        let a = g.add_node(NodeKind::Semantic, test_vsa(1), "alpha");
        let b = g.add_node(NodeKind::Semantic, test_vsa(2), "beta");
        let c = g.add_node(NodeKind::Semantic, test_vsa(3), "gamma");
        let d = g.add_node(NodeKind::Semantic, test_vsa(4), "delta");
        g.add_edge(a, b, EdgeKind::Associative, 0.9);
        g.add_edge(b, c, EdgeKind::Causal, 0.8);
        g.add_edge(a, c, EdgeKind::Temporal, 0.5);
        g.add_edge(c, d, EdgeKind::Associative, 0.7);
        g
    }

    #[test]
    fn test_add_node_increases_count() {
        let mut g = MemoryGraph::new(100);
        assert_eq!(g.node_count(), 0);
        let id = g.add_node(NodeKind::Semantic, test_vsa(1), "x");
        assert_eq!(g.node_count(), 1);
        assert!(id > 0);
    }

    #[test]
    fn test_add_edge_creates_connection() {
        let mut g = MemoryGraph::new(100);
        let a = g.add_node(NodeKind::Semantic, test_vsa(1), "a");
        let b = g.add_node(NodeKind::Semantic, test_vsa(2), "b");
        assert_eq!(g.edge_count(), 0);
        g.add_edge(a, b, EdgeKind::Associative, 0.8);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn test_activate_from_vsa_returns_ranked() {
        let mut g = small_graph();
        let results = g.activate_from_vsa(&test_vsa(1), 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].1, "apple");
        assert!(results[0].2 >= results[1].2);
    }

    #[test]
    fn test_spread_propagates_activation() {
        let mut g = linked_graph();
        let nodes_before = g.node_count();
        let edges_before = g.edge_count();

        g.activate_from_vsa(&test_vsa(1), 1);
        let first_hop = g.spread(1, 0.85, 0.3);

        let beta_act = first_hop
            .iter()
            .find(|(_, l, _)| l == "beta")
            .map(|(_, _, a)| *a)
            .unwrap_or(0.0);
        assert!(
            beta_act > 0.0,
            "beta should receive activation from alpha (sim={})",
            beta_act
        );

        assert_eq!(g.node_count(), nodes_before);
        assert_eq!(g.edge_count(), edges_before);
    }

    #[test]
    fn test_similar_nodes_get_higher_activation() {
        let mut g = empty_graph();
        let q_vsa = test_vsa(42);
        g.add_node(NodeKind::Semantic, q_vsa.clone(), "target");
        g.add_node(NodeKind::Semantic, test_vsa(99), "distractor");

        let results = g.activate_from_vsa(&q_vsa, 2);
        assert_eq!(results[0].1, "target");
        assert!((results[0].2 - 1.0).abs() < 1e-6);

        let sim_distractor = QuantizedVSA::similarity(&test_vsa(99), &q_vsa);
        assert!(
            results[0].2 > results[1].2,
            "target (sim=1.0) should rank above distractor (sim={})",
            sim_distractor
        );
    }

    #[test]
    fn test_decay_all_reduces_activation() {
        let mut g = small_graph();
        g.activate_from_vsa(&test_vsa(1), 1);
        let before = g.nodes.iter().map(|n| n.activation).sum::<f64>();
        g.decay_all(0.5);
        let after = g.nodes.iter().map(|n| n.activation).sum::<f64>();
        assert!(after < before, "decay should reduce total activation");
        assert!(after > 0.0, "decay should not zero all activation");
    }

    #[test]
    fn test_reset_activation_clears_all() {
        let mut g = small_graph();
        g.activate_from_vsa(&test_vsa(1), 3);
        assert!(g.nodes.iter().any(|n| n.activation > 0.0));
        g.reset_activation();
        assert!(g.nodes.iter().all(|n| n.activation == 0.0));
    }

    #[test]
    fn test_max_nodes_enforces_eviction() {
        let mut g = MemoryGraph::new(3);
        g.add_node(NodeKind::Semantic, test_vsa(1), "a");
        g.add_node(NodeKind::Semantic, test_vsa(2), "b");
        g.add_node(NodeKind::Semantic, test_vsa(3), "c");
        assert_eq!(g.node_count(), 3);
        g.add_node(NodeKind::Semantic, test_vsa(4), "d");
        assert_eq!(g.node_count(), 3, "should not exceed max_nodes=3");
    }

    #[test]
    fn test_node_kind_differentiates() {
        let mut g = empty_graph();
        let e = g.add_node(NodeKind::Episodic, test_vsa(1), "memory_of_trip");
        let s = g.add_node(NodeKind::Semantic, test_vsa(2), "paris_capital");

        let e_node = g.nodes.iter().find(|n| n.id == e).unwrap();
        let s_node = g.nodes.iter().find(|n| n.id == s).unwrap();
        assert_eq!(e_node.kind, NodeKind::Episodic);
        assert_eq!(s_node.kind, NodeKind::Semantic);
        assert_ne!(e_node.kind, s_node.kind);
    }

    #[test]
    fn test_multi_hop_reaches_second_degree() {
        let mut g = linked_graph();
        g.activate_from_vsa(&test_vsa(1), 1);
        let result = g.spread(2, 0.85, 0.3);

        let gamma = result.iter().find(|(_, l, _)| l == "gamma");
        assert!(
            gamma.is_some(),
            "gamma (2nd degree from alpha) should be reachable"
        );
        if let Some((_, _, act)) = gamma {
            assert!(*act > 0.0, "gamma should have positive activation");
        }
    }

    #[test]
    fn test_inhibition_reduces_activation() {
        let mut g_none = MemoryGraph::new(20);
        let mut g_inh = MemoryGraph::new(20);

        let a = g_none.add_node(NodeKind::Semantic, test_vsa(1), "alpha");
        let b = g_none.add_node(NodeKind::Semantic, test_vsa(10), "beta");
        let c = g_none.add_node(NodeKind::Semantic, test_vsa(20), "gamma");
        let d = g_none.add_node(NodeKind::Semantic, test_vsa(30), "delta");
        g_none.add_edge(a, d, EdgeKind::Associative, 1.0);
        g_none.add_edge(b, d, EdgeKind::Associative, 1.0);
        g_none.add_edge(c, d, EdgeKind::Associative, 1.0);

        let a2 = g_inh.add_node(NodeKind::Semantic, test_vsa(1), "alpha");
        let b2 = g_inh.add_node(NodeKind::Semantic, test_vsa(10), "beta");
        let c2 = g_inh.add_node(NodeKind::Semantic, test_vsa(20), "gamma");
        let d2 = g_inh.add_node(NodeKind::Semantic, test_vsa(30), "delta");
        g_inh.add_edge(a2, d2, EdgeKind::Associative, 1.0);
        g_inh.add_edge(b2, d2, EdgeKind::Associative, 1.0);
        g_inh.add_edge(c2, d2, EdgeKind::Associative, 1.0);

        g_none.activate_from_vsa(&test_vsa(1), 1);
        g_inh.activate_from_vsa(&test_vsa(1), 1);

        let result_none = g_none.spread(1, 0.85, 0.0);
        let result_inh = g_inh.spread(1, 0.85, 0.9);

        let delta_none = result_none
            .iter()
            .find(|(_, l, _)| l == "delta")
            .map(|(_, _, a)| *a)
            .unwrap_or(0.0);
        let delta_inh = result_inh
            .iter()
            .find(|(_, l, _)| l == "delta")
            .map(|(_, _, a)| *a)
            .unwrap_or(0.0);

        assert!(
            delta_inh < delta_none,
            "inhibition should reduce delta activation: inh={} < none={}",
            delta_inh,
            delta_none
        );
    }

    #[test]
    fn test_self_activation_returns_exact_match_first() {
        let mut g = empty_graph();
        let target_vsa = test_vsa(7);
        g.add_node(NodeKind::Semantic, test_vsa(1), "other1");
        g.add_node(NodeKind::Semantic, target_vsa.clone(), "exact_match");
        g.add_node(NodeKind::Semantic, test_vsa(3), "other2");

        let results = g.activate_from_vsa(&target_vsa, 3);
        assert_eq!(results[0].1, "exact_match");
        assert!((results[0].2 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_retrieve_produces_results() {
        let mut g = linked_graph();
        let results = g.retrieve(&test_vsa(1), 3, None);
        assert!(!results.is_empty(), "retrieve should return results");
        assert!(results.len() <= 3);
        assert_eq!(results[0].1, "alpha");
    }

    #[test]
    fn test_spread_empty_graph_no_panic() {
        let mut g = MemoryGraph::new(10);
        let r = g.spread(3, 0.85, 0.3);
        assert!(r.is_empty());
    }

    #[test]
    fn test_edge_kind_names() {
        assert_eq!(EdgeKind::Temporal.name(), "temporal");
        assert_eq!(EdgeKind::Causal.name(), "causal");
        assert_eq!(EdgeKind::Associative.name(), "associative");
    }

    #[test]
    fn test_node_kind_names() {
        assert_eq!(NodeKind::Episodic.name(), "episodic");
        assert_eq!(NodeKind::Semantic.name(), "semantic");
    }
}
