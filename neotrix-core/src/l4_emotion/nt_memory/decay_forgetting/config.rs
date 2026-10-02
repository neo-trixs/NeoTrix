#![forbid(unsafe_code)]

//! Decay configuration (R-P121).
//!
//! Centralizes all tuning knobs for the decay-forgetting pipeline:
//! curve selection, half-life, salience threshold, max entries, and
//! enable/disable toggle.

use serde::{Deserialize, Serialize};

// 2026-09-30：`DecayCurve` 原被导入但本文件**未使用**（只调固有方法
// `ExponentialDecay::from_half_life`）⇒ 移除，避免 unused 警告。
use super::curves::ExponentialDecay;
// 2026-09-30：`PruneMode` 在本文件未使用 ⇒ 移除（类型本身已在 pruner 中补齐）。
use super::pruner::MemoryPruner;
use super::salience::SalienceCalculator;

/// Configuration for the decay-based forgetting system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    /// Half-life in days for the exponential decay curve.
    pub half_life_days: f64,
    /// Minimum salience threshold; entries below this are pruned.
    pub min_salience_threshold: f64,
    /// Hard cap on total entries. `None` = unlimited.
    pub max_entries: Option<usize>,
    /// Whether pruning is enabled.
    pub enable_pruning: bool,
    /// Weight breakdown for salience computation.
    #[serde(default)]
    pub weights: SalienceWeightsDef,
    /// Normalization cap for access count in salience scoring.
    #[serde(default = "default_max_access_count")]
    pub max_access_count: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalienceWeightsDef {
    pub access_freq: f64,
    pub recency: f64,
    pub importance: f64,
}

fn default_max_access_count() -> f64 {
    100.0
}

impl Default for SalienceWeightsDef {
    fn default() -> Self {
        Self {
            access_freq: 0.3,
            recency: 0.4,
            importance: 0.3,
        }
    }
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            half_life_days: 30.0,
            min_salience_threshold: 0.1,
            max_entries: None,
            enable_pruning: true,
            weights: SalienceWeightsDef::default(),
            max_access_count: default_max_access_count(),
        }
    }
}

impl DecayConfig {
    /// Build a `SalienceCalculator` from this config.
    pub fn build_calculator(&self) -> SalienceCalculator {
        let decay = ExponentialDecay::from_half_life(self.half_life_days);
        let weights = super::salience::SalienceWeights {
            access_freq: self.weights.access_freq,
            recency: self.weights.recency,
            importance: self.weights.importance,
        };
        SalienceCalculator::new(decay, weights).with_max_access_count(self.max_access_count)
    }

    /// Build a `MemoryPruner` from this config.
    pub fn build_pruner(&self) -> MemoryPruner {
        let calc = self.build_calculator();
        let mut pruner = MemoryPruner::new(calc, self.min_salience_threshold);
        if let Some(max) = self.max_entries {
            pruner = pruner.with_max_entries(max);
        }
        pruner
    }
}

/// A compact preset library for common decay profiles.
impl DecayConfig {
    /// Aggressive: short half-life, low threshold. Good for ephemeral caches.
    pub fn aggressive() -> Self {
        Self {
            half_life_days: 7.0,
            min_salience_threshold: 0.2,
            max_entries: Some(1000),
            enable_pruning: true,
            ..Default::default()
        }
    }

    /// Conservative: long half-life, low threshold. Good for long-term knowledge.
    pub fn conservative() -> Self {
        Self {
            half_life_days: 180.0,
            min_salience_threshold: 0.05,
            max_entries: None,
            enable_pruning: true,
            ..Default::default()
        }
    }

    /// Balanced: 30-day half-life, moderate settings.
    pub fn balanced() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = DecayConfig::default();
        assert!(cfg.half_life_days > 0.0);
        assert!((0.0..=1.0).contains(&cfg.min_salience_threshold));
        assert!(cfg.enable_pruning);
    }

    #[test]
    fn build_calculator_uses_config_half_life() {
        let cfg = DecayConfig {
            half_life_days: 60.0,
            ..Default::default()
        };
        let calc = cfg.build_calculator();
        let hl = calc.decay.half_life();
        assert!((hl - 60.0).abs() < 0.01);
    }

    #[test]
    fn build_pruner_respects_max_entries() {
        let cfg = DecayConfig {
            max_entries: Some(42),
            ..Default::default()
        };
        let pruner = cfg.build_pruner();
        assert_eq!(pruner.max_entries, Some(42));
    }

    #[test]
    fn aggressive_preset() {
        let cfg = DecayConfig::aggressive();
        assert_eq!(cfg.half_life_days, 7.0);
        assert_eq!(cfg.max_entries, Some(1000));
    }

    #[test]
    fn conservative_preset() {
        let cfg = DecayConfig::conservative();
        assert_eq!(cfg.half_life_days, 180.0);
        assert!(cfg.max_entries.is_none());
    }

    #[test]
    fn serialization_roundtrip() {
        let cfg = DecayConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: DecayConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.half_life_days, cfg.half_life_days);
        assert_eq!(restored.enable_pruning, cfg.enable_pruning);
    }

    #[test]
    fn balanced_preset_is_default() {
        let cfg = DecayConfig::balanced();
        let default = DecayConfig::default();
        assert_eq!(cfg.half_life_days, default.half_life_days);
    }

    #[test]
    fn build_pruner_no_max_entries() {
        let cfg = DecayConfig { max_entries: None, ..Default::default() };
        let pruner = cfg.build_pruner();
        assert!(pruner.max_entries.is_none());
    }
}
