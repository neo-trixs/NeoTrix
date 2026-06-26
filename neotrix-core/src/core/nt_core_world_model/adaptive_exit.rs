use super::latent_state::VsaLatentState;

/// Lightweight gate for adaptive computation depth.
///
/// Determines whether latent-state refinement should continue
/// or exit early based on convergence of successive states.
/// The gate emits a score in [0, 1] where values above the
/// threshold indicate that refinement has converged.
#[derive(Debug, Clone)]
pub struct AdaptiveExitGate {
    /// Learned threshold for exit probability (default: 0.95)
    convergence_threshold: f64,
}

impl AdaptiveExitGate {
    /// Create a new adaptive exit gate with default threshold.
    pub fn new() -> Self {
        Self {
            convergence_threshold: 0.95,
        }
    }

    /// Set the convergence threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.convergence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Current convergence threshold.
    pub fn threshold(&self) -> f64 {
        self.convergence_threshold
    }

    /// Compute the exit probability (gate score) for a state.
    ///
    /// Higher energy → lower exit probability (more refinement needed).
    /// Returns a value in [0, 1].
    pub fn gate_score(&self, state: &VsaLatentState) -> f64 {
        let energy_factor = 1.0 - state.energy.clamp(0.0, 1.0);
        let iter_factor = 1.0 - 1.0 / (state.iterations_used as f64 + 1.0);
        let base: f64 = energy_factor * 0.7 + iter_factor * 0.3;
        sigmoid(base)
    }

    /// Decide whether to exit based on the delta between successive states.
    ///
    /// Returns `true` when the delta is small (state has converged)
    /// **and** the gate score exceeds the threshold.
    pub fn should_exit(&self, state: &VsaLatentState, prev_state: &VsaLatentState) -> bool {
        let delta = state.delta(prev_state);
        let score = self.gate_score(state);
        delta < self.convergence_threshold * 0.05 && score >= self.convergence_threshold
    }
}

impl Default for AdaptiveExitGate {
    fn default() -> Self {
        Self::new()
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_score_range() {
        let gate = AdaptiveExitGate::new();
        let state = VsaLatentState::empty(4096);
        let score = gate.gate_score(&state);
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_low_delta_triggers_exit() {
        let gate = AdaptiveExitGate::new().with_threshold(0.5);
        let mut state = VsaLatentState::empty(4096);
        state.energy = 0.01;
        state.iterations_used = 10;
        let prev = VsaLatentState::empty(4096);
        assert!(gate.should_exit(&state, &prev));
    }

    #[test]
    fn test_high_energy_prevents_exit() {
        let gate = AdaptiveExitGate::new();
        let state = VsaLatentState::empty(4096);
        let prev = VsaLatentState::empty(4096);
        assert!(!gate.should_exit(&state, &prev));
    }
}
