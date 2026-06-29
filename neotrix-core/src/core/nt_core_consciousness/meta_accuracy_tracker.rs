use std::collections::VecDeque;

const DEFAULT_WINDOW: usize = 100;

/// Real MetaAccuracy tracker — replaces synthetic `tech_debt / 1000`.
///
/// Records real calibration data (ECE, meta-d') from CalibrationEngine,
/// computes actual meta-accuracy from a sliding window of prediction-outcome pairs.
#[derive(Debug, Clone)]
pub struct MetaAccuracyTracker {
    window: VecDeque<CalibrationRecord>,
    max_window: usize,
    /// Cached actual meta-accuracy (0.0 = none, 0.5 = random, 1.0 = perfect)
    actual_meta_accuracy: f64,
    /// Theoretical upper bound for meta-accuracy (Yoshizawa & Mogi AAAI 2026).
    /// 0.85 = human upper bound, 0.70 = LLM upper bound (estimated)
    pub theoretical_upper_bound: f64,
}

#[derive(Debug, Clone)]
struct CalibrationRecord {
    meta_d: f64,
    ece: f64,
    count: usize,
}

impl MetaAccuracyTracker {
    pub fn new() -> Self {
        Self {
            window: VecDeque::with_capacity(DEFAULT_WINDOW),
            max_window: DEFAULT_WINDOW,
            actual_meta_accuracy: 0.5,
            theoretical_upper_bound: 0.85,
        }
    }

    pub fn with_max_window(max: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max),
            max_window: max,
            actual_meta_accuracy: 0.5,
            theoretical_upper_bound: 0.85,
        }
    }

    /// Record a batch of calibration data from CalibrationEngine.
    ///
    /// - `meta_d`: meta-d' (type-2 sensitivity), higher = better self-knowledge
    /// - `ece`: Expected Calibration Error, lower = better calibrated
    /// - `count`: number of prediction-outcome pairs in this batch
    pub fn record_calibration_data(&mut self, meta_d: f64, ece: f64, count: usize) {
        if self.window.len() >= self.max_window {
            self.window.pop_front();
        }
        self.window
            .push_back(CalibrationRecord { meta_d, ece, count });
        self.recompute();
    }

    /// Current actual meta-accuracy.
    ///
    /// Returns 0.5 (neutral / random baseline) when insufficient data.
    pub fn actual_meta_accuracy(&self) -> f64 {
        self.actual_meta_accuracy
    }

    fn recompute(&mut self) {
        let n = self.window.len();
        if n < 3 {
            self.actual_meta_accuracy = 0.5;
            return;
        }

        let total_count: usize = self.window.iter().map(|r| r.count).sum();
        if total_count == 0 {
            self.actual_meta_accuracy = 0.5;
            return;
        }

        let avg_meta_d: f64 = self
            .window
            .iter()
            .map(|r| r.meta_d * r.count as f64)
            .sum::<f64>()
            / total_count as f64;
        let avg_ece: f64 = self
            .window
            .iter()
            .map(|r| r.ece * r.count as f64)
            .sum::<f64>()
            / total_count as f64;

        // meta-accuracy = weighted combination of meta-d' (self-knowledge) and (1 - ECE) (calibration)
        // Both normalized to [0, 1] range, meta-d' typically in [0, 3+] range so we clamp
        let meta_d_norm = (avg_meta_d / 3.0).clamp(0.0, 1.0);
        let calib_norm = (1.0 - avg_ece).clamp(0.0, 1.0);

        // Weighted: self-knowledge (0.6) + calibration (0.4)
        self.actual_meta_accuracy = 0.6 * meta_d_norm + 0.4 * calib_norm;
    }

    /// Alias for `actual_meta_accuracy()` — used by consciousness_cycle.
    pub fn current_accuracy(&self) -> f64 {
        self.actual_meta_accuracy
    }

    /// Set the theoretical upper bound for meta-accuracy.
    /// 0.85 = human upper bound, 0.70 = LLM upper bound (estimated).
    pub fn set_theoretical_upper_bound(&mut self, bound: f64) {
        self.theoretical_upper_bound = bound.clamp(0.5, 1.0);
    }

    pub fn reset(&mut self) {
        self.window.clear();
        self.actual_meta_accuracy = 0.5;
    }

    pub fn stats(&self) -> MetaAccuracyStats {
        let n = self.window.len();
        let total_count: usize = self.window.iter().map(|r| r.count).sum();
        if n == 0 {
            return MetaAccuracyStats {
                sample_count: 0,
                total_pairs: 0,
                actual_meta_accuracy: 0.5,
                avg_meta_d: 0.0,
                avg_ece: 0.0,
            };
        }
        let avg_meta_d = self.window.iter().map(|r| r.meta_d).sum::<f64>() / n as f64;
        let avg_ece = self.window.iter().map(|r| r.ece).sum::<f64>() / n as f64;
        MetaAccuracyStats {
            sample_count: n,
            total_pairs: total_count,
            actual_meta_accuracy: self.actual_meta_accuracy,
            avg_meta_d,
            avg_ece,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaAccuracyStats {
    pub sample_count: usize,
    pub total_pairs: usize,
    pub actual_meta_accuracy: f64,
    pub avg_meta_d: f64,
    pub avg_ece: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        let tracker = MetaAccuracyTracker::new();
        assert!((tracker.actual_meta_accuracy() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn insufficient_data_returns_neutral() {
        let mut tracker = MetaAccuracyTracker::new();
        tracker.record_calibration_data(1.5, 0.1, 10);
        assert!((tracker.actual_meta_accuracy() - 0.5).abs() < 1e-10);
        tracker.record_calibration_data(1.8, 0.08, 15);
        assert!((tracker.actual_meta_accuracy() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn good_calibration_high_accuracy() {
        let mut tracker = MetaAccuracyTracker::new();
        for _ in 0..5 {
            tracker.record_calibration_data(2.5, 0.05, 20);
        }
        let acc = tracker.actual_meta_accuracy();
        assert!(
            acc > 0.5,
            "good calibration should give high accuracy, got {}",
            acc
        );
    }

    #[test]
    fn poor_calibration_low_accuracy() {
        let mut tracker = MetaAccuracyTracker::new();
        for _ in 0..5 {
            tracker.record_calibration_data(0.3, 0.6, 20);
        }
        let acc = tracker.actual_meta_accuracy();
        assert!(
            acc < 0.6,
            "poor calibration should give low accuracy, got {}",
            acc
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut tracker = MetaAccuracyTracker::new();
        for _ in 0..5 {
            tracker.record_calibration_data(2.0, 0.1, 10);
        }
        tracker.reset();
        assert!((tracker.actual_meta_accuracy() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn stats_returns_summary() {
        let mut tracker = MetaAccuracyTracker::new();
        for _ in 0..5 {
            tracker.record_calibration_data(2.0, 0.1, 10);
        }
        let stats = tracker.stats();
        assert_eq!(stats.sample_count, 5);
        assert_eq!(stats.total_pairs, 50);
        assert!(stats.actual_meta_accuracy > 0.5);
    }
}
