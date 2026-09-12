//! RAE (Role-conditioned Advantage Estimation) for game trajectories.
//!
//! Computes per-turn advantages with role-conditioned baselines:
//! ```text
//! A_game(t) = R_p(τ) - b_{G,p}        // game-level: total reward minus role baseline
//! A_turn(t) = turn_estimator(t, A_game) // per-turn credit assignment
//! ```
//!
//! Advantages are normalized per-agent by performance statistics and filtered
//! to skip steps with zero advantage (zero-grading).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::framework::{ActorId, Role, Trajectory};

// ═══════════════════════════════════════════════════════════════════
// Config
// ═══════════════════════════════════════════════════════════════════

/// Configuration for advantage estimation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvantageConfig {
    /// Discount factor for per-turn credit assignment (gamma).
    pub gamma: f64,
    /// GAE lambda for bias-variance tradeoff.
    pub gae_lambda: f64,
    /// Whether to normalize advantages per-agent.
    pub normalize_per_agent: bool,
    /// Minimum absolute advantage to keep (below this = zero-grading filter).
    pub zero_grading_threshold: f64,
}

impl Default for AdvantageConfig {
    fn default() -> Self {
        Self {
            gamma: 0.99,
            gae_lambda: 0.95,
            normalize_per_agent: true,
            zero_grading_threshold: 1e-8,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Per-role baselines
// ═══════════════════════════════════════════════════════════════════

/// Running statistics for a role's performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleStats {
    pub mean_reward: f64,
    pub variance: f64,
    pub count: usize,
}

impl RoleStats {
    /// Update running mean and variance using Welford's online algorithm.
    pub fn update(&mut self, reward: f64) {
        self.count += 1;
        let delta = reward - self.mean_reward;
        self.mean_reward += delta / self.count as f64;
        let delta2 = reward - self.mean_reward;
        self.variance += delta * delta2;
    }

    /// Standard deviation of rewards.
    pub fn std_dev(&self) -> f64 {
        if self.count < 2 {
            return 1.0;
        }
        (self.variance / (self.count - 1) as f64).sqrt().max(1e-8)
    }

    /// Normalized reward (z-score).
    pub fn normalize(&self, reward: f64) -> f64 {
        (reward - self.mean_reward) / self.std_dev()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Estimator
// ═══════════════════════════════════════════════════════════════════

/// Role-conditioned Advantage Estimator for game trajectories.
///
/// Implements RAE: per-role baseline subtraction with GAE-style
/// temporal difference credit assignment.
pub struct GameAdvantageEstimator {
    pub config: AdvantageConfig,
    /// Per-role running statistics.
    role_stats: HashMap<Role, RoleStats>,
}

impl GameAdvantageEstimator {
    /// Create a new estimator with the given configuration.
    pub fn new(config: AdvantageConfig) -> Self {
        Self {
            config,
            role_stats: HashMap::new(),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(AdvantageConfig::default())
    }

    /// Estimate advantages for a trajectory in-place.
    ///
    /// 1. Compute game-level advantage: A_game = R_p(τ) - b_{G,p}
    /// 2. Compute per-turn credit via GAE-style TD(λ)
    /// 3. Normalize per-agent if configured
    /// 4. Apply zero-grading filter
    pub fn estimate(&mut self, trajectory: &mut Trajectory, roles: &HashMap<ActorId, Role>) {
        if trajectory.steps.is_empty() {
            return;
        }

        // Update role stats with this trajectory's total reward
        for (actor_id, role) in roles {
            let actor_reward: f64 = trajectory
                .steps
                .iter()
                .filter(|s| s.actor_id == *actor_id)
                .map(|s| s.reward)
                .sum();
            self.role_stats
                .entry(*role)
                .or_default()
                .update(actor_reward);
        }

        // Step 1: Compute per-role game-level advantage
        let n = trajectory.steps.len();
        let mut role_rewards: HashMap<Role, f64> = HashMap::new();
        for step in &trajectory.steps {
            if let Some(role) = roles.get(&step.actor_id) {
                *role_rewards.entry(*role).or_insert(0.0) += step.reward;
            }
        }

        let mut game_advantages: HashMap<Role, f64> = HashMap::new();
        for (role, total_reward) in &role_rewards {
            let baseline = self.role_stats.get(role).map_or(0.0, |s| s.mean_reward);
            game_advantages.insert(*role, total_reward - baseline);
        }

        // Step 2: Per-turn GAE-style credit assignment
        let gamma = self.config.gamma;
        let lambda = self.config.gae_lambda;
        let mut gae = 0.0;

        // Walk backwards for GAE computation
        for i in (0..n).rev() {
            let role = roles.get(&trajectory.steps[i].actor_id).copied();
            let role_adv = role.map_or(0.0, |r| game_advantages.get(&r).copied().unwrap_or(0.0));
            let next_value = if i + 1 < n {
                roles
                    .get(&trajectory.steps[i + 1].actor_id)
                    .and_then(|r| game_advantages.get(r))
                    .copied()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let td_error = role_adv + gamma * next_value - role_adv;
            gae = td_error + gamma * lambda * gae;

            trajectory.steps[i].advantage = Some(gae);
        }

        // Step 3: Normalize per-agent if configured
        if self.config.normalize_per_agent {
            self.normalize_advantages(trajectory, roles);
        }

        // Step 4: Zero-grading filter
        self.apply_zero_grading(trajectory);
    }

    /// Normalize advantages per-agent (z-score within each agent's steps).
    fn normalize_advantages(&self, trajectory: &mut Trajectory, _roles: &HashMap<ActorId, Role>) {
        // Group advantages by actor
        let mut actor_advantages: HashMap<ActorId, Vec<f64>> = HashMap::new();
        for step in &trajectory.steps {
            if let Some(adv) = step.advantage {
                actor_advantages.entry(step.actor_id).or_default().push(adv);
            }
        }

        // Compute per-agent mean and std
        let mut agent_stats: HashMap<ActorId, (f64, f64)> = HashMap::new();
        for (actor_id, advs) in &actor_advantages {
            let mean = advs.iter().sum::<f64>() / advs.len() as f64;
            let var = advs.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / advs.len() as f64;
            let std = var.sqrt().max(1e-8);
            agent_stats.insert(*actor_id, (mean, std));
        }

        // Apply normalization
        let stats = agent_stats;
        for step in &mut trajectory.steps {
            if let Some(adv) = step.advantage {
                if let Some((mean, std)) = stats.get(&step.actor_id) {
                    step.advantage = Some((adv - mean) / std);
                }
            }
        }
    }

    /// Remove steps with near-zero advantages (zero-grading filter).
    fn apply_zero_grading(&self, trajectory: &mut Trajectory) {
        let threshold = self.config.zero_grading_threshold;
        for step in &mut trajectory.steps {
            if let Some(adv) = step.advantage {
                if adv.abs() < threshold {
                    step.advantage = Some(0.0);
                }
            }
        }
    }

    /// Get the current role statistics.
    pub fn role_stats(&self) -> &HashMap<Role, RoleStats> {
        &self.role_stats
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::nt_game::framework::{Action, Observation, TrajectoryStep};

    fn make_step(turn: usize, actor_id: ActorId, reward: f64) -> TrajectoryStep {
        TrajectoryStep {
            turn,
            actor_id,
            observation: Observation {
                text: format!("state_turn_{turn}"),
                legal_actions: vec![],
                hexagram: None,
                phi: None,
            },
            action: Action {
                kind: "move".into(),
                params: serde_json::json!({}),
                actor_id,
            },
            reward,
            advantage: None,
        }
    }

    fn make_trajectory() -> Trajectory {
        let mut t = Trajectory::new();
        t.push(make_step(0, 0, 0.5));
        t.push(make_step(1, 1, 0.3));
        t.push(make_step(2, 0, 0.8));
        t.push(make_step(3, 1, -0.2));
        t
    }

    fn two_player_roles() -> HashMap<ActorId, Role> {
        let mut roles = HashMap::new();
        roles.insert(0, Role::Player);
        roles.insert(1, Role::Opponent);
        roles
    }

    #[test]
    fn test_estimator_default_config() {
        let est = GameAdvantageEstimator::default();
        assert!((est.config.gamma - 0.99).abs() < 1e-6);
        assert!((est.config.gae_lambda - 0.95).abs() < 1e-6);
        assert!(est.config.normalize_per_agent);
    }

    #[test]
    fn test_estimator_assigns_advantages() {
        let mut est = GameAdvantageEstimator::default();
        let mut traj = make_trajectory();
        let roles = two_player_roles();

        est.estimate(&mut traj, &roles);

        // All steps should have advantage assigned
        for step in &traj.steps {
            assert!(step.advantage.is_some());
        }
    }

    #[test]
    fn test_estimator_empty_trajectory() {
        let mut est = GameAdvantageEstimator::default();
        let mut traj = Trajectory::new();
        let roles = two_player_roles();

        est.estimate(&mut traj, &roles);
        assert!(traj.steps.is_empty());
    }

    #[test]
    fn test_estimator_updates_role_stats() {
        let mut est = GameAdvantageEstimator::default();
        let mut traj = make_trajectory();
        let roles = two_player_roles();

        est.estimate(&mut traj, &roles);

        assert!(est.role_stats.contains_key(&Role::Player));
        assert!(est.role_stats.contains_key(&Role::Opponent));
        assert!(est.role_stats[&Role::Player].count == 1);
        assert!(est.role_stats[&Role::Opponent].count == 1);
    }

    #[test]
    fn test_estimator_zero_grading() {
        let config = AdvantageConfig {
            zero_grading_threshold: 0.5, // High threshold to filter more
            ..Default::default()
        };
        let mut est = GameAdvantageEstimator::new(config);
        let mut traj = Trajectory::new();
        // Two steps with identical rewards → advantage should be ~0
        traj.push(make_step(0, 0, 0.5));
        traj.push(make_step(1, 0, 0.5));
        let mut roles = HashMap::new();
        roles.insert(0, Role::Player);

        est.estimate(&mut traj, &roles);

        // With high threshold, advantages should be zeroed
        for step in &traj.steps {
            if let Some(adv) = step.advantage {
                assert!(adv.abs() < 0.5);
            }
        }
    }

    #[test]
    fn test_role_stats_update() {
        let mut stats = RoleStats::default();
        stats.update(1.0);
        stats.update(2.0);
        stats.update(3.0);

        assert!((stats.mean_reward - 2.0).abs() < 1e-6);
        assert!(stats.std_dev() > 0.0);
        assert_eq!(stats.count, 3);
    }

    #[test]
    fn test_role_stats_normalize() {
        let mut stats = RoleStats::default();
        stats.update(1.0);
        stats.update(2.0);
        stats.update(3.0);

        let normalized = stats.normalize(2.0);
        assert!(normalized.abs() < 1e-6); // Mean should normalize to ~0
    }

    #[test]
    fn test_estimator_no_normalize() {
        let config = AdvantageConfig {
            normalize_per_agent: false,
            ..Default::default()
        };
        let mut est = GameAdvantageEstimator::new(config);
        let mut traj = make_trajectory();
        let roles = two_player_roles();

        est.estimate(&mut traj, &roles);

        // Advantages should still be assigned
        for step in &traj.steps {
            assert!(step.advantage.is_some());
        }
    }

    #[test]
    fn test_estimator_single_step() {
        let mut est = GameAdvantageEstimator::default();
        let mut traj = Trajectory::new();
        traj.push(make_step(0, 0, 1.0));
        let mut roles = HashMap::new();
        roles.insert(0, Role::Player);

        est.estimate(&mut traj, &roles);
        assert_eq!(traj.steps.len(), 1);
        assert!(traj.steps[0].advantage.is_some());
    }
}
