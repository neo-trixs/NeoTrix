//! # L7 — Capability (能力层)
//!
//! 能力注册、调度、成熟度进化、星脉通信协议。

pub use crate::core::l7_capability::*;

// 子目录模块
pub mod reasoning;
pub mod visual;
pub mod info_theory;
pub mod safety;
pub mod cuda;
pub mod other;

// 意识核心 (从 neotrix/ 迁移)
pub mod nt_consciousness_core;

// 保留的独立模块
pub mod context_assembly;
pub mod nt_core_intra_reflection;
pub mod nt_core_parallel;
pub mod seal;
pub mod persona_routing;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod awareness_monitor;

pub use awareness_monitor::SelfAwarenessMonitor;
