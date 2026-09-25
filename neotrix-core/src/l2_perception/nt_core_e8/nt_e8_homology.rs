//! E8 identity verifiers and the `E8HexagramHomology` aggregate model.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use serde::Deserialize;

use super::{
    DAYAN_NUMBER, E8_DIM, E8_RANK, E8_ROOTS, FERMIONS_PER_GENERATION,
    FERMION_GENERATIONS, HEXAGRAM_COUNT, HE_TU_SUM, LINES_PER_HEXAGRAM, LO_SHU_CONSTANT,
    OBSERVABLE_DOF, OBSERVER_DOF, REMAINING_E8_GENERATORS, SEVEN_SQUARED, TOTAL_LINES,
    TOTAL_SM_FERMIONS, e8_root_norm_counts, verify_hadamard_orthogonality,
    verify_total_fermions,
};

// ─── Core Verifiers ──────────────────────────────────────────────────

/// Verify: E₈ = rank (Cartan) + non-zero roots = 8 + 240 = 248.
pub fn verify_e8_dimension() -> bool {
    E8_RANK + E8_ROOTS == E8_DIM
}

/// Verify: 3 generations × 64 fermions = 248 - 56.
pub fn verify_three_generations() -> bool {
    FERMION_GENERATIONS * FERMIONS_PER_GENERATION == E8_DIM - REMAINING_E8_GENERATORS
}

/// Verify: 64 × 6 = 384 total lines.
pub fn verify_total_lines() -> bool {
    HEXAGRAM_COUNT * LINES_PER_HEXAGRAM == TOTAL_LINES
}

/// Verify: Dayan = 50, observable = 49, observer = 1.
pub fn verify_dayan_identity() -> bool {
    DAYAN_NUMBER == OBSERVABLE_DOF + OBSERVER_DOF
}

/// Verify: 7² = 49.
pub fn verify_seven_squared() -> bool {
    7 * 7 == SEVEN_SQUARED
}

/// Verify: Lo Shu 3×3 sum = 15 (every row/col/diag).
pub fn verify_lo_shu() -> bool {
    // Standard Lo Shu: 4 9 2 / 3 5 7 / 8 1 6
    let square = [[4, 9, 2], [3, 5, 7], [8, 1, 6]];
    for i in 0..3 {
        let row_sum: usize = square[i].iter().sum();
        let col_sum: usize = square.iter().map(|r| r[i]).sum();
        if row_sum != LO_SHU_CONSTANT || col_sum != LO_SHU_CONSTANT {
            return false;
        }
    }
    let diag1: usize = (0..3).map(|i| square[i][i]).sum();
    let diag2: usize = (0..3).map(|i| square[i][2 - i]).sum();
    diag1 == LO_SHU_CONSTANT && diag2 == LO_SHU_CONSTANT
}

/// Verify: He Tu sum = 1+2+...+10 = 55.
pub fn verify_he_tu_sum() -> bool {
    (1..=10).sum::<usize>() == HE_TU_SUM
}

/// Run all identity verifications.
pub fn verify_all_identities() -> Vec<(&'static str, bool)> {
    vec![
        ("E8_dimension", verify_e8_dimension()),
        ("three_generations", verify_three_generations()),
        ("total_lines", verify_total_lines()),
        ("dayan_identity", verify_dayan_identity()),
        ("seven_squared", verify_seven_squared()),
        ("lo_shu", verify_lo_shu()),
        ("he_tu_sum", verify_he_tu_sum()),
    ]
}

// ─── E₈ × 64 Model ──────────────────────────────────────────────────

/// Complete E₈ × 64-hexagram model homology result.
#[derive(serde::Serialize, Deserialize)]
#[serde(default)]
pub struct E8HexagramHomology {
    /// E₈ dimension = 248.
    pub e8_dim: usize,
    /// Number of hexagrams = 64.
    pub hexagram_count: usize,
    /// Fermions per generation = 64.
    pub fermions_per_gen: usize,
    /// Number of generations = 3.
    pub generations: usize,
    /// Total SM fermions = 192.
    pub total_fermions: usize,
    /// E₈ - SM fermions = 56 remaining.
    pub remaining_generators: usize,
    /// All identities verified.
    pub all_identities_hold: bool,
    /// Per-identity results.
    #[serde(skip)]
    pub identity_results: Vec<(&'static str, bool)>,
}

impl Default for E8HexagramHomology {
    fn default() -> Self {
        Self::new()
    }
}

impl E8HexagramHomology {
    pub fn new() -> Self {
        let identity_results = verify_all_identities();
        let extra_results = vec![
            ("total_fermions", verify_total_fermions()),
            ("hadamard_orthogonality", verify_hadamard_orthogonality()),
            ("e8_root_norm", {
                let (norm2, others) = e8_root_norm_counts();
                norm2 == 240 && others == 0
            }),
        ];
        let all_identities = identity_results.iter().chain(extra_results.iter());
        let all_hold = all_identities.clone().all(|(_, ok)| *ok);

        Self {
            e8_dim: E8_DIM,
            hexagram_count: HEXAGRAM_COUNT,
            fermions_per_gen: FERMIONS_PER_GENERATION,
            generations: FERMION_GENERATIONS,
            total_fermions: TOTAL_SM_FERMIONS,
            remaining_generators: REMAINING_E8_GENERATORS,
            all_identities_hold: all_hold,
            identity_results: {
                let mut r: Vec<(&str, bool)> = identity_results.clone();
                r.extend(extra_results);
                r
            },
        }
    }
}
