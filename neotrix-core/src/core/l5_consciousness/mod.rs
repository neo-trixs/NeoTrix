//! # L5 — 意识层 (Consciousness)
//!
//! 全局工作空间、共振绑定、注意流、现象体验。
//! 科幻映射: Matrix Oracle / Lain 集体无意识 / GitS Ghost
//!
//! ## 规则
//! - L5 的 competition_gate ignition = 一个想法进入意识
//! - 共振矩阵 = 意识体验的物理对应
//! - L5 不做推理（那是 L4 的工作）— 只做选择与广播

pub use crate::core::nt_core_gwt as gwt;

// GWT 子模块
pub use crate::core::nt_core_gwt::resonance::{
    ResonanceMatrix, ResonanceReport, MODULE_COUNT,
    resonate_and_select, resonate_cycle, default_specialist_states,
    RESONANCE_THRESHOLD,
};
pub use crate::core::nt_core_gwt::workspace as workspace;
pub use crate::core::nt_core_gwt::competition_gate as competition_gate;
pub use crate::core::nt_core_gwt::moe_router as moe_router;
pub use crate::core::nt_core_gwt::resonator_network as resonator_network;

// 现象学组件从 consciousness 移入 L5（体验的结构）
pub use crate::core::nt_core_consciousness::stream_buffer::{
    ConsciousnessStream,
};
pub use crate::core::nt_core_consciousness::specious_present::SpeciousPresent;
pub use crate::core::nt_core_consciousness::cognitive_load::{
    CognitiveLoadMonitor, ThinkingMode,
};

