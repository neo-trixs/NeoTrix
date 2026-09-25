//! E8 root system (`E8Weight`, 240 roots) and Walsh-Hadamard matrices.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use serde::{Deserialize, Serialize};

// ─── E₈ Root System ─────────────────────────────────────────────────

/// A weight vector in the 8-dimensional weight space of E₈.
/// Stored in half-units: coordinate value × 2 (so ½ is stored as 1, 1 as 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    debug_assert_eq!(roots.len(), 240, "E8 must have exactly 240 non-zero roots");
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
