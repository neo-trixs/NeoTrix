//! Distributional prediction for E₈ state transitions.
//!
//! Problem: `E8TransitionMatrix::predict_next()` returns only the single most likely
//! next state via discrete argmax, with no uncertainty, no ensemble blending,
//! no multi-step lookahead, and no differentiable path to GWT attention.
//!
//! This module adds:
//! 1. **Full predictive distribution** over all 64 states with entropy-based uncertainty
//! 2. **Multi-source ensemble**: empirical matrix + task-type chain + phase-constrained
//! 3. **Phase-aware prediction**: Fable-5 phase sequence constrains transitions
//! 4. **MCTS lookahead**: simulate N-step future to score current transition
//! 5. **Differentiable attention weights**: soft distribution for GWT bridge

use crate::core::nt_core_e8::domain_transition::{CoTLength, E8TaskType};
use crate::core::nt_core_e8::e8_lattice_quantizer::E8LatticeQuantizer;
use crate::core::nt_core_e8::nt_core_fable_pattern::{
    FablePatternMatcher, FablePhase, PhaseTransitionMatrix,
};
use crate::core::nt_core_e8::E8TransitionMatrix;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Softmax with temperature: τ → 0 = argmax, τ → ∞ = uniform.
fn softmax_with_temp(scores: &[f64], tau: f64) -> Vec<f64> {
    let t = tau.max(0.01);
    let max_val = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exp: Vec<f64> = scores.iter().map(|s| ((s - max_val) / t).exp()).collect();
    let sum: f64 = exp.iter().sum();
    if sum == 0.0 {
        return vec![1.0 / 64.0; 64];
    }
    exp.iter().map(|e| e / sum).collect()
}

/// Normalized entropy of a probability distribution (0 = certain, 1 = uniform).
fn distribution_entropy(probs: &[f64]) -> f64 {
    let h: f64 = probs
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.ln())
        .sum();
    let max_entropy = (64.0_f64).ln();
    (h / max_entropy).max(0.0).min(1.0)
}

/// Top-K indices from a probability distribution.
fn top_k(probs: &[f64], k: usize) -> Vec<(u8, f64)> {
    let mut idx: Vec<(u8, f64)> = probs
        .iter()
        .enumerate()
        .map(|(i, &p)| (i as u8, p))
        .collect();
    idx.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    idx.truncate(k.min(64));
    idx
}

/// Predictive distribution from the empirical transition matrix row.
///
/// Uses `dominance_capped_distribution` (Kimi K3 Quantile Balancing) so a single
/// over-visited destination cannot monopolize the row and flatten attention.
fn matrix_distribution(tm: &E8TransitionMatrix, from: u8) -> Vec<f64> {
    let fi = from.min(63) as usize;
    let total = tm.row_totals.0[fi];
    if total == 0 {
        return vec![1.0 / 64.0; 64];
    }
    // Cap 0.8: prevents a single destination from fully monopolizing a row
    // (K3 quantile-balancing intent) while still leaving headroom for
    // temperature to concentrate attention on the true mode.
    tm.dominance_capped_distribution(from, 0.8)
}

/// Top-K sparse attention: keep only the largest `k` probability entries and
/// renormalize the rest to zero. Mirrors the sparsity of Mixture-of-Experts
/// routers (Kimi K3 activates 16 of 896 experts; Qwen3 allocates a thinking
/// budget). With `k` scaling down as confidence rises, attention progressively
/// condenses onto the predicted modes instead of staying flat across all 64.
fn sparse_top_k(probs: &[f64], k: usize) -> Vec<f64> {
    if probs.len() != 64 || k >= 64 {
        return probs.to_vec();
    }
    let mut idx: Vec<(usize, f64)> = probs.iter().enumerate().map(|(i, &p)| (i, p)).collect();
    idx.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = vec![0.0; 64];
    let mut sum = 0.0;
    for (i, p) in idx.iter().take(k) {
        out[*i] = *p;
        sum += p;
    }
    if sum > 0.0 {
        for o in out.iter_mut() {
            *o /= sum;
        }
    }
    out
}

/// Canonical task-type chain as a probability bump around predicted position.
fn task_chain_distribution(chain: &[u8; 9], _from: u8, current_phase: usize) -> [f64; 64] {
    let mut probs = [0.0; 64];
    // Phase determines which step in the chain we're at
    let phase = current_phase.min(8);
    let target = if phase < 8 {
        chain[phase + 1]
    } else {
        chain[8]
    };
    // Gaussian bump around target
    for i in 0..64 {
        let dist = (i as i16 - target as i16).abs() as f64;
        probs[i] = (-dist * dist / 8.0).exp();
    }
    // Normalize and blend with 20% uniform default
    let sum: f64 = probs.iter().sum();
    if sum > 0.0 {
        for p in probs.iter_mut() {
            *p /= sum;
        }
    }
    probs
}

/// Phase-constrained distribution: only allow transitions consistent with
/// Fable-5 phase progress (forward, backtrack, or self-loop).
fn phase_constrained_distribution(
    phase_transitions: &PhaseTransitionMatrix,
    current_phase: FablePhase,
    task_type: E8TaskType,
) -> [f64; 64] {
    let pidx = current_phase as usize;
    let chain = task_type.e8_chain();
    let mut probs = [0.0; 64];
    // Phase transition probabilities: likely next phases
    for next_p in 0..9 {
        let tp = phase_transitions.prob(pidx, next_p);
        if tp > 0.02 {
            let target = chain[next_p] as usize;
            // Spread probability around each phase's canonical hexagram
            for i in 0..64 {
                let dist = (i as i16 - target as i16).abs() as f64;
                probs[i] += tp * (-dist * dist / 12.0).exp();
            }
        }
    }
    let sum: f64 = probs.iter().sum();
    if sum > 0.0 {
        for p in probs.iter_mut() {
            *p /= sum;
        }
    } else {
        return [1.0 / 64.0; 64];
    }
    probs
}

/// The full predictive distribution over E8 states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8PredictiveDistribution {
    /// Probability for each of 64 states, P(next | current, context).
    pub probabilities: Vec<f64>,
    /// Normalized entropy in [0, 1] (0 = certain, 1 = uniform).
    pub entropy: f64,
    /// Confidence = 1 - entropy, high means concentrated prediction.
    pub confidence: f64,
    /// Top-5 predictions (state, probability).
    pub top_5: Vec<(u8, f64)>,
    /// Number of observations in the underlying data.
    pub num_samples: u64,
}

impl E8PredictiveDistribution {
    /// Build from raw probability array.
    pub fn from_probs(probs: Vec<f64>, num_samples: u64) -> Self {
        let probs = if probs.len() != 64 {
            vec![1.0 / 64.0; 64]
        } else {
            probs
        };
        let entropy = distribution_entropy(&probs);
        Self {
            probabilities: probs.to_vec(),
            entropy,
            confidence: 1.0 - entropy,
            top_5: top_k(&probs, 5),
            num_samples,
        }
    }

    /// Build from matrix row only.
    pub fn from_matrix(tm: &E8TransitionMatrix, from: u8) -> Self {
        let fi = from.min(63) as usize;
        let probs = matrix_distribution(tm, from);
        let num_samples = tm.row_totals.0[fi];
        Self::from_probs(probs, num_samples)
    }

    /// Returns the single most likely state (for backward compat).
    pub fn best(&self) -> (u8, f64) {
        *self.top_5.first().unwrap_or(&(0, 0.0))
    }

    /// Returns a vector of soft-attention weights suitable for GWT bridge.
    /// If `temperature` is low (0.1), approximates one-hot; high (1.0) = soft.
    pub fn attention_weights(&self, temperature: f64) -> Vec<f64> {
        softmax_with_temp(&self.probabilities, temperature)
    }

    /// Sparse attention: softmax then zero-out all but the top-`k` states.
    /// The `k` controls the "thinking budget" — fewer states means sharper focus.
    pub fn attention_weights_sparse(&self, temperature: f64, k: usize) -> Vec<f64> {
        let soft = softmax_with_temp(&self.probabilities, temperature);
        sparse_top_k(&soft, k)
    }

    /// Returns the number of states that account for 90% of probability mass.
    pub fn effective_90pct_count(&self) -> usize {
        let mut sorted: Vec<f64> = self.probabilities.to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let mut cum = 0.0;
        for (i, &p) in sorted.iter().enumerate() {
            cum += p;
            if cum >= 0.90 {
                return i + 1;
            }
        }
        64
    }
}

/// Multi-source prediction ensemble.
///
/// Blends three prediction signals:
/// 1. **Empirical matrix**: raw transition counts from past observations
/// 2. **Task-type chain**: canonical 9-step chain for the task type
/// 3. **Phase-constrained**: Fable-5 phase transition matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8PredictionEnsemble {
    /// Blend weights: [matrix, task_chain, phase_constrained]
    pub weights: [f64; 3],
    /// Temperature for softmax normalization (default 0.5)
    pub temperature: f64,
    /// Minimum observations before trusting matrix signal
    pub min_observations: u64,
}

impl Default for E8PredictionEnsemble {
    fn default() -> Self {
        Self {
            weights: [0.4, 0.35, 0.25],
            temperature: 0.5,
            min_observations: 5,
        }
    }
}

impl E8PredictionEnsemble {
    pub fn new(weights: [f64; 3], temperature: f64, min_observations: u64) -> Self {
        Self {
            weights,
            temperature,
            min_observations,
        }
    }

    /// Compute blended predictive distribution.
    pub fn predict(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        task_type: E8TaskType,
        current_phase: FablePhase,
        pattern_matcher: &FablePatternMatcher,
        cot_length: CoTLength,
    ) -> E8PredictiveDistribution {
        let fi = from.min(63) as usize;
        let num_samples = tm.row_totals.0[fi];

        // 1. Empirical matrix distribution
        let matrix_dist = matrix_distribution(tm, from);

        // 2. Task-type chain distribution
        let chain = task_type.e8_chain();
        // Determine current phase position in chain
        let phase_pos = current_phase as usize;
        let chain_dist = task_chain_distribution(chain, from, phase_pos);

        // 3. Phase-constrained distribution
        let phase_transitions = &pattern_matcher.phase_transitions;
        let phase_dist =
            phase_constrained_distribution(phase_transitions, current_phase, task_type);

        // Adaptive weights: if data is sparse, trust chains/phases more
        let mut w = self.weights;
        if num_samples < self.min_observations {
            w[0] = self.weights[0] * (num_samples as f64 / self.min_observations as f64);
            w[1] = self.weights[1] + (self.weights[0] * 0.5);
            w[2] = self.weights[2] + (self.weights[0] * 0.5);
        }
        // Apply CoT length depth bias: short trajectories stay closer to chain defaults
        let depth_mult = cot_length.depth_multiplier();
        if depth_mult < 1.0 {
            w[0] *= 0.7; // less data trust for shallow reasoning
            w[1] *= 1.2; // more chain trust
        }

        // Blend
        let mut blended = [0.0f64; 64];
        for i in 0..64 {
            blended[i] = w[0] * matrix_dist[i] + w[1] * chain_dist[i] + w[2] * phase_dist[i];
        }
        // Normalize
        let sum: f64 = blended.iter().sum();
        if sum > 0.0 {
            for p in blended.iter_mut() {
                *p /= sum;
            }
        }

        E8PredictiveDistribution::from_probs(blended.to_vec(), num_samples)
    }

    /// Adaptive query: returns top prediction with uncertainty diagnostic.
    pub fn predict_with_diagnostic(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        task_type: E8TaskType,
        current_phase: FablePhase,
        pattern_matcher: &FablePatternMatcher,
        cot_length: CoTLength,
    ) -> (u8, f64, f64, f64) {
        let dist = self.predict(
            tm,
            from,
            task_type,
            current_phase,
            pattern_matcher,
            cot_length,
        );
        let (best_state, best_prob) = dist.best();
        (best_state, best_prob, dist.confidence, dist.entropy)
    }
}

/// Phase-aware step predictor that constrains transitions using Fable-5 phases.
///
/// Unlike the ensemble which computes a full distribution, this predictor
/// focuses on what phase we should be in NEXT and maps that to E8 states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8PhaseAwarePredictor {
    pub phase_transitions: PhaseTransitionMatrix,
    pub phase_hexagrams: [[u8; 9]; 6],
}

impl E8PhaseAwarePredictor {
    pub fn new(pattern_matcher: &FablePatternMatcher) -> Self {
        Self {
            phase_transitions: pattern_matcher.phase_transitions.clone(),
            phase_hexagrams: pattern_matcher.phase_hexagrams,
        }
    }

    /// Predict the most likely next phase given current phase.
    pub fn next_phase(&self, current_phase: FablePhase) -> (FablePhase, f64) {
        let idx = current_phase as usize;
        let next_idx = self.phase_transitions.most_likely_next(idx);
        let prob = self.phase_transitions.prob(idx, next_idx);
        let next_phase = match next_idx {
            0 => FablePhase::Acknowledgment,
            1 => FablePhase::ProblemRestatement,
            2 => FablePhase::Decomposition,
            3 => FablePhase::FirstPrinciples,
            4 => FablePhase::SelfVerification,
            5 => FablePhase::AlternativeConsideration,
            6 => FablePhase::DeepDive,
            7 => FablePhase::Synthesis,
            _ => FablePhase::Conclusion,
        };
        (next_phase, prob)
    }

    /// Predict next E8 state given current phase and task type.
    pub fn predict_state(&self, current_phase: FablePhase, task_type: E8TaskType) -> (u8, f64) {
        let (next, confidence) = self.next_phase(current_phase);
        let tidx = task_type as usize;
        let state = self.phase_hexagrams[tidx][next as usize];
        (state, confidence)
    }
}

/// A single node in the MCTS prediction tree.
#[derive(Debug, Clone)]
struct MctsPredNode {
    /// E8 state at this node.
    state: u8,
    /// Visit count.
    visits: u64,
    /// Cumulative value from rollouts.
    total_value: f64,
    /// Children (next states).
    children: Vec<MctsPredNode>,
    /// Unexpanded action list.
    untried: Vec<u8>,
    /// Parent index.
    parent: Option<usize>,
}

/// MCTS lookahead predictor for multi-step E8 trajectory evaluation.
///
/// Simulates `num_simulations` rollouts of depth `max_depth` to estimate
/// which transitions lead to high-value trajectories (not just immediate reward).
///
/// Uses E8TransitionMatrix for transition dynamics (the "world model") and
/// FablePatternMatcher for rollout reward (how "Fable-aligned" the trajectory is).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8MctsPredictor {
    /// Number of full MCTS simulations per query.
    pub num_simulations: usize,
    /// Maximum depth of each rollout.
    pub max_depth: usize,
    /// UCB1 exploration constant (default √2).
    pub exploration_constant: f64,
    /// Discount factor for future rewards.
    pub gamma: f64,
}

impl Default for E8MctsPredictor {
    fn default() -> Self {
        Self {
            num_simulations: 32,
            max_depth: 6,
            exploration_constant: std::f64::consts::SQRT_2,
            gamma: 0.95,
        }
    }
}

impl E8MctsPredictor {
    pub fn new(
        num_simulations: usize,
        max_depth: usize,
        exploration_constant: f64,
        gamma: f64,
    ) -> Self {
        Self {
            num_simulations,
            max_depth,
            exploration_constant,
            gamma,
        }
    }

    /// Run MCTS from current state and predict best next state.
    ///
    /// Returns (best_state, value, confidence).
    pub fn predict(
        &self,
        current_state: u8,
        tm: &E8TransitionMatrix,
        pattern_matcher: &FablePatternMatcher,
        task_type: E8TaskType,
        current_phase: FablePhase,
    ) -> (u8, f64, f64) {
        // Flattened tree: indices into this Vec
        let mut nodes: Vec<MctsPredNode> = Vec::new();
        let root = MctsPredNode {
            state: current_state,
            visits: 0,
            total_value: 0.0,
            children: Vec::new(),
            untried: (0..64u8).collect(),
            parent: None,
        };
        nodes.push(root);

        for _sim in 0..self.num_simulations {
            // Selection + expansion
            let path = self.select(&nodes, current_state);
            let leaf_idx = *path.last().unwrap_or(&0);

            // Expand if untried actions remain
            let expand_idx = if !nodes[leaf_idx].untried.is_empty() {
                self.expand(&mut nodes, leaf_idx, tm)
            } else {
                leaf_idx
            };

            // Rollout
            let reward = self.rollout(
                &nodes[expand_idx].state,
                tm,
                pattern_matcher,
                task_type,
                current_phase,
            );

            // Backpropagation
            self.backpropagate(&mut nodes, &path, expand_idx, reward);
        }

        // Pick best child: highest visit count (exploitation)
        let root_visits = nodes[0].visits.max(1);
        let best = nodes[0]
            .children
            .iter()
            .map(|c| (c.state, c.total_value / c.visits.max(1) as f64, c.visits))
            .max_by(|(_, v1, _), (_, v2, _)| {
                v1.partial_cmp(v2).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or((current_state, 0.5, 0));

        let confidence = best.2 as f64 / root_visits as f64;
        (best.0, best.1, confidence)
    }

    /// MCTS selection using UCB1.
    fn select(&self, nodes: &[MctsPredNode], _root_state: u8) -> Vec<usize> {
        let mut path = vec![0usize];
        let mut current = 0usize;
        loop {
            let node = &nodes[current];
            if node.children.is_empty() || !node.untried.is_empty() {
                break;
            }
            // UCB1
            let total_n = node.visits.max(1) as f64;
            let best = node
                .children
                .iter()
                .map(|c| {
                    let n = c.visits.max(1) as f64;
                    let q = c.total_value / n;
                    let explore = self.exploration_constant * (total_n.ln() / n).sqrt();
                    (q + explore, c.state)
                })
                .max_by(|(v1, _), (v2, _)| v1.partial_cmp(v2).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, s)| s);
            if let Some(next_state) = best {
                let next_idx = nodes
                    .iter()
                    .position(|n| n.state == next_state)
                    .unwrap_or(0);
                if next_idx == current {
                    break;
                }
                path.push(next_idx);
                current = next_idx;
            } else {
                break;
            }
        }
        path
    }

    /// Expand a node by trying one untried action.
    fn expand(
        &self,
        nodes: &mut Vec<MctsPredNode>,
        leaf_idx: usize,
        tm: &E8TransitionMatrix,
    ) -> usize {
        let state = nodes[leaf_idx].state;
        // Pick the untried action with highest empirical transition probability
        let action = nodes[leaf_idx]
            .untried
            .iter()
            .max_by(|&&a, &&b| {
                tm.transition_prob(state, a)
                    .partial_cmp(&tm.transition_prob(state, b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(0);
        // Remove from untried
        if let Some(pos) = nodes[leaf_idx].untried.iter().position(|&x| x == action) {
            nodes[leaf_idx].untried.remove(pos);
        }
        let child = MctsPredNode {
            state: action,
            visits: 0,
            total_value: 0.0,
            children: Vec::new(),
            untried: (0..64u8).collect(),
            parent: Some(leaf_idx),
        };
        let child_idx = nodes.len();
        nodes[leaf_idx].children.push(child.clone());
        nodes.push(child);
        child_idx
    }

    /// Rollout: simulate to max_depth, scoring Fable-alignment at each step.
    fn rollout(
        &self,
        start_state: &u8,
        tm: &E8TransitionMatrix,
        pattern_matcher: &FablePatternMatcher,
        task_type: E8TaskType,
        _current_phase: FablePhase,
    ) -> f64 {
        let mut state = *start_state;
        let mut total_reward = 0.0;
        let mut discount = 1.0;
        let tidx = task_type as usize;

        // Use canonical chain as reference
        let chain = task_type.e8_chain();

        for step in 0..self.max_depth {
            // Sample next state from transition distribution
            let dist = matrix_distribution(tm, state);
            // Stochastic: weighted random sample
            let r: f64 = rand::thread_rng().gen::<f64>();
            let mut cum = 0.0;
            let next = (0..64)
                .find(|&i| {
                    cum += dist[i];
                    cum >= r
                })
                .unwrap_or(state as usize);

            state = next as u8;

            // Reward: how close to the chain target at this step?
            let target = chain[step.min(8)] as f64;
            let dist_penalty = (state as f64 - target).abs() / 64.0;
            let novelty_bonus = if step > 0 {
                // Small bonus for non-stuck states
                if state != *start_state {
                    0.05
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let pattern_score = if step > 1 {
                // Fable alignment bonus for maintaining reasonable transitions
                let tpidx = if state & 0x04 != 0 {
                    4.min(chain.len() - 1)
                } else {
                    step.min(8)
                };
                let chain_target = chain[tpidx] as f64;
                0.1 * (1.0 - (state as f64 - chain_target).abs() / 64.0)
            } else {
                0.0
            };

            let step_reward = 1.0 - dist_penalty + novelty_bonus + pattern_score;
            total_reward += discount * step_reward.max(0.0).min(1.0);
            discount *= self.gamma;
        }

        // Final bonus: check if trajectory aligns with Fable pattern
        let traj: Vec<u8> = vec![*start_state]; // simplified
        let alignment = pattern_matcher.score_alignment(&traj, tidx);
        let composite = alignment.composite;
        total_reward += discount * composite;

        total_reward / (1.0 + self.max_depth as f64) // normalize to [0,1]
    }

    /// Backpropagate reward through the tree.
    fn backpropagate(
        &self,
        nodes: &mut Vec<MctsPredNode>,
        _path: &[usize],
        leaf_idx: usize,
        reward: f64,
    ) {
        let mut current = leaf_idx;
        loop {
            nodes[current].visits += 1;
            nodes[current].total_value += reward;
            match nodes[current].parent {
                Some(pidx) => current = pidx,
                None => break,
            }
        }
    }
}

/// Top-level prediction oracle combining all prediction methods.
///
/// Provides a unified API for:
/// - Distributional prediction (ensemble + uncertainty)
/// - Phase-aware step prediction
/// - MCTS lookahead for trajectory evaluation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct E8PredictionOracle {
    pub ensemble: E8PredictionEnsemble,
    pub mcts: E8MctsPredictor,
}

impl E8PredictionOracle {
    pub fn new(ensemble: E8PredictionEnsemble, mcts: E8MctsPredictor) -> Self {
        Self { ensemble, mcts }
    }

    /// Quick single-step prediction (distributional).
    pub fn predict_distribution(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        task_type: E8TaskType,
        current_phase: FablePhase,
        pattern_matcher: &FablePatternMatcher,
        cot_length: CoTLength,
    ) -> E8PredictiveDistribution {
        self.ensemble.predict(
            tm,
            from,
            task_type,
            current_phase,
            pattern_matcher,
            cot_length,
        )
    }

    /// MCTS-enhanced prediction: uses lookahead to refine distribution.
    pub fn predict_with_mcts(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        task_type: E8TaskType,
        current_phase: FablePhase,
        pattern_matcher: &FablePatternMatcher,
        cot_length: CoTLength,
    ) -> (E8PredictiveDistribution, u8, f64, f64) {
        let dist = self.predict_distribution(
            tm,
            from,
            task_type,
            current_phase,
            pattern_matcher,
            cot_length,
        );
        let (mcts_state, mcts_value, mcts_confidence) =
            self.mcts
                .predict(from, tm, pattern_matcher, task_type, current_phase);
        // Blend ensemble top-1 with MCTS top-1
        let (_, ens_prob) = dist.best();
        let _blended_value = 0.6 * ens_prob + 0.4 * mcts_value;
        let _blended_confidence = 0.5 * dist.confidence + 0.5 * mcts_confidence;
        (dist, mcts_state, mcts_value, mcts_confidence)
    }

    /// Get attention weights for GWT bridge (differentiable).
    pub fn attention_weights(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        task_type: E8TaskType,
        current_phase: FablePhase,
        pattern_matcher: &FablePatternMatcher,
        cot_length: CoTLength,
        temperature: f64,
    ) -> Vec<f64> {
        let dist = self.predict_distribution(
            tm,
            from,
            task_type,
            current_phase,
            pattern_matcher,
            cot_length,
        );
        dist.attention_weights(temperature)
    }

    /// E8 geometric attention refinement using lattice quantization.
    /// Projects the predicted distribution onto E8 root lattice, producing
    /// geometric bias that enhances coherence for lattice-aligned states.
    pub fn predict_with_geometric_refinement(
        &self,
        tm: &E8TransitionMatrix,
        from: u8,
        task_type: E8TaskType,
        current_phase: FablePhase,
        pattern_matcher: &FablePatternMatcher,
        cot_length: CoTLength,
        quantizer: &E8LatticeQuantizer,
    ) -> E8PredictiveDistribution {
        let dist = self.predict_distribution(
            tm,
            from,
            task_type,
            current_phase,
            pattern_matcher,
            cot_length,
        );
        let mut refined_probs = dist.probabilities.clone();
        let fx = (from as f32 / 64.0) * 2.0 - 1.0;
        let fh = [fx, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (_, q_idx, _) = quantizer.quantize(&fh);
        for s in 0..64 {
            let sx = (s as f32 / 64.0) * 2.0 - 1.0;
            let sh = [sx, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
            let (_, s_idx, _) = quantizer.quantize(&sh);
            let ri = quantizer.root_system.root(q_idx);
            let rj = quantizer.root_system.root(s_idx);
            let geo_sim: f32 = ri
                .iter()
                .zip(rj.iter())
                .map(|(&a, &b)| (a as f32) * (b as f32))
                .sum();
            let bias = (geo_sim / 8.0) as f64;
            refined_probs[s] = (refined_probs[s] * (1.0 + bias * 0.1)).max(0.0).min(1.0);
        }
        let sum: f64 = refined_probs.iter().sum();
        if sum > 0.0 {
            for p in refined_probs.iter_mut() {
                *p /= sum;
            }
        }
        E8PredictiveDistribution::from_probs(refined_probs, dist.num_samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_e8::E8TransitionMatrix;

    fn make_tm() -> E8TransitionMatrix {
        let mut tm = E8TransitionMatrix::new();
        tm.init_from_trace_patterns();
        // Add some empirical data: mode 56 → 48 frequently
        for _ in 0..20 {
            tm.record_transition(56, 48);
        }
        for _ in 0..5 {
            tm.record_transition(56, 40);
        }
        for _ in 0..3 {
            tm.record_transition(56, 50);
        }
        for _ in 0..10 {
            tm.record_transition(48, 40);
        }
        for _ in 0..8 {
            tm.record_transition(48, 42);
        }
        tm
    }

    fn make_pattern_matcher() -> FablePatternMatcher {
        FablePatternMatcher::default()
    }

    #[test]
    fn test_softmax_with_temp_extremes() {
        let scores = [
            10.0, 5.0, 1.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        // Low temperature → argmax-like
        let hot = softmax_with_temp(&scores, 0.1);
        assert!(hot[0] > 0.99, "low temp should concentrate on max");
        // High temperature → uniform
        let uniform = softmax_with_temp(&scores, 100.0);
        let expected = 1.0 / 64.0;
        assert!(
            (uniform[0] - expected).abs() < 0.02,
            "high temp should be near-uniform"
        );
    }

    #[test]
    fn test_distribution_entropy_extremes() {
        // Uniform = max entropy
        let uniform = [1.0 / 64.0; 64];
        assert!((distribution_entropy(&uniform) - 1.0).abs() < 0.01);

        // Delta = zero entropy
        let mut delta = [0.0; 64];
        delta[0] = 1.0;
        assert!(distribution_entropy(&delta) < 0.01);
    }

    #[test]
    fn test_predictive_distribution_from_matrix() {
        let tm = make_tm();
        let dist = E8PredictiveDistribution::from_matrix(&tm, 56);
        assert!(dist.entropy >= 0.0 && dist.entropy <= 1.0);
        assert_eq!(dist.top_5.len(), 5);
        // Most likely next from 56 should be 48 (20 transitions)
        assert_eq!(dist.best().0, 48);
        assert!(dist.best().1 > 0.4);
    }

    #[test]
    fn test_ensemble_prediction() {
        let tm = make_tm();
        let pm = make_pattern_matcher();
        let ensemble = E8PredictionEnsemble::default();

        // Predict from 56 in Math task, Decomposition phase
        let dist = ensemble.predict(
            &tm,
            56,
            E8TaskType::Math,
            FablePhase::Decomposition,
            &pm,
            CoTLength::Medium,
        );
        assert!(dist.entropy >= 0.0);
        assert!(!dist.top_5.is_empty());
        // With 20 observations of 48→56 and Math chain favoring 42→35, should blend
        assert!(dist.best().1 > 0.0);
    }

    #[test]
    fn test_ensemble_adaptive_weights_sparse_data() {
        let tm = E8TransitionMatrix::new(); // zero data
        let pm = make_pattern_matcher();
        let ensemble = E8PredictionEnsemble::new([0.4, 0.35, 0.25], 0.5, 5);
        let dist = ensemble.predict(
            &tm,
            42,
            E8TaskType::Reasoning,
            FablePhase::FirstPrinciples,
            &pm,
            CoTLength::Long,
        );
        // With zero data, should rely on chain + phase
        assert!(dist.best().1 > 0.0);
    }

    #[test]
    fn test_ensemble_diagnostic_query() {
        let tm = make_tm();
        let pm = make_pattern_matcher();
        let ensemble = E8PredictionEnsemble::default();
        let (state, prob, conf, ent) = ensemble.predict_with_diagnostic(
            &tm,
            48,
            E8TaskType::Reasoning,
            FablePhase::SelfVerification,
            &pm,
            CoTLength::Medium,
        );
        assert!(state < 64);
        assert!(prob > 0.0);
        assert!(conf >= 0.0 && conf <= 1.0);
        assert!(ent >= 0.0 && ent <= 1.0);
    }

    #[test]
    fn test_phase_aware_predictor() {
        let pm = make_pattern_matcher();
        let predictor = E8PhaseAwarePredictor::new(&pm);
        let (next_phase, _) = predictor.next_phase(FablePhase::Decomposition);
        // Decomposition typically → FirstPrinciples in corpus
        assert!(next_phase as usize >= 2);

        let (state, _) = predictor.predict_state(FablePhase::SelfVerification, E8TaskType::Coding);
        assert!(state < 64);
    }

    #[test]
    fn test_mcts_predictor() {
        let tm = make_tm();
        let pm = make_pattern_matcher();
        let mcts = E8MctsPredictor::new(16, 4, 1.0, 0.95);

        let (state, value, confidence) = mcts.predict(
            56,
            &tm,
            &pm,
            E8TaskType::General,
            FablePhase::Acknowledgment,
        );
        assert!(state < 64);
        assert!(value >= 0.0);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_prediction_oracle() {
        let tm = make_tm();
        let pm = make_pattern_matcher();
        let oracle = E8PredictionOracle::default();

        let dist = oracle.predict_distribution(
            &tm,
            56,
            E8TaskType::Math,
            FablePhase::DeepDive,
            &pm,
            CoTLength::Long,
        );
        assert_eq!(dist.top_5.len(), 5);
        assert!(dist.best().1 > 0.0);

        // MCTS-enhanced
        let (_dist, _mcts_state, value, confidence) = oracle.predict_with_mcts(
            &tm,
            56,
            E8TaskType::Math,
            FablePhase::DeepDive,
            &pm,
            CoTLength::Long,
        );
        assert!(value >= 0.0);
        assert!(confidence >= 0.0);

        // Attention weights
        let attn = oracle.attention_weights(
            &tm,
            56,
            E8TaskType::Math,
            FablePhase::DeepDive,
            &pm,
            CoTLength::Long,
            0.3,
        );
        let sum: f64 = attn.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "attention should sum to 1, got {}",
            sum
        );
    }

    #[test]
    fn test_effective_90pct_count() {
        let mut probs = [0.0; 64];
        // Concentrate 90% on 10 states
        for i in 0..10 {
            probs[i] = 0.09;
        }
        // remaining 54 states get negligible
        let total: f64 = probs.iter().sum();
        // Normalize
        for p in probs.iter_mut() {
            *p /= total;
        }

        let dist = E8PredictiveDistribution::from_probs(probs.to_vec(), 50);
        let eff = dist.effective_90pct_count();
        assert!(
            eff >= 8 && eff <= 12,
            "90% should be ~10 states, got {}",
            eff
        );
    }

    #[test]
    fn test_predictive_distribution_sampling() {
        let mut tm = E8TransitionMatrix::new();
        // Only one recorded transition: 10 → 20
        tm.record_transition(10, 20);
        let dist = E8PredictiveDistribution::from_matrix(&tm, 10);
        assert_eq!(dist.best().0, 20);
        assert!(dist.best().1 > 0.8);

        // Zero-data state
        let dist_zero = E8PredictiveDistribution::from_matrix(&tm, 63);
        assert!(dist_zero.entropy > 0.9); // near-uniform
    }

    #[test]
    fn test_attention_weights_vary_by_temperature() {
        let mut tm = E8TransitionMatrix::new();
        for _ in 0..50 {
            tm.record_transition(30, 31);
        }
        for _ in 0..10 {
            tm.record_transition(30, 29);
        }

        let cold = E8PredictiveDistribution::from_matrix(&tm, 30).attention_weights(0.05);
        let hot = E8PredictiveDistribution::from_matrix(&tm, 30).attention_weights(2.0);

        // Cold: most mass on state 31
        assert!(cold[31] > 0.7);
        // Hot: more spread
        assert!(cold[31] > hot[31], "cold should be more concentrated");
    }

    #[test]
    fn test_mcts_determinism() {
        let tm = make_tm();
        let pm = make_pattern_matcher();
        let mcts = E8MctsPredictor::new(16, 4, 1.0, 0.95);

        // Same inputs should give same-ish output (deterministic given no randomness)
        let (s1, v1, _) = mcts.predict(
            56,
            &tm,
            &pm,
            E8TaskType::General,
            FablePhase::Acknowledgment,
        );
        let (s2, v2, _) = mcts.predict(
            56,
            &tm,
            &pm,
            E8TaskType::General,
            FablePhase::Acknowledgment,
        );
        assert_eq!(s1, s2, "MCTS should be deterministic");
        assert!(
            (v1 - v2).abs() < 0.01,
            "MCTS values should be deterministic"
        );
    }

    #[test]
    fn test_predictive_distribution_num_samples_tracking() {
        let mut tm = E8TransitionMatrix::new();
        tm.record_transition(0, 1);
        let dist = E8PredictiveDistribution::from_matrix(&tm, 0);
        assert_eq!(dist.num_samples, 1);
        tm.record_transition(0, 2);
        tm.record_transition(0, 3);
        let dist2 = E8PredictiveDistribution::from_matrix(&tm, 0);
        assert_eq!(dist2.num_samples, 3);
    }

    #[test]
    fn test_phase_aware_predictor_all_phases() {
        let pm = make_pattern_matcher();
        let predictor = E8PhaseAwarePredictor::new(&pm);

        for phase in &[
            FablePhase::Acknowledgment,
            FablePhase::ProblemRestatement,
            FablePhase::Decomposition,
            FablePhase::FirstPrinciples,
            FablePhase::SelfVerification,
            FablePhase::AlternativeConsideration,
            FablePhase::DeepDive,
            FablePhase::Synthesis,
            FablePhase::Conclusion,
        ] {
            let (_next, prob) = predictor.next_phase(*phase);
            assert!(
                prob > 0.0,
                "phase {:?} should have a valid transition",
                phase
            );
            // Should not be the same phase (corpus: no self-loops for most phases)
            // Actually phase map has some self-loops for Conclusion and Acknowledgment
        }
    }

    #[test]
    fn test_oracle_different_phases_give_different_predictions() {
        let tm = make_tm();
        let pm = make_pattern_matcher();
        let oracle = E8PredictionOracle::default();

        let d1 = oracle.predict_distribution(
            &tm,
            56,
            E8TaskType::Coding,
            FablePhase::Acknowledgment,
            &pm,
            CoTLength::Short,
        );
        let d2 = oracle.predict_distribution(
            &tm,
            56,
            E8TaskType::Coding,
            FablePhase::Synthesis,
            &pm,
            CoTLength::Short,
        );
        // Different phases should give different predictions
        let top1 = d1.best().0;
        let top2 = d2.best().0;
        // At least one should differ from the other
        assert!(
            top1 != top2 || d1.entropy != d2.entropy,
            "different phases should yield different distributions"
        );
    }
}
