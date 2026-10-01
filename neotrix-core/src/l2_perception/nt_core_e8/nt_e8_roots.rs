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

// ─── 单点真身收敛（2026-09-30）────────────────────────────────────────
// 以下 4 个函数此前在 `neotrix-types/src/core/nt_core_e8.rs` 与本文件
// 各有一份**逐字相同**的实现（`nt_diverge.py` 实测：owner 同为 `(free)`、
// 归一化后完全相同）。现以 types 侧为**唯一实现**，本模块直接引用。
//
// 方向合法性：`neotrix-core` 依赖 `neotrix-types`（`Cargo.toml:97`），
// 所以「高层引用低层」是唯一合法方向（types 不能反向依赖 core）。
// 为什么必须做：两份逐字相同的实现**只会各自漂移** —— 本会话已实测同类漂移
// （`now_ts` 13 份副本里 2 份 panic、`truncate` 8 份里 2 份字节切 panic）。
//
// ⚠️ 本模块下方的 `mod tests` 保留：它验证的是**行为**，
// 实现位置改变不应让这些测试消失（否则就成了「删测试让门变绿」）。
pub use neotrix_types::core::nt_core_e8::{
    e8_root_norm_counts, hadamard_matrix, hexagram_hadamard, verify_hadamard_orthogonality,
};
