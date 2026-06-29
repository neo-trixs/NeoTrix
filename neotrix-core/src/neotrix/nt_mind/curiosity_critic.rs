use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CriticStats {
    pub step: u64,
    pub critic_estimate: f64,
    pub cumulative_error: f64,
    pub epistemic_ratio: f64,
    pub aleatoric_ratio: f64,
    pub avg_reward: f64,
}

#[derive(Debug, Clone)]
pub struct CuriosityCritic {
    pub critic_estimate: f64,
    pub cumulative_error: f64,
    pub error_history: VecDeque<f64>,
    pub reward_history: VecDeque<f64>,
    pub learning_rate: f64,
    pub window_size: usize,
    pub step: u64,
}

impl CuriosityCritic {
    pub fn new(learning_rate: f64, window_size: usize) -> Self {
        Self {
            critic_estimate: 0.5,
            cumulative_error: 0.0,
            error_history: VecDeque::with_capacity(window_size),
            reward_history: VecDeque::with_capacity(window_size),
            learning_rate: learning_rate.clamp(0.01, 0.5),
            window_size: window_size.max(5),
            step: 0,
        }
    }

    pub fn compute_reward(&mut self, prediction_error: f64) -> f64 {
        self.step += 1;
        let error = prediction_error.clamp(0.0, 1.0);
        self.cumulative_error += error;
        self.error_history.push_back(error);
        if self.error_history.len() > self.window_size {
            self.error_history.pop_front();
        }

        let _old_estimate = self.critic_estimate;
        self.critic_estimate += self.learning_rate * (error - self.critic_estimate);

        let reward = if error > self.critic_estimate * 1.2 {
            (error - self.critic_estimate).clamp(0.0, 1.0)
        } else if error < self.critic_estimate * 0.8 {
            0.0
        } else {
            (error - self.critic_estimate).max(0.0) * 0.5
        };

        self.reward_history.push_back(reward);
        if self.reward_history.len() > self.window_size {
            self.reward_history.pop_front();
        }

        reward
    }

    pub fn epistemic_ratio(&self) -> f64 {
        if self.error_history.is_empty() {
            return 0.0;
        }
        let count = self
            .error_history
            .iter()
            .filter(|&&e| e > self.critic_estimate * 1.2)
            .count();
        count as f64 / self.error_history.len() as f64
    }

    pub fn aleatoric_ratio(&self) -> f64 {
        if self.error_history.is_empty() {
            return 0.0;
        }
        let count = self
            .error_history
            .iter()
            .filter(|&&e| e < self.critic_estimate * 0.8)
            .count();
        count as f64 / self.error_history.len() as f64
    }

    pub fn avg_reward(&self) -> f64 {
        if self.reward_history.is_empty() {
            return 0.0;
        }
        self.reward_history.iter().sum::<f64>() / self.reward_history.len() as f64
    }

    pub fn stats(&self) -> CriticStats {
        CriticStats {
            step: self.step,
            critic_estimate: self.critic_estimate,
            cumulative_error: self.cumulative_error,
            epistemic_ratio: self.epistemic_ratio(),
            aleatoric_ratio: self.aleatoric_ratio(),
            avg_reward: self.avg_reward(),
        }
    }
}

impl Default for CuriosityCritic {
    fn default() -> Self {
        Self::new(0.1, 20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_critic() {
        let critic = CuriosityCritic::new(0.1, 20);
        assert!((critic.critic_estimate - 0.5).abs() < 0.01);
        assert_eq!(critic.learning_rate, 0.1);
        assert_eq!(critic.window_size, 20);
        assert_eq!(critic.step, 0);
    }

    #[test]
    fn test_high_epistemic_reward() {
        let mut critic = CuriosityCritic::new(0.1, 20);
        let reward = critic.compute_reward(0.9);
        let _eps = critic.critic_estimate;
        assert!(
            reward > 0.0,
            "high error should give positive reward, got {}",
            reward
        );
        assert!(reward <= 1.0);
    }

    #[test]
    fn test_low_aleatoric_reward() {
        let mut critic = CuriosityCritic::new(0.1, 20);
        critic.compute_reward(0.5);
        let estimate = critic.critic_estimate;
        let reward = critic.compute_reward(estimate * 0.5);
        assert!(
            (reward - 0.0).abs() < 0.01,
            "error below estimate should give ~0 reward, got {}",
            reward
        );
    }

    #[test]
    fn test_critic_convergence() {
        let mut critic = CuriosityCritic::new(0.2, 20);
        for _ in 0..50 {
            critic.compute_reward(0.3);
        }
        assert!(
            (critic.critic_estimate - 0.3).abs() < 0.1,
            "critic should converge to error, got {}",
            critic.critic_estimate
        );
    }

    #[test]
    fn test_epistemic_ratio_all_high() {
        let mut critic = CuriosityCritic::new(0.1, 20);
        for _ in 0..20 {
            critic.compute_reward(0.9);
        }
        let ratio = critic.epistemic_ratio();
        assert!(
            ratio > 0.5,
            "all high errors should give high epistemic ratio, got {}",
            ratio
        );
    }

    #[test]
    fn test_stats() {
        let mut critic = CuriosityCritic::new(0.1, 10);
        critic.compute_reward(0.8);
        critic.compute_reward(0.2);
        let s = critic.stats();
        assert_eq!(s.step, 2);
        assert!(s.critic_estimate > 0.0);
        assert!(s.cumulative_error > 0.0);
    }

    #[test]
    fn test_default() {
        let critic: CuriosityCritic = Default::default();
        assert!((critic.critic_estimate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_reward_bounds() {
        let mut critic = CuriosityCritic::new(0.1, 10);
        for _ in 0..10 {
            let r = critic.compute_reward(0.7);
            assert!(r >= 0.0 && r <= 1.0, "reward must be in [0,1], got {}", r);
        }
    }
}
