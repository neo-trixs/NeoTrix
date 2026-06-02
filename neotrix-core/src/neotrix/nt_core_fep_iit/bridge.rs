use serde::{Deserialize, Serialize};
use crate::core::nt_core_hcube::vsa::{VsaBackend, VSAEngine};
use crate::neotrix::nt_world_infer::FreeEnergyReport;
use crate::neotrix::nt_core_iit_phi::{IITPhiCalculator, PhiReport};
use super::types::{BridgeReport, FepIitHypervector, VSAUnifiedState};
use super::{DEFAULT_ALPHA, DEFAULT_BETA, DEFAULT_GAMMA, FE_NORMALIZE_MAX, VSA_DIM};

/// FEP-IIT bridge state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FEPIITBridge {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    vsa_dim: usize,
    phi_sigma: f64,
}

impl Default for FEPIITBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl FEPIITBridge {
    pub fn new() -> Self {
        Self {
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
            gamma: DEFAULT_GAMMA,
            vsa_dim: VSA_DIM,
            phi_sigma: 0.15,
        }
    }

    pub(crate) fn vsa(&self) -> VSAEngine {
        VSAEngine::new(self.vsa_dim)
    }

    fn phi_calc(&self) -> IITPhiCalculator {
        IITPhiCalculator::with_sigma(IITPhiCalculator::new(), self.phi_sigma)
    }

    // ============================================================
    // VSA Unified Representation
    // ============================================================

    /// Encode a FreeEnergyReport into a VSA hypervector.
    ///
    /// Each FE component (prediction energy, entropy, gradient, epistemic value,
    /// trend) is bound to a distinct seed position in hyperdimensional space,
    /// then bundled into a single hypervector that preserves the structure.
    fn encode_fe_to_vsa(&self, report: &FreeEnergyReport) -> Vec<f64> {
        let pred_hv = self.vsa_from_scalar(report.prediction_energy, 0.0);
        let entropy_hv = self.vsa_from_scalar(report.entropy_contribution, 1.0);
        let gradient_hv = self.vsa_from_scalar(report.gradient_penalty, 2.0);
        let epistemic_hv = self.vsa_from_scalar(report.epistemic_value, 3.0);
        let trend_hv = self.vsa_from_scalar(report.fe_trend, 4.0);

        self.vsa().bundle(&[&pred_hv, &entropy_hv, &gradient_hv, &epistemic_hv, &trend_hv])
    }

    /// Encode a PhiReport into a VSA hypervector (IIT cause-effect repertoire).
    fn encode_phi_to_vsa(&self, report: &PhiReport) -> Vec<f64> {
        let phi_hv = self.vsa_from_scalar(report.phi, 10.0);
        let resonance_hv = self.vsa_from_scalar(report.total_resonance, 11.0);
        let energy_hv = self.vsa_from_scalar(report.state_energy, 12.0);
        let dims_hv = self.vsa_from_scalar(report.effective_dims as f64 / 64.0, 13.0);
        let trend_hv = self.vsa_from_scalar(report.phi_trend, 14.0);

        self.vsa().bundle(&[&phi_hv, &resonance_hv, &energy_hv, &dims_hv, &trend_hv])
    }

    /// Create a deterministic VSA hypervector from a scalar value using a
    /// seed position to ensure orthogonality across different quantities.
    pub(crate) fn vsa_from_scalar(&self, value: f64, seed: f64) -> Vec<f64> {
        let dim = self.vsa().dimensions();
        (0..dim)
            .map(|i| {
                let phase = (i as f64 * 0.1 + seed * 1.7).sin();
                let amplitude = (i as f64 * 0.07 + seed * 0.3).cos();
                value * phase * amplitude
            })
            .collect()
    }

    /// Build unified VSA representation from FEP and IIT reports.
    /// The unified HV = bind(FE_hv, IIT_hv) via elementwise multiplication
    /// (MAP-VSA binding), capturing the interaction between predictive
    /// accuracy and integrated information.
    pub fn build_unified_state(
        &self,
        fe_report: &FreeEnergyReport,
        phi_report: &PhiReport,
    ) -> VSAUnifiedState {
        let fe_hv = self.encode_fe_to_vsa(fe_report);
        let iit_hv = self.encode_phi_to_vsa(phi_report);
        let unified = self.vsa().bind(&fe_hv, &iit_hv);
        let coherence = self.vsa().similarity(&fe_hv, &iit_hv);

        VSAUnifiedState {
            fe_hypervector: fe_hv,
            iit_hypervector: iit_hv,
            unified_hv: unified,
            vsa_coherence: coherence,
        }
    }

    // ============================================================
    // FEP → IIT Mapping
    // ============================================================

    /// Map an FEP state to an IIT cause-effect repertoire.
    ///
    /// The FE hypervector is downsampled to 64 dimensions and fed to
    /// IIT's Phi computation. This expresses the level of integrated
    /// information implicit in the current predictive state: a well-
    /// calibrated generative model (low FE) should exhibit coherent
    /// integration across its dimensions.
    pub fn fep_to_iit(&self, fe_report: &FreeEnergyReport) -> PhiReport {
        let fe_hv = self.encode_fe_to_vsa(fe_report);
        let state_64 = self.hv_to_64(&fe_hv);
        self.phi_calc().compute_phi(&state_64)
    }

    // ============================================================
    // IIT → FEP Mapping
    // ============================================================

    /// Use IIT's integrated information to bound effective free energy.
    ///
    /// Higher Phi → more integrated system → lower effective free energy:
    ///   FE_effective = FE × (1 - β·Φ)
    ///
    /// A highly integrated system requires less prediction error because
    /// its internal coherence compensates for uncertainty.
    pub fn iit_bounded_free_energy(&self, free_energy: f64, phi: f64) -> f64 {
        let phi_scale = 1.0 - self.beta * phi.clamp(0.0, 1.0);
        (free_energy * phi_scale).max(0.0)
    }

    /// IIT-informed theoretical lower bound on free energy.
    ///
    /// A system with higher integrated information can achieve a lower
    /// free energy because its internal structure provides additional
    /// constraints that reduce prediction error.
    pub fn free_energy_bound(&self, phi: f64) -> f64 {
        FE_NORMALIZE_MAX * (1.0 - phi.clamp(0.0, 1.0) * 0.5)
    }

    // ============================================================
    // Bidirectional Reward
    // ============================================================

    /// Normalize free energy to [0,1] where 0 = worst, 1 = best
    pub fn normalize_fe(&self, free_energy: f64) -> f64 {
        (1.0 - (free_energy / FE_NORMALIZE_MAX).clamp(0.0, 1.0)).max(0.0)
    }

    /// Combined consciousness score:
    ///   S = α·(1 - FEₙ) + β·Φ + γ·VSA_coherence
    ///
    /// Where:
    /// - FEₙ = normalized free energy (0 = worst, 1 = best)
    /// - Φ = normalized integrated information [0,1]
    /// - VSA_coherence = cosine similarity between FE and IIT subspaces
    pub fn compute_consciousness_score(
        &self,
        free_energy: f64,
        phi: f64,
        vsa_coherence: f64,
    ) -> f64 {
        let fe_term = self.alpha * self.normalize_fe(free_energy);
        let phi_term = self.beta * phi.clamp(0.0, 1.0);
        let coherence_term = self.gamma * vsa_coherence.clamp(-1.0, 1.0).max(0.0);
        (fe_term + phi_term + coherence_term).clamp(0.0, 1.0)
    }

    /// Compute bidirectional reward: how much does improving one side help the other?
    ///
    /// Returns `(fe_improvement_from_iit, phi_improvement_from_fep)`:
    /// - fe_improvement: reduction in free energy attributable to IIT integration
    /// - phi_improvement: enhancement of Phi expression from low FE
    pub fn bidirectional_reward(
        &self,
        fe_report: &FreeEnergyReport,
        phi_report: &PhiReport,
    ) -> (f64, f64) {
        let fe_reduction = fe_report.variational_fe
            - self.iit_bounded_free_energy(fe_report.variational_fe, phi_report.phi);

        let fe_norm = self.normalize_fe(fe_report.variational_fe);
        let phi_improvement = phi_report.phi * fe_norm;

        (fe_reduction.max(0.0), phi_improvement)
    }

    /// Full bridge cycle: FEP ↔ IIT ↔ VSA unified score.
    ///
    /// 1. Build unified VSA representation from both reports
    /// 2. FEP → IIT: compute Phi from FE-derived state
    /// 3. IIT → FEP: compute bounded free energy from Phi
    /// 4. Bidirectional reward calculation
    /// 5. Combined consciousness score
    pub fn bridge_cycle(
        &self,
        fe_report: &FreeEnergyReport,
        phi_report: &PhiReport,
    ) -> BridgeReport {
        let unified = self.build_unified_state(fe_report, phi_report);
        let fe_derived_phi = self.fep_to_iit(fe_report);
        let bounded_fe = self.iit_bounded_free_energy(
            fe_report.variational_fe,
            phi_report.phi,
        );
        let fe_bound = self.free_energy_bound(phi_report.phi);
        let (fe_improvement, phi_improvement) =
            self.bidirectional_reward(fe_report, phi_report);
        let score = self.compute_consciousness_score(
            fe_report.variational_fe,
            phi_report.phi,
            unified.vsa_coherence,
        );

        BridgeReport {
            consciousness_score: score,
            vsa_coherence: unified.vsa_coherence,
            fe_derived_phi: fe_derived_phi.phi,
            bounded_free_energy: bounded_fe,
            free_energy_bound: fe_bound,
            fe_improvement_from_iit: fe_improvement,
            phi_improvement_from_fep: phi_improvement,
            state_classification: self.classify_state(
                fe_report.variational_fe,
                phi_report.phi,
            ),
        }
    }

    /// Compute scores for multiple (FE, Φ) pairs
    pub fn compute_mixed_phi(&self, free_energies: &[f64], phis: &[f64]) -> Vec<f64> {
        let n = free_energies.len().min(phis.len());
        (0..n)
            .map(|i| self.compute_score(free_energies[i], phis[i]))
            .collect()
    }

    /// Classify the system state based on FE and Φ
    pub fn classify_state(&self, free_energy: f64, phi: f64) -> &'static str {
        let fe_norm = self.normalize_fe(free_energy);
        match (fe_norm > 0.6, phi > 0.2) {
            (true, true) => "optimal",
            (true, false) => "fragmented",
            (false, true) => "confused",
            (false, false) => "degraded",
        }
    }

    /// Legacy combined evolution score: α·(1 - FEₙ) + β·Φ
    pub fn compute_score(&self, free_energy: f64, phi: f64) -> f64 {
        let fe_term = self.alpha * self.normalize_fe(free_energy);
        let phi_term = self.beta * phi.clamp(0.0, 1.0);
        (fe_term + phi_term).clamp(0.0, 1.0)
    }

    /// Score improvement from a change in FE and Φ
    pub fn evolution_gain(fe_change: f64, phi_change: f64) -> f64 {
        let fe_improvement = -fe_change.clamp(-1.0, 1.0);
        let phi_improvement = phi_change.clamp(0.0, 0.5);
        0.5 * fe_improvement + 0.5 * phi_improvement
    }

    /// Map a 0-100 project evolution score to FEP-IIT score
    pub fn project_to_fep_iit(
        &self,
        project_score: f64,
        free_energy: f64,
        phi: f64,
    ) -> f64 {
        let fep_iit = self.compute_score(free_energy, phi);
        0.7 * (project_score / 100.0) + 0.3 * fep_iit
    }

    pub fn with_weights(mut self, alpha: f64, beta: f64) -> Self {
        self.alpha = alpha;
        self.beta = beta;
        self
    }

    pub fn with_vsa_weights(mut self, alpha: f64, beta: f64, gamma: f64) -> Self {
        self.alpha = alpha;
        self.beta = beta;
        self.gamma = gamma;
        self
    }

    pub fn with_sigma(mut self, sigma: f64) -> Self {
        self.phi_sigma = sigma;
        self
    }

    /// Downsample a 4096-dim VSA hypervector to 64-dim for IIT computation
    pub(crate) fn hv_to_64(&self, hv: &[f64]) -> Vec<f64> {
        let dim = hv.len();
        let step = dim / 64;
        (0..64)
            .map(|i| {
                let start = i * step;
                let end = if i == 63 { dim } else { (i + 1) * step };
                hv[start..end].iter().sum::<f64>() / (end - start) as f64
            })
            .collect()
    }

    // ============================================================
    // Pure VSA-based FEP-IIT Operations (Kearney 2026)
    // ============================================================

    /// Compute variational free energy from belief and observation hypervectors.
    ///
    /// Free energy = 1 - cosine_similarity(belief_hv, observation_hv)
    /// This represents prediction error in hyperdimensional space:
    /// a well-predicting belief → high similarity → low free energy.
    pub fn compute_free_energy(
        &self,
        belief: &FepIitHypervector,
        observation: &FepIitHypervector,
    ) -> f64 {
        let bel_f64 = belief.as_f64();
        let obs_f64 = observation.as_f64();
        let sim = self.vsa().similarity(&bel_f64, &obs_f64);
        let fe = 1.0 - sim.max(-1.0).min(1.0);
        fe.max(0.0).min(FE_NORMALIZE_MAX)
    }

    /// Compute Φ (integrated information) for a set of system state hypervectors.
    ///
    /// Full system coherence vs partitioned coherence. Higher Φ means the
    /// system is more integrated (can't be split into independent parts).
    /// Single-element systems → Φ = 0 (no integration possible).
    pub fn compute_phi(&self, states: &[FepIitHypervector]) -> f64 {
        let n = states.len();
        if n < 2 {
            return 0.0;
        }
        let full_coherence = self.avg_pairwise_similarity(states);
        let mid = n / 2;
        let left: Vec<FepIitHypervector> = states[..mid].to_vec();
        let right: Vec<FepIitHypervector> = states[mid..].to_vec();
        let left_coherence = self.avg_pairwise_similarity(&left);
        let right_coherence = self.avg_pairwise_similarity(&right);

        let w_left = left.len() as f64 / n as f64;
        let w_right = right.len() as f64 / n as f64;
        let partition_coherence = w_left * left_coherence + w_right * right_coherence;
        let phi = full_coherence - partition_coherence;
        phi.max(0.0).min(1.0)
    }

    fn avg_pairwise_similarity(&self, states: &[FepIitHypervector]) -> f64 {
        let n = states.len();
        if n < 2 {
            return if n == 1 { 1.0 } else { 0.0 };
        }
        let mut total = 0.0;
        let mut count = 0;
        for i in 0..n {
            let a = states[i].as_f64();
            for j in (i + 1)..n {
                let b = states[j].as_f64();
                total += self.vsa().similarity(&a, &b);
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { total / count as f64 }
    }

    /// Unified reward signal for the evolution loop:
    ///   R = FE * (1 - Φ) + α * Φ
    ///
    /// Where FE is normalized [0,1] (1=best), Φ is integrated info [0,1],
    /// and α is the bridge's alpha weight.
    /// When FE is low or Φ is high, the reward is dominated by the
    /// integration term; when FE is high, the penalty dominates.
    pub fn unified_reward(&self, free_energy: f64, phi: f64) -> f64 {
        let fe_norm = self.normalize_fe(free_energy);
        let phi_clamped = phi.clamp(0.0, 1.0);
        fe_norm * (1.0 - phi_clamped) + self.alpha * phi_clamped
    }

    /// Select the action index that minimizes variational free energy.
    ///
    /// Each action is encoded as a belief hypervector; the action whose
    /// predicted belief best matches the observation (i.e., lowest FE)
    /// is selected. Returns None for an empty action set.
    pub fn action_selection(
        &self,
        action_beliefs: &[FepIitHypervector],
        observation: &FepIitHypervector,
    ) -> Option<usize> {
        if action_beliefs.is_empty() {
            return None;
        }
        action_beliefs
            .iter()
            .enumerate()
            .map(|(i, belief)| {
                let fe = self.compute_free_energy(belief, observation);
                (i, fe)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let _instance = FEPIITBridge::new();
        assert!(true);
    }
}
