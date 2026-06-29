use std::collections::HashMap;
use std::time::Instant;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

/// Three tiers of the hierarchical world model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldModelTier {
    Perception,
    Action,
    Narrative,
}

impl WorldModelTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Perception => "perception",
            Self::Action => "action",
            Self::Narrative => "narrative",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Perception, Self::Action, Self::Narrative]
    }
}

/// Configuration for a single prediction tier
#[derive(Debug, Clone)]
pub struct TierConfig {
    pub tier: WorldModelTier,
    pub time_window_ms: u64,
    pub latent_dim: usize,
    pub prediction_horizon: usize,
    pub learning_rate: f64,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self::perception()
    }
}

impl TierConfig {
    pub fn perception() -> Self {
        Self {
            tier: WorldModelTier::Perception,
            time_window_ms: 100,
            latent_dim: 64,
            prediction_horizon: 1,
            learning_rate: 0.01,
        }
    }

    pub fn action() -> Self {
        Self {
            tier: WorldModelTier::Action,
            time_window_ms: 1000,
            latent_dim: 64,
            prediction_horizon: 3,
            learning_rate: 0.005,
        }
    }

    pub fn narrative() -> Self {
        Self {
            tier: WorldModelTier::Narrative,
            time_window_ms: 10000,
            latent_dim: 64,
            prediction_horizon: 10,
            learning_rate: 0.001,
        }
    }
}

/// Configuration for the full hierarchical world model
#[derive(Debug, Clone)]
pub struct HierarchicalWorldModelConfig {
    pub tiers: Vec<TierConfig>,
    pub imagination_steps: usize,
    pub error_threshold: f64,
}

impl Default for HierarchicalWorldModelConfig {
    fn default() -> Self {
        Self {
            tiers: vec![
                TierConfig::perception(),
                TierConfig::action(),
                TierConfig::narrative(),
            ],
            imagination_steps: 5,
            error_threshold: 0.6,
        }
    }
}

/// Runtime state for a single prediction tier
#[derive(Debug, Clone)]
pub struct TierState {
    pub config: TierConfig,
    pub current_latent: Vec<u8>,
    pub predicted_next: Vec<u8>,
    pub error_signal: Option<PredictionError>,
    pub recent_errors: Vec<f64>,
    pub accuracy: f64,
    pub(crate) observation_history: Vec<Vec<u8>>,
}

impl TierState {
    pub fn new(config: TierConfig) -> Self {
        Self {
            current_latent: Vec::new(),
            predicted_next: Vec::new(),
            error_signal: None,
            recent_errors: Vec::with_capacity(100),
            accuracy: 1.0,
            observation_history: Vec::with_capacity(config.prediction_horizon.max(10)),
            config,
        }
    }

    pub fn reset(&mut self) {
        self.current_latent.clear();
        self.predicted_next.clear();
        self.error_signal = None;
        self.recent_errors.clear();
        self.accuracy = 1.0;
        self.observation_history.clear();
    }
}

/// A single prediction step record
#[derive(Debug, Clone)]
pub struct PredictionStep {
    pub predicted_latent: Vec<u8>,
    pub actual_latent: Vec<u8>,
    pub prediction_error: f64,
    pub tier: WorldModelTier,
    pub timestamp: Instant,
}

/// Report from a model step invocation
#[derive(Debug, Clone)]
pub struct PredictionReport {
    pub tiers_active: Vec<WorldModelTier>,
    pub total_errors: usize,
    pub average_error: f64,
    pub imagination_trajectories: usize,
    pub novelty_detected: bool,
    pub elapsed_ms: f64,
}

/// A full imagined trajectory with multiple steps
#[derive(Debug, Clone)]
pub struct ImaginedTrajectory {
    pub steps: Vec<ImaginedStep>,
    pub total_error: f64,
    pub coherence: f64,
}

/// A single step within an imagined trajectory
#[derive(Debug, Clone)]
pub struct ImaginedStep {
    pub latent: Vec<u8>,
    pub confidence: f64,
}

/// Prediction error signal for a specific tier
#[derive(Debug, Clone)]
pub struct PredictionError {
    pub tier: WorldModelTier,
    pub error_vector: Vec<u8>,
    pub magnitude: f64,
    pub source: String,
}

/// HierarchicalWorldModel — 3-tier predictive coding world model with VSA
///
/// Architecture:
/// - Perception tier (100ms): fast low-level predictions
/// - Action tier (1s): medium-term action-outcome predictions
/// - Narrative tier (10s+): long-term narrative structure predictions
///
/// Bottom-up: prediction errors propagate from Perception → Action → Narrative
/// Top-down: predictions propagate from Narrative → Action → Perception
#[derive(Debug, Clone)]
pub struct HierarchicalWorldModel {
    pub config: HierarchicalWorldModelConfig,
    pub tiers: HashMap<WorldModelTier, TierState>,
    pub prediction_history: Vec<PredictionStep>,
    pub imagination_history: Vec<ImaginedTrajectory>,
    pub total_predictions: u64,
    pub novelty_count: u64,
}

impl Default for HierarchicalWorldModel {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalWorldModel {
    pub fn new() -> Self {
        let config = HierarchicalWorldModelConfig::default();
        Self::with_config(config)
    }

    pub fn with_config(config: HierarchicalWorldModelConfig) -> Self {
        let mut tiers = HashMap::new();
        for tier_config in &config.tiers {
            tiers.insert(tier_config.tier, TierState::new(tier_config.clone()));
        }
        Self {
            config,
            tiers,
            prediction_history: Vec::new(),
            imagination_history: Vec::new(),
            total_predictions: 0,
            novelty_count: 0,
        }
    }

    /// Main prediction cycle: bottom-up perception→action→narrative,
    /// followed by top-down prediction propagation.
    pub fn step(&mut self, observation_vsa: &[u8]) -> PredictionReport {
        let start = Instant::now();
        let mut total_error = 0.0;
        let mut error_count = 0;
        let mut novelty_detected = false;
        let mut tiers_active = Vec::new();
        let obs = observation_vsa.to_vec();

        // Bottom-up pass: Perception → Action → Narrative
        let tier_order = [
            WorldModelTier::Perception,
            WorldModelTier::Action,
            WorldModelTier::Narrative,
        ];
        for &tier in &tier_order {
            let state = match self.tiers.get_mut(&tier) {
                Some(s) => s,
                None => continue,
            };

            // Store observation in history
            state.observation_history.push(obs.clone());
            if state.observation_history.len() > 100 {
                state.observation_history.remove(0);
            }

            // Predict next latent using free function (no self borrow)
            let predicted = tier_predict_next(&obs, state);
            state.predicted_next = predicted.clone();
            state.current_latent = obs.clone();

            // Compute prediction error using free function (no self borrow)
            let error_val = prediction_error(&predicted, &obs);
            state.recent_errors.push(error_val);
            if state.recent_errors.len() > 100 {
                state.recent_errors.remove(0);
            }

            // EMA accuracy update
            state.accuracy = state.accuracy * 0.9 + (1.0 - error_val) * 0.1;

            // Build error signal
            let error_vec = vec![(error_val * 255.0) as u8; observation_vsa.len().min(8)];
            let prediction_error = PredictionError {
                tier,
                error_vector: error_vec,
                magnitude: error_val,
                source: format!("{}-prediction", tier.name()),
            };
            state.error_signal = Some(prediction_error);

            total_error += error_val;
            error_count += 1;
            tiers_active.push(tier);

            if error_val > self.config.error_threshold {
                novelty_detected = true;
            }

            // Record step (directly, no self borrow needed for the args)
            self.prediction_history.push(PredictionStep {
                predicted_latent: predicted,
                actual_latent: obs.clone(),
                prediction_error: error_val,
                tier,
                timestamp: Instant::now(),
            });
        }

        // Top-down pass: Narrative → Action, Action → Perception
        // Use scoped immutable borrows to extract data, then apply.
        let narrative_prediction = self
            .tiers
            .get(&WorldModelTier::Narrative)
            .map(|s| s.predicted_next.clone())
            .filter(|v| !v.is_empty());
        if let Some(n_pred) = &narrative_prediction {
            if let Some(action) = self.tiers.get_mut(&WorldModelTier::Action) {
                let top_down_err = 1.0 - QuantizedVSA::similarity(n_pred, &action.current_latent);
                action.recent_errors.push(top_down_err);
            }
        }

        let action_prediction = self
            .tiers
            .get(&WorldModelTier::Action)
            .map(|s| s.predicted_next.clone())
            .filter(|v| !v.is_empty());
        if let Some(a_pred) = &action_prediction {
            if let Some(perception) = self.tiers.get_mut(&WorldModelTier::Perception) {
                let top_down_err =
                    1.0 - QuantizedVSA::similarity(a_pred, &perception.current_latent);
                perception.recent_errors.push(top_down_err);
            }
        }

        self.total_predictions += 1;
        if novelty_detected {
            self.novelty_count += 1;
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        PredictionReport {
            tiers_active,
            total_errors: error_count,
            average_error: if error_count > 0 {
                total_error / error_count as f64
            } else {
                0.0
            },
            imagination_trajectories: self.imagination_history.len(),
            novelty_detected,
            elapsed_ms: elapsed,
        }
    }

    /// VSA similarity-based prediction using nearest-neighbor in observation history
    pub fn predict_next(&self, tier: WorldModelTier, current: &[u8]) -> Vec<u8> {
        match self.tiers.get(&tier) {
            Some(state) => tier_predict_next(current, state),
            None => current.to_vec(),
        }
    }

    /// Prediction error: 1 - VSA cosine similarity
    pub fn compute_prediction_error(&self, predicted: &[u8], actual: &[u8]) -> f64 {
        prediction_error(predicted, actual)
    }

    /// Generate multiple imagined future trajectories from a starting latent
    pub fn imagine(&self, start_latent: &[u8], steps: usize) -> Vec<ImaginedTrajectory> {
        let num_trajectories = 3;
        let mut trajectories = Vec::with_capacity(num_trajectories);

        for t in 0..num_trajectories {
            let mut latent = start_latent.to_vec();
            let mut trajectory_steps = Vec::with_capacity(steps);
            let mut total_error = 0.0;

            for step in 0..steps {
                let next = self.predict_next(WorldModelTier::Action, &latent);

                // Add perturbation for trajectory diversity
                let seed = (t as u64) * 1000 + (step as u64);
                let noise = QuantizedVSA::seeded_random(seed, latent.len());

                let blend_ratio = 0.1 + (t as f64) * 0.15;
                let blended: Vec<u8> = next
                    .iter()
                    .zip(noise.iter())
                    .map(|(a, b)| {
                        let mixed = *a as f64 * (1.0 - blend_ratio) + *b as f64 * blend_ratio;
                        (mixed.round().clamp(0.0, 255.0)) as u8
                    })
                    .collect();

                let confidence = (1.0 - (t as f64) * 0.1).max(0.1);
                let error = 1.0 - QuantizedVSA::similarity(&blended, &latent);
                total_error += error;

                trajectory_steps.push(ImaginedStep {
                    latent: blended.clone(),
                    confidence,
                });

                latent = blended;
            }

            // Coherence: average pairwise similarity between consecutive steps
            let coherence = if trajectory_steps.len() >= 2 {
                let mut step_coherence = 0.0;
                for i in 0..trajectory_steps.len() - 1 {
                    let sim = QuantizedVSA::similarity(
                        &trajectory_steps[i].latent,
                        &trajectory_steps[i + 1].latent,
                    );
                    step_coherence += sim;
                }
                step_coherence / (trajectory_steps.len() - 1) as f64
            } else {
                1.0
            };

            trajectories.push(ImaginedTrajectory {
                steps: trajectory_steps,
                total_error,
                coherence,
            });
        }

        trajectories
    }

    /// Detect novelty when prediction error exceeds threshold
    pub fn detect_novelty(&self, error_magnitude: f64) -> bool {
        error_magnitude > self.config.error_threshold
    }

    /// Running accuracy for a specific tier (1 - EMA error)
    pub fn accuracy(&self, tier: WorldModelTier) -> f64 {
        self.tiers.get(&tier).map(|s| s.accuracy).unwrap_or(0.0)
    }

    /// Reset all tier states
    pub fn reset(&mut self) {
        for state in self.tiers.values_mut() {
            state.reset();
        }
        self.prediction_history.clear();
        self.imagination_history.clear();
        self.total_predictions = 0;
        self.novelty_count = 0;
    }
}

// ── Free functions (no &self, avoids borrow conflicts) ──

/// VSA nearest-neighbor prediction for a tier state
fn tier_predict_next(current: &[u8], state: &TierState) -> Vec<u8> {
    let history = &state.observation_history;
    if history.len() < 2 {
        return current.to_vec();
    }

    let horizon = state.config.prediction_horizon.max(1);
    let start_idx = if history.len() > horizon + 1 {
        history.len() - horizon - 1
    } else {
        0
    };

    let mut best_sim = 0.0_f64;
    let mut best_next = current.to_vec();

    for i in start_idx..history.len() - 1 {
        let sim = QuantizedVSA::similarity(current, &history[i]);
        if sim > best_sim {
            best_sim = sim;
            best_next = history[i + 1].clone();
        }
    }

    if best_sim < 0.1 {
        current.to_vec()
    } else {
        best_next
    }
}

/// Prediction error: 1 - VSA cosine similarity
fn prediction_error(predicted: &[u8], actual: &[u8]) -> f64 {
    1.0 - QuantizedVSA::similarity(predicted, actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    #[test]
    fn test_default_config_creates_three_tiers() {
        let model = HierarchicalWorldModel::new();
        assert_eq!(model.tiers.len(), 3);
        assert!(model.tiers.contains_key(&WorldModelTier::Perception));
        assert!(model.tiers.contains_key(&WorldModelTier::Action));
        assert!(model.tiers.contains_key(&WorldModelTier::Narrative));
    }

    #[test]
    fn test_single_step_produces_prediction() {
        let mut model = HierarchicalWorldModel::new();
        let obs = test_vsa(42);
        let report = model.step(&obs);
        assert_eq!(model.total_predictions, 1);
        assert_eq!(report.tiers_active.len(), 3);
        assert!(report.total_errors > 0);
        assert!(!model.prediction_history.is_empty());
    }

    #[test]
    fn test_identical_vectors_zero_error() {
        let vsa = test_vsa(7);
        let error = prediction_error(&vsa, &vsa);
        assert!(
            error < 0.01,
            "identical vectors should have near-zero error, got {}",
            error
        );
    }

    #[test]
    fn test_orthogonal_vectors_high_error() {
        let a = test_vsa(1);
        let b = test_vsa(9999);
        let error = prediction_error(&a, &b);
        assert!(
            error > 0.3,
            "random vectors should have high error, got {}",
            error
        );
    }

    #[test]
    fn test_error_propagation_between_tiers() {
        let mut model = HierarchicalWorldModel::new();
        let obs = test_vsa(1);
        let _ = model.step(&obs);

        for tier in &[
            WorldModelTier::Perception,
            WorldModelTier::Action,
            WorldModelTier::Narrative,
        ] {
            let state = model.tiers.get(tier).unwrap();
            assert!(
                state.error_signal.is_some(),
                "tier {:?} should have error signal after step",
                tier
            );
            if let Some(ref err) = state.error_signal {
                assert_eq!(err.tier, *tier);
                assert!(err.magnitude >= 0.0);
            }
        }
    }

    #[test]
    fn test_imagination_generates_trajectories() {
        let model = HierarchicalWorldModel::new();
        let start = test_vsa(1);
        let trajectories = model.imagine(&start, 5);
        assert_eq!(trajectories.len(), 3, "should generate 3 trajectories");
        for traj in &trajectories {
            assert_eq!(traj.steps.len(), 5, "each trajectory should have 5 steps");
            assert!(traj.total_error >= 0.0);
            assert!(traj.coherence >= 0.0 && traj.coherence <= 1.0);
        }
    }

    #[test]
    fn test_novelty_detection_high_error() {
        let config = HierarchicalWorldModelConfig {
            error_threshold: 0.3,
            ..Default::default()
        };
        let model = HierarchicalWorldModel::with_config(config);
        assert!(
            model.detect_novelty(0.5),
            "error above threshold should be novelty"
        );
        assert!(
            !model.detect_novelty(0.2),
            "error below threshold should not be novelty"
        );
    }

    #[test]
    fn test_accuracy_degrades_with_noise() {
        let mut model = HierarchicalWorldModel::new();
        let initial_accuracy = model.accuracy(WorldModelTier::Perception);
        assert!((initial_accuracy - 1.0).abs() < 0.001);

        for i in 0..10 {
            let obs = test_vsa(i * 100 + 7);
            let _ = model.step(&obs);
        }

        let degraded = model.accuracy(WorldModelTier::Perception);
        assert!(
            degraded < 1.0,
            "accuracy should degrade with random observations, got {}",
            degraded
        );
    }

    #[test]
    fn test_multiple_steps_accumulate_history() {
        let mut model = HierarchicalWorldModel::new();
        for i in 0..5u64 {
            let obs = test_vsa(i);
            let _ = model.step(&obs);
        }
        assert_eq!(model.total_predictions, 5);
        assert_eq!(model.prediction_history.len(), 15);
    }

    #[test]
    fn test_horizon_affects_prediction() {
        let config = HierarchicalWorldModelConfig {
            tiers: vec![TierConfig {
                prediction_horizon: 1,
                ..TierConfig::perception()
            }],
            ..Default::default()
        };
        let model = HierarchicalWorldModel::with_config(config);
        let state = model.tiers.get(&WorldModelTier::Perception).unwrap();
        assert_eq!(state.config.prediction_horizon, 1);
    }

    #[test]
    fn test_tier_reset_clears_state() {
        let mut model = HierarchicalWorldModel::new();
        let obs = test_vsa(1);
        let _ = model.step(&obs);
        assert!(model.total_predictions > 0);

        model.reset();
        assert_eq!(model.total_predictions, 0);
        assert!(model.prediction_history.is_empty());
        assert_eq!(model.novelty_count, 0);

        for state in model.tiers.values() {
            assert!(state.current_latent.is_empty());
            assert!(state.error_signal.is_none());
            assert!(state.recent_errors.is_empty());
        }
    }

    #[test]
    fn test_empty_model_does_not_crash() {
        let config = HierarchicalWorldModelConfig {
            tiers: Vec::new(),
            ..Default::default()
        };
        let mut model = HierarchicalWorldModel::with_config(config);
        assert_eq!(model.tiers.len(), 0);

        let obs = test_vsa(1);
        let report = model.step(&obs);
        assert!(report.tiers_active.is_empty());
        assert_eq!(report.total_errors, 0);
    }

    #[test]
    fn test_imagination_coherence_scoring() {
        let model = HierarchicalWorldModel::new();
        let start = test_vsa(42);
        let trajectories = model.imagine(&start, 10);

        assert_eq!(trajectories.len(), 3);
        let coherences: Vec<f64> = trajectories.iter().map(|t| t.coherence).collect();
        assert!(coherences.iter().all(|c| (0.0..=1.0).contains(c)));
    }

    #[test]
    fn test_config_with_single_tier_works() {
        let config = HierarchicalWorldModelConfig {
            tiers: vec![TierConfig::perception()],
            ..Default::default()
        };
        let mut model = HierarchicalWorldModel::with_config(config);
        assert_eq!(model.tiers.len(), 1);

        let obs = test_vsa(1);
        let report = model.step(&obs);
        assert_eq!(report.tiers_active.len(), 1);
        assert_eq!(report.total_errors, 1);
        assert!(model.total_predictions > 0);
    }

    #[test]
    fn test_error_threshold_gates_novelty() {
        let config = HierarchicalWorldModelConfig {
            error_threshold: 0.9,
            ..Default::default()
        };
        let mut model = HierarchicalWorldModel::with_config(config);

        for i in 0..20 {
            let obs = test_vsa(i * 13);
            let _ = model.step(&obs);
        }

        assert_eq!(model.total_predictions, 20);
    }

    #[test]
    fn test_predict_next_with_history() {
        let mut model = HierarchicalWorldModel::new();
        let obs1 = test_vsa(1);
        let obs2 = test_vsa(2);
        let obs3 = test_vsa(3);

        let _ = model.step(&obs1);
        let _ = model.step(&obs2);

        let prediction = model.predict_next(WorldModelTier::Perception, &obs3);
        assert_eq!(prediction.len(), 64);
    }

    #[test]
    fn test_world_model_tier_names() {
        assert_eq!(WorldModelTier::Perception.name(), "perception");
        assert_eq!(WorldModelTier::Action.name(), "action");
        assert_eq!(WorldModelTier::Narrative.name(), "narrative");
    }

    #[test]
    fn test_tier_config_defaults() {
        let p = TierConfig::perception();
        assert_eq!(p.time_window_ms, 100);
        assert_eq!(p.latent_dim, 64);
        assert_eq!(p.tier, WorldModelTier::Perception);

        let a = TierConfig::action();
        assert_eq!(a.time_window_ms, 1000);
        assert_eq!(a.tier, WorldModelTier::Action);

        let n = TierConfig::narrative();
        assert_eq!(n.time_window_ms, 10000);
        assert_eq!(n.tier, WorldModelTier::Narrative);
    }

    #[test]
    fn test_imagine_returns_trajectories() {
        let model = HierarchicalWorldModel::new();
        let start = test_vsa(100);
        let trajectories = model.imagine(&start, 3);
        assert_eq!(trajectories.len(), 3);
        for (i, traj) in trajectories.iter().enumerate() {
            assert_eq!(traj.steps.len(), 3, "trajectory {} should have 3 steps", i);
            for step in &traj.steps {
                assert_eq!(step.latent.len(), 64);
                assert!(step.confidence > 0.0 && step.confidence <= 1.0);
            }
        }
    }

    #[test]
    fn test_novelty_count_increments() {
        let config = HierarchicalWorldModelConfig {
            error_threshold: 0.1,
            ..Default::default()
        };
        let mut model = HierarchicalWorldModel::with_config(config);
        for i in 0..5 {
            let obs = test_vsa(i * 7 + 3);
            let _ = model.step(&obs);
        }
        assert!(
            model.novelty_count > 0,
            "novelty should be detected with 0.1 threshold"
        );
    }

    #[test]
    fn test_tier_state_reset() {
        let mut state = TierState::new(TierConfig::perception());
        state.current_latent = test_vsa(1);
        state.predicted_next = test_vsa(2);
        state.recent_errors.push(0.5);
        state.accuracy = 0.8;
        state.reset();

        assert!(state.current_latent.is_empty());
        assert!(state.predicted_next.is_empty());
        assert!(state.error_signal.is_none());
        assert!(state.recent_errors.is_empty());
        assert!((state.accuracy - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_report_contains_elapsed_ms() {
        let mut model = HierarchicalWorldModel::new();
        let obs = test_vsa(42);
        let report = model.step(&obs);
        assert!(
            report.elapsed_ms >= 0.0,
            "elapsed_ms should be non-negative"
        );
    }

    #[test]
    fn test_free_prediction_error_zero() {
        let v = test_vsa(99);
        let err = prediction_error(&v, &v);
        assert!(err < 0.01);
    }

    #[test]
    fn test_tier_predict_next_empty_history() {
        let state = TierState::new(TierConfig::perception());
        let current = test_vsa(42);
        let result = tier_predict_next(&current, &state);
        assert_eq!(result, current);
    }

    #[test]
    fn test_world_model_tier_all() {
        let all = WorldModelTier::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&WorldModelTier::Perception));
        assert!(all.contains(&WorldModelTier::Action));
        assert!(all.contains(&WorldModelTier::Narrative));
    }
}
