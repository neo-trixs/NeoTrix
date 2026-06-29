#[derive(Debug, Clone)]
pub struct CalibrationRecord {
    pub prediction: f64,
    pub actual: f64,
    pub confidence: f64,
    pub timestamp: u64,
}

pub struct ConfidenceCalibrator {
    history: Vec<CalibrationRecord>,
    max_history: usize,
    calibration_bias: f64,
    miscalibration_count: usize,
}

impl ConfidenceCalibrator {
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(256),
            max_history: 256,
            calibration_bias: 0.0,
            miscalibration_count: 0,
        }
    }

    pub fn record(&mut self, prediction: f64, actual: f64, confidence: f64, timestamp: u64) {
        let miscalibrated = (prediction - actual).abs() > (1.0 - confidence) * 0.5;
        if miscalibrated {
            self.miscalibration_count += 1;
        }
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(CalibrationRecord {
            prediction,
            actual,
            confidence,
            timestamp,
        });
        self.recalibrate();
    }

    pub fn calibrate(&self, raw_confidence: f64) -> f64 {
        (raw_confidence - self.calibration_bias).clamp(0.0, 1.0)
    }

    pub fn calibration_bias(&self) -> f64 {
        self.calibration_bias
    }

    pub fn miscalibration_rate(&self) -> f64 {
        let total = self.history.len();
        if total == 0 {
            return 0.0;
        }
        self.miscalibration_count as f64 / total as f64
    }

    fn recalibrate(&mut self) {
        if self.history.len() < 10 {
            return;
        }
        let recent: Vec<_> = self.history.iter().rev().take(50).collect();
        let mut sum_error = 0.0;
        for r in &recent {
            sum_error += r.prediction - r.actual;
        }
        self.calibration_bias = sum_error / recent.len() as f64;
    }

    pub fn epistemic_uncertainty(&self) -> f64 {
        if self.history.len() < 5 {
            return 1.0;
        }
        let recent: Vec<_> = self.history.iter().rev().take(20).collect();
        let mean: f64 = recent.iter().map(|r| r.actual).sum::<f64>() / recent.len() as f64;
        let variance: f64 = recent.iter().map(|r| (r.actual - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        (variance / (variance + 1.0)).clamp(0.0, 1.0)
    }
}

impl Default for ConfidenceCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrator_initial_state() {
        let c = ConfidenceCalibrator::new();
        assert!((c.calibration_bias() - 0.0).abs() < 1e-6);
        assert!((c.miscalibration_rate() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_calibrator_records_and_adjusts() {
        let mut c = ConfidenceCalibrator::new();
        for i in 0..20 {
            c.record(0.9, 0.7, 0.95, i as u64);
        }
        assert!(c.calibration_bias().abs() > 0.01);
        assert!(c.epistemic_uncertainty() > 0.0);
    }

    #[test]
    fn test_calibrate_clamps() {
        let c = ConfidenceCalibrator::new();
        assert!((c.calibrate(1.2) - 1.0).abs() < 1e-6);
        assert!((c.calibrate(-0.5) - 0.0).abs() < 1e-6);
    }
}
