#![forbid(unsafe_code)]

use std::fmt;

/// Severity of an anomaly detection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnomalySeverity {
    Normal,
    Warning,
    Critical,
}

impl fmt::Display for AnomalySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnomalySeverity::Normal => write!(f, "NORMAL"),
            AnomalySeverity::Warning => write!(f, "WARNING"),
            AnomalySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Result of an anomaly detection check.
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    /// Whether the data point is considered anomalous.
    pub is_anomaly: bool,
    /// Severity classification of the anomaly.
    pub severity: AnomalySeverity,
    /// Standard deviations from the moving average (z-score).
    pub deviation: f64,
}

/// Detects anomalies in time-series data using z-score and moving average.
pub struct AnomalyDetector {
    /// Number of historical points used for the baseline window.
    baseline_window: usize,
    /// Z-score threshold for Warning level.
    warning_threshold: f64,
    /// Z-score threshold for Critical level.
    critical_threshold: f64,
}

impl AnomalyDetector {
    /// Create a new detector with a given baseline window size.
    pub fn new(baseline_window: usize) -> Self {
        Self {
            baseline_window: baseline_window.max(2),
            warning_threshold: 2.0,
            critical_threshold: 3.0,
        }
    }

    /// Create a detector with custom thresholds.
    pub fn with_thresholds(
        baseline_window: usize,
        warning_threshold: f64,
        critical_threshold: f64,
    ) -> Self {
        Self {
            baseline_window: baseline_window.max(2),
            warning_threshold,
            critical_threshold,
        }
    }

    /// Analyze a data point against the given history.
    ///
    /// If `history` has fewer points than `baseline_window`, all available points
    /// are used for the baseline. The data point is compared against the computed
    /// z-score of the baseline.
    pub fn detect(&self, data_point: f64, history: &[f64]) -> AnomalyResult {
        let window = if history.len() < self.baseline_window {
            history
        } else {
            &history[history.len() - self.baseline_window..]
        };

        let (mean, stddev) = self.compute_stats(window);

        let z_score = if stddev > 1e-10 {
            (data_point - mean).abs() / stddev
        } else {
            // Low variance — any deviation is significant.
            if (data_point - mean).abs() > 1e-10 {
                f64::INFINITY
            } else {
                0.0
            }
        };

        let severity = if z_score >= self.critical_threshold {
            AnomalySeverity::Critical
        } else if z_score >= self.warning_threshold {
            AnomalySeverity::Warning
        } else {
            AnomalySeverity::Normal
        };

        AnomalyResult {
            is_anomaly: severity != AnomalySeverity::Normal,
            severity,
            deviation: z_score,
        }
    }

    /// Compute mean and standard deviation of a slice.
    fn compute_stats(&self, data: &[f64]) -> (f64, f64) {
        if data.is_empty() {
            return (0.0, 0.0);
        }
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();
        (mean, stddev)
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stable_history() -> Vec<f64> {
        (0..20).map(|i| 100.0 + (i as f64) * 0.1).collect()
    }

    #[test]
    fn test_normal_within_window() {
        let detector = AnomalyDetector::new(20);
        let history = stable_history();
        let result = detector.detect(101.0, &history);
        assert!(!result.is_anomaly);
        assert_eq!(result.severity, AnomalySeverity::Normal);
    }

    #[test]
    fn test_anomaly_detected() {
        let detector = AnomalyDetector::new(20);
        let history = stable_history();
        let result = detector.detect(500.0, &history);
        assert!(result.is_anomaly);
        assert_eq!(result.severity, AnomalySeverity::Critical);
        assert!(result.deviation > 3.0);
    }

    #[test]
    fn test_warning_level() {
        let detector = AnomalyDetector::with_thresholds(20, 1.5, 3.0);
        let mut history: Vec<f64> = vec![10.0; 20];
        // Inject slight variance.
        history[19] = 12.0;
        let result = detector.detect(14.0, &history);
        assert!(result.is_anomaly);
        assert_eq!(result.severity, AnomalySeverity::Warning);
    }

    #[test]
    fn test_empty_history() {
        let detector = AnomalyDetector::new(5);
        let result = detector.detect(42.0, &[]);
        // With empty history, mean=0, stddev=0 → infinity z-score → Critical.
        assert_eq!(result.severity, AnomalySeverity::Critical);
    }

    #[test]
    fn test_insufficient_history_uses_all() {
        let detector = AnomalyDetector::new(50);
        let history = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = detector.detect(3.0, &history);
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_low_variance_infinity_deviation() {
        let detector = AnomalyDetector::new(5);
        let history = vec![100.0; 5];
        let result = detector.detect(100.001, &history);
        // Nearly zero stddev → large z-score.
        assert!(result.deviation > 100.0);
    }

    #[test]
    fn test_display() {
        assert_eq!(AnomalySeverity::Normal.to_string(), "NORMAL");
        assert_eq!(AnomalySeverity::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn test_anomaly_result_is_anomaly_matches_severity() {
        let detector = AnomalyDetector::new(10);
        let history: Vec<f64> = (0..20).map(|i| 100.0 + i as f64 * 0.1).collect();
        let normal = detector.detect(101.0, &history);
        assert!(!normal.is_anomaly);
        assert_eq!(normal.severity, AnomalySeverity::Normal);

        let anomaly = detector.detect(500.0, &history);
        assert!(anomaly.is_anomaly);
        assert_ne!(anomaly.severity, AnomalySeverity::Normal);
    }

    #[test]
    fn test_custom_thresholds() {
        let detector = AnomalyDetector::with_thresholds(10, 1.0, 2.0);
        let history: Vec<f64> = (0..10).map(|i| 10.0 + i as f64 * 0.01).collect();
        // Small deviation with tight thresholds
        let result = detector.detect(11.0, &history);
        assert!(result.is_anomaly);
    }

    #[test]
    fn test_baseline_window_minimum_enforced() {
        // baseline_window should be at least 2
        let detector = AnomalyDetector::new(1);
        assert!(detector.baseline_window >= 2);
    }

    #[test]
    fn test_exact_mean_returns_zero_deviation() {
        let history = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let detector = AnomalyDetector::new(5);
        let result = detector.detect(5.0, &history);
        assert_eq!(result.deviation, 0.0);
        assert_eq!(result.severity, AnomalySeverity::Normal);
    }

    #[test]
    fn test_two_point_history() {
        let history = vec![10.0, 20.0];
        let detector = AnomalyDetector::new(5);
        let result = detector.detect(15.0, &history);
        // 15 is within 1 std dev of [10,20]
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_history_shorter_than_window() {
        let history = vec![100.0, 101.0, 102.0];
        let detector = AnomalyDetector::new(20);
        let result = detector.detect(101.5, &history);
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_deviation_is_positive() {
        let detector = AnomalyDetector::new(10);
        let history: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let result = detector.detect(100.0, &history);
        assert!(result.deviation >= 0.0);
    }

    #[test]
    fn test_default_impl() {
        let detector = AnomalyDetector::default();
        // Default window is 20
        let history: Vec<f64> = (0..20).map(|i| 50.0 + i as f64 * 0.1).collect();
        let result = detector.detect(50.0, &history);
        assert_eq!(result.severity, AnomalySeverity::Normal);
    }

    #[test]
    fn test_anomaly_severity_ordering() {
        assert!(AnomalySeverity::Normal < AnomalySeverity::Warning);
        assert!(AnomalySeverity::Warning < AnomalySeverity::Critical);
    }
}
