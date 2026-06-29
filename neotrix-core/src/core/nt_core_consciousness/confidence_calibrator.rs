/// A single bin in the reliability diagram.
/// Tracks predictions whose confidence falls in [min, max).
#[derive(Debug, Clone)]
pub struct CalibrationBin {
    pub min: f64,
    pub max: f64,
    pub count: u64,
    pub correct_count: u64,
}

impl CalibrationBin {
    fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            count: 0,
            correct_count: 0,
        }
    }

    fn accuracy(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.correct_count as f64 / self.count as f64
        }
    }

    fn avg_confidence(&self) -> f64 {
        (self.min + self.max) / 2.0
    }

    fn gap(&self) -> f64 {
        (self.avg_confidence() - self.accuracy()).abs()
    }
}

/// ConfidenceCalibrator: tracks prediction confidence vs. actual outcomes to calibrate
/// confidence estimates via bin-based Platt-like scaling.
///
/// Provides MetaAccuracy KPI: |self_predicted - actual_performance|.
#[derive(Debug, Clone)]
pub struct ConfidenceCalibrator {
    bins: Vec<CalibrationBin>,
    total_predictions: u64,
    total_correct: u64,
    running_ece: f64, // Exponential moving average of ECE
    running_mae: f64, // Mean Absolute Error of confidence predictions
    alpha: f64,       // EMA smoothing factor (default 0.1)
}

impl Default for ConfidenceCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceCalibrator {
    /// 10 bins: [0, 0.1), [0.1, 0.2), ..., [0.9, 1.0]
    pub fn new() -> Self {
        let bins = (0..10)
            .map(|i| CalibrationBin::new(i as f64 / 10.0, (i + 1) as f64 / 10.0))
            .collect();
        Self {
            bins,
            total_predictions: 0,
            total_correct: 0,
            running_ece: 0.0,
            running_mae: 0.0,
            alpha: 0.1,
        }
    }

    /// Record a prediction: what confidence was predicted, and was the outcome correct?
    pub fn record_prediction(&mut self, predicted_confidence: f64, correct: bool) {
        let clamped = predicted_confidence.clamp(0.0, 0.999);
        let bin_idx = (clamped * 10.0) as usize;
        let idx = bin_idx.min(9);

        self.bins[idx].count += 1;
        if correct {
            self.bins[idx].correct_count += 1;
        }
        self.total_predictions += 1;
        if correct {
            self.total_correct += 1;
        }

        // Update ECE (Expected Calibration Error) as EMA
        let ece = self.compute_ece();
        if self.total_predictions == 1 {
            self.running_ece = ece;
        } else {
            self.running_ece = (1.0 - self.alpha) * self.running_ece + self.alpha * ece;
        }

        // Update MAE = |predicted - actual|
        let actual = if correct { 1.0 } else { 0.0 };
        let mae_sample = (clamped - actual).abs();
        if self.total_predictions == 1 {
            self.running_mae = mae_sample;
        } else {
            self.running_mae = (1.0 - self.alpha) * self.running_mae + self.alpha * mae_sample;
        }
    }

    /// Calibrate a raw confidence using bin-based correction.
    /// If a bin has < 5 observations, returns raw confidence unchanged.
    pub fn calibrate(&self, raw_confidence: f64) -> f64 {
        let clamped = raw_confidence.clamp(0.0, 0.999);
        let bin_idx = (clamped * 10.0) as usize;
        let idx = bin_idx.min(9);

        let bin = &self.bins[idx];
        if bin.count < 5 {
            // Not enough data: Platt-scale-inspired shrinkage toward prior
            let prior = self.overall_accuracy();
            let n = bin.count as f64;
            let lambda = n / (n + 5.0); // Shrinkage factor
            let empirical = bin.accuracy();
            return lambda * empirical + (1.0 - lambda) * prior;
        }

        bin.accuracy()
    }

    /// Monte Carlo calibrated draw: returns a calibrated confidence with noise proportional
    /// to calibration uncertainty.
    pub fn calibrated_with_uncertainty(&self, raw_confidence: f64) -> (f64, f64) {
        let calibrated = self.calibrate(raw_confidence);
        let uncertainty = self.ece(); // Use ECE as uncertainty estimate
        (calibrated, uncertainty)
    }

    /// Expected Calibration Error: Σ (|accuracy(bin) - avg_confidence(bin)| * count(bin) / total)
    pub fn compute_ece(&self) -> f64 {
        if self.total_predictions == 0 {
            return 0.0;
        }
        let total = self.total_predictions as f64;
        self.bins
            .iter()
            .map(|b| b.gap() * (b.count as f64 / total))
            .sum()
    }

    /// Running EMA of ECE
    pub fn ece(&self) -> f64 {
        if self.total_predictions < 5 {
            0.0
        } else {
            self.running_ece
        }
    }

    /// MetaAccuracy KPI: 1.0 - MAE (higher = better calibration)
    pub fn meta_accuracy(&self) -> f64 {
        if self.total_predictions < 5 {
            1.0
        } else {
            (1.0 - self.running_mae).clamp(0.0, 1.0)
        }
    }

    /// Overall accuracy (correct / total)
    pub fn overall_accuracy(&self) -> f64 {
        if self.total_predictions == 0 {
            0.5
        } else {
            self.total_correct as f64 / self.total_predictions as f64
        }
    }

    /// Total number of predictions recorded
    pub fn total_predictions(&self) -> u64 {
        self.total_predictions
    }

    /// Reset the calibrator (clear all bins)
    pub fn reset(&mut self) {
        for bin in &mut self.bins {
            bin.count = 0;
            bin.correct_count = 0;
        }
        self.total_predictions = 0;
        self.total_correct = 0;
        self.running_ece = 0.0;
        self.running_mae = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_calibrator_has_zero_ece() {
        let c = ConfidenceCalibrator::new();
        assert!((c.compute_ece() - 0.0).abs() < f64::EPSILON);
        assert!((c.meta_accuracy() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_perfect_predictions_give_high_accuracy() {
        let mut c = ConfidenceCalibrator::new();
        for _ in 0..100 {
            c.record_prediction(0.9, true);
        }
        assert!(c.meta_accuracy() > 0.8);
        assert!(c.ece() < 0.2);
    }

    #[test]
    fn test_bad_predictions_increase_ece() {
        let mut c = ConfidenceCalibrator::new();
        for _ in 0..100 {
            c.record_prediction(0.9, false);
        }
        assert!(c.ece() > 0.3);
    }

    #[test]
    fn test_calibrate_returns_bin_accuracy() {
        let mut c = ConfidenceCalibrator::new();
        // Fill bin [0.8, 0.9) with 80% accuracy
        for i in 0..10 {
            c.record_prediction(0.85, i < 8);
        }
        let calibrated = c.calibrate(0.85);
        assert!((calibrated - 0.8).abs() < 0.1);
    }

    #[test]
    fn test_sparse_bin_uses_shrinkage() {
        let mut c = ConfidenceCalibrator::new();
        // Fill some other bins
        for _ in 0..20 {
            c.record_prediction(0.5, true);
            c.record_prediction(0.5, false);
        }
        // Bin [0.0, 0.1) has 0 observations -> uses prior (0.5)
        let calibrated = c.calibrate(0.05);
        assert!((calibrated - 0.5).abs() < 0.2);
    }

    #[test]
    fn test_mixed_predictions() {
        let mut c = ConfidenceCalibrator::new();
        for _ in 0..50 {
            c.record_prediction(0.7, true);
            c.record_prediction(0.3, false);
        }
        assert!(c.total_predictions() == 100);
        assert!((c.overall_accuracy() - 0.5).abs() < 0.15);
    }

    #[test]
    fn test_calibrated_with_uncertainty() {
        let mut c = ConfidenceCalibrator::new();
        for _ in 0..30 {
            c.record_prediction(0.8, true);
        }
        let (cal, unc) = c.calibrated_with_uncertainty(0.8);
        assert!(cal >= 0.0 && cal <= 1.0);
        assert!(unc >= 0.0 && unc <= 1.0);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut c = ConfidenceCalibrator::new();
        for _ in 0..50 {
            c.record_prediction(0.9, true);
        }
        assert!(c.total_predictions() > 0);
        c.reset();
        assert_eq!(c.total_predictions(), 0);
        assert!((c.ece() - 0.0).abs() < f64::EPSILON);
    }
}
