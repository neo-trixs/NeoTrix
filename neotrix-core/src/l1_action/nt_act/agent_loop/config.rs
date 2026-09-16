//! Loop configuration for the unified agent loop.
//!
//! flash_mode: quick 3-5s/step vs orchestrated 15-40s/step
//! adaptation_strategy: how the loop adapts based on feedback

use serde::{Deserialize, Serialize};

/// Flash mode determines the step duration and execution style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlashMode {
    /// Quick 3-5s/step — fast iteration, minimal verification.
    Quick,
    /// Orchestrated 15-40s/step — thorough planning, deep verification.
    Orchestrated,
}

/// Adaptation strategies for the loop when receiving feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdaptationStrategy {
    /// Retry with same plan, adjust parameters.
    Retry,
    /// Reformulate plan from scratch.
    Reformulate,
    /// Escalate to human oversight.
    Escalate,
    /// Reduce scope and continue.
    ReduceScope,
}

/// Configuration for the unified agent loop.
#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// Flash mode: Quick (3-5s/step) or Orchestrated (15-40s/step).
    pub flash_mode: FlashMode,
    /// Maximum number of loop iterations before forced termination.
    pub max_iterations: usize,
    /// Depth of verification checks (0 = none, 1 = basic, 2 = deep, 3 = exhaustive).
    pub verification_depth: u8,
    /// Strategy for adapting when feedback indicates failure.
    pub adaptation_strategy: AdaptationStrategy,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            flash_mode: FlashMode::Orchestrated,
            max_iterations: 20,
            verification_depth: 2,
            adaptation_strategy: AdaptationStrategy::Reformulate,
        }
    }
}

impl LoopConfig {
    /// Create a quick-flash configuration.
    pub fn quick() -> Self {
        Self {
            flash_mode: FlashMode::Quick,
            max_iterations: 50,
            verification_depth: 1,
            adaptation_strategy: AdaptationStrategy::Retry,
        }
    }

    /// Create an orchestrated configuration.
    pub fn orchestrated() -> Self {
        Self::default()
    }

    /// Step duration in seconds based on flash mode.
    pub fn step_duration_secs(&self) -> u64 {
        match self.flash_mode {
            FlashMode::Quick => 5,
            FlashMode::Orchestrated => 40,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = LoopConfig::default();
        assert_eq!(cfg.flash_mode, FlashMode::Orchestrated);
        assert_eq!(cfg.max_iterations, 20);
        assert_eq!(cfg.verification_depth, 2);
        assert_eq!(cfg.adaptation_strategy, AdaptationStrategy::Reformulate);
    }

    #[test]
    fn test_quick_config() {
        let cfg = LoopConfig::quick();
        assert_eq!(cfg.flash_mode, FlashMode::Quick);
        assert_eq!(cfg.step_duration_secs(), 5);
    }

    #[test]
    fn test_orchestrated_config() {
        let cfg = LoopConfig::orchestrated();
        assert_eq!(cfg.step_duration_secs(), 40);
    }
}