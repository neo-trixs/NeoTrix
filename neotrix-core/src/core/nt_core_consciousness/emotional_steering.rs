// REVIVED Task 1 — dead_code removed 2026-06-24

use super::caa_steering::{CaaDirection, CaaSteeringEngine};
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmotionalDimension {
    Curiosity,
    Satisfaction,
    Frustration,
    Energy,
    Loneliness,
}

#[derive(Debug, Clone)]
pub struct EmotionState {
    pub curiosity: f64,
    pub satisfaction: f64,
    pub frustration: f64,
    pub energy: f64,
    pub loneliness: f64,
}

impl EmotionState {
    pub fn new() -> Self {
        Self {
            curiosity: 0.5,
            satisfaction: 0.5,
            frustration: 0.5,
            energy: 0.5,
            loneliness: 0.5,
        }
    }

    pub fn get(&self, dim: EmotionalDimension) -> f64 {
        match dim {
            EmotionalDimension::Curiosity => self.curiosity,
            EmotionalDimension::Satisfaction => self.satisfaction,
            EmotionalDimension::Frustration => self.frustration,
            EmotionalDimension::Energy => self.energy,
            EmotionalDimension::Loneliness => self.loneliness,
        }
    }

    pub fn set(&mut self, dim: EmotionalDimension, value: f64) {
        let clamped = value.clamp(0.0, 1.0);
        match dim {
            EmotionalDimension::Curiosity => self.curiosity = clamped,
            EmotionalDimension::Satisfaction => self.satisfaction = clamped,
            EmotionalDimension::Frustration => self.frustration = clamped,
            EmotionalDimension::Energy => self.energy = clamped,
            EmotionalDimension::Loneliness => self.loneliness = clamped,
        }
    }

    pub fn modulate(&mut self, dim: EmotionalDimension, delta: f64) {
        let current = self.get(dim);
        self.set(dim, current + delta);
    }

    pub fn dominant(&self) -> EmotionalDimension {
        let mut best = EmotionalDimension::Curiosity;
        let mut best_val = self.curiosity;
        if self.satisfaction > best_val {
            best = EmotionalDimension::Satisfaction;
            best_val = self.satisfaction;
        }
        if self.frustration > best_val {
            best = EmotionalDimension::Frustration;
            best_val = self.frustration;
        }
        if self.energy > best_val {
            best = EmotionalDimension::Energy;
            best_val = self.energy;
        }
        if self.loneliness > best_val {
            best = EmotionalDimension::Loneliness;
        }
        best
    }

    pub fn is_distress(&self) -> bool {
        self.frustration > 0.8 || self.energy < 0.2
    }
}

impl Default for EmotionState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum EmotionalEvent {
    Success { magnitude: f64 },
    Failure { magnitude: f64 },
    Novelty { magnitude: f64 },
    Progress { magnitude: f64 },
    SocialInteraction { quality: f64 },
    CognitiveEffort { duration: f64 },
    Rest { duration: f64 },
}

#[derive(Debug, Clone)]
pub struct EmotionalSteering {
    pub state: EmotionState,
    pub decay_rate: f64,
    pub baseline: EmotionState,
    pub last_update: Instant,
}

impl EmotionalSteering {
    pub fn new() -> Self {
        let baseline = EmotionState::new();
        Self {
            state: baseline.clone(),
            decay_rate: 0.05,
            baseline,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self, tick_delta: f64) {
        for dim in &[
            EmotionalDimension::Curiosity,
            EmotionalDimension::Satisfaction,
            EmotionalDimension::Frustration,
            EmotionalDimension::Energy,
            EmotionalDimension::Loneliness,
        ] {
            let current = self.state.get(*dim);
            let target = self.baseline.get(*dim);
            let diff = target - current;
            let step = diff * self.decay_rate * tick_delta;
            self.state.modulate(*dim, step);
        }
    }

    pub fn apply_event(&mut self, event_type: EmotionalEvent) {
        match event_type {
            EmotionalEvent::Success { magnitude } => {
                self.state
                    .modulate(EmotionalDimension::Satisfaction, 0.2 * magnitude);
                self.state
                    .modulate(EmotionalDimension::Frustration, -0.15 * magnitude);
            }
            EmotionalEvent::Failure { magnitude } => {
                self.state
                    .modulate(EmotionalDimension::Satisfaction, -0.15 * magnitude);
                self.state
                    .modulate(EmotionalDimension::Frustration, 0.2 * magnitude);
            }
            EmotionalEvent::Novelty { magnitude } => {
                self.state
                    .modulate(EmotionalDimension::Curiosity, 0.2 * magnitude);
            }
            EmotionalEvent::Progress { magnitude } => {
                self.state
                    .modulate(EmotionalDimension::Satisfaction, 0.15 * magnitude);
                self.state
                    .modulate(EmotionalDimension::Frustration, -0.1 * magnitude);
            }
            EmotionalEvent::SocialInteraction { quality } => {
                self.state
                    .modulate(EmotionalDimension::Loneliness, -0.3 * quality);
            }
            EmotionalEvent::CognitiveEffort { duration } => {
                self.state
                    .modulate(EmotionalDimension::Energy, -0.2 * duration);
            }
            EmotionalEvent::Rest { duration } => {
                self.state
                    .modulate(EmotionalDimension::Energy, 0.3 * duration);
            }
        }
    }

    pub fn exploration_bonus(&self) -> f64 {
        (self.state.curiosity - self.state.frustration).clamp(0.0, 1.0)
    }

    pub fn energy_for_task(&self, complexity: f64) -> f64 {
        self.state.energy / (1.0 + complexity * 0.5)
    }

    pub fn should_rest(&self) -> bool {
        self.state.energy < 0.3
    }

    pub fn should_escalate(&self) -> bool {
        self.state.frustration > 0.8
    }

    pub fn loneliness_bonus(&self) -> f64 {
        self.state.loneliness
    }
}

impl Default for EmotionalSteering {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AffectChannel {
    Residual,
    Sampling,
    Context,
}

pub struct AffectInjection {
    pub channels: Vec<AffectChannel>,
    pub per_channel_alpha: Vec<(AffectChannel, f64)>,
    pub steering_engine: Option<CaaSteeringEngine>,
}

impl AffectInjection {
    pub fn new() -> Self {
        Self {
            channels: vec![
                AffectChannel::Residual,
                AffectChannel::Sampling,
                AffectChannel::Context,
            ],
            per_channel_alpha: vec![
                (AffectChannel::Residual, 0.3),
                (AffectChannel::Sampling, 0.3),
                (AffectChannel::Context, 0.3),
            ],
            steering_engine: None,
        }
    }

    pub fn set_channel(&mut self, channel: AffectChannel, alpha: f64) {
        if alpha <= 0.0 {
            self.channels.retain(|c| *c != channel);
            self.per_channel_alpha.retain(|(c, _)| *c != channel);
        } else {
            if !self.channels.contains(&channel) {
                self.channels.push(channel);
            }
            if let Some((_, a)) = self
                .per_channel_alpha
                .iter_mut()
                .find(|(c, _)| *c == channel)
            {
                *a = alpha;
            } else {
                self.per_channel_alpha.push((channel, alpha));
            }
        }
    }

    pub fn direction_from_emotion(&self, state: &EmotionState, context_vsa: &[u8]) -> CaaDirection {
        let dominant = state.dominant();
        let value = state.get(dominant);
        let name = format!("{:?}", dominant);
        match &self.steering_engine {
            Some(engine) => engine.compute_direction(&name, value, context_vsa),
            None => {
                let seed = name
                    .bytes()
                    .fold(42u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                let emotion_vsa = QuantizedVSA::seeded_random(seed, VSA_DIM);
                let mut composite = Vec::with_capacity(VSA_DIM);
                for i in 0..VSA_DIM {
                    let e_val = emotion_vsa[i];
                    let c_val = if i < context_vsa.len() {
                        context_vsa[i]
                    } else {
                        0
                    };
                    let mixed = (e_val as f64 * value + c_val as f64 * (1.0 - value)).round() as u8;
                    composite.push(mixed);
                }
                CaaDirection {
                    composite_vector: composite,
                    dominant_emotion: name,
                    alpha: value * 0.25 + 0.05,
                    computed_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            }
        }
    }

    pub fn inject_residual(
        &self,
        direction: &CaaDirection,
        target: &mut [u8],
        _state: &EmotionState,
    ) {
        if !self.channels.contains(&AffectChannel::Residual) {
            return;
        }
        let alpha = self
            .per_channel_alpha
            .iter()
            .find(|(ch, _)| *ch == AffectChannel::Residual)
            .map(|(_, a)| *a)
            .unwrap_or(1.0);
        match &self.steering_engine {
            Some(engine) => {
                let mut dir = direction.clone();
                dir.alpha *= alpha;
                engine.steer_generation(&dir, target);
            }
            None => {
                let len = target.len().min(direction.composite_vector.len());
                for i in 0..len {
                    let t_val = target[i];
                    let d_val = direction.composite_vector[i];
                    target[i] = (t_val as f64 * (1.0 - alpha) + d_val as f64 * alpha).round() as u8;
                }
            }
        }
    }

    pub fn inject_sampling(
        &self,
        state: &EmotionState,
        base_probability: f64,
        dimension: EmotionalDimension,
    ) -> f64 {
        if !self.channels.contains(&AffectChannel::Sampling) {
            return base_probability;
        }
        let emotion_val = state.get(dimension);
        let shift = (emotion_val - 0.5) * 0.2;
        let alpha = self
            .per_channel_alpha
            .iter()
            .find(|(ch, _)| *ch == AffectChannel::Sampling)
            .map(|(_, a)| *a)
            .unwrap_or(1.0);
        (base_probability + shift * alpha).clamp(0.0, 1.0)
    }

    pub fn inject_context(
        &self,
        state: &EmotionState,
        attention_scores: &mut [f64],
        dimension: EmotionalDimension,
    ) {
        if !self.channels.contains(&AffectChannel::Context) {
            return;
        }
        let emotion_val = state.get(dimension);
        let alpha = self
            .per_channel_alpha
            .iter()
            .find(|(ch, _)| *ch == AffectChannel::Context)
            .map(|(_, a)| *a)
            .unwrap_or(1.0);
        let factor = 1.0 + (emotion_val - 0.5) * 0.3 * alpha;
        for score in attention_scores.iter_mut() {
            *score *= factor;
        }
    }

    pub fn inject_all(
        &self,
        state: &EmotionState,
        target_vsa: &mut [u8],
        context_vsa: &[u8],
        decisions: &mut [f64],
        attention: &mut [f64],
    ) {
        let direction = self.direction_from_emotion(state, context_vsa);
        if self.channels.contains(&AffectChannel::Residual) {
            self.inject_residual(&direction, target_vsa, state);
        }
        if self.channels.contains(&AffectChannel::Sampling) {
            let dominant = state.dominant();
            for d in decisions.iter_mut() {
                *d = self.inject_sampling(state, *d, dominant);
            }
        }
        if self.channels.contains(&AffectChannel::Context) {
            let dominant = state.dominant();
            self.inject_context(state, attention, dominant);
        }
    }

    pub fn report(&self) -> String {
        let mut s = String::from("AffectInjection Report:\n");
        for ch in &[
            AffectChannel::Residual,
            AffectChannel::Sampling,
            AffectChannel::Context,
        ] {
            let enabled = if self.channels.contains(ch) {
                "enabled"
            } else {
                "disabled"
            };
            let alpha = self
                .per_channel_alpha
                .iter()
                .find(|(c, _)| c == ch)
                .map(|(_, a)| format!("{:.2}", a))
                .unwrap_or_else(|| "0.00".to_string());
            s.push_str(&format!("  {:?}: {} (alpha={})\n", ch, enabled, alpha));
        }
        match &self.steering_engine {
            Some(engine) => {
                s.push_str(&format!("  Steering Engine: enabled={}", engine.enabled));
            }
            None => {
                s.push_str("  Steering Engine: None");
            }
        }
        s
    }
}

impl Default for AffectInjection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EmotionSnapshot {
    pub state: EmotionState,
    pub tick: u64,
    pub trigger: String,
}

#[derive(Debug, Clone)]
pub struct EmotionalTrail {
    pub history: Vec<EmotionSnapshot>,
    pub max_len: usize,
}

impl EmotionalTrail {
    pub fn new(max_len: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_len),
            max_len,
        }
    }

    pub fn record(&mut self, state: &EmotionState, tick: u64, trigger: &str) {
        if self.history.len() >= self.max_len {
            self.history.remove(0);
        }
        self.history.push(EmotionSnapshot {
            state: state.clone(),
            tick,
            trigger: trigger.to_string(),
        });
    }

    pub fn trend(&self, dim: EmotionalDimension) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 0.0;
        }
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;
        for (i, snapshot) in self.history.iter().enumerate() {
            let x = i as f64;
            let y = snapshot.state.get(dim);
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }
        let nf = n as f64;
        let denom = nf * sum_xx - sum_x * sum_x;
        if denom.abs() < 1e-10 {
            return 0.0;
        }
        (nf * sum_xy - sum_x * sum_y) / denom
    }

    pub fn volatility(&self) -> f64 {
        let n = self.history.len();
        if n < 2 {
            return 0.0;
        }
        let values: Vec<f64> = self
            .history
            .iter()
            .map(|s| s.state.get(s.state.dominant()))
            .collect();
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        variance.sqrt()
    }

    pub fn last_n(&self, n: usize) -> Vec<&EmotionSnapshot> {
        let start = if n >= self.history.len() {
            0
        } else {
            self.history.len() - n
        };
        self.history[start..].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa() -> Vec<u8> {
        QuantizedVSA::seeded_random(9999, VSA_DIM)
    }

    #[test]
    fn test_emotion_state_create() {
        let s = EmotionState::new();
        assert!((s.curiosity - 0.5).abs() < 1e-6);
        assert!((s.satisfaction - 0.5).abs() < 1e-6);
        assert!((s.frustration - 0.5).abs() < 1e-6);
        assert!((s.energy - 0.5).abs() < 1e-6);
        assert!((s.loneliness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_dominant_emotion() {
        let mut s = EmotionState::new();
        s.set(EmotionalDimension::Curiosity, 0.9);
        assert_eq!(s.dominant(), EmotionalDimension::Curiosity);
        s.set(EmotionalDimension::Satisfaction, 0.95);
        assert_eq!(s.dominant(), EmotionalDimension::Satisfaction);
    }

    #[test]
    fn test_apply_event_success() {
        let mut steering = EmotionalSteering::new();
        steering.apply_event(EmotionalEvent::Success { magnitude: 1.0 });
        assert!(
            steering.state.satisfaction > 0.5,
            "success should boost satisfaction"
        );
        assert!(
            steering.state.frustration < 0.5,
            "success should lower frustration"
        );
    }

    #[test]
    fn test_exploration_bonus() {
        let mut steering = EmotionalSteering::new();
        steering.state.curiosity = 0.9;
        steering.state.frustration = 0.1;
        let bonus = steering.exploration_bonus();
        assert!((bonus - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_direction_from_emotion() {
        let injection = AffectInjection::new();
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Curiosity, 0.9);
        let ctx = test_vsa();
        let direction = injection.direction_from_emotion(&state, &ctx);
        assert_eq!(direction.dominant_emotion, "Curiosity");
        assert_eq!(direction.composite_vector.len(), VSA_DIM);
        assert!(direction.alpha > 0.0);
    }

    #[test]
    fn test_inject_residual_changes_vsa() {
        let steering = CaaSteeringEngine::new();
        let injection = AffectInjection {
            channels: vec![AffectChannel::Residual],
            per_channel_alpha: vec![(AffectChannel::Residual, 0.5)],
            steering_engine: Some(steering),
        };
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Curiosity, 0.9);
        let ctx = test_vsa();
        let direction = injection.direction_from_emotion(&state, &ctx);
        let mut target = test_vsa();
        let before = target.clone();
        injection.inject_residual(&direction, &mut target, &state);
        assert_ne!(
            target, before,
            "residual injection should mutate target VSA"
        );
    }

    #[test]
    fn test_inject_sampling_biases_probability() {
        let injection = AffectInjection {
            channels: vec![AffectChannel::Sampling],
            per_channel_alpha: vec![(AffectChannel::Sampling, 1.0)],
            steering_engine: None,
        };
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Energy, 0.9);
        let biased = injection.inject_sampling(&state, 0.5, EmotionalDimension::Energy);
        assert!(biased > 0.5, "high energy should increase probability bias");
        assert!(biased <= 1.0);
    }

    #[test]
    fn test_inject_context_rewights_attention() {
        let injection = AffectInjection {
            channels: vec![AffectChannel::Context],
            per_channel_alpha: vec![(AffectChannel::Context, 1.0)],
            steering_engine: None,
        };
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Satisfaction, 0.9);
        let mut scores = vec![1.0, 1.0, 1.0];
        injection.inject_context(&state, &mut scores, EmotionalDimension::Satisfaction);
        for s in &scores {
            assert!(*s > 1.0, "high satisfaction should amplify attention");
        }
    }

    #[test]
    fn test_inject_all_three_channels() {
        let injection = AffectInjection::new();
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Curiosity, 0.8);
        state.set(EmotionalDimension::Energy, 0.7);
        state.set(EmotionalDimension::Satisfaction, 0.6);
        let ctx = test_vsa();
        let mut target = test_vsa();
        let target_before = target.clone();
        let mut decisions = vec![0.5, 0.5];
        let decisions_before = decisions.clone();
        let mut attention = vec![1.0, 1.0, 1.0];
        let attention_before = attention.clone();
        injection.inject_all(&state, &mut target, &ctx, &mut decisions, &mut attention);
        assert_ne!(target, target_before, "inject_all should change VSA");
        assert_ne!(
            decisions, decisions_before,
            "inject_all should bias decisions"
        );
        assert_ne!(
            attention, attention_before,
            "inject_all should reweight attention"
        );
    }

    #[test]
    fn test_emotional_trail_record() {
        let mut trail = EmotionalTrail::new(10);
        let state = EmotionState::new();
        trail.record(&state, 0, "test");
        assert_eq!(trail.history.len(), 1);
        assert_eq!(trail.history[0].tick, 0);
        assert_eq!(trail.history[0].trigger, "test");
    }

    #[test]
    fn test_emotional_trail_trend() {
        let mut trail = EmotionalTrail::new(10);
        for i in 0..5 {
            let mut state = EmotionState::new();
            state.set(EmotionalDimension::Curiosity, 0.5 + i as f64 * 0.1);
            trail.record(&state, i as u64, "tick");
        }
        let slope = trail.trend(EmotionalDimension::Curiosity);
        assert!(
            slope > 0.0,
            "increasing emotion should yield positive trend"
        );
    }

    #[test]
    fn test_volatility_positive() {
        let mut trail = EmotionalTrail::new(10);
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Curiosity, 0.9);
        trail.record(&state, 0, "high");
        state.set(EmotionalDimension::Curiosity, 0.3);
        trail.record(&state, 1, "low");
        state.set(EmotionalDimension::Curiosity, 0.8);
        trail.record(&state, 2, "high");
        let vol = trail.volatility();
        assert!(vol >= 0.0, "volatility should be non-negative");
        assert!(
            vol > 0.0,
            "varying emotion should produce positive volatility"
        );
    }

    #[test]
    fn test_channel_toggle() {
        let mut injection = AffectInjection::new();
        injection.set_channel(AffectChannel::Residual, 0.0);
        let mut state = EmotionState::new();
        state.set(EmotionalDimension::Curiosity, 0.9);
        let ctx = test_vsa();
        let direction = injection.direction_from_emotion(&state, &ctx);
        let mut target = test_vsa();
        let before = target.clone();
        injection.inject_residual(&direction, &mut target, &state);
        assert_eq!(
            target, before,
            "disabled residual channel should not change target"
        );
    }

    #[test]
    fn test_emotional_trail_last_n() {
        let mut trail = EmotionalTrail::new(10);
        for i in 0..5 {
            trail.record(&EmotionState::new(), i, &format!("tick-{}", i));
        }
        let last = trail.last_n(3);
        assert_eq!(last.len(), 3);
        assert_eq!(last[0].tick, 2);
        assert_eq!(last[2].tick, 4);
    }

    #[test]
    fn test_emotional_steering_update() {
        let mut steering = EmotionalSteering::new();
        steering.state.curiosity = 0.1;
        steering.state.satisfaction = 0.9;
        steering.update(1.0);
        assert!(
            steering.state.curiosity > 0.1,
            "decay should move curiosity toward baseline"
        );
        assert!(
            steering.state.satisfaction < 0.9,
            "decay should move satisfaction toward baseline"
        );
    }

    #[test]
    fn test_affect_injection_report() {
        let injection = AffectInjection::new();
        let report = injection.report();
        assert!(report.contains("Residual"));
        assert!(report.contains("enabled"));
        assert!(report.contains("Steering Engine"));
    }

    #[test]
    fn test_emotional_trail_max_len() {
        let mut trail = EmotionalTrail::new(3);
        for i in 0..5 {
            trail.record(&EmotionState::new(), i, "overflow");
        }
        assert_eq!(trail.history.len(), 3);
        assert_eq!(trail.history[0].tick, 2);
    }

    #[test]
    fn test_volatility_zero_for_flat() {
        let mut trail = EmotionalTrail::new(5);
        for i in 0..3 {
            trail.record(&EmotionState::new(), i, "flat");
        }
        let vol = trail.volatility();
        assert!((vol - 0.0).abs() < 1e-10);
    }
}
