use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{
    BehaviorPattern, PatternType, StealthProfile, StealthReport,
};

const DEFAULT_PROFILES: &[&str] = &["default", "stealth", "aggressive", "conservative"];

pub struct StealthManager {
    profiles: Vec<StealthProfile>,
    active_index: usize,
}

impl StealthManager {
    pub fn new() -> Self {
        let profiles: Vec<StealthProfile> = DEFAULT_PROFILES
            .iter()
            .enumerate()
            .map(|(i, name)| build_profile(name, i as u64))
            .collect();
        Self {
            profiles,
            active_index: 0,
        }
    }

    pub fn assess_stealth(&self) -> StealthReport {
        let active_profile = &self.profiles[self.active_index];
        let active_profiles: Vec<String> = self
            .profiles
            .iter()
            .map(|p| p.profile_id.clone())
            .collect();

        let base_stealth = active_profile.success_rate;
        let pattern_coverage: f64 = if active_profile.behavior_patterns.is_empty() {
            0.0
        } else {
            let weighted: f64 = active_profile
                .behavior_patterns
                .iter()
                .map(|p| p.weight)
                .sum();
            weighted / active_profile.behavior_patterns.len() as f64
        };
        let current_stealth_level = (base_stealth * 0.6 + pattern_coverage * 0.4).max(0.0).min(1.0);

        let age_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
            - active_profile.last_used;
        let staleness_risk = (age_secs as f64 / 86400.0).min(1.0);

        let risk_assessment = ((1.0 - current_stealth_level) * 0.7 + staleness_risk * 0.3)
            .max(0.0)
            .min(1.0);

        let mut recommended_actions = Vec::new();
        if current_stealth_level < 0.4 {
            recommended_actions.push(
                "stealth level critically low — rotate profile immediately".to_string(),
            );
        } else if current_stealth_level < 0.7 {
            recommended_actions.push(
                "moderate stealth level — consider profile rotation or pattern diversification"
                    .to_string(),
            );
        }
        if staleness_risk > 0.5 {
            recommended_actions.push(
                "profile stale — refresh behavior patterns".to_string(),
            );
        }
        if age_secs > 3600 {
            recommended_actions.push(
                "timing jitter patterns may need recalibration".to_string(),
            );
        }

        StealthReport {
            current_stealth_level,
            active_profiles,
            recommended_actions,
            risk_assessment,
        }
    }

    pub fn rotate_profile(&mut self) -> Option<StealthProfile> {
        if self.profiles.is_empty() {
            return None;
        }
        self.active_index = (self.active_index + 1) % self.profiles.len();
        let profile = &mut self.profiles[self.active_index];
        profile.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Some(profile.clone())
    }

    pub fn record_outcome(&mut self, profile_id: &str, detected: bool) {
        if let Some(profile) = self
            .profiles
            .iter_mut()
            .find(|p| p.profile_id == profile_id)
        {
            let alpha = 0.3;
            let outcome = if detected { 0.0 } else { 1.0 };
            profile.success_rate =
                (profile.success_rate * (1.0 - alpha) + outcome * alpha).max(0.0).min(1.0);
        }
    }

    pub fn get_active_profile(&self) -> &StealthProfile {
        &self.profiles[self.active_index]
    }

    pub fn profile_count(&self) -> usize {
        self.profiles.len()
    }
}

impl Default for StealthManager {
    fn default() -> Self {
        Self::new()
    }
}

fn build_profile(name: &str, seed: u64) -> StealthProfile {
    let fingerprint = format!("fp_{}_{:x}", name, seed.wrapping_mul(0x9e3779b9));
    let patterns = match name {
        "stealth" => vec![
            BehaviorPattern::new(
                PatternType::TimingJitter,
                HashMap::from([("jitter_ms".to_string(), 500.0), ("jitter_std".to_string(), 150.0)]),
                0.8,
            ),
            BehaviorPattern::new(
                PatternType::RequestRandomization,
                HashMap::from([("batch_size".to_string(), 3.0), ("interval_s".to_string(), 2.0)]),
                0.7,
            ),
            BehaviorPattern::new(
                PatternType::NoiseInjection,
                HashMap::from([("noise_level".to_string(), 0.05), ("noise_type".to_string(), 1.0)]),
                0.5,
            ),
            BehaviorPattern::new(
                PatternType::ProfileRotation,
                HashMap::from([("rotation_interval".to_string(), 300.0)]),
                0.6,
            ),
        ],
        "aggressive" => vec![
            BehaviorPattern::new(
                PatternType::RequestRandomization,
                HashMap::from([("batch_size".to_string(), 10.0), ("interval_s".to_string(), 0.5)]),
                0.9,
            ),
            BehaviorPattern::new(
                PatternType::ProfileRotation,
                HashMap::from([("rotation_interval".to_string(), 60.0)]),
                0.8,
            ),
        ],
        "conservative" => vec![
            BehaviorPattern::new(
                PatternType::TimingJitter,
                HashMap::from([("jitter_ms".to_string(), 2000.0), ("jitter_std".to_string(), 500.0)]),
                0.9,
            ),
            BehaviorPattern::new(
                PatternType::NoiseInjection,
                HashMap::from([("noise_level".to_string(), 0.02), ("noise_type".to_string(), 0.0)]),
                0.6,
            ),
        ],
        _ => vec![BehaviorPattern::new(
            PatternType::TimingJitter,
            HashMap::from([("jitter_ms".to_string(), 100.0), ("jitter_std".to_string(), 50.0)]),
            0.5,
        )],
    };
    StealthProfile::new(name, patterns, fingerprint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_assessment_baseline() {
        let manager = StealthManager::new();
        let report = manager.assess_stealth();
        assert!(report.current_stealth_level >= 0.0);
        assert!(report.current_stealth_level <= 1.0);
        assert!(!report.active_profiles.is_empty());
        assert_eq!(report.active_profiles.len(), 4);
    }

    #[test]
    fn test_profile_rotation_changes_active() {
        let mut manager = StealthManager::new();
        let first = manager.get_active_profile().profile_id.clone();
        let rotated = manager.rotate_profile();
        assert!(rotated.is_some());
        let second = rotated.unwrap();
        if manager.profile_count() > 1 {
            assert!(first != second.profile_id || manager.profile_count() == 1);
        }
    }

    #[test]
    fn test_profile_rotation_wraps_around() {
        let mut manager = StealthManager::new();
        let n = manager.profile_count();
        for _ in 0..n {
            manager.rotate_profile();
        }
        assert!(manager.get_active_profile().last_used > 0);
    }

    #[test]
    fn test_success_rate_tracking_detected() {
        let mut manager = StealthManager::new();
        let initial = manager.get_active_profile().success_rate;
        manager.record_outcome("default", true);
        let after = manager.get_active_profile().success_rate;
        assert!(after <= initial + 1e-10);
    }

    #[test]
    fn test_success_rate_tracking_undetected() {
        let mut manager = StealthManager::new();
        let initial = manager.get_active_profile().success_rate;
        manager.record_outcome("default", false);
        let after = manager.get_active_profile().success_rate;
        assert!(after >= initial - 1e-10);
    }

    #[test]
    fn test_record_outcome_unknown_profile() {
        let mut manager = StealthManager::new();
        manager.record_outcome("nonexistent", true);
        assert_eq!(manager.profile_count(), 4);
    }

    #[test]
    fn test_recommended_actions_on_low_stealth() {
        let mut manager = StealthManager::new();
        for _ in 0..5 {
            manager.record_outcome("default", true);
        }
        let report = manager.assess_stealth();
        let has_rotate = report.recommended_actions.iter().any(|a| a.contains("rotate"));
        assert!(has_rotate);
    }

    #[test]
    fn test_risk_assessment_range() {
        let manager = StealthManager::new();
        let report = manager.assess_stealth();
        assert!(report.risk_assessment >= 0.0);
        assert!(report.risk_assessment <= 1.0);
    }

    #[test]
    fn test_active_profile_count() {
        let manager = StealthManager::new();
        assert_eq!(manager.profile_count(), 4);
    }

    #[test]
    fn test_default_profile_has_timing_jitter() {
        let profile = build_profile("default", 0);
        let has_jitter = profile.behavior_patterns.iter().any(|p| {
            p.pattern_type == PatternType::TimingJitter
        });
        assert!(has_jitter);
    }

    #[test]
    fn test_stealth_profile_has_all_patterns() {
        let profile = build_profile("stealth", 1);
        assert_eq!(profile.behavior_patterns.len(), 4);
        let types: Vec<&PatternType> = profile
            .behavior_patterns
            .iter()
            .map(|p| &p.pattern_type)
            .collect();
        assert!(types.contains(&&PatternType::TimingJitter));
        assert!(types.contains(&&PatternType::RequestRandomization));
        assert!(types.contains(&&PatternType::NoiseInjection));
        assert!(types.contains(&&PatternType::ProfileRotation));
    }

    #[test]
    fn test_rotate_returns_profile() {
        let mut manager = StealthManager::new();
        let profile = manager.rotate_profile();
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().profile_id, "stealth");
    }

    #[test]
    fn test_repeated_detections_lower_stealth() {
        let mut manager = StealthManager::new();
        for _ in 0..10 {
            manager.record_outcome("default", true);
        }
        let report = manager.assess_stealth();
        assert!(report.current_stealth_level < 0.5);
    }
}
