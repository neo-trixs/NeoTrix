use std::collections::HashMap;

/// AdmissionPolicy — configurable thresholds and toggles for write admission.
#[derive(Debug, Clone)]
pub struct AdmissionPolicy {
    pub enabled: bool,
    pub min_score_threshold: f32,
    pub per_type_thresholds: HashMap<String, f32>,
    pub flag_for_review_above: f32,
    pub max_entries_per_cycle: usize,
}

impl Default for AdmissionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            min_score_threshold: 0.3,
            per_type_thresholds: HashMap::new(),
            flag_for_review_above: 0.7,
            max_entries_per_cycle: 100,
        }
    }
}

impl AdmissionPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve the effective threshold for a given entry type.
    /// Per-type overrides take precedence over the global minimum.
    pub fn threshold_for(&self, entry_type: &str) -> f32 {
        self.per_type_thresholds
            .get(entry_type)
            .copied()
            .unwrap_or(self.min_score_threshold)
    }

    /// Builder: set global minimum threshold.
    pub fn with_min_score(mut self, threshold: f32) -> Self {
        self.min_score_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: set per-type threshold override.
    pub fn with_type_threshold(mut self, entry_type: impl Into<String>, threshold: f32) -> Self {
        self.per_type_thresholds
            .insert(entry_type.into(), threshold.clamp(0.0, 1.0));
        self
    }

    /// Builder: enable or disable admission control entirely.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builder: set the flag-for-review upper bound.
    pub fn with_flag_above(mut self, threshold: f32) -> Self {
        self.flag_for_review_above = threshold.clamp(0.0, 1.0);
        self
    }

    /// Builder: set max entries per cycle.
    pub fn with_max_entries_per_cycle(mut self, max: usize) -> Self {
        self.max_entries_per_cycle = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let p = AdmissionPolicy::default();
        assert!(p.enabled);
        assert!((p.min_score_threshold - 0.3).abs() < 1e-6);
        assert!(p.per_type_thresholds.is_empty());
    }

    #[test]
    fn test_threshold_for_unknown_type() {
        let p = AdmissionPolicy::default();
        assert!((p.threshold_for("observation") - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_threshold_for_known_type() {
        let p = AdmissionPolicy::new()
            .with_type_threshold("critical", 0.9);
        assert!((p.threshold_for("critical") - 0.9).abs() < 1e-6);
        assert!((p.threshold_for("other") - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_builder_chaining() {
        let p = AdmissionPolicy::new()
            .with_min_score(0.5)
            .with_enabled(false)
            .with_flag_above(0.8)
            .with_max_entries_per_cycle(50);
        assert!(!p.enabled);
        assert!((p.min_score_threshold - 0.5).abs() < 1e-6);
        assert!((p.flag_for_review_above - 0.8).abs() < 1e-6);
        assert_eq!(p.max_entries_per_cycle, 50);
    }

    #[test]
    fn test_multiple_type_thresholds() {
        let p = AdmissionPolicy::new()
            .with_type_threshold("critical", 0.9)
            .with_type_threshold("low", 0.1);
        assert!((p.threshold_for("critical") - 0.9).abs() < 1e-6);
        assert!((p.threshold_for("low") - 0.1).abs() < 1e-6);
        assert!((p.threshold_for("other") - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_threshold_clamped_to_unit() {
        let p = AdmissionPolicy::new().with_min_score(5.0);
        assert!((p.min_score_threshold - 1.0).abs() < 1e-6);
        let p = AdmissionPolicy::new().with_min_score(-1.0);
        assert!((p.min_score_threshold - 0.0).abs() < 1e-6);
    }
}
