use serde::{Deserialize, Serialize};
use crate::neotrix::signal::core::Vector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaPredictor {
    pub latent_dim: usize,
    pub hidden_dim: usize,
    pub w1: Vec<Vec<f64>>,
    pub b1: Vec<f64>,
    pub w2: Vec<Vec<f64>>,
    pub b2: Vec<f64>,
}

impl JepaPredictor {
    pub fn new(latent_dim: usize, hidden_dim: usize) -> Self {
        let std1 = (2.0 / (latent_dim + hidden_dim) as f64).sqrt();
        let std2 = (2.0 / (hidden_dim + latent_dim) as f64).sqrt();

        let w1 = Self::random_matrix(hidden_dim, latent_dim, std1);
        let b1 = vec![0.0; hidden_dim];
        let w2 = Self::random_matrix(latent_dim, hidden_dim, std2);
        let b2 = vec![0.0; latent_dim];

        Self { latent_dim, hidden_dim, w1, b1, w2, b2 }
    }

    fn random_matrix(rows: usize, cols: usize, std: f64) -> Vec<Vec<f64>> {
        (0..rows).map(|_|
            (0..cols).map(|_| (rand::random::<f64>() - 0.5) * 2.0 * std).collect()
        ).collect()
    }

    pub fn predict(&self, z_context: &[f64]) -> Vector {
        let h: Vector = (0..self.hidden_dim).map(|i| {
            let mut sum = self.b1[i];
            for (j, &val) in z_context.iter().enumerate().take(self.latent_dim) {
                sum += self.w1[i][j] * val;
            }
            sum.tanh()
        }).collect();

        (0..self.latent_dim).map(|i| {
            let mut sum = self.b2[i];
            for (j, &val) in h.iter().enumerate().take(self.hidden_dim) {
                sum += self.w2[i][j] * val;
            }
            sum
        }).collect()
    }

    pub fn predict_with_uncertainty(&self, z_context: &[f64], n_samples: usize) -> (Vector, Vector) {
        let predictions: Vec<Vector> = (0..n_samples).map(|_| {
            let h: Vector = (0..self.hidden_dim).map(|i| {
                let mut sum = self.b1[i];
                for (j, &val) in z_context.iter().enumerate().take(self.latent_dim) {
                    sum += self.w1[i][j] * val;
                }
                if rand::random::<f64>() > 0.9 { sum = 0.0; }
                sum.tanh()
            }).collect();

            (0..self.latent_dim).map(|i| {
                let mut sum = self.b2[i];
                for (j, &val) in h.iter().enumerate().take(self.hidden_dim) {
                    sum += self.w2[i][j] * val;
                }
                sum
            }).collect()
        }).collect();

        let mean: Vector = (0..self.latent_dim).map(|i| {
            predictions.iter().map(|p| p[i]).sum::<f64>() / n_samples as f64
        }).collect();

        let variance: Vector = (0..self.latent_dim).map(|i| {
            let m = mean[i];
            predictions.iter().map(|p| {
                let d = p[i] - m;
                d * d
            }).sum::<f64>() / n_samples as f64
        }).collect();

        (mean, variance)
    }

    #[allow(clippy::needless_range_loop)]
    pub fn update(&mut self, z_context: &[f64], target: &[f64], lr: f64) {
        let prediction = self.predict(z_context);

        let h: Vector = (0..self.hidden_dim).map(|i| {
            let mut sum = self.b1[i];
            for (j, &val) in z_context.iter().enumerate().take(self.latent_dim) {
                sum += self.w1[i][j] * val;
            }
            sum
        }).collect();

        let output_error: Vector = (0..self.latent_dim).map(|i| {
            target[i] - prediction[i]
        }).collect();

        for i in 0..self.latent_dim {
            self.b2[i] += lr * output_error[i];
            for j in 0..self.hidden_dim {
                self.w2[i][j] += lr * output_error[i] * h[j].tanh();
            }
        }

        let hidden_error: Vector = (0..self.hidden_dim).map(|j| {
            let mut err = 0.0;
            for i in 0..self.latent_dim {
                err += output_error[i] * self.w2[i][j];
            }
            err * (1.0 - h[j].tanh().powi(2))
        }).collect();

        for i in 0..self.hidden_dim {
            self.b1[i] += lr * hidden_error[i];
            for j in 0..self.latent_dim {
                self.w1[i][j] += lr * hidden_error[i] * z_context[j];
            }
        }
    }
}
