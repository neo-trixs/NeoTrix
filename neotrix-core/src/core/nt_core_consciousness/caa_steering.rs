// REVIVED Task 1 — dead_code removed 2026-06-24

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SteeringChannel {
    Residual,
    Sampling,
    Context,
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: SteeringChannel,
    pub alpha: f64,
    pub enabled: bool,
}

fn default_channel_configs() -> Vec<ChannelConfig> {
    vec![
        ChannelConfig {
            channel: SteeringChannel::Residual,
            alpha: 0.3,
            enabled: true,
        },
        ChannelConfig {
            channel: SteeringChannel::Sampling,
            alpha: 0.1,
            enabled: true,
        },
        ChannelConfig {
            channel: SteeringChannel::Context,
            alpha: 0.2,
            enabled: true,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct CaaDirection {
    pub composite_vector: Vec<u8>,
    pub dominant_emotion: String,
    pub alpha: f64,
    pub computed_at: u64,
}

#[derive(Debug, Clone)]
pub struct CaaSteeringRecord {
    pub timestamp: u64,
    pub emotion_before: String,
    pub emotion_after: String,
    pub alpha_used: f64,
    pub divergence: f64,
    pub passed_validation: bool,
}

#[derive(Debug, Clone)]
pub struct CaaSteeringEngine {
    pub alpha_min: f64,
    pub alpha_max: f64,
    pub enabled: bool,
    pub history: Vec<CaaSteeringRecord>,
    max_history: usize,
    channel_configs: Vec<ChannelConfig>,
}

impl Default for CaaSteeringEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CaaSteeringEngine {
    pub fn new() -> Self {
        Self {
            alpha_min: 0.05,
            alpha_max: 0.3,
            enabled: true,
            history: Vec::with_capacity(64),
            max_history: 256,
            channel_configs: default_channel_configs(),
        }
    }

    pub fn with_channels(channels: Vec<ChannelConfig>) -> Self {
        Self {
            alpha_min: 0.05,
            alpha_max: 0.3,
            enabled: true,
            history: Vec::with_capacity(64),
            max_history: 256,
            channel_configs: channels,
        }
    }

    fn emotion_seed(emotion: &str) -> u64 {
        let bytes: Vec<u8> = emotion.bytes().collect();
        bytes
            .iter()
            .fold(42u64, |acc, b| acc.wrapping_mul(31).wrapping_add(*b as u64))
    }

    pub fn compute_direction(
        &self,
        emotion: &str,
        intensity: f64,
        context_vsa: &[u8],
    ) -> CaaDirection {
        let intensity = intensity.clamp(0.0, 1.0);
        let seed = Self::emotion_seed(emotion);
        let emotion_vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
        let blend = intensity.clamp(0.0, 1.0);
        let mut composite = Vec::with_capacity(VSA_DIM);
        let ctx_len = context_vsa.len().min(VSA_DIM);
        for i in 0..VSA_DIM {
            let e_val = emotion_vsa[i];
            let c_val = if i < ctx_len { context_vsa[i] } else { 0 };
            let mixed = (e_val as f64 * blend + c_val as f64 * (1.0 - blend)).round() as u8;
            composite.push(mixed);
        }
        CaaDirection {
            composite_vector: composite,
            dominant_emotion: emotion.to_string(),
            alpha: intensity * (self.alpha_max - self.alpha_min) + self.alpha_min,
            computed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn steer_generation(&self, direction: &CaaDirection, generation_vsa: &mut [u8]) {
        if !self.enabled {
            return;
        }
        let alpha = direction.alpha.clamp(0.0, 1.0);
        let len = generation_vsa.len().min(direction.composite_vector.len());
        for i in 0..len {
            let dir_val = direction.composite_vector[i];
            let gen_val = generation_vsa[i];
            let blended = (gen_val as f64 * (1.0 - alpha) + dir_val as f64 * alpha).round() as u8;
            generation_vsa[i] = blended;
        }
    }

    pub fn steer_channel(
        &self,
        direction: &CaaDirection,
        target: &mut [u8],
        channel: SteeringChannel,
    ) {
        let config = match self.channel_configs.iter().find(|c| c.channel == channel) {
            Some(c) => c,
            None => return,
        };
        if !config.enabled || !self.enabled {
            return;
        }
        let alpha = config.alpha.clamp(0.0, 1.0);
        let len = target.len().min(direction.composite_vector.len());
        match channel {
            SteeringChannel::Residual => {
                for i in 0..len {
                    let dir_val = direction.composite_vector[i];
                    let gen_val = target[i];
                    let blended =
                        (gen_val as f64 * (1.0 - alpha) + dir_val as f64 * alpha).round() as u8;
                    target[i] = blended;
                }
            }
            SteeringChannel::Sampling => {
                for i in 0..len {
                    if direction.composite_vector[i] > 128 {
                        let delta = (alpha * 255.0).round() as u8;
                        target[i] = target[i].saturating_add(delta);
                    }
                }
            }
            SteeringChannel::Context => {
                let ctx_weight = alpha * 0.5 + 0.5;
                for i in 0..len {
                    let dir_val = direction.composite_vector[i];
                    let gen_val = target[i];
                    let blended = (gen_val as f64 * (1.0 - ctx_weight)
                        + dir_val as f64 * ctx_weight)
                        .round() as u8;
                    target[i] = blended;
                }
            }
        }
    }

    pub fn bias_sampling(&self, direction: &CaaDirection, probability: f64) -> f64 {
        let ones = direction
            .composite_vector
            .iter()
            .filter(|&&x| x > 128)
            .count();
        let density = ones as f64 / direction.composite_vector.len() as f64;
        let shift = direction.alpha * (density - 0.5);
        (probability + shift).clamp(0.0, 1.0)
    }

    pub fn bias_context(&self, direction: &CaaDirection, context_vsas: &[Vec<u8>]) -> Vec<f64> {
        let sims: Vec<f64> = context_vsas
            .iter()
            .map(|v| QuantizedVSA::similarity(&direction.composite_vector, v))
            .collect();
        let total: f64 = sims.iter().sum();
        if total.abs() < 1e-12 {
            return context_vsas
                .iter()
                .map(|_| 1.0 / context_vsas.len() as f64)
                .collect();
        }
        sims.iter().map(|s| s / total).collect()
    }

    pub fn probe_steering(&self, direction: &CaaDirection, baseline: &[u8]) -> f64 {
        let mut simulated = baseline.to_vec();
        self.steer_generation(direction, &mut simulated);
        1.0 - QuantizedVSA::similarity(baseline, &simulated)
    }

    pub fn steer_ode_state(&self, direction: &CaaDirection, ode_state: &mut [f64]) {
        if !self.enabled {
            return;
        }
        let alpha = direction.alpha.clamp(0.0, 1.0);
        let len = ode_state.len().min(direction.composite_vector.len());
        for i in 0..len {
            let dir_bias = if direction.composite_vector[i] > 128 {
                1.0
            } else {
                -1.0
            };
            ode_state[i] = ode_state[i] * (1.0 - alpha) + dir_bias * alpha;
        }
    }

    pub fn channel_report(&self) -> String {
        let mut lines = String::from("CAA Steering Channel Report:\n");
        for config in &self.channel_configs {
            let name = match config.channel {
                SteeringChannel::Residual => "Residual",
                SteeringChannel::Sampling => "Sampling",
                SteeringChannel::Context => "Context",
            };
            lines.push_str(&format!(
                "  {}: alpha={:.3} enabled={}\n",
                name, config.alpha, config.enabled
            ));
        }
        lines
    }

    pub fn record_steering(
        &mut self,
        before_emotion: &str,
        after_emotion: &str,
        alpha: f64,
        divergence: f64,
        passed: bool,
    ) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let record = CaaSteeringRecord {
            timestamp: ts,
            emotion_before: before_emotion.to_string(),
            emotion_after: after_emotion.to_string(),
            alpha_used: alpha,
            divergence,
            passed_validation: passed,
        };
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(record);
    }

    pub fn validate_steering(baseline: &[u8], steered: &[u8], threshold: f64) -> bool {
        let divergence = 1.0 - QuantizedVSA::similarity(baseline, steered);
        divergence > threshold
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn channel_enable(&mut self, channel: SteeringChannel) {
        if let Some(c) = self
            .channel_configs
            .iter_mut()
            .find(|c| c.channel == channel)
        {
            c.enabled = true;
        }
    }

    pub fn channel_disable(&mut self, channel: SteeringChannel) {
        if let Some(c) = self
            .channel_configs
            .iter_mut()
            .find(|c| c.channel == channel)
        {
            c.enabled = false;
        }
    }

    pub fn report(&self) -> String {
        let total = self.history.len();
        let passed = self.history.iter().filter(|r| r.passed_validation).count();
        let avg_div: f64 = if total > 0 {
            self.history.iter().map(|r| r.divergence).sum::<f64>() / total as f64
        } else {
            0.0
        };
        format!(
            "CAA Steering Report:\n  Total: {total}\n  Passed: {passed}\n  Avg Divergence: {avg_div:.4}\n  Enabled: {enabled}",
            enabled = self.enabled
        )
    }

    pub fn recent_history(&self, n: usize) -> Vec<&CaaSteeringRecord> {
        let n = n.min(self.history.len());
        self.history.iter().skip(self.history.len() - n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa() -> Vec<u8> {
        QuantizedVSA::seeded_random(12345, VSA_DIM)
    }

    fn test_context_vsas() -> Vec<Vec<u8>> {
        vec![
            QuantizedVSA::seeded_random(100, VSA_DIM),
            QuantizedVSA::seeded_random(200, VSA_DIM),
            QuantizedVSA::seeded_random(300, VSA_DIM),
        ]
    }

    #[test]
    fn test_compute_direction() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("curiosity", 0.8, &ctx);
        assert_eq!(dir.composite_vector.len(), VSA_DIM);
        assert_eq!(dir.dominant_emotion, "curiosity");
        assert!(dir.alpha >= engine.alpha_min && dir.alpha <= engine.alpha_max);
    }

    #[test]
    fn test_steer_generation() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("excitement", 0.6, &ctx);
        let mut gen = test_vsa();
        let before = gen.clone();
        engine.steer_generation(&dir, &mut gen);
        let divergence = 1.0 - QuantizedVSA::similarity(&before, &gen);
        assert!(divergence > 0.0);
    }

    #[test]
    fn test_validate_steering() {
        let a = test_vsa();
        let mut b = a.clone();
        b[0] = b[0].wrapping_add(1);
        assert!(CaaSteeringEngine::validate_steering(&a, &b, 0.0));
        assert!(!CaaSteeringEngine::validate_steering(&a, &a, 0.0));
    }

    #[test]
    fn test_three_channels_exist() {
        let engine = CaaSteeringEngine::new();
        assert_eq!(engine.channel_configs.len(), 3);
        let channels: Vec<SteeringChannel> =
            engine.channel_configs.iter().map(|c| c.channel).collect();
        assert!(channels.contains(&SteeringChannel::Residual));
        assert!(channels.contains(&SteeringChannel::Sampling));
        assert!(channels.contains(&SteeringChannel::Context));
    }

    #[test]
    fn test_residual_channel_steers() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("joy", 0.7, &ctx);
        let mut target = test_vsa();
        let before = target.clone();
        engine.steer_channel(&dir, &mut target, SteeringChannel::Residual);
        let divergence = 1.0 - QuantizedVSA::similarity(&before, &target);
        assert!(divergence > 0.0);
    }

    #[test]
    fn test_sampling_channel_bias() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("calm", 0.5, &ctx);
        let p = engine.bias_sampling(&dir, 0.5);
        assert!(p >= 0.0 && p <= 1.0);
    }

    #[test]
    fn test_context_channel_bias() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("fear", 0.4, &ctx);
        let contexts = test_context_vsas();
        let weights = engine.bias_context(&dir, &contexts);
        assert_eq!(weights.len(), 3);
        for &w in &weights {
            assert!(w >= 0.0 && w <= 1.0);
        }
    }

    #[test]
    fn test_probe_steering() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("surprise", 0.9, &ctx);
        let baseline = test_vsa();
        let divergence = engine.probe_steering(&dir, &baseline);
        assert!(divergence >= 0.0 && divergence <= 1.0);
        let after_probe = baseline.clone();
        assert_eq!(baseline, after_probe);
    }

    #[test]
    fn test_steer_ode_state() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("anger", 0.6, &ctx);
        let mut ode = vec![0.0; 100];
        engine.steer_ode_state(&dir, &mut ode);
        let non_zero = ode.iter().filter(|&&x| x != 0.0).count();
        assert!(non_zero > 0);
    }

    #[test]
    fn test_channel_report() {
        let engine = CaaSteeringEngine::new();
        let report = engine.channel_report();
        assert!(report.contains("Residual"));
        assert!(report.contains("Sampling"));
        assert!(report.contains("Context"));
    }

    #[test]
    fn test_enable_disable_per_channel() {
        let mut engine = CaaSteeringEngine::new();
        engine.channel_disable(SteeringChannel::Sampling);
        let config = engine
            .channel_configs
            .iter()
            .find(|c| c.channel == SteeringChannel::Sampling)
            .unwrap();
        assert!(!config.enabled);
        engine.channel_enable(SteeringChannel::Sampling);
        let config = engine
            .channel_configs
            .iter()
            .find(|c| c.channel == SteeringChannel::Sampling)
            .unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn test_emotion_seed_deterministic() {
        let s1 = CaaSteeringEngine::emotion_seed("curiosity");
        let s2 = CaaSteeringEngine::emotion_seed("curiosity");
        assert_eq!(s1, s2);
        let s3 = CaaSteeringEngine::emotion_seed("excitement");
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_bias_sampling_bounds() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("trust", 1.0, &ctx);
        for p in [0.0, 0.5, 1.0] {
            let biased = engine.bias_sampling(&dir, p);
            assert!(
                biased >= 0.0 && biased <= 1.0,
                "biased={} out of [0,1]",
                biased
            );
        }
    }

    #[test]
    fn test_context_bias_sum_approx_one() {
        let engine = CaaSteeringEngine::new();
        let ctx = test_vsa();
        let dir = engine.compute_direction("anticipation", 0.3, &ctx);
        let contexts = test_context_vsas();
        let weights = engine.bias_context(&dir, &contexts);
        let sum: f64 = weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "weights sum to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_record_steering() {
        let mut engine = CaaSteeringEngine::new();
        engine.record_steering("neutral", "curiosity", 0.2, 0.15, true);
        assert_eq!(engine.history.len(), 1);
        assert_eq!(engine.recent_history(1)[0].emotion_before, "neutral");
    }

    #[test]
    fn test_enable_disable() {
        let mut engine = CaaSteeringEngine::new();
        assert!(engine.enabled);
        engine.disable();
        assert!(!engine.enabled);
        let ctx = test_vsa();
        let dir = engine.compute_direction("frustration", 0.5, &ctx);
        let mut gen = test_vsa();
        let before = gen.clone();
        engine.steer_generation(&dir, &mut gen);
        assert_eq!(gen, before);
    }

    #[test]
    fn test_report() {
        let mut engine = CaaSteeringEngine::new();
        engine.record_steering("a", "b", 0.1, 0.2, true);
        let report = engine.report();
        assert!(report.contains("CAA Steering Report"));
    }

    #[test]
    fn test_recent_history() {
        let mut engine = CaaSteeringEngine::new();
        for i in 0..10 {
            engine.record_steering("a", "b", 0.1, i as f64 * 0.1, i % 2 == 0);
        }
        assert_eq!(engine.recent_history(3).len(), 3);
        assert_eq!(engine.recent_history(100).len(), 10);
    }

    #[test]
    fn test_max_history_bounded() {
        let mut engine = CaaSteeringEngine::new();
        engine.max_history = 5;
        for i in 0..10 {
            engine.record_steering("x", "y", 0.1, i as f64, true);
        }
        assert_eq!(engine.history.len(), 5);
    }
}
