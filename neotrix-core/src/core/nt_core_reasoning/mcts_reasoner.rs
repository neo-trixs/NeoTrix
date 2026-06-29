use std::collections::{HashMap, HashSet, VecDeque};

use super::vsa_blackboard::{ExpertType, Hypothesis, VsaBlackboard};
use crate::core::nt_core_hcube::QuantizedVSA;
use crate::core::unix_now_ms;

#[derive(Debug, Clone)]
pub struct MctsConfig {
    pub max_simulations: usize,
    pub exploration_constant: f64,
    pub max_depth: usize,
    pub reward_discount: f64,
    pub expansion_budget: usize,
    pub confidence_threshold: f64,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            max_simulations: 30,
            exploration_constant: 1.414,
            max_depth: 10,
            reward_discount: 0.95,
            expansion_budget: 5,
            confidence_threshold: 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MctsNode {
    pub id: u64,
    pub state_id: u64,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
    pub visits: usize,
    pub total_value: f64,
    pub prior: f64,
    pub action_label: String,
    pub depth: usize,
    pub is_terminal: bool,
    pub reasoning_path: Vec<Hypothesis>,
}

#[derive(Debug, Clone)]
pub struct MctsReasoner {
    pub config: MctsConfig,
    pub nodes: HashMap<u64, MctsNode>,
    pub root_id: u64,
    pub next_node_id: u64,
    pub blackboard: VsaBlackboard,
    pub prm_scores: VecDeque<f64>,
    shared_memory: HashMap<u64, f64>,
}

#[derive(Debug, Clone)]
pub struct MctsStats {
    pub total_nodes: usize,
    pub root_visits: usize,
    pub best_value: f64,
    pub prm_mean: f64,
    pub prm_std: f64,
    pub exploration_efficiency: f64,
}

impl MctsReasoner {
    pub fn new(config: MctsConfig) -> Self {
        let blackboard = VsaBlackboard::new(256);
        Self {
            config,
            nodes: HashMap::new(),
            root_id: 0,
            next_node_id: 1,
            blackboard,
            prm_scores: VecDeque::with_capacity(100),
            shared_memory: HashMap::new(),
        }
    }

    pub fn search(&mut self, initial_state: Hypothesis) -> Vec<Hypothesis> {
        self.nodes.clear();
        self.blackboard.clear();
        let root = MctsNode {
            id: self.next_node_id,
            state_id: initial_state.id,
            parent: None,
            children: Vec::new(),
            visits: 0,
            total_value: 0.0,
            prior: initial_state.confidence,
            action_label: "root".into(),
            depth: 0,
            is_terminal: false,
            reasoning_path: vec![initial_state.clone()],
        };
        self.root_id = root.id;
        self.next_node_id += 1;
        self.nodes.insert(root.id, root);

        for _ in 0..self.config.max_simulations {
            let leaf = self.select(self.root_id);
            let expanded = self.expand(leaf);
            let reward = self.simulate(leaf);
            self.backpropagate(leaf, reward);
            for child_id in expanded {
                self.backpropagate(child_id, reward * self.config.reward_discount);
            }
        }

        self.prune_low_confidence();
        self.best_path(self.root_id)
    }

    fn select(&self, node_id: u64) -> u64 {
        let mut current = node_id;
        loop {
            let node = self.nodes.get(&current).expect("select: missing node");
            if node.is_terminal || node.children.is_empty() {
                return current;
            }

            let exploration = self.config.exploration_constant / (1.0 + node.depth as f64 * 0.1);
            let best_child = node
                .children
                .iter()
                .max_by(|&&a, &&b| {
                    let score_a = self.uct_score(a, exploration);
                    let score_b = self.uct_score(b, exploration);
                    score_a
                        .partial_cmp(&score_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied();

            match best_child {
                Some(id) => current = id,
                None => return current,
            }
        }
    }

    fn uct_score(&self, child_id: u64, exploration: f64) -> f64 {
        let child = self.nodes.get(&child_id).expect("uct_score: missing child");
        if child.visits == 0 {
            return f64::MAX;
        }
        let parent = child
            .parent
            .and_then(|pid| self.nodes.get(&pid))
            .expect("uct_score: missing parent");
        let exploitation = child.total_value / child.visits as f64;
        let exploration_term =
            exploration * (parent.visits as f64).ln().sqrt() / (1.0 + child.visits as f64).sqrt();
        exploitation + exploration_term * child.prior
    }

    fn expand(&mut self, node_id: u64) -> Vec<u64> {
        let (depth, state_id, _is_terminal, parent_path) = {
            let node = self.nodes.get(&node_id).expect("expand: missing node");
            if node.depth >= self.config.max_depth || node.is_terminal {
                return Vec::new();
            }
            (
                node.depth + 1,
                node.state_id,
                node.is_terminal,
                node.reasoning_path.clone(),
            )
        };

        let mut candidates: Vec<(f64, ExpertType, String)> = Vec::new();
        for expert in &[
            ExpertType::Analogical,
            ExpertType::Causal,
            ExpertType::MultiHop,
            ExpertType::Contradiction,
            ExpertType::Synthesis,
        ] {
            for _ in 0..3 {
                let prior = self.heuristic_prior_for_expert(expert, depth);
                if prior > self.config.confidence_threshold {
                    let label = format!("{:?}:{}", expert, node_id);
                    candidates.push((prior, *expert, label));
                }
            }
        }

        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(self.config.expansion_budget);

        let mut child_ids = Vec::new();
        for (prior, _expert, label) in candidates {
            let child_id = self.next_node_id;
            self.next_node_id += 1;

            let hypothesis = Hypothesis {
                id: unix_now_ms().wrapping_mul(child_id),
                content: QuantizedVSA::random_vector(),
                confidence: prior,
                expert: _expert,
                supporting_evidence: vec![state_id],
                created_at: unix_now_ms(),
                is_contradicted: false,
            };

            let mut path = parent_path.clone();
            path.push(hypothesis.clone());

            let child = MctsNode {
                id: child_id,
                state_id: hypothesis.id,
                parent: Some(node_id),
                children: Vec::new(),
                visits: 0,
                total_value: 0.0,
                prior,
                action_label: label,
                depth,
                is_terminal: depth >= self.config.max_depth,
                reasoning_path: path,
            };
            self.nodes.insert(child_id, child);
            child_ids.push(child_id);

            let prm_score = self.prm_evaluate(&hypothesis);
            self.prm_scores.push_back(prm_score);
            if self.prm_scores.len() > 100 {
                self.prm_scores.pop_front();
            }
        }

        if let Some(n) = self.nodes.get_mut(&node_id) {
            n.children.extend(&child_ids);
            if child_ids.is_empty() {
                n.is_terminal = true;
            }
        }

        child_ids
    }

    fn simulate(&self, node_id: u64) -> f64 {
        let node = self.nodes.get(&node_id).expect("simulate: missing node");

        let coherence = self.vsa_coherence_static(&node.reasoning_path);
        let depth_bonus = if node.depth > 0 {
            (node.depth as f64).ln_1p() * 0.05
        } else {
            0.0
        };
        let prior_confidence = node.prior;
        let shared_bonus = self
            .shared_memory
            .get(&node.state_id)
            .copied()
            .unwrap_or(0.0)
            * 0.1;

        let base = (coherence + prior_confidence + shared_bonus) / 3.0;
        (base + depth_bonus).clamp(0.0, 1.0)
    }

    fn vsa_coherence_static(&self, path: &[Hypothesis]) -> f64 {
        if path.len() < 2 {
            return 1.0;
        }
        let mut total_sim = 0.0;
        let mut count = 0;
        for w in path.windows(2) {
            let sim = QuantizedVSA::similarity(&w[0].content, &w[1].content);
            total_sim += sim;
            count += 1;
        }
        let mean_sim = total_sim / count as f64;
        let consistency_penalty = if mean_sim > 0.95 {
            (mean_sim - 0.95) * 2.0
        } else {
            0.0
        };
        (mean_sim - consistency_penalty).clamp(0.0, 1.0)
    }

    fn backpropagate(&mut self, node_id: u64, reward: f64) {
        let mut current = Some(node_id);
        let mut depth_offset = 0.0;
        while let Some(cid) = current {
            let discounted = reward * self.config.reward_discount.powf(depth_offset);
            if let Some(node) = self.nodes.get_mut(&cid) {
                node.visits += 1;
                node.total_value += discounted;
                depth_offset += 1.0;
                current = node.parent;
            } else {
                break;
            }
        }
    }

    #[allow(dead_code)]
    fn heuristic_prior(&self, hypothesis: &Hypothesis) -> f64 {
        let existing: Vec<&Hypothesis> = self
            .blackboard
            .hypotheses
            .iter()
            .filter(|h| h.id != hypothesis.id)
            .collect();

        if existing.is_empty() {
            return hypothesis.confidence;
        }

        let alignment: f64 = existing
            .iter()
            .map(|h| {
                let sim = QuantizedVSA::similarity(&hypothesis.content, &h.content);
                sim * h.confidence
            })
            .sum::<f64>()
            / existing.len() as f64;

        (hypothesis.confidence * 0.6 + alignment * 0.4).clamp(0.0, 1.0)
    }

    fn heuristic_prior_for_expert(&self, expert: &ExpertType, depth: usize) -> f64 {
        let expert_hypotheses: Vec<&Hypothesis> = self
            .blackboard
            .hypotheses
            .iter()
            .filter(|h| h.expert == *expert && !h.is_contradicted)
            .collect();

        if expert_hypotheses.is_empty() {
            return 0.5;
        }

        let avg_confidence: f64 = expert_hypotheses.iter().map(|h| h.confidence).sum::<f64>()
            / expert_hypotheses.len() as f64;

        let diversity: f64 = if expert_hypotheses.len() > 1 {
            let mut sim_sum = 0.0;
            let mut sim_count = 0;
            for i in 0..expert_hypotheses.len() {
                for j in (i + 1)..expert_hypotheses.len() {
                    let sim = QuantizedVSA::similarity(
                        &expert_hypotheses[i].content,
                        &expert_hypotheses[j].content,
                    );
                    sim_sum += sim;
                    sim_count += 1;
                }
            }
            1.0 - (sim_sum / sim_count as f64)
        } else {
            0.5
        };

        let depth_factor = 1.0 / (1.0 + depth as f64 * 0.2);
        let prior = avg_confidence * 0.5 + diversity * 0.3 + depth_factor * 0.2;
        prior.clamp(0.0, 1.0)
    }

    fn prm_evaluate(&mut self, hypothesis: &Hypothesis) -> f64 {
        let existing: Vec<&Hypothesis> = self
            .blackboard
            .hypotheses
            .iter()
            .filter(|h| h.id != hypothesis.id)
            .collect();

        if existing.is_empty() {
            self.blackboard.post_hypothesis(
                hypothesis.content.clone(),
                hypothesis.confidence,
                hypothesis.expert,
                vec![],
            );
            return hypothesis.confidence;
        }

        let mut novelty = 0.0;
        let mut novelty_count = 0;
        for h in &existing {
            let sim = QuantizedVSA::similarity(&hypothesis.content, &h.content);
            novelty += 1.0 - sim;
            novelty_count += 1;
        }
        let avg_novelty = novelty / novelty_count as f64;

        let support_count = existing
            .iter()
            .filter(|h| {
                let sim = QuantizedVSA::similarity(&hypothesis.content, &h.content);
                sim > 0.7 && !h.is_contradicted
            })
            .count();

        let support_ratio = support_count as f64 / existing.len().max(1) as f64;
        let contradiction_penalty = if hypothesis.is_contradicted { 0.5 } else { 0.0 };

        let score = hypothesis.confidence * 0.4 + avg_novelty * 0.2 + support_ratio * 0.3
            - contradiction_penalty * 0.1;

        let score_clamped = score.clamp(0.0, 1.0);

        self.blackboard.post_hypothesis(
            hypothesis.content.clone(),
            score_clamped,
            hypothesis.expert,
            vec![],
        );

        score_clamped
    }

    #[allow(dead_code)]
    fn vsa_coherence_value(&mut self, path: &[Hypothesis]) -> f64 {
        if path.len() < 2 {
            return 1.0;
        }

        let mut total_sim = 0.0;
        let mut count = 0;
        for w in path.windows(2) {
            let sim = QuantizedVSA::similarity(&w[0].content, &w[1].content);
            total_sim += sim;
            count += 1;

            let state_key = w[0].id.wrapping_mul(31).wrapping_add(w[1].id);
            let entry = self.shared_memory.entry(state_key).or_insert(sim);
            *entry = *entry * 0.9 + sim * 0.1;
        }

        let mean_sim = total_sim / count as f64;
        let ideal_range = 0.4..=0.85;
        if ideal_range.contains(&mean_sim) {
            mean_sim
        } else if mean_sim < 0.4 {
            mean_sim * 0.5
        } else {
            1.0 - (mean_sim - 0.85)
        }
        .clamp(0.0, 1.0)
    }

    fn best_path(&self, node_id: u64) -> Vec<Hypothesis> {
        let mut current = node_id;
        let mut best_path = Vec::new();
        let mut visited = HashSet::new();

        loop {
            if !visited.insert(current) {
                break;
            }
            let node = match self.nodes.get(&current) {
                Some(n) => n,
                None => break,
            };

            if !node.reasoning_path.is_empty() {
                let last = node.reasoning_path.last().expect("empty path");
                if !best_path.iter().any(|h: &Hypothesis| h.id == last.id) {
                    best_path.push(last.clone());
                }
            }

            if node.children.is_empty() {
                break;
            }

            let next = node
                .children
                .iter()
                .max_by(|&&a, &&b| {
                    let na = self.nodes.get(&a);
                    let nb = self.nodes.get(&b);
                    let va = na
                        .map(|n| n.total_value / n.visits.max(1) as f64)
                        .unwrap_or(0.0);
                    let vb = nb
                        .map(|n| n.total_value / n.visits.max(1) as f64)
                        .unwrap_or(0.0);
                    va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied();

            match next {
                Some(id) => current = id,
                None => break,
            }
        }

        best_path
    }

    fn prune_low_confidence(&mut self) {
        let threshold = self.config.confidence_threshold;
        let prune_ids: Vec<u64> = self
            .nodes
            .iter()
            .filter(|(&id, node)| {
                id != self.root_id
                    && node.visits > 0
                    && (node.total_value / node.visits as f64) < threshold
                    && node.parent.is_some()
                    && node.parent.map_or(false, |pid| {
                        self.nodes.get(&pid).map_or(false, |p| p.children.len() > 1)
                    })
            })
            .map(|(&id, _)| id)
            .collect();

        for pid in &prune_ids {
            if let Some(node) = self.nodes.remove(pid) {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = self.nodes.get_mut(&parent_id) {
                        parent.children.retain(|c| c != pid);
                    }
                }
            }
        }
    }

    pub fn stats(&self) -> MctsStats {
        let total_nodes = self.nodes.len();
        let root_visits = self.nodes.get(&self.root_id).map(|n| n.visits).unwrap_or(0);
        let best_value = self
            .nodes
            .values()
            .filter(|n| n.visits > 0)
            .map(|n| n.total_value / n.visits as f64)
            .fold(0.0f64, f64::max);

        let prm_len = self.prm_scores.len() as f64;
        let prm_mean = if prm_len > 0.0 {
            self.prm_scores.iter().sum::<f64>() / prm_len
        } else {
            0.0
        };
        let prm_std = if prm_len > 1.0 {
            let variance = self
                .prm_scores
                .iter()
                .map(|s| (s - prm_mean).powi(2))
                .sum::<f64>()
                / (prm_len - 1.0);
            variance.sqrt()
        } else {
            0.0
        };

        let explored = self.nodes.values().filter(|n| n.visits > 0).count();
        let exploration_efficiency = if total_nodes > 0 && root_visits > 0 {
            explored as f64 / total_nodes as f64
        } else {
            0.0
        };

        MctsStats {
            total_nodes,
            root_visits,
            best_value,
            prm_mean,
            prm_std,
            exploration_efficiency,
        }
    }
}

impl MctsNode {
    pub fn avg_value(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_value / self.visits as f64
        }
    }
}
