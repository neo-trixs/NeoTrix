use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Verified,
    Rejected,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisNode {
    pub id: String,
    pub content: String,
    pub status: HypothesisStatus,
    pub confidence: f64,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub created_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTreeConfig {
    pub max_depth: usize,
    pub min_confidence_for_leaf: f64,
}

impl Default for HypothesisTreeConfig {
    fn default() -> Self {
        Self { max_depth: 10, min_confidence_for_leaf: 0.3 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTreeStats {
    pub total_nodes: usize,
    pub verified: usize,
    pub rejected: usize,
    pub max_depth: usize,
    pub leaf_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTree {
    nodes: HashMap<String, HypothesisNode>,
    root_ids: Vec<String>,
    config: HypothesisTreeConfig,
}

impl HypothesisTree {
    pub fn new(config: HypothesisTreeConfig) -> Self {
        Self { nodes: HashMap::new(), root_ids: Vec::new(), config }
    }

    pub fn add_root(&mut self, id: &str, content: &str, confidence: f64) -> Option<&HypothesisNode> {
        if self.nodes.contains_key(id) {
            return None;
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let node = HypothesisNode {
            id: id.to_string(),
            content: content.to_string(),
            status: HypothesisStatus::Proposed,
            confidence,
            parent_id: None,
            children: Vec::new(),
            created_at: ts,
            tags: Vec::new(),
        };
        self.root_ids.push(id.to_string());
        self.nodes.insert(id.to_string(), node);
        self.nodes.get(id)
    }

    pub fn add_child(&mut self, parent_id: &str, id: &str, content: &str, confidence: f64) -> Option<&HypothesisNode> {
        let _parent = self.nodes.get(parent_id)?;
        let depth = self.depth_of(parent_id)?;
        if depth >= self.config.max_depth {
            return None;
        }
        if self.nodes.contains_key(id) {
            return None;
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let node = HypothesisNode {
            id: id.to_string(),
            content: content.to_string(),
            status: HypothesisStatus::Proposed,
            confidence,
            parent_id: Some(parent_id.to_string()),
            children: Vec::new(),
            created_at: ts,
            tags: Vec::new(),
        };
        if let Some(p) = self.nodes.get_mut(parent_id) {
            p.children.push(id.to_string());
        }
        self.nodes.insert(id.to_string(), node);
        self.nodes.get(id)
    }

    pub fn update_status(&mut self, id: &str, status: HypothesisStatus) -> bool {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = status;
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<&HypothesisNode> {
        self.nodes.get(id)
    }

    pub fn depth_of(&self, id: &str) -> Option<usize> {
        let node = self.nodes.get(id)?;
        let mut depth = 0;
        let mut current = node;
        while let Some(ref parent_id) = current.parent_id {
            current = self.nodes.get(parent_id)?;
            depth += 1;
        }
        Some(depth)
    }

    pub fn subtree(&self, root_id: &str) -> Vec<&HypothesisNode> {
        let mut result = Vec::new();
        if let Some(node) = self.nodes.get(root_id) {
            result.push(node);
            for child_id in &node.children {
                result.extend(self.subtree(child_id));
            }
        }
        result
    }

    pub fn leaves(&self) -> Vec<&HypothesisNode> {
        self.nodes.values().filter(|n| n.children.is_empty()).collect()
    }

    pub fn stats(&self) -> HypothesisTreeStats {
        let verified = self.nodes.values().filter(|n| n.status == HypothesisStatus::Verified).count();
        let rejected = self.nodes.values().filter(|n| n.status == HypothesisStatus::Rejected).count();
        let leaves = self.leaves().len();
        let max_depth = self.root_ids.iter().filter_map(|r| self.depth_of(r)).max().unwrap_or(0);
        HypothesisTreeStats {
            total_nodes: self.nodes.len(),
            verified,
            rejected,
            max_depth,
            leaf_count: leaves,
        }
    }

    pub fn prune_low_confidence(&mut self, min_confidence: f64) -> usize {
        let to_remove: Vec<String> = self.nodes.iter()
            .filter(|(_, n)| n.confidence < min_confidence && n.children.is_empty())
            .map(|(id, _)| id.clone())
            .collect();
        let count = to_remove.len();
        for id in &to_remove {
            if let Some(node) = self.nodes.remove(id) {
                if let Some(ref parent_id) = node.parent_id {
                    if let Some(parent) = self.nodes.get_mut(parent_id) {
                        parent.children.retain(|c| c != id);
                    }
                } else {
                    self.root_ids.retain(|r| r != id);
                }
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_root() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let n = tree.add_root("r1", "Root hypothesis", 0.7);
        assert!(n.is_some());
        assert_eq!(tree.stats().total_nodes, 1);
    }

    #[test]
    fn test_add_child() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Root", 0.7);
        let c = tree.add_child("r1", "c1", "Child", 0.5);
        assert!(c.is_some());
        assert_eq!(tree.stats().total_nodes, 2);
    }

    #[test]
    fn test_depth_of() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Root", 0.7);
        tree.add_child("r1", "c1", "Child", 0.5);
        tree.add_child("c1", "gc1", "Grandchild", 0.3);
        assert_eq!(tree.depth_of("r1"), Some(0));
        assert_eq!(tree.depth_of("c1"), Some(1));
        assert_eq!(tree.depth_of("gc1"), Some(2));
    }

    #[test]
    fn test_subtree() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Root", 0.7);
        tree.add_child("r1", "c1", "Child 1", 0.5);
        tree.add_child("r1", "c2", "Child 2", 0.4);
        let sub = tree.subtree("r1");
        assert_eq!(sub.len(), 3);
    }

    #[test]
    fn test_leaves() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Root", 0.7);
        tree.add_child("r1", "c1", "Leaf 1", 0.5);
        tree.add_child("r1", "c2", "Leaf 2", 0.4);
        assert_eq!(tree.leaves().len(), 2);
    }

    #[test]
    fn test_prune_low_confidence() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Root", 0.7);
        tree.add_child("r1", "c1", "High", 0.6);
        tree.add_child("r1", "c2", "Low", 0.1);
        let pruned = tree.prune_low_confidence(0.3);
        assert_eq!(pruned, 1);
        assert!(tree.get("c2").is_none());
        assert!(tree.get("c1").is_some());
    }

    #[test]
    fn test_update_status() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Test", 0.5);
        assert!(tree.update_status("r1", HypothesisStatus::Verified));
        assert_eq!(tree.get("r1").unwrap().status, HypothesisStatus::Verified);
    }

    #[test]
    fn test_max_depth_enforced() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig { max_depth: 2, min_confidence_for_leaf: 0.0 });
        tree.add_root("r1", "Root", 0.7);
        tree.add_child("r1", "c1", "Child", 0.5);
        let result = tree.add_child("c1", "gc1", "Too deep", 0.3);
        assert!(result.is_some(), "gc1 should be added at depth 2 (max_depth=2)");
        let deeper = tree.add_child("gc1", "ggc1", "Too deep", 0.3);
        assert!(deeper.is_none(), "ggc1 at depth 3 should be rejected");
    }

    #[test]
    fn test_stats() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        tree.add_root("r1", "Root", 0.7);
        tree.add_child("r1", "c1", "Verified", 0.8);
        tree.update_status("c1", HypothesisStatus::Verified);
        let s = tree.stats();
        assert_eq!(s.total_nodes, 2);
        assert_eq!(s.verified, 1);
    }
}
