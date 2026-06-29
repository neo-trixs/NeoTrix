use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::value_system::{CoreValue, ValueSystem};

const MAX_CONFLICT_HISTORY: usize = 100;
const MAX_PROFILES: usize = 1000;
const DEFAULT_LEARNING_RATE: f64 = 0.05;
const CONFLICT_WEIGHT_DIFF_THRESHOLD: f64 = 0.15;
const CONFLICT_LOW_THRESHOLD: f64 = 0.12;
const CONFLICT_HIGH_THRESHOLD: f64 = 0.28;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserValueConflict {
    pub conflicting_values: (String, String),
    pub resolution: String,
    pub user_preference: String,
    pub confidence: f64,
    pub cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserValueProfile {
    pub user_id: String,
    pub value_weights: HashMap<String, f64>,
    pub interaction_count: u64,
    pub avg_satisfaction: f64,
    pub conflict_history: Vec<UserValueConflict>,
    pub last_updated: u64,
}

impl UserValueProfile {
    pub fn new(user_id: &str) -> Self {
        let mut value_weights = HashMap::new();
        for cv in CoreValue::all() {
            value_weights.insert(cv.name().to_string(), cv.default_weight());
        }
        Self {
            user_id: user_id.to_string(),
            value_weights,
            interaction_count: 0,
            avg_satisfaction: 0.5,
            conflict_history: Vec::with_capacity(MAX_CONFLICT_HISTORY),
            last_updated: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueAlignmentEngine {
    pub profiles: HashMap<String, UserValueProfile>,
    pub max_profiles: usize,
    pub default_profile: UserValueProfile,
    pub base_system: ValueSystem,
}

fn detect_conflicts(profile: &mut UserValueProfile, now: u64) {
    let values: Vec<(String, f64)> = profile
        .value_weights
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();

    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            let (name_i, w_i) = &values[i];
            let (name_j, w_j) = &values[j];
            let diff = (w_i - w_j).abs();

            if diff < CONFLICT_WEIGHT_DIFF_THRESHOLD {
                let both_low = *w_i < CONFLICT_LOW_THRESHOLD && *w_j < CONFLICT_LOW_THRESHOLD;
                let both_high = *w_i > CONFLICT_HIGH_THRESHOLD && *w_j > CONFLICT_HIGH_THRESHOLD;

                if both_low || both_high {
                    if profile.conflict_history.len() >= MAX_CONFLICT_HISTORY {
                        profile.conflict_history.remove(0);
                    }

                    let already_exists = profile.conflict_history.iter().any(|c| {
                        (c.conflicting_values.0 == *name_i && c.conflicting_values.1 == *name_j)
                            || (c.conflicting_values.0 == *name_j
                                && c.conflicting_values.1 == *name_i)
                    });

                    if !already_exists {
                        let preference = if *w_i > *w_j {
                            name_i.clone()
                        } else {
                            name_j.clone()
                        };
                        profile.conflict_history.push(UserValueConflict {
                            conflicting_values: (name_i.clone(), name_j.clone()),
                            resolution: "auto_weight_update".to_string(),
                            user_preference: preference,
                            confidence: diff.max(0.01),
                            cycle: now,
                        });
                    }
                }
            }
        }
    }
}

impl ValueAlignmentEngine {
    pub fn new() -> Self {
        let base_system = ValueSystem::new();
        let default_profile = UserValueProfile::new("__default__");
        Self {
            profiles: HashMap::new(),
            max_profiles: MAX_PROFILES,
            default_profile,
            base_system,
        }
    }

    pub fn get_or_create_profile(&mut self, user_id: &str) -> &mut UserValueProfile {
        let max = self.max_profiles;
        if !self.profiles.contains_key(user_id) && self.profiles.len() >= max {
            let oldest_id = self
                .profiles
                .iter()
                .min_by(|a, b| a.1.last_updated.cmp(&b.1.last_updated))
                .map(|(id, _)| id.clone());
            if let Some(id) = oldest_id {
                self.profiles.remove(&id);
            }
        }
        let profile = UserValueProfile::new(user_id);
        self.profiles.entry(user_id.to_string()).or_insert(profile)
    }

    pub fn record_interaction(&mut self, user_id: &str, _action: &str, satisfaction: f64) {
        let now = {
            self.base_system.total_cycles += 1;
            self.base_system.total_cycles
        };
        let profile = self.get_or_create_profile(user_id);
        profile.interaction_count += 1;
        profile.last_updated = now;

        let decayed_lr = DEFAULT_LEARNING_RATE / (1.0 + 0.01 * profile.interaction_count as f64);
        let lr = decayed_lr.clamp(0.001, 0.05);

        let sat = satisfaction.clamp(0.0, 1.0);
        for w in profile.value_weights.values_mut() {
            *w = *w * (1.0 - lr) + sat * lr;
        }

        let sum: f64 = profile.value_weights.values().sum();
        if sum > 0.0 {
            for w in profile.value_weights.values_mut() {
                *w = (*w / sum).clamp(0.0, 1.0);
            }
        }

        let n = profile.interaction_count as f64;
        profile.avg_satisfaction = profile.avg_satisfaction * ((n - 1.0) / n) + sat * (1.0 / n);

        detect_conflicts(profile, profile.last_updated);
    }

    pub fn resolve_conflict(
        &mut self,
        user_id: &str,
        v1: &str,
        v2: &str,
        preference: &str,
    ) -> bool {
        self.base_system.total_cycles += 1;
        let now = self.base_system.total_cycles;
        let profile = match self.profiles.get_mut(user_id) {
            Some(p) => p,
            None => return false,
        };

        profile.last_updated = now;

        let boost = 0.1;
        if let Some(w) = profile.value_weights.get_mut(preference) {
            *w = (*w + boost).min(1.0);
        }
        let non_preferred = if preference == v1 { v2 } else { v1 };
        if let Some(w) = profile.value_weights.get_mut(non_preferred) {
            *w = (*w - boost).max(0.0);
        }

        let sum: f64 = profile.value_weights.values().sum();
        if sum > 0.0 {
            for w in profile.value_weights.values_mut() {
                *w = (*w / sum).clamp(0.0, 1.0);
            }
        }

        if profile.conflict_history.len() >= MAX_CONFLICT_HISTORY {
            profile.conflict_history.remove(0);
        }

        profile.conflict_history.push(UserValueConflict {
            conflicting_values: (v1.to_string(), v2.to_string()),
            resolution: "manual_resolution".to_string(),
            user_preference: preference.to_string(),
            confidence: boost,
            cycle: profile.last_updated,
        });

        true
    }

    pub fn profile_summary(&self, user_id: &str) -> String {
        let profile = match self.profiles.get(user_id) {
            Some(p) => p,
            None => return format!("No profile for user '{}'", user_id),
        };

        let mut weights: Vec<(&String, &f64)> = profile.value_weights.iter().collect();
        weights.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        let weights_str: Vec<String> = weights
            .iter()
            .map(|(k, v)| format!("{}: {:.3}", k, v))
            .collect();

        let conflict_count = profile.conflict_history.len();
        format!(
            "User: {} | interactions: {} | avg_satisfaction: {:.3} | conflicts: {} | weights: [{}]",
            user_id,
            profile.interaction_count,
            profile.avg_satisfaction,
            conflict_count,
            weights_str.join(", "),
        )
    }

    pub fn user_alignment_score(&self, user_id: &str, _action: &str) -> f64 {
        let profile = match self.profiles.get(user_id) {
            Some(p) => p,
            None => return self.default_alignment_score(),
        };

        if profile.value_weights.is_empty() {
            return 0.5;
        }

        let dominant_weight = profile
            .value_weights
            .values()
            .cloned()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        let avg_weight: f64 =
            profile.value_weights.values().sum::<f64>() / profile.value_weights.len() as f64;

        let satisfaction_bonus = profile.avg_satisfaction * 0.2;

        (avg_weight * 0.5 + dominant_weight * 0.3 + satisfaction_bonus).clamp(0.0, 1.0)
    }

    fn default_alignment_score(&self) -> f64 {
        0.5
    }

    pub fn prune_inactive_profiles(&mut self, max_age: u64) {
        let now = self.base_system.total_cycles;
        self.profiles.retain(|_, p| {
            now.saturating_sub(p.last_updated) <= max_age || p.user_id == "__default__"
        });
    }
}

impl Default for ValueAlignmentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine_has_default_profile() {
        let mut engine = ValueAlignmentEngine::new();
        let profile = engine.get_or_create_profile("test_user");
        assert_eq!(profile.user_id, "test_user");
        assert_eq!(profile.value_weights.len(), CoreValue::all().len());
        assert_eq!(profile.interaction_count, 0);
    }

    #[test]
    fn test_record_interaction_adjusts_weights() {
        let mut engine = ValueAlignmentEngine::new();
        let initial = {
            let p = engine.get_or_create_profile("user1");
            p.value_weights.get("curiosity").copied().unwrap()
        };

        engine.record_interaction("user1", "test_action", 0.9);
        let profile = engine.profiles.get("user1").unwrap();
        let updated = profile.value_weights.get("curiosity").copied().unwrap();

        assert!(
            (updated - initial).abs() > 0.001,
            "weight should change after interaction"
        );
        assert_eq!(profile.interaction_count, 1);
        assert!((profile.avg_satisfaction - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_profile_summary_includes_weights() {
        let mut engine = ValueAlignmentEngine::new();
        engine.record_interaction("user2", "test_action", 0.85);

        let summary = engine.profile_summary("user2");

        assert!(
            summary.contains("curiosity"),
            "summary should contain value names"
        );
        assert!(summary.contains("user2"), "summary should contain user id");
        assert!(
            summary.contains("weights:"),
            "summary should contain weights section"
        );
    }

    #[test]
    fn test_conflict_detection_and_resolution() {
        let mut engine = ValueAlignmentEngine::new();

        for _ in 0..15 {
            engine.record_interaction("user3", "test_action", 0.95);
        }

        let has_conflicts = {
            let p = engine.profiles.get("user3").unwrap();
            !p.conflict_history.is_empty()
        };
        assert!(
            has_conflicts,
            "repeated high satisfaction should trigger conflict detection"
        );

        let resolved =
            engine.resolve_conflict("user3", "curiosity", "knowledge_growth", "curiosity");
        assert!(resolved, "conflict resolution should succeed");

        let profile = engine.profiles.get("user3").unwrap();
        let last_conflict = profile.conflict_history.last().unwrap();
        assert_eq!(last_conflict.resolution, "manual_resolution");
        assert_eq!(last_conflict.user_preference, "curiosity");
    }

    #[test]
    fn test_alignment_score_higher_for_aligned_actions() {
        let mut engine = ValueAlignmentEngine::new();

        let score_before = engine.user_alignment_score("aligned_user", "some_action");
        assert!(
            (score_before - 0.5).abs() < 0.1,
            "default score should be near 0.5"
        );

        for _ in 0..5 {
            engine.record_interaction("aligned_user", "aligned_action", 0.95);
        }

        let score_after = engine.user_alignment_score("aligned_user", "aligned_action");
        assert!(
            score_after > score_before,
            "score should increase after positive interactions"
        );
    }

    #[test]
    fn test_prune_inactive_profiles() {
        let mut engine = ValueAlignmentEngine::new();

        engine.get_or_create_profile("active_user");
        engine.get_or_create_profile("inactive_user");

        engine.base_system.total_cycles = 100;

        {
            let active = engine.get_or_create_profile("active_user");
            active.last_updated = 95;
        }
        {
            let inactive = engine.get_or_create_profile("inactive_user");
            inactive.last_updated = 50;
        }

        engine.prune_inactive_profiles(30);

        assert!(
            engine.profiles.contains_key("active_user"),
            "active user should remain"
        );
        assert!(
            !engine.profiles.contains_key("inactive_user"),
            "inactive user should be pruned"
        );
    }
}
