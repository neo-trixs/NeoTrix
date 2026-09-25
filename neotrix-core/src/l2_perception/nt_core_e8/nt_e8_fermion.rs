//! Spin(11,3) 64-fermion decomposition: `FermionState` and generators.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use serde::Deserialize;

use super::TOTAL_SM_FERMIONS;

// ─── Spin(11,3) 64-Fermion Decomposition ────────────────────────────

/// A single fermion state in the Spin(11,3) 64-dimensional spinor.
#[derive(Debug, Clone, PartialEq, serde::Serialize, Deserialize)]
pub struct FermionState {
    /// Spinor weight coordinates (8-dim weight space).
    pub weight: [i8; 8],
    /// Quantum numbers: (electric charge Q, weak isospin I₃, strong color)
    pub q: f64,
    pub i3: f64,
    pub color: String,
    /// Particle/antiparticle.
    pub is_particle: bool,
}

impl FermionState {
    pub fn new(weight: [i8; 8], q: f64, i3: f64, color: &str, is_particle: bool) -> Self {
        Self {
            weight,
            q,
            i3,
            color: color.to_string(),
            is_particle,
        }
    }
}

/// Generate the 64 fermion states for one generation.
/// Each of the 64 hexagrams corresponds to one fermion.
pub fn fermion_states_for_generation(_gen: usize) -> Vec<FermionState> {
    let mut states = Vec::with_capacity(64);

    // Each hexagram (0-63) maps to a fermion via its 6 bits:
    //   bits[0..3] → SU(3) color weight (3 bits = 8 colors)
    //   bits[3..5] → SU(2) weak isospin (2 bits = 4 states)
    //   bit[5]     → U(1) hypercharge sign
    for hex in 0..64 {
        let color_bits = (hex >> 3) & 0x7;
        let weak_bits = (hex >> 1) & 0x3;
        let hyper_sign = hex & 0x1;

        let (color, r, g, b) = match color_bits {
            0 => ("red", 1, 0, 0),
            1 => ("green", 0, 1, 0),
            2 => ("blue", 0, 0, 1),
            3 => ("antired", -1, 0, 0),
            4 => ("antigreen", 0, -1, 0),
            5 => ("antiblue", 0, 0, -1),
            6 => ("white", 0, 0, 0),
            7 => ("black", 0, 0, 0),
            _ => ("unknown", 0, 0, 0),
        };

        let (i3, q) = match weak_bits {
            0 => (0.5, 2.0 / 3.0),   // up-type left
            1 => (-0.5, -1.0 / 3.0), // down-type left
            2 => (0.0, 2.0 / 3.0),   // up-type right
            3 => (0.0, -1.0 / 3.0),  // down-type right
            _ => (0.0, 0.0),
        };

        let q_adj = if hyper_sign == 1 { q } else { -q };

        let weight = [r, g, b, 0, 0, 0, 0, 0];

        states.push(FermionState::new(weight, q_adj, i3, color, hyper_sign == 0));
    }

    states
}

/// Total SM fermions across 3 generations: 3 × 64 = 192.
pub fn all_sm_fermions() -> Vec<FermionState> {
    let mut all = Vec::with_capacity(192);
    for gen in 0..3 {
        all.extend(fermion_states_for_generation(gen));
    }
    all
}

/// Verify: exactly 192 fermions across 3 generations.
pub fn verify_total_fermions() -> bool {
    all_sm_fermions().len() == TOTAL_SM_FERMIONS
}
