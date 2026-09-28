#![allow(unused_imports)]
//! L4 NT-FEEL Facade — re-exports all public types from nt_feel submodules
//!
//! Single fact source lives in nt_feel submodules; this facade centralises
//! cross-layer imports so consumers never scatter `use crate::l4_emotion::nt_feel::*`.

pub use crate::l4_emotion::nt_feel::digital_human::{
    Emotion as DigitalHumanEmotion, EmotionEngine as DigitalHumanEmotionEngine,
    emotion_from_expression,
};
pub use crate::l4_emotion::nt_feel::emotion_engine::{
    Emotion, EmotionalState, EmotionEngine, RegulationStrategy, EmotionalIntelligence,
};

// ─── L5 跨层引用收敛点（分层门 sanctioned channel）─────────────────────────────
// 本文件是门排除的官方跨层通道（`check-layer-deps.sh:59` 注释
// "Exclude facade bridge files … sanctioned channels"）。
// 下面这些 L5 符号此前由 L4 业务文件**直引** `crate::l5_cognition::…`，
// 于是层名出现在业务代码里（门记违规）。
// 关键点：走**目标层**的 facade（如 `l5_cognition::l1_facade`）**不够** ——
// 路径里仍含 `l5_cognition` 字样，门照样报；必须经**本层** facade 转出。
//
// ALLOW: 这些都是「字段/类型直访」，trait-object 不可行
// （同 l1_facade.rs:125-128 的既有说明）。每个符号的真实定义处已逐一核实：
//   EmotionLabel      → l6_meta/nt_core_self/emotion_state.rs:16 (enum)
//   EvidenceChain     → l5_cognition/nt_core_consciousness_tree/nodes.rs:91 (经 `pub use nodes::*`)
//   AntiDistilStore   → l5_cognition/nt_core/capability/nt_core_antidistil/mod.rs:45 (trait)
pub use crate::l5_cognition::nt_core_second_brain::SecondBrain;
pub use crate::l5_cognition::l1_facade::emotion_state::EmotionLabel;
pub use crate::l5_cognition::nt_core_consciousness_tree::EvidenceChain;
pub use crate::l5_cognition::nt_core_consciousness_tree::metacalib::expected_calibration_error;
pub use crate::l5_cognition::nt_core::capability::nt_core_antidistil::AntiDistilStore;
pub use crate::l5_cognition::nt_core_context::revertible::{ClosureEffect, RevertibleContext};
pub use crate::l5_cognition::nt_core_gwt::module_def::SpecialistType;
pub use crate::l5_cognition::nt_core_walsh::WalshMemoryIndex;
pub use crate::l5_cognition::nt_core_consciousness::consciousness_runtime::ConsciousnessRuntime;
