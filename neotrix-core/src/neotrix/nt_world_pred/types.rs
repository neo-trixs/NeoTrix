use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePrediction {
    pub predicted_latent: Vec<f64>,
    pub uncertainty: f64,
    pub plausible_states: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionInput {
    pub current_latent: Vec<f64>,
    pub action_encoding: Vec<f64>,
    pub context: Vec<f64>,
}

impl PredictionInput {
    pub fn new(current_latent: Vec<f64>, action_encoding: Vec<f64>, context: Vec<f64>) -> Self {
        Self { current_latent, action_encoding, context }
    }

    pub fn to_features(&self) -> Vec<f64> {
        let mut features = Vec::with_capacity(
            self.current_latent.len() + self.action_encoding.len() + self.context.len(),
        );
        features.extend_from_slice(&self.current_latent);
        features.extend_from_slice(&self.action_encoding);
        features.extend_from_slice(&self.context);
        features
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictorConfig {
    pub latent_dim: usize,
    pub action_dim: usize,
    pub context_dim: usize,
    pub ensemble_size: usize,
    pub uncertainty_threshold: f64,
}

impl Default for PredictorConfig {
    fn default() -> Self {
        Self {
            latent_dim: 32,
            action_dim: 8,
            context_dim: 16,
            ensemble_size: 5,
            uncertainty_threshold: 0.3,
        }
    }
}
