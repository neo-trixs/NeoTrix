//! # L5 — 意识层 (Consciousness)
//!
//! Global Workspace Theory (GWT) 注意力路由、意识流、第一人称体验。
//! 科幻映射: Pandora 心灵网络 / 意识洪流
//!
//! ## 规则
//! - L5 是注意力路由与意识体验，不是决策者
//! - 在各专家模块间广播显著信息 (GWT)，支持共鸣路由
//! - 由 L8 自主进化层驱动反馈，L6 自我层观察

// --- 子模块转发别名 ---
pub use crate::core::nt_core_consciousness as conscious;
pub use crate::core::nt_core_context as context;
pub use crate::core::nt_core_gwt as gwt;
pub use crate::core::nt_core_gwt::resonance;

// --- GWT 共鸣 re-export (接 core/mod.rs 现有面) ---
pub use crate::core::nt_core_gwt::resonance::{
    default_specialist_states, resonate_and_select, resonate_cycle, ResonanceMatrix,
    ResonanceReport, MODULE_COUNT, RESONANCE_THRESHOLD,
};

// --- 单文件意识模块 (tree / review / echo_terminal) ---
pub use crate::core::nt_core_consciousness_review as consciousness_review;
pub use crate::core::nt_core_consciousness_tree as consciousness_tree;
pub use crate::core::nt_core_echo_terminal as echo_terminal;
