#![forbid(unsafe_code)]

/// Predicts future values from time-series history using linear regression
/// combined with exponential smoothing.
pub struct Predictor {
    /// Smoothing factor for exponential smoothing (0.0 ..= 1.0).
    alpha: f64,
}

impl Predictor {
    pub fn new() -> Self {
        Self { alpha: 0.3 }
    }

    pub fn with_alpha(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.01, 0.99),
        }
    }

    /// Predict `steps` future values using linear regression + exponential smoothing.
    ///
    /// The final prediction blends the linear trend extrapolation with the
    /// exponential smoothing baseline.
    pub fn predict(&self, history: &[f64], steps: usize) -> Vec<f64> {
        if history.is_empty() || steps == 0 {
            return vec![];
        }

        let (slope, intercept) = self.linear_regression(history);
        let smoothed = self.exponential_smoothing(history);

        let n = history.len() as f64;
        let baseline = if smoothed.is_empty() {
            *history.last().unwrap()
        } else {
            *smoothed.last().unwrap()
        };

        (1..=steps)
            .map(|i| {
                let linear_pred = intercept + slope * (n + i as f64);
                // Blend: 60% linear trend + 40% exponential smoothing level.
                0.6 * linear_pred + 0.4 * baseline
            })
            .collect()
    }

    /// Forecast when the metric will cross `threshold`.
    ///
    /// Returns `Some(turns)` if the predicted trend will cross the threshold
    /// within a reasonable horizon (up to 1000 steps). Returns `None` if the
    /// trend moves away from the threshold or the horizon is exceeded.
    pub fn forecast_failure(&self, history: &[f64], threshold: f64) -> Option<usize> {
        if history.is_empty() {
            return None;
        }

        let last = *history.last().unwrap();
        let (slope, intercept) = self.linear_regression(history);
        let n = history.len() as f64;

        // Determine direction relative to threshold.
        let moving_towards_threshold = if threshold > last {
            slope > 0.0
        } else {
            slope < 0.0
        };

        if !moving_towards_threshold || slope.abs() < 1e-12 {
            return None;
        }

        let max_horizon = 1000usize;
        for i in 1..=max_horizon {
            let predicted = intercept + slope * (n + i as f64);
            if (threshold > last && predicted >= threshold)
                || (threshold < last && predicted <= threshold)
            {
                return Some(i);
            }
        }

        None
    }

    /// Simple linear regression: returns (slope, intercept).
    fn linear_regression(&self, data: &[f64]) -> (f64, f64) {
        let n = data.len() as f64;
        if n < 2.0 {
            return (0.0, data[0]);
        }

        let sum_x: f64 = (0..data.len()).map(|i| i as f64).sum();
        let sum_y: f64 = data.iter().sum();
        let sum_xy: f64 = data.iter().enumerate().map(|(i, y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..data.len()).map(|i| (i as f64).powi(2)).sum();

        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-12 {
            return (0.0, sum_y / n);
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        (slope, intercept)
    }

    /// Single-pass exponential smoothing.
    fn exponential_smoothing(&self, data: &[f64]) -> Vec<f64> {
        if data.is_empty() {
            return vec![];
        }
        let mut result = vec![data[0]];
        for &val in &data[1..] {
            let prev = *result.last().unwrap();
            result.push(self.alpha * val + (1.0 - self.alpha) * prev);
        }
        result
    }
}

impl Default for Predictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_linear_trend() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..20).map(|i| i as f64 * 2.0).collect();
        let predicted = predictor.predict(&history, 5);
        assert_eq!(predicted.len(), 5);
        for window in predicted.windows(2) {
            assert!(window[1] > window[0]);
        }
    }

    #[test]
    fn test_predict_empty_history() {
        let predictor = Predictor::new();
        assert!(predictor.predict(&[], 5).is_empty());
    }

    #[test]
    fn test_predict_zero_steps() {
        let predictor = Predictor::new();
        assert!(predictor.predict(&[1.0, 2.0], 0).is_empty());
    }

    #[test]
    fn test_forecast_failure_rising() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let turns = predictor.forecast_failure(&history, 30.0);
        assert!(turns.is_some());
        assert!(turns.unwrap() > 0);
    }

    #[test]
    fn test_forecast_failure_no_crossing() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let turns = predictor.forecast_failure(&history, -10.0);
        assert!(turns.is_none());
    }

    #[test]
    fn test_forecast_failure_empty() {
        let predictor = Predictor::new();
        assert!(predictor.forecast_failure(&[], 100.0).is_none());
    }

    #[test]
    fn test_forecast_failure_flat_line() {
        let predictor = Predictor::new();
        let history = vec![10.0; 20];
        let turns = predictor.forecast_failure(&history, 100.0);
        assert!(turns.is_none());
    }

    #[test]
    fn test_linear_regression_single_point() {
        let predictor = Predictor::new();
        let (slope, intercept) = predictor.linear_regression(&[5.0]);
        assert_eq!(slope, 0.0);
        assert_eq!(intercept, 5.0);
    }

    #[test]
    fn test_predict_with_noisy_data() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..30)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.5;
                let noise = if i % 3 == 0 { 10.0 } else { -5.0 };
                base + noise
            })
            .collect();
        let predicted = predictor.predict(&history, 5);
        assert_eq!(predicted.len(), 5);
        // Predictions should be roughly in range
        for p in &predicted {
            assert!(*p > 50.0 && *p < 300.0);
        }
    }

    #[test]
    fn test_predict_with_alpha_zero_one() {
        let predictor = Predictor::with_alpha(0.01);
        let history: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let predicted = predictor.predict(&history, 3);
        assert_eq!(predicted.len(), 3);
    }

    #[test]
    fn test_predict_with_alpha_high() {
        let predictor = Predictor::with_alpha(0.99);
        let history: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let predicted = predictor.predict(&history, 3);
        assert_eq!(predicted.len(), 3);
    }

    #[test]
    fn test_alpha_clamped() {
        let p_low = Predictor::with_alpha(-0.5);
        let p_high = Predictor::with_alpha(5.0);
        // Both should be clamped to valid range
        let history = vec![1.0, 2.0, 3.0];
        assert_eq!(p_low.predict(&history, 1).len(), 1);
        assert_eq!(p_high.predict(&history, 1).len(), 1);
    }

    #[test]
    fn test_forecast_failure_decreasing_towards_threshold() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..20).map(|i| 100.0 - i as f64).collect();
        // Threshold below last value, trend is decreasing
        let turns = predictor.forecast_failure(&history, 50.0);
        assert!(turns.is_some());
    }

    #[test]
    fn test_forecast_failure_away_from_threshold() {
        let predictor = Predictor::new();
        let history: Vec<f64> = (0..20).map(|i| 10.0 + i as f64).collect();
        // Threshold is below and trend is increasing → no crossing
        let turns = predictor.forecast_failure(&history, 5.0);
        assert!(turns.is_none());
    }

    #[test]
    fn test_exponential_smoothing_length_matches_input() {
        let predictor = Predictor::new();
        let history = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let smoothed = predictor.exponential_smoothing(&history);
        assert_eq!(smoothed.len(), history.len());
    }

    #[test]
    fn test_exponential_smoothing_first_value_unchanged() {
        let predictor = Predictor::new();
        let history = vec![10.0, 20.0, 30.0];
        let smoothed = predictor.exponential_smoothing(&history);
        assert_eq!(smoothed[0], 10.0);
    }

    #[test]
    fn test_linear_regression_perfect_line() {
        let predictor = Predictor::new();
        let data = vec![0.0, 2.0, 4.0, 6.0, 8.0];
        let (slope, intercept) = predictor.linear_regression(&data);
        assert!((slope - 2.0).abs() < 1e-6);
        assert!((intercept - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_regression_constant() {
        let predictor = Predictor::new();
        let data = vec![5.0, 5.0, 5.0, 5.0];
        let (slope, intercept) = predictor.linear_regression(&data);
        assert!(slope.abs() < 1e-10);
        assert!((intercept - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_default_impl() {
        let p1 = Predictor::new();
        let p2 = Predictor::default();
        let history = vec![1.0, 2.0, 3.0];
        assert_eq!(p1.predict(&history, 2), p2.predict(&history, 2));
    }
}
