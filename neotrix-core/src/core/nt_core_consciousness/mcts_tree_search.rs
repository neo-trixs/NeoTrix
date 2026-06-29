
const VSA_DIM: usize = 4096;

#[derive(Debug, Clone)]
pub struct MCTSConfig {
    pub max_iterations: u32,
    pub exploration_constant: f64,
    pub rollout_depth: u32,
    pub max_children: u32,
    pub simulation_budget: u32,
    pub discount_factor: f64,
    pub temperature: f64,
}

impl Default for MCTSConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            exploration_constant: 1.414,
            rollout_depth: 10,
            max_children: 16,
            simulation_budget: 50,
            discount_factor: 0.95,
            temperature: 1.0,
        }
    }
}

pub type NodeId = usize;

#[derive(Debug, Clone)]
pub struct MCTSNode {
    pub id: NodeId,
    pub state: Vec<u8>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub visits: u32,
    pub total_value: f64,
    pub prior: f64,
    pub action_label: String,
    pub depth: u32,
    pub is_terminal: bool,
}

impl MCTSNode {
    pub fn new(id: NodeId, state: Vec<u8>, prior: f64, action_label: String, depth: u32) -> Self {
        Self {
            id,
            state,
            parent: None,
            children: Vec::new(),
            visits: 0,
            total_value: 0.0,
            prior,
            action_label,
            depth,
            is_terminal: false,
        }
    }

    pub fn ucb_value(&self, parent_visits: u32, exploration: f64) -> f64 {
        if self.visits == 0 {
            return f64::MAX;
        }
        let exploitation = self.total_value / self.visits as f64;
        let exploration_term =
            exploration * self.prior * (parent_visits as f64).sqrt() / (1.0 + self.visits as f64);
        exploitation + exploration_term
    }

    pub fn win_rate(&self) -> f64 {
        if self.visits == 0 {
            return 0.0;
        }
        self.total_value / self.visits as f64
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningEdge {
    pub from_action: String,
    pub to_state: Vec<u8>,
    pub transition_prob: f64,
    pub reward: f64,
}

pub trait ReasoningDomain {
    fn get_possible_actions(&self, state: &[u8]) -> Vec<ReasoningEdge>;
    fn evaluate_state(&self, state: &[u8]) -> f64;
    fn is_terminal(&self, state: &[u8]) -> bool;
    fn action_prior(&self, state: &[u8], action: &str) -> f64;
}

#[derive(Debug, Clone)]
pub struct MCTSTree {
    pub nodes: Vec<MCTSNode>,
    pub config: MCTSConfig,
    root: Option<NodeId>,
}

impl MCTSTree {
    pub fn new(config: MCTSConfig) -> Self {
        Self {
            nodes: Vec::new(),
            config,
            root: None,
        }
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn node(&self, id: NodeId) -> Option<&MCTSNode> {
        self.nodes.get(id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut MCTSNode> {
        self.nodes.get_mut(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn total_simulations(&self) -> u32 {
        self.nodes.iter().map(|n| n.visits).sum()
    }

    pub fn add_node(
        &mut self,
        state: Vec<u8>,
        prior: f64,
        action: String,
        parent: Option<NodeId>,
    ) -> NodeId {
        let depth = parent.map_or(0, |p| self.nodes[p].depth + 1);
        let id = self.nodes.len();
        let mut node = MCTSNode::new(id, state, prior, action, depth);
        node.parent = parent;
        if let Some(pid) = parent {
            self.nodes[pid].children.push(id);
        }
        if self.root.is_none() {
            self.root = Some(id);
        }
        self.nodes.push(node);
        id
    }

    pub fn select(&self, _domain: &dyn ReasoningDomain) -> NodeId {
        let mut current = match self.root {
            Some(id) => id,
            None => return 0,
        };
        loop {
            let node = &self.nodes[current];
            if node.children.is_empty() || node.is_terminal {
                return current;
            }
            let parent_visits = node.visits.max(1);
            let best = node
                .children
                .iter()
                .map(|&cid| {
                    let child = &self.nodes[cid];
                    (
                        cid,
                        child.ucb_value(parent_visits, self.config.exploration_constant),
                    )
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(id, _)| id)
                .unwrap_or_else(|| node.children[0]);
            current = best;
        }
    }

    pub fn expand(&mut self, node_id: NodeId, domain: &dyn ReasoningDomain) {
        let state = self.nodes[node_id].state.clone();
        let actions = domain.get_possible_actions(&state);
        for edge in actions.iter().take(self.config.max_children as usize) {
            let prior = domain.action_prior(&state, &edge.from_action);
            let child = self.add_node(
                edge.to_state.clone(),
                prior,
                edge.from_action.clone(),
                Some(node_id),
            );
            self.nodes[child].is_terminal = domain.is_terminal(&self.nodes[child].state);
        }
    }

    pub fn simulate(&self, node_id: NodeId, domain: &dyn ReasoningDomain) -> f64 {
        let mut state = self.nodes[node_id].state.clone();
        let mut total_reward = 0.0;
        let mut gamma = 1.0;

        for _step in 0..self.config.rollout_depth {
            if domain.is_terminal(&state) {
                break;
            }
            let actions = domain.get_possible_actions(&state);
            if actions.is_empty() {
                break;
            }
            let idx = (state.iter().filter(|&&b| b != 0).sum::<u8>() as usize) % actions.len();
            let chosen = &actions[idx];
            total_reward += gamma * chosen.reward;
            gamma *= self.config.discount_factor;
            state = chosen.to_state.clone();
        }

        total_reward + gamma * domain.evaluate_state(&state)
    }

    pub fn backpropagate(&mut self, node_id: NodeId, value: f64) {
        let mut current = node_id;
        let mut discounted = value;
        loop {
            if let Some(node) = self.nodes.get_mut(current) {
                node.visits += 1;
                node.total_value += discounted;
                discounted *= self.config.discount_factor;
                if let Some(parent_id) = node.parent {
                    current = parent_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn search(&mut self, root_state: Vec<u8>, domain: &dyn ReasoningDomain) {
        self.nodes.clear();
        self.root = None;

        let root_prior = domain.evaluate_state(&root_state);
        self.add_node(root_state, root_prior, "root".into(), None);

        for _iter in 0..self.config.max_iterations {
            let leaf = self.select(domain);

            if self.nodes[leaf].visits > 0 {
                self.expand(leaf, domain);
            }

            let sim_node = if self.nodes[leaf].children.is_empty() {
                leaf
            } else {
                let children: Vec<NodeId> = self.nodes[leaf].children.clone();
                let idx = (leaf.wrapping_mul(2654435761) as usize) % children.len();
                children[idx]
            };

            let value = self.simulate(sim_node, domain);
            self.backpropagate(sim_node, value);
        }
    }

    pub fn best_child(&self, temperature: f64) -> Option<NodeId> {
        let root_id = self.root?;
        let root = &self.nodes[root_id];
        if root.children.is_empty() {
            return None;
        }
        if temperature < 0.01 {
            root.children
                .iter()
                .map(|&cid| (cid, self.nodes[cid].visits))
                .max_by_key(|&(_, v)| v)
                .map(|(id, _)| id)
        } else {
            let max_visits = root
                .children
                .iter()
                .map(|&cid| self.nodes[cid].visits)
                .max()
                .unwrap_or(1) as f64;
            let weights: Vec<(NodeId, f64)> = root
                .children
                .iter()
                .map(|&cid| {
                    let w = (self.nodes[cid].visits as f64 / max_visits).powf(1.0 / temperature);
                    (cid, w)
                })
                .collect();
            let total: f64 = weights.iter().map(|(_, w)| w).sum();
            if total <= 0.0 {
                return Some(root.children[0]);
            }
            let mut r = rand::random::<f64>() * total;
            for (id, w) in &weights {
                r -= w;
                if r <= 0.0 {
                    return Some(*id);
                }
            }
            Some(root.children[root.children.len() - 1])
        }
    }

    pub fn best_path(&self, temperature: f64) -> Vec<NodeId> {
        let mut path = Vec::new();
        let mut current = self.root;
        while let Some(cid) = current {
            path.push(cid);
            let node = &self.nodes[cid];
            if node.children.is_empty() {
                break;
            }
            let best = if temperature < 0.01 {
                node.children
                    .iter()
                    .map(|&cid| (cid, self.nodes[cid].visits))
                    .max_by_key(|&(_, v)| v)
                    .map(|(id, _)| id)
            } else {
                let children: Vec<NodeId> = node.children.clone();
                let idx = (cid.wrapping_mul(2654435761) as usize) % children.len();
                Some(children[idx])
            };
            current = best;
        }
        path
    }

    pub fn action_sequence(&self, temperature: f64) -> Vec<(NodeId, String)> {
        let path = self.best_path(temperature);
        path.iter()
            .filter_map(|&id| {
                let node = &self.nodes[id];
                if node.action_label == "root" {
                    None
                } else {
                    Some((id, node.action_label.clone()))
                }
            })
            .collect()
    }

    pub fn to_dot(&self) -> String {
        let mut dot =
            String::from("digraph MCTS {\n  rankdir=TB;\n  node [shape=box,style=filled];\n");
        for node in &self.nodes {
            let label = format!(
                "{}\\n{:.2} ({})",
                node.action_label,
                node.win_rate(),
                node.visits
            );
            let color = if Some(node.id) == self.root {
                "lightblue"
            } else {
                "white"
            };
            dot.push_str(&format!(
                "  n{} [label=\"{}\",fillcolor={}];\n",
                node.id, label, color
            ));
            if let Some(pid) = node.parent {
                dot.push_str(&format!("  n{} -> n{};\n", pid, node.id));
            }
        }
        dot.push_str("}\n");
        dot
    }
}

#[derive(Debug)]
pub struct DefaultReasoningDomain;

impl ReasoningDomain for DefaultReasoningDomain {
    fn get_possible_actions(&self, state: &[u8]) -> Vec<ReasoningEdge> {
        let mut edges = Vec::new();
        let actions = ["hypothesize", "decompose", "verify", "generalize", "refine"];
        for (i, action) in actions.iter().enumerate() {
            let mut next = state.to_vec();
            let flip = (state.iter().filter(|&&b| b != 0).sum::<u8>() as usize + i) % VSA_DIM;
            if flip < next.len() {
                next[flip] ^= 1;
            }
            edges.push(ReasoningEdge {
                from_action: action.to_string(),
                to_state: next,
                transition_prob: 1.0 / actions.len() as f64,
                reward: 0.01 * (i as f64 + 1.0),
            });
        }
        edges
    }

    fn evaluate_state(&self, state: &[u8]) -> f64 {
        let ones = state.iter().filter(|&&b| b != 0).count() as f64;
        ones / state.len() as f64
    }

    fn is_terminal(&self, state: &[u8]) -> bool {
        state.iter().filter(|&&b| b != 0).count() > state.len() / 2
    }

    fn action_prior(&self, _state: &[u8], action: &str) -> f64 {
        match action {
            "hypothesize" => 0.3,
            "decompose" => 0.25,
            "verify" => 0.2,
            "generalize" => 0.15,
            "refine" => 0.1,
            _ => 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::QuantizedVSA;

    fn random_state() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    #[test]
    fn test_mcts_node_creation() {
        let state = random_state();
        let node = MCTSNode::new(0, state.clone(), 0.5, "test".into(), 0);
        assert_eq!(node.id, 0);
        assert_eq!(node.state, state);
        assert_eq!(node.visits, 0);
    }

    #[test]
    fn test_mcts_tree_create_root() {
        let mut tree = MCTSTree::new(MCTSConfig::default());
        let state = random_state();
        tree.add_node(state.clone(), 0.5, "root".into(), None);
        assert_eq!(tree.node_count(), 1);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_mcts_add_child() {
        let mut tree = MCTSTree::new(MCTSConfig::default());
        let root_state = random_state();
        let root = tree.add_node(root_state, 0.5, "root".into(), None);
        let child_state = random_state();
        let child = tree.add_node(child_state, 0.3, "hypothesize".into(), Some(root));
        assert_eq!(tree.node_count(), 2);
        assert_eq!(tree.node(root).unwrap().children.len(), 1);
        assert_eq!(tree.node(child).unwrap().parent, Some(root));
    }

    #[test]
    fn test_mcts_search_runs() {
        let mut tree = MCTSTree::new(MCTSConfig {
            max_iterations: 50,
            ..Default::default()
        });
        let domain = DefaultReasoningDomain;
        tree.search(random_state(), &domain);
        assert!(tree.node_count() > 1);
        assert!(tree.total_simulations() > 0);
    }

    #[test]
    fn test_mcts_best_child() {
        let mut tree = MCTSTree::new(MCTSConfig {
            max_iterations: 100,
            ..Default::default()
        });
        let domain = DefaultReasoningDomain;
        tree.search(random_state(), &domain);
        let best = tree.best_child(0.0);
        assert!(best.is_some());
        let node = tree.node(best.unwrap()).unwrap();
        assert!(node.visits > 0);
    }

    #[test]
    fn test_mcts_action_sequence() {
        let mut tree = MCTSTree::new(MCTSConfig {
            max_iterations: 100,
            ..Default::default()
        });
        let domain = DefaultReasoningDomain;
        tree.search(random_state(), &domain);
        let actions = tree.action_sequence(0.0);
        for (_, action) in &actions {
            assert!(!action.is_empty());
            assert_ne!(action.as_str(), "root");
        }
    }

    #[test]
    fn test_ucb_value_untouched_node() {
        let state = random_state();
        let node = MCTSNode::new(0, state, 0.5, "test".into(), 0);
        let ucb = node.ucb_value(100, 1.414);
        assert_eq!(ucb, f64::MAX);
    }

    #[test]
    fn test_win_rate() {
        let mut node = MCTSNode::new(0, random_state(), 0.5, "test".into(), 0);
        node.visits = 10;
        node.total_value = 7.0;
        assert!((node.win_rate() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_backpropagate() {
        let mut tree = MCTSTree::new(MCTSConfig::default());
        let root = tree.add_node(random_state(), 0.5, "root".into(), None);
        let child = tree.add_node(random_state(), 0.3, "action".into(), Some(root));
        tree.backpropagate(child, 1.0);
        assert_eq!(tree.node(child).unwrap().visits, 1);
        assert!((tree.node(child).unwrap().total_value - 1.0).abs() < 1e-6);
        assert_eq!(tree.node(root).unwrap().visits, 1);
    }

    #[test]
    fn test_default_domain() {
        let domain = DefaultReasoningDomain;
        let state = random_state();
        let actions = domain.get_possible_actions(&state);
        assert_eq!(actions.len(), 5);
        assert!(!domain.is_terminal(&vec![0; VSA_DIM]));
        assert!(domain.is_terminal(&vec![1; VSA_DIM]));
    }

    #[test]
    fn test_mcts_to_dot() {
        let mut tree = MCTSTree::new(MCTSConfig {
            max_iterations: 30,
            ..Default::default()
        });
        let domain = DefaultReasoningDomain;
        tree.search(random_state(), &domain);
        let dot = tree.to_dot();
        assert!(dot.starts_with("digraph"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_mcts_exploration_vs_exploitation() {
        let mut tree = MCTSTree::new(MCTSConfig {
            max_iterations: 200,
            exploration_constant: 5.0,
            ..Default::default()
        });
        let domain = DefaultReasoningDomain;
        tree.search(random_state(), &domain);
        assert!(tree.node_count() > 2);
        let actions = tree.action_sequence(1.0);
        assert!(!actions.is_empty());
    }
}
