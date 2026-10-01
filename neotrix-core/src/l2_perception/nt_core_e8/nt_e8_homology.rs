//! E8 identity verifiers and the `E8HexagramHomology` aggregate model.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use serde::Deserialize;

use super::{
    E8_DIM, FERMIONS_PER_GENERATION, FERMION_GENERATIONS, HEXAGRAM_COUNT,
    REMAINING_E8_GENERATORS, TOTAL_SM_FERMIONS, e8_root_norm_counts,
    verify_hadamard_orthogonality, verify_total_fermions,
};

// ─── Core Verifiers ──

// ─── 单点真身收敛（2026-09-30）────────────────────────────────────────
// 下面 8 个身份校验函数此前在 `neotrix-types/src/core/nt_core_e8.rs` 与本文件
// 各有一份**逐字相同**的实现（`nt_diverge.py` 实测：owner 同为 `(free)`、归一化后
// 完全相同）。现以 types 侧为**唯一实现**，本模块直接引用。
//
// 收敛前已验证**行为中性**（关键一步，不能靠「看起来一样」）：
// 函数体依赖的 16 个常量（`E8_DIM` `LO_SHU_CONSTANT` `TOTAL_LINES` …）在
// types / `nt_e8_constants.rs` / `nt_world_e8.rs` **三方逐个比对全一致**
// （`TOTAL_LINES` 连表达式写法 `64 * 6` 都相同）⇒ 换实现不会换行为。
//
// 方向合法性：`neotrix-core` 依赖 `neotrix-types`（`Cargo.toml:97`）
// ⇒「高层引用低层」唯一合法。必要原因：两份逐字相同的实现**只会各自漂移**
//（本会话已实测 `now_ts` 13 份里 2 份 panic、`truncate` 8 份里 2 份字节切 panic）。
pub use neotrix_types::core::nt_core_e8::{
    verify_all_identities, verify_dayan_identity, verify_e8_dimension, verify_he_tu_sum,
    verify_lo_shu, verify_seven_squared, verify_three_generations, verify_total_lines,
};

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
