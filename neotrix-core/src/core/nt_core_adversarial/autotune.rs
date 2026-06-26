use log;

#[derive(Debug, Clone)]
pub struct AutoTuner {
    pub escape_rate_target: f64,
    pub sensitivity_min: f64,
    pub sensitivity_max: f64,
    pub adjustment_rate: f64,
    pub adjustments_made: u64,
}

impl AutoTuner {
    pub fn new() -> Self {
        Self {
            escape_rate_target: 0.05,
            sensitivity_min: 0.1,
            sensitivity_max: 0.95,
            adjustment_rate: 0.1,
            adjustments_made: 0,
        }
    }

    pub fn tune(&mut self, current_escape_rate: f64, sensitivities: &mut [f64]) {
        for s in sensitivities.iter_mut() {
            let old = *s;
            if current_escape_rate > self.escape_rate_target {
                *s = (*s + self.adjustment_rate).min(self.sensitivity_max);
            } else {
                *s = (*s - self.adjustment_rate).max(self.sensitivity_min);
            }
            if (*s - old).abs() > f64::EPSILON {
                self.adjustments_made += 1;
            }
        }
        log::info!(
            "ADVERSARIAL: AutoTuner escape_rate={:.4} target={:.2} adjustments_made={} sensitivities={:.2?}",
            current_escape_rate,
            self.escape_rate_target,
            self.adjustments_made,
            sensitivities,
        );
    }

    pub fn tune_with_labels(
        &mut self,
        current_escape_rate: f64,
        sensitivities: &mut [(&str, f64)],
    ) {
        for (label, s) in sensitivities.iter_mut() {
            let old = *s;
            if current_escape_rate > self.escape_rate_target {
                *s = (*s + self.adjustment_rate).min(self.sensitivity_max);
            } else {
                *s = (*s - self.adjustment_rate).max(self.sensitivity_min);
            }
            if (*s - old).abs() > f64::EPSILON {
                self.adjustments_made += 1;
                log::info!(
                    "ADVERSARIAL: AutoTuner adjusted '{}' {:.4} -> {:.4}",
                    label,
                    old,
                    *s,
                );
            }
        }
    }
}

impl Default for AutoTuner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tune_increases_when_above_target() {
        let mut tuner = AutoTuner::new();
        let mut sensitivities = vec![0.5, 0.5, 0.5];
        tuner.tune(0.10, &mut sensitivities);
        for s in &sensitivities {
            assert!(
                *s > 0.5,
                "sensitivity should increase when escape_rate > target"
            );
        }
    }

    #[test]
    fn test_tune_decreases_when_below_target() {
        let mut tuner = AutoTuner::new();
        let mut sensitivities = vec![0.5, 0.5, 0.5];
        tuner.tune(0.01, &mut sensitivities);
        for s in &sensitivities {
            assert!(
                *s < 0.5,
                "sensitivity should decrease when escape_rate < target"
            );
        }
    }

    #[test]
    fn test_tune_clamps_to_bounds() {
        let mut tuner = AutoTuner::new();
        let mut sensitivities = vec![0.01];
        tuner.tune(0.01, &mut sensitivities);
        assert!(sensitivities[0] >= 0.1);

        let mut sensitivities = vec![0.99];
        tuner.tune(0.10, &mut sensitivities);
        assert!(sensitivities[0] <= 0.95);
    }

    #[test]
    fn test_adjustments_counted() {
        let mut tuner = AutoTuner::new();
        let mut sensitivities = vec![0.5];
        tuner.tune(0.10, &mut sensitivities);
        assert_eq!(tuner.adjustments_made, 1);
    }

    #[test]
    fn test_no_adjustment_when_no_change() {
        let mut tuner = AutoTuner::new();
        let mut sensitivities = vec![0.1];
        // Already at min — decreasing further won't change
        tuner.tune(0.01, &mut sensitivities);
        // adjustment only counted when |new - old| > EPSILON
        // Since old = 0.1, new = max(0.1 - 0.1, 0.1) = 0.1, no change
        assert_eq!(tuner.adjustments_made, 0);
    }
}
