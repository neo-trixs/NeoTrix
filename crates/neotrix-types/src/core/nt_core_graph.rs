use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HyperNodeType {
    Concept,
    Memory,
    Skill,
    Pattern,
    Goal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeRelation {
    DerivesFrom,
    SimilarTo,
    PrerequisiteOf,
    Enhances,
    ConflictsWith,
    PartOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub from_id: String,
    pub to_id: String,
    pub relation: EdgeRelation,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperNode {
    pub id: String,
    pub node_type: HyperNodeType,
    pub content: String,
    pub embedding: Vec<f64>,
    pub edges: Vec<HyperEdge>,
    pub weight: f64,
    pub created_at: u64,
    pub access_count: u64,
}

impl HyperNode {
    pub fn new(id: &str, node_type: HyperNodeType, content: &str, weight: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: id.to_string(),
            node_type,
            content: content.to_string(),
            embedding: Vec::new(),
            edges: Vec::new(),
            weight,
            created_at: now,
            access_count: 0,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f64>) -> Self {
        self.embedding = embedding;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperGraph {
    pub nodes: HashMap<String, HyperNode>,
    pub adjacency: HashMap<String, Vec<HyperEdge>>,
}

impl HyperGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(capacity),
            adjacency: HashMap::with_capacity(capacity),
        }
    }

    pub fn add_node(&mut self, node: HyperNode) {
        let node_id = node.id.clone();
        self.adjacency.entry(node_id.clone()).or_default();
        self.nodes.insert(node_id, node);
    }

    pub fn add_edge(&mut self, from: &str, to: &str, relation: EdgeRelation, strength: f64) {
        let edge = HyperEdge {
            from_id: from.to_string(),
            to_id: to.to_string(),
            relation,
            strength: strength.clamp(0.0, 1.0),
        };
        let rev = HyperEdge {
            from_id: to.to_string(),
            to_id: from.to_string(),
            relation: edge.relation.clone(),
            strength: edge.strength,
        };
        self.adjacency.entry(from.to_string()).or_default().push(edge);
        self.adjacency.entry(to.to_string()).or_default().push(rev);
    }

    pub fn traverse(&self, start_id: &str, depth: usize) -> Vec<&HyperNode> {
        if !self.nodes.contains_key(start_id) {
            return Vec::new();
        }
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut result: Vec<&HyperNode> = Vec::new();

        queue.push_back((start_id.to_string(), 0));
        visited.insert(start_id.to_string());

        while let Some((current_id, current_depth)) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&current_id) {
                result.push(node);
            }
            if current_depth >= depth {
                continue;
            }
            if let Some(edges) = self.adjacency.get(&current_id) {
                for edge in edges {
                    if visited.insert(edge.to_id.clone()) {
                        queue.push_back((edge.to_id.clone(), current_depth + 1));
                    }
                }
            }
        }
        result
    }

    pub fn find_related(
        &self,
        node_id: &str,
        relation: Option<EdgeRelation>,
        limit: usize,
    ) -> Vec<&HyperNode> {
        let mut scored: Vec<(f64, &HyperNode)> = Vec::new();
        if let Some(edges) = self.adjacency.get(node_id) {
            let mut seen = HashSet::new();
            for edge in edges {
                if let Some(ref rel) = relation {
                    if edge.relation != *rel {
                        continue;
                    }
                }
                if !seen.insert(edge.to_id.clone()) {
                    continue;
                }
                if let Some(node) = self.nodes.get(&edge.to_id) {
                    scored.push((edge.strength, node));
                }
            }
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored.into_iter().map(|(_, n)| n).collect()
    }

    pub fn subgraph(&self, node_ids: &[String]) -> HyperGraph {
        let id_set: HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();
        let mut sub = HyperGraph::new();
        for id in node_ids {
            if let Some(node) = self.nodes.get(id) {
                let mut sub_node = node.clone();
                sub_node.edges.retain(|e| id_set.contains(e.to_id.as_str()));
                sub.add_node(sub_node);
            }
        }
        for id in node_ids {
            if let Some(edges) = self.adjacency.get(id) {
                let edges: Vec<HyperEdge> = edges
                    .iter()
                    .filter(|e| id_set.contains(e.to_id.as_str()))
                    .cloned()
                    .collect();
                sub.adjacency.insert(id.clone(), edges);
            }
        }
        sub
    }

    pub fn merge_from(&mut self, other: HyperGraph) {
        for (id, node) in other.nodes {
            if !self.nodes.contains_key(&id) {
                self.add_node(node);
            }
        }
        for (from_id, edges) in other.adjacency {
            let entry = self.adjacency.entry(from_id).or_default();
            for edge in edges {
                if !entry.iter().any(|e| e.to_id == edge.to_id && e.relation == edge.relation) {
                    entry.push(edge);
                }
            }
        }
    }

    pub fn prune(&mut self, threshold: f64) {
        let mut to_remove: Vec<(String, usize)> = Vec::new();
        for (from_id, edges) in self.adjacency.iter_mut() {
            edges.retain(|e| e.strength >= threshold);
            for (i, edge) in edges.iter().enumerate() {
                if edge.strength < threshold {
                    to_remove.push((from_id.clone(), i));
                }
            }
        }
        to_remove.sort_by(|a, b| b.1.cmp(&a.1));
        for (from_id, idx) in to_remove {
            if let Some(edges) = self.adjacency.get_mut(&from_id) {
                if idx < edges.len() {
                    edges.remove(idx);
                }
            }
        }

        let connected: HashSet<String> = self
            .adjacency
            .iter()
            .flat_map(|(from, edges)| {
                let mut ids = vec![from.clone()];
                ids.extend(edges.iter().map(|e| e.to_id.clone()));
                ids
            })
            .collect();
        self.nodes.retain(|id, _| connected.contains(id));
        self.adjacency.retain(|id, edges| {
            let has_edges = !edges.is_empty();
            let is_node = self.nodes.contains_key(id);
            has_edges || is_node
        });
    }

    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 && norm_b == 0.0 {
            return 1.0;
        }
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|e| e.len()).sum()
    }
}

impl Default for HyperGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> HyperGraph {
        let mut graph = HyperGraph::with_capacity(10);
        graph.add_node(HyperNode::new("n1", HyperNodeType::Concept, "machine learning", 0.9));
        graph.add_node(HyperNode::new("n2", HyperNodeType::Concept, "neural networks", 0.85));
        graph.add_node(HyperNode::new("n3", HyperNodeType::Concept, "deep learning", 0.8));
        graph.add_node(HyperNode::new("n4", HyperNodeType::Skill, "Python", 0.95));
        graph.add_node(HyperNode::new("n5", HyperNodeType::Pattern, "attention mechanism", 0.75));

        graph.add_edge("n1", "n2", EdgeRelation::DerivesFrom, 0.9);
        graph.add_edge("n2", "n3", EdgeRelation::PartOf, 0.85);
        graph.add_edge("n1", "n3", EdgeRelation::SimilarTo, 0.7);
        graph.add_edge("n4", "n2", EdgeRelation::Enhances, 0.6);
        graph.add_edge("n5", "n3", EdgeRelation::PartOf, 0.95);
        graph
    }

    #[test]
    fn test_hypergraph_add_node_edge() {
        let mut graph = HyperGraph::new();
        let node = HyperNode::new("test1", HyperNodeType::Memory, "test memory", 0.5);
        graph.add_node(node);
        assert_eq!(graph.node_count(), 1);

        let node2 = HyperNode::new("test2", HyperNodeType::Goal, "test goal", 0.8);
        graph.add_node(node2);
        graph.add_edge("test1", "test2", EdgeRelation::Enhances, 0.75);
        assert_eq!(graph.edge_count(), 2);
        assert!(graph.nodes.contains_key("test1"));
        assert!(graph.nodes.contains_key("test2"));
    }

    #[test]
    fn test_hypergraph_bfs_traverse() {
        let graph = create_test_graph();
        let result = graph.traverse("n1", 1);
        assert_eq!(result.len(), 3);
        let ids: Vec<&str> = result.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"n1"));
        assert!(ids.contains(&"n2"));
        assert!(ids.contains(&"n3"));

        let result_deep = graph.traverse("n1", 2);
        let ids_deep: Vec<&str> = result_deep.iter().map(|n| n.id.as_str()).collect();
        assert!(ids_deep.contains(&"n4"));
        assert!(ids_deep.contains(&"n5"));
    }

    #[test]
    fn test_hypergraph_find_related() {
        let graph = create_test_graph();
        let related = graph.find_related("n2", None, 10);
        assert_eq!(related.len(), 3);
        let ids: Vec<&str> = related.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"n1"));
        assert!(ids.contains(&"n3"));
        assert!(ids.contains(&"n4"));

        let part_of = graph.find_related("n3", Some(EdgeRelation::PartOf), 10);
        assert_eq!(part_of.len(), 2);
    }

    #[test]
    fn test_hypergraph_merge() {
        let mut graph1 = create_test_graph();
        let mut graph2 = HyperGraph::new();
        graph2.add_node(HyperNode::new("n6", HyperNodeType::Goal, "new goal", 0.7));
        graph2.add_node(HyperNode::new("n1", HyperNodeType::Concept, "different content", 0.3));
        graph2.add_edge("n6", "n1", EdgeRelation::Enhances, 0.5);

        graph1.merge_from(graph2);
        assert_eq!(graph1.node_count(), 6);
        assert_eq!(graph1.nodes.get("n1").expect("n1 should exist in merged graph").content, "machine learning");
        assert!(graph1.nodes.contains_key("n6"));

        let n1_edges = graph1.adjacency.get("n1").expect("n1 should have adjacency entry");
        let has_n6_edge = n1_edges.iter().any(|e| e.to_id == "n6");
        assert!(has_n6_edge);
    }

    #[test]
    fn test_hypergraph_prune() {
        let mut graph = create_test_graph();
        let before_edges = graph.edge_count();
        graph.prune(0.85);
        let after_edges = graph.edge_count();
        assert!(after_edges < before_edges);
        if let Some(edges) = graph.adjacency.get("n1") {
            for e in edges {
                assert!(e.strength >= 0.85);
            }
        }
    }

    #[test]
    fn test_hypergraph_subgraph() {
        let graph = create_test_graph();
        let ids = vec!["n1".to_string(), "n2".to_string(), "n3".to_string()];
        let sub = graph.subgraph(&ids);
        assert_eq!(sub.node_count(), 3);
        assert!(sub.nodes.contains_key("n1"));
        assert!(sub.nodes.contains_key("n2"));
        assert!(sub.nodes.contains_key("n3"));
        assert!(!sub.nodes.contains_key("n4"));
    }

    #[test]
    fn test_hypergraph_traverse_nonexistent() {
        let graph = create_test_graph();
        let result = graph.traverse("nonexistent", 2);
        assert!(result.is_empty());
    }
}
