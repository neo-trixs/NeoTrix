use serde::{Deserialize, Serialize};
type Vector = Vec<f64>;

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum();
    let norm_b: f64 = b.iter().map(|x| x * x).sum();
    if norm_a == 0.0 && norm_b == 0.0 { 1.0 } else { dot / (norm_a.sqrt() * norm_b.sqrt()).max(1e-10) }
}
use super::types::{
    JEPA_VAR_WEIGHT, JEPA_INV_WEIGHT, JEPA_COV_WEIGHT,
    JEPA_VARIANCE_TARGET, JEPA_LATENT_DIM,
};

/// Loss function selection for JEPA world model training.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum JepaLossType {
    /// VICReg: invariance + variance + covariance (default)
    #[default]
    VicReg,
    /// Non-contrastive: LevL-style bidirectional prediction loss
    NonContrastive,
}

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
        let total = self.inv_weight * inv_loss
                  + self.var_weight * var_loss
                  + self.cov_weight * cov_loss;
        (total, inv_loss, var_loss, cov_loss)
    }

    fn invariance_loss(&self, pred: &[f64], target: &[f64]) -> f64 {
        let n = pred.len().min(target.len());
        if n == 0 { return 0.0; }
        let mse: f64 = pred.iter().zip(target.iter())
            .take(n)
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>() / n as f64;
        mse
    }

    fn variance_loss(&self, z: &[f64]) -> f64 {
        let n = z.len() as f64;
        if n == 0.0 { return self.variance_target.powi(2); }
        let mean = z.iter().sum::<f64>() / n;
        let std = (z.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();
        (self.variance_target - std).max(0.0).powi(2)
    }

    fn covariance_loss(&self, z: &[f64]) -> f64 {
        let n = z.len();
        if n <= 1 { return 0.0; }
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
                if n == 0 { return 0.0; }
                let mse: f64 = prediction.iter().zip(target.iter())
                    .take(n)
                    .map(|(p, t)| (p - t).powi(2))
                    .sum::<f64>() / n as f64;
                mse / self.temperature
            }
        }
    }

    pub fn gaussian_regularizer(z: &[f64], target_std: f64) -> f64 {
        let n = z.len() as f64;
        if n == 0.0 { return target_std.abs(); }
        let mean = z.iter().sum::<f64>() / n;
        let variance = z.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();
        (std - target_std).abs()
    }
}

/// Non-contrastive cross-modal prediction loss (LeVLJEPA, arXiv 2607.00784).
///
/// Key innovations over contrastive VICReg:
/// 1. No negatives — no contrastive pairs needed
/// 2. No temperature — no tunable scaling parameter
/// 3. No momentum encoder — stop-gradient on target is sufficient
/// 4. Cross-modal prediction with per-modality distributional regularization
/// 5. Produces dense semantic features (not pooled embeddings)
///
/// L = ||pred - sg(target)||² + λ_reg * (Σ D_reg(m))
///   where D_reg(m) = distributional regularization per modality m
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonContrastiveLoss {
    /// Weight for distributional regularization (default: 0.1)
    pub reg_weight: f64,
    /// Whether to apply stop-gradient on target (default: true)
    pub stop_gradient: bool,
    /// Variance target for per-modality regularization (default: 0.5)
    pub modality_variance_target: f64,
    /// Dimensionality of the latent space
    pub latent_dim: usize,
}

impl Default for NonContrastiveLoss {
    fn default() -> Self {
        Self {
            reg_weight: 0.1,
            stop_gradient: true,
            modality_variance_target: 0.5,
            latent_dim: JEPA_LATENT_DIM,
        }
    }
}

impl NonContrastiveLoss {
    pub fn new() -> Self { Self::default() }

    /// Compute non-contrastive cross-modal prediction loss.
    ///
    /// Args:
    ///   prediction: encoder output for modality A (predictor input)
    ///   target: encoder output for modality B (target, with optional stop-grad)
    ///
    /// Returns (total_loss, prediction_mse, regularization_loss)
    pub fn compute(&self, prediction: &[f64], target: &[f64]) -> (f64, f64, f64) {
        // Cross-modal prediction MSE (no negatives, no temperature)
        let n = prediction.len().min(target.len());
        let pred_mse: f64 = prediction.iter().zip(target.iter())
            .take(n)
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f64>() / n.max(1) as f64;

        // Per-modality distributional regularization
        // Encourages dense semantic features within each modality
        let reg = self.distributional_regularization(prediction)
            + self.distributional_regularization(target);

        let total = pred_mse + self.reg_weight * reg;
        (total, pred_mse, reg)
    }

    /// Distributional regularization: encourages feature diversity within
    /// a modality. Computed as variance penalty + covariance off-diagonal.
    fn distributional_regularization(&self, z: &[f64]) -> f64 {
        if z.is_empty() { return 0.0; }
        let n = z.len() as f64;
        let mean = z.iter().sum::<f64>() / n;
        let variance = z.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();

        // Variance penalty: push std toward target
        let var_loss = (self.modality_variance_target - std).max(0.0).powi(2);

        // Coarse covariance off-diagonal (dense version using feature chunks)
        let chunk_size = (z.len() / 4).max(1);
        let mut cov_off_diag = 0.0;
        let mut cov_count = 0;
        for i in 0..z.len().saturating_sub(chunk_size) {
            for j in (i + chunk_size..z.len()).step_by(chunk_size) {
                cov_off_diag += ((z[i] - mean) * (z[j] - mean)).powi(2);
                cov_count += 1;
            }
        }
        let cov_loss = if cov_count > 0 { cov_off_diag / cov_count as f64 } else { 0.0 };

        var_loss + 0.5 * cov_loss
    }
}

/// LeVLJEPA end-to-end loss: cross-modal prediction without negatives.
///
/// Designed for vision-language pretraining where two modalities
/// (e.g., image patches and text tokens) are aligned in latent space
/// without requiring negative pairs, temperature scaling, or momentum encoders.
pub fn levl_jepa_loss(
    vision_prediction: &[f64],
    text_prediction: &[f64],
    vision_target: &[f64],
    text_target: &[f64],
    config: &NonContrastiveLoss,
) -> (f64, f64, f64) {
    // Cross-modal predictions
    let (v_loss, v_mse, v_reg) = config.compute(vision_prediction, vision_target);
    let (t_loss, t_mse, t_reg) = config.compute(text_prediction, text_target);

    // Bi-directional: vision→text + text→vision
    let (vt_loss, vt_mse, vt_reg) = config.compute(vision_prediction, text_target);
    let (tv_loss, tv_mse, tv_reg) = config.compute(text_prediction, vision_target);

    let total = v_loss + t_loss + 0.5 * (vt_loss + tv_loss);
    let avg_mse = (v_mse + t_mse + vt_mse + tv_mse) / 4.0;
    let avg_reg = (v_reg + t_reg + vt_reg + tv_reg) / 4.0;

    (total, avg_mse, avg_reg)
}

#[cfg(test)]
mod non_contrastive_tests {
    use super::*;

    fn make_vec(val: f64, len: usize) -> Vec<f64> {
        (0..len).map(|i| val + (i as f64 * 0.01)).collect()
    }

    #[test]
    fn test_non_contrastive_loss_defaults() {
        let loss = NonContrastiveLoss::default();
        assert!((loss.reg_weight - 0.1).abs() < 1e-9);
        assert!(loss.stop_gradient);
        assert_eq!(loss.latent_dim, JEPA_LATENT_DIM);
    }

    #[test]
    fn test_non_contrastive_loss_identical() {
        let loss = NonContrastiveLoss::default();
        let v = make_vec(0.5, 64);
        let (total, mse, reg) = loss.compute(&v, &v);
        assert!(mse < 0.01, "identical vectors should have near-zero mse: {mse}");
        assert!(total >= 0.0);
        assert!(reg >= 0.0);
    }

    #[test]
    fn test_non_contrastive_loss_different() {
        let loss = NonContrastiveLoss::default();
        let a = make_vec(0.0, 64);
        let b = make_vec(1.0, 64);
        let (total, mse, _) = loss.compute(&a, &b);
        assert!(mse > 0.5);
        assert!(total > 0.0);
    }

    #[test]
    fn test_levl_jepa_loss_bidirectional() {
        let config = NonContrastiveLoss::default();
        let v_pred = make_vec(0.1, 64);
        let t_pred = make_vec(0.2, 64);
        let v_targ = make_vec(0.15, 64);
        let t_targ = make_vec(0.25, 64);

        let (total, mse, reg) = levl_jepa_loss(&v_pred, &t_pred, &v_targ, &t_targ, &config);
        assert!(total >= 0.0);
        assert!(mse >= 0.0);
        assert!(reg >= 0.0);
    }

    #[test]
    fn test_distributional_regularization_penalizes_low_variance() {
        let loss = NonContrastiveLoss::default();
        let constant = vec![0.5; 64];
        let varied = make_vec(0.0, 64);
        let const_reg = loss.distributional_regularization(&constant);
        let var_reg = loss.distributional_regularization(&varied);
        assert!(
            const_reg > var_reg,
            "constant input should have higher regularization penalty"
        );
    }
}
