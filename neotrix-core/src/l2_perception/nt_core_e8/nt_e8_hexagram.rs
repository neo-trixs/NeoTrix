//! Hexagram system: `Hexagram` type, Shao Yong / King Wen sequences,
//! 8x8 trigram matrix and SU(3) trigram mapping.
//!
//! Moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).


// ─── Hexagram System ─────────────────────────────────────────────────

/// `Hexagram` 收敛到低层唯一实现（2026-10-02）。
///
/// 此前本文件与 `neotrix-types/core/nt_core_e8.rs` **各定义一份同名类型**
/// —— 这正是 `62cb175f` 记下的阻塞点：
/// > 同文件的 `king_wen_sequence` / `shao_yong_sequence` 返回 `Vec<Hexagram>`，
/// > 而 `Hexagram` 在两 crate 各自定义 ⇒ 那两个**不可**收敛。
/// > ⇒ **阻塞根因是类型，不是函数**。本笔收敛类型，两个函数随之解锁。
///
/// 收敛前核实（沿用 `ReasoningHexagram` 那次的纪律：契约才是判据）：
/// · 字段逐字相同（`pub bits: u8`），方法集相同（6 个，无缺失）；
/// · 唯一语义差异在 `new()`：types=`assert!(bits<64)` panic，
///   core=`bits & 0x3F` 静默掩码 ⇒ 照字面去重会把掩码变成 panic；
/// · panic **不可达**：core 侧 `Hexagram::new` 只有 1 个生产调用方
///   `nt_memory_e8_agent.rs:56`，其值来自 `E8Phase::bits()` ——
///   那是**编译期字面量 match**（0x3F/23/5/7/53/20/25…）⇒ 恒 < 64。
/// ⇒ 取 types 的**带检查**版：静默掩码会把越界值变成「看似合理的错值」。
///
/// ⛔ 连带删除 `use super::WEN_SEQUENCE;` —— 它仅被本处删除的
/// `wen_index()` 与 `king_wen_sequence()` 使用，删后成为未用项。
/// 常量本身在 `nt_e8_constants.rs` 仍有他用，不动。
pub use neotrix_types::core::nt_core_e8::Hexagram;

/// 64 卦先天图（Shao Yong 二进制序）—— 转出低层唯一实现。
pub use neotrix_types::core::nt_core_e8::shao_yong_sequence;

/// 64 卦周易序（King Wen）—— 转出低层唯一实现。
pub use neotrix_types::core::nt_core_e8::king_wen_sequence;

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
