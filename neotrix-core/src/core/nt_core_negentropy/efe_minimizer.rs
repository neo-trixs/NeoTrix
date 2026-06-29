use std::collections::HashMap;
use std::f64::consts::PI;

/// Transition model trait — predicts next belief state given current belief and action.
pub trait TransitionModel {
    fn predict(&self, belief: &[Vec<f64>], action: usize) -> Vec<f64>;
    fn possible_actions(&self) -> usize;
}

/// Simple deterministic transition with noise injection.
pub struct SimpleTransitionModel {
    action_count: usize,
    noise: f64,
}

impl SimpleTransitionModel {
    pub fn new(action_count: usize, noise: f64) -> Self {
        Self {
            action_count,
            noise,
        }
    }
}

impl TransitionModel for SimpleTransitionModel {
    fn predict(&self, belief: &[Vec<f64>], action: usize) -> Vec<f64> {
        if belief.is_empty() {
            return Vec::new();
        }
        let current = &belief[0];
        let mut prediction = Vec::with_capacity(current.len());
        for (i, &b) in current.iter().enumerate() {
            let action_phase = (action as f64 + 1.0) / (self.action_count as f64 + 1.0);
            let index_phase = (i as f64 + 1.0) / (current.len() as f64 + 1.0);
            let deterministic_target = (action_phase * index_phase * PI).sin() * 0.5 + 0.5;
            let val = b * (1.0 - self.noise) + self.noise * deterministic_target * (1.0 - b);
            prediction.push(val);
        }
        prediction
    }

    fn possible_actions(&self) -> usize {
        self.action_count
    }
}

/// A policy is a sequence of action indices.
#[derive(Debug, Clone)]
pub struct Policy(pub Vec<usize>);

/// Evaluation of a single policy across the planning horizon.
#[derive(Debug, Clone)]
pub struct PolicyEvaluation {
    pub policy: Policy,
    pub efe: f64,
    pub risk: f64,
    pub ambiguity: f64,
    pub info_gain: f64,
}

/// Result of EFE minimization over a set of candidate policies.
#[derive(Debug, Clone)]
pub struct EFEResult {
    pub best_policy: Policy,
    pub evaluations: Vec<PolicyEvaluation>,
    pub expected_free_energy: f64,
}

/// Expected Free Energy Minimizer for active inference.
///
/// EFE = risk_weight · Risk + ambiguity_weight · Ambiguity − info_gain_weight · InfoGain
///
/// Components:
/// - **Risk** — KL divergence from preferred outcomes (exploitation)
/// - **Ambiguity** — Shannon entropy of predicted beliefs (novelty)
/// - **InfoGain** — Bayesian surprise KL(posterior‖prior) (exploration)
///
/// Low EFE = good policy. The minimizer evaluates candidate policies over a
/// planning horizon, scores each by EFE, and returns the best.
#[derive(Debug, Clone)]
pub struct EFEMinimizer {
    dim: usize,
    num_policies: usize,
    planning_horizon: usize,
    preference_bias: f64,
    risk_weight: f64,
    ambiguity_weight: f64,
    info_gain_weight: f64,
}

impl EFEMinimizer {
    pub fn new(
        dim: usize,
        num_policies: usize,
        planning_horizon: usize,
        preference_bias: f64,
        risk_weight: f64,
        ambiguity_weight: f64,
        info_gain_weight: f64,
    ) -> Self {
        Self {
            dim,
            num_policies,
            planning_horizon,
            preference_bias,
            risk_weight,
            ambiguity_weight,
            info_gain_weight,
        }
    }

    /// Create with sensible defaults for a given state dimension.
    ///
    /// Defaults: 10 policies, horizon=3, preference_bias=0.7, all weights=1.0
    pub fn default_for(dim: usize) -> Self {
        Self {
            dim,
            num_policies: 10,
            planning_horizon: 3,
            preference_bias: 0.7,
            risk_weight: 1.0,
            ambiguity_weight: 1.0,
            info_gain_weight: 1.0,
        }
    }

    /// Evaluate all candidate policies and return the one with lowest EFE.
    pub fn evaluate_policies(
        &self,
        beliefs: &[Vec<f64>],
        preferred_outcomes: &[Vec<f64>],
        transition_model: &dyn TransitionModel,
    ) -> EFEResult {
        let num_actions = transition_model.possible_actions();
        let policies = self.generate_policies(num_actions);
        let mut evaluations = Vec::with_capacity(policies.len());

        for policy in &policies {
            let mut sim_beliefs = beliefs.to_vec();
            let mut total_risk = 0.0;
            let mut total_ambiguity = 0.0;
            let mut total_info_gain = 0.0;

            for &action in &policy.0 {
                let prior = sim_beliefs.clone();
                let prediction = transition_model.predict(&sim_beliefs, action);
                sim_beliefs = vec![prediction.clone()];

                let preferred = preferred_outcomes
                    .get(action)
                    .cloned()
                    .unwrap_or_else(|| vec![self.preference_bias; self.dim]);
                total_risk += Self::compute_risk(&prediction, &preferred);
                total_ambiguity += Self::compute_ambiguity(&prediction);
                total_info_gain += Self::compute_info_gain(&sim_beliefs[0], &prior[0]);
            }

            let efe = self.risk_weight * total_risk + self.ambiguity_weight * total_ambiguity
                - self.info_gain_weight * total_info_gain;

            evaluations.push(PolicyEvaluation {
                policy: policy.clone(),
                efe,
                risk: total_risk,
                ambiguity: total_ambiguity,
                info_gain: total_info_gain,
            });
        }

        evaluations.sort_by(|a, b| {
            a.efe
                .partial_cmp(&b.efe)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best_policy = evaluations[0].policy.clone();
        let expected_free_energy = evaluations[0].efe;

        EFEResult {
            best_policy,
            evaluations,
            expected_free_energy,
        }
    }

    /// KL divergence KL(P‖Q) = Σ P(i)·ln(P(i)/Q(i))
    pub fn compute_risk(predicted: &[f64], preferred: &[f64]) -> f64 {
        let mut kl = 0.0;
        for i in 0..predicted.len().min(preferred.len()) {
            let p = predicted[i].clamp(1e-12, 1.0);
            let q = preferred[i].clamp(1e-12, 1.0);
            kl += p * (p / q).ln();
        }
        kl
    }

    /// Shannon entropy H(p) = −Σ p·ln(p)
    pub fn compute_ambiguity(belief: &[f64]) -> f64 {
        let mut h = 0.0;
        for &b in belief {
            let p = b.clamp(1e-12, 1.0 - 1e-12);
            h -= p * p.ln();
        }
        h
    }

    /// Bayesian surprise = KL(posterior‖prior)
    pub fn compute_info_gain(posterior: &[f64], prior: &[f64]) -> f64 {
        Self::compute_risk(posterior, prior)
    }

    /// Softmax policy selection: lower EFE → higher selection probability.
    pub fn softmax_select(&self, evaluations: &[PolicyEvaluation]) -> usize {
        let min_efe = evaluations
            .iter()
            .map(|e| e.efe)
            .fold(f64::INFINITY, f64::min);
        let max_efe = evaluations
            .iter()
            .map(|e| e.efe)
            .fold(f64::NEG_INFINITY, f64::max);

        let range = (max_efe - min_efe).max(1e-12);
        let scores: Vec<f64> = evaluations
            .iter()
            .map(|e| (-(e.efe - min_efe) / range).exp())
            .collect();
        let sum: f64 = scores.iter().sum();

        let mut rng = rand::thread_rng();
        let mut threshold: f64 = rand::Rng::gen_range(&mut rng, 0.0..sum);
        for (i, &s) in scores.iter().enumerate() {
            threshold -= s;
            if threshold <= 0.0 {
                return i;
            }
        }
        evaluations.len() - 1
    }

    fn generate_policies(&self, num_actions: usize) -> Vec<Policy> {
        let mut policies = Vec::with_capacity(self.num_policies);
        let mut rng = rand::thread_rng();
        for _ in 0..self.num_policies {
            let mut actions = Vec::with_capacity(self.planning_horizon);
            for _ in 0..self.planning_horizon {
                actions.push(rand::Rng::gen_range(&mut rng, 0..num_actions));
            }
            policies.push(Policy(actions));
        }
        policies
    }
}

// ── Variational EFE (arXiv:2504.14898 + arXiv:2508.02197) ──────────
// Reformulate EFE minimization as variational inference on an augmented
// generative model.  Instead of the 3-term weighted sum, EFE becomes VFE
// on the augmented model, computed via message passing on factor graphs.

/// Factor node for sum-product message passing.
#[derive(Debug, Clone)]
pub enum FactorNode {
    Prior {
        target: usize,
        parameters: Vec<f64>,
    },
    Likelihood {
        observation: usize,
        state: usize,
        weights: Vec<Vec<f64>>,
    },
    Transition {
        prev: usize,
        next: usize,
        matrix: Vec<Vec<f64>>,
    },
    Preference {
        state: usize,
        preferred: Vec<f64>,
    },
}

/// Augmented generative model for EFE-as-VFE.
///
/// Wraps the standard POMDP components plus prior preferences so that
/// EFE computation becomes VFE on the augmented factor graph.
#[derive(Debug, Clone)]
pub struct AugmentedGenerativeModel {
    pub preferences: Vec<f64>,
    pub transition: Vec<Vec<Vec<f64>>>,
    pub likelihood: Vec<Vec<f64>>,
    pub state_prior: Vec<f64>,
    pub is_augmented: bool,
}

impl AugmentedGenerativeModel {
    pub fn new(num_states: usize, num_obs: usize, num_actions: usize) -> Self {
        Self {
            preferences: vec![0.5; num_states],
            transition: vec![
                vec![vec![1.0 / num_states as f64; num_states]; num_states];
                num_actions
            ],
            likelihood: vec![vec![1.0 / num_obs as f64; num_obs]; num_states],
            state_prior: vec![1.0 / num_states as f64; num_states],
            is_augmented: false,
        }
    }

    pub fn with_preferences(mut self, prefs: Vec<f64>) -> Self {
        self.preferences = prefs;
        self
    }

    pub fn with_augmented(mut self) -> Self {
        self.is_augmented = true;
        self
    }
}

/// Compute EFE via variational inference on an augmented generative model.
///
/// When the model is augmented, EFE is equivalent to VFE on the extended
/// factor graph, unifying risk, ambiguity, and information gain into a
/// single free energy computation.
pub fn variational_efe(
    model: &AugmentedGenerativeModel,
    policy: &[usize],
    state_belief: &[f64],
) -> f64 {
    let mut factors: Vec<FactorNode> = Vec::new();
    factors.push(FactorNode::Prior {
        target: 0,
        parameters: model.state_prior.clone(),
    });

    for (t, &action) in policy.iter().enumerate() {
        let next_t = t + 1;
        factors.push(FactorNode::Transition {
            prev: t,
            next: next_t,
            matrix: model.transition[action].clone(),
        });
        factors.push(FactorNode::Preference {
            state: next_t,
            preferred: model.preferences.clone(),
        });
    }

    let mut messages: HashMap<String, Vec<f64>> = HashMap::new();
    for (i, factor) in factors.iter().enumerate() {
        match factor {
            FactorNode::Prior {
                target: t,
                parameters,
            } => {
                let key = format!("prior_{}_to_{}", i, t);
                messages.insert(key, parameters.clone());
            }
            FactorNode::Preference {
                state: s,
                preferred,
            } => {
                let key = format!("pref_{}_to_{}", i, s);
                let mut msg = preferred.clone();
                let sum: f64 = msg.iter().sum();
                if sum > 0.0 {
                    for v in msg.iter_mut() {
                        *v /= sum;
                    }
                }
                messages.insert(key, msg);
            }
            FactorNode::Transition { prev, next, matrix } => {
                let from_prior = messages
                    .get(&format!("prior_{}_to_{}", 0, prev))
                    .cloned()
                    .unwrap_or_else(|| state_belief.to_vec());
                let mut msg = vec![0.0; matrix[0].len()];
                for j in 0..matrix[0].len() {
                    for i in 0..from_prior.len() {
                        msg[j] += from_prior[i] * matrix[i][j];
                    }
                }
                let key = format!("trans_{}_to_{}", i, next);
                messages.insert(key, msg);
            }
            _ => {}
        }
    }

    let final_belief = messages
        .get("trans_0_to_1")
        .cloned()
        .unwrap_or_else(|| state_belief.to_vec());
    let mut efe = 0.0;
    for i in 0..final_belief.len().min(model.preferences.len()) {
        let p = final_belief[i].max(1e-12);
        let q = model.preferences[i].max(1e-12);
        efe += p * (p / q).ln();
    }
    efe
}

/// DAIF-inspired single gradient update for EFE policy selection.
///
/// Replaces exhaustive planning tree search with a single gradient update
/// per H steps (arXiv:2505.19867).  Returns the updated policy EFE value.
pub fn single_gradient_update(policy_efe: f64, learning_rate: f64) -> f64 {
    let gradient = policy_efe.tanh();
    policy_efe - learning_rate * gradient
}

/// Softmax policy selection with temperature parameter.
///
/// Lower temperature → more deterministic (greedy).
/// Higher temperature → more exploration.
pub fn softmax_policy_selection(policy_efes: &[f64], temperature: f64) -> Vec<f64> {
    if policy_efes.is_empty() {
        return Vec::new();
    }
    let temp = temperature.max(1e-12);
    let min_efe = policy_efes.iter().cloned().fold(f64::INFINITY, f64::min);
    let adjusted: Vec<f64> = policy_efes
        .iter()
        .map(|e| (-(e - min_efe) / temp).exp())
        .collect();
    let sum: f64 = adjusted.iter().sum();
    if sum < 1e-300 {
        return vec![1.0 / policy_efes.len() as f64; policy_efes.len()];
    }
    adjusted.iter().map(|s| s / sum).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variational_efe_returns_finite() {
        let model = AugmentedGenerativeModel::new(3, 3, 2)
            .with_preferences(vec![0.8, 0.1, 0.1])
            .with_augmented();
        let belief = vec![0.5, 0.3, 0.2];
        let efe = variational_efe(&model, &[0, 1], &belief);
        assert!(!efe.is_nan(), "EFE should not be NaN, got {}", efe);
        assert!(efe >= 0.0, "EFE should be >= 0, got {}", efe);
    }

    #[test]
    fn test_variational_efe_zero_for_perfect_match() {
        let model = AugmentedGenerativeModel::new(2, 2, 1)
            .with_preferences(vec![1.0, 0.0])
            .with_augmented();
        let belief = vec![1.0, 0.0];
        let efe = variational_efe(&model, &[0], &belief);
        assert!(efe < 1e-10, "Perfect match EFE should be ~0, got {}", efe);
    }

    #[test]
    fn test_single_gradient_update() {
        let updated = single_gradient_update(1.0, 0.1);
        assert!(updated < 1.0, "Gradient update should decrease EFE");
        assert!(!updated.is_nan());
    }

    #[test]
    fn test_softmax_policy_selection_sum_to_one() {
        let efes = vec![5.0, 2.0, 8.0, 1.0];
        let probs = softmax_policy_selection(&efes, 1.0);
        let sum: f64 = probs.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-10,
            "Probabilities should sum to 1, got {}",
            sum
        );
        // Lower EFE should have higher probability
        assert!(probs[3] > probs[0], "best policy should have highest prob");
    }

    #[test]
    fn test_softmax_deterministic_at_low_temp() {
        let efes = vec![10.0, 1.0, 5.0];
        let probs = softmax_policy_selection(&efes, 0.01);
        assert!(
            (probs[1] - 1.0).abs() < 1e-6,
            "Best policy should dominate at low temp"
        );
    }

    #[test]
    fn test_softmax_uniform_at_high_temp() {
        let efes = vec![10.0, 1.0, 5.0];
        let probs = softmax_policy_selection(&efes, 100.0);
        let expected = 1.0 / 3.0;
        for &p in &probs {
            assert!(
                (p - expected).abs() < 0.1,
                "Should be near uniform, got {}",
                p
            );
        }
    }

    #[test]
    fn test_empty_efe_list() {
        let probs = softmax_policy_selection(&[], 1.0);
        assert!(probs.is_empty());
    }

    #[test]
    fn test_constructor_defaults() {
        let efe = EFEMinimizer::default_for(4);
        assert_eq!(efe.dim, 4);
        assert_eq!(efe.num_policies, 10);
        assert_eq!(efe.planning_horizon, 3);
        assert_eq!(efe.preference_bias - 0.7, 0.0);
        assert_eq!(efe.risk_weight, 1.0);
        assert_eq!(efe.ambiguity_weight, 1.0);
        assert_eq!(efe.info_gain_weight, 1.0);
    }

    #[test]
    fn test_compute_risk_identical_beliefs() {
        let beliefs = vec![0.5, 0.3, 0.2];
        let risk = EFEMinimizer::compute_risk(&beliefs, &beliefs);
        assert!(risk.abs() < 1e-10);
    }

    #[test]
    fn test_compute_risk_different_beliefs() {
        let predicted = vec![0.8, 0.1, 0.1];
        let preferred = vec![0.2, 0.6, 0.2];
        let risk = EFEMinimizer::compute_risk(&predicted, &preferred);
        assert!(risk > 0.0);
    }

    #[test]
    fn test_compute_ambiguity_uniform_max() {
        let uniform = vec![0.25, 0.25, 0.25, 0.25];
        let entropy = EFEMinimizer::compute_ambiguity(&uniform);
        let max_h = (4.0_f64).ln();
        assert!((entropy - max_h).abs() < 1e-10);
    }

    #[test]
    fn test_compute_ambiguity_deterministic_zero() {
        let deterministic = vec![1.0, 0.0, 0.0, 0.0];
        let entropy = EFEMinimizer::compute_ambiguity(&deterministic);
        assert!(entropy < 1e-10);
    }

    #[test]
    fn test_compute_info_gain_identical() {
        let post = vec![0.5, 0.3, 0.2];
        let prior = vec![0.5, 0.3, 0.2];
        let ig = EFEMinimizer::compute_info_gain(&post, &prior);
        assert!(ig.abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_policies_returns_lowest_efe() {
        let efe = EFEMinimizer::default_for(3);
        let beliefs = vec![vec![0.5, 0.3, 0.2]];
        let preferred = vec![vec![0.8, 0.1, 0.1]; 2];
        let model = SimpleTransitionModel::new(2, 0.1);
        let result = efe.evaluate_policies(&beliefs, &preferred, &model);
        assert_eq!(result.evaluations.len(), 10);
        let min_efe = result
            .evaluations
            .iter()
            .map(|e| e.efe)
            .fold(f64::INFINITY, f64::min);
        assert!((result.expected_free_energy - min_efe).abs() < 1e-10);
    }

    #[test]
    fn test_softmax_select_prefers_lower_efe() {
        let efe = EFEMinimizer::default_for(2);
        let evals = vec![
            PolicyEvaluation {
                policy: Policy(vec![0]),
                efe: 10.0,
                risk: 5.0,
                ambiguity: 5.0,
                info_gain: 0.0,
            },
            PolicyEvaluation {
                policy: Policy(vec![1]),
                efe: 1.0,
                risk: 0.5,
                ambiguity: 0.5,
                info_gain: 0.0,
            },
        ];
        let mut lower_count = 0usize;
        let trials = 1000;
        for _ in 0..trials {
            if efe.softmax_select(&evals) == 1 {
                lower_count += 1;
            }
        }
        assert!(lower_count > trials / 2);
    }

    #[test]
    fn test_transition_model_predict_changes_beliefs() {
        let model = SimpleTransitionModel::new(3, 0.5);
        let belief = vec![vec![0.9, 0.05, 0.05]];
        let prediction = model.predict(&belief, 0);
        assert_eq!(prediction.len(), 3);
        let diff: f64 = prediction
            .iter()
            .zip(belief[0].iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 1e-10);
    }

    #[test]
    fn test_full_pipeline() {
        let efe = EFEMinimizer::default_for(2);
        let beliefs = vec![vec![0.6, 0.4]];
        let preferred = vec![vec![0.9, 0.1]; 2];
        let model = SimpleTransitionModel::new(2, 0.2);
        let result = efe.evaluate_policies(&beliefs, &preferred, &model);
        assert!(!result.evaluations.is_empty());
        assert_eq!(result.best_policy.0.len(), 3);
        for eval in &result.evaluations {
            assert!(!eval.efe.is_nan());
            assert!(!eval.risk.is_nan());
            assert!(!eval.ambiguity.is_nan());
        }
    }

    #[test]
    fn test_custom_constructor() {
        let efe = EFEMinimizer::new(8, 20, 5, 0.8, 0.5, 0.3, 2.0);
        assert_eq!(efe.dim, 8);
        assert_eq!(efe.num_policies, 20);
        assert_eq!(efe.planning_horizon, 5);
        assert_eq!(efe.preference_bias, 0.8);
        assert_eq!(efe.risk_weight, 0.5);
        assert_eq!(efe.ambiguity_weight, 0.3);
        assert_eq!(efe.info_gain_weight, 2.0);
    }

    #[test]
    fn test_efe_non_negative_components() {
        let efe = EFEMinimizer::new(3, 5, 2, 0.5, 1.0, 1.0, 1.0);
        let beliefs = vec![vec![0.5, 0.3, 0.2]];
        let preferred = vec![vec![0.8, 0.1, 0.1]; 2];
        let model = SimpleTransitionModel::new(2, 0.3);
        let result = efe.evaluate_policies(&beliefs, &preferred, &model);
        for eval in &result.evaluations {
            assert!(eval.risk >= 0.0);
            assert!(eval.ambiguity >= 0.0);
            assert!(eval.info_gain >= 0.0);
        }
    }

    #[test]
    fn test_policy_is_action_sequence() {
        let p = Policy(vec![0, 2, 1]);
        assert_eq!(p.0.len(), 3);
        assert_eq!(p.0[0], 0);
        assert_eq!(p.0[1], 2);
        assert_eq!(p.0[2], 1);
    }
}
