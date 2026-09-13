//! Adaptive difficulty adjuster — tracks win rate and scales difficulty.
//!
//! Monitors episode outcomes and adjusts difficulty via a rolling win rate
//! window. When the player deviates too far from the target win rate,
//! difficulty is increased or decreased proportionally.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Config
// ═══════════════════════════════════════════════════════════════════

/// Configuration for the adaptive difficulty system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveDifficultyConfig {
    /// Target win rate (0.0–1.0). Default 0.55.
    pub target_win_rate: f64,
    /// How aggressively to adjust per tick (magnitude multiplier). Default 0.05.
    pub adjustment_rate: f64,
    /// Minimum episodes before adjustment is allowed. Default 20.
    pub min_samples: usize,
}

impl Default for AdaptiveDifficultyConfig {
    fn default() -> Self {
        Self {
            target_win_rate: 0.55,
            adjustment_rate: 0.05,
            min_samples: 20,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Difficulty Adjustment
// ═══════════════════════════════════════════════════════════════════

/// The difficulty adjustment recommended by the adjuster.
#[derive(Debug, Clone, Copy, PartialSerialize, Deserialize)]
pub enum DifficultyAdjustment {
    /// Increase difficulty by the given magnitude.
    Increase(f64),
    /// No change needed.
    Maintain,
    /// Decrease difficulty by the given magnitude.
    Decrease(f64),
}

// ═══════════════════════════════════════════════════════════════════
// Adjuster
// ═══════════════════════════════════════════════════════════════════

/// Tracks win/loss outcomes and recommends difficulty changes.
pub struct DifficultyAdjuster {
    config: AdaptiveDifficultyConfig,
    /// Rolling history of episode outcomes (true = won).
    win_rate_history: Vec<bool>,
    /// Current accumulated adjustment magnitude.
    current_adjustment: f64,
}

impl DifficultyAdjuster {
    /// Create a new adjuster with the given configuration.
    pub fn new(config: AdaptiveDifficultyConfig) -> Self {
        Self {
            config,
            win_rate_history: Vec::new(),
            current_adjustment: 0.0,
        }
    }

    /// Create with default configuration.
    pub fn default_config() -> Self {
        Self::new(AdaptiveDifficultyConfig::default())
    }

    /// Record the outcome of an episode.
    pub fn record_episode(&mut self, won: bool) {
        self.win_rate_history.push(won);
    }

    /// Rolling win rate over all recorded episodes.
    pub fn current_win_rate(&self) -> f64 {
        if self.win_rate_history.is_empty() {
            return 0.0;
        }
        let wins = self.win_rate_history.iter().filter(|&&w| w).count();
        wins as f64 / self.win_rate_history.len() as f64
    }

    /// Total episodes recorded.
    pub fn episode_count(&self) -> usize {
        self.win_rate_history.len()
    }

    /// Current accumulated adjustment.
    pub fn current_adjustment(&self) -> f64 {
        self.current_adjustment
    }

    /// Returns true if enough samples exist and win rate deviates from target.
    pub fn should_adjust(&self) -> bool {
        if self.win_rate_history.len() < self.config.min_samples {
            return false;
        }
        let wr = self.current_win_rate();
        let deviation = (wr - self.config.target_win_rate).abs();
        deviation > 0.1
    }

    /// Recommended difficulty adjustment based on current win rate.
    pub fn recommended_difficulty(&self) -> DifficultyAdjustment {
        if self.win_rate_history.len() < self.config.min_samples {
            return DifficultyAdjustment::Maintain;
        }
        let wr = self.current_win_rate();
        let delta = wr - self.config.target_win_rate;

        if delta > 0.1 {
            // Winning too much → increase difficulty
            let magnitude = self.config.adjustment_rate * delta;
            DifficultyAdjustment::Increase(magnitude)
        } else if delta < -0.1 {
            // Losing too much → decrease difficulty
            let magnitude = self.config.adjustment_rate * delta.abs();
            DifficultyAdjustment::Decrease(magnitude)
        } else {
            DifficultyAdjustment::Maintain
        }
    }

    /// Apply the recommended adjustment and update internal state.
    pub fn apply_adjustment(&mut self, adjustment: DifficultyAdjustment) {
        match adjustment {
            DifficultyAdjustment::Increase(mag) => {
                self.current_adjustment += mag;
            }
            DifficultyAdjustment::Decrease(mag) => {
                self.current_adjustment -= mag;
            }
            DifficultyAdjustment::Maintain => {}
        }
    }

    /// Reset history and adjustment state.
    pub fn reset(&mut self) {
        self.win_rate_history.clear();
        self.current_adjustment = 0.0;
    }

    /// Get the configuration.
    pub fn config(&self) -> &AdaptiveDifficultyConfig {
        &self.config
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
        let cfg = AdaptiveDifficultyConfig::default();
        assert!((cfg.target_win_rate - 0.55).abs() < f64::EPSILON);
        assert!((cfg.adjustment_rate - 0.05).abs() < f64::EPSILON);
        assert_eq!(cfg.min_samples, 20);
    }

    #[test]
    fn test_new_adjuster() {
        let adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig::default());
        assert_eq!(adj.episode_count(), 0);
        assert_eq!(adj.current_adjustment(), 0.0);
    }

    #[test]
    fn test_record_episode() {
        let mut adj = DifficultyAdjuster::default_config();
        adj.record_episode(true);
        adj.record_episode(false);
        adj.record_episode(true);
        assert_eq!(adj.episode_count(), 3);
        assert!((adj.current_win_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_current_win_rate_empty() {
        let adj = DifficultyAdjuster::default_config();
        assert!((adj.current_win_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_should_adjust_insufficient_samples() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            min_samples: 10,
            ..Default::default()
        });
        // Only 5 wins out of 5 → 100% win rate, but below min_samples
        for _ in 0..5 {
            adj.record_episode(true);
        }
        assert!(!adj.should_adjust());
    }

    #[test]
    fn test_should_adjust_no_deviation() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            min_samples: 5,
            ..Default::default()
        });
        // 55% win rate → no deviation
        for i in 0..20 {
            adj.record_episode(i < 11); // 11 wins = 55%
        }
        assert!(!adj.should_adjust());
    }

    #[test]
    fn test_should_adjust_high_win_rate() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            min_samples: 5,
            ..Default::default()
        });
        // 80% win rate → should adjust
        for i in 0..20 {
            adj.record_episode(i < 16); // 16 wins = 80%
        }
        assert!(adj.should_adjust());
    }

    #[test]
    fn test_should_adjust_low_win_rate() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            min_samples: 5,
            ..Default::default()
        });
        // 20% win rate → should adjust
        for i in 0..20 {
            adj.record_episode(i < 4); // 4 wins = 20%
        }
        assert!(adj.should_adjust());
    }

    #[test]
    fn test_recommended_difficulty_maintain() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            min_samples: 5,
            ..Default::default()
        });
        for i in 0..20 {
            adj.record_episode(i < 11); // 55%
        }
        assert_eq!(adj.recommended_difficulty(), DifficultyAdjustment::Maintain);
    }

    #[test]
    fn test_recommended_difficulty_increase() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            adjustment_rate: 0.05,
            min_samples: 5,
        });
        for i in 0..20 {
            adj.record_episode(i < 18); // 90% → delta = 0.35
        }
        let rec = adj.recommended_difficulty();
        match rec {
            DifficultyAdjustment::Increase(mag) => {
                assert!(mag > 0.0);
                // 0.05 * 0.35 = 0.0175
                assert!((mag - 0.0175).abs() < 1e-10);
            }
            _ => panic!("expected Increase"),
        }
    }

    #[test]
    fn test_recommended_difficulty_decrease() {
        let mut adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            adjustment_rate: 0.05,
            min_samples: 5,
        });
        for i in 0..20 {
            adj.record_episode(i < 2); // 10% → delta = -0.45
        }
        let rec = adj.recommended_difficulty();
        match rec {
            DifficultyAdjustment::Decrease(mag) => {
                assert!(mag > 0.0);
                // 0.05 * 0.45 = 0.0225
                assert!((mag - 0.0225).abs() < 1e-10);
            }
            _ => panic!("expected Decrease"),
        }
    }

    #[test]
    fn test_recommended_difficulty_below_min_samples() {
        let adj = DifficultyAdjuster::new(AdaptiveDifficultyConfig {
            target_win_rate: 0.55,
            min_samples: 50,
            ..Default::default()
        });
        assert_eq!(adj.recommended_difficulty(), DifficultyAdjustment::Maintain);
    }

    #[test]
    fn test_apply_adjustment_increase() {
        let mut adj = DifficultyAdjuster::default_config();
        adj.apply_adjustment(DifficultyAdjustment::Increase(0.1));
        assert!((adj.current_adjustment() - 0.1).abs() < f64::EPSILON);
        adj.apply_adjustment(DifficultyAdjustment::Increase(0.05));
        assert!((adj.current_adjustment() - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_adjustment_decrease() {
        let mut adj = DifficultyAdjuster::default_config();
        adj.apply_adjustment(DifficultyAdjustment::Decrease(0.1));
        assert!((adj.current_adjustment() - (-0.1)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_adjustment_maintain() {
        let mut adj = DifficultyAdjuster::default_config();
        adj.apply_adjustment(DifficultyAdjustment::Maintain);
        assert!((adj.current_adjustment()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reset() {
        let mut adj = DifficultyAdjuster::default_config();
        for _ in 0..10 {
            adj.record_episode(true);
        }
        adj.apply_adjustment(DifficultyAdjustment::Increase(0.2));
        adj.reset();
        assert_eq!(adj.episode_count(), 0);
        assert!((adj.current_adjustment()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_serialization() {
        let cfg = AdaptiveDifficultyConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let de: AdaptiveDifficultyConfig = serde_json::from_str(&json).unwrap();
        assert!((de.target_win_rate - cfg.target_win_rate).abs() < f64::EPSILON);
        assert_eq!(de.min_samples, cfg.min_samples);
    }

    #[test]
    fn test_serialization_adjustment_enum() {
        let adj = DifficultyAdjustment::Increase(0.15);
        let json = serde_json::to_string(&adj).unwrap();
        let de: DifficultyAdjustment = serde_json::from_str(&json).unwrap();
        assert_eq!(adj, de);
    }
}
