use std::collections::HashMap;
use std::f64;

/// UCB1 multi-armed bandit for selecting retrieval strategies
pub struct BanditScorer {
    /// Arm name -> (total_reward, play_count)
    arms: HashMap<String, (f64, usize)>,
    total_plays: usize,
    exploration_constant: f64,
}

impl BanditScorer {
    pub fn new(exploration_constant: f64) -> Self {
        Self {
            arms: HashMap::new(),
            total_plays: 0,
            exploration_constant,
        }
    }

    pub fn register_arm(&mut self, name: &str) {
        self.arms.entry(name.to_string()).or_insert((0.0, 0));
    }

    pub fn score(&self, name: &str) -> f64 {
        match self.arms.get(name) {
            Some(&(reward, count)) if count > 0 => {
                let exploit = reward / count as f64;
                let explore = self.exploration_constant * (self.total_plays as f64).ln().sqrt()
                    / count as f64;
                exploit + explore
            }
            _ => f64::MAX,
        }
    }

    pub fn best_arm(&self) -> Option<&str> {
        self.arms
            .keys()
            .max_by(|a, b| {
                self.score(a)
                    .partial_cmp(&self.score(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.as_str())
    }

    pub fn update(&mut self, name: &str, reward: f64) {
        if let Some((total, count)) = self.arms.get_mut(name) {
            *total += reward;
            *count += 1;
            self.total_plays += 1;
        }
    }

    pub fn arm_count(&self) -> usize {
        self.arms.len()
    }
    pub fn total_plays(&self) -> usize {
        self.total_plays
    }

    pub fn confidence_weighted_score(&self, name: &str) -> f64 {
        match self.arms.get(name) {
            Some(&(reward, count)) if count >= 3 => reward / count as f64,
            Some(&(reward, count)) if count > 0 => {
                let exploit = reward / count as f64;
                let bonus = self.exploration_constant / (count as f64).sqrt();
                exploit + bonus
            }
            _ => f64::MAX,
        }
    }
}

/// Reward model: logistic regression predictor for retrieval utility
pub struct RewardModel {
    weights: Vec<f64>,
    bias: f64,
}

impl RewardModel {
    pub fn new(dim: usize) -> Self {
        Self {
            weights: vec![0.1; dim],
            bias: 0.0,
        }
    }

    pub fn predict(&self, features: &[f64]) -> f64 {
        let dot: f64 = self.weights.iter().zip(features).map(|(w, f)| w * f).sum();
        1.0 / (1.0 + (-(dot + self.bias)).exp())
    }

    pub fn update(&mut self, features: &[f64], target: f64, lr: f64) {
        let pred = self.predict(features);
        let error = pred - target;
        for (w, f) in self.weights.iter_mut().zip(features) {
            *w -= lr * error * pred * (1.0 - pred) * f;
        }
        self.bias -= lr * error * pred * (1.0 - pred);
    }
}

/// Staleness detector: identifies outdated or contradictory rules
pub struct StalenessDetector {
    decay_rate: f64,
    age_threshold_secs: u64,
}

impl StalenessDetector {
    pub fn new(decay_rate: f64, age_threshold_secs: u64) -> Self {
        Self {
            decay_rate,
            age_threshold_secs,
        }
    }

    pub fn score(&self, created_at: u64, last_used_at: u64, now: u64) -> f64 {
        let age = now.saturating_sub(created_at);
        let inactivity = now.saturating_sub(last_used_at);
        let age_factor = (-self.decay_rate * age as f64).exp();
        let inactivity_factor = (-self.decay_rate * 0.5 * inactivity as f64).exp();
        age_factor * inactivity_factor
    }

    pub fn is_stale(&self, created_at: u64, last_used_at: u64, now: u64) -> bool {
        now.saturating_sub(created_at) > self.age_threshold_secs
            && now.saturating_sub(last_used_at) > self.age_threshold_secs / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arm_registration_and_scoring() {
        let mut scorer = BanditScorer::new(2.0);
        scorer.register_arm("alpha");
        scorer.register_arm("beta");
        assert_eq!(scorer.arm_count(), 2);
        assert_eq!(scorer.score("alpha"), f64::MAX);
        assert_eq!(scorer.score("beta"), f64::MAX);
    }

    #[test]
    fn test_best_arm_selection() {
        let mut scorer = BanditScorer::new(2.0);
        scorer.register_arm("a");
        scorer.register_arm("b");
        scorer.update("a", 10.0);
        scorer.update("a", 10.0);
        scorer.update("a", 10.0);
        scorer.update("b", 1.0);
        let best = scorer.best_arm();
        assert_eq!(best, Some("a"));
    }

    #[test]
    fn test_update_and_confidence_weighting() {
        let mut scorer = BanditScorer::new(2.0);
        scorer.register_arm("x");
        scorer.update("x", 5.0);
        let raw = scorer.score("x");
        let cw = scorer.confidence_weighted_score("x");
        assert!(raw < cw, "raw={}, cw={}", raw, cw);
        scorer.update("x", 5.0);
        scorer.update("x", 5.0);
        let cw_confident = scorer.confidence_weighted_score("x");
        assert!((cw_confident - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_reward_model() {
        let mut model = RewardModel::new(3);
        let features = vec![0.5, 1.2, -0.3];
        let pred_before = model.predict(&features);
        model.update(&features, 1.0, 0.1);
        let pred_after = model.predict(&features);
        assert!((pred_after - 1.0).abs() < (pred_before - 1.0).abs());
    }

    #[test]
    fn test_staleness_detection() {
        let detector = StalenessDetector::new(0.01, 100);
        let score_fresh = detector.score(1000, 1050, 1055);
        assert!(score_fresh > 0.5);
        assert!(!detector.is_stale(1000, 1050, 1055));
        assert!(detector.is_stale(100, 150, 1000));
    }
}
