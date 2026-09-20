//! # 统一意识模块 (Unified Consciousness Module)
//!
//! 合并原 8 个碎片目录为单一入口:
//! - `nt_core_consciousness` → `features` (觉醒/认知负荷/内省等)
//! - `consciousness_core` → `core` (持久化意识核心/任务环)
//! - `nt_core_consciousness_tree` → `tree` (意识树结构)
//! - `nt_core/nt_consciousness_core` → `engine` (完整引擎, 62文件)
//! - `nt_core_consciousness_crystal` → `crystal` (水晶变体)
//! - `nt_mind::consciousness` → `mind` (心智意识)
//! - `nt_game::consciousness` → `game` (游戏化意识)
//!
//! 消费方应使用 `crate::l5_cognition::nt_consciousness::*`
//! 旧路径仍可通过 re-export 到达 (向后兼容).

// ── Features: 觉醒/认知负荷/内省/意志等纯意识特征 ──
pub use crate::l5_cognition::nt_consciousness::features as features;

// ── Core: 持久化意识核心 + 任务环 ──
pub use crate::l5_cognition::nt_consciousness::core as core;

// ── Tree: 意识树结构 + 生命周期 ──
pub use crate::l5_cognition::nt_consciousness::tree as tree;

// ── Engine: 完整意识引擎 (62文件, pipeline/state_machine/evolution) ──
pub use crate::l5_cognition::nt_core::nt_consciousness_core as engine;

// ── Re-export 最常用类型 ──
pub use features::{
    AwakeningReport, ConsciousnessAwakening,
    BubbleWall, TokenBill,
    CognitiveLoadMonitor, ThinkingMode,
    FirstPersonRef,
    CritiqueResult, InnerCritic,
    SpeciousPresent,
    ConsciousnessStream,
    ActionCandidate, VolitionEngine,
    VsaOrigin, VsaSelfCategory, VsaTagged, VsaWorldCategory,
};
