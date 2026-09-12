//! ScalingInter-RL progressive horizon scheduler.
//!
//! Gradually increases the maximum number of turns allowed per episode,
//! enabling curriculum learning from short to long-horizon reasoning.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Config
// ═══════════════════════════════════════════════════════════════════

/// Configuration for the ScalingInter-RL scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Starting horizon (max turns per episode).
    pub initial_horizon: usize,
    /// How many steps before the horizon increases.
    pub step_size: usize,
    /// Number of scheduler ticks between horizon increments.
    pub scaling_interval: usize,
    /// Maximum horizon (ceiling).
    pub max_horizon: usize,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            initial_horizon: 10,
            step_size: 5,
            scaling_interval: 100,
            max_horizon: 200,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Scheduler
// ═══════════════════════════════════════════════════════════════════

/// Progressive horizon scheduler for ScalingInter-RL.
///
/// Tracks training steps and scales the maximum episode horizon
/// according to a linear schedule.
pub struct ScalingScheduler {
    config: ScalingConfig,
    current_horizon: usize,
    step_count: usize,
}

impl ScalingScheduler {
    /// Create a new scheduler with the given configuration.
    pub fn new(config: ScalingConfig) -> Self {
        let current_horizon = config.initial_horizon;
        Self {
            config,
            current_horizon,
            step_count: 0,
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(ScalingConfig::default())
    }

    /// Increment the step counter and scale the horizon if the interval
    /// has been reached. Returns the new horizon after potential scaling.
    pub fn maybe_scale(&mut self) -> usize {
        self.step_count += 1;
        if self.step_count % self.config.scaling_interval == 0 {
            self.current_horizon =
                (self.current_horizon + self.config.step_size).min(self.config.max_horizon);
        }
        self.current_horizon
    }

    /// Get the current maximum number of turns per episode.
    pub fn max_turns(&self) -> usize {
        self.current_horizon
    }

    /// Get the total number of ticks since creation.
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ScalingConfig {
        &self.config
    }

    /// Check if the horizon has reached the maximum.
    pub fn at_max(&self) -> bool {
        self.current_horizon >= self.config.max_horizon
    }

    /// Reset the scheduler to initial state.
    pub fn reset(&mut self) {
        self.current_horizon = self.config.initial_horizon;
        self.step_count = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg = ScalingConfig::default();
        assert_eq!(cfg.initial_horizon, 10);
        assert_eq!(cfg.step_size, 5);
        assert_eq!(cfg.scaling_interval, 100);
        assert_eq!(cfg.max_horizon, 200);
    }

    #[test]
    fn test_scheduler_new() {
        let sched = ScalingScheduler::new(ScalingConfig::default());
        assert_eq!(sched.max_turns(), 10);
        assert_eq!(sched.step_count(), 0);
    }

    #[test]
    fn test_scheduler_default() {
        let sched = ScalingScheduler::default();
        assert_eq!(sched.max_turns(), 10);
    }

    #[test]
    fn test_maybe_scale_no_change_before_interval() {
        let mut sched = ScalingScheduler::new(ScalingConfig {
            scaling_interval: 10,
            step_size: 5,
            ..Default::default()
        });

        for _ in 0..9 {
            let h = sched.maybe_scale();
            assert_eq!(h, 10);
        }
        assert_eq!(sched.step_count(), 9);
    }

    #[test]
    fn test_maybe_scale_increments_at_interval() {
        let mut sched = ScalingScheduler::new(ScalingConfig {
            initial_horizon: 10,
            step_size: 5,
            scaling_interval: 5,
            max_horizon: 50,
        });

        // Tick 5 times to trigger scaling
        for _ in 0..5 {
            sched.maybe_scale();
        }
        assert_eq!(sched.max_turns(), 15);
        assert_eq!(sched.step_count(), 5);
    }

    #[test]
    fn test_maybe_scale_respects_max() {
        let mut sched = ScalingScheduler::new(ScalingConfig {
            initial_horizon: 90,
            step_size: 20,
            scaling_interval: 1,
            max_horizon: 100,
        });

        // After 1 tick: 90 + 20 = 110 → clamped to 100
        sched.maybe_scale();
        assert_eq!(sched.max_turns(), 100);
    }

    #[test]
    fn test_at_max() {
        let mut sched = ScalingScheduler::new(ScalingConfig {
            initial_horizon: 10,
            step_size: 100,
            scaling_interval: 1,
            max_horizon: 10,
        });
        assert!(sched.at_max());

        let mut sched2 = ScalingScheduler::new(ScalingConfig {
            initial_horizon: 5,
            step_size: 5,
            scaling_interval: 1,
            max_horizon: 20,
        });
        assert!(!sched2.at_max());
        sched2.maybe_scale();
        assert!(sched2.at_max());
    }

    #[test]
    fn test_reset() {
        let mut sched = ScalingScheduler::new(ScalingConfig {
            initial_horizon: 10,
            step_size: 5,
            scaling_interval: 2,
            max_horizon: 50,
        });

        // Scale a few times
        for _ in 0..6 {
            sched.maybe_scale();
        }
        assert!(sched.max_turns() > 10);

        sched.reset();
        assert_eq!(sched.max_turns(), 10);
        assert_eq!(sched.step_count(), 0);
    }

    #[test]
    fn test_multiple_scaling_intervals() {
        let mut sched = ScalingScheduler::new(ScalingConfig {
            initial_horizon: 10,
            step_size: 5,
            scaling_interval: 3,
            max_horizon: 30,
        });

        // Interval 1: tick 3
        for _ in 0..3 {
            sched.maybe_scale();
        }
        assert_eq!(sched.max_turns(), 15);

        // Interval 2: tick 6
        for _ in 0..3 {
            sched.maybe_scale();
        }
        assert_eq!(sched.max_turns(), 20);

        // Interval 3: tick 9
        for _ in 0..3 {
            sched.maybe_scale();
        }
        assert_eq!(sched.max_turns(), 25);

        // Interval 4: tick 12 → would be 30, at max
        for _ in 0..3 {
            sched.maybe_scale();
        }
        assert_eq!(sched.max_turns(), 30);
    }

    #[test]
    fn test_config_accessor() {
        let sched = ScalingScheduler::new(ScalingConfig::default());
        assert_eq!(sched.config().initial_horizon, 10);
    }

    #[test]
    fn test_serialization() {
        let sched = ScalingScheduler::new(ScalingConfig::default());
        let json = serde_json::to_string(&sched.config()).unwrap();
        let de: ScalingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.initial_horizon, sched.config().initial_horizon);
        assert_eq!(de.max_horizon, sched.config().max_horizon);
    }
}
