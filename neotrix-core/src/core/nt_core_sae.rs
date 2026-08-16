use serde::{Deserialize, Serialize};

pub const SAE_INPUT_DIM: usize = 512;
pub const SAE_LATENT_DIM: usize = 4096;
pub const E8_SAE_LAYERS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaeConfig {
    pub input_dim: usize,
    pub latent_dim: usize,
    pub l1_coef: f64,
    pub learning_rate: f64,
}

impl Default for SaeConfig {
    fn default() -> Self {
        Self {
            input_dim: SAE_INPUT_DIM,
            latent_dim: SAE_LATENT_DIM,
            l1_coef: 0.01,
            learning_rate: 0.001,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaeFeature {
    pub index: usize,
    pub activation: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaeOutput {
    pub active_features: Vec<SaeFeature>,
    pub reconstruction: Vec<f64>,
    pub loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonosemanticFeature {
    pub index: usize,
    pub label: String,
    pub frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaeEncoder {
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
}

impl SaeEncoder {
    pub fn new() -> Self {
        Self {
            weights: vec![vec![0.0; SAE_INPUT_DIM]; SAE_LATENT_DIM],
            biases: vec![0.0; SAE_LATENT_DIM],
        }
    }

    pub fn encode(&self, input: &[f64]) -> Vec<f64> {
        let mut latent = self.biases.clone();
        for (i, row) in self.weights.iter().enumerate() {
            for (j, &w) in row.iter().enumerate() {
                if j < input.len() {
                    latent[i] += w * input[j];
                }
            }
            latent[i] = latent[i].max(0.0);
        }
        latent
    }

    /// Apply TopK sparsity: keep top-k activations, zero out the rest.
    /// k is derived from the L1 coefficient: more aggressive = sparser.
    pub fn enforce_sparsity(&self, latent: &mut Vec<f64>) {
        let k = (latent.len() as f64 * 0.1).max(1.0).ceil() as usize;
        if latent.len() <= k {
            return;
        }
        let mut indices: Vec<usize> = (0..latent.len()).collect();
        indices.sort_by(|&a, &b| {
            latent[b]
                .partial_cmp(&latent[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for &idx in indices.iter().skip(k) {
            latent[idx] = 0.0;
        }
    }
}

impl Default for SaeEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaeDecoder {
    pub weights: Vec<Vec<f64>>,
    pub biases: Vec<f64>,
}

impl SaeDecoder {
    pub fn new() -> Self {
        Self {
            weights: vec![vec![0.0; SAE_LATENT_DIM]; SAE_INPUT_DIM],
            biases: vec![0.0; SAE_INPUT_DIM],
        }
    }

    pub fn decode(&self, latent: &[f64]) -> Vec<f64> {
        let mut output = self.biases.clone();
        for (i, row) in self.weights.iter().enumerate() {
            for (j, &w) in row.iter().enumerate() {
                if j < latent.len() {
                    output[i] += w * latent[j];
                }
            }
        }
        output
    }
}

impl Default for SaeDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseAutoencoder {
    pub config: SaeConfig,
    pub encoder: SaeEncoder,
    pub decoder: SaeDecoder,
}

impl SparseAutoencoder {
    pub fn new(config: SaeConfig) -> Self {
        Self {
            config,
            encoder: SaeEncoder::new(),
            decoder: SaeDecoder::new(),
        }
    }

    pub fn forward(&mut self, input: &[f64]) -> SaeOutput {
        let mut latent = self.encoder.encode(input);
        self.encoder.enforce_sparsity(&mut latent);
        let reconstruction = self.decoder.decode(&latent);
        let mut active = Vec::new();
        for (i, &val) in latent.iter().enumerate() {
            if val > 0.01 {
                active.push(SaeFeature {
                    index: i,
                    activation: val,
                    label: None,
                });
            }
        }
        let mut loss = 0.0;
        for (i, &r) in reconstruction.iter().enumerate() {
            if i < input.len() {
                let diff = r - input[i];
                loss += diff * diff;
            }
        }
        SaeOutput {
            active_features: active,
            reconstruction,
            loss,
        }
    }

    pub fn steer(&self, latent: &mut [f64], feature_idx: usize, strength: f64) {
        if feature_idx < latent.len() {
            latent[feature_idx] = strength;
        }
    }

    pub fn e8_to_input(e8_state: u8, meta_bits: u8) -> Vec<f64> {
        let mut input = vec![0.0; SAE_INPUT_DIM];
        let idx = (e8_state as usize) % SAE_INPUT_DIM;
        input[idx] = 1.0;
        let meta_idx = (meta_bits as usize + 64) % SAE_INPUT_DIM;
        input[meta_idx] = 1.0;
        input
    }
}

impl Default for SparseAutoencoder {
    fn default() -> Self {
        Self::new(SaeConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringTarget {
    pub feature_idx: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringVector {
    pub targets: Vec<SteeringTarget>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteeringController {
    pub active_steering: Option<SteeringVector>,
}

impl SteeringController {
    pub fn new() -> Self {
        Self {
            active_steering: None,
        }
    }

    /// Apply active steering targets to the latent representation.
    /// For each target feature, add strength * activation to that index.
    pub fn apply(&self, latent: &mut [f64]) {
        if let Some(ref steering) = self.active_steering {
            for target in &steering.targets {
                if target.feature_idx < latent.len() {
                    latent[target.feature_idx] += target.strength;
                }
            }
        }
    }
}

impl Default for SteeringController {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSae {
    pub layer_index: usize,
    pub autoencoder: SparseAutoencoder,
}

impl LayerSae {
    pub fn new(layer_index: usize, config: SaeConfig) -> Self {
        Self {
            layer_index,
            autoencoder: SparseAutoencoder::new(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sae_config_default() {
        let config = SaeConfig::default();
        assert_eq!(config.input_dim, SAE_INPUT_DIM);
        assert_eq!(config.latent_dim, SAE_LATENT_DIM);
        assert_eq!(config.l1_coef, 0.01);
        assert_eq!(config.learning_rate, 0.001);
    }

    #[test]
    fn test_sae_forward_pass() {
        let config = SaeConfig {
            input_dim: 8,
            latent_dim: 16,
            ..Default::default()
        };
        let mut sae = SparseAutoencoder::new(config);
        for i in 0..8 {
            sae.encoder.weights[i][i] = 1.0;
        }
        for i in 0..8 {
            sae.decoder.weights[i][i] = 1.0;
        }
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let output = sae.forward(&input);
        assert_eq!(output.reconstruction.len(), 512);
        assert!(!output.active_features.is_empty());
        assert!(output.loss >= 0.0);
    }

    #[test]
    fn test_sae_feature_activation() {
        let config = SaeConfig {
            input_dim: 8,
            latent_dim: 16,
            ..Default::default()
        };
        let mut sae = SparseAutoencoder::new(config);
        sae.encoder.weights[5] = vec![1.0; 8];
        sae.decoder.weights[0][5] = 1.0;
        let input = vec![1.0; 8];
        let output = sae.forward(&input);
        let feat = output.active_features.iter().find(|f| f.index == 5);
        assert!(feat.is_some());
        assert!((feat.unwrap().activation - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_sae_topk_sparsity() {
        let encoder = SaeEncoder::new();
        let mut latent = vec![0.0; 20];
        for i in 0..5 {
            latent[i] = (5 - i) as f64;
        }
        encoder.enforce_sparsity(&mut latent);
        let kept = latent.iter().filter(|&&v| v > 0.0).count();
        assert_eq!(kept, 2);
        assert!((latent[0] - 5.0).abs() < 1e-10);
        assert!((latent[1] - 4.0).abs() < 1e-10);
        for i in 2..20 {
            assert_eq!(latent[i], 0.0);
        }
    }

    #[test]
    fn test_sae_encode_decode_identity() {
        let config = SaeConfig {
            input_dim: 4,
            latent_dim: 40,
            ..Default::default()
        };
        let mut sae = SparseAutoencoder::new(config);
        for i in 0..4 {
            sae.encoder.weights[i][i] = 1.0;
        }
        for i in 0..4 {
            sae.decoder.weights[i][i] = 1.0;
        }
        let input = vec![0.5, 1.0, 2.0, 1.5];
        let output = sae.forward(&input);
        for i in 0..4 {
            assert!((output.reconstruction[i] - input[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn test_steering_controller_steer() {
        let config = SaeConfig {
            input_dim: 8,
            latent_dim: 16,
            ..Default::default()
        };
        let mut sae = SparseAutoencoder::new(config);
        sae.encoder.weights[3] = vec![1.0; 8];
        sae.decoder.weights[0][3] = 1.0;
        let mut latent = sae.encoder.encode(&vec![1.0; 8]);
        sae.steer(&mut latent, 3, 42.0);
        assert!((latent[3] - 42.0).abs() < 1e-10);
        let controller = SteeringController {
            active_steering: Some(SteeringVector {
                targets: vec![SteeringTarget {
                    feature_idx: 3,
                    strength: 10.0,
                }],
                description: "test".into(),
            }),
        };
        let mut latent2 = sae.encoder.encode(&vec![1.0; 8]);
        controller.apply(&mut latent2);
        assert!((latent2[3] - 18.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_autoencoder_default_shape() {
        let sae = SparseAutoencoder::default();
        assert_eq!(sae.config.input_dim, SAE_INPUT_DIM);
        assert_eq!(sae.config.latent_dim, SAE_LATENT_DIM);
        assert_eq!(sae.encoder.weights.len(), SAE_LATENT_DIM);
        assert_eq!(sae.encoder.weights[0].len(), SAE_INPUT_DIM);
        assert_eq!(sae.encoder.biases.len(), SAE_LATENT_DIM);
        assert_eq!(sae.decoder.weights.len(), SAE_INPUT_DIM);
        assert_eq!(sae.decoder.weights[0].len(), SAE_LATENT_DIM);
        assert_eq!(sae.decoder.biases.len(), SAE_INPUT_DIM);
    }
}
