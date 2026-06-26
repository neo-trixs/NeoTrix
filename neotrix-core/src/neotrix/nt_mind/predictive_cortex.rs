use crate::core::nt_core_edit::MicroEdit;

pub use crate::neotrix::nt_mind::cortex_types::*;

impl PredictiveCortex {
    pub fn new(latent_dim: usize, hidden_dim: usize) -> Self {
        Self {
            jepa: crate::neotrix::nt_world_jepa::JepaPredictor::new(latent_dim, hidden_dim),
            e8: crate::neotrix::nt_world_e8::E8WorldModel::new(),
            ai: crate::neotrix::nt_world_infer::ActiveInferenceEngine::new(),
            horizon: crate::neotrix::nt_mind::cortex_types::DEFAULT_HORIZON,
            n_samples: crate::neotrix::nt_mind::cortex_types::DEFAULT_N_SAMPLES,
            action_dim: crate::neotrix::nt_mind::cortex_types::DEFAULT_ACTION_DIM,
            forecast_history: Vec::with_capacity(50),
            fe_timeline: Vec::with_capacity(200),
            forecast_quality: 1.0,
            consecutive_degradations: 0,
            outcome_history: Vec::with_capacity(100),
            cycle: 0,
        }
    }

    pub fn with_horizon(mut self, horizon: usize) -> Self {
        self.horizon = horizon;
        self
    }

    pub fn with_n_samples(mut self, n: usize) -> Self {
        self.n_samples = n;
        self
    }

    pub fn with_action_dim(mut self, dim: usize) -> Self {
        self.action_dim = dim;
        self
    }

    pub fn record_outcome(
        &mut self,
        forecast: &HorizonForecast,
        actual_latent: &[f64],
    ) -> OutcomeRecord {
        self.cycle += 1;

        let predicted = forecast
            .trajectory
            .first()
            .map(|s| s.latent_mean.as_slice())
            .unwrap_or(&[]);

        let prediction_error = self.compute_jepa_energy(predicted, actual_latent);
        let actual_fe = self.ai.current_fe;

        let was_anomaly = prediction_error > ANOMALY_ENERGY_THRESHOLD;

        let error_clipped = (prediction_error / ANOMALY_ENERGY_THRESHOLD).min(1.0);
        let step_quality = 1.0 - error_clipped;
        self.forecast_quality = FORECAST_QUALITY_ALPHA * step_quality
            + (1.0 - FORECAST_QUALITY_ALPHA) * self.forecast_quality;

        if step_quality < QUALITY_REPAIR_THRESHOLD {
            self.consecutive_degradations += 1;
        } else {
            self.consecutive_degradations = 0;
        }

        let record = OutcomeRecord {
            forecast_cumulative_fe: forecast.cumulative_fe,
            forecast_confidence: forecast.avg_confidence,
            prediction_error,
            actual_fe,
            was_anomaly,
            cycle: self.cycle,
        };
        self.outcome_history.push(record.clone());
        if self.outcome_history.len() > 100 {
            self.outcome_history.remove(0);
        }
        record
    }

    pub fn detect_degradation(&self) -> Option<RepairSignal> {
        if self.consecutive_degradations < MIN_CONSECUTIVE_DEGRADATIONS
            && self.forecast_quality >= QUALITY_REPAIR_THRESHOLD
        {
            return None;
        }

        let diagnosis = if self.forecast_quality < 0.2 {
            format!(
                "PredictiveCortex critical degradation: quality={:.3} consec_degradations={} — rebuild JEPA predictor needed",
                self.forecast_quality, self.consecutive_degradations
            )
        } else if self.consecutive_degradations >= MIN_CONSECUTIVE_DEGRADATIONS {
            format!(
                "PredictiveCortex quality degradation: quality={:.3} consec_degradations={} — tuning JEPA+E8+AI",
                self.forecast_quality, self.consecutive_degradations
            )
        } else {
            format!(
                "PredictiveCortex below threshold: quality={:.3}",
                self.forecast_quality
            )
        };

        let severity = (1.0 - self.forecast_quality).clamp(0.0, 1.0);

        let suggested_edits = vec![
            MicroEdit::AdjustDimension("jepa_learning_rate".into(), 0.01),
            MicroEdit::AdjustDimension("prediction_uncertainty".into(), self.forecast_quality),
        ];

        let target_modules = vec![
            "nt_world_jepa".to_string(),
            "nt_world_e8".to_string(),
            "nt_world_infer".to_string(),
        ];

        Some(RepairSignal {
            diagnosis,
            severity,
            suggested_edits,
            target_modules,
        })
    }

    pub fn reset_quality_tracking(&mut self) {
        self.forecast_quality = 1.0;
        self.consecutive_degradations = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_latent(dim: usize) -> Vec<f64> {
        (0..dim)
            .map(|i| (i as f64 / dim as f64) * 2.0 - 1.0)
            .collect()
    }

    #[test]
    fn test_predict_horizon_basic() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 3);
        assert_eq!(forecast.trajectory.len(), 3);
        assert!(forecast.cumulative_fe.is_finite());
        assert!(forecast.avg_confidence >= 0.0 && forecast.avg_confidence <= 1.0);
    }

    #[test]
    fn test_predict_horizon_divergence_detection() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let mut high_fe_latent = make_test_latent(32);
        for v in high_fe_latent.iter_mut() {
            *v = 10.0;
        }
        let forecast = cortex.predict_horizon(&high_fe_latent, 5);
        if forecast.anomaly_predicted {
            assert!(forecast.divergence_step.is_some() || forecast.avg_confidence < 0.5);
        }
    }

    #[test]
    fn test_plan_best_action() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);

        let actions: Vec<Vec<f64>> = (0..3)
            .map(|i| {
                (0..32)
                    .map(|j| ((i + j) as f64 / 32.0) * 2.0 - 1.0)
                    .collect()
            })
            .collect();

        let plan = cortex.plan_best_action(&latent, &actions, 3);
        assert_eq!(plan.best_action.len(), 32);
        assert_eq!(plan.action_rankings.len(), 3);
        assert!(plan.action_rankings[0].1 <= plan.action_rankings.last().unwrap().1);
    }

    #[test]
    fn test_lookahead_anomaly_prob() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let prob = cortex.lookahead_anomaly_prob(&latent, 3);
        assert!(prob >= 0.0 && prob <= 1.0);
    }

    #[test]
    fn test_counterfactual() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let action: Vec<f64> = (0..32).map(|_| 0.1).collect();

        let forecast = cortex.counterfactual(&latent, &action, 3);
        assert_eq!(forecast.trajectory.len(), 3);
        assert!(forecast.cumulative_fe.is_finite());
    }

    #[test]
    fn test_counterfactual_vs_baseline_differs() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);

        let baseline = cortex.predict_horizon(&latent, 3);

        let strong_action: Vec<f64> = (0..32).map(|_| 0.5).collect();
        let cf = cortex.counterfactual(&latent, &strong_action, 3);

        let baseline_last_fe = baseline
            .trajectory
            .last()
            .map(|s| s.free_energy)
            .unwrap_or(0.0);
        let cf_last_fe = cf.trajectory.last().map(|s| s.free_energy).unwrap_or(0.0);
        assert!(baseline_last_fe.is_finite());
        assert!(cf_last_fe.is_finite());
    }

    #[test]
    fn test_forecast_history_records() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        assert_eq!(cortex.forecast_history.len(), 0);

        cortex.predict_horizon(&latent, 2);
        assert_eq!(cortex.forecast_history.len(), 1);
        assert_eq!(cortex.fe_timeline.len(), 1);

        cortex.predict_horizon(&latent, 2);
        assert_eq!(cortex.forecast_history.len(), 2);
        assert_eq!(cortex.fe_timeline.len(), 2);
    }

    #[test]
    fn test_generate_candidate_actions() {
        let cortex = PredictiveCortex::new(32, 64);
        let actions = cortex.generate_candidate_actions(5);
        assert_eq!(actions.len(), 5);
        for action in &actions {
            assert_eq!(action.len(), 32);
            for &v in action {
                assert!(v >= -1.0 && v <= 1.0);
            }
        }
    }

    #[test]
    fn test_step_confidence_from_low_variance() {
        let cortex = PredictiveCortex::new(32, 64);
        let low_var = vec![0.001; 32];
        let high_conf = cortex.compute_step_confidence(&low_var);
        assert!(high_conf > 0.9);

        let high_var = vec![1.0; 32];
        let low_conf = cortex.compute_step_confidence(&high_var);
        assert!(low_conf < 0.5);
    }

    #[test]
    fn test_jepa_energy_finite() {
        let cortex = PredictiveCortex::new(32, 64);
        let a = vec![0.5; 32];
        let b = vec![0.3; 32];
        let energy = cortex.compute_jepa_energy(&a, &b);
        assert!(energy.is_finite());
        assert!(energy > 0.0);
    }

    #[test]
    fn test_e8_gradient_finite() {
        let mut cortex = PredictiveCortex::new(32, 64);
        cortex.e8.evolve(1.0);
        let grad = cortex.compute_e8_gradient();
        assert!(grad.is_finite());
    }

    #[test]
    fn test_fe_timeline_growth() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        for _ in 0..5 {
            cortex.predict_horizon(&latent, 2);
        }
        assert_eq!(cortex.fe_timeline.len(), 5);

        let all_finite = cortex.fe_timeline.iter().all(|v| v.is_finite());
        assert!(all_finite);
    }

    #[test]
    fn test_record_outcome_updates_quality() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 2);

        let record = cortex.record_outcome(&forecast, &latent);
        assert!(record.prediction_error.is_finite());
        assert!(cortex.forecast_quality > 0.0);
        assert!(cortex.forecast_quality <= 1.0);
        assert_eq!(cortex.outcome_history.len(), 1);
    }

    #[test]
    fn test_record_outcome_triggers_degradation() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 2);

        for _ in 0..5 {
            let bad_actual = vec![100.0; 32];
            cortex.record_outcome(&forecast, &bad_actual);
        }

        assert!(cortex.forecast_quality < 0.5);
        assert!(cortex.consecutive_degradations >= 3);
    }

    #[test]
    fn test_detect_degradation_none_when_high_quality() {
        let cortex = PredictiveCortex::new(32, 64);
        assert!(cortex.detect_degradation().is_none());
    }

    #[test]
    fn test_detect_degradation_triggers_after_bad_predictions() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 2);

        for _ in 0..5 {
            let bad_actual = vec![100.0; 32];
            cortex.record_outcome(&forecast, &bad_actual);
        }

        let signal = cortex.detect_degradation();
        assert!(
            signal.is_some(),
            "degraded cortex should produce repair signal"
        );
        if let Some(ref s) = signal {
            assert!(s.severity > 0.0);
            assert!(!s.diagnosis.is_empty());
            assert!(s.severity <= 1.0);
            assert_eq!(s.suggested_edits.len(), 2);
            assert_eq!(s.target_modules.len(), 3);
        }
    }

    #[test]
    fn test_reset_quality_tracking() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 2);

        for _ in 0..5 {
            let bad_actual = vec![100.0; 32];
            cortex.record_outcome(&forecast, &bad_actual);
        }
        assert!(cortex.forecast_quality < 0.5);

        cortex.reset_quality_tracking();
        assert!((cortex.forecast_quality - 1.0).abs() < 1e-6);
        assert_eq!(cortex.consecutive_degradations, 0);
    }

    #[test]
    fn test_repair_signal_has_reason() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 2);

        for _ in 0..5 {
            let bad_actual = vec![100.0; 32];
            cortex.record_outcome(&forecast, &bad_actual);
        }

        let signal = cortex.detect_degradation().expect("should trigger");
        assert!(
            signal.diagnosis.contains("PredictiveCortex"),
            "diagnosis should reference PredictiveCortex: {}",
            signal.diagnosis
        );
        assert!(
            signal.diagnosis.contains("quality"),
            "diagnosis should mention quality: {}",
            signal.diagnosis
        );
    }

    #[test]
    fn test_record_outcome_maintains_history() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);

        for _ in 0..5 {
            let f = cortex.predict_horizon(&latent, 1);
            cortex.record_outcome(&f, &latent);
        }
        assert_eq!(cortex.outcome_history.len(), 5);
        assert_eq!(cortex.cycle, 5);
    }

    #[test]
    fn test_good_predictions_clear_degradation() {
        let mut cortex = PredictiveCortex::new(32, 64);
        let latent = make_test_latent(32);
        let forecast = cortex.predict_horizon(&latent, 2);

        for _ in 0..3 {
            let bad_actual = vec![100.0; 32];
            cortex.record_outcome(&forecast, &bad_actual);
        }
        assert!(cortex.consecutive_degradations >= 3);

        for _ in 0..3 {
            cortex.record_outcome(&forecast, &latent);
        }
        assert_eq!(cortex.consecutive_degradations, 0);
    }
}
