use crate::core::nt_core_hcube::QuantizedVSA;
use crate::neotrix::nt_mind::predictive_cortex::PredictiveCortex;

#[derive(Debug, Clone)]
pub struct WorldModelReport {
    pub predicted_state: Vec<u8>,
    pub anomaly_prob: f64,
    pub has_degradation: bool,
    pub prediction_confidence: f64,
    pub cycle: u64,
}

impl Default for WorldModelReport {
    fn default() -> Self {
        Self {
            predicted_state: Vec::new(),
            anomaly_prob: 0.0,
            has_degradation: false,
            prediction_confidence: 0.0,
            cycle: 0,
        }
    }
}

pub struct WorldModelBridge {
    pub predictive_cortex: Option<PredictiveCortex>,
    pub prediction_error_ema: f64,
    pub consecutive_degradations: usize,
    cycle: u64,
    vsa_dim: usize,
}

impl WorldModelBridge {
    pub fn new(vsa_dim: usize) -> Self {
        Self {
            predictive_cortex: None,
            prediction_error_ema: 0.0,
            consecutive_degradations: 0,
            cycle: 0,
            vsa_dim,
        }
    }

    fn vsa_to_latent(vsa: &[u8]) -> Vec<f64> {
        vsa.iter().map(|&b| b as f64 / 255.0).collect()
    }

    fn latent_to_vsa(latent: &[f64]) -> Vec<u8> {
        latent
            .iter()
            .map(|&f| (f.clamp(0.0, 1.0) * 255.0) as u8)
            .collect()
    }

    pub fn tick(&mut self, attractor_state: &[u8]) -> WorldModelReport {
        if self.predictive_cortex.is_none() {
            if attractor_state.len() != self.vsa_dim {
                return WorldModelReport::default();
            }
            let latent_dim = self.vsa_dim;
            let hidden_dim = (latent_dim / 4).max(64);
            self.predictive_cortex = Some(PredictiveCortex::new(latent_dim, hidden_dim));
        }

        if let Some(ref mut cortex) = self.predictive_cortex {
            let latent = Self::vsa_to_latent(attractor_state);
            if latent.is_empty() {
                return WorldModelReport::default();
            }

            let forecast = cortex.predict_horizon(&latent, 3);
            let anomaly = cortex.lookahead_anomaly_prob(&latent, 3);
            let degradation = cortex.detect_degradation();

            self.prediction_error_ema =
                self.prediction_error_ema * 0.9 + (1.0 - forecast.avg_confidence) * 0.1;
            if degradation.is_some() {
                self.consecutive_degradations = self.consecutive_degradations.saturating_add(1);
            } else {
                self.consecutive_degradations = self.consecutive_degradations.saturating_sub(1);
            }

            let predicted_latent = forecast
                .trajectory
                .last()
                .map(|s| s.latent_mean.clone())
                .unwrap_or_else(|| latent.clone());
            let predicted_vsa = Self::latent_to_vsa(&predicted_latent);

            self.cycle += 1;
            WorldModelReport {
                predicted_state: predicted_vsa,
                anomaly_prob: anomaly,
                has_degradation: degradation.is_some(),
                prediction_confidence: forecast.avg_confidence,
                cycle: self.cycle,
            }
        } else {
            WorldModelReport::default()
        }
    }

    pub fn bundle_vsa_buffer(&self, vsa_buffer: &std::collections::VecDeque<Vec<u8>>) -> Vec<u8> {
        if vsa_buffer.is_empty() {
            return Vec::new();
        }
        if vsa_buffer.len() == 1 {
            return vsa_buffer[0].clone();
        }
        let mut bundled = vsa_buffer[0].clone();
        for v in vsa_buffer.iter().skip(1) {
            let permuted = QuantizedVSA::permute(v, 1);
            bundled = QuantizedVSA::bundle(&[&bundled[..], &permuted[..]]);
        }
        bundled
    }
}
