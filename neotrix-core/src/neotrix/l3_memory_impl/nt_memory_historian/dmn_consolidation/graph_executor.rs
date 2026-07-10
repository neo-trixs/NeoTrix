#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::three_tier::MemoryItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedGraphExecutor {
    pub graph: HashMap<usize, Vec<(usize, f64)>>,
    pub nodes: HashMap<usize, MemoryItem>,
    pub learning_rate: f64,
    pub next_id: usize,
}

impl LearnedGraphExecutor {
    pub fn new(learning_rate: f64) -> Self {
        Self {
            graph: HashMap::new(),
            nodes: HashMap::new(),
            learning_rate,
            next_id: 1,
        }
    }

    pub fn add_node(&mut self, item: MemoryItem) -> usize {
        let id = item.id;
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        self.nodes.insert(id, item);
        id
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        let weight = weight.max(0.0).min(1.0);
        self.graph
            .entry(from)
            .or_insert_with(Vec::new)
            .push((to, weight));
    }

    pub fn learn_relationship(&mut self, a: usize, b: usize, delta: f64) {
        let adjusted = delta * self.learning_rate;
        if let Some(edges) = self.graph.get_mut(&a) {
            if let Some(existing) = edges.iter_mut().find(|(to, _)| *to == b) {
                existing.1 = (existing.1 + adjusted).max(0.0).min(1.0);
                return;
            }
        }
        self.add_edge(a, b, adjusted.max(0.0).min(1.0));
    }

    pub fn query(
        &self,
        seed_ids: &[usize],
        max_depth: usize,
        max_results: usize,
    ) -> Vec<MemoryItem> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = VecDeque::new();
        let mut results = Vec::new();

        for &seed in seed_ids {
            if self.nodes.contains_key(&seed) && !visited.contains(&seed) {
                visited.insert(seed);
                queue.push_back((seed, 0));
            }
        }

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }
            if let Some(item) = self.nodes.get(&node_id) {
                results.push(item.clone());
            }
            if depth < max_depth {
                if let Some(edges) = self.graph.get(&node_id) {
                    for &(neighbor, _) in edges {
                        if !visited.contains(&neighbor) && self.nodes.contains_key(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back((neighbor, depth + 1));
                        }
                    }
                }
            }
            if results.len() >= max_results {
                break;
            }
        }

        results.truncate(max_results);
        results
    }

    pub fn spreading_activation(
        &self,
        seed_ids: &[usize],
        decay: f64,
        threshold: f64,
    ) -> Vec<(usize, f64)> {
        let mut activations: HashMap<usize, f64> = HashMap::new();
        let mut queue = VecDeque::new();

        for &seed in seed_ids {
            if self.nodes.contains_key(&seed) {
                activations.insert(seed, 1.0);
                queue.push_back(seed);
            }
        }

        while let Some(current) = queue.pop_front() {
            let current_act = activations[&current];
            if let Some(edges) = self.graph.get(&current) {
                for &(neighbor, weight) in edges {
                    if !self.nodes.contains_key(&neighbor) {
                        continue;
                    }
                    let spread = current_act * weight * decay;
                    if spread < threshold {
                        continue;
                    }
                    let entry = activations.entry(neighbor).or_insert(0.0);
                    if *entry < spread {
                        *entry = spread;
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        let mut result: Vec<(usize, f64)> = activations.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn consolidate_edge(&mut self, from: usize, to: usize) {
        if let Some(edges) = self.graph.get_mut(&from) {
            if let Some(existing) = edges.iter_mut().find(|(t, _)| *t == to) {
                existing.1 = (existing.1 + self.learning_rate).max(0.0).min(1.0);
            }
        }
    }

    pub fn prune_edges(&mut self, threshold: f64) {
        self.graph.retain(|_, edges| {
            edges.retain(|&(_, weight)| weight >= threshold);
            !edges.is_empty()
        });
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.values().map(|e| e.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_historian::dmn_consolidation::three_tier::{
        MemoryItem, MemoryTier,
    };

    fn make_item(id: usize, content: &str) -> MemoryItem {
        MemoryItem::new(id, content.to_string(), 0.5, MemoryTier::LongTerm)
    }

    #[test]
    fn test_add_nodes() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        let id = executor.add_node(make_item(1, "node1"));
        assert_eq!(id, 1);
        assert_eq!(executor.node_count(), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.add_edge(1, 2, 0.8);
        assert_eq!(executor.edge_count(), 1);
    }

    #[test]
    fn test_bfs_query() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "root"));
        executor.add_node(make_item(2, "child1"));
        executor.add_node(make_item(3, "child2"));
        executor.add_node(make_item(4, "grandchild"));
        executor.add_edge(1, 2, 1.0);
        executor.add_edge(1, 3, 1.0);
        executor.add_edge(2, 4, 1.0);

        let results = executor.query(&[1], 1, 10);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_bfs_query_max_depth() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "root"));
        executor.add_node(make_item(2, "child"));
        executor.add_node(make_item(3, "grandchild"));
        executor.add_edge(1, 2, 1.0);
        executor.add_edge(2, 3, 1.0);

        let results = executor.query(&[1], 0, 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_spreading_activation() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.add_node(make_item(3, "c"));
        executor.add_edge(1, 2, 0.8);
        executor.add_edge(2, 3, 0.5);

        let results = executor.spreading_activation(&[1], 0.5, 0.01);
        assert!(results.len() >= 2);
        let act_map: HashMap<usize, f64> = results.into_iter().collect();
        assert!(act_map.contains_key(&1));
        assert!(act_map.contains_key(&2));
    }

    #[test]
    fn test_spreading_activation_threshold() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.add_edge(1, 2, 0.1);

        let results = executor.spreading_activation(&[1], 0.5, 0.9);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_learn_relationship_new() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.learn_relationship(1, 2, 0.8);
        assert_eq!(executor.edge_count(), 1);
    }

    #[test]
    fn test_learn_relationship_update() {
        let mut executor = LearnedGraphExecutor::new(0.5);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.add_edge(1, 2, 0.5);
        executor.learn_relationship(1, 2, 0.6);
        let edges = executor.graph.get(&1).unwrap();
        let weight = edges.iter().find(|(t, _)| *t == 2).unwrap().1;
        assert!((weight - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_consolidate_edge() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.add_edge(1, 2, 0.5);
        executor.consolidate_edge(1, 2);
        let weight = executor.graph.get(&1).unwrap()[0].1;
        assert!((weight - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_prune_edges() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        executor.add_node(make_item(1, "a"));
        executor.add_node(make_item(2, "b"));
        executor.add_node(make_item(3, "c"));
        executor.add_edge(1, 2, 0.1);
        executor.add_edge(1, 3, 0.9);
        executor.prune_edges(0.5);
        assert_eq!(executor.edge_count(), 1);
    }

    #[test]
    fn test_node_and_edge_counts() {
        let mut executor = LearnedGraphExecutor::new(0.1);
        assert_eq!(executor.node_count(), 0);
        assert_eq!(executor.edge_count(), 0);
        executor.add_node(make_item(1, "x"));
        executor.add_node(make_item(2, "y"));
        executor.add_edge(1, 2, 0.7);
        assert_eq!(executor.node_count(), 2);
        assert_eq!(executor.edge_count(), 1);
    }
}
