use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::{HashMap, HashSet};
use std::f64::consts::SQRT_2;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum HypothesisStatus {
    Active,
    Pruned,
    Verified,
    Incorporated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HypothesisNode {
    pub id: u64,
    pub name: String,
    pub hypothesis: Vec<u8>,
    pub insights: Vec<Vec<u8>>,
    pub status: HypothesisStatus,
    pub depth: usize,
    pub parent_id: Option<u64>,
    pub children: Vec<u64>,
    pub constraints_frontier: Vec<Vec<u8>>,
    pub dev_score: f64,
    pub test_score: f64,
    pub visit_count: usize,
    pub observation_ids: Vec<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HypothesisTreeConfig {
    pub branch_factor: usize,
    pub similarity_threshold: f64,
    pub explore_constant: f64,
    pub max_nodes: usize,
    pub prune_after_verified: bool,
}

impl Default for HypothesisTreeConfig {
    fn default() -> Self {
        Self {
            branch_factor: 3,
            similarity_threshold: 0.65,
            explore_constant: SQRT_2,
            max_nodes: 500,
            prune_after_verified: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HypothesisTree {
    pub nodes: HashMap<u64, HypothesisNode>,
    pub root_id: Option<u64>,
    next_id: u64,
    pub config: HypothesisTreeConfig,
}

impl HypothesisTree {
    pub fn new(config: HypothesisTreeConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: None,
            next_id: 1,
            config,
        }
    }

    fn random_vsa(&self) -> Vec<u8> {
        QuantizedVSA::seeded_random(self.next_id, 4096)
    }

    pub fn add_hypothesis(&mut self, name: &str, parent_id: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let hypothesis = self.random_vsa();
        let depth = parent_id
            .and_then(|p| self.nodes.get(&p))
            .map(|p| p.depth + 1)
            .unwrap_or(0);
        let node = HypothesisNode {
            id,
            name: name.to_string(),
            hypothesis,
            insights: Vec::new(),
            status: HypothesisStatus::Active,
            depth,
            parent_id,
            children: Vec::new(),
            constraints_frontier: Vec::new(),
            dev_score: 0.0,
            test_score: 0.0,
            visit_count: 0,
            observation_ids: Vec::new(),
        };
        if let Some(parent) = parent_id.and_then(|p| self.nodes.get_mut(&p)) {
            parent.children.push(id);
        }
        if parent_id.is_none() {
            self.root_id = Some(id);
        }
        self.nodes.insert(id, node);
        id
    }

    pub fn add_insight(&mut self, node_id: u64, insight: &[u8]) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            let threshold = self.config.similarity_threshold;
            let is_dup = node
                .insights
                .iter()
                .any(|existing| QuantizedVSA::similarity(existing, insight) >= threshold);
            if !is_dup {
                node.insights.push(insight.to_vec());
            }
        }
    }

    pub fn add_constraint(&mut self, node_id: u64, constraint: &[u8]) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.constraints_frontier.push(constraint.to_vec());
        }
    }

    pub fn record_outcome(&mut self, node_id: u64, dev_score: f64, test_score: f64) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.dev_score = dev_score;
            node.test_score = test_score;
            node.visit_count += 1;
            if test_score >= 0.8 {
                node.status = HypothesisStatus::Verified;
            }
        }
    }

    pub fn prune(&mut self, node_id: u64) {
        let mut to_remove: Vec<u64> = Vec::new();
        let mut stack: Vec<u64> = vec![node_id];
        while let Some(id) = stack.pop() {
            if let Some(node) = self.nodes.get(&id) {
                for child in &node.children {
                    stack.push(*child);
                }
            }
            to_remove.push(id);
        }
        for id in &to_remove {
            self.nodes.remove(id);
        }
        let ids: HashSet<u64> = to_remove.iter().copied().collect();
        for node in self.nodes.values_mut() {
            node.children.retain(|c| !ids.contains(c));
        }
    }

    pub fn incorporate(&mut self, node_id: u64) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.status = HypothesisStatus::Incorporated;
        }
    }

    pub fn propagate_insights(&mut self, node_id: u64) {
        let insight_acc = match self.nodes.get(&node_id) {
            Some(n) if !n.insights.is_empty() => {
                let bundled = QuantizedVSA::bundle(
                    &n.insights
                        .iter()
                        .map(|v| v.as_slice())
                        .collect::<Vec<&[u8]>>(),
                );
                Some(bundled)
            }
            _ => None,
        };
        let insight_acc = match insight_acc {
            Some(v) => v,
            None => return,
        };
        let ancestors: Vec<u64> = {
            let mut chain = Vec::new();
            let mut current = node_id;
            loop {
                let parent = match self.nodes.get(&current) {
                    Some(n) => n.parent_id,
                    None => break,
                };
                match parent {
                    Some(p) => {
                        chain.push(p);
                        current = p;
                    }
                    None => break,
                }
            }
            chain
        };
        for ancestor_id in ancestors {
            if let Some(ancestor) = self.nodes.get_mut(&ancestor_id) {
                let already_present = ancestor.insights.iter().any(|iv| {
                    QuantizedVSA::similarity(iv, &insight_acc) >= self.config.similarity_threshold
                });
                if !already_present {
                    ancestor.insights.push(insight_acc.clone());
                }
            }
        }
    }

    pub fn get_constraints_block(&self, node_id: u64) -> Vec<Vec<u8>> {
        let node = match self.nodes.get(&node_id) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let mut constraints = node.constraints_frontier.clone();
        let mut current = node.parent_id;
        while let Some(pid) = current {
            if let Some(parent) = self.nodes.get(&pid) {
                for c in &parent.constraints_frontier {
                    let is_dup = constraints.iter().any(|existing| {
                        QuantizedVSA::similarity(existing, c) >= self.config.similarity_threshold
                    });
                    if !is_dup {
                        constraints.push(c.clone());
                    }
                }
                current = parent.parent_id;
            } else {
                break;
            }
        }
        constraints
    }

    pub fn select_frontier(&self) -> Option<u64> {
        let active_ids: Vec<u64> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.status == HypothesisStatus::Active)
            .map(|(id, _)| *id)
            .collect();
        if active_ids.is_empty() {
            return None;
        }
        let total_visits: usize = active_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .map(|n| n.visit_count)
            .sum();
        let total_visits_f = total_visits as f64;
        let best_id = active_ids
            .iter()
            .max_by(|&&a, &&b| {
                let na = self.nodes.get(&a).unwrap();
                let nb = self.nodes.get(&b).unwrap();
                let ucb_a = na.dev_score
                    + self.config.explore_constant
                        * (total_visits_f.ln() / (na.visit_count.max(1) as f64)).sqrt();
                let ucb_b = nb.dev_score
                    + self.config.explore_constant
                        * (total_visits_f.ln() / (nb.visit_count.max(1) as f64)).sqrt();
                ucb_a
                    .partial_cmp(&ucb_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied();
        best_id
    }

    pub fn get_leaves(&self) -> Vec<u64> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.children.is_empty() && n.status == HypothesisStatus::Active)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Search for the best candidate hypotheses.
    /// Returns (node_id, name, dev_score, test_score) sorted by composite score descending.
    /// Verified nodes first (by test_score), then active leaf nodes (by UCB score).
    pub fn search(&self, max_results: usize) -> Vec<(u64, String, f64, f64)> {
        let mut results: Vec<(u64, String, f64, f64)> = Vec::new();

        // 1. Verified nodes sorted by test_score descending
        let mut verified: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.status == HypothesisStatus::Verified)
            .map(|(id, n)| (*id, n.name.clone(), n.dev_score, n.test_score))
            .collect();
        verified.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        results.extend(verified);

        // 2. Active leaf nodes sorted by UCB score descending
        let total_visits: usize = self.nodes.values().map(|n| n.visit_count).sum();
        let total_visits_f = total_visits as f64;
        let mut active: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.status == HypothesisStatus::Active && n.children.is_empty())
            .map(|(id, n)| {
                let ucb = n.dev_score
                    + self.config.explore_constant
                        * (total_visits_f.ln() / (n.visit_count.max(1) as f64)).sqrt();
                (*id, n.name.clone(), n.dev_score, ucb)
            })
            .collect();
        active.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        results.extend(active);

        results.truncate(max_results);
        results
    }

    pub fn stat_summary(&self) -> HypothesisTreeStats {
        let total = self.nodes.len();
        let active = self
            .nodes
            .values()
            .filter(|n| n.status == HypothesisStatus::Active)
            .count();
        let pruned = self
            .nodes
            .values()
            .filter(|n| n.status == HypothesisStatus::Pruned)
            .count();
        let verified = self
            .nodes
            .values()
            .filter(|n| n.status == HypothesisStatus::Verified)
            .count();
        let incorporated = self
            .nodes
            .values()
            .filter(|n| n.status == HypothesisStatus::Incorporated)
            .count();
        let total_depth: usize = self.nodes.values().map(|n| n.depth).sum();
        let avg_depth = if total > 0 {
            total_depth as f64 / total as f64
        } else {
            0.0
        };
        let max_depth = self.nodes.values().map(|n| n.depth).max().unwrap_or(0);
        let total_insights: usize = self.nodes.values().map(|n| n.insights.len()).sum();
        HypothesisTreeStats {
            total_nodes: total,
            active_nodes: active,
            pruned_nodes: pruned,
            verified_nodes: verified,
            incorporated_nodes: incorporated,
            avg_depth,
            max_depth,
            total_insights,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HypothesisTreeStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub pruned_nodes: usize,
    pub verified_nodes: usize,
    pub incorporated_nodes: usize,
    pub avg_depth: f64,
    pub max_depth: usize,
    pub total_insights: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_root_hypothesis() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let id = tree.add_hypothesis("root", None);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.root_id, Some(id));
        assert_eq!(tree.nodes[&id].depth, 0);
    }

    #[test]
    fn test_add_child_hypothesis() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let root = tree.add_hypothesis("root", None);
        let child = tree.add_hypothesis("child", Some(root));
        assert_eq!(tree.nodes.len(), 2);
        assert_eq!(tree.nodes[&child].depth, 1);
        assert_eq!(tree.nodes[&child].parent_id, Some(root));
        assert_eq!(tree.nodes[&root].children, vec![child]);
    }

    #[test]
    fn test_add_insight_dedup() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let id = tree.add_hypothesis("root", None);
        let v = vec![1u8; 128];
        tree.add_insight(id, &v);
        tree.add_insight(id, &v);
        assert_eq!(tree.nodes[&id].insights.len(), 1);
    }

    #[test]
    fn test_record_outcome_and_verify() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let id = tree.add_hypothesis("root", None);
        tree.record_outcome(id, 0.7, 0.85);
        assert_eq!(tree.nodes[&id].test_score, 0.85);
        assert_eq!(tree.nodes[&id].visit_count, 1);
        assert_eq!(tree.nodes[&id].status, HypothesisStatus::Verified);
    }

    #[test]
    fn test_record_outcome_not_verified_below_threshold() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let id = tree.add_hypothesis("root", None);
        tree.record_outcome(id, 0.5, 0.6);
        assert_eq!(tree.nodes[&id].status, HypothesisStatus::Active);
    }

    #[test]
    fn test_prune_removes_subtree() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let root = tree.add_hypothesis("root", None);
        let child = tree.add_hypothesis("child", Some(root));
        let _grandchild = tree.add_hypothesis("grandchild", Some(child));
        tree.prune(root);
        assert!(tree.nodes.is_empty());
    }

    #[test]
    fn test_propagate_insights_upwards() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let root = tree.add_hypothesis("root", None);
        let child = tree.add_hypothesis("child", Some(root));
        let insight = vec![42u8; 64];
        tree.add_insight(child, &insight);
        tree.propagate_insights(child);
        assert!(!tree.nodes[&root].insights.is_empty());
    }

    #[test]
    fn test_get_constraints_block_inherits_ancestors() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let root = tree.add_hypothesis("root", None);
        let child = tree.add_hypothesis("child", Some(root));
        let c_root = vec![1u8; 64];
        let c_child = vec![2u8; 64];
        tree.add_constraint(root, &c_root);
        tree.add_constraint(child, &c_child);
        let block = tree.get_constraints_block(child);
        assert_eq!(block.len(), 2);
    }

    #[test]
    fn test_select_frontier_ucb() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let a = tree.add_hypothesis("a", None);
        let b = tree.add_hypothesis("b", None);
        tree.record_outcome(a, 0.9, 0.6);
        tree.record_outcome(b, 0.5, 0.4);
        let selected = tree.select_frontier();
        assert!(selected.is_some());
    }

    #[test]
    fn test_select_frontier_empty() {
        let tree = HypothesisTree::new(HypothesisTreeConfig::default());
        assert!(tree.select_frontier().is_none());
    }

    #[test]
    fn test_get_leaves() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let root = tree.add_hypothesis("root", None);
        let _child = tree.add_hypothesis("child", Some(root));
        let leaves = tree.get_leaves();
        assert_eq!(leaves.len(), 1);
    }

    #[test]
    fn test_incorporate_node() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let id = tree.add_hypothesis("root", None);
        tree.incorporate(id);
        assert_eq!(tree.nodes[&id].status, HypothesisStatus::Incorporated);
    }

    #[test]
    fn test_stat_summary() {
        let mut tree = HypothesisTree::new(HypothesisTreeConfig::default());
        let a = tree.add_hypothesis("a", None);
        let b = tree.add_hypothesis("b", Some(a));
        tree.record_outcome(b, 0.9, 0.85);
        let stats = tree.stat_summary();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.verified_nodes, 1);
        assert_eq!(stats.max_depth, 1);
    }
}
