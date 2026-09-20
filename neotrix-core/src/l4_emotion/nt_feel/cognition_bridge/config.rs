use serde::{Deserialize, Serialize};

/// Configuration for emotion-cognition coupling behavior.
///
/// Controls decay rates, coupling strength, and intensity bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingConfig {
    /// How fast emotions decay toward baseline per tick (0.0–1.0).
    pub emotion_decay_rate: f32,
    /// Strength of emotional influence on cognition (0.0–1.0).
    pub coupling_strength: f32,
    /// Maximum allowed emotion intensity.
    pub max_intensity: f32,
    /// Baseline intensity that emotions decay toward.
    pub baseline_intensity: f32,
    /// Minimum intensity threshold below which emotion is treated as neutral.
    pub neutral_threshold: f32,
}

impl Default for CouplingConfig {
    fn default() -> Self {
        Self {
            emotion_decay_rate: 0.05,
            coupling_strength: 0.7,
            max_intensity: 1.0,
            baseline_intensity: 0.3,
            neutral_threshold: 0.1,
        }
    }
}

impl CouplingConfig {
    pub fn validate(&self) -> bool {
        self.emotion_decay_rate >= 0.0
            && self.emotion_decay_rate <= 1.0
            && self.coupling_strength >= 0.0
            && self.coupling_strength <= 1.0
            && self.max_intensity > 0.0
            && self.baseline_intensity >= 0.0
            && self.neutral_threshold >= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let cfg = CouplingConfig::default();
        assert!(cfg.validate());
    }

    #[test]
    fn test_custom_config() {
        let cfg = CouplingConfig {
            emotion_decay_rate: 0.1,
            coupling_strength: 0.5,
            max_intensity: 2.0,
            baseline_intensity: 0.2,
            neutral_threshold: 0.05,
        };
        assert!(cfg.validate());
    }

    #[test]
    fn test_invalid_config() {
        let cfg = CouplingConfig {
            emotion_decay_rate: 1.5, // out of range
            ..Default::default()
        };
        assert!(!cfg.validate());
    }
}
