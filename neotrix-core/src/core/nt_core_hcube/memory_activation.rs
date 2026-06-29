use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

#[allow(dead_code)]
/// VSA dimension used by test helpers — 512-bit sparse test vectors.
/// Different from the production VSA_DIM (4096) to keep tests lightweight.
const VSA_DIM: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryNodeType {
    Semantic,
    Causal,
    Temporal,
    Episodic,
    Procedural,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Semantic,
    Causal,
    Temporal,
    Sequential,
    CoAccess,
}

#[derive(Debug, Clone)]
pub struct MemoryNode {
    pub id: String,
    pub vsa_vector: Vec<u8>,
    pub activation: f64,
    pub base_activation: f64,
    pub node_type: MemoryNodeType,
    pub created_at: Instant,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MemoryEdge {
    pub source: String,
    pub target: String,
    pub weight: f64,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone)]
pub struct SpreadingActivation {
    nodes: HashMap<String, MemoryNode>,
    adjacency: HashMap<String, Vec<(String, f64)>>,
    edges: Vec<MemoryEdge>,
    activation_decay: f64,
    side_inhibition: f64,
    max_steps: usize,
    activation_threshold: f64,
}

impl Default for SpreadingActivation {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            edges: Vec::new(),
            activation_decay: 0.85,
            side_inhibition: 0.15,
            max_steps: 3,
            activation_threshold: 0.01,
        }
    }
}

impl SpreadingActivation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: String, vsa_vector: Vec<u8>, node_type: MemoryNodeType) {
        let node = MemoryNode {
            id: id.clone(),
            vsa_vector,
            activation: 0.0,
            base_activation: 0.1,
            node_type,
            created_at: Instant::now(),
            metadata: HashMap::new(),
        };
        self.nodes.insert(id, node);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn nodes(&self) -> &HashMap<String, MemoryNode> {
        &self.nodes
    }

    pub fn add_edge(&mut self, source: String, target: String, weight: f64, edge_type: EdgeType) {
        let edge = MemoryEdge {
            source: source.clone(),
            target: target.clone(),
            weight,
            edge_type,
        };
        self.edges.push(edge);
        self.adjacency
            .entry(source.clone())
            .or_default()
            .push((target.clone(), weight));
        self.adjacency
            .entry(target)
            .or_default()
            .push((source, weight));
    }

    pub fn activate_from_query(&mut self, query_vsa: &[u8]) -> Vec<(String, f64)> {
        let mut seeds: Vec<(String, f64)> = Vec::new();
        for (id, node) in &self.nodes {
            let sim = cosine_similarity(query_vsa, &node.vsa_vector);
            if sim > 0.3 {
                let seed = sim * node.base_activation;
                seeds.push((id.clone(), seed));
            }
        }
        for (id, seed) in &seeds {
            if let Some(n) = self.nodes.get_mut(id) {
                n.activation = *seed;
            }
        }

        let mut queue: VecDeque<(String, f64, usize)> =
            seeds.into_iter().map(|(id, seed)| (id, seed, 0)).collect();

        let mut visited: HashSet<String> = HashSet::new();

        while let Some((cur_id, act, step)) = queue.pop_front() {
            if step >= self.max_steps || act < self.activation_threshold {
                continue;
            }
            if !visited.insert(cur_id.clone()) {
                continue;
            }

            let neighbors = self.adjacency.get(&cur_id).cloned().unwrap_or_default();
            let cur_vec = self.nodes.get(&cur_id).map(|n| n.vsa_vector.clone());
            for (neighbor_id, edge_weight) in &neighbors {
                let propagated = act * self.activation_decay * edge_weight;
                if let Some(neighbor) = self.nodes.get_mut(neighbor_id) {
                    let overlap = match (&cur_vec, &neighbor.vsa_vector) {
                        (Some(a), b) if a.len() == b.len() => average_overlap(a, b),
                        _ => 0.0,
                    };
                    let inhibited = propagated * (1.0 - self.side_inhibition * overlap);
                    if inhibited > neighbor.activation {
                        neighbor.activation = inhibited;
                    }
                    queue.push_back((neighbor_id.clone(), inhibited, step + 1));
                }
            }
        }

        let mut results: Vec<(String, f64)> = self
            .nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.activation))
            .filter(|(_, a)| *a > self.activation_threshold)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn reset_activations(&mut self) {
        for node in self.nodes.values_mut() {
            node.activation = 0.0;
        }
    }

    pub fn apply_temporal_decay(&mut self, decay_rate: f64) {
        for node in self.nodes.values_mut() {
            node.base_activation *= decay_rate;
            if node.base_activation < 0.01 {
                node.base_activation = 0.01;
            }
        }
    }

    pub fn reinforce_edge(&mut self, source: &str, target: &str, increment: f64) {
        let max_weight = 1.0;
        for edge in &mut self.edges {
            if (edge.source == source && edge.target == target)
                || (edge.source == target && edge.target == source)
            {
                edge.weight = (edge.weight + increment).min(max_weight);
                if let Some(neighbors) = self.adjacency.get_mut(source) {
                    for (n, w) in neighbors.iter_mut() {
                        if *n == target {
                            *w = (*w + increment).min(max_weight);
                        }
                    }
                }
                if let Some(neighbors) = self.adjacency.get_mut(target) {
                    for (n, w) in neighbors.iter_mut() {
                        if *n == source {
                            *w = (*w + increment).min(max_weight);
                        }
                    }
                }
                return;
            }
        }
    }

    pub fn get_activation(&self, id: &str) -> f64 {
        self.nodes.get(id).map(|n| n.activation).unwrap_or(0.0)
    }

    pub fn get_base_activation(&self, id: &str) -> f64 {
        self.nodes.get(id).map(|n| n.base_activation).unwrap_or(0.0)
    }

    pub fn active_nodes(&self) -> Vec<(String, f64)> {
        let mut v: Vec<_> = self
            .nodes
            .iter()
            .map(|(id, n)| (id.clone(), n.activation))
            .filter(|(_, a)| *a > self.activation_threshold)
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }
}

fn average_overlap(a: &[u8], b: &[u8]) -> f64 {
    let min_len = a.len().min(b.len());
    if min_len == 0 {
        return 0.0;
    }
    let mut pop = 0u64;
    for i in 0..min_len {
        pop += (a[i] & b[i]).count_ones() as u64;
    }
    let total_bits = (min_len * 8) as f64;
    pop as f64 / total_bits
}

fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
    let min_len = a.len().min(b.len());
    if min_len == 0 {
        return 0.0;
    }
    let mut hd = 0u64;
    for i in 0..min_len {
        hd += (a[i] ^ b[i]).count_ones() as u64;
    }
    let dim = (min_len * 8) as f64;
    1.0 - 2.0 * hd as f64 / dim
}

pub struct MemoryActivationGraph {
    pub activation: SpreadingActivation,
    cycle: usize,
    co_access_history: Vec<(String, String)>,
}

impl MemoryActivationGraph {
    pub fn new() -> Self {
        Self {
            activation: SpreadingActivation::new(),
            cycle: 0,
            co_access_history: Vec::new(),
        }
    }

    pub fn record_co_access(&mut self, source: &str, target: &str) {
        self.co_access_history
            .push((source.to_string(), target.to_string()));
    }

    pub fn step(&mut self) {
        self.cycle += 1;
        if self.cycle % 4 == 0 {
            for (s, t) in self.co_access_history.drain(..) {
                self.activation.reinforce_edge(&s, &t, 0.05);
            }
        }
        if self.cycle % 8 == 0 {
            self.activation.apply_temporal_decay(0.95);
        }
    }

    pub fn rerank(
        &self,
        vsa_results: &[(String, f64)],
        activation_weight: f64,
    ) -> Vec<(String, f64)> {
        let act_scores: HashMap<&str, f64> = self
            .activation
            .nodes()
            .iter()
            .map(|(id, n)| (id.as_str(), n.activation))
            .collect();

        let mut combined: Vec<(String, f64)> = vsa_results
            .iter()
            .map(|(id, vsa_score)| {
                let act_score = act_scores.get(id.as_str()).copied().unwrap_or(0.0);
                let combined_score =
                    vsa_score * (1.0 - activation_weight) + act_score * activation_weight;
                (id.clone(), combined_score)
            })
            .collect();

        combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        combined
    }
}

static GLOBAL_MEMORY_ACTIVATION: std::sync::OnceLock<std::sync::Mutex<MemoryActivationGraph>> =
    std::sync::OnceLock::new();

pub fn global_memory_activation() -> &'static std::sync::Mutex<MemoryActivationGraph> {
    GLOBAL_MEMORY_ACTIVATION.get_or_init(|| std::sync::Mutex::new(MemoryActivationGraph::new()))
}

pub fn step_memory_activation() {
    if let Ok(mut graph) = global_memory_activation().lock() {
        graph.step();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn make_test_vec(val: u8) -> Vec<u8> {
        vec![val; VSA_DIM]
    }

    #[serial]
    #[test]
    fn test_spreading_activation_add_node() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        assert_eq!(sa.node_count(), 1);
    }

    #[test]
    fn test_spreading_activation_add_edge() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        sa.add_node("b".into(), make_test_vec(0xBB), MemoryNodeType::Semantic);
        sa.add_edge("a".into(), "b".into(), 0.8, EdgeType::Semantic);
        assert_eq!(sa.edge_count(), 1);
    }

    #[test]
    fn test_activate_from_query_triggers_propagation() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        sa.add_node("b".into(), make_test_vec(0xBB), MemoryNodeType::Semantic);
        sa.add_node("c".into(), make_test_vec(0xCC), MemoryNodeType::Semantic);
        sa.add_edge("a".into(), "b".into(), 0.9, EdgeType::Semantic);
        sa.add_edge("b".into(), "c".into(), 0.7, EdgeType::Semantic);

        let results = sa.activate_from_query(&make_test_vec(0xAA));
        assert!(!results.is_empty());
        assert_eq!(sa.get_activation("a"), results[0].1);
    }

    #[test]
    fn test_activate_empty_returns_empty() {
        let mut sa = SpreadingActivation::new();
        let results = sa.activate_from_query(&make_test_vec(0xFF));
        assert!(results.is_empty());
    }

    #[test]
    fn test_reset_activations() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        sa.activate_from_query(&make_test_vec(0xAA));
        assert!(sa.get_activation("a") > 0.0);
        sa.reset_activations();
        assert_eq!(sa.get_activation("a"), 0.0);
    }

    #[test]
    fn test_temporal_decay() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        let before = sa.get_base_activation("a");
        sa.apply_temporal_decay(0.5);
        let after = sa.get_base_activation("a");
        assert!(after < before);
        assert!(after >= 0.01);
    }

    #[test]
    fn test_reinforce_edge_increases_weight() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        sa.add_node("b".into(), make_test_vec(0xBB), MemoryNodeType::Semantic);
        sa.add_edge("a".into(), "b".into(), 0.5, EdgeType::CoAccess);
        sa.reinforce_edge("a", "b", 0.3);
        assert!((sa.edges[0].weight - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_rerank_combines_scores() {
        let mut graph = MemoryActivationGraph::new();
        graph
            .activation
            .add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        graph
            .activation
            .add_node("b".into(), make_test_vec(0xBB), MemoryNodeType::Semantic);
        graph
            .activation
            .add_node("c".into(), make_test_vec(0xCC), MemoryNodeType::Semantic);

        graph.activation.activate_from_query(&make_test_vec(0xAA));

        let vsa_results = vec![
            ("c".to_string(), 0.9),
            ("b".to_string(), 0.7),
            ("a".to_string(), 0.5),
        ];

        let reranked = graph.rerank(&vsa_results, 0.3);
        assert_eq!(reranked.len(), 3);
    }

    #[test]
    fn test_global_memory_activation() {
        let _g = global_memory_activation().lock();
        step_memory_activation();
    }

    #[test]
    fn test_record_co_access_builds_edges() {
        let mut graph = MemoryActivationGraph::new();
        graph
            .activation
            .add_node("a".into(), make_test_vec(0xAA), MemoryNodeType::Semantic);
        graph
            .activation
            .add_node("b".into(), make_test_vec(0xBB), MemoryNodeType::Semantic);
        graph
            .activation
            .add_node("c".into(), make_test_vec(0xCC), MemoryNodeType::Semantic);

        graph
            .activation
            .add_edge("a".into(), "b".into(), 0.5, EdgeType::Semantic);
        graph
            .activation
            .add_edge("b".into(), "c".into(), 0.5, EdgeType::Semantic);

        graph.record_co_access("a", "b");
        graph.record_co_access("a", "b");
        graph.step();

        assert!(graph.activation.edges[0].weight > 0.5);
    }

    #[test]
    fn test_side_inhibition_dampens_similar_activation() {
        let mut sa = SpreadingActivation::new();
        sa.add_node("a".into(), vec![0xFF; VSA_DIM], MemoryNodeType::Semantic);
        sa.add_node("b".into(), vec![0xFE; VSA_DIM], MemoryNodeType::Semantic);
        sa.add_edge("a".into(), "b".into(), 1.0, EdgeType::Semantic);

        let results = sa.activate_from_query(&vec![0xFF; VSA_DIM]);
        let a_act = results
            .iter()
            .find(|(id, _)| id == "a")
            .map(|(_, a)| *a)
            .unwrap_or(0.0);
        let b_act = results
            .iter()
            .find(|(id, _)| id == "b")
            .map(|(_, a)| *a)
            .unwrap_or(0.0);
        assert!(a_act > 0.0);
        assert!(b_act < a_act);
    }
}
