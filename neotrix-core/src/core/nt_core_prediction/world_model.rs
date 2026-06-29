use super::encoder::JepaEncoder;
use super::loss::{EnergyModel, VicRegLoss};
use super::predictor::JepaPredictor;
use super::rgm_jepa::{CGBlock, MultiScaleJEPA};
use super::td_jepa::TDDynamics;
use super::types::{
    LatentState, MultiScalePrediction, TDExperience, WorldModelState, JEPA_EMA_MOMENTUM,
    JEPA_GAUSS_STD_TARGET, JEPA_GAUSS_WEIGHT, JEPA_HIDDEN_DIM, JEPA_LATENT_DIM, JEPA_LEARNING_RATE,
};
use crate::core::nt_core_sigreg::{HermiteSpectralMonitor, IdentifiabilityReport, SpectralHealth};
use crate::core::nt_core_td::{TDFlowsConfig, TemporalDifferenceFlows};
use crate::neotrix::nt_core_signal::attribution::SIGReg;
use crate::neotrix::nt_core_signal::core::Vector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaWorldModel {
    pub latent_dim: usize,
    pub context_encoder: JepaEncoder,
    pub target_encoder: JepaEncoder,
    pub predictor: JepaPredictor,
    pub energy_model: EnergyModel,
    pub vicreg: VicRegLoss,
    pub learning_rate: f64,
    pub momentum: f64,
    pub training_steps: usize,
    pub td_target_critic: Vector,
    pub td_gamma: f64,
    pub td_n_step: usize,
    pub num_rgm_scales: usize,
    pub rgm_blocks: Vec<CGBlock>,
    pub multiscale_jepa: MultiScaleJEPA,
    pub sigreg: Option<SIGReg>,
    pub sigreg_lambda: f64,
    pub td_flows: Option<TemporalDifferenceFlows>,
    #[serde(skip)]
    pub spectral_monitor: Option<HermiteSpectralMonitor>,
}

impl JepaWorldModel {
    pub fn new(input_dim: usize) -> Self {
        let context_encoder = JepaEncoder::new(input_dim, JEPA_LATENT_DIM);
        let target_encoder = context_encoder.clone();
        let multiscale_jepa = MultiScaleJEPA::new(1, JEPA_LATENT_DIM, JEPA_HIDDEN_DIM);

        Self {
            latent_dim: JEPA_LATENT_DIM,
            context_encoder,
            target_encoder,
            predictor: JepaPredictor::new(JEPA_LATENT_DIM, JEPA_HIDDEN_DIM),
            energy_model: EnergyModel::new(),
            vicreg: VicRegLoss::new(),
            learning_rate: JEPA_LEARNING_RATE,
            momentum: JEPA_EMA_MOMENTUM,
            training_steps: 0,
            td_target_critic: (0..JEPA_LATENT_DIM)
                .map(|_| (rand::random::<f64>() - 0.5) * 0.1)
                .collect(),
            td_gamma: 0.95,
            td_n_step: 10,
            num_rgm_scales: 1,
            rgm_blocks: Vec::new(),
            multiscale_jepa,
            sigreg: None,
            sigreg_lambda: 0.01,
            td_flows: None,
            spectral_monitor: None,
        }
    }

    pub fn with_rgm_scales(mut self, num_scales: usize) -> Self {
        let ms = MultiScaleJEPA::new(num_scales, self.latent_dim, JEPA_HIDDEN_DIM);
        self.num_rgm_scales = ms.num_scales;
        self.rgm_blocks = ms.blocks.clone();
        self.multiscale_jepa = ms;
        self
    }

    pub fn with_sigreg(mut self, num_projections: usize, lambda: f64) -> Self {
        if num_projections > 0 {
            self.sigreg = Some(SIGReg::new(num_projections, lambda, self.latent_dim));
        }
        self.sigreg_lambda = lambda;
        self
    }

    pub fn with_td_flows(mut self, config: TDFlowsConfig) -> Self {
        self.td_flows = Some(TemporalDifferenceFlows::new(config, self.latent_dim));
        self
    }

    /// Enable spectral monitoring with a HermiteSpectralMonitor of the
    /// given window size.
    pub fn with_spectral_monitor(mut self, window_size: usize) -> Self {
        self.spectral_monitor = Some(HermiteSpectralMonitor::new(window_size, 4, self.latent_dim));
        self
    }

    /// Record a reward or prediction-error value as a "gradient" proxy
    /// for spectral monitoring.
    pub fn record_rollout_reward(&mut self, step: usize, reward: f64) {
        if let Some(ref mut monitor) = self.spectral_monitor {
            let pseudo_grad: Vec<f64> = (0..self.latent_dim)
                .map(|i| reward * (1.0 + 0.01 * (i as f64)))
                .collect();
            monitor.record_gradient(step, &pseudo_grad);
        }
    }

    /// Return the current spectral health of the world model's gradient
    /// dynamics.
    pub fn spectral_health(&self) -> Option<SpectralHealth> {
        self.spectral_monitor.as_ref().map(|m| m.spectral_health())
    }

    /// Check rollout stability by comparing spectral health against an
    /// acceptable threshold.  Returns `true` if the model is stable
    /// (Healthy) or has insufficient data.
    pub fn check_rollout_stability(&self) -> bool {
        match self.spectral_health() {
            None => true,
            Some(SpectralHealth::Healthy) => true,
            Some(SpectralHealth::Degrading) => false,
            Some(SpectralHealth::Collapsed) => false,
        }
    }

    /// Generate an identifiability report from the spectral monitor.
    pub fn identifiability_report(&self) -> Option<IdentifiabilityReport> {
        self.spectral_monitor
            .as_ref()
            .map(IdentifiabilityReport::from_monitor)
    }

    pub fn predict(&self, context_features: &[f64]) -> (Vector, f64) {
        let z_context = self.context_encoder.encode(context_features);
        let z_pred = self.predictor.predict(&z_context);
        let energy = self.energy_model.energy(&z_pred, &z_context);
        (z_pred, energy)
    }

    pub fn predict_multi_scale(
        &self,
        context_features: &[f64],
        horizon: usize,
    ) -> MultiScalePrediction {
        let z_current = self.context_encoder.encode(context_features);

        let short_term = self.predictor.predict(&z_current);

        let mut medium_term = Vec::with_capacity(horizon);
        let mut uncertainties = Vec::with_capacity(horizon);
        let mut z = z_current.clone();

        for _step in 0..horizon {
            let (pred, variance) = self.predictor.predict_with_uncertainty(&z, 5);
            medium_term.push(pred.clone());
            uncertainties.push(variance.iter().sum::<f64>() / variance.len() as f64);
            z = pred;
        }

        let long_term_trend: Vector = (0..self.latent_dim)
            .map(|i| medium_term.iter().map(|v| v[i]).sum::<f64>() / medium_term.len() as f64)
            .collect();

        let total_energy = self.energy_model.energy(&short_term, &z_current);

        MultiScalePrediction {
            short_term,
            medium_term,
            long_term_trend,
            uncertainties,
            total_energy,
        }
    }

    pub fn predict_multi_scale_rgm(
        &self,
        n_steps: usize,
        observations: &[f64],
    ) -> Vec<Vec<LatentState>> {
        if n_steps == 0 || self.num_rgm_scales == 0 {
            return Vec::new();
        }

        let z = if observations.is_empty() {
            vec![0.0; self.latent_dim]
        } else {
            self.context_encoder.encode(observations)
        };

        let latents = self.multiscale_jepa.coarse_grain_chain(&z);
        let mut all_states = Vec::with_capacity(self.num_rgm_scales);

        for s in 0..self.num_rgm_scales {
            let predictor = &self.multiscale_jepa.predictors[s];
            let dim_s = latents[s].data.len();
            let mut z_s = latents[s].data.clone();
            let mut states = Vec::with_capacity(n_steps);

            for _ in 0..n_steps {
                let z_next = predictor.predict(&z_s);

                let delta: Vector = z_next
                    .iter()
                    .zip(z_s.iter())
                    .map(|(nxt, cur)| (nxt - cur).clamp(-10.0, 10.0))
                    .collect();

                let gamma = if n_steps >= 50 { 0.99 } else { 0.95 };
                let dynamics = TDDynamics::new(dim_s, gamma);
                let v = dynamics.value(&z_next, &self.td_target_critic);
                let value: Vector = (0..dim_s).map(|_| v / dim_s as f64).collect();

                states.push(LatentState { value, delta });
                z_s = z_next;
            }

            all_states.push(states);
        }

        all_states
    }

    pub fn train_step(&mut self, x_features: &[f64], y_features: &[f64]) -> (f64, f64, f64, f64) {
        let z_context = self.context_encoder.encode(x_features);
        let z_target = self.target_encoder.encode(y_features);

        let z_pred = self.predictor.predict(&z_context);

        let energy = self.energy_model.energy(&z_pred, &z_target);

        let (vicreg_total, inv_loss, _var_loss, _cov_loss) =
            self.vicreg.compute(&z_pred, &z_target);

        let gauss_loss =
            JEPA_GAUSS_WEIGHT * EnergyModel::gaussian_regularizer(&z_pred, JEPA_GAUSS_STD_TARGET);

        let total_loss = energy + vicreg_total + gauss_loss;

        self.predictor
            .update(&z_context, &z_target, self.learning_rate);

        self.target_encoder
            .ema_update(&self.context_encoder, self.momentum);

        self.training_steps += 1;

        (total_loss, energy, vicreg_total, inv_loss)
    }

    pub fn train_batch(&mut self, batch_x: &[Vec<f64>], batch_y: &[Vec<f64>]) -> f64 {
        let n = batch_x.len().min(batch_y.len());
        if n == 0 {
            return 0.0;
        }

        let batch_latents: Vec<Vec<f64>> = if self.sigreg.is_some() {
            batch_x
                .iter()
                .map(|x| self.context_encoder.encode(x))
                .collect()
        } else {
            Vec::new()
        };

        let total_loss: f64 = (0..n)
            .map(|i| {
                let (loss, _, _, _) = self.train_step(&batch_x[i], &batch_y[i]);
                loss
            })
            .sum();

        let mut avg_loss = total_loss / n as f64;

        if let Some(ref sigreg) = self.sigreg {
            let sigreg_loss = sigreg.compute_loss(&batch_latents);
            avg_loss += self.sigreg_lambda * sigreg_loss;
        }

        avg_loss
    }

    pub fn predict_with_confidence(&self, context_features: &[f64]) -> (Vector, f64, f64) {
        let z_context = self.context_encoder.encode(context_features);
        let (z_pred, variance) = self.predictor.predict_with_uncertainty(&z_context, 10);
        let energy = self.energy_model.energy(&z_pred, &z_context);
        let uncertainty = variance.iter().sum::<f64>() / variance.len() as f64;
        (z_pred, energy, uncertainty)
    }

    pub fn detect_anomaly(&self, context_features: &[f64], threshold: f64) -> bool {
        let (_z_pred, energy) = self.predict(context_features);
        energy > threshold
    }

    pub fn encode(&self, features: &[f64]) -> Vector {
        self.context_encoder.encode(features)
    }

    pub fn predict_next_latent(&self, z: &[f64]) -> Vector {
        self.predict_next_latent_with_dt(z, 1.0)
    }

    pub fn predict_next_latent_with_dt(&self, z: &[f64], dt: f64) -> Vector {
        if let Some(ref td_flows) = self.td_flows {
            let z_f32: Vec<f32> = z.iter().map(|&x| x as f32).collect();
            let result_f32 = td_flows.predict_at_time(&z_f32, dt);
            let mut result: Vector = result_f32.into_iter().map(|x| x as f64).collect();
            for val in result.iter_mut() {
                *val = val.clamp(-10.0, 10.0);
            }
            result
        } else {
            let mut z_next = self.predictor.predict(z);
            for val in z_next.iter_mut() {
                *val = val.clamp(-10.0, 10.0);
            }
            z_next
        }
    }

    pub fn td_predict_n(
        &self,
        n_steps: usize,
        _actions: &[f64],
        observations: &[f64],
    ) -> (Vec<LatentState>, Vec<f64>) {
        if n_steps == 0 {
            return (Vec::new(), Vec::new());
        }

        let gamma = if n_steps >= 50 { 0.99 } else { 0.95 };
        let dynamics = TDDynamics::new(self.latent_dim, gamma);

        let mut z = if observations.is_empty() {
            vec![0.0; self.latent_dim]
        } else {
            self.context_encoder.encode(observations)
        };

        let mut states = Vec::with_capacity(n_steps);
        let mut td_errors = Vec::with_capacity(n_steps);

        for _step in 0..n_steps {
            let z_next = self.predict_next_latent(&z);

            let delta: Vector = z_next
                .iter()
                .zip(z.iter())
                .map(|(nxt, cur)| (nxt - cur).clamp(-10.0, 10.0))
                .collect();

            let v = dynamics.value(&z_next, &self.td_target_critic);
            let value: Vector = (0..self.latent_dim)
                .map(|_| v / self.latent_dim as f64)
                .collect();

            let td_err = dynamics.td_error(0.0, &z, &z_next, &self.td_target_critic);

            states.push(LatentState { value, delta });
            td_errors.push(td_err);

            z = z_next;
        }

        (states, td_errors)
    }

    pub fn td_learn(&mut self, experiences: &[TDExperience]) -> f64 {
        if experiences.is_empty() {
            return 0.0;
        }

        let dynamics = TDDynamics::new(self.latent_dim, self.td_gamma);
        let lr = self.learning_rate;
        let mut total_error = 0.0;

        for exp in experiences {
            let td_err = dynamics.td_error(
                exp.reward,
                &exp.z_t,
                &exp.z_t_plus_n,
                &self.td_target_critic,
            );
            let td_err_clamped = td_err.clamp(-10.0, 10.0);

            dynamics.update_critic(&exp.z_t, td_err_clamped, lr, &mut self.td_target_critic);

            total_error += td_err_clamped.abs();
        }

        total_error / experiences.len() as f64
    }

    pub fn predict_next_state(&self, current: &WorldModelState) -> WorldModelState {
        let features = current.to_features();
        let (z_pred, _energy) = self.predict(&features);
        let n = self.latent_dim.min(7);
        let mut deltas = vec![0.0; 7];
        for i in 0..n {
            deltas[i] = z_pred[i].clamp(-0.3, 0.3);
        }
        WorldModelState {
            cpu_usage: (current.cpu_usage + deltas[0]).max(0.0).min(1.0),
            memory_available: (current.memory_available + deltas[1]).max(0.0).min(1.0),
            network_latency: (current.network_latency + deltas[2] * 50.0).max(0.0),
            task_queue_depth: ((current.task_queue_depth as f64) + deltas[3] * 10.0).max(0.0)
                as u32,
            error_rate: (current.error_rate + deltas[4] * 0.1).max(0.0).min(1.0),
            iteration_count: current.iteration_count + 1,
            token_usage_pct: (current.token_usage_pct + deltas[6] * 0.1)
                .max(0.0)
                .min(1.0),
        }
    }

    pub fn long_horizon_rollout(&self, horizon: usize) -> Vec<LatentState> {
        if horizon == 0 {
            return Vec::new();
        }

        let gamma = if horizon >= 50 { 0.99 } else { 0.95 };
        let dynamics = TDDynamics::new(self.latent_dim, gamma);
        let mut z = vec![0.0; self.latent_dim];
        let mut trajectory = Vec::with_capacity(horizon);

        for _step in 0..horizon {
            let z_next = self.predict_next_latent(&z);

            let delta: Vector = z_next
                .iter()
                .zip(z.iter())
                .map(|(nxt, cur)| (nxt - cur).clamp(-10.0, 10.0))
                .collect();

            let v = dynamics.value(&z_next, &self.td_target_critic);
            let value: Vector = (0..self.latent_dim)
                .map(|_| v / self.latent_dim as f64)
                .collect();

            trajectory.push(LatentState { value, delta });
            z = z_next;
        }

        trajectory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_monitor_can_be_enabled() {
        let model = JepaWorldModel::new(32).with_spectral_monitor(10);
        assert!(model.spectral_monitor.is_some());
    }

    #[test]
    fn test_spectral_health_none_without_monitor() {
        let model = JepaWorldModel::new(32);
        assert!(model.spectral_health().is_none());
    }

    #[test]
    fn test_check_rollout_stability_default() {
        let model = JepaWorldModel::new(32);
        assert!(model.check_rollout_stability());
    }

    #[test]
    fn test_record_rollout_reward_does_not_panic() {
        let mut model = JepaWorldModel::new(32).with_spectral_monitor(10);
        for step in 0..5 {
            model.record_rollout_reward(step, 0.5 + 0.1 * step as f64);
        }
        let health = model.spectral_health();
        assert!(health.is_some());
    }

    #[test]
    fn test_identifiability_report_available() {
        let mut model = JepaWorldModel::new(32).with_spectral_monitor(10);
        for step in 0..10 {
            model.record_rollout_reward(step, 0.3);
        }
        assert!(model.identifiability_report().is_some());
    }
}
