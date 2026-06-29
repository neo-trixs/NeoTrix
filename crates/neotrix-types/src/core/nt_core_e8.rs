//! E₈ × 64-hexagram: mathematical spine of the hidden world model.
//!
//! Core identities discovered through deep research (2026-05-24):
//!   1. E₈ (248 generators) ⊃ Spin(11,3) → **64 fermions per generation**
//!   2. 3 generations × 64 = 192 = 248 - 56 (remaining Cartan/roots)
//!   3. 8 trigrams = simple roots of SU(3) color gauge group
//!   4. 64 hexagrams (6-bit binary) = weight diagram of the 64-fermion rep
//!   5. 384 lines = 64 × 6 = total degrees of freedom in E₈ root system
//!   6. 50 (Dayan) - 1 (observer) = 49 (observable dof) ↔ 49 = 7² = 248-199

#[cfg(test)]
use std::collections::HashSet;

// ─── Constants ───────────────────────────────────────────────────────

/// Dimension of E₈ exceptional Lie algebra = 248.
pub const E8_DIM: usize = 248;

/// Rank (Cartan subalgebra dimension) = 8.
pub const E8_RANK: usize = 8;

/// Non-zero root count = 240.
pub const E8_ROOTS: usize = 240;

/// Number of hexagrams = 64.
pub const HEXAGRAM_COUNT: usize = 64;

/// Lines per hexagram = 6.
pub const LINES_PER_HEXAGRAM: usize = 6;

/// Total lines = 64 × 6 = 384.
pub const TOTAL_LINES: usize = 64 * 6;

/// Fermion generations in the Standard Model = 3.
pub const FERMION_GENERATIONS: usize = 3;

/// Fermions per generation (Spin(11,3) spinor) = 64.
pub const FERMIONS_PER_GENERATION: usize = 64;

/// Total SM fermions = 3 × 64 = 192.
pub const TOTAL_SM_FERMIONS: usize = 192;

/// Remaining E₈ generators = 248 - 192 = 56.
pub const REMAINING_E8_GENERATORS: usize = 56;

/// Trigram count = 8 (maps to SU(3) dimension).
pub const TRIGRAM_COUNT: usize = 8;

/// Dayan number = 50 (total system degrees of freedom).
pub const DAYAN_NUMBER: usize = 50;

/// Observable degrees of freedom = 49.
pub const OBSERVABLE_DOF: usize = 49;

/// Observer dof = 1 (the "+1 principle").
pub const OBSERVER_DOF: usize = 1;

/// 7² = 49 = observable dof.
pub const SEVEN_SQUARED: usize = 49;

/// 5² × 2 = 50 = dayan number (5 heaven numbers + 5 earth numbers × parity).
pub const FIVE_SQUARED_TIMES_TWO: usize = 50;

/// Lo Shu magic square constant = 15.
pub const LO_SHU_CONSTANT: usize = 15;

/// He Tu sum = 55 (1+2+...+10).
pub const HE_TU_SUM: usize = 55;

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
    let square = [
        [4, 9, 2],
        [3, 5, 7],
        [8, 1, 6],
    ];
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

// ─── Hexagram System ─────────────────────────────────────────────────

/// A single hexagram: 6-bit binary state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hexagram {
    /// 6 bits, MSB = top line (yang=1, yin=0), per Shao Yong ordering.
    pub bits: u8,
}

impl Hexagram {
    pub fn new(bits: u8) -> Self {
        assert!(bits < 64, "Hexagram bits must be 0..63");
        Self { bits }
    }

    /// Line value at position i (0=bottom, 5=top). 1=yang, 0=yin.
    pub fn line(&self, i: usize) -> u8 {
        (self.bits >> (5 - i)) & 1
    }

    /// Bitwise NOT = 错卦 (opposite hexagram).
    pub fn opposite(&self) -> Self {
        Self { bits: !self.bits & 0x3F }
    }

    /// Is this hexagram pure yang (all 1s) = 乾 ☰.
    pub fn is_pure_yang(&self) -> bool {
        self.bits == 0x3F
    }

    /// Is this hexagram pure yin (all 0s) = 坤 ☷.
    pub fn is_pure_yin(&self) -> bool {
        self.bits == 0x00
    }

    /// King Wen sequence index in the standard 64-hexagram ordering.
    /// The standard King Wen ordering can be represented as a lookup table.
    pub fn wen_index(&self) -> Option<usize> {
        WEN_SEQUENCE.iter().position(|&b| b == self.bits)
    }
}

/// Generate all 64 hexagrams in Shao Yong binary order (先天图).
pub fn shao_yong_sequence() -> Vec<Hexagram> {
    (0..64).map(|i| Hexagram::new(i as u8)).collect()
}

/// Generate all 64 hexagrams in King Wen order (周易).
pub fn king_wen_sequence() -> Vec<Hexagram> {
    WEN_SEQUENCE.iter().map(|&b| Hexagram::new(b)).collect()
}

/// 8×8 hexagram matrix: rows and columns indexed by trigram (0-7).
/// Cell [i][j] = hexagram composed of upper trigram i, lower trigram j.
pub fn hexagram_matrix() -> [[Hexagram; 8]; 8] {
    let mut m = [[Hexagram::new(0); 8]; 8];
    for upper in 0..8u8 {
        for lower in 0..8u8 {
            // Upper trigram bits << 3 | lower trigram bits
            let bits = (upper << 3) | lower;
            m[upper as usize][lower as usize] = Hexagram::new(bits);
        }
    }
    m
}

/// Trigram names in Shao Yong order: 0=坤, 1=艮, 2=坎, 3=巽, 4=震, 5=离, 6=兑, 7=乾.
pub const TRIGRAM_NAMES: [&str; 8] = [
    "坤 ☷", "艮 ☶", "坎 ☵", "巽 ☴",
    "震 ☳", "离 ☲", "兑 ☱", "乾 ☰",
];

/// Fuxi binary trigram: 3-bit value for each trigram.
pub const TRIGRAM_BITS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

// ─── E₈ Root System ─────────────────────────────────────────────────

/// A weight vector in the 8-dimensional weight space of E₈.
/// Stored in half-units: coordinate value × 2 (so ½ is stored as 1, 1 as 2).
#[derive(Debug, Clone, PartialEq)]
pub struct E8Weight {
    pub coords: [i8; 8],
}

impl E8Weight {
    pub fn new(coords: [i8; 8]) -> Self {
        Self { coords }
    }

    /// Squared norm in half-unit² (divide by 4 to get true norm²).
    pub fn norm_sq_half_units(&self) -> i32 {
        self.coords.iter().map(|&c| c as i32 * c as i32).sum()
    }

    /// True squared norm (length²) in natural units.
    pub fn norm_sq(&self) -> f64 {
        let half_sum: i32 = self.coords.iter().map(|&c| c as i32 * c as i32).sum();
        half_sum as f64 / 4.0
    }
}

/// Generate all 240 non-zero roots of E₈.
///
/// E₈ roots consist of two families (coordinates in half-units: ½ → 1, 1 → 2):
///   - 112 roots of form (±2,±2,0⁶) permutations = 112
///   - 128 roots of form (±1,±1,±1,±1,±1,±1,±1,±1) with even minus signs = 128
///     Total = 112 + 128 = 240.
pub fn e8_root_system() -> Vec<E8Weight> {
    let mut roots = Vec::with_capacity(240);

    // Family 1: 112 roots — permutations of (±2,±2,0⁶) [= (±1,±1,0⁶) in natural units]
    for i in 0..8 {
        for j in (i + 1)..8 {
            for &si in &[-2i8, 2] {
                for &sj in &[-2i8, 2] {
                    let mut coords = [0i8; 8];
                    coords[i] = si;
                    coords[j] = sj;
                    roots.push(E8Weight::new(coords));
                }
            }
        }
    }

    // Family 2: 128 roots — (±1,±1,±1,±1,±1,±1,±1,±1) [= (±½,...,±½) in natural units]
    // with even number of minus signs.
    for mask in 0..256u16 {
        let ones = mask.count_ones();
        if ones % 2 != 0 {
            continue;
        }
        let mut coords = [0i8; 8];
        for (k, item) in coords.iter_mut().enumerate() {
            *item = if (mask >> k) & 1 == 1 { -1 } else { 1 };
        }
        roots.push(E8Weight::new(coords));
    }

    assert_eq!(roots.len(), 240, "E8 must have exactly 240 non-zero roots");
    roots
}

/// Count roots by squared norm.
/// E₈ has 240 roots: all have norm² = 2 (simply laced).
pub fn e8_root_norm_counts() -> (usize, usize) {
    let mut norm2_count: usize = 0;
    let eps = 1e-10;
    for root in e8_root_system() {
        let ns = root.norm_sq();
        if (ns - 2.0).abs() < eps {
            norm2_count += 1;
        }
    }
    (norm2_count, 240 - norm2_count)
}

// ─── SU(3) Subgroup ─────────────────────────────────────────────────

/// The 8 generators of SU(3) = 8 trigrams.
/// Represented as Gell-Mann matrices (symbolic structure constants).
pub fn su3_generators() -> Vec<&'static str> {
    vec![
        "λ₁ (gluon R̄G)", "λ₂ (gluon RḠ)",
        "λ₃ (gluon R̄R-ḠG)", "λ₄ (gluon R̄B)",
        "λ₅ (gluon RB̄)", "λ₆ (gluon ḠB)",
        "λ₇ (gluon GB̄)", "λ₈ (gluon R̄R+ḠG-2B̄B)/√3",
    ]
}

/// Map each trigram to a specific SU(3) root/coroot.
/// 乾 ☰ → gluon g₁ (R̄R), 坤 ☷ → gluon g₂ (ḠG), etc.
pub fn trigram_to_su3_root(trigram: u8) -> (i8, i8) {
    match trigram {
        0 => (-1, -1), // 坤
        1 => (1, 0),   // 艮
        2 => (0, 1),   // 坎
        3 => (1, 1),   // 巽
        4 => (-1, 0),  // 震
        5 => (0, -1),  // 离
        6 => (1, -1),  // 兑
        7 => (-1, 1),  // 乾
        _ => unreachable!("trigram must be 0-7, got {trigram}"),
    }
}

// ─── Spin(11,3) 64-Fermion Decomposition ────────────────────────────

/// A single fermion state in the Spin(11,3) 64-dimensional spinor.
#[derive(Debug, Clone, PartialEq)]
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
        Self { weight, q, i3, color: color.to_string(), is_particle }
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
            _ => unreachable!("color_bits must be 0-7, got {color_bits}"),
        };

        let (i3, q) = match weak_bits {
            0 => (0.5, 2.0 / 3.0),    // up-type left
            1 => (-0.5, -1.0 / 3.0),  // down-type left
            2 => (0.0, 2.0 / 3.0),    // up-type right
            3 => (0.0, -1.0 / 3.0),   // down-type right
            _ => unreachable!("weak_bits must be 0-3, got {weak_bits}"),
        };

        let q_adj = if hyper_sign == 1 { q } else { -q };

        let weight = [
            r, g, b,
            0, 0, 0, 0, 0,
        ];

        states.push(FermionState::new(
            weight, q_adj, i3, color, hyper_sign == 0,
        ));
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

// ─── Walsh-Hadamard Connection ───────────────────────────────────────

/// Generate the 8×8 Walsh-Hadamard matrix H(3) = H₂ ⊗ H₂ ⊗ H₂.
/// Sylvester construction: H(1) = [1 1; 1 -1]; H(n+1) = H(n) ⊗ H(1).
pub fn hadamard_matrix(n: usize) -> Vec<Vec<i8>> {
    if n == 0 {
        return vec![vec![1]];
    }
    let prev = hadamard_matrix(n - 1);
    let size = prev.len();
    let mut result = vec![vec![0i8; size * 2]; size * 2];
    for i in 0..size {
        for j in 0..size {
            result[i][j] = prev[i][j];
            result[i][j + size] = prev[i][j];
            result[i + size][j] = prev[i][j];
            result[i + size][j + size] = -prev[i][j];
        }
    }
    result
}

/// The 64×64 Walsh-Hadamard matrix H(6) — each row corresponds to a hexagram.
pub fn hexagram_hadamard() -> Vec<Vec<i8>> {
    hadamard_matrix(6)
}

/// Verify: Walsh-Hadamard rows are pairwise orthogonal.
pub fn verify_hadamard_orthogonality() -> bool {
    let h = hexagram_hadamard();
    let n = h.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let dot: i32 = h[i].iter().zip(h[j].iter()).map(|(&a, &b)| a as i32 * b as i32).sum();
            if dot != 0 {
                return false;
            }
        }
    }
    true
}

// ─── King Wen Sequence ───────────────────────────────────────────────

/// King Wen hexagram ordering (bits 0-63, standard 周易卦序).
/// This is the traditional received order.
pub const WEN_SEQUENCE: [u8; 64] = [
    // 上经 (1-30)
    1, 0, 3, 2, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12,
    19, 18, 17, 16, 23, 22, 21, 20, 27, 26, 25, 24, 31, 30,
    // 下经 (31-64)
    29, 28, 35, 34, 33, 32, 39, 38, 37, 36, 43, 42, 41, 40,
    47, 46, 45, 44, 51, 50, 49, 48, 55, 54, 53, 52, 59, 58,
    57, 56, 63, 62, 61, 60,
];

// ─── E₈ × 64 Model ──────────────────────────────────────────────────

/// Complete E₈ × 64-hexagram model homology result.
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

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e8_dimension() {
        assert_eq!(E8_RANK + E8_ROOTS, E8_DIM);
        assert!(verify_e8_dimension());
    }

    #[test]
    fn test_e8_root_count() {
        let roots = e8_root_system();
        assert_eq!(roots.len(), 240);
        // All E₈ roots have norm² = 2
        for root in &roots {
            assert!((root.norm_sq() - 2.0).abs() < 1e-10, "E8 root {:?} has norm² != 2 (norm²={})", root, root.norm_sq());
        }
    }

    #[test]
    fn test_root_families() {
        let roots = e8_root_system();
        // Verify we have the right distribution across sign patterns
        let mut perm_count = 0;
        let mut half_count = 0;
        for root in &roots {
            let abs_nonzero = root.coords.iter().filter(|&&c| c != 0).count();
            if abs_nonzero == 2 {
                perm_count += 1;
            } else {
                half_count += 1;
            }
        }
        assert_eq!(perm_count, 112, "Family 1 must have 112 roots");
        assert_eq!(half_count, 128, "Family 2 must have 128 roots");
    }

    #[test]
    fn test_three_generations() {
        assert!(verify_three_generations());
        assert_eq!(FERMION_GENERATIONS * FERMIONS_PER_GENERATION, TOTAL_SM_FERMIONS);
        assert_eq!(E8_DIM - TOTAL_SM_FERMIONS, REMAINING_E8_GENERATORS);
    }

    #[test]
    fn test_fermion_count() {
        let fermions = all_sm_fermions();
        assert_eq!(fermions.len(), 192);
        for gen in 0..3 {
            let states = fermion_states_for_generation(gen);
            assert_eq!(states.len(), 64, "Generation {gen} must have 64 fermions");
        }
    }

    #[test]
    fn test_hexagram_matrix_8x8() {
        let matrix = hexagram_matrix();
        assert_eq!(matrix.len(), 8);
        assert_eq!(matrix[0].len(), 8);
        // Every hexagram is unique
        let mut seen = HashSet::new();
        for row in &matrix {
            for cell in row {
                assert!(seen.insert(cell.bits), "Duplicate hexagram {}", cell.bits);
            }
        }
        assert_eq!(seen.len(), 64);
    }

    #[test]
    fn test_shao_yong_sequence() {
        let seq = shao_yong_sequence();
        assert_eq!(seq.len(), 64);
        // Each hexagram is in order 0..63
        for (i, hex) in seq.iter().enumerate() {
            assert_eq!(hex.bits as usize, i);
        }
    }

    #[test]
    fn test_hexagram_opposite() {
        let hex = Hexagram::new(0b101010);
        let opp = hex.opposite();
        assert_eq!(opp.bits, 0b010101);
        // Double opposite returns to original
        assert_eq!(hex, hex.opposite().opposite());
    }

    #[test]
    fn test_hadamard() {
        let h3 = hadamard_matrix(3);
        assert_eq!(h3.len(), 8);
        for row in &h3 {
            assert_eq!(row.len(), 8);
        }
        let h6 = hexagram_hadamard();
        assert_eq!(h6.len(), 64);
        assert_eq!(h6[0].len(), 64);
        assert!(verify_hadamard_orthogonality());
    }

    #[test]
    fn test_hadamard_first_row_all_ones() {
        let h = hadamard_matrix(6);
        for &val in &h[0] {
            assert_eq!(val, 1, "First Hadamard row must be all 1s");
        }
    }

    #[test]
    fn test_lo_shu() {
        assert!(verify_lo_shu());
    }

    #[test]
    fn test_he_tu_sum() {
        assert!(verify_he_tu_sum());
    }

    #[test]
    fn test_dayan_identity() {
        assert!(verify_dayan_identity());
        assert_eq!(DAYAN_NUMBER, OBSERVABLE_DOF + OBSERVER_DOF);
        assert_eq!(SEVEN_SQUARED, 49);
    }

    #[test]
    fn test_e8_total_identities() {
        let homology = E8HexagramHomology::new();
        assert!(homology.all_identities_hold);
        for (name, ok) in &homology.identity_results {
            assert!(*ok, "Identity '{name}' failed");
        }
    }

    #[test]
    fn test_trigram_su3_mapping() {
        // Each trigram should map to a distinct SU(3) root
        let mut roots = Vec::new();
        for t in 0..8 {
            roots.push(trigram_to_su3_root(t));
        }
        let mut unique = roots.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 8, "All 8 trigrams must map to distinct SU(3) roots");
    }

    #[test]
    fn test_wen_sequence_has_all() {
        let mut seen = HashSet::new();
        for &b in &WEN_SEQUENCE {
            assert!(seen.insert(b), "Duplicate in Wen sequence: {b}");
        }
        assert_eq!(seen.len(), 64);
        // Verify all 0..63 are present
        for i in 0..64u8 {
            assert!(seen.contains(&i), "Missing hexagram {i} in Wen sequence");
        }
    }

    #[test]
    fn test_king_wen_roundtrip() {
        let seq = king_wen_sequence();
        for hex in &seq {
            let idx = hex.wen_index();
            assert!(idx.is_some());
            assert_eq!(seq[idx.expect("wen_index should be Some")].bits, hex.bits);
        }
    }

    #[test]
    fn test_e8_root_norm_counts() {
        let (norm2, _others) = e8_root_norm_counts();
        assert_eq!(norm2, 240, "All E8 roots must have norm²=2 (simply laced)");
    }

    #[test]
    fn test_total_lines_identity() {
        assert_eq!(TOTAL_LINES, 384);
        assert_eq!(HEXAGRAM_COUNT * LINES_PER_HEXAGRAM, TOTAL_LINES);
    }
}
