use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionDimension {
    Frustration,
    Confidence,
    Joy,
    Urgency,
    Curiosity,
    Fatigue,
}

impl EmotionDimension {
    pub fn all() -> [EmotionDimension; 6] {
        use EmotionDimension::*;
        [Frustration, Confidence, Joy, Urgency, Curiosity, Fatigue]
    }

    pub fn index(&self) -> usize {
        use EmotionDimension::*;
        match self {
            Frustration => 0,
            Confidence => 1,
            Joy => 2,
            Urgency => 3,
            Curiosity => 4,
            Fatigue => 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionConfig {
    pub alpha: f64,
    pub decay_rate: f64,
    pub max_history: usize,
}

impl Default for EmotionConfig {
    fn default() -> Self {
        Self {
            alpha: 0.3,
            decay_rate: 0.05,
            max_history: 200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    values: [f64; 6],
}

impl EmotionState {
    pub fn new() -> Self {
        Self { values: [0.5; 6] }
    }

    pub fn update(&mut self, dim: EmotionDimension, observed: f64) {
        let idx = dim.index();
        let observed = observed.max(0.0).min(1.0);
        self.values[idx] = (0.3 * observed + 0.7 * self.values[idx]).max(0.0).min(1.0);
    }

    pub fn decay(&mut self) {
        for v in self.values.iter_mut() {
            let delta = 0.05 * (0.5 - *v);
            *v = (*v + delta).max(0.0).min(1.0);
        }
    }

    pub fn get(&self, dim: EmotionDimension) -> f64 {
        self.values[dim.index()]
    }

    pub fn vector(&self) -> [f64; 6] {
        self.values
    }

    pub fn dominant(&self) -> (EmotionDimension, f64) {
        let mut max_dev = 0.0f64;
        let mut dominant = EmotionDimension::Frustration;
        for dim in EmotionDimension::all() {
            let dev = (self.get(dim) - 0.5).abs();
            if dev > max_dev {
                max_dev = dev;
                dominant = dim;
            }
        }
        (dominant, max_dev)
    }

    pub fn arousal(&self) -> f64 {
        (self.get(EmotionDimension::Frustration) + self.get(EmotionDimension::Urgency)) / 2.0
    }

    pub fn valence(&self) -> f64 {
        (self.get(EmotionDimension::Joy) + self.get(EmotionDimension::Confidence)
            - self.get(EmotionDimension::Frustration)
            - self.get(EmotionDimension::Fatigue))
            / 4.0
            + 0.5
    }

    pub fn confidence_score(&self) -> f64 {
        (0.3 * self.get(EmotionDimension::Confidence)
            + 0.2 * self.get(EmotionDimension::Joy)
            + 0.1 * (1.0 - self.get(EmotionDimension::Frustration))
            + 0.2 * self.get(EmotionDimension::Curiosity)
            + 0.1 * (1.0 - self.get(EmotionDimension::Fatigue))
            + 0.1 * self.get(EmotionDimension::Urgency))
        .max(0.0)
        .min(1.0)
    }
}

impl Default for EmotionState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionObservation {
    pub dimension: EmotionDimension,
    pub value: f64,
    pub trigger: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionReport {
    pub frustration: f64,
    pub confidence: f64,
    pub joy: f64,
    pub urgency: f64,
    pub curiosity: f64,
    pub fatigue: f64,
    pub arousal: f64,
    pub valence: f64,
    pub confidence_score: f64,
    pub dominant: (EmotionDimension, f64),
    pub observation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionEngine {
    pub state: EmotionState,
    pub config: EmotionConfig,
    pub history: VecDeque<EmotionObservation>,
}

impl EmotionEngine {
    pub fn new(config: EmotionConfig) -> Self {
        Self {
            state: EmotionState::new(),
            history: VecDeque::with_capacity(config.max_history),
            config,
        }
    }

    pub fn default() -> Self {
        Self::new(EmotionConfig::default())
    }

    pub fn observe(&mut self, dim: EmotionDimension, value: f64, trigger: impl Into<String>) {
        self.state.update(dim, value);
        let obs = EmotionObservation {
            dimension: dim,
            value,
            trigger: trigger.into(),
            timestamp: 0,
        };
        if self.history.len() >= self.config.max_history {
            self.history.pop_front();
        }
        self.history.push_back(obs);
    }

    pub fn tick(&mut self) {
        self.state.decay();
    }

    pub fn report(&self) -> EmotionReport {
        EmotionReport {
            frustration: self.state.get(EmotionDimension::Frustration),
            confidence: self.state.get(EmotionDimension::Confidence),
            joy: self.state.get(EmotionDimension::Joy),
            urgency: self.state.get(EmotionDimension::Urgency),
            curiosity: self.state.get(EmotionDimension::Curiosity),
            fatigue: self.state.get(EmotionDimension::Fatigue),
            arousal: self.state.arousal(),
            valence: self.state.valence(),
            confidence_score: self.state.confidence_score(),
            dominant: self.state.dominant(),
            observation_count: self.history.len(),
        }
    }

    pub fn recent_triggers(&self, n: usize) -> Vec<String> {
        self.history
            .iter()
            .rev()
            .take(n)
            .map(|o| o.trigger.clone())
            .collect()
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_all_half() {
        let state = EmotionState::new();
        for dim in EmotionDimension::all() {
            assert!((state.get(dim) - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_ema_convergence() {
        let mut state = EmotionState::new();
        for _ in 0..20 {
            state.update(EmotionDimension::Confidence, 0.9);
        }
        let val = state.get(EmotionDimension::Confidence);
        assert!(val > 0.85);
        assert!(val <= 1.0);
    }

    #[test]
    fn test_decay_to_baseline() {
        let mut state = EmotionState::new();
        state.update(EmotionDimension::Frustration, 0.9);
        for _ in 0..100 {
            state.decay();
        }
        let val = state.get(EmotionDimension::Frustration);
        assert!((val - 0.5).abs() < 0.05);
    }

    #[test]
    fn test_arousal_valence() {
        let mut state = EmotionState::new();
        state.update(EmotionDimension::Frustration, 0.8);
        state.update(EmotionDimension::Urgency, 0.7);
        assert!(state.arousal() > 0.5);
        state.update(EmotionDimension::Joy, 0.9);
        state.update(EmotionDimension::Confidence, 0.8);
        assert!(state.valence() > 0.5);
    }

    #[test]
    fn test_confidence_range() {
        let mut state = EmotionState::new();
        assert!((state.confidence_score() - 0.5).abs() < 0.1);
        state.update(EmotionDimension::Confidence, 1.0);
        state.update(EmotionDimension::Joy, 1.0);
        assert!(state.confidence_score() > 0.5);
    }

    #[test]
    fn test_engine_observe_and_report() {
        let mut engine = EmotionEngine::default();
        engine.observe(EmotionDimension::Fatigue, 0.7, "long session");
        engine.observe(EmotionDimension::Curiosity, 0.8, "new task");
        let report = engine.report();
        assert!(report.fatigue > 0.5);
        assert!(report.curiosity > 0.5);
        assert_eq!(report.observation_count, 2);
        let triggers = engine.recent_triggers(5);
        assert_eq!(triggers.len(), 2);
    }

    #[test]
    fn test_dominant_dimension() {
        let mut state = EmotionState::new();
        state.update(EmotionDimension::Joy, 0.95);
        let (dim, dev) = state.dominant();
        assert_eq!(dim, EmotionDimension::Joy);
        assert!(dev > 0.1);
    }

    #[test]
    fn test_clamp_on_update() {
        let mut state = EmotionState::new();
        state.update(EmotionDimension::Frustration, 2.0);
        assert!(state.get(EmotionDimension::Frustration) <= 1.0);
        state.update(EmotionDimension::Frustration, -1.0);
        assert!(state.get(EmotionDimension::Frustration) >= 0.0);
    }
}
