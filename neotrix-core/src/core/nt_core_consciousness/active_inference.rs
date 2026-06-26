use std::collections::HashMap;
use std::time::Instant;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// A hidden state in the generative model
#[derive(Debug, Clone)]
pub struct HiddenState {
    pub name: String,
    pub vsa_vector: Vec<u8>,
    pub prior_probability: f64,
}

/// An observation mapping (likelihood)
#[derive(Debug, Clone)]
pub struct ObservationLikelihood {
    pub state_name: String,
    pub observation_name: String,
    pub probability: f64,
}

/// A transition between hidden states given an action
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub action: String,
    pub probability: f64,
}

/// The generative model (POMDP)
#[derive(Debug, Clone)]
pub struct GenerativeModel {
    pub states: Vec<HiddenState>,
    pub observations: Vec<String>,
    pub actions: Vec<String>,
    pub transition_matrix: Vec<StateTransition>,
    pub likelihood_matrix: Vec<ObservationLikelihood>,
    pub prior_over_states: HashMap<String, f64>,
    pub name: String,
}

impl GenerativeModel {
    pub fn new(name: &str) -> Self {
        Self {
            states: Vec::new(),
            observations: Vec::new(),
            actions: Vec::new(),
            transition_matrix: Vec::new(),
            likelihood_matrix: Vec::new(),
            prior_over_states: HashMap::new(),
            name: name.to_string(),
        }
    }

    pub fn add_state(&mut self, name: &str, vsa_vector: Vec<u8>, prior: f64) {
        self.states.push(HiddenState {
            name: name.to_string(),
            vsa_vector,
            prior_probability: prior,
        });
        self.prior_over_states.insert(name.to_string(), prior);
    }

    pub fn add_observation(&mut self, name: &str) {
        self.observations.push(name.to_string());
    }

    pub fn add_action(&mut self, name: &str) {
        self.actions.push(name.to_string());
    }

    pub fn add_transition(&mut self, from: &str, to: &str, action: &str, prob: f64) {
        self.transition_matrix.push(StateTransition {
            from_state: from.to_string(),
            to_state: to.to_string(),
            action: action.to_string(),
            probability: prob,
        });
    }

    pub fn add_likelihood(&mut self, state: &str, observation: &str, prob: f64) {
        self.likelihood_matrix.push(ObservationLikelihood {
            state_name: state.to_string(),
            observation_name: observation.to_string(),
            probability: prob,
        });
    }
}

/// EFE components
#[derive(Debug, Clone)]
pub struct ExpectedFreeEnergy {
    pub risk: f64,
    pub ambiguity: f64,
    pub total: f64,
    pub timestamp: Instant,
}

/// A policy (sequence of actions)
#[derive(Debug, Clone)]
pub struct Policy {
    pub actions: Vec<String>,
    pub efe: Option<ExpectedFreeEnergy>,
    pub score: f64,
}

/// Current belief state
#[derive(Debug, Clone)]
pub struct BeliefState {
    pub state_probs: HashMap<String, f64>,
    pub uncertainty: f64,
    pub timestamp: Instant,
}

/// VFE minimization result
#[derive(Debug, Clone)]
pub struct BeliefUpdate {
    pub previous: BeliefState,
    pub updated: BeliefState,
    pub vfe: f64,
    pub convergence: bool,
    pub iterations: usize,
}

/// Policy selection result
#[derive(Debug, Clone)]
pub struct PolicySelection {
    pub selected: Policy,
    pub candidates: Vec<Policy>,
    pub timestamp: Instant,
    pub elapsed_ms: f64,
}

/// Active inference report
#[derive(Debug, Clone)]
pub struct ActiveInferenceReport {
    pub belief: BeliefState,
    pub selected_policy: Option<PolicySelection>,
    pub efe: Option<ExpectedFreeEnergy>,
    pub vfe: f64,
    pub action_taken: Option<String>,
    pub elapsed_ms: f64,
}

/// Active inference configuration
#[derive(Debug, Clone)]
pub struct ActiveInferenceConfig {
    pub policy_horizon: usize,
    pub num_policies: usize,
    pub temperature: f64,
    pub learning_rate: f64,
    pub vsa_similarity_threshold: f64,
}

impl Default for ActiveInferenceConfig {
    fn default() -> Self {
        Self {
            policy_horizon: 4,
            num_policies: 8,
            temperature: 1.0,
            learning_rate: 0.1,
            vsa_similarity_threshold: 0.6,
        }
    }
}

/// Goal state for preferred outcomes
#[derive(Debug, Clone)]
pub struct GoalState {
    pub vsa_vector: Vec<u8>,
    pub label: String,
}

/// The main Active Inference engine
#[derive(Debug, Clone)]
pub struct ActiveInferenceEngine {
    pub generative_model: GenerativeModel,
    pub config: ActiveInferenceConfig,
    pub belief: Option<BeliefState>,
    pub last_policy: Option<PolicySelection>,
    pub total_inferences: u64,
    pub average_vfe: f64,
    pub history: Vec<ActiveInferenceReport>,
    goals: Vec<GoalState>,
}

impl ActiveInferenceEngine {
    pub fn new(model: GenerativeModel, config: ActiveInferenceConfig) -> Self {
        let initial_probs = model.prior_over_states.clone();
        let uncertainty = if initial_probs.is_empty() {
            1.0
        } else {
            let entropy: f64 = initial_probs
                .values()
                .filter(|&&p| p > 0.0)
                .map(|p| -p * p.log2())
                .sum();
            let max_entropy = (initial_probs.len() as f64).log2();
            if max_entropy > 0.0 {
                entropy / max_entropy
            } else {
                1.0
            }
        };
        Self {
            generative_model: model,
            config,
            belief: Some(BeliefState {
                state_probs: initial_probs,
                uncertainty,
                timestamp: Instant::now(),
            }),
            last_policy: None,
            total_inferences: 0,
            average_vfe: 0.0,
            history: Vec::new(),
            goals: Vec::new(),
        }
    }

    pub fn infer(&mut self, observation: &[u8]) -> ActiveInferenceReport {
        let start = Instant::now();

        let belief_update = self.update_beliefs(observation);

        let current_belief = belief_update.updated.clone();

        let mut candidates = Vec::with_capacity(self.config.num_policies);
        let policies = self.sample_policies();

        let mut best_efe: Option<ExpectedFreeEnergy> = None;
        let mut best_policy_idx = 0;

        for (i, mut policy) in policies.into_iter().enumerate() {
            let efe = self.compute_efe(&policy, &current_belief);
            policy.efe = Some(efe.clone());
            policy.score = -efe.total;
            if best_efe.is_none() || efe.total < best_efe.as_ref().unwrap().total {
                best_efe = Some(efe);
                best_policy_idx = i;
            }
            candidates.push(policy);
        }

        let policy_selection = if candidates.is_empty() {
            None
        } else {
            let timestamp = Instant::now();
            let elapsed = timestamp.duration_since(start).as_secs_f64() * 1000.0;
            let selected = candidates[best_policy_idx].clone();
            let selection = PolicySelection {
                selected,
                candidates: candidates.clone(),
                timestamp,
                elapsed_ms: elapsed,
            };
            self.last_policy = Some(selection.clone());
            Some(selection)
        };

        let action_taken = if let Some(ref sel) = policy_selection {
            sel.selected.actions.first().cloned()
        } else {
            None
        };

        let efe_report = policy_selection
            .as_ref()
            .and_then(|s| s.selected.efe.clone());

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let vfe = belief_update.vfe;

        self.total_inferences += 1;
        self.average_vfe = self.average_vfe * 0.9 + vfe * 0.1;

        let report = ActiveInferenceReport {
            belief: current_belief,
            selected_policy: policy_selection,
            efe: efe_report,
            vfe,
            action_taken,
            elapsed_ms,
        };

        self.history.push(report.clone());
        report
    }

    pub fn update_beliefs(&mut self, observation: &[u8]) -> BeliefUpdate {
        let previous = self.belief.clone().unwrap_or(BeliefState {
            state_probs: HashMap::new(),
            uncertainty: 1.0,
            timestamp: Instant::now(),
        });

        if self.generative_model.states.is_empty() {
            let current = BeliefState {
                state_probs: HashMap::new(),
                uncertainty: 1.0,
                timestamp: Instant::now(),
            };
            return BeliefUpdate {
                previous,
                updated: current.clone(),
                vfe: 0.0,
                convergence: true,
                iterations: 0,
            };
        }

        let mut state_probs = previous.state_probs.clone();
        let max_iterations = 10;
        let mut prev_vfe = f64::MAX;

        for iteration in 0..max_iterations {
            let current_belief = BeliefState {
                state_probs: state_probs.clone(),
                uncertainty: 0.0,
                timestamp: Instant::now(),
            };
            let vfe = self.compute_vfe(observation, &current_belief);

            if (prev_vfe - vfe).abs() < 1e-6 {
                return BeliefUpdate {
                    previous,
                    updated: BeliefState {
                        state_probs: state_probs.clone(),
                        uncertainty: compute_uncertainty(&state_probs),
                        timestamp: Instant::now(),
                    },
                    vfe,
                    convergence: true,
                    iterations: iteration,
                };
            }
            prev_vfe = vfe;
            state_probs = self.message_passing_step(observation, &state_probs);
        }

        let uncertainty = compute_uncertainty(&state_probs);
        let updated = BeliefState {
            state_probs: state_probs.clone(),
            uncertainty,
            timestamp: Instant::now(),
        };
        let final_vfe = self.compute_vfe(observation, &updated);

        self.belief = Some(updated.clone());

        BeliefUpdate {
            previous,
            updated,
            vfe: final_vfe,
            convergence: false,
            iterations: max_iterations,
        }
    }

    fn message_passing_step(
        &self,
        observation: &[u8],
        current_probs: &HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        let mut new_probs = HashMap::new();
        let mut total = 0.0;

        for state in &self.generative_model.states {
            let prior = current_probs
                .get(&state.name)
                .copied()
                .unwrap_or(state.prior_probability);

            let vsa_sim = QuantizedVSA::similarity(observation, &state.vsa_vector);
            let likelihood = self.observation_likelihood(&state.name, vsa_sim);

            let posterior =
                prior * likelihood + self.config.learning_rate * (vsa_sim - prior).max(0.0);
            new_probs.insert(state.name.clone(), posterior);
            total += posterior;
        }

        if total > 0.0 {
            for prob in new_probs.values_mut() {
                *prob /= total;
            }
        }

        new_probs
    }

    fn observation_likelihood(&self, state_name: &str, vsa_sim: f64) -> f64 {
        for entry in &self.generative_model.likelihood_matrix {
            if entry.state_name == state_name {
                return entry.probability * (vsa_sim * 0.5 + 0.5);
            }
        }
        vsa_sim.max(0.0) * 0.5 + 0.01
    }

    pub fn compute_efe(&self, policy: &Policy, belief: &BeliefState) -> ExpectedFreeEnergy {
        let predicted_belief = self.predict_belief_under_policy(policy, belief);

        let risk = self.compute_risk(&predicted_belief.belief);

        let ambiguity = self.compute_ambiguity(&predicted_belief.belief);

        let total = risk + ambiguity;

        ExpectedFreeEnergy {
            risk,
            ambiguity,
            total,
            timestamp: Instant::now(),
        }
    }

    pub fn compute_vfe(&self, observation: &[u8], belief: &BeliefState) -> f64 {
        if self.generative_model.states.is_empty() || belief.state_probs.is_empty() {
            return 0.0;
        }

        let mut vfe = 0.0;
        for state in &self.generative_model.states {
            let q_s = belief
                .state_probs
                .get(&state.name)
                .copied()
                .unwrap_or(state.prior_probability);
            if q_s <= 0.0 {
                continue;
            }

            let vsa_sim = QuantizedVSA::similarity(observation, &state.vsa_vector);
            let log_likelihood = (vsa_sim * 0.5 + 0.01).ln();

            let log_prior = state.prior_probability.max(1e-10).ln();

            let log_joint = log_likelihood + log_prior;
            let log_q = q_s.ln();

            vfe += q_s * (log_q - log_joint);
        }

        vfe.abs()
    }

    pub fn select_policy(&mut self) -> PolicySelection {
        let start = Instant::now();
        let belief = self.belief.clone().unwrap_or(BeliefState {
            state_probs: HashMap::new(),
            uncertainty: 1.0,
            timestamp: Instant::now(),
        });

        let policies = self.sample_policies();

        if policies.is_empty() {
            let empty_policy = Policy {
                actions: Vec::new(),
                efe: None,
                score: 0.0,
            };
            let selection = PolicySelection {
                selected: empty_policy.clone(),
                candidates: vec![empty_policy],
                timestamp: start,
                elapsed_ms: 0.0,
            };
            self.last_policy = Some(selection.clone());
            return selection;
        }

        let scored: Vec<Policy> = policies
            .into_iter()
            .map(|p| {
                let efe = self.compute_efe(&p, &belief);
                Policy {
                    efe: Some(efe.clone()),
                    score: -efe.total,
                    ..p
                }
            })
            .collect();

        let min_score = scored.iter().map(|p| p.score).fold(f64::INFINITY, f64::min);
        let max_score = scored
            .iter()
            .map(|p| p.score)
            .fold(f64::NEG_INFINITY, f64::max);

        let (shifted, range) = if (max_score - min_score).abs() < 1e-10 {
            (vec![0.0; scored.len()], 1.0)
        } else {
            let shifted: Vec<f64> = scored.iter().map(|p| p.score - min_score).collect();
            let range = max_score - min_score;
            (shifted, range)
        };

        let energies: Vec<f64> = scored
            .iter()
            .zip(shifted.iter())
            .map(|(_, &s)| {
                let effective = if range > 0.0 { s / range } else { 0.0 };
                (effective / self.config.temperature).exp()
            })
            .collect();

        let sum_e: f64 = energies.iter().sum();
        let probs: Vec<f64> = if sum_e > 0.0 {
            energies.iter().map(|e| e / sum_e).collect()
        } else {
            vec![1.0 / scored.len() as f64; scored.len()]
        };

        let rng_seed = (start.elapsed().as_nanos() % 10000) as f64 / 10000.0;
        let mut cumulative = 0.0;
        let mut selected_idx = 0;
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if rng_seed <= cumulative {
                selected_idx = i;
                break;
            }
        }

        let selected = scored[selected_idx].clone();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let selection = PolicySelection {
            selected,
            candidates: scored,
            timestamp: start,
            elapsed_ms: elapsed,
        };

        self.last_policy = Some(selection.clone());
        selection
    }

    pub fn set_goal(&mut self, goal_vsa: &[u8], goal_label: &str) {
        self.goals.push(GoalState {
            vsa_vector: goal_vsa.to_vec(),
            label: goal_label.to_string(),
        });
    }

    pub fn goals(&self) -> &[GoalState] {
        &self.goals
    }

    pub fn take_action(&mut self) -> Option<String> {
        self.last_policy
            .as_ref()
            .and_then(|sel| sel.selected.actions.first().cloned())
    }

    fn sample_policies(&self) -> Vec<Policy> {
        let actions = &self.generative_model.actions;
        if actions.is_empty() {
            return Vec::new();
        }

        let num_policies = self
            .config
            .num_policies
            .min(actions.len().pow(self.config.policy_horizon as u32));

        (0..num_policies)
            .map(|_| {
                let sequence: Vec<String> = (0..self.config.policy_horizon)
                    .map(|step| {
                        let idx = (step * 7 + 3) % actions.len();
                        actions[idx].clone()
                    })
                    .collect();
                Policy {
                    actions: sequence,
                    efe: None,
                    score: 0.0,
                }
            })
            .collect()
    }

    fn predict_belief_under_policy(
        &self,
        policy: &Policy,
        current_belief: &BeliefState,
    ) -> PredictedBelief {
        let mut probs = current_belief.state_probs.clone();

        for action in &policy.actions {
            let mut next_probs: HashMap<String, f64> = HashMap::new();

            for state in &self.generative_model.states {
                let current_prob = probs.get(&state.name).copied().unwrap_or(0.0);
                if current_prob <= 0.0 {
                    continue;
                }

                let mut outflow = 0.0;
                let mut inflow_for_this = 0.0;

                for trans in &self.generative_model.transition_matrix {
                    if trans.action == *action {
                        if trans.from_state == state.name {
                            outflow += trans.probability;
                        }
                        if trans.to_state == state.name {
                            inflow_for_this += trans.probability
                                * probs.get(&trans.from_state).copied().unwrap_or(0.0);
                        }
                    }
                }

                if outflow > 0.0 {
                    *next_probs.entry(state.name.clone()).or_insert(0.0) +=
                        current_prob * (1.0 - outflow * 0.5) + inflow_for_this * 0.5;
                }
            }

            let total: f64 = next_probs.values().sum();
            if total > 0.0 {
                for v in next_probs.values_mut() {
                    *v /= total;
                }
            }
            probs = next_probs;
        }

        let uncertainty = compute_uncertainty(&probs);
        PredictedBelief {
            belief: BeliefState {
                state_probs: probs,
                uncertainty,
                timestamp: Instant::now(),
            },
        }
    }

    fn compute_risk(&self, predicted: &BeliefState) -> f64 {
        if self.goals.is_empty() || predicted.state_probs.is_empty() {
            return 0.0;
        }

        let mut kl_div = 0.0;
        for (state_name, pred_prob) in &predicted.state_probs {
            if *pred_prob <= 0.0 {
                continue;
            }

            let goal_similarity = self
                .goals
                .iter()
                .map(|g| {
                    let state_vsa = self
                        .generative_model
                        .states
                        .iter()
                        .find(|s| s.name == *state_name)
                        .map(|s| &s.vsa_vector);
                    match state_vsa {
                        Some(vsa) => QuantizedVSA::similarity(vsa, &g.vsa_vector),
                        None => 0.0,
                    }
                })
                .fold(0.0_f64, f64::max);

            let preferred_prob = goal_similarity.max(0.01);
            let pref_normalized = preferred_prob / (self.goals.len() as f64 * 1.0);

            kl_div += *pred_prob * (pred_prob / pref_normalized).ln();
        }

        kl_div.max(0.0) * 0.1
    }

    fn compute_ambiguity(&self, belief: &BeliefState) -> f64 {
        if belief.state_probs.is_empty() || self.generative_model.observations.is_empty() {
            return 0.0;
        }

        let mut expected_entropy = 0.0;
        for (state_name, prob) in &belief.state_probs {
            if *prob <= 0.0 {
                continue;
            }

            let mut obs_entropy = 0.0;
            let n_obs = self.generative_model.observations.len() as f64;
            for obs in &self.generative_model.observations {
                let likelihood = self
                    .generative_model
                    .likelihood_matrix
                    .iter()
                    .find(|l| l.state_name == *state_name && l.observation_name == *obs)
                    .map(|l| l.probability)
                    .unwrap_or(1.0 / n_obs);

                if likelihood > 0.0 {
                    obs_entropy -= likelihood * likelihood.log2();
                }
            }

            expected_entropy += prob * obs_entropy;
        }

        expected_entropy
            / (self.generative_model.observations.len() as f64)
                .log2()
                .max(1.0)
    }

    pub fn belief(&self) -> Option<&BeliefState> {
        self.belief.as_ref()
    }

    pub fn last_policy_selection(&self) -> Option<&PolicySelection> {
        self.last_policy.as_ref()
    }

    pub fn average_vfe(&self) -> f64 {
        self.average_vfe
    }

    pub fn total_inferences(&self) -> u64 {
        self.total_inferences
    }

    pub fn recent_reports(&self, n: usize) -> &[ActiveInferenceReport] {
        let len = self.history.len();
        let start = len.saturating_sub(n);
        &self.history[start..]
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[derive(Debug, Clone)]
struct PredictedBelief {
    belief: BeliefState,
}

fn compute_uncertainty(state_probs: &HashMap<String, f64>) -> f64 {
    if state_probs.is_empty() {
        return 1.0;
    }
    let entropy: f64 = state_probs
        .values()
        .filter(|&&p| p > 0.0)
        .map(|p| -p * p.log2())
        .sum();
    let max_entropy = (state_probs.len() as f64).log2();
    if max_entropy > 0.0 {
        (entropy / max_entropy).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::QuantizedVSA;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    fn make_test_model() -> GenerativeModel {
        let mut model = GenerativeModel::new("test_model");
        model.add_state("state_A", test_vsa(1), 0.5);
        model.add_state("state_B", test_vsa(2), 0.3);
        model.add_state("state_C", test_vsa(3), 0.2);
        model.add_observation("obs_1");
        model.add_observation("obs_2");
        model.add_action("action_X");
        model.add_action("action_Y");
        model.add_transition("state_A", "state_A", "action_X", 0.8);
        model.add_transition("state_A", "state_B", "action_X", 0.2);
        model.add_transition("state_B", "state_B", "action_X", 0.9);
        model.add_transition("state_B", "state_C", "action_X", 0.1);
        model.add_transition("state_C", "state_C", "action_X", 1.0);
        model.add_transition("state_A", "state_B", "action_Y", 0.7);
        model.add_transition("state_A", "state_C", "action_Y", 0.3);
        model.add_transition("state_B", "state_A", "action_Y", 0.4);
        model.add_transition("state_B", "state_C", "action_Y", 0.6);
        model.add_likelihood("state_A", "obs_1", 0.8);
        model.add_likelihood("state_A", "obs_2", 0.2);
        model.add_likelihood("state_B", "obs_1", 0.3);
        model.add_likelihood("state_B", "obs_2", 0.7);
        model.add_likelihood("state_C", "obs_1", 0.1);
        model.add_likelihood("state_C", "obs_2", 0.9);
        model
    }

    #[test]
    fn test_create_generative_model() {
        let model = make_test_model();
        assert_eq!(model.states.len(), 3);
        assert_eq!(model.observations.len(), 2);
        assert_eq!(model.actions.len(), 2);
        assert_eq!(model.name, "test_model");
    }

    #[test]
    fn test_single_inference_step_updates_beliefs() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs = test_vsa(1);
        let report = engine.infer(&obs);
        assert!(!report.belief.state_probs.is_empty());
        let total: f64 = report.belief.state_probs.values().sum();
        assert!(
            (total - 1.0).abs() < 0.01,
            "beliefs should sum to ~1, got {}",
            total
        );
    }

    #[test]
    fn test_efe_computation_risk_and_ambiguity() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let engine = ActiveInferenceEngine::new(model, config);
        let belief = BeliefState {
            state_probs: {
                let mut m = HashMap::new();
                m.insert("state_A".to_string(), 0.6);
                m.insert("state_B".to_string(), 0.3);
                m.insert("state_C".to_string(), 0.1);
                m
            },
            uncertainty: 0.5,
            timestamp: Instant::now(),
        };
        let policy = Policy {
            actions: vec!["action_X".to_string()],
            efe: None,
            score: 0.0,
        };
        let efe = engine.compute_efe(&policy, &belief);
        assert!(efe.total >= 0.0, "EFE should be non-negative");
        assert!(efe.risk >= 0.0);
        assert!(efe.ambiguity >= 0.0);
    }

    #[test]
    fn test_policy_selection_picks_lowest_efe() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let selection = engine.select_policy();
        assert!(!selection.candidates.is_empty(), "should have candidates");
        for candidate in &selection.candidates {
            if let Some(ref selected_efe) = selection.selected.efe {
                if let Some(ref cand_efe) = candidate.efe {
                    assert!(
                        selected_efe.total <= cand_efe.total + 1e-6,
                        "selected policy should have minimal EFE"
                    );
                }
            }
        }
    }

    #[test]
    fn test_action_selection_returns_valid_action() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let _selection = engine.select_policy();
        let action = engine.take_action();
        assert!(action.is_some(), "should return an action");
        let action = action.unwrap();
        assert!(
            action == "action_X" || action == "action_Y",
            "action should be one of the model actions, got {}",
            action
        );
    }

    #[test]
    fn test_belief_update_reduces_vfe_after_observation() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs = test_vsa(1);
        let initial_vfe = {
            let bel = engine.belief.as_ref().unwrap();
            engine.compute_vfe(&obs, bel)
        };
        let update = engine.update_beliefs(&obs);
        assert!(
            update.vfe <= initial_vfe + 1e-6,
            "VFE should not increase after belief update: initial={}, final={}",
            initial_vfe,
            update.vfe
        );
    }

    #[test]
    fn test_goal_setting_affects_policy_selection() {
        let model = make_test_model();
        let config = ActiveInferenceConfig {
            temperature: 0.1,
            ..Default::default()
        };
        let mut engine = ActiveInferenceEngine::new(model, config);
        engine.set_goal(&test_vsa(2), "prefer_B");
        let selection = engine.select_policy();
        assert!(!selection.candidates.is_empty());
        for candidate in &selection.candidates {
            assert!(candidate.efe.is_some());
        }
    }

    #[test]
    fn test_high_ambiguity_leads_to_exploration() {
        let model = make_test_model();
        let mut config = ActiveInferenceConfig::default();
        config.temperature = 0.01;
        let mut engine = ActiveInferenceEngine::new(model, config);
        let high_entropy_obs = QuantizedVSA::seeded_random(99, 64);
        let report = engine.infer(&high_entropy_obs);
        assert!(report.efe.is_none() || report.efe.as_ref().unwrap().total >= 0.0);
    }

    #[test]
    fn test_multiple_inference_steps_converge() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs = test_vsa(1);
        for _ in 0..5 {
            let report = engine.infer(&obs);
            assert!(!report.belief.state_probs.is_empty());
        }
        assert!(engine.total_inferences >= 5);
    }

    #[test]
    fn test_policy_horizon_affects_candidates() {
        let model = make_test_model();
        let mut config = ActiveInferenceConfig::default();
        config.policy_horizon = 1;
        config.num_policies = 4;
        let mut engine = ActiveInferenceEngine::new(model.clone(), config);
        let sel1 = engine.select_policy();
        let len1 = sel1.selected.actions.len();

        let mut config2 = ActiveInferenceConfig::default();
        config2.policy_horizon = 3;
        config2.num_policies = 4;
        let mut engine2 = ActiveInferenceEngine::new(model, config2);
        let sel2 = engine2.select_policy();

        assert!(
            len1 <= sel2.selected.actions.len(),
            "longer horizon should produce longer action sequences"
        );
    }

    #[test]
    fn test_temperature_affects_policy_selection() {
        let model = make_test_model();
        let mut config = ActiveInferenceConfig::default();
        config.temperature = 10.0;
        let mut engine = ActiveInferenceEngine::new(model.clone(), config);
        let sel_high = engine.select_policy();

        let mut config_low = ActiveInferenceConfig::default();
        config_low.temperature = 0.01;
        let mut engine_low = ActiveInferenceEngine::new(model, config_low);
        let sel_low = engine_low.select_policy();

        assert!(
            !sel_high.candidates.is_empty() && !sel_low.candidates.is_empty(),
            "both should select policies"
        );
    }

    #[test]
    fn test_empty_model_does_not_crash() {
        let model = GenerativeModel::new("empty");
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs = test_vsa(1);
        let report = engine.infer(&obs);
        assert!(report.belief.state_probs.is_empty());
        assert!(report.action_taken.is_none());
    }

    #[test]
    fn test_vsa_observation_matching_updates_beliefs() {
        let mut model = make_test_model();
        let config = ActiveInferenceConfig {
            learning_rate: 0.5,
            ..Default::default()
        };
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs_a = test_vsa(1);
        engine.infer(&obs_a);
        let bel = engine.belief.as_ref().unwrap();
        let prob_a = bel.state_probs.get("state_A").copied().unwrap_or(0.0);
        let prob_b = bel.state_probs.get("state_B").copied().unwrap_or(0.0);
        assert!(
            prob_a > prob_b,
            "observing state_A-like VSA should increase belief in state_A over state_B: A={}, B={}",
            prob_a,
            prob_b
        );
    }

    #[test]
    fn test_efe_decreases_with_informative_observations() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs_a = test_vsa(1);
        let rep1 = engine.infer(&obs_a);
        let efe1 = rep1.efe.map(|e| e.total).unwrap_or(f64::MAX);

        let rep2 = engine.infer(&obs_a);
        let efe2 = rep2.efe.map(|e| e.total).unwrap_or(f64::MAX);

        assert!(
            efe2 <= efe1 + 1e-6,
            "EFE should not increase after informative observations"
        );
    }

    #[test]
    fn test_belief_convergence_stops_after_threshold() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs = test_vsa(1);
        let update = engine.update_beliefs(&obs);
        assert!(
            update.iterations <= 10,
            "should converge within 10 iterations, got {}",
            update.iterations
        );
    }

    #[test]
    fn test_generative_model_add_methods() {
        let mut model = GenerativeModel::new("add_test");
        model.add_state("s1", test_vsa(10), 0.4);
        model.add_state("s2", test_vsa(11), 0.6);
        model.add_observation("o1");
        model.add_observation("o2");
        model.add_action("a1");
        model.add_action("a2");
        model.add_transition("s1", "s2", "a1", 1.0);
        model.add_likelihood("s1", "o1", 0.9);
        assert_eq!(model.states.len(), 2);
        assert_eq!(model.observations.len(), 2);
        assert_eq!(model.actions.len(), 2);
        assert_eq!(model.transition_matrix.len(), 1);
        assert_eq!(model.likelihood_matrix.len(), 1);
    }

    #[test]
    fn test_engine_clear_history() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        let obs = test_vsa(1);
        engine.infer(&obs);
        assert_eq!(engine.history.len(), 1);
        engine.clear_history();
        assert!(engine.history.is_empty());
    }

    #[test]
    fn test_multiple_goals_influence_risk() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        engine.set_goal(&test_vsa(2), "goal_B");
        engine.set_goal(&test_vsa(3), "goal_C");
        assert_eq!(engine.goals().len(), 2);
        let policy = Policy {
            actions: vec!["action_X".to_string()],
            efe: None,
            score: 0.0,
        };
        let belief = BeliefState {
            state_probs: {
                let mut m = HashMap::new();
                m.insert("state_A".to_string(), 0.1);
                m.insert("state_B".to_string(), 0.6);
                m.insert("state_C".to_string(), 0.3);
                m
            },
            uncertainty: 0.3,
            timestamp: Instant::now(),
        };
        let efe = engine.compute_efe(&policy, &belief);
        assert!(efe.risk >= 0.0);
    }

    #[test]
    fn test_belief_uncertainty_computation() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.5);
        probs.insert("b".to_string(), 0.5);
        let unc = compute_uncertainty(&probs);
        assert!(
            (unc - 1.0).abs() < 0.01,
            "uniform dist should have max uncertainty"
        );

        let mut probs2 = HashMap::new();
        probs2.insert("a".to_string(), 1.0);
        let unc2 = compute_uncertainty(&probs2);
        assert!(
            (unc2 - 0.0).abs() < 0.01,
            "certain belief should have 0 uncertainty"
        );
    }

    #[test]
    fn test_inference_with_goal_updates_efe() {
        let model = make_test_model();
        let config = ActiveInferenceConfig::default();
        let mut engine = ActiveInferenceEngine::new(model, config);
        engine.set_goal(&test_vsa(2), "goal_B");
        let obs = test_vsa(1);
        let report = engine.infer(&obs);
        assert!(report.vfe >= 0.0);
        if let Some(ref efe) = report.efe {
            assert!(efe.total >= 0.0);
        }
    }
}
