use serde::{Deserialize, Serialize};
use crate::neotrix::nt_core_signal::core::Vector;
use super::predictor::JepaPredictor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDTarget {
    pub gamma: f64,
    pub n_step: usize,
}

impl TDTarget {
    pub fn new(gamma: f64, n_step: usize) -> Self {
        Self { gamma, n_step }
    }

    pub fn compute_return(&self, rewards: &[f64]) -> f64 {
        if rewards.is_empty() {
            return 0.0;
        }
        let n = rewards.len().min(self.n_step);
        let mut ret = 0.0;
        let mut discount = 1.0;
        for i in 0..n {
            ret += discount * rewards[i];
            discount *= self.gamma;
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDDynamics {
    pub latent_dim: usize,
    pub gamma: f64,
}

impl TDDynamics {
    pub fn new(latent_dim: usize, gamma: f64) -> Self {
        Self { latent_dim, gamma }
    }

    pub fn value(&self, z: &[f64], critic_weights: &[f64]) -> f64 {
        let n = z.len().min(critic_weights.len());
        z.iter().take(n).zip(critic_weights.iter()).map(|(a, b)| a * b).sum()
    }

    pub fn td_error(&self, reward: f64, z_t: &[f64], z_t_plus_n: &[f64], critic_weights: &[f64]) -> f64 {
        reward + self.gamma * self.value(z_t_plus_n, critic_weights) - self.value(z_t, critic_weights)
    }

    pub fn n_step_predict(
        &self,
        z_current: &[f64],
        predictor: &JepaPredictor,
        critic_weights: &[f64],
        n: usize,
    ) -> (Vector, f64, f64) {
        let mut z = z_current.to_vec();
        for _ in 0..n {
            z = predictor.predict(&z);
            for val in z.iter_mut() {
                *val = val.clamp(-10.0, 10.0);
            }
        }
        let v_current = self.value(z_current, critic_weights);
        let td_err = self.td_error(0.0, z_current, &z, critic_weights);
        (z, v_current, td_err)
    }

    pub fn update_critic(&self, z: &[f64], td_error: f64, lr: f64, critic_weights: &mut [f64]) {
        let n = z.len().min(critic_weights.len());
        for i in 0..n {
            critic_weights[i] += lr * td_error * z[i];
        }
    }
}
