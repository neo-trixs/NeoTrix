use super::VSA_DIM;
use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};
use serde::{Deserialize, Serialize};

/// VSA-based unified representation of FEP state and IIT cause-effect repertoire.
///
/// Both FEP (prediction error, entropy, gradient) and IIT (phi, resonance,
/// state energy) are encoded as hypervectors via VSA binding/bundling.
/// The unified hypervector = bind(FE_hv, IIT_hv) constitutes the shared
/// substrate on which both theories operate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VSAUnifiedState {
    /// Hypervector encoding of FEP components (prediction_error, entropy, gradient)
    pub fe_hypervector: Vec<f64>,
    /// Hypervector encoding of IIT cause-effect structure (phi, resonance, dims)
    pub iit_hypervector: Vec<f64>,
    /// Bound hypervector: bind(FE_hv, IIT_hv) — the unified representation
    pub unified_hv: Vec<f64>,
    /// VSA coherence: cosine similarity between FE and IIT subspaces
    pub vsa_coherence: f64,
}

/// Full bridge cycle report combining FEP, IIT, and VSA analyses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeReport {
    /// Combined consciousness score S = α·(1-FEₙ) + β·Φ + γ·VSA_coherence
    pub consciousness_score: f64,
    /// VSA coherence between FEP and IIT hypervector subspaces
    pub vsa_coherence: f64,
    /// Phi computed from FE-derived state (FEP → IIT mapping)
    pub fe_derived_phi: f64,
    /// Effective free energy after IIT-informed bounding
    pub bounded_free_energy: f64,
    /// Lower bound on free energy imposed by system's integrated information
    pub free_energy_bound: f64,
    /// How much IIT improves FE (reward direction: IIT → FEP)
    pub fe_improvement_from_iit: f64,
    /// How much FE improvement enhances Phi expression (reward direction: FEP → IIT)
    pub phi_improvement_from_fep: f64,
    /// Text classification of the system state
    pub state_classification: &'static str,
}

/// 4096-dimensional MAP-hypervector for pure VSA-based FEP-IIT operations.
///
/// Uses f32 for storage efficiency while interoperating with the
/// existing f64-based VSA engine and FEPIITBridge.
///
/// Provides:
/// - Deterministic construction from scalar seeds
/// - Zero-state (empty system)
/// - Unit normalization
/// - f64 projection for VSAEngine interop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FepIitHypervector {
    pub hv: Vec<f32>,
}

impl FepIitHypervector {
    /// Create a zero-initialized hypervector (empty/dead system).
    pub fn zeros() -> Self {
        Self {
            hv: vec![0.0; VSA_DIM],
        }
    }

    /// Create a hypervector from a scalar value and seed position.
    /// Deterministic: same (value, seed) → identical HV.
    pub fn from_scalar(value: f64, seed: f64) -> Self {
        let hv: Vec<f32> = (0..VSA_DIM)
            .map(|i| {
                let phase = (i as f64 * 0.1 + seed * 1.7).sin();
                let amplitude = (i as f64 * 0.07 + seed * 0.3).cos();
                (value * phase * amplitude) as f32
            })
            .collect();
        Self { hv }
    }

    /// Create a pseudo-random hypervector using seed-based determinism.
    pub fn random_from_seed(seed: u64) -> Self {
        let hv: Vec<f32> = (0..VSA_DIM)
            .map(|i| {
                let x = (i as f64 * 0.137 + seed as f64 * 1.907).sin()
                    * (i as f64 * 0.239 + seed as f64 * 0.431).cos();
                x as f32
            })
            .collect();
        Self { hv }
    }

    /// Project to f64 slice for use with VSAEngine.
    pub fn as_f64(&self) -> Vec<f64> {
        self.hv.iter().map(|&x| x as f64).collect()
    }

    /// Normalize to unit length in-place.
    pub fn normalize(&mut self) {
        let norm_sq: f64 = self.hv.iter().map(|&x| (x as f64) * (x as f64)).sum();
        let norm = norm_sq.sqrt();
        if norm > 1e-12 {
            for x in &mut self.hv {
                *x = (*x as f64 / norm) as f32;
            }
        }
    }

    /// Cosine similarity with another FepIitHypervector.
    pub fn similarity(&self, other: &Self) -> f64 {
        let a = self.as_f64();
        let b = other.as_f64();
        let engine = VSAEngine::new(VSA_DIM);
        engine.similarity(&a, &b)
    }

    pub fn dim() -> usize {
        VSA_DIM
    }
}

impl Default for FepIitHypervector {
    fn default() -> Self {
        Self::zeros()
    }
}
