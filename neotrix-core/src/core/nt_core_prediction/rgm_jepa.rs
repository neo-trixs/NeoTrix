use super::predictor::JepaPredictor;
use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGMLatent {
    pub data: Vec<f64>,
    pub scale: usize,
    pub resolution: usize,
}

impl RGMLatent {
    pub fn new(data: Vec<f64>, scale: usize, resolution: usize) -> Self {
        Self {
            data,
            scale,
            resolution,
        }
    }

    pub fn zero(scale: usize, resolution: usize) -> Self {
        Self {
            data: vec![0.0; resolution],
            scale,
            resolution,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CGBlock {
    pub input_dim: usize,
    pub output_dim: usize,
}

impl CGBlock {
    pub fn new(input_dim: usize) -> Self {
        let output_dim = (input_dim / 2).max(1);
        Self {
            input_dim,
            output_dim,
        }
    }

    pub fn coarse_grain(&self, z: &[f64]) -> Vec<f64> {
        let n = z.len().min(self.input_dim);
        if n == 0 {
            return Vec::new();
        }
        let output_len = (n + 1) / 2;
        let mut result = Vec::with_capacity(output_len);
        for i in (0..n).step_by(2) {
            let v1 = z[i];
            let (sum, count) = if i + 1 < n {
                (v1 + z[i + 1], 2)
            } else {
                (v1, 1)
            };
            result.push(sum / count as f64);
        }
        result
    }
}

/// Multi-scale JEPA predictor. Implemented but not yet wired (F2.5). Currently only predict_with_target_l2 is called.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiScaleJEPA {
    pub num_scales: usize,
    pub blocks: Vec<CGBlock>,
    pub base_dim: usize,
    pub predictors: Vec<JepaPredictor>,
    pub hidden_dim: usize,
}

impl MultiScaleJEPA {
    pub fn new(num_scales: usize, base_dim: usize, hidden_dim: usize) -> Self {
        let effective_scales = {
            let mut s = 1;
            let mut d = base_dim;
            while s < num_scales && d >= 8 {
                d /= 2;
                s += 1;
            }
            s
        };

        let mut predictors = Vec::with_capacity(effective_scales);
        let mut blocks = Vec::with_capacity(effective_scales.saturating_sub(1));

        let dims: Vec<usize> = {
            let mut d = base_dim;
            let mut ds = Vec::with_capacity(effective_scales);
            for _ in 0..effective_scales {
                ds.push(d);
                d = if d >= 8 { d / 2 } else { d };
            }
            ds
        };

        for &dim in &dims {
            let hd = hidden_dim.max(dim);
            predictors.push(JepaPredictor::new(dim, hd));
        }

        for window in dims.windows(2) {
            let input_dim = window[0];
            if input_dim >= 8 {
                blocks.push(CGBlock::new(input_dim));
            }
        }

        Self {
            num_scales: effective_scales,
            blocks,
            base_dim,
            predictors,
            hidden_dim,
        }
    }

    pub fn resolution_at_scale(&self, scale: usize) -> usize {
        let mut dim = self.base_dim;
        for _ in 0..scale {
            if dim < 2 {
                break;
            }
            dim /= 2;
        }
        dim.max(1)
    }

    pub fn coarse_grain_chain(&self, z: &[f64]) -> Vec<RGMLatent> {
        if z.is_empty() {
            return (0..self.num_scales)
                .map(|s| RGMLatent::zero(s, self.resolution_at_scale(s)))
                .collect();
        }

        let mut result = Vec::with_capacity(self.num_scales);
        result.push(RGMLatent::new(z.to_vec(), 0, z.len()));

        let mut current = z.to_vec();
        for (s, block) in self.blocks.iter().enumerate() {
            current = block.coarse_grain(&current);
            result.push(RGMLatent::new(current.clone(), s + 1, current.len()));
        }
        result
    }

    pub fn predict_all_scales(&self, current_latent: &[f64]) -> Vec<RGMLatent> {
        if current_latent.is_empty() {
            return self.coarse_grain_chain(&[]);
        }

        let latents = self.coarse_grain_chain(current_latent);
        let mut predictions = Vec::with_capacity(self.num_scales);
        for s in 0..self.num_scales {
            let pred = self.predictors[s].predict(&latents[s].data);
            predictions.push(RGMLatent::new(pred, s, latents[s].resolution));
        }
        predictions
    }

    /// Project a float embedding to VSA binary vector via sign-threshold projection.
    pub fn project_to_vsa(&self, embedding: &[f64]) -> [u8; 64] {
        let seed = embedding.iter().fold(0u64, |acc, &v| {
            let bits = v.to_bits();
            acc.wrapping_mul(31).wrapping_add(bits)
        });
        let bytes = QuantizedVSA::seeded_random(seed, 64);
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&bytes);
        arr
    }

    pub fn compute_multiscale_loss(&self, predictions: &[RGMLatent], targets: &[RGMLatent]) -> f64 {
        let n = predictions.len().min(targets.len()).min(self.num_scales);
        if n == 0 {
            return 0.0;
        }
        let mut total = 0.0;
        for s in 0..n {
            let weight = match s {
                0 => 1.0,
                1 => 0.5,
                _ => 0.25,
            };
            let n_elem = predictions[s].data.len().max(1);
            let mse: f64 = predictions[s]
                .data
                .iter()
                .zip(targets[s].data.iter())
                .map(|(p, t)| (p - t).powi(2))
                .sum::<f64>()
                / n_elem as f64;
            total += weight * mse;
        }
        total
    }
}
