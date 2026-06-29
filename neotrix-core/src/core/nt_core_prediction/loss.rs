use super::types::{JEPA_COV_WEIGHT, JEPA_INV_WEIGHT, JEPA_VARIANCE_TARGET, JEPA_VAR_WEIGHT};
use crate::neotrix::nt_core_signal::core::Vector;
use crate::neotrix::nt_core_signal::ops::cosine_similarity;
use serde::{Deserialize, Serialize};

/// VICReg loss for JEPA. Implemented but not yet wired (F2.5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VicRegLoss {
    pub var_weight: f64,
    pub inv_weight: f64,
    pub cov_weight: f64,
    pub variance_target: f64,
}

impl Default for VicRegLoss {
    fn default() -> Self {
        Self::new()
    }
}

impl VicRegLoss {
    pub fn new() -> Self {
        Self {
            var_weight: JEPA_VAR_WEIGHT,
            inv_weight: JEPA_INV_WEIGHT,
            cov_weight: JEPA_COV_WEIGHT,
            variance_target: JEPA_VARIANCE_TARGET,
        }
    }

    pub fn compute(&self, prediction: &[f64], target: &[f64]) -> (f64, f64, f64, f64) {
        let inv_loss = self.invariance_loss(prediction, target);
        let var_loss = self.variance_loss(prediction);
        let cov_loss = self.covariance_loss(prediction);
        let total =
            self.inv_weight * inv_loss + self.var_weight * var_loss + self.cov_weight * cov_loss;
        (total, inv_loss, var_loss, cov_loss)
    }

    fn invariance_loss(&self, pred: &[f64], target: &[f64]) -> f64 {
        let n = pred.len().min(target.len());
        let mse: f64 = pred
            .iter()
            .zip(target.iter())
            .take(n)
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>()
            / n as f64;
        mse
    }

    fn variance_loss(&self, z: &[f64]) -> f64 {
        let n = z.len() as f64;
        let mean = z.iter().sum::<f64>() / n;
        let std = (z.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
        (self.variance_target - std).max(0.0).powi(2)
    }

    fn covariance_loss(&self, z: &[f64]) -> f64 {
        let n = z.len();
        if n <= 1 {
            return 0.0;
        }
        let mean = z.iter().sum::<f64>() / n as f64;
        let centered: Vector = z.iter().map(|v| v - mean).collect();
        let mut off_diag_sq = 0.0;
        for i in 0..n {
            for j in (i + 1)..n {
                off_diag_sq += (centered[i] * centered[j]).powi(2);
            }
        }
        off_diag_sq / (n * (n - 1) / 2) as f64
    }
}

/// Energy model for JEPA prediction. Implemented but not yet wired (F2.5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyModel {
    pub metric: String,
    pub temperature: f64,
}

impl Default for EnergyModel {
    fn default() -> Self {
        Self::new()
    }
}

impl EnergyModel {
    pub fn new() -> Self {
        Self {
            metric: "l2".to_string(),
            temperature: 1.0,
        }
    }

    pub fn energy(&self, prediction: &[f64], target: &[f64]) -> f64 {
        let n = prediction.len().min(target.len());
        match self.metric.as_str() {
            "cosine" => {
                let sim = cosine_similarity(prediction, target);
                (-sim + 1.0) / self.temperature
            }
            _ => {
                let mse: f64 = prediction
                    .iter()
                    .zip(target.iter())
                    .take(n)
                    .map(|(p, t)| (p - t).powi(2))
                    .sum::<f64>()
                    / n as f64;
                mse / self.temperature
            }
        }
    }

    pub fn gaussian_regularizer(z: &[f64], target_std: f64) -> f64 {
        let n = z.len() as f64;
        let mean = z.iter().sum::<f64>() / n;
        let variance = z.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();
        (std - target_std).abs()
    }
}
