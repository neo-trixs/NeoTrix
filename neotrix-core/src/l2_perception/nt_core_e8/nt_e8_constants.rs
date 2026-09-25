//! E8 scalar constants, lookup tables and stage/block maps.
//!
//! Pure data moved out of `nt_core_e8/mod.rs` (facade slimming).
//! Behaviour-neutral: items are re-exported from the parent module.

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

/// Trigram names in Shao Yong order: 0=坤, 1=艮, 2=坎, 3=巽, 4=震, 5=离, 6=兑, 7=乾.
pub const TRIGRAM_NAMES: [&str; 8] = [
    "坤 ☷", "艮 ☶", "坎 ☵", "巽 ☴", "震 ☳", "离 ☲", "兑 ☱", "乾 ☰",
];

/// Fuxi binary trigram: 3-bit value for each trigram.
pub const TRIGRAM_BITS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

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

// ─── Fable 5 / Mythos Structured Reasoning Pattern ─────────────────
//
// Fable 5 (Claude Fable 2.3M trace dataset) expanded to the full 9-stage
// Mythos reasoning chain discovered in the WithinUsAI/cuPjQHL8Yv4n dataset:
//
// 1. Acknowledgment  (认识问题)   — frame the scope
// 2. Problem Restatement (重述)  — reformulate in own words
// 3. Decomposition (分解)       — break into sub-problems
// 4. First-Principles (原理)    — root-cause / fundamental analysis
// 5. Self-Verification (自验)   — check own understanding
// 6. Alternative (替代)        — consider alternate approaches
// 7. Deep Dive (深度)          — detailed exploration
// 8. Synthesis (综合)          — integrate findings
// 9. Conclusion (结论)         — final answer
//
// Each stage maps to a dedicated E8 hexagram block and aligns with
// sentence-level function tags from the Qwen3-4B MATH labeling:
//   problem_setup, plan_generation, fact_retrieval, active_computation,
//   result_consolidation, uncertainty_management, final_answer_emission, self_checking

/// Mythos 9-stage reasoning chain: stage index → (block_start, label, function_tags)
pub const MYTHOS_STAGE_MAP: [(usize, &str, &[&str]); 9] = [
    (
        56,
        "Acknowledgment — scope/intent framing",
        &["problem_setup"],
    ),
    (
        48,
        "Restatement — reformulate problem",
        &["plan_generation"],
    ),
    (
        40,
        "Decomposition — break into sub-problems",
        &["plan_generation", "problem_setup"],
    ),
    (
        32,
        "First-Principles — fundamental analysis",
        &["active_computation", "fact_retrieval"],
    ),
    (
        24,
        "Self-Verification — check own understanding",
        &["self_checking"],
    ),
    (
        16,
        "Alternative — alternate approaches",
        &["uncertainty_management", "plan_generation"],
    ),
    (
        8,
        "Deep Dive — detailed exploration",
        &["active_computation", "fact_retrieval"],
    ),
    (
        0,
        "Synthesis — integrate findings",
        &["result_consolidation"],
    ),
    (4, "Conclusion — final answer", &["final_answer_emission"]),
];

/// Legacy Fable 5 4-stage block map (backward-compatible).
pub const FABLE5_BLOCK_MAP: [(usize, &str); 4] = [
    (
        56,
        "Goal — Trace/Meta: system design, meta-cognition, intent framing",
    ),
    (
        48,
        "Reason — Pattern/Generate: pattern matching, generation, analysis",
    ),
    (
        24,
        "Boundaries — Data/Architecture: constraints, architecture, vision",
    ),
    (
        40,
        "Verification — Test/Scaffold: unit test, integration, validation",
    ),
];

/// Function tag → E8 hexagram block mapping.
/// Each tag selects a 2-entry range (base..base+2) where offset 0 = analytical, offset 1 = generative.
pub const FUNCTION_TAG_BLOCKS: [(&str, usize); 8] = [
    ("problem_setup", 56),          // 56-57: Trace/Meta analytical → generative
    ("plan_generation", 48),        // 48-49: Pattern Match → Refactor
    ("fact_retrieval", 40),         // 40-41: Unit Test → Integration
    ("active_computation", 32),     // 32-33: Syntax Check → Quick Fix
    ("result_consolidation", 24),   // 24-25: Boundaries → Safety
    ("uncertainty_management", 16), // 16-17: Formal Proof → Model Check
    ("final_answer_emission", 8),   // 8-9: Deep Dive Analysis
    ("self_checking", 0),           // 0-1: Deep Debug → Guided Debug
];
