#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct POMDPBeliefUpdater {
    pub num_states: usize,
}

impl POMDPBeliefUpdater {
    pub fn new(num_states: usize) -> Self {
        POMDPBeliefUpdater { num_states }
    }

    pub fn update_belief(
        &self,
        belief: &[f64],
        observation: usize,
        likelihood: &[Vec<f64>],
    ) -> Vec<f64> {
        if self.num_states == 0 {
            return vec![];
        }
        let mut posterior = vec![0.0; self.num_states];
        let mut evidence = 0.0;

        for s in 0..self.num_states {
            // 防 OOB: observation 越界视为无观测
            let l = likelihood.get(s).and_then(|row| row.get(observation)).copied().unwrap_or(0.0);
            posterior[s] = l * belief.get(s).copied().unwrap_or(0.0);
            evidence += posterior[s];
        }

        if evidence > 0.0 {
            for s in &mut posterior {
                *s /= evidence;
            }
        } else {
            let uniform = 1.0 / self.num_states as f64;
            posterior.fill(uniform);
        }

        posterior
    }

    pub fn predict_belief(&self, belief: &[f64], transition: &[Vec<f64>]) -> Vec<f64> {
        let mut next = vec![0.0; self.num_states];
        for i in 0..self.num_states {
            let b = belief.get(i).copied().unwrap_or(0.0);
            for j in 0..self.num_states {
                let t = transition.get(i).and_then(|row| row.get(j)).copied().unwrap_or(0.0);
                next[j] += b * t;
            }
        }
        next
    }

    pub fn belief_entropy(&self, belief: &[f64]) -> f64 {
        let mut entropy = 0.0;
        for &p in belief {
            if p > 0.0 {
                entropy -= p * (p + 1e-10).ln();
            }
        }
        entropy.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_belief_basic() {
        let updater = POMDPBeliefUpdater::new(2);
        let belief = vec![0.5, 0.5];
        let likelihood = vec![vec![0.9, 0.1], vec![0.2, 0.8]];
        let posterior = updater.update_belief(&belief, 0, &likelihood);
        let sum: f64 = posterior.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
        assert!(posterior[0] > posterior[1]);
    }

    #[test]
    fn test_update_belief_zero_evidence() {
        let updater = POMDPBeliefUpdater::new(3);
        let belief = vec![0.0, 0.0, 1.0];
        let likelihood = vec![
            vec![1.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        let posterior = updater.update_belief(&belief, 0, &likelihood);
        let sum: f64 = posterior.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
        let uniform = 1.0 / 3.0;
        for &p in &posterior {
            assert!((p - uniform).abs() < 1e-10);
        }
    }

    #[test]
    fn test_predict_belief() {
        let updater = POMDPBeliefUpdater::new(2);
        let belief = vec![1.0, 0.0];
        let transition = vec![vec![0.8, 0.2], vec![0.3, 0.7]];
        let next = updater.predict_belief(&belief, &transition);
        assert!((next[0] - 0.8).abs() < 1e-10);
        assert!((next[1] - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_belief_entropy_uniform() {
        let updater = POMDPBeliefUpdater::new(4);
        let belief = vec![0.25, 0.25, 0.25, 0.25];
        let entropy = updater.belief_entropy(&belief);
        let expected = (4.0f64).ln();
        assert!((entropy - expected).abs() < 1e-9);
    }

    #[test]
    fn test_belief_entropy_certain() {
        let updater = POMDPBeliefUpdater::new(2);
        let belief = vec![1.0, 0.0];
        let entropy = updater.belief_entropy(&belief);
        assert!((entropy).abs() < 1e-9);
    }

    #[test]
    fn test_update_belief_deterministic_likelihood() {
        let updater = POMDPBeliefUpdater::new(2);
        let belief = vec![0.5, 0.5];
        let likelihood = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let posterior = updater.update_belief(&belief, 0, &likelihood);
        assert!((posterior[0] - 1.0).abs() < 1e-10);
        assert!((posterior[1]).abs() < 1e-10);
    }
}
