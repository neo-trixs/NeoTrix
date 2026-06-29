use std::collections::HashMap;

const MAX_TREE_DEPTH: usize = 12;
const DEFAULT_EXPLORATION_CONST: f64 = 1.414;
const HYPOTHESIS_BIAS: f64 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Hypothesis,
    Execution,
    Root,
}

#[derive(Debug, Clone)]
pub struct MctsNode {
    pub id: u64,
    pub node_type: NodeType,
    pub state_vsa: Vec<f64>,
    pub visits: u64,
    pub cumulative_value: f64,
    pub depth: usize,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
    pub failure_attribution: Option<FailureAttribution>,
}

#[derive(Debug, Clone, Copy)]
pub struct FailureAttribution {
    pub hypothesis_quality: f64,
    pub execution_quality: f64,
}

#[derive(Debug, Clone)]
pub struct MctsTree {
    pub nodes: HashMap<u64, MctsNode>,
    pub root_id: u64,
    next_id: u64,
    exploration_const: f64,
    node_count: u64,
    hyp_failures: u64,
    exec_failures: u64,
}

impl MctsTree {
    pub fn new(root_state: Vec<f64>) -> Self {
        Self::with_exploration(root_state, DEFAULT_EXPLORATION_CONST)
    }

    pub fn with_exploration(root_state: Vec<f64>, exploration_const: f64) -> Self {
        let root = MctsNode {
            id: 0,
            node_type: NodeType::Root,
            state_vsa: root_state,
            visits: 1,
            cumulative_value: 0.5,
            depth: 0,
            parent: None,
            children: Vec::new(),
            failure_attribution: None,
        };
        let mut nodes = HashMap::new();
        nodes.insert(0, root);
        Self {
            nodes,
            root_id: 0,
            next_id: 1,
            exploration_const,
            node_count: 1,
            hyp_failures: 0,
            exec_failures: 0,
        }
    }

    pub fn add_hypothesis(&mut self, parent_id: u64, state: Vec<f64>) -> Option<u64> {
        let parent = self.nodes.get(&parent_id)?;
        if parent.depth >= MAX_TREE_DEPTH {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        let node = MctsNode {
            id,
            node_type: NodeType::Hypothesis,
            state_vsa: state,
            visits: 0,
            cumulative_value: 0.0,
            depth: parent.depth + 1,
            parent: Some(parent_id),
            children: Vec::new(),
            failure_attribution: None,
        };
        self.node_count += 1;
        self.nodes.insert(id, node);
        if let Some(p) = self.nodes.get_mut(&parent_id) {
            p.children.push(id);
        }
        Some(id)
    }

    pub fn add_execution(&mut self, parent_id: u64, state: Vec<f64>) -> Option<u64> {
        let parent = self.nodes.get(&parent_id)?;
        let id = self.next_id;
        self.next_id += 1;
        let node = MctsNode {
            id,
            node_type: NodeType::Execution,
            state_vsa: state,
            visits: 0,
            cumulative_value: 0.0,
            depth: parent.depth + 1,
            parent: Some(parent_id),
            children: Vec::new(),
            failure_attribution: None,
        };
        self.node_count += 1;
        self.nodes.insert(id, node);
        if let Some(p) = self.nodes.get_mut(&parent_id) {
            p.children.push(id);
        }
        Some(id)
    }

    pub fn select(&self) -> Option<u64> {
        self.select_from(self.root_id)
    }

    fn select_from(&self, node_id: u64) -> Option<u64> {
        let node = self.nodes.get(&node_id)?;
        if node.children.is_empty() {
            return Some(node_id);
        }
        let mut best_id = node_id;
        let mut best_uct = f64::NEG_INFINITY;
        let parent_visits = 1.max(node.visits);
        for child_id in &node.children {
            if let Some(child) = self.nodes.get(child_id) {
                let exploitation = if child.visits > 0 {
                    child.cumulative_value / child.visits as f64
                } else {
                    0.0
                };
                let exploration = self.exploration_const * (parent_visits as f64).ln().sqrt()
                    / (1.0 + child.visits as f64).sqrt();
                let hypothesis_bonus = if child.node_type == NodeType::Hypothesis {
                    HYPOTHESIS_BIAS
                } else {
                    0.0
                };
                let uct = exploitation + exploration + hypothesis_bonus;
                if uct > best_uct {
                    best_uct = uct;
                    best_id = *child_id;
                }
            }
        }
        if best_id == node_id {
            return Some(node_id);
        }
        self.select_from(best_id)
    }

    pub fn backpropagate(&mut self, node_id: u64, value: f64) {
        let mut current = Some(node_id);
        while let Some(id) = current {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.visits += 1;
                node.cumulative_value += value;
                current = node.parent;
            } else {
                break;
            }
        }
    }

    pub fn record_failure(&mut self, node_id: u64, hyp_quality: f64, exec_quality: f64) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.failure_attribution = Some(FailureAttribution {
                hypothesis_quality: hyp_quality,
                execution_quality: exec_quality,
            });
            if hyp_quality < 0.3 {
                self.hyp_failures += 1;
            }
            if exec_quality < 0.3 {
                self.exec_failures += 1;
            }
        }
    }

    pub fn best_hypothesis(&self) -> Option<u64> {
        self.nodes.values()
            .filter(|n| n.node_type == NodeType::Hypothesis && n.visits > 0)
            .max_by(|a, b| {
                let avg_a = a.cumulative_value / a.visits as f64;
                let avg_b = b.cumulative_value / b.visits as f64;
                avg_a.partial_cmp(&avg_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|n| n.id)
    }

    pub fn dominant_failure_mode(&self) -> &'static str {
        if self.hyp_failures > self.exec_failures * 2 {
            "hypothesis"
        } else if self.exec_failures > self.hyp_failures * 2 {
            "execution"
        } else {
            "mixed"
        }
    }

    pub fn node_count(&self) -> u64 {
        self.node_count
    }

    pub fn summary(&self) -> String {
        format!("MCTS[nodes={} hyp_fail={} exec_fail={} mode={}]",
            self.node_count, self.hyp_failures, self.exec_failures, self.dominant_failure_mode())
    }
}

impl Default for MctsTree {
    fn default() -> Self {
        Self::new(vec![0.5; 8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> MctsTree {
        let mut tree = MctsTree::new(vec![0.5; 8]);
        let h1 = tree.add_hypothesis(0, vec![0.6; 8]).unwrap();
        let e1 = tree.add_execution(h1, vec![0.7; 8]).unwrap();
        tree.backpropagate(e1, 0.8);
        let h2 = tree.add_hypothesis(0, vec![0.4; 8]).unwrap();
        let e2 = tree.add_execution(h2, vec![0.5; 8]).unwrap();
        tree.backpropagate(e2, 0.3);
        tree
    }

    #[test]
    fn test_new_tree() {
        let tree = MctsTree::new(vec![0.5; 8]);
        assert_eq!(tree.node_count(), 1);
        assert_eq!(tree.nodes[&0].node_type, NodeType::Root);
    }

    #[test]
    fn test_add_hypothesis() {
        let mut tree = MctsTree::new(vec![0.5; 8]);
        let id = tree.add_hypothesis(0, vec![0.6; 8]);
        assert!(id.is_some());
        assert_eq!(tree.node_count(), 2);
    }

    #[test]
    fn test_add_execution() {
        let mut tree = MctsTree::new(vec![0.5; 8]);
        let h = tree.add_hypothesis(0, vec![0.6; 8]).unwrap();
        let id = tree.add_execution(h, vec![0.7; 8]);
        assert!(id.is_some());
        assert_eq!(tree.node_count(), 3);
    }

    #[test]
    fn test_backpropagation() {
        let mut tree = sample_tree();
        let root = &tree.nodes[&0];
        assert!(root.visits > 1);
        assert!(root.cumulative_value > 0.0);
    }

    #[test]
    fn test_select_returns_leaf() {
        let tree = sample_tree();
        let selected = tree.select();
        assert!(selected.is_some());
    }

    #[test]
    fn test_record_failure() {
        let mut tree = sample_tree();
        let h = tree.add_hypothesis(0, vec![0.5; 8]).unwrap();
        tree.record_failure(h, 0.2, 0.9);
        assert_eq!(tree.hyp_failures, 1);
        assert_eq!(tree.exec_failures, 0);
    }

    #[test]
    fn test_dominant_failure_mode() {
        let mut tree = MctsTree::new(vec![0.5; 8]);
        for _ in 0..10 {
            let h = tree.add_hypothesis(0, vec![0.5; 8]).unwrap();
            tree.record_failure(h, 0.2, 0.9);
        }
        assert_eq!(tree.dominant_failure_mode(), "hypothesis");
    }

    #[test]
    fn test_best_hypothesis() {
        let tree = sample_tree();
        let best = tree.best_hypothesis();
        assert!(best.is_some());
    }

    #[test]
    fn test_summary_format() {
        let tree = sample_tree();
        let s = tree.summary();
        assert!(s.starts_with("MCTS["));
        assert!(s.contains("nodes="));
    }

    #[test]
    fn test_max_depth_limit() {
        let mut tree = MctsTree::new(vec![0.5; 8]);
        let mut id = 0;
        for _ in 0..MAX_TREE_DEPTH + 5 {
            let next = tree.add_hypothesis(id, vec![0.5; 8]);
            match next {
                Some(nid) => id = nid,
                None => break,
            }
        }
        assert!(tree.node_count() <= MAX_TREE_DEPTH as u64 + 1);
    }
}
