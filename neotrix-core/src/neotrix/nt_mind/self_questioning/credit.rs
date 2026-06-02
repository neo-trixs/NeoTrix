use std::collections::HashSet;

use super::types::ExplorationTrajectory;
use neotrix_types::CapabilityVector;

pub struct StepCredit {
    pub step_index: usize,
    pub action: String,
    pub state_before: String,
    pub state_after: String,
    pub attribution: f64,
    pub rationale: String,
    pub advantage: f64,
    pub key_tokens: Vec<String>,
}

pub struct CreditAssignment {
    pub trajectory_id: String,
    pub step_credits: Vec<StepCredit>,
    pub composite_reward: f64,
    pub outcome_score: f64,
    pub positive_steps: Vec<usize>,
    pub negative_steps: Vec<usize>,
}

pub struct AdcaCreditAssigner;

impl AdcaCreditAssigner {
    pub fn assign(
        trajectory: &ExplorationTrajectory,
        outcome_score: f64,
    ) -> CreditAssignment {
        let mut step_credits = Self::llm_attribution(trajectory);
        let mean_attr = if step_credits.is_empty() {
            0.0
        } else {
            step_credits.iter().map(|s| s.attribution).sum::<f64>() / step_credits.len() as f64
        };
        for step in &mut step_credits {
            step.advantage = Self::calculate_advantage(step, mean_attr);
        }
        let composite_reward = Self::composite_reward(&step_credits, outcome_score, trajectory);
        let positive_steps: Vec<usize> = step_credits.iter()
            .filter(|s| s.advantage > 0.0)
            .map(|s| s.step_index)
            .collect();
        let negative_steps: Vec<usize> = step_credits.iter()
            .filter(|s| s.advantage < 0.0)
            .map(|s| s.step_index)
            .collect();
        let trajectory_id = format!(
            "traj-b{}-d{}",
            trajectory.breadth_phase_steps,
            trajectory.depth_phase_steps,
        );
        CreditAssignment {
            trajectory_id,
            step_credits,
            composite_reward,
            outcome_score,
            positive_steps,
            negative_steps,
        }
    }

    fn llm_attribution(trajectory: &ExplorationTrajectory) -> Vec<StepCredit> {
        let total_steps = trajectory.states.len();
        if total_steps == 0 {
            return Vec::new();
        }
        let mut credits = Vec::with_capacity(total_steps);
        let mut seen_actions: HashSet<String> = HashSet::new();
        for i in 0..total_steps {
            let action = trajectory.actions.get(i).cloned().unwrap_or_default();
            let state_before = if i > 0 {
                trajectory.states[i - 1].clone()
            } else {
                String::new()
            };
            let state_after = trajectory.states[i].clone();
            let state_changed = state_before != state_after;
            let is_repeat = !seen_actions.insert(action.clone());
            let mut score = 0.5;
            if state_changed {
                score += 0.3;
            } else {
                score -= 0.2;
            }
            if is_repeat {
                score -= 0.25;
            }
            let position_ratio = i as f64 / total_steps as f64;
            if position_ratio < 0.3 {
                score += 0.15 * (1.0 - position_ratio / 0.3);
            }
            if position_ratio >= 0.8 {
                score += 0.2 * (position_ratio - 0.8) / 0.2;
            }
            let attribution = score.clamp(0.0, 1.0);
            let rationale = if state_changed {
                format!("State changed from '{}' to '{}'", state_before, state_after)
            } else {
                "No state change detected".to_string()
            };
            let key_tokens: Vec<String> = action.split_whitespace()
                .map(|t| t.to_string())
                .collect();
            credits.push(StepCredit {
                step_index: i,
                action,
                state_before,
                state_after,
                attribution,
                rationale,
                advantage: 0.0,
                key_tokens,
            });
        }
        credits
    }

    fn calculate_advantage(step: &StepCredit, baseline: f64) -> f64 {
        step.attribution - baseline
    }

    fn composite_reward(steps: &[StepCredit], outcome_score: f64, trajectory: &ExplorationTrajectory) -> f64 {
        let mean_attr = if steps.is_empty() {
            0.0
        } else {
            steps.iter().map(|s| s.attribution).sum::<f64>() / steps.len() as f64
        };
        let total_actions = trajectory.actions.len() as f64;
        let unique_actions: HashSet<&str> = trajectory.actions.iter().map(|a| a.as_str()).collect();
        let task_diversity = if total_actions > 0.0 {
            unique_actions.len() as f64 / total_actions
        } else {
            0.0
        };
        0.5 * mean_attr + 0.3 * outcome_score + 0.2 * task_diversity
    }

    pub fn apply_to_capability(
        assignment: &CreditAssignment,
        capability: &mut CapabilityVector,
        learning_rate: f64,
    ) {
        let num_positive = assignment.positive_steps.len() as f64;
        let num_negative = assignment.negative_steps.len() as f64;
        let total = (num_positive + num_negative).max(1.0);
        let net_signal = (num_positive - num_negative) / total;
        let update = learning_rate * net_signal * assignment.composite_reward;
        for val in capability.arr_mut().iter_mut() {
            *val += update;
        }
        capability.normalize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::self_questioning::types::{EnvironmentProfile, EntityDesc, OperationDesc};

    fn make_trajectory(states: Vec<&str>, actions: Vec<&str>) -> ExplorationTrajectory {
        ExplorationTrajectory {
            env_profile: EnvironmentProfile {
                domain: "test".into(),
                entities: vec![EntityDesc {
                    name: "T".into(),
                    attributes: vec!["a".into()],
                    parent: None,
                }],
                operations: vec![OperationDesc {
                    name: "op".into(),
                    input_params: vec!["p".into()],
                    output_effect: "eff".into(),
                }],
                constraints: vec![],
            },
            states: states.into_iter().map(String::from).collect(),
            actions: actions.into_iter().map(String::from).collect(),
            observations: vec![],
            breadth_phase_steps: 0,
            depth_phase_steps: 0,
        }
    }

    #[test]
    fn test_assign_returns_valid_credit() {
        let traj = make_trajectory(
            vec!["s0", "s1", "s2"],
            vec!["examine(A, b)", "refine(X, y)", "verify(result)"],
        );
        let assignment = AdcaCreditAssigner::assign(&traj, 0.85);
        assert_eq!(assignment.step_credits.len(), 3);
        assert!(assignment.composite_reward > 0.0);
        assert_eq!(assignment.positive_steps.len(), 3);
        assert!(assignment.negative_steps.is_empty());
    }

    #[test]
    fn test_repeat_action_negative() {
        let traj = make_trajectory(
            vec!["s0", "s1", "s1"],
            vec!["examine(A, b)", "examine(A, b)", "examine(A, b)"],
        );
        let assignment = AdcaCreditAssigner::assign(&traj, 0.5);
        assert!(assignment.negative_steps.contains(&2));
    }

    #[test]
    fn test_no_state_change_penalty() {
        let traj = make_trajectory(
            vec!["s0", "s0", "s0"],
            vec!["step1", "step2", "step3"],
        );
        let assignment = AdcaCreditAssigner::assign(&traj, 0.5);
        assert!(assignment.composite_reward < 0.8);
    }

    #[test]
    fn test_primacy_higher_attribution() {
        let traj = make_trajectory(
            vec!["s0", "s1", "s2"],
            vec!["first", "second", "third"],
        );
        let credits = AdcaCreditAssigner::llm_attribution(&traj);
        assert!(credits[0].attribution >= credits[1].attribution);
    }

    #[test]
    fn test_recency_boost_for_final_steps() {
        let mut states = Vec::new();
        let mut actions = Vec::new();
        for i in 0..10 {
            states.push(format!("s{}", i));
            actions.push(format!("action{}", i));
        }
        let traj = ExplorationTrajectory {
            env_profile: EnvironmentProfile {
                domain: "test".into(),
                entities: vec![],
                operations: vec![],
                constraints: vec![],
            },
            states,
            actions,
            observations: vec![],
            breadth_phase_steps: 5,
            depth_phase_steps: 5,
        };
        let credits = AdcaCreditAssigner::llm_attribution(&traj);
        assert!(credits[9].attribution > credits[5].attribution);
    }

    #[test]
    fn test_apply_to_capability_updates_vector() {
        let traj = make_trajectory(
            vec!["s0", "s1", "s2"],
            vec!["examine(A, b)", "refine(X, y)", "verify(result)"],
        );
        let assignment = AdcaCreditAssigner::assign(&traj, 0.9);
        let mut cv = CapabilityVector::default();
        AdcaCreditAssigner::apply_to_capability(&assignment, &mut cv, 0.01);
        assert!(cv.arr().iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_empty_trajectory() {
        let traj = make_trajectory(vec![], vec![]);
        let assignment = AdcaCreditAssigner::assign(&traj, 0.0);
        assert!(assignment.step_credits.is_empty());
        assert_eq!(assignment.composite_reward, 0.0);
    }
}
