use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

const DEFAULT_GRAPH_DEGREE: usize = 32;
const DEFAULT_PRUNING_RATIO: f64 = 0.3;
const RECOMPUTE_THRESHOLD_ACCESS: u64 = 5;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: usize,
    pub label: String,
    pub feature_dims: usize,
    pub access_count: u64,
    pub last_access: Instant,
    pub degree: usize,
    pub stored_embedding: bool,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
    pub created: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecomputeStrategy {
    OnAccess,
    OnThreshold,
    Cached,
    Adaptive,
}

#[derive(Debug, Clone)]
pub struct LeannConfig {
    pub graph_degree: usize,
    pub pruning_ratio: f64,
    pub default_strategy: RecomputeStrategy,
    pub max_nodes: usize,
    pub enable_compaction: bool,
    pub recompute_threshold: u64,
}

impl Default for LeannConfig {
    fn default() -> Self {
        Self {
            graph_degree: DEFAULT_GRAPH_DEGREE,
            pruning_ratio: DEFAULT_PRUNING_RATIO,
            default_strategy: RecomputeStrategy::Adaptive,
            max_nodes: 100_000,
            enable_compaction: true,
            recompute_threshold: RECOMPUTE_THRESHOLD_ACCESS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StorageComparison {
    pub dense_embeddings_bytes: u64,
    pub graph_index_bytes: u64,
    pub leann_storage_bytes: u64,
    pub savings_percent: f64,
    pub node_count: usize,
    pub embedding_dim: usize,
}

impl StorageComparison {
    pub fn estimate(node_count: usize, embedding_dim: usize, bytes_per_float: usize) -> Self {
        let dense = (node_count * embedding_dim * bytes_per_float) as u64;
        let graph_nodes = (node_count * 64) as u64;
        let graph_edges = (node_count * DEFAULT_GRAPH_DEGREE * 16) as u64;
        let leann_storage = graph_nodes + graph_edges;
        let savings = if dense > 0 {
            1.0 - (leann_storage as f64 / dense as f64)
        } else {
            0.0
        };
        Self {
            dense_embeddings_bytes: dense,
            graph_index_bytes: graph_nodes + graph_edges,
            leann_storage_bytes: leann_storage,
            savings_percent: (savings * 10000.0).round() / 100.0,
            node_count,
            embedding_dim,
        }
    }
}

pub struct LeannGraphStore {
    config: LeannConfig,
    nodes: HashMap<usize, GraphNode>,
    edges: HashMap<usize, Vec<GraphEdge>>,
    adjacency: HashMap<usize, HashSet<usize>>,
    next_id: usize,
    access_log: VecDeque<usize>,
}

impl LeannGraphStore {
    pub fn new(config: LeannConfig) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
            next_id: 1,
            access_log: VecDeque::with_capacity(1000),
        }
    }

    pub fn insert_node(&mut self, label: &str, feature_dims: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let node = GraphNode {
            id,
            label: label.to_string(),
            feature_dims,
            access_count: 0,
            last_access: Instant::now(),
            degree: 0,
            stored_embedding: false,
        };
        self.nodes.insert(id, node);
        self.adjacency.insert(id, HashSet::new());
        self.edges.insert(id, Vec::new());
        id
    }

    pub fn connect(&mut self, from: usize, to: usize, weight: f64) {
        if from == to {
            return;
        }
        let weight = weight.max(0.0).min(1.0);
        let edge = GraphEdge {
            from,
            to,
            weight,
            created: Instant::now(),
        };
        self.edges.entry(from).or_default().push(edge.clone());
        self.adjacency.entry(from).or_default().insert(to);
        if let Some(node) = self.nodes.get_mut(&from) {
            node.degree = self.adjacency[&from].len();
        }
        let rev = GraphEdge {
            from: to,
            to: from,
            weight,
            created: Instant::now(),
        };
        self.edges.entry(to).or_default().push(rev);
        self.adjacency.entry(to).or_default().insert(from);
        if let Some(node) = self.nodes.get_mut(&to) {
            node.degree = self.adjacency[&to].len();
        }
    }

    pub fn access(&mut self, id: usize) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.access_count += 1;
            node.last_access = Instant::now();
            self.access_log.push_back(id);
            if self.access_log.len() > 1000 {
                self.access_log.pop_front();
            }
        }
    }

    pub fn needs_recompute(&self, id: usize) -> bool {
        self.nodes.get(&id).is_some_and(|node| {
            match self.config.default_strategy {
                RecomputeStrategy::OnAccess => !node.stored_embedding,
                RecomputeStrategy::OnThreshold => {
                    node.access_count >= self.config.recompute_threshold
                }
                RecomputeStrategy::Cached => false,
                RecomputeStrategy::Adaptive => {
                    if !node.stored_embedding {
                        true
                    } else {
                        node.access_count >= self.config.recompute_threshold
                    }
                }
            }
        })
    }

    pub fn mark_stored(&mut self, id: usize) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.stored_embedding = true;
            node.access_count = 0;
        }
    }

    pub fn high_degree_pruning(&mut self) -> usize {
        let threshold = (self.config.graph_degree as f64 * (1.0 + self.config.pruning_ratio)) as usize;
        let mut pruned = 0;
        let ids: Vec<usize> = self.nodes.keys().copied().collect();
        for id in ids {
            if let Some(neighbors) = self.adjacency.get(&id) {
                if neighbors.len() > threshold {
                    let mut sorted: Vec<(&usize, f64)> = self.edges.get(&id)
                        .map(|e| e.iter().map(|e| (&e.to, e.weight)).collect())
                        .unwrap_or_default();
                    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    let keep: HashSet<usize> = sorted.iter()
                        .take(self.config.graph_degree)
                        .map(|(id, _)| **id)
                        .collect();
                    let old_count = neighbors.len();
                    if let Some(adj) = self.adjacency.get_mut(&id) {
                        adj.retain(|n| keep.contains(n));
                    }
                    if let Some(edge_list) = self.edges.get_mut(&id) {
                        edge_list.retain(|e| keep.contains(&e.to));
                    }
                    if let Some(node) = self.nodes.get_mut(&id) {
                        node.degree = keep.len();
                    }
                    pruned += old_count.saturating_sub(keep.len());
                }
            }
        }
        pruned
    }

    pub fn greedy_search(&self, query_id: usize, top_k: usize) -> Vec<(usize, f64)> {
        let mut visited = HashSet::new();
        let mut candidates: Vec<(usize, f64)> = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(query_id);
        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) || candidates.len() >= top_k * 2 {
                continue;
            }
            visited.insert(current);
            if let Some(neighbors) = self.adjacency.get(&current) {
                for neighbor in neighbors.iter() {
                    if !visited.contains(neighbor) {
                        let weight = self.edges.get(&current)
                            .and_then(|e| e.iter().find(|e| e.to == *neighbor))
                            .map(|e| e.weight)
                            .unwrap_or(0.5);
                        candidates.push((*neighbor, weight));
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(top_k);
        candidates
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|e| e.len()).sum::<usize>() / 2
    }

    pub fn avg_degree(&self) -> f64 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        let total: usize = self.nodes.values().map(|n| n.degree).sum();
        total as f64 / self.nodes.len() as f64
    }

    pub fn estimate_storage(&self, embedding_dim: usize) -> StorageComparison {
        StorageComparison::estimate(self.nodes.len(), embedding_dim, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> LeannGraphStore {
        let config = LeannConfig {
            graph_degree: 8,
            pruning_ratio: 0.3,
            default_strategy: RecomputeStrategy::Adaptive,
            max_nodes: 1000,
            enable_compaction: true,
            recompute_threshold: 3,
        };
        LeannGraphStore::new(config)
    }

    #[test]
    fn test_insert_and_count() {
        let mut store = test_store();
        let n1 = store.insert_node("alpha", 128);
        let n2 = store.insert_node("beta", 128);
        assert_eq!(store.node_count(), 2);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_connect_and_degree() {
        let mut store = test_store();
        let n1 = store.insert_node("a", 64);
        let n2 = store.insert_node("b", 64);
        let n3 = store.insert_node("c", 64);
        store.connect(n1, n2, 0.9);
        store.connect(n1, n3, 0.7);
        assert_eq!(store.edge_count(), 2);
        assert!((store.avg_degree() - 4.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_access_and_recompute() {
        let mut store = test_store();
        let n1 = store.insert_node("frequent", 256);
        assert!(store.needs_recompute(n1));
        for _ in 0..4 {
            store.access(n1);
        }
        assert!(store.needs_recompute(n1));
        store.mark_stored(n1);
        store.access(n1);
        assert!(!store.needs_recompute(n1));
    }

    #[test]
    fn test_high_degree_pruning() {
        let mut store = test_store();
        let center = store.insert_node("center", 64);
        let mut neighbors = Vec::new();
        for i in 0..20 {
            let n = store.insert_node(&format!("n{}", i), 64);
            store.connect(center, n, 0.5 + (i as f64 * 0.02));
            neighbors.push(n);
        }
        assert_eq!(store.edge_count(), 20);
        let pruned = store.high_degree_pruning();
        assert!(pruned > 0);
        assert!(store.edge_count() < 20);
    }

    #[test]
    fn test_greedy_search() {
        let mut store = test_store();
        let nodes: Vec<usize> = (0..10).map(|i| store.insert_node(&format!("n{}", i), 32)).collect();
        for i in 0..9 {
            store.connect(nodes[i], nodes[i + 1], 0.8);
        }
        let results = store.greedy_search(nodes[0], 3);
        assert!(!results.is_empty());
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_self_connect_rejected() {
        let mut store = test_store();
        let n = store.insert_node("self", 8);
        store.connect(n, n, 1.0);
        assert_eq!(store.edge_count(), 0);
    }

    #[test]
    fn test_storage_estimation() {
        let comp = StorageComparison::estimate(1_000_000, 768, 4);
        assert!(comp.savings_percent > 80.0);
        assert_eq!(comp.node_count, 1_000_000);
        assert_eq!(comp.embedding_dim, 768);
    }

    #[test]
    fn test_default_config() {
        let config = LeannConfig::default();
        assert_eq!(config.graph_degree, 32);
        assert!((config.pruning_ratio - 0.3).abs() < 0.001);
        assert_eq!(config.default_strategy, RecomputeStrategy::Adaptive);
    }
}
