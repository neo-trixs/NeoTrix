#![forbid(unsafe_code)]

pub mod belief;
pub mod free_energy;
pub mod generative_model;
pub mod policy;

use serde::{Deserialize, Serialize};

pub use self::belief::pomdp::POMDPBeliefUpdater;
pub use self::free_energy::FreeEnergyCalculator;
pub use self::generative_model::GenerativeModel;
pub use self::policy::PolicyEvaluator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiStepReport {
    pub vfe: f64,
    pub efe: f64,
    pub selected_policy: usize,
    pub belief_entropy: f64,
    pub policy_confidence: f64,
}

pub struct FreeEnergyEngine {
    pub model: GenerativeModel,
    pub belief_updater: POMDPBeliefUpdater,
    pub policy_evaluator: PolicyEvaluator,
    pub free_energy: FreeEnergyCalculator,
}

impl FreeEnergyEngine {
    pub fn new(model: GenerativeModel, horizon: usize) -> Self {
        let num_states = model.num_states;
        FreeEnergyEngine {
            belief_updater: POMDPBeliefUpdater::new(num_states),
            policy_evaluator: PolicyEvaluator::new(num_states, horizon),
            free_energy: FreeEnergyCalculator::new(),
            model,
        }
    }

    pub fn compute_vfe(&self, belief: &[f64], observation: &[f64]) -> f64 {
        self.free_energy
            .compute_vfe(belief, observation, &self.model.likelihood_matrix)
    }

    pub fn compute_efe(&self, belief: &[f64], policy: &[usize], horizon: usize) -> f64 {
        let efe = self.free_energy.compute_efe(
            belief,
            &self.model.transition_matrix,
            &self.model.prior_over_states,
            horizon,
        );
        let policy_efe = self
            .policy_evaluator
            .evaluate_policy(policy, belief, &self.model);
        efe + policy_efe
    }

    pub fn update_belief(&mut self, belief: &mut [f64], observation: &[f64]) {
        if observation.len() == 1 {
            let obs_idx = observation[0] as usize;
            let new_belief = self
                .belief_updater
                .update_belief(belief, obs_idx, &self.model.likelihood_matrix);
            for (b, nb) in belief.iter_mut().zip(new_belief.iter()) {
                *b = *nb;
            }
        }
    }

    pub fn select_policy(
        &self,
        beliefs: &[f64],
        policies: &[Vec<usize>],
        horizon: usize,
    ) -> (usize, f64) {
        let evaluator = PolicyEvaluator::new(self.model.num_states, horizon);
        evaluator.select_best_policy(policies, beliefs, &self.model)
    }

    pub fn run_active_inference_step(
        &mut self,
        belief: &mut [f64],
        observation_idx: usize,
        policies: &[Vec<usize>],
        horizon: usize,
    ) -> AiStepReport {
        let obs_vec = vec![observation_idx as f64];
        let vfe = self.compute_vfe(belief, &obs_vec);

        let (selected_policy, efe) = self.select_policy(belief, policies, horizon);

        let new_belief = self
            .belief_updater
            .update_belief(belief, observation_idx, &self.model.likelihood_matrix);
        for (b, nb) in belief.iter_mut().zip(new_belief.iter()) {
            *b = *nb;
        }

        let belief_entropy = self.belief_updater.belief_entropy(belief);

        let policy_confidence = if !policies.is_empty() && selected_policy < policies.len() {
            1.0 - (efe / (policies.len() as f64)).max(0.0).min(1.0)
        } else {
            0.0
        };

        AiStepReport {
            vfe,
            efe,
            selected_policy,
            belief_entropy,
            policy_confidence: policy_confidence.max(0.0).min(1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_model() -> GenerativeModel {
        let mut model = GenerativeModel::new(3, 2);
        model.transition_matrix = vec![
            vec![0.7, 0.2, 0.1],
            vec![0.3, 0.4, 0.3],
            vec![0.1, 0.2, 0.7],
        ];
        model.likelihood_matrix = vec![
            vec![0.9, 0.1],
            vec![0.5, 0.5],
            vec![0.1, 0.9],
        ];
        model
    }

    #[test]
    fn test_new_engine() {
        let model = make_test_model();
        let engine = FreeEnergyEngine::new(model, 5);
        assert_eq!(engine.model.num_states, 3);
        assert_eq!(engine.model.num_observations, 2);
    }

    #[test]
    fn test_compute_vfe_cycle() {
        let model = make_test_model();
        let engine = FreeEnergyEngine::new(model, 5);
        let belief = vec![0.5, 0.3, 0.2];
        let observation = vec![1.0, 0.0];
        let vfe = engine.compute_vfe(&belief, &observation);
        assert!(vfe.is_finite());
    }

    #[test]
    fn test_update_belief() {
        let mut engine = FreeEnergyEngine::new(make_test_model(), 5);
        let mut belief = vec![0.5, 0.3, 0.2];
        let observation = vec![0.0];
        let before = belief.clone();
        engine.update_belief(&mut belief, &observation);
        let changed = belief
            .iter()
            .zip(before.iter())
            .any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(changed);
        let sum: f64 = belief.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_select_policy() {
        let model = make_test_model();
        let engine = FreeEnergyEngine::new(model, 5);
        let belief = vec![0.5, 0.3, 0.2];
        let policies = vec![vec![0, 0, 0], vec![1, 1, 1], vec![2, 2, 2]];
        let (idx, efe) = engine.select_policy(&belief, &policies, 5);
        assert!(idx < 3);
        assert!(efe.is_finite());
    }

    #[test]
    fn test_full_active_inference_step() {
        let mut engine = FreeEnergyEngine::new(make_test_model(), 5);
        let mut belief = vec![0.5, 0.3, 0.2];
        let policies = vec![vec![0, 0, 0], vec![1, 1, 1], vec![2, 2, 2]];
        let report = engine.run_active_inference_step(&mut belief, 0, &policies, 3);
        assert!(report.vfe.is_finite());
        assert!(report.efe.is_finite());
        assert!(report.selected_policy < 3);
        assert!(report.belief_entropy >= 0.0);
        assert!(report.policy_confidence >= 0.0);
        assert!(report.policy_confidence <= 1.0);
        let sum: f64 = belief.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_multiple_active_inference_steps() {
        let mut engine = FreeEnergyEngine::new(make_test_model(), 5);
        let mut belief = vec![0.5, 0.3, 0.2];
        let policies = vec![vec![0, 0, 0], vec![1, 1, 1], vec![2, 2, 2]];

        for step in 0..3 {
            let report =
                engine.run_active_inference_step(&mut belief, step % 2, &policies, 3);
            assert!(report.vfe.is_finite());
            let sum: f64 = belief.iter().sum();
            assert!((sum - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_compute_efe_with_policy() {
        let model = make_test_model();
        let engine = FreeEnergyEngine::new(model, 5);
        let belief = vec![0.5, 0.3, 0.2];
        let policy = vec![0, 1, 2];
        let efe = engine.compute_efe(&belief, &policy, 3);
        assert!(efe.is_finite());
    }
}
