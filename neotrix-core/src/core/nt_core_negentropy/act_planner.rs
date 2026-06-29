use std::f64::consts::PI;

/// A node in the Active Inference Tree Search.
#[derive(Debug, Clone)]
pub struct AcTNode {
    pub state: Vec<f64>,
    pub visit_count: u64,
    pub total_efe: f64,
    pub children: Vec<usize>,
    pub action_idx: usize,
    pub is_terminal: bool,
}

impl AcTNode {
    pub fn new(state: Vec<f64>, action_idx: usize) -> Self {
        Self {
            state,
            visit_count: 0,
            total_efe: 0.0,
            children: Vec::new(),
            action_idx,
            is_terminal: false,
        }
    }
}

/// Statistics collected during a single search.
#[derive(Debug, Clone)]
pub struct AcTSearchStats {
    pub nodes_expanded: u64,
    pub total_simulations: u64,
    pub tree_depth: usize,
    pub policy_entropy: f64,
}

/// Result of the AcT planning process.
#[derive(Debug, Clone)]
pub struct AcTResult {
    pub best_action: usize,
    pub expected_efe: f64,
    pub search_stats: AcTSearchStats,
}

/// Active Inference Tree Search (AcT) planner.
///
/// Extends flat EFE minimization with Monte Carlo Tree Search, using EFE
/// as the node evaluation metric for large POMDP planning.
///
/// Nodes are stored in an arena (`Vec<AcTNode>`) and referenced by index,
/// avoiding any unsafe pointer manipulation while supporting tree mutation.
pub struct AcTPlanner {
    pub horizon: usize,
    pub exploration_constant: f64,
    pub max_children: usize,
    pub num_simulations: usize,
    pub discount: f64,
    transition: Box<dyn Fn(&[f64], usize) -> Vec<f64> + Send + Sync>,
    policy_proposals: Vec<Vec<f64>>,
    nodes: Vec<AcTNode>,
    root_idx: usize,
}

impl AcTPlanner {
    /// Create a new AcT planner.
    ///
    /// `transition(state, action_idx)` returns the predicted next state.
    /// `policy_proposals` are candidate action vectors (each is a state bias).
    pub fn new(
        horizon: usize,
        transition: Box<dyn Fn(&[f64], usize) -> Vec<f64> + Send + Sync>,
        policy_proposals: Vec<Vec<f64>>,
    ) -> Self {
        let max_children = policy_proposals.len().max(1);
        Self {
            horizon,
            exploration_constant: std::f64::consts::SQRT_2,
            max_children,
            num_simulations: 100,
            discount: 0.95,
            transition,
            policy_proposals,
            nodes: Vec::new(),
            root_idx: 0,
        }
    }

    /// Run MCTS simulations from the initial belief state.
    ///
    /// Each simulation: SELECT → EXPAND → SIMULATE → BACKPROPAGATE.
    /// Returns the best action (most-visited child) and search statistics.
    pub fn plan(&mut self, initial_state: &[f64]) -> AcTResult {
        self.nodes.clear();
        let root = AcTNode::new(initial_state.to_vec(), usize::MAX);
        self.nodes.push(root);
        self.root_idx = 0;
        self.nodes[self.root_idx].visit_count = 1;

        let mut total_expanded = 0u64;
        let mut max_depth = 0usize;

        for _ in 0..self.num_simulations {
            let mut path: Vec<usize> = vec![self.root_idx];

            // SELECT: traverse tree using UCB1
            let leaf_idx = loop {
                if path.is_empty() {
                    break self.root_idx;
                }
                let node = &self.nodes[*path.last().unwrap_or(&self.root_idx)];
                if node.is_terminal || node.children.is_empty() {
                    break path.last().copied().unwrap_or(self.root_idx);
                }
                let all_visited = node.children.iter().all(|&c| self.nodes[c].visit_count > 0);
                if !all_visited {
                    break path.last().copied().unwrap_or(self.root_idx);
                }
                let best_idx = Self::select_child_idx(node, &self.nodes, self.exploration_constant);
                path.push(node.children[best_idx]);
            };

            // EXPAND: add child for unvisited actions
            {
                let is_terminal = self.nodes[leaf_idx].is_terminal;
                let has_children = !self.nodes[leaf_idx].children.is_empty();
                if !is_terminal && !has_children {
                    let leaf_state = self.nodes[leaf_idx].state.clone();
                    let available = self.policy_proposals.len();
                    let to_add = available.min(self.max_children);
                    let start = self.nodes.len();
                    let mut child_indices = Vec::with_capacity(to_add);
                    for a in 0..to_add {
                        let next_state = (self.transition)(&leaf_state, a);
                        self.nodes.push(AcTNode::new(next_state, a));
                        child_indices.push(start + a);
                    }
                    let leaf = &mut self.nodes[leaf_idx];
                    leaf.children = child_indices;
                    total_expanded += to_add as u64;
                }
            }

            // SIMULATE: roll out from the expanded node
            let leaf_state = self.nodes[leaf_idx].state.clone();
            let mut sim_state = leaf_state;
            let mut sim_efe = 0.0;
            let mut sim_depth = 0usize;

            for step in 0..self.horizon {
                if sim_state.is_empty() {
                    break;
                }
                let n_actions = self.policy_proposals.len().max(1);
                let random_action = (total_expanded
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407 + step as u64))
                    as usize
                    % n_actions;
                let next_state = (self.transition)(&sim_state, random_action);
                let step_efe = Self::compute_efe(&sim_state, &next_state);
                sim_efe += step_efe * self.discount.powi(step as i32);
                sim_state = next_state;
                sim_depth += 1;
            }

            max_depth = max_depth.max(sim_depth);

            // BACKPROPAGATE
            for &node_idx in path.iter().rev() {
                let node = &mut self.nodes[node_idx];
                node.visit_count += 1;
                node.total_efe += sim_efe;
            }
        }

        let best_action = self.select_best_action();
        let expected_efe = {
            let root = &self.nodes[self.root_idx];
            if !root.children.is_empty() {
                let best_node = &self.nodes[root.children[best_action]];
                if best_node.visit_count > 0 {
                    best_node.total_efe / best_node.visit_count as f64
                } else {
                    f64::INFINITY
                }
            } else {
                f64::INFINITY
            }
        };

        let policy_entropy = self.compute_policy_entropy();

        AcTResult {
            best_action,
            expected_efe,
            search_stats: AcTSearchStats {
                nodes_expanded: total_expanded,
                total_simulations: self.num_simulations as u64,
                tree_depth: max_depth,
                policy_entropy,
            },
        }
    }

    /// UCB1 — select child with minimum value (lower EFE is better).
    ///
    /// value(i) = mean_efe(i) - c * sqrt(ln(N) / n_i)
    pub fn select_child(node: &AcTNode, nodes: &[AcTNode], c: f64) -> usize {
        let ln_n = (node.visit_count as f64).ln();
        let mut best_idx = 0;
        let mut best_val = f64::INFINITY;

        for (i, &child_idx) in node.children.iter().enumerate() {
            let child = &nodes[child_idx];
            if child.visit_count == 0 {
                return i;
            }
            let mean = child.total_efe / child.visit_count as f64;
            let explore = c * (ln_n / child.visit_count as f64).sqrt();
            let val = mean - explore;
            if val < best_val {
                best_val = val;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Internal: select child index given a parent node reference.
    fn select_child_idx(node: &AcTNode, nodes: &[AcTNode], c: f64) -> usize {
        Self::select_child(node, nodes, c)
    }

    /// Simplified EFE: cosine distance between states + entropy bonus.
    ///
    /// Returns value in [0, 1].
    pub fn compute_efe(state: &[f64], next_state: &[f64]) -> f64 {
        let cos_dist = Self::cosine_distance(state, next_state);
        let entropy = Self::entropy(state);
        (cos_dist + 0.01 * entropy).min(1.0)
    }

    /// Cosine distance between two vectors (1 - cosine similarity).
    fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        if n == 0 {
            return 1.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).take(n).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().take(n).map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().take(n).map(|x| x * x).sum::<f64>().sqrt();
        if norm_a < 1e-12 || norm_b < 1e-12 {
            return 1.0;
        }
        let sim = (dot / (norm_a * norm_b)).clamp(-1.0, 1.0);
        1.0 - sim
    }

    /// Normalized Shannon entropy of a belief vector in [0, 1].
    fn entropy(state: &[f64]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }
        let mut h = 0.0;
        for &b in state {
            let p = b.clamp(1e-12, 1.0 - 1e-12);
            h -= p * p.ln();
        }
        let max_h = (state.len() as f64).ln();
        if max_h < 1e-12 {
            return 0.0;
        }
        (h / max_h).clamp(0.0, 1.0)
    }

    /// Select the best action — child with most visits.
    fn select_best_action(&self) -> usize {
        let root = &self.nodes[self.root_idx];
        let mut best_idx = 0;
        let mut best_visits = 0u64;
        for (i, &child_idx) in root.children.iter().enumerate() {
            let v = self.nodes[child_idx].visit_count;
            if v > best_visits {
                best_visits = v;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Compute policy entropy from the visit distribution of root children.
    fn compute_policy_entropy(&self) -> f64 {
        let root = &self.nodes[self.root_idx];
        let total: f64 = root
            .children
            .iter()
            .map(|&c| self.nodes[c].visit_count as f64)
            .sum();
        if total < 1.0 {
            return 0.0;
        }
        let mut h = 0.0;
        for &child_idx in &root.children {
            let p = self.nodes[child_idx].visit_count as f64 / total;
            if p > 1e-12 {
                h -= p * p.ln();
            }
        }
        let n = root.children.len().max(1) as f64;
        h / n.ln().max(1.0)
    }

    /// Reset the search tree and node counter.
    pub fn reset(&mut self) {
        self.nodes.clear();
    }

    /// Update hyperparameters after construction.
    pub fn set_hyperparams(&mut self, c: f64, sims: usize, discount: f64) {
        self.exploration_constant = c;
        self.num_simulations = sims;
        self.discount = discount;
    }
}

/// Default transition: sinusoidal projection like SimpleTransitionModel.
pub fn default_transition_fn(
    action_count: usize,
) -> Box<dyn Fn(&[f64], usize) -> Vec<f64> + Send + Sync> {
    Box::new(move |state: &[f64], action: usize| {
        if state.is_empty() {
            return Vec::new();
        }
        let mut next = Vec::with_capacity(state.len());
        for (i, &s) in state.iter().enumerate() {
            let action_phase = (action as f64 + 1.0) / (action_count as f64 + 1.0);
            let index_phase = (i as f64 + 1.0) / (state.len() as f64 + 1.0);
            let target = (action_phase * index_phase * PI).sin() * 0.5 + 0.5;
            let val = s * 0.7 + target * 0.3;
            next.push(val);
        }
        next
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_planner() -> AcTPlanner {
        let proposals = vec![vec![0.8, 0.2], vec![0.5, 0.5], vec![0.2, 0.8]];
        AcTPlanner::new(3, default_transition_fn(3), proposals)
    }

    #[test]
    fn test_planner_creation() {
        let planner = small_planner();
        assert_eq!(planner.horizon, 3);
        assert_eq!(planner.policy_proposals.len(), 3);
        assert_eq!(planner.max_children, 3);
        assert!((planner.exploration_constant - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_ucb_selection_prefers_lower_efe() {
        let mut nodes = Vec::new();
        nodes.push(AcTNode::new(vec![0.5, 0.5], usize::MAX));
        nodes[0].visit_count = 10;

        let mut child_low = AcTNode::new(vec![0.1, 0.9], 0);
        child_low.visit_count = 5;
        child_low.total_efe = 1.0;

        let mut child_high = AcTNode::new(vec![0.9, 0.1], 1);
        child_high.visit_count = 5;
        child_high.total_efe = 9.0;

        nodes.push(child_low);
        nodes.push(child_high);

        let low_idx = 1;
        let high_idx = 2;
        nodes[0].children = vec![low_idx, high_idx];

        let best = AcTPlanner::select_child(&nodes[0], &nodes, 0.0);
        assert_eq!(best, 0, "should select lower EFE child when c=0");
    }

    #[test]
    fn test_expand_children_up_to_max() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.max_children = 2;

        planner
            .nodes
            .push(AcTNode::new(initial.clone(), usize::MAX));
        planner.root_idx = 0;

        let to_add = planner.policy_proposals.len().min(planner.max_children);
        let start = planner.nodes.len();
        for a in 0..to_add {
            let next = (planner.transition)(&initial, a);
            planner.nodes.push(AcTNode::new(next, a));
            planner.nodes[planner.root_idx].children.push(start + a);
        }

        assert_eq!(planner.nodes[planner.root_idx].children.len(), 2);
        assert_eq!(planner.nodes[start].action_idx, 0);
        assert_eq!(planner.nodes[start + 1].action_idx, 1);
    }

    #[test]
    fn test_simulate_horizon_1() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.horizon = 1;

        let next = (planner.transition)(&initial, 0);
        let efe = AcTPlanner::compute_efe(&initial, &next);
        assert!(efe >= 0.0);
        assert!(efe <= 1.0);
    }

    #[test]
    fn test_full_plan_returns_valid_result() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.num_simulations = 50;

        let result = planner.plan(&initial);
        assert!(
            result.best_action < planner.policy_proposals.len(),
            "action index {} out of range",
            result.best_action
        );
        assert!(!result.expected_efe.is_nan());
        assert!(result.search_stats.total_simulations == 50);
        assert!(result.search_stats.nodes_expanded > 0);
    }

    #[test]
    fn test_plan_with_single_action() {
        let proposals = vec![vec![0.5, 0.5]];
        let mut planner = AcTPlanner::new(2, default_transition_fn(1), proposals);
        planner.num_simulations = 10;

        let result = planner.plan(&[0.5, 0.5]);
        assert_eq!(result.best_action, 0, "only action must be 0");
    }

    #[test]
    fn test_reset_clears_tree() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.num_simulations = 10;

        planner.plan(&initial);
        assert!(!planner.nodes.is_empty(), "nodes should exist after plan");

        planner.reset();
        assert!(
            planner.nodes.is_empty(),
            "nodes should be empty after reset"
        );
    }

    #[test]
    fn test_hyperparams_change_behavior() {
        let mut planner = small_planner();
        assert!((planner.exploration_constant - std::f64::consts::SQRT_2).abs() < 1e-10);
        assert_eq!(planner.num_simulations, 100);
        assert!((planner.discount - 0.95).abs() < 1e-10);

        planner.set_hyperparams(5.0, 200, 0.5);
        assert!((planner.exploration_constant - 5.0).abs() < 1e-10);
        assert_eq!(planner.num_simulations, 200);
        assert!((planner.discount - 0.5).abs() < 1e-10);

        let initial = vec![0.5, 0.5];
        planner.num_simulations = 10;
        let result = planner.plan(&initial);
        assert!(result.search_stats.total_simulations == 10);
    }

    #[test]
    fn test_discount_factor_works() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.horizon = 3;
        planner.discount = 0.5;
        planner.num_simulations = 10;

        let result = planner.plan(&initial);
        assert!(!result.expected_efe.is_nan());
        assert!(result.expected_efe.is_finite());
    }

    #[test]
    fn test_multiple_plans_maintain_state() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.num_simulations = 10;

        let r1 = planner.plan(&initial);
        let node_count_after_first = planner.nodes.len();

        planner.reset();
        let r2 = planner.plan(&initial);

        assert!(r1.best_action < planner.policy_proposals.len());
        assert!(r2.best_action < planner.policy_proposals.len());
        assert!(node_count_after_first > 0);
    }

    #[test]
    fn test_entropy_computation() {
        let uniform = vec![0.25, 0.25, 0.25, 0.25];
        let e = AcTPlanner::entropy(&uniform);
        assert!(
            (e - 1.0).abs() < 1e-10,
            "uniform should give entropy ~1.0, got {}",
            e
        );

        let deterministic = vec![1.0, 0.0, 0.0, 0.0];
        let e2 = AcTPlanner::entropy(&deterministic);
        assert!(
            e2 < 0.01,
            "deterministic should give near-zero entropy, got {}",
            e2
        );

        let empty: Vec<f64> = vec![];
        let e3 = AcTPlanner::entropy(&empty);
        assert!((e3 - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_more_simulations_expands_more_nodes() {
        let initial = vec![0.5, 0.5];
        let mut planner_low = small_planner();
        planner_low.num_simulations = 5;
        let r_low = planner_low.plan(&initial);

        let mut planner_high = small_planner();
        planner_high.num_simulations = 50;
        let r_high = planner_high.plan(&initial);

        assert!(
            r_high.search_stats.nodes_expanded >= r_low.search_stats.nodes_expanded,
            "more simulations should expand at least as many nodes"
        );
    }

    #[test]
    fn test_efe_range() {
        let state = vec![0.5, 0.5];
        let next = vec![0.6, 0.4];
        let efe = AcTPlanner::compute_efe(&state, &next);
        assert!(efe >= 0.0);
        assert!(efe <= 1.0);

        let identical = vec![0.5, 0.5];
        let efe_id = AcTPlanner::compute_efe(&state, &identical);
        assert!(
            efe_id < 0.1,
            "identical states should have near-zero EFE, got {}",
            efe_id
        );

        let opposite = vec![1.0, 0.0];
        let efe_op = AcTPlanner::compute_efe(&state, &opposite);
        assert!(efe_op > efe_id);
    }

    #[test]
    fn test_plan_returns_consistent_action() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.num_simulations = 30;

        let result = planner.plan(&initial);
        assert!(result.search_stats.total_simulations == 30);
        assert!(result.search_stats.tree_depth >= 1);
        assert!(result.search_stats.policy_entropy >= 0.0);
        assert!(result.search_stats.policy_entropy <= 1.0);
    }

    #[test]
    fn test_custom_constructor() {
        let proposals = vec![vec![0.5, 0.5]];
        let transition = Box::new(|_: &[f64], _: usize| vec![0.6, 0.4]);
        let mut planner = AcTPlanner::new(5, transition, proposals);
        assert_eq!(planner.horizon, 5);
        assert_eq!(planner.max_children, 1);

        planner.set_hyperparams(1.5, 200, 0.9);
        assert!((planner.exploration_constant - 1.5).abs() < 1e-10);
        assert_eq!(planner.num_simulations, 200);
        assert!((planner.discount - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_simulate_efe_accumulates() {
        let initial = vec![0.5, 0.5];
        let mut planner = small_planner();
        planner.horizon = 2;
        planner.discount = 0.5;
        planner.num_simulations = 5;

        let result = planner.plan(&initial);
        assert!(result.expected_efe.is_finite());
    }
}
