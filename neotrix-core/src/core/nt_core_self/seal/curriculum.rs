#![forbid(unsafe_code)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumRecord {
    pub task_id: String,
    pub difficulty: f64,
    pub success: bool,
    pub reward: f64,
    pub iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub reason: String,
    pub suggested_action: String,
}

pub struct CalibratedCurriculumGenerator {
    pub tasks_completed: VecDeque<CurriculumRecord>,
    pub difficulty_level: f64,
    pub mastery_threshold: f64,
    pub window_size: usize,
}

pub struct IterationValidator {
    pub min_reward_threshold: f64,
    pub max_failures: u32,
    pub convergence_window: usize,
}

pub struct LearnabilityWindowAnalyzer {
    pub recent_performance: VecDeque<f64>,
    pub window_size: usize,
}

impl CalibratedCurriculumGenerator {
    pub fn new(mastery_threshold: f64, window_size: usize) -> Self {
        Self {
            tasks_completed: VecDeque::new(),
            difficulty_level: 0.5,
            mastery_threshold,
            window_size,
        }
    }

    pub fn generate_next_task(
        &self,
        available_tasks: &[(String, f64, Vec<String>)],
    ) -> Option<(String, f64, Vec<String>)> {
        let max_diff = self.difficulty_level + 0.2;
        let mut best: Option<&(String, f64, Vec<String>)> = None;
        let mut best_dist = f64::MAX;
        for task in available_tasks {
            if task.1 <= max_diff {
                let dist = (task.1 - self.difficulty_level).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best = Some(task);
                }
            }
        }
        best.cloned()
    }

    pub fn record_outcome(&mut self, record: CurriculumRecord) {
        self.tasks_completed.push_back(record);
        while self.tasks_completed.len() > self.window_size {
            self.tasks_completed.pop_front();
        }
    }

    pub fn adjust_difficulty(&mut self) -> f64 {
        if self.tasks_completed.is_empty() {
            return self.difficulty_level;
        }
        let success_count = self
            .tasks_completed
            .iter()
            .filter(|r| r.success)
            .count();
        let rate = success_count as f64 / self.tasks_completed.len() as f64;
        if rate >= self.mastery_threshold {
            self.difficulty_level = (self.difficulty_level + 0.1).min(1.0);
        } else if rate < 0.3 {
            self.difficulty_level = (self.difficulty_level - 0.05).max(0.0);
        }
        self.difficulty_level
    }

    pub fn mastery_level(&self) -> f64 {
        if self.tasks_completed.is_empty() {
            return 0.0;
        }
        let success_count = self
            .tasks_completed
            .iter()
            .filter(|r| r.success)
            .count();
        success_count as f64 / self.tasks_completed.len() as f64
    }

    pub fn learning_curve(&self) -> Vec<f64> {
        if self.tasks_completed.is_empty() {
            return Vec::new();
        }
        let w = self.window_size.min(self.tasks_completed.len());
        let chunk_size = (self.tasks_completed.len() + w - 1) / w;
        let mut curve = Vec::new();
        for chunk in self.tasks_completed.iter().collect::<Vec<_>>().chunks(chunk_size) {
            let successes = chunk.iter().filter(|r| r.success).count();
            curve.push(successes as f64 / chunk.len() as f64);
        }
        curve
    }
}

impl IterationValidator {
    pub fn new(min_reward_threshold: f64, max_failures: u32, convergence_window: usize) -> Self {
        Self {
            min_reward_threshold,
            max_failures,
            convergence_window,
        }
    }

    pub fn validate(&self, reward: f64, recent_rewards: &[f64]) -> ValidationResult {
        if reward < self.min_reward_threshold {
            return ValidationResult {
                is_valid: false,
                reason: format!(
                    "reward {} below threshold {}",
                    reward, self.min_reward_threshold
                ),
                suggested_action: "increase temperature or adjust target".into(),
            };
        }
        if self.check_convergence(recent_rewards) {
            return ValidationResult {
                is_valid: true,
                reason: "converged".into(),
                suggested_action: "proceed to next difficulty".into(),
            };
        }
        ValidationResult {
            is_valid: true,
            reason: "reward acceptable, not yet converged".into(),
            suggested_action: "continue training".into(),
        }
    }

    pub fn check_convergence(&self, recent_rewards: &[f64]) -> bool {
        if recent_rewards.len() < 2 {
            return false;
        }
        let window = recent_rewards.len().min(self.convergence_window);
        let sliced = &recent_rewards[recent_rewards.len() - window..];
        if sliced.is_empty() {
            return false;
        }
        let mean = sliced.iter().sum::<f64>() / sliced.len() as f64;
        let variance = sliced.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / sliced.len() as f64;
        variance < 0.01
    }
}

impl LearnabilityWindowAnalyzer {
    pub fn new(window_size: usize) -> Self {
        Self {
            recent_performance: VecDeque::new(),
            window_size,
        }
    }

    pub fn add_performance(&mut self, score: f64) {
        self.recent_performance.push_back(score);
        while self.recent_performance.len() > self.window_size {
            self.recent_performance.pop_front();
        }
    }

    pub fn is_learning(&self) -> bool {
        if self.recent_performance.len() < 6 {
            return false;
        }
        let vec: Vec<f64> = self.recent_performance.iter().copied().collect();
        let first_3: f64 = vec.iter().take(3).sum();
        let last_3: f64 = vec.iter().rev().take(3).sum();
        last_3 > first_3
    }

    pub fn learning_rate(&self) -> f64 {
        let n = self.recent_performance.len();
        if n < 2 {
            return 0.0;
        }
        let vec: Vec<f64> = self.recent_performance.iter().copied().collect();
        let mean_x = (n - 1) as f64 / 2.0;
        let mean_y = vec.iter().sum::<f64>() / n as f64;
        let mut num = 0.0;
        let mut den = 0.0;
        for (i, &y) in vec.iter().enumerate() {
            let dx = i as f64 - mean_x;
            let dy = y - mean_y;
            num += dx * dy;
            den += dx * dx;
        }
        if den.abs() < 1e-12 {
            0.0
        } else {
            num / den
        }
    }

    pub fn variance(&self) -> f64 {
        let n = self.recent_performance.len();
        if n < 2 {
            return 0.0;
        }
        let mean = self.recent_performance.iter().sum::<f64>() / n as f64;
        self.recent_performance
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / n as f64
    }

    pub fn plateau_detected(&self, tolerance: f64) -> bool {
        let n = self.recent_performance.len();
        if n < 3 {
            return false;
        }
        self.variance() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_new_default_difficulty() {
        let cc = CalibratedCurriculumGenerator::new(0.7, 10);
        assert!((cc.difficulty_level - 0.5).abs() < 1e-6);
        assert!((cc.mastery_threshold - 0.7).abs() < 1e-6);
        assert_eq!(cc.window_size, 10);
    }

    #[test]
    fn test_generate_next_task_selects_closest() {
        let cc = CalibratedCurriculumGenerator::new(0.7, 10);
        let tasks = vec![
            ("easy".into(), 0.3, vec![]),
            ("medium".into(), 0.5, vec![]),
            ("hard".into(), 0.8, vec![]),
        ];
        let next = cc.generate_next_task(&tasks);
        assert!(next.is_some());
        let (name, diff, _) = next.unwrap();
        assert_eq!(name, "medium");
        assert!((diff - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_generate_next_task_skips_too_hard() {
        let cc = CalibratedCurriculumGenerator::new(0.7, 10);
        let tasks = vec![("x".into(), 0.8, vec![])];
        let next = cc.generate_next_task(&tasks);
        assert!(next.is_none());
    }

    #[test]
    fn test_record_outcome_limits_window() {
        let mut cc = CalibratedCurriculumGenerator::new(0.7, 3);
        for i in 0..5 {
            cc.record_outcome(CurriculumRecord {
                task_id: format!("t{}", i),
                difficulty: 0.5,
                success: true,
                reward: 1.0,
                iterations: 1,
            });
        }
        assert_eq!(cc.tasks_completed.len(), 3);
    }

    #[test]
    fn test_adjust_difficulty_no_records() {
        let mut cc = CalibratedCurriculumGenerator::new(0.7, 10);
        let d = cc.adjust_difficulty();
        assert!((d - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_adjust_difficulty_increases_on_mastery() {
        let mut cc = CalibratedCurriculumGenerator::new(0.5, 10);
        for _ in 0..6 {
            cc.record_outcome(CurriculumRecord {
                task_id: "t".into(),
                difficulty: 0.5,
                success: true,
                reward: 1.0,
                iterations: 1,
            });
        }
        let d = cc.adjust_difficulty();
        assert!((d - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_adjust_difficulty_decreases_on_failure() {
        let mut cc = CalibratedCurriculumGenerator::new(0.7, 10);
        for _ in 0..6 {
            cc.record_outcome(CurriculumRecord {
                task_id: "t".into(),
                difficulty: 0.5,
                success: false,
                reward: 0.0,
                iterations: 1,
            });
        }
        let d = cc.adjust_difficulty();
        assert!((d - 0.45).abs() < 1e-6);
    }

    #[test]
    fn test_mastery_level() {
        let mut cc = CalibratedCurriculumGenerator::new(0.7, 10);
        for _ in 0..3 {
            cc.record_outcome(CurriculumRecord {
                task_id: "t".into(),
                difficulty: 0.5,
                success: true,
                reward: 1.0,
                iterations: 1,
            });
        }
        cc.record_outcome(CurriculumRecord {
            task_id: "t".into(),
            difficulty: 0.5,
            success: false,
            reward: 0.0,
            iterations: 1,
        });
        assert!((cc.mastery_level() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_learning_curve() {
        let mut cc = CalibratedCurriculumGenerator::new(0.7, 10);
        for i in 0..8 {
            cc.record_outcome(CurriculumRecord {
                task_id: format!("t{}", i),
                difficulty: 0.5,
                success: i % 2 == 0,
                reward: if i % 2 == 0 { 1.0 } else { 0.0 },
                iterations: 1,
            });
        }
        let curve = cc.learning_curve();
        assert!(!curve.is_empty());
        for v in &curve {
            assert!(*v >= 0.0 && *v <= 1.0);
        }
    }

    #[test]
    fn test_validation_valid() {
        let v = IterationValidator::new(0.3, 3, 5);
        let result = v.validate(0.8, &[0.7, 0.8, 0.75]);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validation_below_threshold() {
        let v = IterationValidator::new(0.5, 3, 5);
        let result = v.validate(0.2, &[]);
        assert!(!result.is_valid);
        assert!(result.reason.contains("below threshold"));
    }

    #[test]
    fn test_convergence_detected() {
        let v = IterationValidator::new(0.3, 3, 5);
        assert!(v.check_convergence(&[0.5, 0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_convergence_not_detected() {
        let v = IterationValidator::new(0.3, 3, 5);
        assert!(!v.check_convergence(&[0.1, 0.5, 0.9, 0.2]));
    }

    #[test]
    fn test_convergence_short_window() {
        let v = IterationValidator::new(0.3, 3, 5);
        assert!(!v.check_convergence(&[0.5]));
    }

    #[test]
    fn test_learnability_new() {
        let a = LearnabilityWindowAnalyzer::new(10);
        assert_eq!(a.window_size, 10);
        assert!(a.recent_performance.is_empty());
    }

    #[test]
    fn test_add_performance_limits_window() {
        let mut a = LearnabilityWindowAnalyzer::new(3);
        for i in 0..5 {
            a.add_performance(i as f64);
        }
        assert_eq!(a.recent_performance.len(), 3);
    }

    #[test]
    fn test_is_learning_positive_trend() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        for i in 0..8 {
            a.add_performance(i as f64 * 0.1);
        }
        assert!(a.is_learning());
    }

    #[test]
    fn test_is_learning_negative_trend() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        for i in (0..8).rev() {
            a.add_performance(i as f64 * 0.1);
        }
        assert!(!a.is_learning());
    }

    #[test]
    fn test_is_learning_too_few_points() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        a.add_performance(0.5);
        a.add_performance(0.6);
        assert!(!a.is_learning());
    }

    #[test]
    fn test_learning_rate_positive() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        for i in 0..5 {
            a.add_performance(i as f64);
        }
        assert!(a.learning_rate() > 0.0);
    }

    #[test]
    fn test_learning_rate_negative() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        for i in (0..5).rev() {
            a.add_performance(i as f64);
        }
        assert!(a.learning_rate() < 0.0);
    }

    #[test]
    fn test_learning_rate_too_few() {
        let a = LearnabilityWindowAnalyzer::new(10);
        assert!((a.learning_rate() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_variance() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        a.add_performance(0.5);
        a.add_performance(0.5);
        a.add_performance(0.5);
        assert!(a.variance() < 1e-6);
    }

    #[test]
    fn test_plateau_detected() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        for _ in 0..5 {
            a.add_performance(0.5);
        }
        assert!(a.plateau_detected(0.01));
    }

    #[test]
    fn test_plateau_not_detected() {
        let mut a = LearnabilityWindowAnalyzer::new(10);
        a.add_performance(0.1);
        a.add_performance(0.9);
        a.add_performance(0.1);
        assert!(!a.plateau_detected(0.01));
    }

    #[test]
    fn test_plateau_too_few_points() {
        let a = LearnabilityWindowAnalyzer::new(10);
        assert!(!a.plateau_detected(0.01));
    }
}
