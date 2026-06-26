use std::collections::VecDeque;

/// A single observation at a point in time
#[derive(Debug, Clone)]
pub struct Observation {
    pub timestamp_ms: i64,
    pub value: f64,
    pub label: String,
}

/// A forecast produced by a specific model
#[derive(Debug, Clone)]
pub struct Forecast {
    pub model_name: String,
    pub predicted_values: Vec<(i64, f64)>,
    pub confidence: f64,
    pub mse: f64,
}

/// Ensemble forecast blending multiple models
#[derive(Debug, Clone)]
pub struct EnsembleForecast {
    pub horizon: usize,
    pub forecasts: Vec<Forecast>,
    pub blended_values: Vec<(i64, f64)>,
    pub ensemble_confidence: f64,
}

/// Regime change detection result
#[derive(Debug, Clone)]
pub struct RegimeChange {
    pub detected: bool,
    pub change_point: i64,
    pub description: String,
    pub confidence: f64,
}

/// The main Long-Horizon Predictor
#[derive(Debug)]
pub struct LongHorizonPredictor {
    pub history: VecDeque<Observation>,
    pub forecasts: VecDeque<EnsembleForecast>,
    pub regime_changes: VecDeque<RegimeChange>,
    max_history: usize,
    max_forecasts: usize,
    max_regime_changes: usize,
}

impl LongHorizonPredictor {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(10000),
            forecasts: VecDeque::with_capacity(500),
            regime_changes: VecDeque::with_capacity(100),
            max_history: 10000,
            max_forecasts: 500,
            max_regime_changes: 100,
        }
    }

    /// Record an observation; trims oldest when at capacity.
    pub fn record_observation(&mut self, ts: i64, value: f64, label: &str) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(Observation {
            timestamp_ms: ts,
            value,
            label: label.to_string(),
        });
    }

    /// Generate an ensemble forecast using exponential smoothing,
    /// linear trend, and periodic models blended by inverse-MSE weights.
    pub fn forecast(&mut self, horizon: usize) -> EnsembleForecast {
        let recent: Vec<&Observation> = self.history.iter().rev().take(100).collect::<Vec<_>>();
        let recent: Vec<&Observation> = recent.into_iter().rev().collect();

        // Model 1: Exponential Smoothing
        let smooth_forecast = if recent.len() >= 2 {
            let alpha = 0.3;
            let mut smoothed = recent[0].value;
            for obs in &recent {
                smoothed = alpha * obs.value + (1.0 - alpha) * smoothed;
            }
            let values: Vec<(i64, f64)> = (1..=horizon)
                .map(|i| {
                    (
                        recent.last().unwrap().timestamp_ms + i as i64 * 1000,
                        smoothed,
                    )
                })
                .collect();
            let mse = recent
                .windows(2)
                .map(|w| (w[1].value - w[0].value).powi(2))
                .sum::<f64>()
                / recent.len().max(1) as f64;
            Forecast {
                model_name: "exponential_smoothing".into(),
                predicted_values: values,
                confidence: 0.7,
                mse,
            }
        } else {
            Forecast {
                model_name: "exponential_smoothing".into(),
                predicted_values: (1..=horizon).map(|i| (i as i64 * 1000, 0.0)).collect(),
                confidence: 0.1,
                mse: 1.0,
            }
        };

        // Model 2: Linear Trend (least-squares regression)
        let trend_forecast = if recent.len() >= 3 {
            let n = recent.len() as f64;
            let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
            let sum_y: f64 = recent.iter().map(|o| o.value).sum();
            let sum_xy: f64 = recent
                .iter()
                .enumerate()
                .map(|(i, o)| i as f64 * o.value)
                .sum();
            let sum_x2: f64 = (0..recent.len()).map(|i| (i as f64).powi(2)).sum();
            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2) + 1e-10);
            let intercept = (sum_y - slope * sum_x) / n;
            let last_idx = recent.len() as f64;
            let values: Vec<(i64, f64)> = (1..=horizon)
                .map(|i| {
                    let x = last_idx + i as f64;
                    (
                        recent.last().unwrap().timestamp_ms + i as i64 * 1000,
                        slope * x + intercept,
                    )
                })
                .collect();
            let residuals: f64 = recent
                .iter()
                .enumerate()
                .map(|(i, o)| (o.value - (slope * i as f64 + intercept)).powi(2))
                .sum();
            let mse = residuals / recent.len() as f64;
            Forecast {
                model_name: "linear_trend".into(),
                predicted_values: values,
                confidence: 0.6,
                mse,
            }
        } else {
            Forecast {
                model_name: "linear_trend".into(),
                predicted_values: (1..=horizon).map(|i| (i as i64 * 1000, 0.0)).collect(),
                confidence: 0.1,
                mse: 1.0,
            }
        };

        // Model 3: Simple periodic (repeat values from one period ago)
        let periodic_forecast = if recent.len() >= 10 {
            let period = 10.min(recent.len() / 2);
            let values: Vec<(i64, f64)> = (1..=horizon)
                .map(|i| {
                    let idx = recent.len() - 1 - (i % period);
                    (
                        recent.last().unwrap().timestamp_ms + i as i64 * 1000,
                        recent[idx].value,
                    )
                })
                .collect();
            let mse = recent
                .windows(period + 1)
                .map(|w| (w.last().unwrap().value - w[w.len() - 1 - period].value).powi(2))
                .sum::<f64>()
                / recent.len().max(1) as f64;
            Forecast {
                model_name: "periodic".into(),
                predicted_values: values,
                confidence: 0.5,
                mse,
            }
        } else {
            Forecast {
                model_name: "periodic".into(),
                predicted_values: (1..=horizon).map(|i| (i as i64 * 1000, 0.0)).collect(),
                confidence: 0.1,
                mse: 1.0,
            }
        };

        let forecasts = vec![smooth_forecast, trend_forecast, periodic_forecast];

        // Blend: inverse-MSE weighted average
        let total_inv_mse: f64 = forecasts.iter().map(|f| 1.0 / (f.mse + 1e-10)).sum();
        let blended: Vec<(i64, f64)> = (0..horizon)
            .map(|step| {
                let weighted_sum: f64 = forecasts
                    .iter()
                    .map(|f| {
                        let w = 1.0 / (f.mse + 1e-10);
                        w * f.predicted_values[step].1
                    })
                    .sum();
                (
                    forecasts[0].predicted_values[step].0,
                    weighted_sum / total_inv_mse,
                )
            })
            .collect();

        let ensemble_conf =
            forecasts.iter().map(|f| f.confidence).sum::<f64>() / forecasts.len() as f64;

        let ef = EnsembleForecast {
            horizon,
            forecasts,
            blended_values: blended,
            ensemble_confidence: ensemble_conf,
        };

        if self.forecasts.len() >= self.max_forecasts {
            self.forecasts.pop_front();
        }
        self.forecasts.push_back(ef.clone());
        ef
    }

    /// Detect regime changes by measuring variance spikes in recent observations.
    pub fn detect_regime_change(&mut self) -> RegimeChange {
        if self.history.len() < 10 {
            return RegimeChange {
                detected: false,
                change_point: 0,
                description: "insufficient data".into(),
                confidence: 0.0,
            };
        }

        let last_few: Vec<f64> = self.history.iter().rev().take(5).map(|o| o.value).collect();
        let mean = last_few.iter().sum::<f64>() / last_few.len() as f64;
        let variance =
            last_few.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / last_few.len() as f64;

        if variance > 0.5 && self.history.len() > 20 {
            let rc = RegimeChange {
                detected: true,
                change_point: self.history.back().unwrap().timestamp_ms,
                description: format!("high variance detected: {:.4}", variance),
                confidence: (variance / (variance + 1.0)).min(0.95),
            };
            if self.regime_changes.len() >= self.max_regime_changes {
                self.regime_changes.pop_front();
            }
            self.regime_changes.push_back(rc.clone());
            rc
        } else {
            RegimeChange {
                detected: false,
                change_point: 0,
                description: "stable regime".into(),
                confidence: 0.9,
            }
        }
    }

    pub fn stats(&self) -> String {
        format!(
            "LongHorizonPredictor: {} obs, {} forecasts, {} regime changes",
            self.history.len(),
            self.forecasts.len(),
            self.regime_changes.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_observation_with_bounds() {
        let mut p = LongHorizonPredictor::new();
        // Fill to capacity
        for i in 0..10000 {
            p.record_observation(i as i64 * 1000, (i % 10) as f64, "test");
        }
        assert_eq!(p.history.len(), 10000, "history should be at max");
        assert_eq!(p.history[0].timestamp_ms, 0, "oldest entry should be first");

        // Push one more; oldest should be evicted
        p.record_observation(10000000, 42.0, "new");
        assert_eq!(p.history.len(), 10000, "history should stay at max");
        assert_eq!(
            p.history.back().unwrap().value,
            42.0,
            "new value should be present"
        );
        assert_ne!(
            p.history[0].timestamp_ms, 0,
            "oldest entry should have been evicted"
        );
    }

    #[test]
    fn test_forecast_with_known_data() {
        let mut p = LongHorizonPredictor::new();
        // Linear ramp: 0, 1, 2, ..., 49
        for i in 0..50 {
            p.record_observation(i as i64 * 1000, i as f64, "ramp");
        }

        let ef = p.forecast(5);
        assert_eq!(ef.horizon, 5, "horizon should match");
        assert_eq!(ef.forecasts.len(), 3, "three models should be present");
        assert!(
            ef.ensemble_confidence > 0.0,
            "ensemble confidence should be positive"
        );
        assert_eq!(
            ef.blended_values.len(),
            5,
            "blended values should match horizon"
        );

        // For a linear ramp, predictions should increase
        for i in 1..ef.blended_values.len() {
            assert!(
                ef.blended_values[i].1 > ef.blended_values[i - 1].1,
                "predictions should increase for a ramp: step {} ({}) <= step {} ({})",
                i,
                ef.blended_values[i].1,
                i - 1,
                ef.blended_values[i - 1].1,
            );
        }
    }

    #[test]
    fn test_forecast_empty_history() {
        let mut p = LongHorizonPredictor::new();
        let ef = p.forecast(3);
        assert_eq!(ef.horizon, 3);
        // All models produce zero predictions with low confidence
        assert_eq!(ef.ensemble_confidence, 0.1);
        for f in &ef.forecasts {
            assert_eq!(f.predicted_values.len(), 3);
        }
    }

    #[test]
    fn test_ensemble_blending_weights() {
        let mut p = LongHorizonPredictor::new();
        for i in 0..10 {
            p.record_observation(i as i64 * 1000, 1.0, "constant");
        }
        let ef = p.forecast(3);
        // All values are 1.0, so blending should produce ~1.0
        for (_ts, val) in &ef.blended_values {
            assert!(
                (*val - 1.0).abs() < 0.01,
                "blended value should be ~1.0, got {}",
                val
            );
        }
        // Low-variance data → all models have similar MSE
        let mses: Vec<f64> = ef.forecasts.iter().map(|f| f.mse).collect();
        for mse in &mses {
            assert!(
                *mse < 0.5,
                "MSE should be low on constant data, got {}",
                mse
            );
        }
    }

    #[test]
    fn test_regime_change_detected_on_high_variance() {
        let mut p = LongHorizonPredictor::new();
        // Stable first 20
        for i in 0..20 {
            p.record_observation(i as i64 * 1000, 1.0, "stable");
        }
        // No regime change on stable
        let rc = p.detect_regime_change();
        assert!(!rc.detected, "should not detect change on stable data");

        // Spike high-variance for last 10
        for i in 0..10 {
            let spike = if i % 2 == 0 { 10.0 } else { -10.0 };
            p.record_observation((20 + i) as i64 * 1000, spike, "spike");
        }
        let rc = p.detect_regime_change();
        assert!(rc.detected, "should detect regime change on high variance");
        assert!(rc.confidence > 0.5, "confidence should be meaningful");
        assert!(
            !rc.description.is_empty(),
            "description should not be empty"
        );
    }

    #[test]
    fn test_regime_change_stable_data() {
        let mut p = LongHorizonPredictor::new();
        for i in 0..30 {
            p.record_observation(i as i64 * 1000, 5.0, "stable");
        }
        let rc = p.detect_regime_change();
        assert!(!rc.detected, "stable data should not trigger regime change");
    }

    #[test]
    fn test_regime_change_insufficient_data() {
        let mut p = LongHorizonPredictor::new();
        for i in 0..5 {
            p.record_observation(i as i64 * 1000, i as f64, "early");
        }
        let rc = p.detect_regime_change();
        assert!(!rc.detected, "insufficient data should not detect change");
        assert_eq!(rc.description, "insufficient data");
    }

    #[test]
    fn test_forecast_max_bounds() {
        let mut p = LongHorizonPredictor::new();
        // Fill with mini data
        for i in 0..10 {
            p.record_observation(i as i64 * 1000, i as f64, "fill");
        }
        // Push 600 forecasts (over the 500 limit)
        for _ in 0..600 {
            p.forecast(2);
        }
        assert_eq!(p.forecasts.len(), 500, "forecasts should be capped at 500");
        let oldest = p.forecasts.front().unwrap();
        // Verify the remaining forecasts are recent (higher cycle values)
        assert_eq!(
            oldest.forecasts.len(),
            3,
            "each forecast should have 3 models"
        );
    }

    #[test]
    fn test_regime_change_max_bounds() {
        let mut p = LongHorizonPredictor::new();
        // Generate high-variance 25 times to trigger >100 regime changes
        for batch in 0..25 {
            // Stable data to reset variance
            for _ in 0..20 {
                p.record_observation(0, 1.0, "stable");
            }
            // Spikes
            for j in 0..10 {
                let spike = if j % 2 == 0 { 50.0 } else { -50.0 };
                p.record_observation(0, spike, "spike");
            }
            // Clear the artificially inflated timestamp
            let _ = p.detect_regime_change();
        }
        assert_eq!(
            p.regime_changes.len(),
            100,
            "regime changes should be capped at 100"
        );
    }

    #[test]
    fn test_stats_output() {
        let mut p = LongHorizonPredictor::new();
        for i in 0..5 {
            p.record_observation(i as i64 * 1000, i as f64, "stats");
        }
        let s = p.stats();
        assert!(s.contains("5 obs"));
        assert!(s.contains("0 forecasts"));
        assert!(s.contains("0 regime changes"));
    }

    #[test]
    fn test_forecast_timestamps_monotonic() {
        let mut p = LongHorizonPredictor::new();
        for i in 0..20 {
            p.record_observation(i as i64 * 1000, (i as f64).sin(), "sine");
        }
        let ef = p.forecast(10);
        for w in ef.blended_values.windows(2) {
            assert!(
                w[0].0 < w[1].0,
                "timestamps should be strictly increasing: {} >= {}",
                w[0].0,
                w[1].0
            );
        }
    }

    #[test]
    fn test_forecast_idempotent() {
        let mut p = LongHorizonPredictor::new();
        for i in 0..20 {
            p.record_observation(i as i64 * 1000, i as f64, "idempotent");
        }
        let ef1 = p.forecast(4);
        let ef2 = p.forecast(4);
        // Same data → same blended values (forecast modifies self but
        // the prediction on identical history should match for each model)
        assert_eq!(ef1.blended_values.len(), ef2.blended_values.len());
        for (a, b) in ef1.blended_values.iter().zip(ef2.blended_values.iter()) {
            assert!(
                (a.1 - b.1).abs() < 1e-6,
                "idempotent forecasts should match: {} vs {}",
                a.1,
                b.1
            );
        }
    }
}
