#![forbid(unsafe_code)]

/// FEP ↔ IIT Bridge
///
/// Connects Free Energy Principle (FEP) precision estimation with
/// Integrated Information Theory (IIT) phi values to produce
/// a unified consciousness score.
///
/// The bridge implements the theoretical mapping:
/// - FEP minimizes prediction error (free energy)
/// - IIT measures integrated information (phi)
/// - Consciousness = f(precision, phi, coherence)
///
/// Reference: "Consciousness as a Global State Variable" (Clark 2023)

/// Bridge between FEP and IIT subsystems.
///
/// Maintains calibration state for mapping between FEP precision
/// estimates and IIT phi values. The mapping is:
/// `score = precision * phi * (1.0 - exp(-coherence * alpha))`
#[derive(Debug, Clone)]
pub struct FepIitBridge {
    /// Calibration weight for phi-to-precision mapping
    alpha: f64,
    /// Minimum phi threshold below which consciousness score is zero
    phi_threshold: f64,
    /// Exponential decay factor for coherence contribution
    coherence_decay: f64,
}

impl Default for FepIitBridge {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            phi_threshold: 0.01,
            coherence_decay: 2.0,
        }
    }
}

impl FepIitBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute unified consciousness score from FEP and IIT signals.
    ///
    /// # Arguments
    /// * `free_energy` - FEP free energy (lower = better prediction)
    /// * `phi` - IIT integrated information (higher = more integrated)
    /// * `coherence` - System coherence [0.0, 1.0]
    ///
    /// # Returns
    /// Score in [0.0, 1.0] where 1.0 = maximum consciousness
    pub fn compute_consciousness_score(
        &self,
        free_energy: f64,
        phi: f64,
        coherence: f64,
    ) -> f64 {
        if phi < self.phi_threshold {
            return 0.0;
        }

        // Precision inversely related to free energy
        let precision = 1.0 / (1.0 + free_energy.abs());

        // Coherence contribution with exponential saturation
        let coherence_factor =
            1.0 - (-self.coherence_decay * coherence.max(0.0).min(1.0)).exp();

        // Unified score: precision × phi × coherence_factor
        (precision * phi * coherence_factor).max(0.0).min(1.0)
    }

    /// Compute IIT-bounded free energy.
    ///
    /// When phi is high (high integration), free energy is bounded
    /// more tightly — the system cannot easily escape its current state.
    pub fn iit_bounded_free_energy(&self, free_energy: f64, phi: f64) -> f64 {
        let bound = if phi > self.phi_threshold {
            // Higher phi → tighter bound on free energy change
            free_energy * (1.0 - (-self.alpha * phi).exp())
        } else {
            free_energy
        };
        bound.abs()
    }

    /// Select domains via Expected Free Energy (EFE) criterion.
    ///
    /// Given a set of domain scores (lower free energy = preferred),
    /// returns indices of domains ranked by EFE (most preferred first).
    pub fn efe_select_domain(
        &self,
        domain_free_energies: &[(String, f64)],
        epistemic_scale: f64,
    ) -> Vec<(String, f64)> {
        let mut scored: Vec<(String, f64)> = domain_free_energies
            .iter()
            .map(|(name, fe)| {
                // EFE = pragmatic value (free energy) + epistemic value (information gain proxy)
                let efe = fe.abs() + epistemic_scale * (1.0 / (1.0 + fe.abs()));
                (name.clone(), efe)
            })
            .collect();

        // Sort by EFE ascending (lower = more preferred)
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_score_zero_phi() {
        let bridge = FepIitBridge::new();
        let score = bridge.compute_consciousness_score(0.5, 0.0, 0.8);
        assert_eq!(score, 0.0, "phi below threshold should yield zero score");
    }

    #[test]
    fn test_consciousness_score_high_phi_low_fe() {
        let bridge = FepIitBridge::new();
        let score = bridge.compute_consciousness_score(0.0, 1.0, 1.0);
        assert!(score > 0.8, "high phi + low FE + high coherence should yield high score, got {}", score);
    }

    #[test]
    fn test_consciousness_score_bounded() {
        let bridge = FepIitBridge::new();
        let score = bridge.compute_consciousness_score(-100.0, 100.0, 1.0);
        assert!(score <= 1.0, "score must be bounded to [0,1], got {}", score);
    }

    #[test]
    fn test_iit_bounded_free_energy_reduces_with_phi() {
        let bridge = FepIitBridge::new();
        let fe_no_phi = bridge.iit_bounded_free_energy(10.0, 0.0);
        let fe_high_phi = bridge.iit_bounded_free_energy(10.0, 5.0);
        assert!(fe_high_phi < fe_no_phi, "high phi should tighten free energy bound");
    }

    #[test]
    fn test_efe_select_domain_ranks_correctly() {
        let bridge = FepIitBridge::new();
        let domains = vec![
            ("hard".to_string(), 5.0),
            ("easy".to_string(), 0.1),
            ("medium".to_string(), 2.0),
        ];
        let ranked = bridge.efe_select_domain(&domains, 1.0);
        assert_eq!(ranked[0].0, "easy", "lowest FE domain should be first");
        assert_eq!(ranked[1].0, "medium");
        assert_eq!(ranked[2].0, "hard");
    }
}
