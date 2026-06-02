use serde::{Deserialize, Serialize};
use crate::neotrix::signal::core::Vector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaEncoder {
    pub input_dim: usize,
    pub latent_dim: usize,
    pub weights: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
}

impl JepaEncoder {
    pub fn new(input_dim: usize, latent_dim: usize) -> Self {
        let mut weights = vec![vec![0.0; input_dim]; latent_dim];
        let bias = vec![0.0; latent_dim];

        let std = (2.0 / (input_dim + latent_dim) as f64).sqrt();
        for row in weights.iter_mut() {
            for val in row.iter_mut() {
                *val = (rand::random::<f64>() - 0.5) * 2.0 * std;
            }
        }

        Self { input_dim, latent_dim, weights, bias }
    }

    pub fn encode(&self, input: &[f64]) -> Vector {
        let mut z = vec![0.0; self.latent_dim];
        for (i, item) in z.iter_mut().enumerate() {
            let mut sum = self.bias[i];
            for (j, &val) in input.iter().enumerate().take(self.input_dim) {
                sum += self.weights[i][j] * val;
            }
            *item = sum.tanh();
        }
        z
    }

    pub fn ema_update(&mut self, source: &JepaEncoder, momentum: f64) {
        for i in 0..self.latent_dim {
            self.bias[i] = momentum * self.bias[i] + (1.0 - momentum) * source.bias[i];
            for j in 0..self.input_dim {
                self.weights[i][j] = momentum * self.weights[i][j] + (1.0 - momentum) * source.weights[i][j];
            }
        }
    }
}
