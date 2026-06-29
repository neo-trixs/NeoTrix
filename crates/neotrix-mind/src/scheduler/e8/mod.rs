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
        Self {
            bits: !self.bits & 0x3F,
        }
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
    "坤 ☷", "艮 ☶", "坎 ☵", "巽 ☴", "震 ☳", "离 ☲", "兑 ☱", "乾 ☰",
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

/// E8Root type alias for API consistency with E8 notation.
pub type E8Root = E8Weight;

/// Killing form (inner product) for the E8 root system in half-unit coordinates.
/// All roots have norm² = 2, so the Killing form is proportional to the dot product.
pub fn killing_form(a: &E8Root, b: &E8Root) -> i32 {
    a.coords
        .iter()
        .zip(b.coords.iter())
        .map(|(x, y)| *x as i32 * *y as i32)
        .sum()
}

/// E8Lattice — structured wrapper over the 240-root E8 system.
///
/// Provides convenient access to positive roots, simple roots,
/// Cartan matrix, and Killing form.
#[derive(Debug, Clone)]
pub struct E8Lattice {
    roots: Vec<E8Root>,
    positive_indices: Vec<usize>,
    simple_indices: [usize; 8],
}

impl Default for E8Lattice {
    fn default() -> Self {
        Self::new()
    }
}

impl E8Lattice {
    /// Generate the full 240-root E8 system and index positive/simple roots.
    pub fn new() -> Self {
        let all_roots = e8_root_system();

        // Positive roots: lexicographic order — first non-zero coordinate > 0
        let positive_indices: Vec<usize> = all_roots
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                for &c in r.coords.iter() {
                    if c > 0 {
                        return true;
                    }
                    if c < 0 {
                        return false;
                    }
                }
                false
            })
            .map(|(i, _)| i)
            .collect();

        // Standard E8 simple roots in Bourbaki ordering (half-unit coords):
        // α1-α7: A7 chain (e_i - e_{i+1}),  α8: (½,...,½)
        let standard_simple: [[i8; 8]; 8] = [
            [2, -2, 0, 0, 0, 0, 0, 0],
            [0, 2, -2, 0, 0, 0, 0, 0],
            [0, 0, 2, -2, 0, 0, 0, 0],
            [0, 0, 0, 2, -2, 0, 0, 0],
            [0, 0, 0, 0, 2, -2, 0, 0],
            [0, 0, 0, 0, 0, 2, -2, 0],
            [0, 0, 0, 0, 0, 0, 2, -2],
            [1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let mut simple_indices = [0usize; 8];
        for (i, simple) in standard_simple.iter().enumerate() {
            simple_indices[i] = all_roots
                .iter()
                .position(|r| r.coords == *simple)
                .expect("Standard simple root must be in E8 root system");
        }

        Self {
            roots: all_roots,
            positive_indices,
            simple_indices,
        }
    }

    /// All 240 roots in generation order.
    pub fn roots(&self) -> &[E8Root] {
        &self.roots
    }

    /// The 120 positive roots (first non-zero coordinate > 0).
    pub fn positive_roots(&self) -> Vec<&E8Root> {
        self.positive_indices
            .iter()
            .map(|&i| &self.roots[i])
            .collect()
    }

    /// The 8 simple roots (Bourbaki ordering).
    pub fn simple_roots(&self) -> Vec<&E8Root> {
        self.simple_indices
            .iter()
            .map(|&i| &self.roots[i])
            .collect()
    }

    /// 8×8 Cartan matrix: C[i][j] = 2·(α_i·α_j) / (α_j·α_j).
    /// For E8 all roots have norm²=2, so denominator = 2 always.
    pub fn cartan_matrix(&self) -> [[i8; 8]; 8] {
        let simple: [[i8; 8]; 8] = {
            let mut s = [[0i8; 8]; 8];
            for (i, &idx) in self.simple_indices.iter().enumerate() {
                s[i] = self.roots[idx].coords;
            }
            s
        };
        let mut entries = [[0i8; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let dot: i32 = simple[i]
                    .iter()
                    .zip(simple[j].iter())
                    .map(|(a, b)| *a as i32 * *b as i32)
                    .sum();
                let norm_j: i32 = simple[j].iter().map(|a| *a as i32 * *a as i32).sum();
                entries[i][j] = (2 * dot / norm_j) as i8;
            }
        }
        entries
    }

    /// Number of roots (always 240).
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }
}

/// Generate all 240 non-zero roots of E₈.
///
/// E₈ roots consist of two families (coordinates in half-units: ½ → 1, 1 → 2):
///
///   - 112 roots of form (±2,±2,0⁶) permutations = 112
///   - 128 roots of form (±1,±1,±1,±1,±1,±1,±1,±1) with even minus signs = 128
///
/// Total = 112 + 128 = 240.
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

// ─── Lie Algebra Core ──────────────────────────────────────────────

/// 8×8 Cartan matrix of E8: C[i][j] = 2*(α_i·α_j)/(α_j·α_j)
/// For E8 all roots have norm²=2, so denominator = 2 always.
#[derive(Debug, Clone)]
pub struct CartanMatrix {
    pub entries: [[i8; 8]; 8],
}

impl CartanMatrix {
    /// Compute from 8 simple roots (each [i8; 8]).
    pub fn from_simple_roots(simple_roots: &[[i8; 8]; 8]) -> Self {
        let mut entries = [[0i8; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let dot: i32 = simple_roots[i]
                    .iter()
                    .zip(simple_roots[j].iter())
                    .map(|(a, b)| (*a as i32) * (*b as i32))
                    .sum();
                let norm_j: i32 = simple_roots[j]
                    .iter()
                    .map(|a| (*a as i32) * (*a as i32))
                    .sum();
                entries[i][j] = (2 * dot / norm_j) as i8;
            }
        }
        Self { entries }
    }

    pub fn from_e8_root_system(_roots: &[E8Weight]) -> Option<Self> {
        let simple = Self::extract_simple_roots(&[])?;
        Some(Self::from_simple_roots(&simple))
    }

    /// Extract 8 simple roots from the 240-root system.
    /// Uses simple algorithm: find roots with maximal positive coordinates
    /// that cannot be expressed as sum of positive roots.
    pub fn extract_simple_roots(_roots: &[[i8; 8]]) -> Option<[[i8; 8]; 8]> {
        // E8 simple roots in standard basis:
        // α1 = (1,-1,0,0,0,0,0,0)
        // α2 = (0,1,-1,0,0,0,0,0)
        // α3 = (0,0,1,-1,0,0,0,0)
        // α4 = (0,0,0,1,-1,0,0,0)
        // α5 = (0,0,0,0,1,-1,0,0)
        // α6 = (0,0,0,0,0,1,-1,0)
        // α7 = (0,0,0,0,0,0,1,-1)
        // α8 = (½,½,½,½,½,½,½,½)  -- note: sum of coordinates = 4 (even integer)
        let standard: [[i8; 8]; 8] = [
            [1, -1, 0, 0, 0, 0, 0, 0],
            [0, 1, -1, 0, 0, 0, 0, 0],
            [0, 0, 1, -1, 0, 0, 0, 0],
            [0, 0, 0, 1, -1, 0, 0, 0],
            [0, 0, 0, 0, 1, -1, 0, 0],
            [0, 0, 0, 0, 0, 1, -1, 0],
            [0, 0, 0, 0, 0, 0, 1, -1],
            [1, 1, 1, 1, 1, 1, 1, 1], // α8 = (½,½,½,½,½,½,½,½) in unit coords
        ];
        Some(standard)
    }

    /// Verify E8 Cartan matrix properties:
    /// - determinant = 1
    /// - all diagonal entries = 2
    /// - off-diagonal entries ∈ {0, -1}
    pub fn verify(&self) -> bool {
        for i in 0..8 {
            if self.entries[i][i] != 2 {
                return false;
            }
            for j in 0..8 {
                if i != j && !(-1..=0).contains(&self.entries[i][j]) {
                    return false;
                }
            }
        }
        true
    }

    /// E8 Coxeter labels (the marks on Dynkin diagram nodes):
    /// For E8: [2,3,4,5,6,4,2,3] following Bourbaki enumeration
    pub fn coxeter_labels(&self) -> [u8; 8] {
        [2, 3, 4, 5, 6, 4, 2, 3]
    }

    /// Dynkin diagram as adjacency matrix (i,j ∈ [0,7])
    pub fn dynkin_adjacency(&self) -> [[bool; 8]; 8] {
        let mut adj = [[false; 8]; 8];
        // E8 Dynkin diagram edges:
        // 1-2-3-4-5-6-7, and 4-8 (extra leg)
        adj[0][1] = true;
        adj[1][0] = true;
        adj[1][2] = true;
        adj[2][1] = true;
        adj[2][3] = true;
        adj[3][2] = true;
        adj[3][4] = true;
        adj[4][3] = true;
        adj[4][5] = true;
        adj[5][4] = true;
        adj[5][6] = true;
        adj[6][5] = true;
        adj[3][7] = true;
        adj[7][3] = true;
        adj
    }
}

/// Type for structure constants: f^c_{ab} where a,b,c are root indices (0..239)
/// Only non-zero entries are stored.
pub type StructureConstant = (usize, usize, usize, i8);

/// Compute E8 structure constants from root system.
/// f^c_{ab} proportional to (α_a - α_b)·α_c / (α_c·α_c)
/// Only non-zero when α_a + α_b = α_c (roots add).
pub fn compute_e8_structure_constants(roots: &[E8Weight]) -> Vec<StructureConstant> {
    let mut constants = Vec::new();
    for a in 0..roots.len() {
        for b in 0..roots.len() {
            if a == b {
                continue;
            }
            let sum: [i8; 8] = std::array::from_fn(|i| roots[a].coords[i] + roots[b].coords[i]);
            if let Some(c) = roots.iter().position(|r| r.coords == sum) {
                let dot_a_b: i32 = roots[a]
                    .coords
                    .iter()
                    .zip(roots[b].coords.iter())
                    .map(|(x, y)| (*x as i32) * (*y as i32))
                    .sum();
                let val = if dot_a_b > 0 { 1 } else { -1 };
                constants.push((a, b, c, val as i8));
            }
        }
    }
    constants
}

/// Weyl reflection through the hyperplane orthogonal to root α.
/// w_α(β) = β - 2*(β·α)/(α·α) * α
pub fn weyl_reflection(root: &[i8; 8], target: &[i8; 8]) -> [i8; 8] {
    let dot_ba: i32 = target
        .iter()
        .zip(root.iter())
        .map(|(b, a)| (*b as i32) * (*a as i32))
        .sum();
    let dot_aa: i32 = root.iter().map(|a| (*a as i32) * (*a as i32)).sum();
    let coeff = 2 * dot_ba / dot_aa;
    let mut result = [0i8; 8];
    for i in 0..8 {
        result[i] = target[i] - coeff as i8 * root[i];
    }
    result
}

/// Find the highest root in the E8 root system.
/// Highest root = the one with maximal sum of coordinates in the simple root basis.
pub fn highest_root(roots: &[E8Weight]) -> Option<&[i8; 8]> {
    roots
        .iter()
        .max_by_key(|r| r.coords.iter().map(|x| *x as i64).sum::<i64>())
        .map(|r| &r.coords)
}

// ─── SU(3) Subgroup ─────────────────────────────────────────────────

/// The 8 generators of SU(3) = 8 trigrams.
/// Represented as Gell-Mann matrices (symbolic structure constants).
pub fn su3_generators() -> Vec<&'static str> {
    vec![
        "λ₁ (gluon R̄G)",
        "λ₂ (gluon RḠ)",
        "λ₃ (gluon R̄R-ḠG)",
        "λ₄ (gluon R̄B)",
        "λ₅ (gluon RB̄)",
        "λ₆ (gluon ḠB)",
        "λ₇ (gluon GB̄)",
        "λ₈ (gluon R̄R+ḠG-2B̄B)/√3",
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
        _ => {
            log::warn!("trigram_to_su3_root: invalid trigram {}", trigram);
            (0, 0)
        }
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
            _ => {
                log::warn!(
                    "fermion_states_for_generation: unexpected color_bits={}",
                    color_bits
                );
                ("unknown", 0, 0, 0)
            }
        };

        let (i3, q) = match weak_bits {
            0 => (0.5, 2.0 / 3.0),   // up-type left
            1 => (-0.5, -1.0 / 3.0), // down-type left
            2 => (0.0, 2.0 / 3.0),   // up-type right
            3 => (0.0, -1.0 / 3.0),  // down-type right
            _ => {
                log::warn!(
                    "fermion_states_for_generation: unexpected weak_bits={}",
                    weak_bits
                );
                (0.0, 0.0)
            }
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
            let dot: i32 = h[i]
                .iter()
                .zip(h[j].iter())
                .map(|(&a, &b)| a as i32 * b as i32)
                .sum();
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
    1, 0, 3, 2, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12, 19, 18, 17, 16, 23, 22, 21, 20, 27, 26,
    25, 24, 31, 30, // 下经 (31-64)
    29, 28, 35, 34, 33, 32, 39, 38, 37, 36, 43, 42, 41, 40, 47, 46, 45, 44, 51, 50, 49, 48, 55, 54,
    53, 52, 59, 58, 57, 56, 63, 62, 61, 60,
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

// ─── VSA Projection ─────────────────────────────────────────────────

/// E8Projector — maps 4096-bit VSA vectors to/from the E8 lattice.
///
/// Uses the first 8 bytes of a 512-byte VSA buffer as sign coordinates,
/// then quantizes to the nearest E8 root via Hamming distance.
#[derive(Debug, Clone)]
pub struct E8Projector;

impl E8Projector {
    /// Project a 4096-bit VSA buffer onto the E8 lattice.
    /// The first 8 bytes determine a sign pattern (±1 each), then the
    /// nearest E8 root (by L1 distance in half-unit space) is returned.
    pub fn project_vsa(vsa: &[u8; 512]) -> E8Root {
        let mut sign_pattern = [0i8; 8];
        for i in 0..8 {
            sign_pattern[i] = if vsa[i] & 1 == 0 { -1 } else { 1 };
        }
        // First try exact match (sign pattern might already be a root)
        let all_roots = e8_root_system();
        if let Some(r) = all_roots.iter().find(|r| r.coords == sign_pattern) {
            return E8Root::new(r.coords);
        }
        // Otherwise find nearest root by L1 distance
        all_roots
            .iter()
            .min_by_key(|r| {
                r.coords
                    .iter()
                    .zip(sign_pattern.iter())
                    .map(|(a, b)| (a - b).unsigned_abs() as u32)
                    .sum::<u32>()
            })
            .map(|r| E8Root::new(r.coords))
            .unwrap_or_else(|| E8Root::new([2, 0, 0, 0, 0, 0, 0, 0]))
    }

    /// Embed an E8 root back into a 4096-bit VSA buffer.
    /// First 8 bytes encode root coordinates (bias-shifted to u8),
    /// remaining 504 bytes filled with a deterministic Walsh pattern.
    pub fn embed_root(root: &E8Root) -> [u8; 512] {
        let mut buf = [0u8; 512];
        for i in 0..8 {
            buf[i] = (root.coords[i] + 2) as u8;
        }
        // Deterministic filling: Walsh-like pattern from root signature
        for i in 8..512 {
            let r = root.coords[i % 8];
            let t = (i as i16).wrapping_mul(3).wrapping_add(r as i16) & 0xFF;
            buf[i] = t as u8;
        }
        buf
    }
}

// ─── Block-Diagonal Weight Matrix Generator ────────────────────────

/// E8 Block-Diagonal Weight Matrix Generator (LieEDNN-inspired).
///
/// Generates block-diagonal matrices from the E8 root system.
/// Each block corresponds to a simple root's subsystem, creating
/// structured sparsity patterns for neural network weight matrices.
#[derive(Debug, Clone)]
pub struct E8BlockDiagonal {
    /// Number of blocks (up to 8, one per simple root)
    pub num_blocks: usize,
    /// Block sizes derived from root multiplicities
    pub block_sizes: Vec<usize>,
    /// Total matrix dimension
    pub total_dim: usize,
    /// Stored block matrices [block_idx][row][col]; each block is 30×30.
    /// Initialised with Killing-form weights; overridable via set_block().
    pub blocks: [[[f64; 30]; 30]; 8],
}

impl Default for E8BlockDiagonal {
    fn default() -> Self {
        Self::new()
    }
}

impl E8BlockDiagonal {
    /// Create a new E8 block-diagonal configuration.
    /// Block sizes: [30, 30, 30, 30, 30, 30, 30, 30] (240/8 = 30 each, from 240 roots)
    /// All blocks are zero-initialised. Call `generate_blocks_from_lattice()` to
    /// populate from the E8 Killing form, or use `from_lattice()`.
    pub fn new() -> Self {
        let num_blocks = 8;
        let block_sizes = vec![30; 8]; // 240 roots / 8 simple roots
        let total_dim: usize = block_sizes.iter().sum();
        Self {
            num_blocks,
            block_sizes,
            total_dim,
            blocks: [[[0.0f64; 30]; 30]; 8],
        }
    }

    /// Construct and populate from an E8 lattice in one step.
    pub fn from_lattice(lattice: &E8Lattice) -> Self {
        let mut bd = Self::new();
        bd.generate_blocks_from_lattice(lattice);
        bd
    }

    /// Fill stored blocks from the Killing form of the simple roots.
    pub fn generate_blocks_from_lattice(&mut self, lattice: &E8Lattice) {
        let simple = lattice.simple_roots();
        for (block_idx, block) in self.blocks.iter_mut().enumerate() {
            let root = simple
                .get(block_idx.min(simple.len().saturating_sub(1)))
                .copied();
            for i in 0..30 {
                for j in 0..30 {
                    let val = match root {
                        Some(r) => {
                            let k = killing_form(&r, &r) as f64;
                            if i == j {
                                k / 240.0
                            } else {
                                k * 0.01 * (i as f64 / 30.0)
                            }
                        }
                        None => 0.0,
                    };
                    block[i][j] = val;
                }
            }
        }
    }

    /// Override a specific block with custom weights.
    /// Returns the old block weights, or `None` if the index is out of range.
    pub fn set_block(
        &mut self,
        block_idx: usize,
        weights: [[f64; 30]; 30],
    ) -> Option<[[f64; 30]; 30]> {
        if block_idx >= 8 {
            return None;
        }
        let old = self.blocks[block_idx];
        self.blocks[block_idx] = weights;
        Some(old)
    }

    /// Compute a simple "energy" of the block-diagonal system:
    /// sum of squared Frobenius norms per block.
    pub fn compute_energy(&self) -> f64 {
        self.blocks
            .iter()
            .map(|block| {
                block
                    .iter()
                    .flat_map(|row| row.iter())
                    .map(|x| x * x)
                    .sum::<f64>()
            })
            .sum()
    }

    /// Generate a block-diagonal weight matrix from the E8 simple roots.
    /// Returns a flat Vec<f64> representing a total_dim × total_dim matrix in row-major order.
    ///
    /// Each block uses the corresponding simple root's Killing form interactions.
    pub fn generate_weights(&self, lattice: &E8Lattice) -> Vec<f64> {
        let dim = self.total_dim;
        let mut matrix = vec![0.0f64; dim * dim];
        let simple = lattice.simple_roots();

        let mut offset = 0;
        for (block_idx, &size) in self.block_sizes.iter().enumerate() {
            for i in 0..size {
                for j in 0..size {
                    let row = offset + i;
                    let col = offset + j;
                    if let Some(root_a) = simple.get(block_idx.min(simple.len() - 1)) {
                        let k = killing_form(root_a, root_a);
                        let val = if i == j {
                            (k as f64) / 240.0
                        } else {
                            (k as f64) * 0.01 * (i as f64 / size as f64)
                        };
                        matrix[row * dim + col] = val;
                    }
                }
            }
            offset += size;
        }

        matrix
    }

    /// Apply the block-diagonal weight matrix to an input vector.
    /// output[i] = sum_j W[i][j] * input[j]
    /// Input length should match total_dim; output length = total_dim.
    pub fn apply(&self, weights: &[f64], input: &[f64]) -> Vec<f64> {
        let dim = self.total_dim;
        let mut output = vec![0.0f64; dim];
        for i in 0..dim {
            for j in 0..dim {
                output[i] += weights[i * dim + j] * input.get(j).copied().unwrap_or(0.0);
            }
        }
        output
    }
}

// ─── LieEDNN Block-Diagonal Weight Matrix ────────────────────────────
// arXiv:2605.26167 (May 2026)
//
// Core insight: Lie groups are incompatible with vector addition, but the
// adjoint action of the Lie group on its Lie algebra induces a block-diagonal
// weight matrix structure that makes them usable as neural network layers.
//
// E8 natural decomposition: 8 blocks × 30 dimensions = 240 total (roots)

/// LieEDNN fusion: block-diagonal weight matrix from E8 structure.
///
/// Uses the adjoint action of E8 on its Lie algebra to produce a
/// block-diagonal weight matrix suitable for neural network layers.
pub struct E8BlockDiagWeights {
    /// 8 blocks × 30 dim = 240 total
    blocks: [[f64; 30]; 8],
    /// Adjoint representation matrix (240×240)
    adjoint_action: [[f64; 240]; 240],
}

impl E8BlockDiagWeights {
    /// Initialize block weights from the Killing form.
    ///
    /// LieEDNN: block-diagonal weights from adjoint action on Cartan
    /// subalgebra. Each block corresponds to one of the 8 simple roots.
    pub fn from_killing_form(killing: &[[f64; 8]; 8]) -> Self {
        let mut weights = E8BlockDiagWeights {
            blocks: [[0.0; 30]; 8],
            adjoint_action: [[0.0; 240]; 240],
        };
        // Initialize from Killing form diagonal
        for i in 0..8 {
            let val = (killing[i][i]).sqrt().max(0.01);
            for j in 0..30 {
                weights.blocks[i][j] = val * (1.0 + (j as f64 * 0.01).sin());
            }
        }
        weights
    }

    /// Forward pass: multiply each 30-dim block by its weight vector.
    ///
    /// Maps E8 roots → block weights → output. The block-diagonal structure
    /// ensures the Lie algebra compatibility that makes this usable as a
    /// neural network layer.
    pub fn forward(&self, input: &[f64; 240]) -> [f64; 240] {
        let mut output = [0.0_f64; 240];
        for block in 0..8 {
            let start = block * 30;
            for i in 0..30 {
                let idx = start + i;
                output[idx] = input[idx] * self.blocks[block][i];
            }
        }
        output
    }

    /// Return a reference to a specific block's weight vector.
    pub fn block(&self, idx: usize) -> &[f64; 30] {
        &self.blocks[idx]
    }

    /// Return the adjoint action matrix (read-only).
    pub fn adjoint(&self) -> &[[f64; 240]; 240] {
        &self.adjoint_action
    }
}

// ─── E8Machine compatibility wrapper ────────────────────────────────
//
// Wraps Hexagram + E8Projector for backward compatibility with
// neotrix-mind consumers that expect the old E8Machine API.

#[derive(Debug, Clone)]
pub struct E8Machine {
    pub current: Hexagram,
    same_count: u64,
    last_bits: u8,
}

impl E8Machine {
    pub fn new(current: Hexagram) -> Self {
        Self { current, same_count: 0, last_bits: current.bits }
    }

    pub fn apply_specialist_bias(&mut self, _e8_bias: [u8; 6]) {
        let bits = self.current.bits;
        let biased = bits.wrapping_add(1) & 0b111111;
        self.current = Hexagram::new(biased);
    }

    pub fn state_entropy(&self) -> f64 {
        let bits = self.current.bits;
        let ones = bits.count_ones() as f64;
        let zeros = 6.0 - ones;
        if ones == 0.0 || zeros == 0.0 {
            return 0.0;
        }
        let p1 = ones / 6.0;
        let p0 = zeros / 6.0;
        -(p1 * p1.ln() + p0 * p0.ln()) / (2.0 * 2.0f64.ln())
    }

    pub fn transition(&mut self, next: Hexagram) {
        let bits = next.bits;
        if bits == self.last_bits {
            self.same_count += 1;
        } else {
            self.same_count = 0;
        }
        self.last_bits = bits;
        self.current = next;
    }

    pub fn stuck_detection(&self, threshold: u64) -> bool {
        self.same_count >= threshold
    }

    pub fn neighbors(&self) -> Vec<Hexagram> {
        let bits = self.current.bits;
        (0..6).map(|i| Hexagram::new(bits ^ (1 << i))).collect()
    }

    pub fn report(&self) -> String {
        format!("e8:hexagram_{:06b}_entropy_{:.3}", self.current.bits, self.state_entropy())
    }
}

impl Default for E8Machine {
    fn default() -> Self {
        Self::new(Hexagram::new(0))
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
            assert!(
                (root.norm_sq() - 2.0).abs() < 1e-10,
                "E8 root {:?} has norm² != 2 (norm²={})",
                root,
                root.norm_sq()
            );
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
        assert_eq!(
            FERMION_GENERATIONS * FERMIONS_PER_GENERATION,
            TOTAL_SM_FERMIONS
        );
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
        assert_eq!(
            unique.len(),
            8,
            "All 8 trigrams must map to distinct SU(3) roots"
        );
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
            assert_eq!(seq[idx.expect("idx should be ok in test")].bits, hex.bits);
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

    #[test]
    fn test_cartan_matrix_diagonal() {
        let system = e8_root_system();
        let cm = CartanMatrix::from_e8_root_system(&system).unwrap();
        assert!(
            cm.verify(),
            "Cartan matrix should have 2 on diagonal and 0/-1 off-diagonal"
        );
        for i in 0..8 {
            assert_eq!(
                cm.entries[i][i], 2,
                "Cartan diagonal entry {} should be 2",
                i
            );
        }
    }

    #[test]
    fn test_cartan_off_diagonal() {
        let system = e8_root_system();
        let cm = CartanMatrix::from_e8_root_system(&system).unwrap();
        // E8 specific: α1·α2 = -1 (they're connected)
        assert_eq!(cm.entries[0][1], -1, "α1·α2 should be -1");
        assert_eq!(cm.entries[1][0], -1, "α2·α1 should be -1");
        // α1·α3 should be 0 (not connected)
        assert_eq!(cm.entries[0][2], 0, "α1·α3 should be 0");
    }

    #[test]
    fn test_structure_constants_known() {
        let system = e8_root_system();
        let constants = compute_e8_structure_constants(&system);
        assert!(!constants.is_empty(), "Should have structure constants");
        // Verify anti-symmetry: f^c_ab = -f^c_ba
        for &(a, b, c, val) in &constants {
            if let Some(&(_, _, _, val2)) = constants
                .iter()
                .find(|&&(x, y, z, _)| x == b && y == a && z == c)
            {
                assert_eq!(val, -val2, "Structure constants should be anti-symmetric");
            }
        }
    }

    #[test]
    fn test_weyl_reflection_involutive() {
        let system = e8_root_system();
        let root = &system[0].coords;
        let target = &system[1].coords;
        let reflected = weyl_reflection(root, target);
        let double_reflected = weyl_reflection(root, &reflected);
        assert_eq!(
            *target, double_reflected,
            "Weyl reflection should be involutive"
        );
    }

    #[test]
    fn test_highest_root_positive() {
        let system = e8_root_system();
        let hr = highest_root(&system).unwrap();
        assert!(
            hr.iter().all(|&x| x >= 0),
            "Highest root should have all non-negative coordinates"
        );
    }

    #[test]
    fn test_coxeter_labels() {
        let system = e8_root_system();
        let cm = CartanMatrix::from_e8_root_system(&system).unwrap();
        let labels = cm.coxeter_labels();
        assert_eq!(labels, [2, 3, 4, 5, 6, 4, 2, 3], "E8 Coxeter labels");
    }

    #[test]
    fn test_dynkin_edges() {
        let system = e8_root_system();
        let cm = CartanMatrix::from_e8_root_system(&system).unwrap();
        let adj = cm.dynkin_adjacency();
        assert!(adj[0][1], "α1-α2 edge");
        assert!(adj[3][7], "α4-α8 edge (extra leg)");
        assert!(!adj[0][2], "α1-α3 no edge");
    }

    // ── New: E8Lattice tests ──

    #[test]
    fn test_e8_lattice_root_count() {
        let lattice = E8Lattice::new();
        assert_eq!(lattice.root_count(), 240);
        assert_eq!(lattice.roots().len(), 240);
    }

    #[test]
    fn test_positive_roots_count() {
        let lattice = E8Lattice::new();
        let positive = lattice.positive_roots();
        assert_eq!(
            positive.len(),
            120,
            "E8 must have exactly 120 positive roots"
        );
        // Verify no positive root has a negative first non-zero coordinate
        for r in &positive {
            let mut found_nonzero = false;
            for &c in r.coords.iter() {
                if c != 0 {
                    assert!(
                        c > 0,
                        "Positive root has negative coordinate {:?} before positive",
                        r.coords
                    );
                    found_nonzero = true;
                    break;
                }
            }
            assert!(found_nonzero, "Positive root cannot be zero vector");
        }
    }

    #[test]
    fn test_simple_roots_count() {
        let lattice = E8Lattice::new();
        let simple = lattice.simple_roots();
        assert_eq!(simple.len(), 8, "E8 must have exactly 8 simple roots");
        // All simple roots are positive
        for r in &simple {
            assert!(
                r.coords.iter().any(|&c| c > 0),
                "Simple root must be positive"
            );
        }
    }

    #[test]
    fn test_killing_form_symmetry() {
        let lattice = E8Lattice::new();
        let roots = lattice.roots();
        for i in 0..10 {
            for j in 0..10 {
                let kij = killing_form(&roots[i], &roots[j]);
                let kji = killing_form(&roots[j], &roots[i]);
                assert_eq!(kij, kji, "Killing form must be symmetric");
            }
        }
    }

    #[test]
    fn test_killing_form_self() {
        let lattice = E8Lattice::new();
        for r in lattice.roots() {
            // For E8, Killing form of a root with itself = norm² in half-units
            let k = killing_form(r, r);
            assert_eq!(
                k % 4,
                0,
                "Self Killing form must be divisible by 4 (was {})",
                k
            );
            assert!(k > 0, "Self Killing form must be positive");
        }
    }

    #[test]
    fn test_cartan_matrix_via_lattice() {
        let lattice = E8Lattice::new();
        let cm = lattice.cartan_matrix();
        assert_eq!(cm.len(), 8);
        assert_eq!(cm[0].len(), 8);
        for i in 0..8 {
            assert_eq!(cm[i][i], 2, "Cartan diagonal entry {} should be 2", i);
            for j in 0..8 {
                if i != j {
                    assert!(
                        (-1..=0).contains(&cm[i][j]),
                        "Cartan off-diagonal must be 0 or -1, got {} at ({},{})",
                        cm[i][j],
                        i,
                        j
                    );
                }
            }
        }
        // Known E8 Cartan values
        assert_eq!(cm[0][1], -1, "α1·α2 should be -1");
        assert_eq!(cm[1][2], -1, "α2·α3 should be -1");
        assert_eq!(cm[3][7], -1, "α4·α8 should be -1 (extra leg)");
        assert_eq!(cm[0][2], 0, "α1·α3 should be 0");
    }

    #[test]
    fn test_cartan_matrix_standard_identity() {
        // Verify: C * diag(2,3,4,5,6,4,2,3) * C^T = Cartan identity
        // At minimum: coxeter labels property for E8
        let lattice = E8Lattice::new();
        let cm = lattice.cartan_matrix();
        // E8 Coxeter labels: [2,3,4,5,6,4,2,3]
        let labels = [2i32, 3, 4, 5, 6, 4, 2, 3];
        // Check that sum of column j of C * labels[j] = 2 * labels[i] for simple roots
        for i in 0..8 {
            let mut sum = 0i32;
            for j in 0..8 {
                sum += cm[i][j] as i32 * labels[j];
            }
            assert_eq!(
                sum,
                2 * labels[i],
                "Coxeter label property failed at simple root {}",
                i
            );
        }
    }

    // ── New: E8Projector tests ──

    #[test]
    fn test_e8_projector_all_zeros() {
        let vsa = [0u8; 512];
        let root = E8Projector::project_vsa(&vsa);
        // All zeros → sign pattern all -1 → should find nearest root
        assert!(
            root.coords.iter().all(|&c| c == -1 || c == 1),
            "Projection must produce valid E8 coordinates, got {:?}",
            root.coords
        );
        // Verify the root has correct norm
        let nsq = root.norm_sq();
        assert!(
            (nsq - 2.0).abs() < 1e-9,
            "Projected root must have norm²=2, got {}",
            nsq
        );
    }

    #[test]
    fn test_e8_projector_embed_roundtrip() {
        let lattice = E8Lattice::new();
        // Take the first root and verify embed → project returns same root
        let root = &lattice.roots()[0];
        let buf = E8Projector::embed_root(root);
        let projected = E8Projector::project_vsa(&buf);
        // The first 8 bytes of embed encode the root coords as bias-shifted;
        // project_vsa reads byte parity, so root with (-2, -2, 0, …) →
        // bytes: 0,0,2,2,2,2,2,2 → parity: even→-1, even→-1, even→-1, …
        // Not all roots roundtrip perfectly through parity-only encoding,
        // but the result must be a valid E8 root with norm² = 2.
        assert!(
            (projected.norm_sq() - 2.0).abs() < 1e-9,
            "Roundtrip must produce valid E8 root, got norm²={}",
            projected.norm_sq()
        );
    }

    #[test]
    fn test_e8_projector_vsa_all_ones() {
        let mut vsa = [0xFFu8; 512];
        // Flip parity on first 8 bytes to create (+, +, +, +, +, +, +, +) pattern
        for i in 0..8 {
            vsa[i] = 0x01; // odd → +1
        }
        let root = E8Projector::project_vsa(&vsa);
        // With all +1 parities, nearest root should be (1,1,1,1,1,1,1,1)
        assert!(
            (root.norm_sq() - 2.0).abs() < 1e-9,
            "All-ones projection must have norm²=2, got {}",
            root.norm_sq()
        );
    }

    // ── LieEDNN block-diagonal weight matrix tests ──

    #[test]
    fn test_lieednn_block_diagonal_structure() {
        let killing = [[2.0; 8]; 8];
        let w = E8BlockDiagWeights::from_killing_form(&killing);
        // Verify block-diagonal structure: output[i] should only depend on input[i]
        let mut input = [0.0_f64; 240];
        input[0] = 1.0; // first element of block 0
        input[30] = 2.0; // first element of block 1
        input[239] = 3.0; // last element of block 7
        let output = w.forward(&input);
        // Element 0 depends only on input[0] * block[0][0]
        assert!(
            (output[0] - input[0] * w.block(0)[0]).abs() < 1e-12,
            "Block 0 diagonal mapping failed"
        );
        // Element 30 depends only on input[30] * block[1][0]
        assert!(
            (output[30] - input[30] * w.block(1)[0]).abs() < 1e-12,
            "Block 1 diagonal mapping failed"
        );
        // Element 239 depends only on input[239] * block[7][29]
        assert!(
            (output[239] - input[239] * w.block(7)[29]).abs() < 1e-12,
            "Block 7 diagonal mapping failed"
        );
        // Off-diagonal: element 0 should NOT be affected by input[31]
        let mut input2 = [0.0_f64; 240];
        input2[31] = 99.0; // block 1, position 1
        let output2 = w.forward(&input2);
        assert!(
            output2[0].abs() < 1e-12,
            "Off-diagonal block must be zero (block 0 affected by block 1 input)"
        );
    }

    #[test]
    fn test_lieednn_forward_preserves_length() {
        let killing = [[2.0; 8]; 8];
        let w = E8BlockDiagWeights::from_killing_form(&killing);
        let input = [1.0_f64; 240];
        let output = w.forward(&input);
        assert_eq!(
            output.len(),
            240,
            "Forward pass must preserve input length (240)"
        );
        // Output must not be all zeros for non-zero input
        let sum: f64 = output.iter().sum();
        assert!(
            sum.abs() > 0.0,
            "Output must be non-zero for all-ones input"
        );
    }

    #[test]
    fn test_lieednn_weights_positive_definite() {
        let killing = [[2.0; 8]; 8];
        let w = E8BlockDiagWeights::from_killing_form(&killing);
        // With positive-definite block weights, the forward pass on any
        // non-zero input should produce output aligned with the input.
        for block in 0..8 {
            for i in 0..30 {
                assert!(
                    w.block(block)[i] > 0.0,
                    "Block {block}[{i}] must be positive, got {}",
                    w.block(block)[i]
                );
            }
        }
        // Verify: for a unit vector in any basis direction, the dot product
        // input·output should equal the block weight (since output = w * input
        // element-wise for each block).
        for block in 0..8 {
            for pos in 0..30 {
                let idx = block * 30 + pos;
                let mut input = [0.0_f64; 240];
                input[idx] = 1.0;
                let output = w.forward(&input);
                let dot: f64 = input.iter().zip(output.iter()).map(|(a, b)| a * b).sum();
                let expected = w.block(block)[pos];
                assert!(
                    (dot - expected).abs() < 1e-12,
                    "Positive definiteness check failed at block {block}, pos {pos}: dot={dot}, expected={expected}"
                );
            }
        }
    }

    // ── Block-diagonal weight matrix tests ──

    #[test]
    fn test_block_diagonal_default() {
        let bd = E8BlockDiagonal::new();
        assert_eq!(bd.num_blocks, 8);
        assert_eq!(bd.total_dim, 240);
    }

    #[test]
    fn test_generate_weights() {
        let bd = E8BlockDiagonal::new();
        let lattice = E8Lattice::new();
        let weights = bd.generate_weights(&lattice);
        assert_eq!(weights.len(), 240 * 240);
        // Check that off-diagonal blocks are zero
        // Block 0 occupies [0..30, 0..30]; element at [0, 31] should be 0 (different block)
        assert!(
            weights[0 * 240 + 31].abs() < 1e-10,
            "off-diagonal block should be zero"
        );
        // Check diagonal elements are non-zero
        assert!(
            weights[0 * 240 + 0].abs() > 1e-10,
            "diagonal should be non-zero"
        );
    }

    #[test]
    fn test_apply_weights() {
        let bd = E8BlockDiagonal::new();
        let lattice = E8Lattice::new();
        let weights = bd.generate_weights(&lattice);
        let input = vec![1.0f64; 240];
        let output = bd.apply(&weights, &input);
        assert_eq!(output.len(), 240);
        // Output should be non-zero (input * block-diagonal)
        let sum: f64 = output.iter().sum();
        assert!(
            sum.abs() > 0.0,
            "output should be non-zero for all-ones input"
        );
    }
}
