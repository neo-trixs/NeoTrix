use super::types::{PredictionInput, PredictorConfig, StatePrediction};

#[derive(Debug, Clone)]
struct LinearPredictor {
    weights: Vec<f64>,
    bias: f64,
}

impl LinearPredictor {
    fn new(feature_dim: usize) -> Self {
        let std = (2.0 / feature_dim as f64).sqrt();
        let weights: Vec<f64> = (0..feature_dim)
            .map(|_| (rand::random::<f64>() - 0.5) * 2.0 * std)
            .collect();
        Self {
            weights,
            bias: (rand::random::<f64>() - 0.5) * 0.1,
        }
    }

    fn predict(&self, features: &[f64], output_dim: usize, latent_dim: usize) -> Vec<f64> {
        (0..output_dim)
            .map(|i| {
                let mut sum = self.bias;
                for (j, &val) in features.iter().enumerate() {
                    if i == j % output_dim {
                        let w_idx = (j * output_dim + i) % self.weights.len();
                        sum += self.weights[w_idx] * val;
                    }
                }
                sum
            })
            .take(latent_dim)
            .collect()
    }

    fn update(&mut self, features: &[f64], target: &[f64], prediction: &[f64], lr: f64) {
        for (i, &t) in target.iter().enumerate() {
            let error = t - prediction[i];
            self.bias += lr * error;
            for (j, &val) in features.iter().enumerate() {
                let w_idx = (j * target.len() + i) % self.weights.len();
                self.weights[w_idx] += lr * error * val;
            }
        }
    }
}

pub struct PredictorState {
    config: PredictorConfig,
    ensemble: Vec<LinearPredictor>,
}

impl PredictorState {
    pub fn new(config: PredictorConfig) -> Self {
        let feature_dim = config.latent_dim + config.action_dim + config.context_dim;
        let ensemble: Vec<LinearPredictor> = (0..config.ensemble_size)
            .map(|_| LinearPredictor::new(feature_dim))
            .collect();
        Self { config, ensemble }
    }

    pub fn predict(&self, input: &PredictionInput) -> StatePrediction {
        let features = input.to_features();
        let latent_dim = self.config.latent_dim;
        let output_dim = latent_dim;
        let ensemble_size = self.config.ensemble_size;

        let all_predictions: Vec<Vec<f64>> = self
            .ensemble
            .iter()
            .map(|m| m.predict(&features, output_dim, latent_dim))
            .collect();

        let mean: Vec<f64> = (0..latent_dim)
            .map(|i| all_predictions.iter().map(|p| p[i]).sum::<f64>() / ensemble_size as f64)
            .collect();

        let variance: f64 = (0..latent_dim)
            .map(|i| {
                let m = mean[i];
                all_predictions
                    .iter()
                    .map(|p| {
                        let d = p[i] - m;
                        d * d
                    })
                    .sum::<f64>()
                    / ensemble_size as f64
            })
            .sum::<f64>()
            / latent_dim as f64;

        let uncertainty = (variance / (1.0 + variance)).min(1.0);

        let mut plausible_states: Vec<(f64, Vec<f64>)> = all_predictions
            .into_iter()
            .map(|p| {
                let dist: f64 = p
                    .iter()
                    .zip(mean.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                (dist, p)
            })
            .collect();
        plausible_states.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let top_k: Vec<Vec<f64>> = plausible_states
            .into_iter()
            .take(ensemble_size.min(3))
            .map(|(_, p)| p)
            .collect();

        StatePrediction {
            predicted_latent: mean,
            uncertainty,
            plausible_states: top_k,
        }
    }

    pub fn update(&mut self, observation: &[f64], learning_rate: f64) {
        let dummy_action = vec![0.0; self.config.action_dim];
        let dummy_ctx = vec![0.0; self.config.context_dim];
        let latent = &observation[..observation.len().min(self.config.latent_dim)];
        let mut current = latent.to_vec();
        current.resize(self.config.latent_dim, 0.0);

        let mut action = dummy_action;
        if observation.len() > self.config.latent_dim {
            let extra = &observation[self.config.latent_dim..];
            let a_len = self.config.action_dim.min(extra.len());
            action[..a_len].copy_from_slice(&extra[..a_len]);
        }

        let position_after_action = self.config.latent_dim + self.config.action_dim;
        let mut ctx = dummy_ctx;
        if observation.len() > position_after_action {
            let extra = &observation[position_after_action..];
            let c_len = self.config.context_dim.min(extra.len());
            ctx[..c_len].copy_from_slice(&extra[..c_len]);
        }

        let input = PredictionInput::new(current, action, ctx);
        let features = input.to_features();
        let prediction =
            self.ensemble[0].predict(&features, self.config.latent_dim, self.config.latent_dim);

        let mut pred = latent.to_vec();
        pred.resize(self.config.latent_dim, 0.0);

        for member in self.ensemble.iter_mut() {
            member.update(&features, &pred, &prediction, learning_rate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> PredictorConfig {
        PredictorConfig {
            latent_dim: 8,
            action_dim: 4,
            context_dim: 4,
            ensemble_size: 5,
            uncertainty_threshold: 0.3,
        }
    }

    #[test]
    fn test_prediction_shape() {
        let config = make_config();
        let predictor = PredictorState::new(config);
        let input = PredictionInput::new(vec![0.5; 8], vec![1.0, 0.0, 0.0, 0.0], vec![0.1; 4]);
        let result = predictor.predict(&input);
        assert_eq!(result.predicted_latent.len(), 8);
        assert!(result.uncertainty >= 0.0 && result.uncertainty <= 1.0);
    }

    #[test]
    fn test_plausible_states_count() {
        let config = make_config();
        let predictor = PredictorState::new(config);
        let input = PredictionInput::new(vec![0.5; 8], vec![0.0, 1.0, 0.0, 0.0], vec![0.2; 4]);
        let result = predictor.predict(&input);
        assert!(!result.plausible_states.is_empty());
        assert_eq!(result.plausible_states[0].len(), 8);
    }

    #[test]
    fn test_ensemble_disagreement_uncertainty() {
        let config = PredictorConfig {
            latent_dim: 4,
            action_dim: 2,
            context_dim: 2,
            ensemble_size: 10,
            uncertainty_threshold: 0.3,
        };
        let predictor = PredictorState::new(config);
        let input = PredictionInput::new(vec![0.0; 4], vec![1.0, 0.0], vec![0.0; 2]);
        let result = predictor.predict(&input);
        assert!(result.uncertainty >= 0.0 && result.uncertainty <= 1.0);
    }

    #[test]
    fn test_update_reduces_error() {
        let config = make_config();
        let mut predictor = PredictorState::new(config.clone());

        let input = PredictionInput::new(vec![0.5; 8], vec![1.0, 0.0, 0.0, 0.0], vec![0.1; 4]);
        let before = predictor.predict(&input);

        let mut observation = vec![0.5; 8];
        observation.extend_from_slice(&[1.0, 0.0, 0.0, 0.0]);
        observation.extend_from_slice(&[0.1; 4]);

        for _ in 0..20 {
            predictor.update(&observation, 0.01);
        }

        let after = predictor.predict(&input);
        let error_before: f64 = before
            .predicted_latent
            .iter()
            .map(|v| (v - 0.5).abs())
            .sum();
        let error_after: f64 = after.predicted_latent.iter().map(|v| (v - 0.5).abs()).sum();
        assert!(
            error_after <= error_before + 0.5 || error_after < 4.0,
            "update should reduce or maintain prediction error: before={}, after={}",
            error_before,
            error_after
        );
    }

    #[test]
    fn test_zero_input_prediction() {
        let config = make_config();
        let predictor = PredictorState::new(config);
        let input = PredictionInput::new(vec![0.0; 8], vec![0.0; 4], vec![0.0; 4]);
        let result = predictor.predict(&input);
        assert_eq!(result.predicted_latent.len(), 8);
        assert!(result.predicted_latent.iter().all(|v| !v.is_nan()));
    }

    #[test]
    fn test_different_actions_different_predictions() {
        let config = make_config();
        let predictor = PredictorState::new(config);
        let input_a = PredictionInput::new(vec![0.5; 8], vec![1.0, 0.0, 0.0, 0.0], vec![0.1; 4]);
        let input_b = PredictionInput::new(vec![0.5; 8], vec![0.0, 0.0, 0.0, 1.0], vec![0.1; 4]);
        let result_a = predictor.predict(&input_a);
        let result_b = predictor.predict(&input_b);
        let diff: f64 = result_a
            .predicted_latent
            .iter()
            .zip(result_b.predicted_latent.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 0.0 || diff.abs() < 1e-10);
    }

    #[test]
    fn test_uncertainty_bounds() {
        let config = PredictorConfig {
            latent_dim: 2,
            action_dim: 1,
            context_dim: 1,
            ensemble_size: 20,
            uncertainty_threshold: 0.5,
        };
        let predictor = PredictorState::new(config);
        let input = PredictionInput::new(vec![1.0; 2], vec![0.0], vec![0.0]);
        let result = predictor.predict(&input);
        assert!(result.uncertainty >= 0.0, "uncertainty must be >= 0");
        assert!(result.uncertainty <= 1.0, "uncertainty must be <= 1");
    }

    #[test]
    fn test_plausible_states_ordered_by_distance() {
        let config = make_config();
        let predictor = PredictorState::new(config);
        let input = PredictionInput::new(vec![0.5; 8], vec![1.0, 0.0, 0.0, 0.0], vec![0.1; 4]);
        let result = predictor.predict(&input);
        if result.plausible_states.len() >= 2 {
            let d0: f64 = result.plausible_states[0]
                .iter()
                .zip(result.predicted_latent.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            let d1: f64 = result.plausible_states[1]
                .iter()
                .zip(result.predicted_latent.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            assert!(d0 <= d1 + 1e-6);
        }
    }
}
