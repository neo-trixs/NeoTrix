//! Game trajectory replay buffer with ring storage.
//!
//! Stores completed trajectories and supports random mini-batch sampling
//! with optional advantage pre-computation.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::super::framework::Trajectory;

// ═══════════════════════════════════════════════════════════════════
// Stats
// ═══════════════════════════════════════════════════════════════════

/// Aggregate statistics over the buffer contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BufferStats {
    pub total_episodes: usize,
    pub avg_reward: f64,
    pub avg_turns: f64,
    pub win_rate: f64,
    pub total_steps: usize,
}

// ═══════════════════════════════════════════════════════════════════
// Buffer
// ═══════════════════════════════════════════════════════════════════

/// Ring-buffer for completed game trajectories.
///
/// Supports fixed-capacity storage with FIFO eviction and random
/// mini-batch sampling for GRPO training.
pub struct GameTrajectoryBuffer {
    capacity: usize,
    buffer: VecDeque<Trajectory>,
}

impl GameTrajectoryBuffer {
    /// Create a new buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    /// Store a completed trajectory. Evicts oldest if at capacity.
    pub fn store(&mut self, trajectory: Trajectory) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(trajectory);
    }

    /// Sample a random mini-batch of trajectories (with replacement).
    pub fn sample_batch(&self, size: usize) -> Vec<&Trajectory> {
        use std::time::{SystemTime, UNIX_EPOCH};

        if self.buffer.is_empty() {
            return Vec::new();
        }
        let n = size.min(self.buffer.len());
        let mut indices: Vec<usize> = (0..self.buffer.len()).collect();

        // Simple Fisher-Yates partial shuffle using system time as seed
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mut state = seed;
        for i in 0..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = i + (state as usize) % (indices.len() - i);
            indices.swap(i, j);
        }

        indices[..n].iter().map(|&i| &self.buffer[i]).collect()
    }

    /// Sample a batch of trajectories that have advantage values pre-computed.
    ///
    /// Filters to trajectories where at least one step has `advantage != None`.
    pub fn sample_batch_with_advantages(&self, size: usize) -> Vec<&Trajectory> {
        let eligible: Vec<&Trajectory> = self
            .buffer
            .iter()
            .filter(|t| t.steps.iter().any(|s| s.advantage.is_some()))
            .collect();

        if eligible.is_empty() {
            return Vec::new();
        }

        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let mut state = seed;
        let n = size.min(eligible.len());
        let mut indices: Vec<usize> = (0..eligible.len()).collect();
        for i in 0..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = i + (state as usize) % (indices.len() - i);
            indices.swap(i, j);
        }

        indices[..n].iter().map(|&i| eligible[i]).collect()
    }

    /// Compute aggregate statistics over all buffered trajectories.
    pub fn compute_stats(&self) -> BufferStats {
        if self.buffer.is_empty() {
            return BufferStats::default();
        }

        let total = self.buffer.len();
        let total_steps: usize = self.buffer.iter().map(|t| t.length).sum();
        let avg_reward: f64 =
            self.buffer.iter().map(|t| t.total_reward).sum::<f64>() / total as f64;
        let avg_turns: f64 =
            self.buffer.iter().map(|t| t.length as f64).sum::<f64>() / total as f64;

        // Win rate: fraction of trajectories where last step reward > 0
        let wins = self
            .buffer
            .iter()
            .filter(|t| t.steps.last().map_or(false, |s| s.reward > 0.0))
            .count();
        let win_rate = wins as f64 / total as f64;

        BufferStats {
            total_episodes: total,
            avg_reward,
            avg_turns,
            win_rate,
            total_steps,
        }
    }

    /// Number of trajectories in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Remove all trajectories from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Maximum capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_game::framework::{Action, Observation, TrajectoryStep};

    fn make_trajectory(total_reward: f64, length: usize) -> Trajectory {
        let mut t = Trajectory::new();
        for i in 0..length {
            t.push(TrajectoryStep {
                turn: i,
                actor_id: 0,
                observation: Observation {
                    text: format!("state_{i}"),
                    legal_actions: vec![],
                    hexagram: None,
                    phi: None,
                },
                action: Action {
                    kind: "move".into(),
                    params: serde_json::json!({}),
                    actor_id: 0,
                },
                reward: total_reward / length as f64,
                advantage: None,
            });
        }
        t
    }

    fn make_winning_trajectory() -> Trajectory {
        let mut t = Trajectory::new();
        t.push(TrajectoryStep {
            turn: 0,
            actor_id: 0,
            observation: Observation {
                text: "state".into(),
                legal_actions: vec![],
                hexagram: None,
                phi: None,
            },
            action: Action {
                kind: "win".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            reward: 1.0,
            advantage: None,
        });
        t
    }

    fn make_losing_trajectory() -> Trajectory {
        let mut t = Trajectory::new();
        t.push(TrajectoryStep {
            turn: 0,
            actor_id: 0,
            observation: Observation {
                text: "state".into(),
                legal_actions: vec![],
                hexagram: None,
                phi: None,
            },
            action: Action {
                kind: "loss".into(),
                params: serde_json::json!({}),
                actor_id: 0,
            },
            reward: -1.0,
            advantage: None,
        });
        t
    }

    #[test]
    fn test_buffer_new() {
        let buf = GameTrajectoryBuffer::new(100);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 100);
    }

    #[test]
    fn test_buffer_store_and_len() {
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_trajectory(1.0, 5));
        buf.store(make_trajectory(2.0, 10));
        assert_eq!(buf.len(), 2);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_buffer_eviction() {
        let mut buf = GameTrajectoryBuffer::new(3);
        buf.store(make_trajectory(1.0, 5));
        buf.store(make_trajectory(2.0, 5));
        buf.store(make_trajectory(3.0, 5));
        assert_eq!(buf.len(), 3);
        // Adding a 4th should evict the oldest
        buf.store(make_trajectory(4.0, 5));
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_buffer_sample_batch() {
        let mut buf = GameTrajectoryBuffer::new(10);
        for i in 0..5 {
            buf.store(make_trajectory(i as f64, 5));
        }
        let batch = buf.sample_batch(3);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_buffer_sample_batch_empty() {
        let buf = GameTrajectoryBuffer::new(10);
        let batch = buf.sample_batch(5);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_buffer_sample_batch_oversized() {
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_trajectory(1.0, 5));
        buf.store(make_trajectory(2.0, 5));
        let batch = buf.sample_batch(100);
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_buffer_clear() {
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_trajectory(1.0, 5));
        buf.store(make_trajectory(2.0, 5));
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn test_buffer_compute_stats() {
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_winning_trajectory());
        buf.store(make_losing_trajectory());

        let stats = buf.compute_stats();
        assert_eq!(stats.total_episodes, 2);
        assert!((stats.win_rate - 0.5).abs() < 1e-6);
        assert!(stats.avg_reward > -1.0);
        assert!(stats.avg_reward < 1.0);
        assert_eq!(stats.total_steps, 2);
    }

    #[test]
    fn test_buffer_compute_stats_empty() {
        let buf = GameTrajectoryBuffer::new(10);
        let stats = buf.compute_stats();
        assert_eq!(stats.total_episodes, 0);
        assert_eq!(stats.win_rate, 0.0);
    }

    #[test]
    fn test_buffer_sample_batch_with_advantages_empty() {
        let buf = GameTrajectoryBuffer::new(10);
        let batch = buf.sample_batch_with_advantages(5);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_buffer_sample_batch_with_advantages_filtered() {
        let mut buf = GameTrajectoryBuffer::new(10);

        // Trajectory with advantages
        let mut with_adv = make_trajectory(1.0, 3);
        with_adv.steps[0].advantage = Some(0.5);
        buf.store(with_adv);

        // Trajectory without advantages
        buf.store(make_trajectory(2.0, 3));

        let batch = buf.sample_batch_with_advantages(10);
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_buffer_stats_avg_turns() {
        let mut buf = GameTrajectoryBuffer::new(10);
        buf.store(make_trajectory(1.0, 5));
        buf.store(make_trajectory(2.0, 10));

        let stats = buf.compute_stats();
        assert!((stats.avg_turns - 7.5).abs() < 1e-6);
    }
}
