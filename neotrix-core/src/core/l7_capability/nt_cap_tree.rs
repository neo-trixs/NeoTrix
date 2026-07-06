use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceType {
    Native { module_path: String },
    External { confidence: f64 },
    Composed { components: Vec<String>, strategy: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Provenance {
    Manual,
    Absorbed { source_url: String },
    Composed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ms: f64,
    pub success_rate: f64,
    pub cost: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self { latency_ms: 0.0, success_rate: 1.0, cost: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityNode {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub vector: Vec<f64>,
    pub interface_type: InterfaceType,
    pub maturity: u8,
    pub confidence: f64,
    pub usage_count: u64,
    pub last_success: Option<Instant>,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub provenance: Provenance,
    pub description: String,
    pub performance: PerformanceMetrics,
}

impl CapabilityNode {
    pub fn new(id: &str, name: &str, domain: &str, vector: Vec<f64>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            domain: domain.to_string(),
            vector,
            interface_type: InterfaceType::Native { module_path: String::new() },
            maturity: 1,
            confidence: 0.5,
            usage_count: 0,
            last_success: None,
            parent: None,
            children: Vec::new(),
            provenance: Provenance::Manual,
            description: String::new(),
            performance: PerformanceMetrics::default(),
        }
    }

    pub fn similarity(&self, other: &[f64]) -> f64 {
        let dot: f64 = self.vector.iter().zip(other.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f64 = self.vector.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = other.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        dot / (norm_a * norm_b)
    }

    pub fn record_success(&mut self) {
        self.usage_count += 1;
        self.last_success = Some(Instant::now());
        self.confidence = (self.confidence + 0.05).min(1.0);
        if self.maturity < 6 && self.usage_count.is_multiple_of(5) {
            self.maturity = (self.maturity + 1).min(6);
        }
    }

    pub fn record_failure(&mut self) {
        self.confidence = (self.confidence - 0.1).max(0.0);
        if self.confidence < 0.2 && self.maturity > 1 {
            self.maturity -= 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityTree {
    nodes: HashMap<String, CapabilityNode>,
    root_id: Option<String>,
}

impl CapabilityTree {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), root_id: None }
    }

    pub fn set_root(&mut self, id: &str) {
        self.root_id = Some(id.to_string());
    }

    pub fn register(&mut self, node: CapabilityNode) {
        let id = node.id.clone();
        self.nodes.insert(id, node);
    }

    pub fn get(&self, id: &str) -> Option<&CapabilityNode> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut CapabilityNode> {
        self.nodes.get_mut(id)
    }

    pub fn link(&mut self, parent_id: &str, child_id: &str) {
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            if !parent.children.contains(&child_id.to_string()) {
                parent.children.push(child_id.to_string());
            }
        }
        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id.to_string());
        }
    }

    pub fn find_closest(&self, query: &[f64], top_k: usize) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = self.nodes.iter()
            .map(|(id, node)| (id.clone(), node.similarity(query)))
            .filter(|(_, s)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }

    pub fn search_by_domain(&self, domain: &str) -> Vec<&CapabilityNode> {
        self.nodes.values().filter(|n| n.domain == domain).collect()
    }

    pub fn prune(&mut self, min_maturity: u8, min_confidence: f64) -> Vec<String> {
        let to_remove: Vec<String> = self.nodes.iter()
            .filter(|(_, n)| n.maturity < min_maturity && n.confidence < min_confidence)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &to_remove {
            self.nodes.remove(id);
        }
        to_remove
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn all_nodes(&self) -> impl Iterator<Item = &CapabilityNode> {
        self.nodes.values()
    }

    pub fn children_of(&self, parent_id: &str) -> Vec<&CapabilityNode> {
        self.nodes.get(parent_id)
            .map(|p| p.children.iter().filter_map(|cid| self.nodes.get(cid)).collect())
            .unwrap_or_default()
    }

    pub fn subgraph(&self, root_id: &str, depth: usize) -> Vec<&CapabilityNode> {
        let mut result = Vec::new();
        let mut stack = vec![(root_id.to_string(), 0)];
        while let Some((id, d)) = stack.pop() {
            if d > depth { continue; }
            if let Some(node) = self.nodes.get(&id) {
                result.push(node);
                for child in &node.children {
                    stack.push((child.clone(), d + 1));
                }
            }
        }
        result
    }
}

impl Default for CapabilityTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vector(v: &[f64]) -> Vec<f64> {
        let mut arr = vec![0.0; 23];
        for (i, &val) in v.iter().enumerate().take(arr.len()) {
            arr[i] = val;
        }
        arr
    }

    #[test]
    fn test_register_and_get() {
        let mut tree = CapabilityTree::new();
        let node = CapabilityNode::new("e8_1", "E8 Reasoning", "Cognitive", test_vector(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.9, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
        tree.register(node);
        assert!(tree.get("e8_1").is_some());
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn test_find_closest() {
        let mut tree = CapabilityTree::new();
        let v1 = test_vector(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.9, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let v2 = test_vector(&[0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        tree.register(CapabilityNode::new("e8", "E8 Reasoning", "Cognitive", v1.clone()));
        tree.register(CapabilityNode::new("img", "Image Gen", "Creative", v2.clone()));
        let result = tree.find_closest(&v1, 2);
        assert_eq!(result[0].0, "e8");
        assert!((result[0].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_link_and_children() {
        let mut tree = CapabilityTree::new();
        tree.register(CapabilityNode::new("root", "Root", "Meta", vec![0.0; 23]));
        tree.register(CapabilityNode::new("c1", "Child1", "Cognitive", vec![0.0; 23]));
        tree.link("root", "c1");
        let children = tree.children_of("root");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Child1");
    }

    #[test]
    fn test_record_success_increases_maturity() {
        let mut node = CapabilityNode::new("t1", "Test", "Test", vec![0.0; 23]);
        for _ in 0..5 { node.record_success(); }
        assert_eq!(node.maturity, 2);
        assert!((node.confidence - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_record_failure_decreases_confidence() {
        let mut node = CapabilityNode::new("t1", "Test", "Test", vec![0.0; 23]);
        node.record_failure();
        assert!((node.confidence - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_prune_removes_low_maturity_nodes() {
        let mut tree = CapabilityTree::new();
        tree.register(CapabilityNode::new("keep", "Keep", "Test", vec![0.0; 23]));
        let mut low = CapabilityNode::new("prune", "Prune", "Test", vec![0.0; 23]);
        low.maturity = 0;
        low.confidence = 0.0;
        tree.register(low);
        let removed = tree.prune(1, 0.1);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "prune");
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn test_subgraph_depth_limited() {
        let mut tree = CapabilityTree::new();
        tree.register(CapabilityNode::new("r", "R", "Test", vec![0.0; 23]));
        tree.register(CapabilityNode::new("a", "A", "Test", vec![0.0; 23]));
        tree.register(CapabilityNode::new("b", "B", "Test", vec![0.0; 23]));
        tree.register(CapabilityNode::new("c", "C", "Test", vec![0.0; 23]));
        tree.link("r", "a");
        tree.link("r", "b");
        tree.link("a", "c");
        let sg = tree.subgraph("r", 1);
        assert_eq!(sg.len(), 3);
    }

    #[test]
    fn test_search_by_domain() {
        let mut tree = CapabilityTree::new();
        tree.register(CapabilityNode::new("e8", "E8", "Cognitive", vec![0.0; 23]));
        tree.register(CapabilityNode::new("prm", "PRM", "Cognitive", vec![0.0; 23]));
        tree.register(CapabilityNode::new("img", "IMG", "Creative", vec![0.0; 23]));
        let cog = tree.search_by_domain("Cognitive");
        assert_eq!(cog.len(), 2);
    }

    #[test]
    fn test_similarity_orthogonal() {
        let mut a = vec![0.0; 23]; a[0] = 1.0;
        let mut b = vec![0.0; 23]; b[1] = 1.0;
        let node = CapabilityNode::new("n", "N", "T", a);
        let sim = node.similarity(&b);
        assert!((sim - 0.0).abs() < 1e-10);
    }
}
