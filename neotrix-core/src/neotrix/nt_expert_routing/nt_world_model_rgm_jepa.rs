use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// RGMScale — Renormalization Group coarse-graining operator.
///
/// Applies block-averaging at multiple scales to create multi-resolution
/// representations of a state vector. Inspired by Friston's Renormalization
/// Group approach to hierarchical state abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGMScale {
    pub max_scale: usize,
}

impl RGMScale {
    pub fn new(max_scale: usize) -> Self {
        Self {
            max_scale: max_scale.max(1),
        }
    }

    /// Coarse-grain a state vector at the given scale.
    /// scale=0: identity (no change)
    /// scale=1: block size 2, pairwise averaging
    /// scale=k: block size 2^k
    pub fn apply(&self, state: &[f64], scale: usize) -> Vec<f64> {
        if state.is_empty() || scale == 0 {
            return state.to_vec();
        }
        let block_size = 1 << scale.min(self.max_scale);
        let n = state.len();
        let out_dim = (n + block_size - 1) / block_size;
        let mut coarse = vec![0.0; out_dim];
        for i in 0..out_dim {
            let start = i * block_size;
            let end = (start + block_size).min(n);
            let count = end - start;
            let sum: f64 = state[start..end].iter().sum();
            coarse[i] = sum / count as f64;
        }
        coarse
    }

    /// Apply RGM at all scales from 0 to max_scale.
    pub fn apply_all(&self, state: &[f64]) -> Vec<Vec<f64>> {
        (0..=self.max_scale).map(|k| self.apply(state, k)).collect()
    }

    /// Nearest-neighbor upsample from coarse back to original dimension.
    pub fn upsample(&self, coarse: &[f64], scale: usize, original_dim: usize) -> Vec<f64> {
        if coarse.is_empty() || original_dim == 0 {
            return Vec::new();
        }
        if scale == 0 {
            return coarse.to_vec();
        }
        let block_size = 1 << scale.min(self.max_scale);
        let mut up = vec![0.0; original_dim];
        for i in 0..original_dim {
            let coarse_idx = i / block_size;
            if coarse_idx < coarse.len() {
                up[i] = coarse[coarse_idx];
            }
        }
        up
    }
}

/// CausalJEPA — Joint Embedding Predictive Architecture for next-state prediction.
///
/// A simple two-layer predictor that maps a latent state to a predicted next state:
///   h = tanh(W_in · z + b_in)
///   z_next = W_out · h + b_out
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalJEPA {
    pub input_dim: usize,
    pub hidden_dim: usize,
    pub w_in: Vec<Vec<f64>>,
    pub b_in: Vec<f64>,
    pub w_out: Vec<Vec<f64>>,
    pub b_out: Vec<f64>,
}

impl CausalJEPA {
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        let std_in = (2.0 / (input_dim + hidden_dim) as f64).sqrt();
        let std_out = (2.0 / (hidden_dim + input_dim) as f64).sqrt();

        let w_in = Self::random_matrix(hidden_dim, input_dim, std_in);
        let b_in = vec![0.0; hidden_dim];
        let w_out = Self::random_matrix(input_dim, hidden_dim, std_out);
        let b_out = vec![0.0; input_dim];

        Self {
            input_dim,
            hidden_dim,
            w_in,
            b_in,
            w_out,
            b_out,
        }
    }

    fn random_matrix(rows: usize, cols: usize, std: f64) -> Vec<Vec<f64>> {
        (0..rows)
            .map(|_| {
                (0..cols)
                    .map(|_| (rand::random::<f64>() - 0.5) * 2.0 * std)
                    .collect()
            })
            .collect()
    }

    /// Predict next latent state: z_next = W_out · tanh(W_in · z + b_in) + b_out
    pub fn predict(&self, state: &[f64]) -> Vec<f64> {
        let n = state.len().min(self.input_dim);

        let h: Vec<f64> = (0..self.hidden_dim)
            .map(|i| {
                let mut sum = self.b_in[i];
                for (j, &s) in state.iter().enumerate().take(n) {
                    sum += self.w_in[i][j] * s;
                }
                sum.tanh()
            })
            .collect();

        (0..n)
            .map(|i| {
                let mut sum = self.b_out[i];
                for (j, &h_val) in h.iter().enumerate().take(self.hidden_dim) {
                    sum += self.w_out[i][j] * h_val;
                }
                sum
            })
            .collect()
    }

    /// Project a float embedding to VSA binary vector via sign-threshold projection.
    pub fn project_to_vsa(embedding: &[f64]) -> [u8; 64] {
        let seed = embedding.iter().fold(0u64, |acc, &v| {
            let bits = v.to_bits();
            acc.wrapping_mul(31).wrapping_add(bits)
        });
        let bytes = QuantizedVSA::seeded_random(seed, 64);
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        arr
    }

    /// Update predictor weights using observed transition error.
    pub fn update(&mut self, state: &[f64], target: &[f64], lr: f64) {
        let prediction = self.predict(state);
        let n = state.len().min(self.input_dim);

        let h: Vec<f64> = (0..self.hidden_dim)
            .map(|i| {
                let mut sum = self.b_in[i];
                for (j, &s) in state.iter().enumerate().take(n) {
                    sum += self.w_in[i][j] * s;
                }
                sum
            })
            .collect();

        let output_error: Vec<f64> = (0..n).map(|i| target[i] - prediction[i]).collect();

        for i in 0..n {
            self.b_out[i] += lr * output_error[i];
            for j in 0..self.hidden_dim {
                self.w_out[i][j] += lr * output_error[i] * h[j].tanh();
            }
        }

        let hidden_error: Vec<f64> = (0..self.hidden_dim)
            .map(|j| {
                let mut err = 0.0;
                for i in 0..n {
                    err += output_error[i] * self.w_out[i][j];
                }
                err * (1.0 - h[j].tanh().powi(2))
            })
            .collect();

        for i in 0..self.hidden_dim {
            self.b_in[i] += lr * hidden_error[i];
            for j in 0..n {
                self.w_in[i][j] += lr * hidden_error[i] * state[j];
            }
        }
    }
}

// RGMJEPAFusion removed as F2.5 stub (zero callers)

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> Vec<f64> {
        vec![
            0.1, -0.2, 0.3, -0.1, 0.05, -0.05, 0.15, -0.25, 0.4, -0.1, 0.2, -0.3,
        ]
    }

    #[test]
    fn test_rgm_scale_identity() {
        let rgm = RGMScale::new(3);
        let state = sample_state();
        let out = rgm.apply(&state, 0);
        assert_eq!(out, state);
    }

    #[test]
    fn test_rgm_scale_block_average() {
        let rgm = RGMScale::new(3);
        let state = vec![2.0, 4.0, 6.0, 8.0]; // 4 elements
        let out = rgm.apply(&state, 1); // block_size=2 → 2 elements
        assert_eq!(out.len(), 2);
        assert!((out[0] - 3.0).abs() < 1e-10);
        assert!((out[1] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_rgm_upsample() {
        let rgm = RGMScale::new(2);
        let coarse = vec![3.0, 7.0];
        let up = rgm.upsample(&coarse, 1, 4);
        assert_eq!(up.len(), 4);
        assert!((up[0] - 3.0).abs() < 1e-10);
        assert!((up[1] - 3.0).abs() < 1e-10);
        assert!((up[2] - 7.0).abs() < 1e-10);
        assert!((up[3] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_causal_jepa_predict() {
        let jepa = CausalJEPA::new(8, 16);
        let state = sample_state()[..8].to_vec();
        let pred = jepa.predict(&state);
        assert_eq!(pred.len(), 8);
        for &v in &pred {
            assert!(v.is_finite());
        }
    }
}
