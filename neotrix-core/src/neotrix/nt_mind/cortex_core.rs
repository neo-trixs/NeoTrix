use crate::neotrix::nt_mind::cortex_types::{
    PredictiveCortex, PredictedStep, HorizonForecast, ActionPlan,
    ANOMALY_ENERGY_THRESHOLD, CONFIDENCE_LOW_THRESHOLD, FE_DIVERGENCE_THRESHOLD,
};

impl PredictiveCortex {
    pub fn predict_horizon(
        &mut self,
        current_latent: &[f64],
        horizon: usize,
    ) -> HorizonForecast {
        let mut trajectory = Vec::with_capacity(horizon);
        let mut cumulative_fe = 0.0;
        let mut total_confidence = 0.0;
        let mut anomaly_predicted = false;
        let mut divergence_step = None;

        self.e8.from_jepa_latent(current_latent);

        let mut latent = current_latent.to_vec();

        for step in 0..horizon {
            let (mean, variance) = self.jepa.predict_with_uncertainty(&latent, self.n_samples);
            let confidence = self.compute_step_confidence(&variance);

            self.e8.evolve(1.0);
            let hexagram = self.e8.current_state.vector.clone();

            let jepa_energy = self.compute_jepa_energy(&mean, &latent);
            let e8_entropy = self.e8.entropy();
            let e8_gradient = self.compute_e8_gradient();
            let fe_report = self.ai.compute_free_energy(jepa_energy, e8_entropy, e8_gradient);

            cumulative_fe += fe_report.variational_fe;
            total_confidence += confidence;

            if fe_report.variational_fe > FE_DIVERGENCE_THRESHOLD && divergence_step.is_none() {
                divergence_step = Some(step);
                anomaly_predicted = true;
            }
            if confidence < CONFIDENCE_LOW_THRESHOLD {
                anomaly_predicted = true;
            }

            trajectory.push(PredictedStep {
                step,
                latent_mean: mean.clone(),
                latent_variance: variance,
                hexagram_state: hexagram,
                free_energy: fe_report.variational_fe,
                confidence,
            });

            latent = mean;
        }

        let avg_confidence = total_confidence / horizon.max(1) as f64;

        let forecast = HorizonForecast {
            trajectory,
            cumulative_fe,
            avg_confidence,
            anomaly_predicted,
            divergence_step,
        };

        self.fe_timeline.push(forecast.cumulative_fe);
        if self.fe_timeline.len() > 200 {
            self.fe_timeline.remove(0);
        }
        self.forecast_history.push(forecast.clone());
        if self.forecast_history.len() > 50 {
            self.forecast_history.remove(0);
        }

        forecast
    }

    pub fn plan_best_action(
        &mut self,
        current_latent: &[f64],
        candidate_actions: &[Vec<f64>],
        horizon: usize,
    ) -> ActionPlan {
        let mut rankings: Vec<(usize, f64)> = Vec::with_capacity(candidate_actions.len());
        let mut forecasts: Vec<HorizonForecast> = Vec::with_capacity(candidate_actions.len());

        for (idx, action) in candidate_actions.iter().enumerate() {
            let perturbed = self.apply_action(current_latent, action);
            let forecast = self.predict_horizon(&perturbed, horizon);
            rankings.push((idx, forecast.cumulative_fe));
            forecasts.push(forecast);
        }

        rankings.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let best_idx = rankings.first().map(|r| r.0).unwrap_or(0);
        let best_action = candidate_actions[best_idx].clone();

        ActionPlan {
            best_action,
            forecast: forecasts[best_idx].clone(),
            action_rankings: rankings,
        }
    }

    pub fn lookahead_anomaly_prob(
        &mut self,
        current_latent: &[f64],
        horizon: usize,
    ) -> f64 {
        let forecast = self.predict_horizon(current_latent, horizon);
        if forecast.anomaly_predicted {
            let min_conf = forecast.trajectory.iter()
                .map(|s| s.confidence)
                .fold(1.0_f64, f64::min);
            let max_fe = forecast.trajectory.iter()
                .map(|s| s.free_energy)
                .fold(0.0_f64, f64::max);

            let conf_factor = 1.0 - min_conf;
            let fe_factor = (max_fe / ANOMALY_ENERGY_THRESHOLD).min(1.0);
            (conf_factor * 0.6 + fe_factor * 0.4).min(1.0)
        } else {
            0.0
        }
    }

    pub fn counterfactual(
        &mut self,
        current_latent: &[f64],
        action: &[f64],
        horizon: usize,
    ) -> HorizonForecast {
        let perturbed = self.apply_action(current_latent, action);
        self.predict_horizon(&perturbed, horizon)
    }

    pub fn generate_candidate_actions(&self, n: usize) -> Vec<Vec<f64>> {
        let mut actions = Vec::with_capacity(n);
        for _ in 0..n {
            let action: Vec<f64> = (0..self.action_dim)
                .map(|_| (rand::random::<f64>() - 0.5) * 2.0)
                .collect();
            actions.push(action);
        }
        actions
    }

    pub fn apply_action(&self, latent: &[f64], action: &[f64]) -> Vec<f64> {
        let dim = latent.len().min(action.len());
        let mut perturbed = latent.to_vec();
        for i in 0..dim {
            perturbed[i] = (perturbed[i] + action[i] * 0.1).clamp(-1.0, 1.0);
        }
        perturbed
    }

    pub fn compute_jepa_energy(&self, prediction: &[f64], current: &[f64]) -> f64 {
        let dim = prediction.len().min(current.len());
        let mse: f64 = (0..dim).map(|i| {
            let d = prediction[i] - current[i];
            d * d
        }).sum::<f64>() / dim.max(1) as f64;
        mse
    }

    pub fn compute_e8_gradient(&self) -> f64 {
        let e = self.e8.energy();
        if self.e8.evolution_step > 0 {
            if let Some(prev) = self.e8.prediction_history.last() {
                let prev_energy: f64 = prev.vector.iter().map(|v| v * v).sum();
                return (e - prev_energy).abs();
            }
        }
        e
    }

    pub fn compute_step_confidence(&self, variance: &[f64]) -> f64 {
        let mean_var: f64 = variance.iter().copied().sum::<f64>() / variance.len().max(1) as f64;
        (-mean_var / 0.1).exp().clamp(0.0, 1.0)
    }
}
