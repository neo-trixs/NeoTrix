use std::collections::VecDeque;

/// Tracks prediction accuracy over time and computes temporal prediction error.
/// Enables the consciousness system to detect when its model of reality is diverging.
#[derive(Debug, Clone)]
pub struct TemporalPredictionTracker {
    /// Cycle-by-cycle prediction scores (more recent first)
    prediction_history: VecDeque<f64>,
    /// Actual outcomes (more recent first)
    outcome_history: VecDeque<f64>,
    /// Maximum history length
    max_history: usize,
    /// Running prediction error (exponential moving average)
    running_error: f64,
    /// Alpha for EMA
    ema_alpha: f64,
    /// Total cycles observed
    cycle_count: u64,
}

impl TemporalPredictionTracker {
    pub fn new(max_history: usize, ema_alpha: f64) -> Self {
        Self {
            prediction_history: VecDeque::with_capacity(max_history),
            outcome_history: VecDeque::with_capacity(max_history),
            max_history,
            running_error: 0.0,
            ema_alpha,
            cycle_count: 0,
        }
    }

    /// Record a new prediction and its actual outcome.
    /// Returns the instant error: |prediction - outcome|.
    pub fn record_prediction(&mut self, predicted: f64, actual: f64) -> f64 {
        let error = (predicted - actual).abs();
        self.prediction_history.push_front(predicted);
        self.outcome_history.push_front(actual);
        if self.prediction_history.len() > self.max_history {
            self.prediction_history.pop_back();
            self.outcome_history.pop_back();
        }
        if self.cycle_count == 0 {
            self.running_error = error;
        } else {
            self.running_error =
                self.ema_alpha * error + (1.0 - self.ema_alpha) * self.running_error;
        }
        self.cycle_count += 1;
        error
    }

    pub fn running_prediction_error(&self) -> f64 {
        self.running_error
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    pub fn last_outcome(&self) -> Option<f64> {
        self.outcome_history.front().copied()
    }

    pub fn last_prediction(&self) -> Option<f64> {
        self.prediction_history.front().copied()
    }

    /// Detect if prediction error is trending upward (model degradation)
    pub fn is_diverging(&self, threshold: f64) -> bool {
        if self.prediction_history.len() < 10 {
            return false;
        }
        let recent_err = self
            .outcome_history
            .iter()
            .take(5)
            .zip(self.prediction_history.iter().take(5))
            .map(|(o, p)| (o - p).abs())
            .sum::<f64>()
            / 5.0;
        let older_err = self
            .outcome_history
            .iter()
            .skip(5)
            .take(5)
            .zip(self.prediction_history.iter().skip(5).take(5))
            .map(|(o, p)| (o - p).abs())
            .sum::<f64>()
            / 5.0;
        recent_err > older_err * (1.0 + threshold)
    }

    /// Volatility: standard deviation of recent prediction errors
    pub fn volatility(&self, window: usize) -> f64 {
        let window = window.min(self.prediction_history.len());
        if window < 2 {
            return 0.0;
        }
        let errors: Vec<f64> = self
            .prediction_history
            .iter()
            .take(window)
            .zip(self.outcome_history.iter().take(window))
            .map(|(p, o)| (p - o).abs())
            .collect();
        let mean = errors.iter().sum::<f64>() / errors.len() as f64;
        let variance =
            errors.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / (errors.len() - 1) as f64;
        variance.sqrt()
    }

    /// DYSCO-inspired multi-view invariance test: compare recent predictions
    /// with phase-shifted (older) version of the same window size.
    /// Returns mean absolute difference between recent and delayed predictions.
    /// A value significantly above the running prediction error suggests
    /// the underlying dynamics have changed (phase shift / regime change).
    /// Reference: arXiv:2606.13260 — multi-view contrastive dynamics identification.
    pub fn phase_shift_amplitude(&self, window: usize, delay: usize) -> f64 {
        let available = self.prediction_history.len();
        if available < window + delay + 1 || window < 3 {
            return 0.0;
        }
        let recent: Vec<f64> = self
            .prediction_history
            .iter()
            .take(window)
            .copied()
            .collect();
        let older: Vec<f64> = self
            .prediction_history
            .iter()
            .skip(delay)
            .take(window)
            .copied()
            .collect();
        let diff_sum: f64 = recent
            .iter()
            .zip(older.iter())
            .map(|(r, o)| (r - o).abs())
            .sum();
        diff_sum / window as f64
    }

    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "running_prediction_error": self.running_error,
            "diverging": self.is_diverging(0.2),
            "volatility": self.volatility(10),
            "cycle_count": self.cycle_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_records_prediction() {
        let mut t = TemporalPredictionTracker::new(10, 0.3);
        let err = t.record_prediction(0.8, 0.7);
        assert!((err - 0.1).abs() < 1e-10);
        assert!((t.running_prediction_error() - 0.1).abs() < 1e-10);
        assert_eq!(t.cycle_count(), 1);
    }

    #[test]
    fn test_ema_updates() {
        let mut t = TemporalPredictionTracker::new(10, 0.3);
        t.record_prediction(0.8, 0.8);
        t.record_prediction(0.8, 0.5);
        let expected = 0.3 * 0.3 + 0.7 * 0.0;
        assert!((t.running_prediction_error() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_not_diverging_with_few_samples() {
        let mut t = TemporalPredictionTracker::new(10, 0.3);
        for i in 0..9 {
            t.record_prediction(0.5, 0.5 + (i as f64) * 0.01);
        }
        assert!(!t.is_diverging(0.2));
    }

    #[test]
    fn test_diverging_detection() {
        let mut t = TemporalPredictionTracker::new(100, 0.3);
        for i in 0..20 {
            let noise = if i < 10 { 0.05 } else { 0.25 };
            t.record_prediction(0.5, 0.5 + noise);
        }
        assert!(t.is_diverging(0.2));
    }

    #[test]
    fn test_volatility() {
        let mut t = TemporalPredictionTracker::new(100, 0.3);
        t.record_prediction(0.5, 0.5);
        t.record_prediction(0.5, 0.5);
        assert!((t.volatility(10) - 0.0).abs() < 1e-10);
        t.record_prediction(0.5, 1.0);
        assert!(t.volatility(10) > 0.0);
    }

    #[test]
    fn test_metrics_output() {
        let mut t = TemporalPredictionTracker::new(100, 0.3);
        t.record_prediction(0.5, 0.6);
        let m = t.metrics();
        assert!((m["running_prediction_error"].as_f64().unwrap() - 0.1).abs() < 1e-10);
        assert_eq!(m["cycle_count"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_phase_shift_zero_with_few_samples() {
        let t = TemporalPredictionTracker::new(10, 0.3);
        assert!(
            (t.phase_shift_amplitude(5, 2) - 0.0).abs() < 1e-10,
            "insufficient samples should return 0"
        );
    }

    #[test]
    fn test_phase_shift_stationary_signal() {
        let mut t = TemporalPredictionTracker::new(100, 0.3);
        // Stationary signal: predict 0.5 with small noise
        for _ in 0..50 {
            t.record_prediction(0.5, 0.5);
        }
        let amp = t.phase_shift_amplitude(10, 5);
        assert!(
            amp < 0.01,
            "stationary signal should have near-zero phase shift"
        );
    }

    #[test]
    fn test_phase_shift_regime_change() {
        let mut t = TemporalPredictionTracker::new(100, 0.3);
        // First regime: predict 0.5, outcome 0.5
        for _ in 0..30 {
            t.record_prediction(0.5, 0.5);
        }
        // Second regime: predict 0.5, outcome 0.9 (regime change)
        for _ in 0..30 {
            t.record_prediction(0.5, 0.9);
        }
        // Compare recent (second regime) with older (first regime)
        let amp = t.phase_shift_amplitude(10, 20);
        assert!(
            amp > 0.2,
            "regime change should produce detectable phase shift: got {}",
            amp
        );
    }

    /// DYSCO-inspired periodic signal test: multi-view invariance across phase.
    /// If the underlying process is periodic, predictions at the same phase
    /// (separated by period) should align better than out-of-phase predictions.
    #[test]
    fn test_phase_shift_periodic_invariance() {
        let mut t = TemporalPredictionTracker::new(100, 0.3);
        // Periodic signal: sin wave with period ~10 samples
        for i in 0..60 {
            let phase = (i as f64 * std::f64::consts::TAU / 10.0).sin();
            let predicted = 0.5;
            let actual = 0.5 + 0.3 * phase;
            t.record_prediction(predicted, actual);
        }
        // In-phase comparison (shift by period = 10)
        let in_phase = t.phase_shift_amplitude(8, 10);
        // Out-of-phase comparison (shift by half-period = 5)
        let out_phase = t.phase_shift_amplitude(8, 5);
        assert!(
            in_phase < out_phase,
            "in-phase predictions should align better than out-of-phase: in_phase={}, out_phase={}",
            in_phase,
            out_phase
        );
    }
}
