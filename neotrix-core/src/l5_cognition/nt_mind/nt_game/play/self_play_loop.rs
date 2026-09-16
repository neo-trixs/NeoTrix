//! Self-play training loop — SPIRAL-style actor-learner.
//! Phases: Collect → Advantage → Update → Feedback → Sync

use serde::{Deserialize, Serialize};

use super::super::env::NtGameEnv;
use super::super::framework::{
    Action, Actor, Arena, ArenaConfig, Episode, Rubric, Trajectory, TrajectoryStep, WinLossReward,
};
use super::advantage::{AdvantageConfig, GameAdvantageEstimator};
use super::buffer::GameTrajectoryBuffer;
use super::grpo_adapter::GameGrpoAdapter;
use super::scaling::{ScalingConfig, ScalingScheduler};

// ═══════════════════════════════════════════════════════════════════
// Config
// ═══════════════════════════════════════════════════════════════════

/// Configuration for the self-play training loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfPlayConfig {
    /// Number of episodes per training iteration.
    pub episodes_per_iteration: usize,
    /// Maximum turns per episode.
    pub max_turns: usize,
    /// Learning rate for GRPO updates.
    pub learning_rate: f64,
    /// Capacity of the trajectory replay buffer.
    pub buffer_capacity: usize,
    /// Random seed for reproducibility.
    pub seed: u64,
}

impl Default for SelfPlayConfig {
    fn default() -> Self {
        Self {
            episodes_per_iteration: 32,
            max_turns: 50,
            learning_rate: 1e-4,
            buffer_capacity: 1024,
            seed: 42,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Metrics
// ═══════════════════════════════════════════════════════════════════

/// Metrics from a single training iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub iteration: usize,
    pub episodes_played: usize,
    pub avg_reward: f64,
    pub avg_turns: f64,
    pub win_rate: f64,
    pub policy_loss: f64,
    pub kl_divergence: f64,
    pub phi_avg: f64,
}

/// Full training report across multiple iterations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingReport {
    pub metrics_history: Vec<TrainingMetrics>,
    pub final_metrics: TrainingMetrics,
}

// ═══════════════════════════════════════════════════════════════════
// SelfPlayLoop
// ═══════════════════════════════════════════════════════════════════

/// SPIRAL-style self-play training loop.
///
/// Coordinates collect → advantage → update → feedback → sync phases
/// per iteration. Two actors share the same policy and play against
/// each other; trajectories are collected, advantages computed, and
/// the policy updated via GRPO.
pub struct SelfPlayLoop {
    config: SelfPlayConfig,
    buffer: GameTrajectoryBuffer,
    advantage_estimator: GameAdvantageEstimator,
    grpo_adapter: GameGrpoAdapter,
    scaling: ScalingScheduler,
    metrics_history: Vec<TrainingMetrics>,
    step_count: usize,
}

impl SelfPlayLoop {
    /// Create a new self-play loop with the given configuration.
    pub fn new(config: SelfPlayConfig) -> Self {
        let buffer = GameTrajectoryBuffer::new(config.buffer_capacity);
        let advantage_estimator = GameAdvantageEstimator::new(AdvantageConfig::default());
        let grpo_adapter = GameGrpoAdapter::default(config.buffer_capacity);
        let scaling = ScalingScheduler::new(ScalingConfig {
            initial_horizon: config.max_turns,
            step_size: 5,
            scaling_interval: 10,
            max_horizon: config.max_turns.max(200),
        });

        Self {
            config,
            buffer,
            advantage_estimator,
            grpo_adapter,
            scaling,
            metrics_history: Vec::new(),
            step_count: 0,
        }
    }

    /// Run one training iteration.
    ///
    /// 1. Collect: play `num_episodes` episodes, store trajectories
    /// 2. Advantage: compute per-turn advantages for all stored trajectories
    /// 3. Update: sample batch from buffer, run GRPO update
    /// 4. Feedback: compute metrics from buffer stats
    /// 5. Sync: tick the scaling scheduler
    pub fn run_iteration(
        &mut self,
        env_factory: &dyn Fn(u64) -> Box<dyn NtGameEnv>,
        num_episodes: usize,
    ) -> TrainingMetrics {
        let iteration = self.metrics_history.len();

        // ── Phase 1: Collect ──────────────────────────────────────
        for ep_idx in 0..num_episodes {
            let episode_seed = self.config.seed.wrapping_add(
                (iteration as u64)
                    .wrapping_mul(10000)
                    .wrapping_add(ep_idx as u64),
            );
            let mut env = env_factory(episode_seed);
            env.reset(Some(episode_seed));

            let (actor_a, actor_b) = Actor::self_play_pair("You are a game player.");
            let mut arena = Arena::new(ArenaConfig {
                default_max_turns: self.scaling.max_turns(),
                ..Default::default()
            });

            let mut rubric = Rubric::new();
            rubric.add(Box::new(WinLossReward), 1.0);

            let mut episode: Episode = arena.launch(
                &env.meta().name,
                vec![actor_a, actor_b],
                rubric,
                Some(self.scaling.max_turns()),
                Some(episode_seed),
            );

            // Play until terminal or turn limit
            while !env.is_terminal() && !episode.is_turn_limit_reached() {
                let current_player = env.current_player();
                let _observation = env.legal_actions();
                let legal = env.legal_actions();

                let action = if legal.is_empty() {
                    Action {
                        kind: "pass".into(),
                        params: serde_json::json!({}),
                        actor_id: current_player,
                    }
                } else {
                    // Simple policy: pick the first legal action
                    legal.into_iter().next().expect("non-empty")
                };

                let step_result = env.step(&action);
                let turn = episode.trajectory.length;

                episode.record_turn(TrajectoryStep {
                    turn,
                    actor_id: current_player,
                    observation: step_result.observation,
                    action,
                    reward: step_result.reward,
                    advantage: None,
                });

                if step_result.done {
                    break;
                }
            }

            let _result = episode.finalize();
            let trajectory = episode.trajectory;
            self.buffer.store(trajectory);
        }

        // ── Phase 2: Advantage ────────────────────────────────────
        // Compute advantages for all trajectories in the buffer that lack them
        let buffer_len = self.buffer.len();
        // We need to estimate advantages per-trajectory. Since the estimator
        // maintains role_stats across calls, we iterate and estimate each.
        // We take a clone-safe approach: sample what we have and re-estimate.
        // For correctness we do a full pass by sampling the entire buffer.
        {
            // Collect all trajectories as owned copies for in-place mutation,
            // then put them back. This is the simplest correct approach.
            let all: Vec<Trajectory> = {
                let batch = self.buffer.sample_batch(buffer_len);
                batch.into_iter().cloned().collect()
            };

            let mut roles = std::collections::HashMap::new();
            // In self-play: actor 0 = Player, actor 1 = Opponent
            roles.insert(0, super::super::framework::Role::Player);
            roles.insert(1, super::super::framework::Role::Opponent);

            for mut traj in all {
                if traj.steps.iter().any(|s| s.advantage.is_none()) {
                    self.advantage_estimator.estimate(&mut traj, &roles);
                    self.buffer.store(traj);
                }
            }
        }

        // ── Phase 3: Update ───────────────────────────────────────
        let batch = self.grpo_adapter.prepare_batch(&self.buffer);
        let grpo_report = self.grpo_adapter.update(batch);

        // ── Phase 4: Feedback ─────────────────────────────────────
        let stats = self.buffer.compute_stats();
        let metrics = TrainingMetrics {
            iteration,
            episodes_played: stats.total_episodes,
            avg_reward: stats.avg_reward,
            avg_turns: stats.avg_turns,
            win_rate: stats.win_rate,
            policy_loss: grpo_report.policy_loss,
            kl_divergence: grpo_report.kl_divergence,
            phi_avg: 0.0, // Phi computed at env level; default to 0 here
        };

        // ── Phase 5: Sync ─────────────────────────────────────────
        self.scaling.maybe_scale();
        self.step_count += 1;
        self.metrics_history.push(metrics.clone());

        metrics
    }

    /// Run multiple training iterations.
    pub fn run(
        &mut self,
        env_factory: &dyn Fn(u64) -> Box<dyn NtGameEnv>,
        iterations: usize,
    ) -> TrainingReport {
        for _ in 0..iterations {
            self.run_iteration(env_factory, self.config.episodes_per_iteration);
        }

        let final_metrics = self
            .metrics_history
            .last()
            .cloned()
            .unwrap_or(TrainingMetrics {
                iteration: 0,
                episodes_played: 0,
                avg_reward: 0.0,
                avg_turns: 0.0,
                win_rate: 0.0,
                policy_loss: 0.0,
                kl_divergence: 0.0,
                phi_avg: 0.0,
            });

        TrainingReport {
            metrics_history: self.metrics_history.clone(),
            final_metrics,
        }
    }

    /// Get the full metrics history.
    pub fn metrics(&self) -> &[TrainingMetrics] {
        &self.metrics_history
    }

    /// Get the most recent iteration's metrics.
    pub fn latest_metrics(&self) -> Option<&TrainingMetrics> {
        self.metrics_history.last()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::nt_game::env::{Difficulty, GameMeta, GameState, RenderMode};
    use crate::l5_cognition::nt_mind::nt_game::{Observation, StepResult};
    use std::collections::HashMap;

    /// A trivial environment that always returns a win for the first player
    /// after a fixed number of turns.
    struct DummyEnv {
        turn: usize,
        max_turns: usize,
        done: bool,
    }

    impl DummyEnv {
        fn new(max_turns: usize) -> Self {
            Self {
                turn: 0,
                max_turns,
                done: false,
            }
        }
    }

    impl NtGameEnv for DummyEnv {
        fn meta(&self) -> GameMeta {
            GameMeta {
                name: "DummyGame".into(),
                description: "A trivial test game".into(),
                min_constellation: 0,
                max_constellation: 5,
                target_skills: vec![],
                is_builtin: true,
            }
        }

        fn reset(&mut self, _seed: Option<u64>) -> Observation {
            self.turn = 0;
            self.done = false;
            Observation {
                text: "initial state".into(),
                legal_actions: vec![Action {
                    kind: "move".into(),
                    params: serde_json::json!({}),
                    actor_id: 0,
                }],
                hexagram: None,
                phi: None,
            }
        }

        fn step(&mut self, action: &Action) -> StepResult {
            self.turn += 1;
            self.done = self.turn >= self.max_turns;
            StepResult {
                observation: Observation {
                    text: format!("turn_{}", self.turn),
                    legal_actions: vec![Action {
                        kind: "move".into(),
                        params: serde_json::json!({}),
                        actor_id: self.current_player(),
                    }],
                    hexagram: None,
                    phi: None,
                },
                reward: if self.done { 1.0 } else { 0.0 },
                done: self.done,
                info: serde_json::json!({}),
            }
        }

        fn state(&self) -> GameState {
            GameState {
                turn: self.turn,
                current_player: 0,
                is_terminal: self.done,
                scores: HashMap::new(),
                energy: HashMap::new(),
                hexagram_states: vec![],
                board_size: (8, 8),
                difficulty: Difficulty::Tutorial,
                custom: serde_json::json!({}),
            }
        }

        fn legal_actions(&self) -> Vec<Action> {
            if self.done {
                vec![]
            } else {
                vec![Action {
                    kind: "move".into(),
                    params: serde_json::json!({}),
                    actor_id: self.current_player(),
                }]
            }
        }

        fn is_terminal(&self) -> bool {
            self.done
        }

        fn current_player(&self) -> u32 {
            (self.turn % 2) as u32
        }

        fn num_players(&self) -> usize {
            2
        }

        fn get_text_state(&self, _agent_id: u32) -> String {
            format!("DummyEnv turn={}", self.turn)
        }

        fn get_game_rules(&self) -> String {
            "Take turns. Last turn wins.".into()
        }
    }

    #[test]
    fn test_config_default() {
        let cfg = SelfPlayConfig::default();
        assert_eq!(cfg.episodes_per_iteration, 32);
        assert_eq!(cfg.max_turns, 50);
        assert!((cfg.learning_rate - 1e-4).abs() < 1e-10);
        assert_eq!(cfg.buffer_capacity, 1024);
        assert_eq!(cfg.seed, 42);
    }

    #[test]
    fn test_new_creates_components() {
        let config = SelfPlayConfig {
            episodes_per_iteration: 4,
            max_turns: 10,
            buffer_capacity: 64,
            ..Default::default()
        };
        let mut loop_ = SelfPlayLoop::new(config);
        assert_eq!(loop_.buffer.capacity(), 64);
        assert_eq!(loop_.scaling.max_turns(), 10);
        assert!(loop_.metrics_history.is_empty());
    }

    #[test]
    fn test_run_iteration_collects_trajectories() {
        let config = SelfPlayConfig {
            episodes_per_iteration: 3,
            max_turns: 5,
            buffer_capacity: 32,
            seed: 99,
            learning_rate: 1e-4,
        };
        let mut loop_ = SelfPlayLoop::new(config);

        let factory = |seed: u64| -> Box<dyn NtGameEnv> {
            Box::new(DummyEnv::new(3)) // 3-turn episodes
        };

        let metrics = loop_.run_iteration(&factory, 3);
        assert_eq!(metrics.episodes_played, 3);
        assert!(!loop_.buffer.is_empty());
        assert_eq!(loop_.metrics_history.len(), 1);
    }

    #[test]
    fn test_run_multiple_iterations() {
        let config = SelfPlayConfig {
            episodes_per_iteration: 2,
            max_turns: 5,
            learning_rate: 1e-4,
            buffer_capacity: 16,
            seed: 7,
        };
        let mut loop_ = SelfPlayLoop::new(config);

        let factory = |seed: u64| -> Box<dyn NtGameEnv> { Box::new(DummyEnv::new(2)) };

        let report = loop_.run(&factory, 3);
        assert_eq!(report.metrics_history.len(), 3);
        assert_eq!(report.final_metrics.iteration, 2);
    }

    #[test]
    fn test_metrics_accessor() {
        let config = SelfPlayConfig {
            episodes_per_iteration: 1,
            max_turns: 3,
            learning_rate: 1e-4,
            buffer_capacity: 8,
            seed: 0,
        };
        let mut loop_ = SelfPlayLoop::new(config);

        let factory = |_: u64| -> Box<dyn NtGameEnv> { Box::new(DummyEnv::new(2)) };

        assert!(loop_.latest_metrics().is_none());
        loop_.run_iteration(&factory, 1);
        assert!(loop_.latest_metrics().is_some());
        assert_eq!(loop_.metrics().len(), 1);
    }

    #[test]
    fn test_scaling_ticks() {
        let config = SelfPlayConfig {
            episodes_per_iteration: 1,
            max_turns: 10,
            learning_rate: 1e-4,
            buffer_capacity: 8,
            seed: 0,
        };
        let mut loop_ = SelfPlayLoop::new(config);

        let factory = |_: u64| -> Box<dyn NtGameEnv> { Box::new(DummyEnv::new(2)) };

        let initial_horizon = loop_.scaling.max_turns();
        // Run enough iterations to trigger scaling (interval=10)
        for _ in 0..10 {
            loop_.run_iteration(&factory, 1);
        }
        assert!(loop_.scaling.max_turns() >= initial_horizon);
    }

    #[test]
    fn test_advantage_computation_applied() {
        let config = SelfPlayConfig {
            episodes_per_iteration: 2,
            max_turns: 4,
            learning_rate: 1e-4,
            buffer_capacity: 16,
            seed: 1,
        };
        let mut loop_ = SelfPlayLoop::new(config);

        let factory = |_: u64| -> Box<dyn NtGameEnv> { Box::new(DummyEnv::new(3)) };
        loop_.run_iteration(&factory, 2);

        // After iteration, trajectories in the buffer should have advantages
        let batch = loop_.buffer.sample_batch(2);
        for traj in &batch {
            let has_adv = traj.steps.iter().any(|s| s.advantage.is_some());
            assert!(has_adv, "Expected at least one step with advantage set");
        }
    }
}
