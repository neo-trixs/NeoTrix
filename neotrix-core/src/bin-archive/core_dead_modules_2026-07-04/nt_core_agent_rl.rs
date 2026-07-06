//! B123 — Agent RL training pipeline
//!
//! Provides trajectory collection, experience replay, and action-level reward
//! aggregation for E8 reasoning loop training. Bridges step-level PRM scores
//! to trajectory-level GRPO updates.
//!
//! Architecture:
//!   TrajectoryBuffer: ring buffer of (state, action, reward, next_state)
//!   RewardAggregator: step-level scores → action-level advantages
//!   Sampler: prioritised replay for GRPO mini-batches

use crate::core::nt_core_hex::ReasoningHexagram;

/// A single transition in an agent trajectory.
#[derive(Debug, Clone)]
pub struct Transition {
    /// E8 state before action
    pub state: ReasoningHexagram,
    /// Action taken (E8 mode selected)
    pub action: ReasoningHexagram,
    /// Step-level reward (from PRM or environment)
    pub reward: f64,
    /// E8 state after action
    pub next_state: ReasoningHexagram,
    /// Whether this was the terminal step
    pub done: bool,
    /// Task type identifier
    pub task: String,
    /// Timestep within episode
    pub timestep: u32,
}

/// A complete episode trajectory.
#[derive(Debug, Clone)]
pub struct Episode {
    /// All transitions in this episode
    pub transitions: Vec<Transition>,
    /// Final outcome reward (e.g. task success)
    pub final_reward: f64,
    /// Episode ID
    pub id: u64,
}

/// Ring-buffer trajectory store with prioritised replay.
#[derive(Debug, Clone)]
pub struct TrajectoryBuffer {
    /// Stored transitions (ring buffer)
    buffer: Vec<Transition>,
    /// Maximum capacity
    capacity: usize,
    /// Write cursor
    cursor: usize,
    /// Current fill level
    size: usize,
    /// Episode counter
    episode_count: u64,
    /// Stored episodes for GRPO group sampling
    episodes: Vec<Episode>,
}

impl Default for TrajectoryBuffer {
    fn default() -> Self {
        Self::new(5000)
    }
}

impl TrajectoryBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            cursor: 0,
            size: 0,
            episode_count: 0,
            episodes: Vec::new(),
        }
    }

    /// Push a single transition into the ring buffer.
    pub fn push(&mut self, t: Transition) {
        if self.size < self.capacity {
            self.buffer.push(t);
            self.size += 1;
        } else {
            self.buffer[self.cursor] = t;
        }
        self.cursor = (self.cursor + 1) % self.capacity;
    }

    /// Start a new episode: push collected transitions as an Episode.
    pub fn finish_episode(&mut self, transitions: Vec<Transition>, final_reward: f64) {
        if transitions.is_empty() {
            return;
        }
        let episode = Episode {
            transitions,
            final_reward,
            id: self.episode_count,
        };
        self.episode_count += 1;
        // Store episode (keep last 100 by default)
        if self.episodes.len() >= 100 {
            self.episodes.remove(0);
        }
        self.episodes.push(episode);
    }

    /// Sample a batch of transitions uniformly.
    pub fn sample(&self, batch_size: usize) -> Vec<&Transition> {
        if self.size == 0 {
            return vec![];
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bs = batch_size.min(self.size);
        (0..bs).map(|_| &self.buffer[rng.gen_range(0..self.size)]).collect()
    }

    /// Build GRPO trajectory groups from recent episodes.
    /// Each trajectory is (mode_used, effective_reward) for a single episode.
    pub fn build_grpo_batch(&self, group_size: usize) -> Vec<(ReasoningHexagram, f64)> {
        let episodes: Vec<&Episode> = self.episodes.iter().rev().take(group_size).collect();
        if episodes.len() < 2 {
            return vec![];
        }
        let mut result = Vec::with_capacity(episodes.len());
        for ep in &episodes {
            // Action-level reward: blend final reward with average step reward
            let avg_step = if ep.transitions.is_empty() {
                0.0
            } else {
                let sum: f64 = ep.transitions.iter().map(|t| t.reward).sum();
                sum / ep.transitions.len() as f64
            };
            let effective = 0.7 * ep.final_reward + 0.3 * avg_step;
            // Use the first action as the representative mode
            if let Some(first) = ep.transitions.first() {
                result.push((first.action, effective));
            }
        }
        result
    }

    /// Number of stored transitions.
    pub fn len(&self) -> usize {
        self.size
    }

    /// Number of completed episodes.
    pub fn episode_count(&self) -> u64 {
        self.episode_count
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// Aggregates step-level PRM scores into action-level rewards.
///
/// Applies LATA (Length-Adaptive Trajectory Advantage) normalisation
/// and temporal discounting to produce per-action reward signals.
#[derive(Debug, Clone)]
pub struct RewardAggregator {
    /// Temporal discount factor per step
    pub discount: f64,
    /// LATA normalisation: divide by sqrt(L) where L = trajectory length
    pub use_lata: bool,
}

impl Default for RewardAggregator {
    fn default() -> Self {
        Self { discount: 0.95, use_lata: true }
    }
}

impl RewardAggregator {
    pub fn new(discount: f64, use_lata: bool) -> Self {
        Self { discount, use_lata }
    }

    /// Aggregate step-level PRM scores into a single action-level reward.
    ///
    /// For each step i: discounted_reward_i = sum_{j=i}^{L-1} γ^{j-i} · score_j
    /// Then LATA: effective_reward_i = discounted_reward_i / sqrt(L)
    pub fn aggregate(&self, step_scores: &[f64]) -> Vec<f64> {
        let l = step_scores.len();
        if l == 0 {
            return vec![];
        }
        let mut discounted = vec![0.0; l];
        let mut cumulative = 0.0;
        for i in (0..l).rev() {
            cumulative = step_scores[i] + self.discount * cumulative;
            discounted[i] = cumulative;
        }
        if self.use_lata && l > 0 {
            let lata = (l as f64).sqrt();
            for r in discounted.iter_mut() {
                *r /= lata;
            }
        }
        discounted
    }

    /// Compute action-level advantages: for each step, the advantage is
    /// the discounted return minus a baseline (e.g. mean return of the batch).
    pub fn compute_advantages(rewards: &[f64]) -> Vec<f64> {
        let n = rewards.len();
        if n == 0 {
            return vec![];
        }
        let mean: f64 = rewards.iter().sum::<f64>() / n as f64;
        let var: f64 = if n > 1 {
            rewards.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1) as f64
        } else {
            1.0
        };
        let std = var.sqrt().max(1e-8);
        rewards.iter().map(|r| (r - mean) / std).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TrajectoryBuffer Tests ─────────────────────────────────

    #[test]
    fn test_trajectory_buffer_push_and_sample() {
        let mut buf = TrajectoryBuffer::new(100);
        for i in 0..10 {
            buf.push(Transition {
                state: ReasoningHexagram(i),
                action: ReasoningHexagram(i + 1),
                reward: 0.5,
                next_state: ReasoningHexagram(i + 1),
                done: i == 9,
                task: "test".into(),
                timestep: i as u32,
            });
        }
        assert_eq!(buf.len(), 10);
        let batch = buf.sample(5);
        assert_eq!(batch.len(), 5);
    }

    #[test]
    fn test_trajectory_buffer_ring_overflow() {
        let mut buf = TrajectoryBuffer::new(10);
        for i in 0..20 {
            buf.push(Transition {
                state: ReasoningHexagram(0),
                action: ReasoningHexagram(i as u8 % 64),
                reward: 1.0,
                next_state: ReasoningHexagram(1),
                done: false,
                task: "test".into(),
                timestep: i,
            });
        }
        assert_eq!(buf.len(), 10, "ring buffer should cap at capacity");
    }

    #[test]
    fn test_trajectory_buffer_episode() {
        let mut buf = TrajectoryBuffer::new(100);
        let transitions = vec![
            Transition { state: ReasoningHexagram(0), action: ReasoningHexagram(1), reward: 0.3, next_state: ReasoningHexagram(1), done: false, task: "test".into(), timestep: 0 },
            Transition { state: ReasoningHexagram(1), action: ReasoningHexagram(2), reward: 0.6, next_state: ReasoningHexagram(2), done: true, task: "test".into(), timestep: 1 },
        ];
        buf.finish_episode(transitions, 0.9);
        assert_eq!(buf.episode_count(), 1);
    }

    #[test]
    fn test_trajectory_buffer_empty_episode_ignored() {
        let mut buf = TrajectoryBuffer::new(100);
        buf.finish_episode(vec![], 0.0);
        assert_eq!(buf.episode_count(), 0);
    }

    #[test]
    fn test_build_grpo_batch_returns_trajectories() {
        let mut buf = TrajectoryBuffer::new(100);
        for ep in 0..4 {
            let t = Transition {
                state: ReasoningHexagram(0),
                action: ReasoningHexagram(ep),
                reward: 0.3 + ep as f64 * 0.1,
                next_state: ReasoningHexagram(1),
                done: true,
                task: "test".into(),
                timestep: 0,
            };
            buf.finish_episode(vec![t], 0.8 + ep as f64 * 0.05);
        }
        let batch = buf.build_grpo_batch(4);
        assert_eq!(batch.len(), 4, "should build 4 trajectories for GRPO");
        // Batch is built reverse-chronologically (latest first), so batch[0] is the latest
        assert!(batch[0].1 >= batch[3].1, "latest episode should have highest reward");
    }

    #[test]
    fn test_build_grpo_batch_insufficient_episodes() {
        let buf = TrajectoryBuffer::new(100);
        let batch = buf.build_grpo_batch(4);
        assert!(batch.is_empty(), "no episodes → empty batch");
    }

    // ── RewardAggregator Tests ─────────────────────────────────

    #[test]
    fn test_reward_aggregator_discount() {
        let agg = RewardAggregator::new(0.5, false);
        let scores = vec![1.0, 0.5, 0.0];
        let rewards = agg.aggregate(&scores);
        // step 2: 0.0
        // step 1: 0.5 + 0.5 * 0.0 = 0.5
        // step 0: 1.0 + 0.5 * 0.5 = 1.25
        assert!((rewards[0] - 1.25).abs() < 0.01, "step 0 should be 1.25, got {}", rewards[0]);
        assert!((rewards[1] - 0.5).abs() < 0.01, "step 1 should be 0.5, got {}", rewards[1]);
        assert!((rewards[2] - 0.0).abs() < 0.01, "step 2 should be 0.0, got {}", rewards[2]);
    }

    #[test]
    fn test_reward_aggregator_lata() {
        let agg = RewardAggregator::new(0.0, true);
        let scores = vec![1.0, 1.0, 1.0, 1.0]; // L = 4, sqrt = 2.0
        let rewards = agg.aggregate(&scores);
        for r in &rewards {
            assert!((*r - 0.5).abs() < 0.01, "LATA should divide by sqrt(4)=2, got {r}");
        }
    }

    #[test]
    fn test_reward_aggregator_empty() {
        let agg = RewardAggregator::default();
        let rewards = agg.aggregate(&[]);
        assert!(rewards.is_empty());
    }

    #[test]
    fn test_compute_advantages_normalizes() {
        let rewards = vec![1.0, 0.5, 0.0];
        let adv = RewardAggregator::compute_advantages(&rewards);
        assert_eq!(adv.len(), 3);
        let mean: f64 = adv.iter().sum::<f64>() / adv.len() as f64;
        assert!(mean.abs() < 0.01, "advantages should be zero-mean, got {mean}");
    }

    #[test]
    fn test_compute_advantages_single() {
        let adv = RewardAggregator::compute_advantages(&[0.5]);
        assert_eq!(adv.len(), 1);
        assert!((adv[0] - 0.0).abs() < 0.01, "single reward → zero advantage");
    }

    #[test]
    fn test_compute_advantages_empty() {
        let adv = RewardAggregator::compute_advantages(&[]);
        assert!(adv.is_empty());
    }
}
