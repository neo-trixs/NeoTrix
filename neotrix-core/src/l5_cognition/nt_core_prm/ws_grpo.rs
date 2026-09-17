use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::nt_core_policy::E8Policy;
use super::step_grpo::extract_task_type;

/// Weakly-supervised preference model that learns per-task-type reward expectations.
///
/// Maintains a running average of rewards for each extracted task type and
/// exposes the learned preference as a score in [0, 1].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsPreferenceModel {
    /// (task description, final reward) history
    pub trajectory_outcomes: Vec<(String, f64)>,
    /// Learned per-task-type preference scores
    pub preference_scores: HashMap<String, f64>,
    /// Learning rate for exponential moving average (default 0.1)
    pub learning_rate: f64,
    /// Maximum number of stored outcomes (default 100)
    pub max_history: usize,
}

impl WsPreferenceModel {
    pub fn new(learning_rate: f64, max_history: usize) -> Self {
        Self {
            trajectory_outcomes: Vec::new(),
            preference_scores: HashMap::new(),
            learning_rate,
            max_history,
        }
    }

    /// Record a trajectory outcome and update the preference score for its task type.
    ///
    /// Extracts the task type from `task`, stores the outcome pair, then
    /// updates the preference score as an exponential moving average toward
    /// the running mean reward for that type.
    pub fn record_outcome(&mut self, task: &str, reward: f64) {
        let task_type = extract_task_type(task);
        self.trajectory_outcomes.push((task_type.clone(), reward));
        if self.trajectory_outcomes.len() > self.max_history {
            self.trajectory_outcomes.remove(0);
        }
        let outcomes: Vec<f64> = self
            .trajectory_outcomes
            .iter()
            .filter(|(t, _)| t == &task_type)
            .map(|(_, r)| *r)
            .collect();
        let avg = if outcomes.is_empty() {
            0.5
        } else {
            outcomes.iter().sum::<f64>() / outcomes.len() as f64
        };
        let entry = self.preference_scores.entry(task_type).or_insert(0.5);
        *entry = *entry + self.learning_rate * (avg - *entry);
    }

    /// Return the preference score for the task type extracted from `task`.
    ///
    /// Returns 0.5 (neutral) for unknown task types.
    pub fn preference_score(&self, task: &str) -> f64 {
        let task_type = extract_task_type(task);
        self.preference_scores
            .get(&task_type)
            .copied()
            .unwrap_or(0.5)
    }
}

/// A prefix-level reward with a weak supervision signal from the preference model.
#[derive(Debug, Clone)]
pub struct WsPrefixReward {
    /// Step index this reward applies to
    pub step_idx: usize,
    /// Reward assigned to the prefix leading to this step
    pub prefix_reward: f64,
    /// The preference model's contribution to this step
    pub preference_signal: f64,
}

/// WS-GRPO learner that blends weak supervision signals with λ-GRPO.
///
/// Wraps a `ProcessRewardLearner` and a `WsPreferenceModel`. Each learning step:
/// 1. Collects trajectories via the provided closure
/// 2. Extracts task types and queries the preference model
/// 3. Computes prefix-level pseudo-rewards: each step gets `preference_signal * 0.2`
/// 4. Blends with λ-GRPO advantages via z-score normalization
/// 5. Updates the preference model with observed outcomes
/// 6. Feeds the blended advantages into the policy via `learn_from_scores`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsGrpoLearner {
    pub preference_model: WsPreferenceModel,
    #[serde(skip)]
    pub inner: ProcessRewardLearner,
    pub results: Vec<LambdaGrpoResult>,
}

impl WsGrpoLearner {
    pub fn new(
        policy: E8Policy,
        coach: Box<dyn Coach>,
        learning_rate: f64,
        max_history: usize,
    ) -> Self {
        Self {
            preference_model: WsPreferenceModel::new(learning_rate, max_history),
            inner: ProcessRewardLearner::new(policy, coach),
            results: Vec::new(),
        }
    }

    /// Run one WS-GRPO learning step.
    ///
    /// `collect_fn` populates the inner `TrajectoryCollector`. Returns the base
    /// λ-GRPO results (pre-blending) for convergence tracking.
    pub fn learn_step_ws<F>(&mut self, collect_fn: F) -> Vec<LambdaGrpoResult>
    where
        F: FnOnce(&mut TrajectoryCollector),
    {
        collect_fn(&mut self.inner.collector);
        let trajectories: Vec<AgentTrajectory> = self.inner.collector.collected.drain(..).collect();

        // Step 1: Compute base λ-GRPO advantages
        let config = LambdaGrpoConfig::default();
        let step_results = lambda_grpo_loss(&trajectories, &config);

        // Steps 2-4: For each trajectory, extract task type, get preference,
        // compute prefix rewards, blend with λ-GRPO via z-score normalization,
        // and feed into policy
        for (result, traj) in step_results.iter().zip(trajectories.iter()) {
            let pref_score = self.preference_model.preference_score(&traj.task);

            // Step 3: Prefix-level pseudo-rewards: each step gets preference_signal * 0.2
            let prefix_rewards: Vec<WsPrefixReward> = traj
                .steps
                .iter()
                .enumerate()
                .map(|(i, _)| WsPrefixReward {
                    step_idx: i,
                    prefix_reward: pref_score * 0.2,
                    preference_signal: pref_score,
                })
                .collect();

            // Step 4: Blend λ-GRPO advantages with WS prefix rewards, z-score normalize
            let blended: Vec<f64> = result
                .step_advantages
                .iter()
                .zip(prefix_rewards.iter())
                .map(|(sa, pr)| sa.advantage + pr.prefix_reward)
                .collect();

            let normalized = zscore_normalize(&blended);

            // Step 6: Feed normalized advantages into policy
            let mode_scores: Vec<_> = traj
                .steps
                .iter()
                .zip(normalized.iter())
                .enumerate()
                .map(|(i, (step, adv))| (i, step.e8_mode, *adv))
                .collect();
            if !mode_scores.is_empty() {
                let process_scores: Vec<ProcessScore> = mode_scores
                    .iter()
                    .map(|&(orig_idx, _mode, adv)| ProcessScore {
                        step_idx: orig_idx,
                        score: ((adv + 1.0) / 2.0).max(0.0).min(1.0),
                        confidence: 0.5,
                        criteria: Vec::new(),
                        attribution_tags: vec!["ws_grpo".to_string()],
                    })
                    .collect();
                self.inner.policy.learn_from_scores(traj, &process_scores);
            }
        }

        // Step 5: Update preference model with trajectory outcomes
        for traj in &trajectories {
            let reward = traj
                .outcome_reward
                .unwrap_or(if traj.completed { 1.0 } else { 0.0 });
            self.preference_model.record_outcome(&traj.task, reward);
        }

        self.results.extend(step_results.clone());
        self.inner.learning_count += 1;

        step_results
    }
}

// ═══════════════════════════════════════════════════════════════════
// GroundedPRM: MCTS Tree-Guided Step Verification
// ═══════════════════════════════════════════════════════════════════


