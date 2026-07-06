//! Perception Evolution — adaptive sensing and attention
//!
//! The system learns to allocate perceptual resources based on:
//! - Reward history: what kinds of input led to high reward
//! - Novelty: new patterns that deserve attention
//! - Confidence: inputs where the system is uncertain

use std::collections::VecDeque;

/// Which perceptual modality to adjust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerceptualModality {
    Text,
    Code,
    Web,
    Memory,
    Social,
}

/// Recommended perceptual allocation
#[derive(Debug, Clone)]
pub struct PerceptionBudget {
    pub modality: PerceptualModality,
    pub allocation: f64,
    pub sampling_rate: f64,
    pub reasoning: String,
}

/// Perception Evolution engine
#[derive(Debug, Clone)]
pub struct PerceptionEvolution {
    pub modality_rewards: Vec<(PerceptualModality, f64)>,
    pub attention_history: VecDeque<(u64, Vec<PerceptionBudget>)>,
    pub exploration_rate: f64,
    pub max_history: usize,
}

impl PerceptionEvolution {
    pub fn new() -> Self {
        Self {
            modality_rewards: vec![
                (PerceptualModality::Text, 0.5),
                (PerceptualModality::Code, 0.5),
                (PerceptualModality::Web, 0.3),
                (PerceptualModality::Memory, 0.5),
                (PerceptualModality::Social, 0.5),
            ],
            attention_history: VecDeque::with_capacity(20),
            exploration_rate: 0.2,
            max_history: 20,
        }
    }

    /// Update modality reward based on feedback
    pub fn record_reward(&mut self, modality: PerceptualModality, reward: f64) {
        for (m, r) in &mut self.modality_rewards {
            if *m == modality {
                *r = *r * 0.9 + reward * 0.1;
                break;
            }
        }
    }

    /// Compute optimal attention budget for current state
    pub fn compute_budget(&self, _iteration: u64) -> Vec<PerceptionBudget> {
        let total_reward: f64 = self.modality_rewards.iter().map(|(_, r)| r).sum();
        let base = total_reward.max(0.1);
        self.modality_rewards.iter().map(|(m, r)| {
            let alloc = (r / base).min(1.0);
            let noisy_alloc = alloc * (1.0 - self.exploration_rate) + self.exploration_rate * 0.5;
            PerceptionBudget {
                modality: *m,
                allocation: noisy_alloc.clamp(0.1, 1.0),
                sampling_rate: (noisy_alloc * 0.8 + 0.2).clamp(0.2, 1.0),
                reasoning: format!("reward={:.3} → allocation={:.2}", r, noisy_alloc),
            }
        }).collect()
    }

    /// Decay exploration rate over time
    pub fn decay_exploration(&mut self) {
        self.exploration_rate = (self.exploration_rate * 0.995).max(0.05);
    }

    /// Record a budget allocation for history
    pub fn record_budget(&mut self, iteration: u64, budget: Vec<PerceptionBudget>) {
        if self.attention_history.len() >= self.max_history {
            self.attention_history.pop_front();
        }
        self.attention_history.push_back((iteration, budget));
    }

    /// The modality with the highest reward
    pub fn best_modality(&self) -> PerceptualModality {
        self.modality_rewards.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(m, _)| *m)
            .unwrap_or(PerceptualModality::Text)
    }
}

impl Default for PerceptionEvolution {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let pe = PerceptionEvolution::new();
        assert_eq!(pe.modality_rewards.len(), 5);
        assert!((pe.exploration_rate - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_record_reward() {
        let mut pe = PerceptionEvolution::new();
        pe.record_reward(PerceptualModality::Code, 0.9);
        let code_reward = pe.modality_rewards.iter()
            .find(|(m, _)| *m == PerceptualModality::Code).map(|(_, r)| *r).unwrap();
        assert!(code_reward > 0.5);
    }

    #[test]
    fn test_compute_budget_returns_all_modalities() {
        let pe = PerceptionEvolution::new();
        let budget = pe.compute_budget(0);
        assert_eq!(budget.len(), 5);
    }

    #[test]
    fn test_decay_exploration() {
        let mut pe = PerceptionEvolution::new();
        pe.decay_exploration();
        assert!(pe.exploration_rate < 0.2);
    }

    #[test]
    fn test_best_modality_after_reward() {
        let mut pe = PerceptionEvolution::new();
        pe.record_reward(PerceptualModality::Code, 1.0);
        assert_eq!(pe.best_modality(), PerceptualModality::Code);
    }

    #[test]
    fn test_record_budget() {
        let mut pe = PerceptionEvolution::new();
        let budget = pe.compute_budget(1);
        pe.record_budget(1, budget);
        assert_eq!(pe.attention_history.len(), 1);
    }
}
