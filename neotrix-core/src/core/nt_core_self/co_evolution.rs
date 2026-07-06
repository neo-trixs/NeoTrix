#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionMode {
    Teach,
    Collaborate,
    Delegate,
    Observe,
}

#[derive(Debug, Clone)]
pub struct TrustLevel {
    pub trust_score: f64,
    pub calibration: f64,
    pub history_count: u32,
}

impl TrustLevel {
    pub fn new() -> Self {
        Self {
            trust_score: 0.8,
            calibration: 0.5,
            history_count: 0,
        }
    }
}

impl Default for TrustLevel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InteractionRecord {
    pub mode: InteractionMode,
    pub task: String,
    pub success: bool,
    pub user_satisfaction: f64,
    pub tokens_used: u64,
}

#[derive(Debug, Clone)]
pub struct CoEvolutionProfile {
    pub user_id: String,
    pub trust: TrustLevel,
    pub interaction_history: VecDeque<InteractionRecord>,
    pub preferred_modes: Vec<InteractionMode>,
    pub adaptation_rate: f64,
}

impl CoEvolutionProfile {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            trust: TrustLevel::new(),
            interaction_history: VecDeque::with_capacity(200),
            preferred_modes: vec![InteractionMode::Collaborate],
            adaptation_rate: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoEvolutionConfig {
    pub max_history: usize,
    pub trust_decay: f64,
}

impl Default for CoEvolutionConfig {
    fn default() -> Self {
        Self {
            max_history: 200,
            trust_decay: 0.01,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoEvolutionStats {
    pub total_interactions: u32,
    pub profiles: u32,
    pub avg_trust: f64,
    pub mode_distribution: HashMap<InteractionMode, u32>,
}

#[derive(Debug, Clone)]
pub struct CoEvolutionTracker {
    profiles: HashMap<String, CoEvolutionProfile>,
    config: CoEvolutionConfig,
}

impl CoEvolutionTracker {
    pub fn new(config: CoEvolutionConfig) -> Self {
        Self {
            profiles: HashMap::new(),
            config,
        }
    }

    pub fn record_interaction(
        &mut self,
        user_id: &str,
        mode: InteractionMode,
        task: impl Into<String>,
        success: bool,
        satisfaction: f64,
    ) {
        let profile = self
            .profiles
            .entry(user_id.to_string())
            .or_insert_with(|| CoEvolutionProfile::new(user_id));
        let satisfaction = satisfaction.max(0.0).min(1.0);
        let record = InteractionRecord {
            mode,
            task: task.into(),
            success,
            user_satisfaction: satisfaction,
            tokens_used: 0,
        };
        if profile.interaction_history.len() >= self.config.max_history {
            profile.interaction_history.pop_front();
        }
        profile.interaction_history.push_back(record);
        profile.trust.history_count += 1;
        profile.trust.trust_score = (profile.trust.trust_score * 0.95
            + if success { 0.05 }
            else { -0.05 })
        .max(0.0)
        .min(1.0);
        self.update_mode_preference(user_id, mode, success);
    }

    fn update_mode_preference(&mut self, user_id: &str, mode: InteractionMode, success: bool) {
        if let Some(profile) = self.profiles.get_mut(user_id) {
            if success && !profile.preferred_modes.contains(&mode) {
                profile.preferred_modes.push(mode);
            }
        }
    }

    pub fn get_trust(&self, user_id: &str) -> Option<f64> {
        self.profiles.get(user_id).map(|p| p.trust.trust_score)
    }

    pub fn update_trust(&mut self, user_id: &str, delta: f64) {
        if let Some(profile) = self.profiles.get_mut(user_id) {
            profile.trust.trust_score = (profile.trust.trust_score + delta).max(0.0).min(1.0);
        }
    }

    pub fn recommend_mode(&self, user_id: &str) -> InteractionMode {
        if let Some(profile) = self.profiles.get(user_id) {
            if profile.trust.trust_score > 0.7 {
                if profile.preferred_modes.contains(&InteractionMode::Delegate) {
                    return InteractionMode::Delegate;
                }
                return InteractionMode::Collaborate;
            }
            if profile.trust.trust_score > 0.4 {
                return InteractionMode::Teach;
            }
        }
        InteractionMode::Observe
    }

    pub fn detect_pattern(&self, user_id: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        if let Some(profile) = self.profiles.get(user_id) {
            let mut technical = 0u32;
            let mut creative = 0u32;
            let mut debugging = 0u32;
            for record in &profile.interaction_history {
                let task_lower = record.task.to_lowercase();
                if task_lower.contains("code") || task_lower.contains("function") {
                    technical += 1;
                }
                if task_lower.contains("design") || task_lower.contains("create") {
                    creative += 1;
                }
                if task_lower.contains("fix") || task_lower.contains("error") {
                    debugging += 1;
                }
            }
            let total = profile.interaction_history.len() as f64;
            if total > 0.0 {
                let t_ratio = technical as f64 / total;
                let c_ratio = creative as f64 / total;
                let d_ratio = debugging as f64 / total;
                if t_ratio > 0.3 {
                    patterns.push("technical".to_string());
                }
                if c_ratio > 0.3 {
                    patterns.push("creative".to_string());
                }
                if d_ratio > 0.3 {
                    patterns.push("debugging".to_string());
                }
            }
        }
        patterns
    }

    pub fn adapt_rate(&mut self, user_id: &str) {
        if let Some(profile) = self.profiles.get_mut(user_id) {
            let recent: Vec<&InteractionRecord> = profile
                .interaction_history
                .iter()
                .rev()
                .take(20)
                .collect();
            if recent.is_empty() {
                return;
            }
            let success_count = recent.iter().filter(|r| r.success).count();
            let rate = success_count as f64 / recent.len() as f64;
            profile.adaptation_rate = (0.3 * rate + 0.7 * profile.adaptation_rate)
                .max(0.0)
                .min(1.0);
        }
    }

    pub fn stats(&self) -> CoEvolutionStats {
        let mut total_interactions = 0u32;
        let mut trust_sum = 0.0f64;
        let mut mode_distribution: HashMap<InteractionMode, u32> = HashMap::new();

        for profile in self.profiles.values() {
            total_interactions += profile.trust.history_count;
            trust_sum += profile.trust.trust_score;
            for record in &profile.interaction_history {
                *mode_distribution.entry(record.mode).or_insert(0) += 1;
            }
        }

        let avg_trust = if self.profiles.is_empty() {
            0.0
        } else {
            trust_sum / self.profiles.len() as f64
        };

        CoEvolutionStats {
            total_interactions,
            profiles: self.profiles.len() as u32,
            avg_trust,
            mode_distribution,
        }
    }
}

impl Default for CoEvolutionTracker {
    fn default() -> Self {
        Self::new(CoEvolutionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_starts_high() {
        let trust = TrustLevel::new();
        assert!((trust.trust_score - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_record_increases_history() {
        let mut tracker = CoEvolutionTracker::default();
        tracker.record_interaction("user1", InteractionMode::Collaborate, "write code", true, 0.9);
        let stats = tracker.stats();
        assert_eq!(stats.total_interactions, 1);
    }

    #[test]
    fn test_recommend_mode_collaborate_on_high_trust() {
        let mut tracker = CoEvolutionTracker::default();
        tracker.record_interaction("user1", InteractionMode::Collaborate, "task", true, 1.0);
        tracker.record_interaction("user1", InteractionMode::Collaborate, "task2", true, 1.0);
        let mode = tracker.recommend_mode("user1");
        assert_eq!(mode, InteractionMode::Collaborate);
    }

    #[test]
    fn test_detect_pattern_technical() {
        let mut tracker = CoEvolutionTracker::default();
        tracker.record_interaction("u1", InteractionMode::Teach, "write code function", true, 0.8);
        tracker.record_interaction("u1", InteractionMode::Teach, "fix error in code", true, 0.7);
        tracker.record_interaction("u1", InteractionMode::Teach, "design new feature", true, 0.9);
        let patterns = tracker.detect_pattern("u1");
        assert!(patterns.contains(&"technical".to_string()));
    }

    #[test]
    fn test_adapt_rate_changes() {
        let mut tracker = CoEvolutionTracker::default();
        for _ in 0..10 {
            tracker.record_interaction("u1", InteractionMode::Delegate, "task", true, 1.0);
        }
        tracker.adapt_rate("u1");
        let trust = tracker.get_trust("u1").unwrap();
        assert!(trust > 0.8);
    }
}
