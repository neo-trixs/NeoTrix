use std::collections::HashMap;

use super::policy::AdmissionPolicy;
use super::scorer::{ScoreWeights, TypePriorWeights};

/// R-P11 config struct for admission control.
/// All fields have sensible defaults; use the builder to customize.
#[derive(Debug, Clone)]
pub struct AdmissionControlConfig {
    pub enabled: bool,
    pub min_score_threshold: f32,
    pub per_type_thresholds: HashMap<String, f32>,
    pub flag_for_review_above: f32,
    pub max_entries_per_cycle: usize,
    pub weights: ScoreWeights,
    pub type_priors: HashMap<String, f32>,
}

impl Default for AdmissionControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_score_threshold: 0.3,
            per_type_thresholds: HashMap::new(),
            flag_for_review_above: 0.7,
            max_entries_per_cycle: 100,
            weights: ScoreWeights::default(),
            type_priors: HashMap::new(),
        }
    }
}

impl AdmissionControlConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an AdmissionPolicy from this config.
    pub fn to_policy(&self) -> AdmissionPolicy {
        AdmissionPolicy {
            enabled: self.enabled,
            min_score_threshold: self.min_score_threshold,
            per_type_thresholds: self.per_type_thresholds.clone(),
            flag_for_review_above: self.flag_for_review_above,
            max_entries_per_cycle: self.max_entries_per_cycle,
        }
    }

    /// Build a TypePriorWeights from this config.
    pub fn to_type_priors(&self) -> TypePriorWeights {
        TypePriorWeights {
            weights: self.type_priors.clone(),
        }
    }

    /// Builder: enable or disable.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder: set global minimum threshold.
    pub fn with_min_score(mut self, threshold: f32) -> Self {
        self.min_score_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: add a per-type threshold override.
    pub fn with_type_threshold(mut self, entry_type: impl Into<String>, threshold: f32) -> Self {
        self.per_type_thresholds
            .insert(entry_type.into(), threshold.clamp(0.0, 1.0));
        self
    }

    /// Builder: set flag-for-review upper bound.
    pub fn with_flag_above(mut self, threshold: f32) -> Self {
        self.flag_for_review_above = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: set scoring weights.
    pub fn with_weights(mut self, weights: ScoreWeights) -> Self {
        self.weights = weights;
        self
    }

    /// Builder: set a type prior weight.
    pub fn with_type_prior(mut self, entry_type: impl Into<String>, weight: f32) -> Self {
        self.type_priors
            .insert(entry_type.into(), weight.clamp(0.0, 1.0));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = AdmissionControlConfig::default();
        assert!(c.enabled);
        assert!((c.min_score_threshold - 0.3).abs() < 1e-6);
        assert!((c.flag_for_review_above - 0.7).abs() < 1e-6);
        assert_eq!(c.max_entries_per_cycle, 100);
    }

    #[test]
    fn test_to_policy() {
        let c = AdmissionControlConfig::new()
            .with_min_score(0.5)
            .with_type_threshold("critical", 0.9);
        let p = c.to_policy();
        assert!((p.min_score_threshold - 0.5).abs() < 1e-6);
        assert!((p.threshold_for("critical") - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_to_type_priors() {
        let c = AdmissionControlConfig::new()
            .with_type_prior("event", 0.8);
        let tp = c.to_type_priors();
        assert!((tp.get("event") - 0.8).abs() < 1e-6);
        assert!((tp.get("unknown") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_builder_chaining() {
        let c = AdmissionControlConfig::new()
            .with_enabled(false)
            .with_flag_above(0.9)
            .with_weights(ScoreWeights {
                utility: 0.3,
                confidence: 0.3,
                novelty: 0.1,
                recency: 0.1,
                type_prior: 0.2,
            });
        assert!(!c.enabled);
        assert!((c.flag_for_review_above - 0.9).abs() < 1e-6);
        assert!((c.weights.utility - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_min_score_clamped() {
        let c = AdmissionControlConfig::new().with_min_score(2.0);
        assert!((c.min_score_threshold - 1.0).abs() < 1e-6);
        let c = AdmissionControlConfig::new().with_min_score(-1.0);
        assert!((c.min_score_threshold - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_type_threshold_clamped() {
        let c = AdmissionControlConfig::new().with_type_threshold("x", 5.0);
        assert!((c.per_type_thresholds["x"] - 1.0).abs() < 1e-6);
    }
}
