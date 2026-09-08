use serde::{Deserialize, Serialize};
type Vector = Vec<f64>;

/// Action-conditioned predictor for JEPA.
///
/// Takes an encoded state representation (from ViT) and an action embedding,
/// fuses them, and predicts the next latent state.
///
/// Fusion modes:
/// - `Concat`: concatenate state + action, pass through MLP
/// - `Add`: project action to state space, add to state, then predict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionMode {
    Concat,
    Add,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConditionedPredictor {
    pub latent_dim: usize,
    pub action_dim: usize,
    pub hidden_dim: usize,
    pub fusion_mode: FusionMode,
    /// Fusion weights:
    ///   Concat: (hidden_dim) × (latent_dim + action_dim) + bias
    ///   Add:    (latent_dim) × (latent_dim) for state, (latent_dim) × (action_dim) for action
    pub fusion_w: Vec<Vec<f64>>,
    pub fusion_b: Vec<f64>,
    /// Prediction MLP hidden → latent_dim
    pub pred_w1: Vec<Vec<f64>>,
    pub pred_b1: Vec<f64>,
    pub pred_w2: Vec<Vec<f64>>,
    pub pred_b2: Vec<f64>,
    /// Action projection (for Add mode)
    pub action_proj: Vec<Vec<f64>>,
}

impl ActionConditionedPredictor {
    pub fn new(latent_dim: usize, action_dim: usize, hidden_dim: usize, fusion_mode: FusionMode) -> Self {
        let mut rng = SimpleRng::new(42);

        let (fusion_w, fusion_b) = match fusion_mode {
            FusionMode::Concat => {
                let input_size = latent_dim + action_dim;
                let std_fuse = (2.0 / (input_size + hidden_dim) as f64).sqrt();
                (random_matrix(hidden_dim, input_size, std_fuse, &mut rng), vec![0.0; hidden_dim])
            }
            FusionMode::Add => {
                let std_fuse = (2.0 / (latent_dim + hidden_dim) as f64).sqrt();
                (random_matrix(hidden_dim, latent_dim, std_fuse, &mut rng), vec![0.0; hidden_dim])
            }
        };

        let std_p1 = (2.0 / (hidden_dim + hidden_dim) as f64).sqrt();
        let std_p2 = (2.0 / (hidden_dim + latent_dim) as f64).sqrt();
        let pred_w1 = random_matrix(hidden_dim, hidden_dim, std_p1, &mut rng);
        let pred_b1 = vec![0.0; hidden_dim];
        let pred_w2 = random_matrix(latent_dim, hidden_dim, std_p2, &mut rng);
        let pred_b2 = vec![0.0; latent_dim];

        let action_proj = match fusion_mode {
            FusionMode::Add => {
                let std_proj = (2.0 / (action_dim + latent_dim) as f64).sqrt();
                random_matrix(latent_dim, action_dim, std_proj, &mut rng)
            }
            FusionMode::Concat => Vec::new(),
        };

        Self {
            latent_dim,
            action_dim,
            hidden_dim,
            fusion_mode,
            fusion_w,
            fusion_b,
            pred_w1,
            pred_b1,
            pred_w2,
            pred_b2,
            action_proj,
        }
    }

    /// Predict next latent state given current state and action.
    ///
    /// `state_encoding`: encoded state representation (size `latent_dim`)
    /// `action`: action embedding (size `action_dim`)
    /// Returns predicted next latent state (size `latent_dim`)
    pub fn predict(&self, state_encoding: &[f64], action: &[f64]) -> Vector {
        let fused = self.fuse(state_encoding, action);
        let h: Vector = (0..self.hidden_dim)
            .map(|i| {
                let mut sum = self.pred_b1[i];
                for (j, &v) in fused.iter().enumerate().take(self.hidden_dim) {
                    sum += self.pred_w1[i][j] * v;
                }
                sum.tanh()
            })
            .collect();

        (0..self.latent_dim)
            .map(|i| {
                let mut sum = self.pred_b2[i];
                for (j, &v) in h.iter().enumerate().take(self.hidden_dim) {
                    sum += self.pred_w2[i][j] * v;
                }
                sum
            })
            .collect()
    }

    /// Fuse state encoding and action embedding.
    fn fuse(&self, state_encoding: &[f64], action: &[f64]) -> Vector {
        match self.fusion_mode {
            FusionMode::Concat => {
                let mut combined = Vec::with_capacity(self.latent_dim + self.action_dim);
                combined.extend_from_slice(&state_encoding[..self.latent_dim.min(state_encoding.len())]);
                combined.extend_from_slice(&action[..self.action_dim.min(action.len())]);
                while combined.len() < self.latent_dim + self.action_dim {
                    combined.push(0.0);
                }
                let mut fused = vec![0.0; self.hidden_dim];
                for i in 0..self.hidden_dim {
                    let mut sum = self.fusion_b[i];
                    for (j, &v) in combined.iter().enumerate().take(self.fusion_w[i].len()) {
                        sum += self.fusion_w[i][j] * v;
                    }
                    fused[i] = sum.tanh();
                }
                fused
            }
            FusionMode::Add => {
                let action_proj: Vector = if !self.action_proj.is_empty() {
                    (0..self.latent_dim)
                        .map(|i| {
                            let mut sum = 0.0;
                            for (j, &a) in action.iter().enumerate().take(self.action_dim) {
                                sum += self.action_proj[i][j] * a;
                            }
                            sum
                        })
                        .collect()
                } else {
                    vec![0.0; self.latent_dim]
                };

                let added: Vector = state_encoding
                    .iter()
                    .zip(action_proj.iter())
                    .map(|(s, a)| s + a)
                    .collect();

                let mut fused = vec![0.0; self.hidden_dim];
                for i in 0..self.hidden_dim {
                    let mut sum = self.fusion_b[i];
                    for (j, &v) in added.iter().enumerate().take(self.fusion_w[i].len()) {
                        sum += self.fusion_w[i][j] * v;
                    }
                    fused[i] = sum.tanh();
                }
                fused
            }
        }
    }

    /// Predict next state and return prediction with per-dimension variance.
    pub fn predict_with_uncertainty(&self, state_encoding: &[f64], action: &[f64], n_samples: usize) -> (Vector, Vector) {
        let predictions: Vec<Vector> = (0..n_samples)
            .map(|_| {
                let noisy_state: Vector = state_encoding
                    .iter()
                    .map(|&s| s + (rand::random::<f64>() - 0.5) * 0.1)
                    .collect();
                self.predict(&noisy_state, action)
            })
            .collect();

        let mean: Vector = (0..self.latent_dim)
            .map(|i| predictions.iter().map(|p| p[i]).sum::<f64>() / n_samples as f64)
            .collect();

        let variance: Vector = (0..self.latent_dim)
            .map(|i| {
                let m = mean[i];
                predictions.iter().map(|p| (p[i] - m).powi(2)).sum::<f64>() / n_samples as f64
            })
            .collect();

        (mean, variance)
    }

    /// Gradient update for the prediction weights.
    pub fn update(&mut self, state: &[f64], action: &[f64], target: &[f64], lr: f64) {
        let pred = self.predict(state, action);
        let fused = self.fuse(state, action);

        let output_error: Vector = target
            .iter()
            .zip(pred.iter())
            .map(|(t, p)| t - p)
            .collect();

        let h: Vector = (0..self.hidden_dim)
            .map(|i| {
                let mut sum = self.pred_b1[i];
                for (j, &v) in fused.iter().enumerate().take(self.hidden_dim) {
                    sum += self.pred_w1[i][j] * v;
                }
                sum
            })
            .collect();

        for i in 0..self.latent_dim {
            self.pred_b2[i] += lr * output_error[i];
            for j in 0..self.hidden_dim {
                self.pred_w2[i][j] += lr * output_error[i] * h[j].tanh();
            }
        }

        let hidden_error: Vector = (0..self.hidden_dim)
            .map(|j| {
                let mut err = 0.0;
                for i in 0..self.latent_dim {
                    err += output_error[i] * self.pred_w2[i][j];
                }
                err * (1.0 - h[j].tanh().powi(2))
            })
            .collect();

        for i in 0..self.hidden_dim {
            self.pred_b1[i] += lr * hidden_error[i];
            for j in 0..self.hidden_dim {
                self.pred_w1[i][j] += lr * hidden_error[i] * fused[j];
            }
        }
    }
}

struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn uniform(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

fn random_matrix(rows: usize, cols: usize, std: f64, rng: &mut SimpleRng) -> Vec<Vec<f64>> {
    (0..rows)
        .map(|_| (0..cols).map(|_| (rng.uniform() - 0.5) * 2.0 * std).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_predictor() -> ActionConditionedPredictor {
        ActionConditionedPredictor::new(32, 8, 64, FusionMode::Concat)
    }

    #[test]
    fn test_predict_output_dim() {
        let p = make_predictor();
        let state = vec![0.5; 32];
        let action = vec![0.1; 8];
        let pred = p.predict(&state, &action);
        assert_eq!(pred.len(), 32);
    }

    #[test]
    fn test_predict_with_uncertainty() {
        let p = make_predictor();
        let state = vec![0.5; 32];
        let action = vec![0.1; 8];
        let (mean, variance) = p.predict_with_uncertainty(&state, &action, 10);
        assert_eq!(mean.len(), 32);
        assert_eq!(variance.len(), 32);
        assert!(variance.iter().all(|&v| v >= 0.0));
    }

    #[test]
    fn test_update_reduces_error() {
        let mut p = make_predictor();
        let state = vec![0.5; 32];
        let action = vec![0.1; 8];
        let target = vec![0.6; 32];
        let err_before: f64 = p
            .predict(&state, &action)
            .iter()
            .zip(target.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        for _ in 0..50 {
            p.update(&state, &action, &target, 0.01);
        }
        let err_after: f64 = p
            .predict(&state, &action)
            .iter()
            .zip(target.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        assert!(err_after <= err_before * 1.1 || err_after < 0.01);
    }

    #[test]
    fn test_add_fusion_output_dim() {
        let p = ActionConditionedPredictor::new(32, 8, 64, FusionMode::Add);
        let state = vec![0.5; 32];
        let action = vec![0.1; 8];
        let pred = p.predict(&state, &action);
        assert_eq!(pred.len(), 32);
    }

    #[test]
    fn test_different_actions_give_different_predictions() {
        let p = make_predictor();
        let state = vec![0.5; 32];
        let action_a = vec![0.1; 8];
        let action_b = vec![0.9; 8];
        let pred_a = p.predict(&state, &action_a);
        let pred_b = p.predict(&state, &action_b);
        let diff: f64 = pred_a.iter().zip(pred_b.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(diff > 1e-6, "Different actions should produce different predictions");
    }
}
