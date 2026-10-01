//! Spin(11,3) 64-fermion decomposition: `FermionState` and generators.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use serde::Deserialize;

// `TOTAL_SM_FERMIONS` 的导入已删（2026-09-30）：它只被本模块那份已收敛走的
// `verify_total_fermions` 函数体使用；真身实现自带该常量（types 侧）。
// 由编译器 unused 警告抓出（该 crate 开了 `deny(warnings)`，不会静默通过）。

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

// 单点真身收敛（2026-09-30）：此前本文件与 `neotrix-types/core/nt_core_e8.rs`
// 各有一份**逐字相同**的 `verify_total_fermions`（`nt_diverge.py` 实测 owner 同为
// `(free)`、归一化后全等）⇒ 收敛为引用低层唯一实现。
// 判定依据：返回值是 `bool`，**跨 crate 同一类型** ⇒ 可安全收敛。
//（同文件的 `all_sm_fermions` 返回 `Vec<FermionState>`，而 `FermionState` 在两
//  crate 是各自定义的同名类型 ⇒ 那一个**不可**收敛，保留本地实现。）
// 必要性：逐字相同的副本只会各自漂移（本会话已实测 `now_ts` 13 份里 2 份 panic）。
pub use neotrix_types::core::nt_core_e8::verify_total_fermions;
