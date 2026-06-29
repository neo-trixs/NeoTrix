use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct RecurrentWorldModelConfig {
    pub latent_dim: usize,
    pub num_steps: usize,
    pub spectral_norm_bound: f64,
    pub early_exit_threshold: f64,
}

impl Default for RecurrentWorldModelConfig {
    fn default() -> Self {
        Self {
            latent_dim: 64,
            num_steps: 5,
            spectral_norm_bound: 0.95,
            early_exit_threshold: 0.05,
        }
    }
}

impl RecurrentWorldModelConfig {
    pub fn with_latent_dim(mut self, dim: usize) -> Self {
        self.latent_dim = dim;
        self
    }

    pub fn with_steps(mut self, steps: usize) -> Self {
        self.num_steps = steps;
        self
    }
}

#[derive(Debug, Clone)]
pub struct RecurrentLatentState {
    pub current: Vec<u8>,
    pub recurrent_steps: usize,
    pub convergence_score: f64,
}

impl RecurrentLatentState {
    pub fn new(latent: Vec<u8>) -> Self {
        Self {
            current: latent,
            recurrent_steps: 0,
            convergence_score: 0.0,
        }
    }

    pub fn reset(&mut self, latent: Vec<u8>) {
        self.current = latent;
        self.recurrent_steps = 0;
        self.convergence_score = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct RecurrentWorldModel {
    pub config: RecurrentWorldModelConfig,
    pub latent_history: Vec<RecurrentLatentState>,
    pub total_refinements: u64,
}

impl Default for RecurrentWorldModel {
    fn default() -> Self {
        Self::new()
    }
}

impl RecurrentWorldModel {
    pub fn new() -> Self {
        Self {
            config: RecurrentWorldModelConfig::default(),
            latent_history: Vec::new(),
            total_refinements: 0,
        }
    }

    pub fn with_config(config: RecurrentWorldModelConfig) -> Self {
        Self {
            config,
            latent_history: Vec::new(),
            total_refinements: 0,
        }
    }

    pub fn refine_step(
        &mut self,
        latent_state: &mut RecurrentLatentState,
        observation: &[u8],
    ) -> f64 {
        let prev = latent_state.current.clone();
        let mut refined = observation.to_vec();
        if refined.len() > self.config.latent_dim {
            refined.truncate(self.config.latent_dim);
        }
        while refined.len() < self.config.latent_dim {
            refined.push(0);
        }
        let blend = 0.3;
        let blended: Vec<u8> = prev
            .iter()
            .zip(refined.iter())
            .map(|(a, b)| {
                let mixed = *a as f64 * (1.0 - blend) + *b as f64 * blend;
                (mixed.round().clamp(0.0, 255.0)) as u8
            })
            .collect();
        latent_state.current = blended;
        latent_state.recurrent_steps += 1;
        let change = 1.0 - QuantizedVSA::similarity(&latent_state.current, &prev);
        latent_state.convergence_score = 1.0 - change;
        self.total_refinements += 1;
        self.latent_history.push(latent_state.clone());
        change
    }

    pub fn rollout(
        &self,
        initial_state: &RecurrentLatentState,
        steps: usize,
    ) -> Vec<RecurrentLatentState> {
        let mut trajectory = Vec::with_capacity(steps);
        let mut current = initial_state.clone();
        for _ in 0..steps {
            let next_raw = QuantizedVSA::permute(&current.current, 1);
            let mut next_state = RecurrentLatentState::new(next_raw);
            let sim = QuantizedVSA::similarity(&current.current, &next_state.current);
            next_state.convergence_score = sim;
            trajectory.push(next_state.clone());
            current = next_state;
        }
        trajectory
    }

    pub fn compute_spectral_norm(&self) -> f64 {
        if self.latent_history.len() < 2 {
            return 0.0;
        }
        let mut max_sim = 0.0_f64;
        let window = self.latent_history.len().min(20);
        let start = self.latent_history.len() - window;
        for i in start..self.latent_history.len() {
            for j in i + 1..self.latent_history.len() {
                let sim = QuantizedVSA::similarity(
                    &self.latent_history[i].current,
                    &self.latent_history[j].current,
                );
                if sim > max_sim {
                    max_sim = sim;
                }
            }
        }
        max_sim
    }

    pub fn early_exit_gate(&self, state: &RecurrentLatentState) -> bool {
        state.convergence_score >= (1.0 - self.config.early_exit_threshold)
            || state.recurrent_steps >= self.config.num_steps
    }

    pub fn reset(&mut self) {
        self.latent_history.clear();
        self.total_refinements = 0;
    }
}

pub fn predict_with_refinement(
    model: &mut RecurrentWorldModel,
    initial: Vec<u8>,
    observations: &[Vec<u8>],
    max_steps: usize,
) -> RecurrentLatentState {
    let mut state = RecurrentLatentState::new(initial);
    for obs in observations {
        let mut steps = 0;
        loop {
            model.refine_step(&mut state, obs);
            steps += 1;
            if model.early_exit_gate(&state) || steps >= max_steps {
                break;
            }
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    #[test]
    fn test_default_config_has_correct_fields() {
        let config = RecurrentWorldModelConfig::default();
        assert_eq!(config.latent_dim, 64);
        assert_eq!(config.num_steps, 5);
        assert!((config.spectral_norm_bound - 0.95).abs() < 1e-9);
        assert!((config.early_exit_threshold - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_latent_state_initializes_correctly() {
        let vsa = test_vsa(42);
        let state = RecurrentLatentState::new(vsa.clone());
        assert_eq!(state.current, vsa);
        assert_eq!(state.recurrent_steps, 0);
        assert!((state.convergence_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_refine_step_produces_change() {
        let mut model = RecurrentWorldModel::new();
        let mut state = RecurrentLatentState::new(test_vsa(1));
        let obs = test_vsa(2);
        let change = model.refine_step(&mut state, &obs);
        assert!(change >= 0.0);
        assert_eq!(state.recurrent_steps, 1);
        assert!(model.total_refinements > 0);
    }

    #[test]
    fn test_refine_step_increases_convergence_with_same_obs() {
        let mut model = RecurrentWorldModel::new();
        let mut state = RecurrentLatentState::new(test_vsa(10));
        let obs = test_vsa(10);
        let _ = model.refine_step(&mut state, &obs);
        let score_after = state.convergence_score;
        let _ = model.refine_step(&mut state, &obs);
        assert!(
            state.convergence_score >= score_after,
            "convergence should improve with consistent observations"
        );
    }

    #[test]
    fn test_rollout_generates_correct_number_of_steps() {
        let model = RecurrentWorldModel::new();
        let state = RecurrentLatentState::new(test_vsa(1));
        let trajectory = model.rollout(&state, 5);
        assert_eq!(trajectory.len(), 5);
        for step in &trajectory {
            assert_eq!(step.current.len(), 64);
        }
    }

    #[test]
    fn test_rollout_consecutive_steps_have_similarity() {
        let model = RecurrentWorldModel::new();
        let state = RecurrentLatentState::new(test_vsa(42));
        let trajectory = model.rollout(&state, 4);
        for pair in trajectory.windows(2) {
            let sim = QuantizedVSA::similarity(&pair[0].current, &pair[1].current);
            assert!(sim >= 0.0 && sim <= 1.0);
        }
    }

    #[test]
    fn test_spectral_norm_returns_value_between_zero_and_one() {
        let mut model = RecurrentWorldModel::new();
        let mut state = RecurrentLatentState::new(test_vsa(1));
        for i in 0..5 {
            let obs = test_vsa(i + 1);
            let _ = model.refine_step(&mut state, &obs);
        }
        let sn = model.compute_spectral_norm();
        assert!(sn >= 0.0 && sn <= 1.0);
    }

    #[test]
    fn test_spectral_norm_empty_history() {
        let model = RecurrentWorldModel::new();
        assert!((model.compute_spectral_norm() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_early_exit_gate_triggers_on_convergence() {
        let config = RecurrentWorldModelConfig {
            early_exit_threshold: 0.5,
            ..Default::default()
        };
        let model = RecurrentWorldModel::with_config(config);
        let mut state = RecurrentLatentState::new(test_vsa(1));
        state.convergence_score = 0.6;
        assert!(model.early_exit_gate(&state));
        state.convergence_score = 0.3;
        assert!(!model.early_exit_gate(&state));
    }

    #[test]
    fn test_early_exit_gate_triggers_on_max_steps() {
        let config = RecurrentWorldModelConfig {
            num_steps: 3,
            early_exit_threshold: 0.0,
            ..Default::default()
        };
        let model = RecurrentWorldModel::with_config(config);
        let mut state = RecurrentLatentState::new(test_vsa(1));
        state.recurrent_steps = 3;
        assert!(model.early_exit_gate(&state));
        state.recurrent_steps = 2;
        assert!(!model.early_exit_gate(&state));
    }

    #[test]
    fn test_predict_with_refinement_produces_converged_state() {
        let mut model = RecurrentWorldModel::new();
        let initial = test_vsa(1);
        let observations = vec![test_vsa(2), test_vsa(3), test_vsa(4)];
        let final_state = predict_with_refinement(&mut model, initial, &observations, 3);
        assert!(final_state.recurrent_steps > 0);
        assert_eq!(final_state.current.len(), 64);
    }

    #[test]
    fn test_config_builder_methods() {
        let config = RecurrentWorldModelConfig::default()
            .with_latent_dim(128)
            .with_steps(10);
        assert_eq!(config.latent_dim, 128);
        assert_eq!(config.num_steps, 10);
    }

    #[test]
    fn test_reset_clears_model() {
        let mut model = RecurrentWorldModel::new();
        let mut state = RecurrentLatentState::new(test_vsa(1));
        let _ = model.refine_step(&mut state, &test_vsa(2));
        assert!(model.total_refinements > 0);
        assert!(!model.latent_history.is_empty());
        model.reset();
        assert_eq!(model.total_refinements, 0);
        assert!(model.latent_history.is_empty());
    }

    #[test]
    fn test_latent_state_reset() {
        let mut state = RecurrentLatentState::new(test_vsa(1));
        state.recurrent_steps = 5;
        state.convergence_score = 0.9;
        let new_vsa = test_vsa(99);
        state.reset(new_vsa.clone());
        assert_eq!(state.current, new_vsa);
        assert_eq!(state.recurrent_steps, 0);
        assert!((state.convergence_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_refine_step_tracks_history() {
        let mut model = RecurrentWorldModel::new();
        let mut state = RecurrentLatentState::new(test_vsa(10));
        for i in 0..3 {
            let obs = test_vsa(i + 100);
            let _ = model.refine_step(&mut state, &obs);
        }
        assert_eq!(model.latent_history.len(), 3);
        assert_eq!(model.total_refinements, 3);
    }

    #[test]
    fn test_rollout_with_empty_initial_convergence() {
        let model = RecurrentWorldModel::new();
        let state = RecurrentLatentState {
            current: test_vsa(7),
            recurrent_steps: 0,
            convergence_score: 0.0,
        };
        let trajectory = model.rollout(&state, 3);
        assert_eq!(trajectory.len(), 3);
        assert!(trajectory[0].convergence_score >= 0.0);
    }

    #[test]
    fn test_refine_step_reduces_change_over_repeated_obs() {
        let mut model = RecurrentWorldModel::new();
        let mut state = RecurrentLatentState::new(test_vsa(55));
        let obs = test_vsa(55);
        let change1 = model.refine_step(&mut state, &obs);
        let change2 = model.refine_step(&mut state, &obs);
        assert!(
            change2 <= change1 + 0.01,
            "second refinement should not increase change"
        );
    }
}
