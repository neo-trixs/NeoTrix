use super::action_embedding::ActionEmbedding;
use super::adaptive_exit::AdaptiveExitGate;
use super::latent_state::VsaLatentState;
use super::spectral_constraint::SpectralConstraint;

/// Errors produced by the looped dynamics engine.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DynamicsError {
    /// No initial state has been set.  Call `reset` first.
    #[error("no initial state: call reset() before stepping")]
    NoInitialState,
    /// Action and latent dimensions are incompatible.
    #[error("dimension mismatch: action {action_dim}, latent {latent_dim}")]
    DimensionMismatch {
        action_dim: usize,
        latent_dim: usize,
    },
    /// Numerical instability detected during refinement.
    #[error("numerical instability at iteration {iteration}: energy {energy}")]
    NumericalInstability { iteration: usize, energy: f64 },
}

/// The LoopWM (Looped World Model) core engine.
///
/// Implements looped latent-state refinement over K iterations per timestep,
/// as described in "Looped World Models" (arXiv 2606.18208).
///
/// The transition operator is a parameter-shared recurrent block that refines
/// a VSA-encoded latent state `h_t` for `num_iterations` iterations before
/// producing the final `h_{t+1}`.  An optional adaptive-exit gate can
/// short-circuit refinement when the state has converged.
#[derive(Debug, Clone)]
pub struct LoopedDynamics {
    /// Number of refinement iterations per timestep (default: 4)
    num_iterations: usize,
    /// Whether to use adaptive exit gating
    adaptive_exit: bool,
    /// Spectral radius constraint for stability
    spectral_constraint: SpectralConstraint,
    /// Latent state dimension (VSA 4096-bit)
    #[allow(dead_code)]
    latent_dim: usize,
    /// Internal state buffer
    state: Option<VsaLatentState>,
    /// Adaptive exit gate (used when `adaptive_exit` is true)
    exit_gate: Option<AdaptiveExitGate>,
}

impl LoopedDynamics {
    /// Create a new dynamics engine with the given latent dimension.
    pub fn new(latent_dim: usize) -> Self {
        Self {
            num_iterations: 4,
            adaptive_exit: false,
            spectral_constraint: SpectralConstraint::new(),
            latent_dim,
            state: None,
            exit_gate: None,
        }
    }

    /// Set the number of refinement iterations per timestep.
    pub fn with_iterations(mut self, n: usize) -> Self {
        self.num_iterations = n.max(1);
        self
    }

    /// Enable or disable adaptive early-exit gating.
    pub fn with_adaptive_exit(mut self, enabled: bool) -> Self {
        self.adaptive_exit = enabled;
        if enabled && self.exit_gate.is_none() {
            self.exit_gate = Some(AdaptiveExitGate::new());
        }
        self
    }

    /// Set a custom spectral constraint.
    pub fn with_spectral_constraint(mut self, constraint: SpectralConstraint) -> Self {
        self.spectral_constraint = constraint;
        self
    }

    /// Set the initial latent state.
    pub fn reset(&mut self, state: VsaLatentState) {
        self.state = Some(state);
    }

    /// Access the current state, if any.
    pub fn current_state(&self) -> Option<&VsaLatentState> {
        self.state.as_ref()
    }

    /// Number of refinement iterations configured.
    pub fn iterations(&self) -> usize {
        self.num_iterations
    }

    /// Whether adaptive exit is enabled.
    pub fn adaptive_exit_enabled(&self) -> bool {
        self.adaptive_exit
    }

    /// Single timestep rollout with K refinement iterations.
    ///
    /// Starting from the current latent state `h_t`, applies `num_iterations`
    /// refinement passes to produce `h_{t+1}`.  If adaptive exit is enabled,
    /// refinement may stop early when the state converges.
    pub fn step(&mut self, action: &ActionEmbedding) -> Result<VsaLatentState, DynamicsError> {
        let current = self.state.as_ref().ok_or(DynamicsError::NoInitialState)?;

        let mut refined = current.clone();
        let max_iter = self.num_iterations;

        for i in 0..max_iter {
            let prev = refined.clone();
            refined = self.refine(&refined, action, i);

            // Check for numerical instability
            if refined.energy.is_nan() || refined.energy.is_infinite() || refined.energy > 1e6 {
                return Err(DynamicsError::NumericalInstability {
                    iteration: i,
                    energy: refined.energy,
                });
            }

            refined.iterations_used = i + 1;

            // Adaptive early-exit
            if self.adaptive_exit {
                if let Some(ref gate) = self.exit_gate {
                    if gate.should_exit(&refined, &prev) {
                        break;
                    }
                }
            }
        }

        self.state = Some(refined.clone());
        Ok(refined)
    }

    /// Multi-step rollout over a sequence of actions.
    ///
    /// Returns the trajectory of latent states (including the initial state).
    /// The first element is always the initial state before any action.
    pub fn rollout(
        &mut self,
        actions: &[ActionEmbedding],
    ) -> Result<Vec<VsaLatentState>, DynamicsError> {
        if self.state.is_none() {
            return Err(DynamicsError::NoInitialState);
        }

        let mut trajectory = Vec::with_capacity(actions.len() + 1);
        trajectory.push(self.state.clone().unwrap());

        for action in actions {
            let next = self.step(action)?;
            trajectory.push(next);
        }

        Ok(trajectory)
    }

    /// Single refinement iteration of the latent state.
    ///
    /// The transition operator combines the current state and action
    /// via a parameter-shared update.  Each iteration applies:
    ///
    /// 1. Action injection: bind action features into state
    /// 2. Spectral-normalised linear transform
    /// 3. Non-linear gating
    /// 4. Energy update based on state change
    pub fn refine(
        &self,
        state: &VsaLatentState,
        action: &ActionEmbedding,
        iter: usize,
    ) -> VsaLatentState {
        let n = state.vector.len();
        let mut new_vector = state.vector.clone();

        // Action injection: mix action parameters into latent vector
        let action_scale = 0.1 / (action.parameters.len().max(1) as f64);
        for (j, param) in action.parameters.iter().enumerate() {
            if j < n {
                let idx = (j + iter * 7) % n;
                new_vector[idx] = new_vector[idx]
                    .wrapping_add(((*param * action_scale * 255.0) as i16).clamp(0, 255) as u8);
            }
        }

        // Spectral-normalised linear transform
        let flat = self.vector_to_flat(&new_vector);
        let constrained = self.spectral_constraint.constrain(&flat, n);
        new_vector = self.flat_to_vector(&constrained, n);

        // Non-linear gating
        for x in new_vector.iter_mut() {
            let v = *x as f64 / 255.0;
            *x = (sigmoid(v - 0.5) * 255.0) as u8;
        }

        // Energy update
        let delta = state.delta(&VsaLatentState {
            vector: new_vector.clone(),
            energy: state.energy,
            iterations_used: iter,
        });
        let new_energy = state.energy * 0.9 + delta * 0.1;

        VsaLatentState {
            vector: new_vector,
            energy: new_energy,
            iterations_used: iter + 1,
        }
    }

    /// Flatten a byte vector to f64 slice for spectral constraint ops.
    fn vector_to_flat(&self, v: &[u8]) -> Vec<f64> {
        v.iter().map(|&b| b as f64 / 255.0).collect()
    }

    /// Convert a constrained f64 slice back to byte vector.
    fn flat_to_vector(&self, v: &[f64], n: usize) -> Vec<u8> {
        v.iter()
            .take(n)
            .map(|&x| (x.clamp(0.0, 1.0) * 255.0) as u8)
            .collect()
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action() -> ActionEmbedding {
        ActionEmbedding::new("move", vec![0.5, -0.3, 0.1])
    }

    #[test]
    fn test_step_requires_initial_state() {
        let mut dyns = LoopedDynamics::new(4096);
        let action = make_action();
        assert!(matches!(
            dyns.step(&action),
            Err(DynamicsError::NoInitialState)
        ));
    }

    #[test]
    fn test_single_step_produces_state() {
        let mut dyns = LoopedDynamics::new(4096);
        dyns.reset(VsaLatentState::empty(4096));
        let action = make_action();
        let result = dyns.step(&action).unwrap();
        assert_eq!(result.vector.len(), 4096);
        assert!(result.iterations_used <= 4);
    }

    #[test]
    fn test_rollout_produces_trajectory() {
        let mut dyns = LoopedDynamics::new(4096);
        dyns.reset(VsaLatentState::empty(4096));
        let actions = vec![make_action(), make_action(), make_action()];
        let trajectory = dyns.rollout(&actions).unwrap();
        assert_eq!(trajectory.len(), 4); // initial + 3 steps
    }

    #[test]
    fn test_refine_lowers_energy() {
        let dyns = LoopedDynamics::new(4096);
        let state = VsaLatentState::empty(4096);
        let action = make_action();
        let refined = dyns.refine(&state, &action, 0);
        assert!(refined.energy < state.energy);
    }

    #[test]
    fn test_empty_rollout_returns_initial() {
        let mut dyns = LoopedDynamics::new(4096);
        dyns.reset(VsaLatentState::empty(4096));
        let trajectory = dyns.rollout(&[]).unwrap();
        assert_eq!(trajectory.len(), 1);
    }

    #[test]
    fn test_builder_pattern() {
        let dyns = LoopedDynamics::new(4096)
            .with_iterations(6)
            .with_adaptive_exit(true)
            .with_spectral_constraint(SpectralConstraint::new().with_max_radius(0.95));
        assert_eq!(dyns.iterations(), 6);
        assert!(dyns.adaptive_exit_enabled());
    }
}
