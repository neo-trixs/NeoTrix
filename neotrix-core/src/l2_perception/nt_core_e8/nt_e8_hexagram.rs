//! Hexagram system: `Hexagram` type, Shao Yong / King Wen sequences,
//! 8x8 trigram matrix and SU(3) trigram mapping.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).

use serde::Deserialize;

use super::WEN_SEQUENCE;

// ─── Hexagram System ─────────────────────────────────────────────────

/// A single hexagram: 6-bit binary state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, Deserialize)]
pub struct Hexagram {
    /// 6 bits, MSB = top line (yang=1, yin=0), per Shao Yong ordering.
    pub bits: u8,
}

impl Hexagram {
    pub fn new(bits: u8) -> Self {
        Self { bits: bits & 0x3F }
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

// ─── SU(3) Subgroup ─────────────────────────────────────────────────

/// The 8 generators of SU(3) = 8 trigrams.
/// Represented as Gell-Mann matrices (symbolic structure constants).
/// SU(3) Gell-Mann 生成元名称（8 个）。
///
/// 单点真身收敛（2026-09-30）：此前本文件与 `neotrix-types/core/nt_core_e8.rs`
/// 各有一份**逐字相同**的实现 ⇒ 收敛为引用低层唯一实现。
/// 判定依据：返回值 `Vec<&'static str>` 是**跨 crate 同一类型** ⇒ 可安全收敛。
/// ⚠️ 同文件的 `king_wen_sequence` / `shao_yong_sequence` 返回 `Vec<Hexagram>`，
/// 而 `Hexagram` 在两 crate 各自定义 ⇒ 那两个**不可**收敛，保留本地实现。
pub use neotrix_types::core::nt_core_e8::su3_generators;
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
        _ => (0, 0),
    }
}
