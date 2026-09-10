use crate::agents::sim_agent::Personality;

/// Configuration for how personality drifts
pub struct DriftConfig {
    /// How fast personality changes per update (0.001 = slow, 0.01 = fast)
    pub drift_rate: f32,
    /// Maximum drift per update
    pub max_drift_per_update: f32,
    /// Personality becomes more stable over time (drift decreases with age)
    pub age_damping: bool,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            drift_rate: 0.005,
            max_drift_per_update: 0.02,
            age_damping: true,
        }
    }
}

/// Tracks experience signals that drive personality drift
pub struct ExperienceSignal {
    pub social_success: f32,
    pub exploration_reward: f32,
    pub survival_stress: f32,
    pub achievement: f32,
    pub novelty_exposure: f32,
}

impl Default for ExperienceSignal {
    fn default() -> Self {
        Self {
            social_success: 0.0,
            exploration_reward: 0.0,
            survival_stress: 0.0,
            achievement: 0.0,
            novelty_exposure: 0.0,
        }
    }
}

/// Personality drift engine
pub struct PersonalityDrift {
    config: DriftConfig,
    /// Accumulated experience since last drift
    accumulated: ExperienceSignal,
    /// How many updates since last drift
    updates_since_drift: u64,
    /// Drift every N updates
    drift_interval: u64,
}

impl PersonalityDrift {
    pub fn new(config: DriftConfig) -> Self {
        Self {
            config,
            accumulated: ExperienceSignal::default(),
            updates_since_drift: 0,
            drift_interval: 10,
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.drift_interval = interval;
        self
    }

    /// Record an experience signal
    pub fn record(&mut self, signal: ExperienceSignal) {
        self.accumulated.social_success += signal.social_success;
        self.accumulated.exploration_reward += signal.exploration_reward;
        self.accumulated.survival_stress += signal.survival_stress;
        self.accumulated.achievement += signal.achievement;
        self.accumulated.novelty_exposure += signal.novelty_exposure;
        self.updates_since_drift += 1;
    }

    /// Apply drift to personality based on accumulated experience.
    /// Returns the new personality.
    pub fn drift(&mut self, personality: &Personality, age: u64) -> Personality {
        if self.updates_since_drift < self.drift_interval {
            return personality.clone();
        }

        self.updates_since_drift = 0;
        let rate = if self.config.age_damping {
            self.config.drift_rate / (1.0 + age as f32 / 1000.0)
        } else {
            self.config.drift_rate
        };

        let mut new_p = personality.clone();

        // Social success -> increase sociability, cooperativeness
        let social_delta = (self.accumulated.social_success * rate)
            .clamp(-self.config.max_drift_per_update, self.config.max_drift_per_update);
        new_p.sociability = (new_p.sociability + social_delta).clamp(0.0, 1.0);
        new_p.cooperativeness = (new_p.cooperativeness + social_delta * 0.5).clamp(0.0, 1.0);

        // Exploration reward -> increase curiosity, openness
        let explore_delta = (self.accumulated.exploration_reward * rate)
            .clamp(-self.config.max_drift_per_update, self.config.max_drift_per_update);
        new_p.curiosity = (new_p.curiosity + explore_delta).clamp(0.0, 1.0);
        new_p.openness = (new_p.openness + explore_delta * 0.7).clamp(0.0, 1.0);

        // Survival stress -> increase aggression (defensive), decrease openness
        let stress_delta = (self.accumulated.survival_stress * rate)
            .clamp(-self.config.max_drift_per_update, self.config.max_drift_per_update);
        new_p.aggression = (new_p.aggression + stress_delta * 0.3).clamp(0.0, 1.0);
        new_p.openness = (new_p.openness - stress_delta * 0.2).clamp(0.0, 1.0);

        // Achievement -> increase all slightly (confidence effect)
        let achieve_delta = (self.accumulated.achievement * rate * 0.3)
            .clamp(-self.config.max_drift_per_update, self.config.max_drift_per_update);
        new_p.openness = (new_p.openness + achieve_delta).clamp(0.0, 1.0);
        new_p.curiosity = (new_p.curiosity + achieve_delta).clamp(0.0, 1.0);

        // Reset accumulator
        self.accumulated = ExperienceSignal::default();

        new_p
    }

    pub fn accumulated(&self) -> &ExperienceSignal {
        &self.accumulated
    }

    pub fn config(&self) -> &DriftConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_personality() -> Personality {
        Personality::default()
    }

    fn signal_social_success(v: f32) -> ExperienceSignal {
        ExperienceSignal {
            social_success: v,
            ..ExperienceSignal::default()
        }
    }

    fn signal_exploration(v: f32) -> ExperienceSignal {
        ExperienceSignal {
            exploration_reward: v,
            ..ExperienceSignal::default()
        }
    }

    fn signal_stress(v: f32) -> ExperienceSignal {
        ExperienceSignal {
            survival_stress: v,
            ..ExperienceSignal::default()
        }
    }

    fn signal_achievement(v: f32) -> ExperienceSignal {
        ExperienceSignal {
            achievement: v,
            ..ExperienceSignal::default()
        }
    }

    #[test]
    fn no_drift_before_interval() {
        let mut drift = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let p = default_personality();
        // Record 5 signals (less than interval of 10)
        for _ in 0..5 {
            drift.record(signal_social_success(1.0));
        }
        let new_p = drift.drift(&p, 0);
        assert_eq!(new_p.sociability, p.sociability);
        assert_eq!(new_p.openness, p.openness);
    }

    #[test]
    fn social_success_increases_sociability() {
        let mut drift = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let p = default_personality();
        for _ in 0..10 {
            drift.record(signal_social_success(1.0));
        }
        let new_p = drift.drift(&p, 0);
        assert!(new_p.sociability > p.sociability);
        assert!(new_p.cooperativeness > p.cooperativeness);
    }

    #[test]
    fn exploration_increases_curiosity() {
        let mut drift = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let p = default_personality();
        for _ in 0..10 {
            drift.record(signal_exploration(1.0));
        }
        let new_p = drift.drift(&p, 0);
        assert!(new_p.curiosity > p.curiosity);
        assert!(new_p.openness > p.openness);
    }

    #[test]
    fn stress_increases_aggression() {
        let mut drift = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let p = default_personality();
        for _ in 0..10 {
            drift.record(signal_stress(1.0));
        }
        let new_p = drift.drift(&p, 0);
        assert!(new_p.aggression > p.aggression);
    }

    #[test]
    fn age_damping_reduces_drift() {
        let mut drift_young = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let mut drift_old = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let p = default_personality();

        for _ in 0..10 {
            drift_young.record(signal_social_success(1.0));
            drift_old.record(signal_social_success(1.0));
        }

        let new_young = drift_young.drift(&p, 10);
        let new_old = drift_old.drift(&p, 10_000);

        let delta_young = new_young.sociability - p.sociability;
        let delta_old = new_old.sociability - p.sociability;
        assert!(delta_young > delta_old);
    }

    #[test]
    fn personality_stays_in_bounds() {
        let config = DriftConfig {
            drift_rate: 0.5,
            max_drift_per_update: 1.0,
            age_damping: false,
        };
        let mut drift = PersonalityDrift::new(config).with_interval(1);
        // Start at extremes
        let mut p = default_personality();
        p.sociability = 0.99;
        p.curiosity = 0.99;

        // Apply many rounds of positive signals
        for _ in 0..100 {
            drift.record(ExperienceSignal {
                social_success: 1.0,
                exploration_reward: 1.0,
                survival_stress: 0.0,
                achievement: 1.0,
                novelty_exposure: 0.0,
            });
            p = drift.drift(&p, 0);
        }
        assert!(p.sociability >= 0.0 && p.sociability <= 1.0);
        assert!(p.curiosity >= 0.0 && p.curiosity <= 1.0);
        assert!(p.openness >= 0.0 && p.openness <= 1.0);
        assert!(p.aggression >= 0.0 && p.aggression <= 1.0);
        assert!(p.cooperativeness >= 0.0 && p.cooperativeness <= 1.0);
    }

    #[test]
    fn accumulator_resets_after_drift() {
        let mut drift = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        for _ in 0..10 {
            drift.record(signal_social_success(5.0));
        }
        let _ = drift.drift(&default_personality(), 0);
        assert_eq!(drift.accumulated().social_success, 0.0);
        assert_eq!(drift.accumulated().exploration_reward, 0.0);
    }

    #[test]
    fn max_drift_limits_change() {
        let config = DriftConfig {
            drift_rate: 10.0,
            max_drift_per_update: 0.05,
            age_damping: false,
        };
        let mut drift = PersonalityDrift::new(config).with_interval(1);
        let p = default_personality();

        drift.record(signal_social_success(100.0));
        let new_p = drift.drift(&p, 0);
        let delta = (new_p.sociability - p.sociability).abs();
        assert!(delta <= 0.05 + f32::EPSILON);
    }

    #[test]
    fn negative_signal_reverses_drift() {
        let mut drift = PersonalityDrift::new(DriftConfig::default()).with_interval(10);
        let p = default_personality();
        for _ in 0..10 {
            drift.record(ExperienceSignal {
                social_success: -1.0,
                exploration_reward: 0.0,
                survival_stress: 0.0,
                achievement: 0.0,
                novelty_exposure: 0.0,
            });
        }
        let new_p = drift.drift(&p, 0);
        assert!(new_p.sociability < p.sociability);
    }
}
