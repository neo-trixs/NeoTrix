#![recursion_limit = "256"]
//! # NeoTrix — 选择性矩阵运算架构 (Selective State-Space Agent)
//!
//! 核心公式: `Ψ(t+1) = Select(Ô, x) · Select(M, x) · Ψ(t)`
//!
//! ## 架构
//!
//! ```text
//! core/       — 纯数据模型（零外部依赖）
//! agent/      — Agent 运行时
//! cli/        — 终端 UI
//! server/     — HTTP/WebSocket 服务
//! neotrix/    — 全局模块
//! ```
//!
//! 统一版本: 0.18.0 — 推理内核 18 stages

#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), deny(warnings))] // R-P2: 生产代码 0 warning; 测试代码豁免 (测试常有良性 warning)
#![allow(dead_code)] // Large codebase: dead items tracked by auto-patrol, not compilation gate
// Float clamp: project convention uses .max().min() pattern
#![allow(clippy::manual_clamp)]
// Functions in crypto/tool APIs legitimately need many parameters
#![allow(clippy::too_many_arguments)]
// Closure type complexity is inherent in event/middleware systems
#![allow(clippy::type_complexity)]
// &mut Vec needed for API compatibility in several subsystems
#![allow(clippy::ptr_arg)]
// Range loops in ML/AI code are idiomatic and clearer than iterator adaptations
#![allow(clippy::needless_range_loop)]
// Manual strip in pattern matching is more readable than strip_prefix chains
#![allow(clippy::manual_strip)]
// from_str/default methods are intentional; implementing traits would add ceremony
#![allow(clippy::should_implement_trait)]
// is_empty not needed for all collection-like types
#![allow(clippy::len_without_is_empty)]

#[cfg(feature = "ios-bridge")]
uniffi::setup_scaffolding!();

// Re-export FFI types for uniffi scaffolding visibility
#[cfg(feature = "ios-bridge")]
pub use neotrix::ffi::{
    E8ReasoningImpl, VSAHyperCubeImpl, GWTAttentionRouterImpl,
    ConsciousnessTreeImpl, SEALPipelineImpl, KBBridgeImpl,
    SkillTreeImpl, RuneSocketingImpl, ConstellationSystemImpl,
    DualSpecializationImpl, NeoTrixHandle,
};

pub mod cli;
pub mod server;
pub mod agent;
pub mod skill_loader;
pub mod neotrix;
pub mod config;
pub mod unified_cmd;
pub mod pipeline;

// 六层架构 (Consciousness-Embodiment-Capability)
pub mod l0_substrate;
pub mod l1_action;
pub mod l2_perception;
pub mod l3_embodiment;
pub mod l4_emotion;
pub mod l5_cognition;
pub mod l6_meta;

#[macro_export]
macro_rules! make_stage {
    ($name:ident) => {
        #[derive(Default)]
        pub struct $name;
        impl $name {
            pub fn new() -> Self { Self }
        }
    };
}

pub use l5_cognition::nt_mind;
pub use l5_cognition::nt_mind::nt_mind::{
    ReasoningBrain, SelfIteratingBrain, SelfEvolver,
};

pub use l1_action::nt_act::nt_act_orchestrator::Orchestrator;

// Re-export modules used by binary targets via direct layer paths
pub use l1_action::nt_io::nt_io_mention;
pub use l2_perception::nt_world::nt_world_crawl;
pub use l2_perception::nt_world::nt_world_search;
pub use l1_action::nt_io::nt_io_session_recovery;
pub use l1_action::nt_io::nt_io_agents_md;
pub use l1_action::nt_io::nt_io_standalone;
pub use l3_embodiment::nt_shield;
pub use l3_embodiment::nt_shield::nt_shield_sentry;
pub use l1_action::nt_io::nt_io_logging;
pub use l1_action::nt_io;
pub use l1_action::nt_io::nt_io_standalone::ReasoningKernel;

// Re-export feature-gated modules for binary/example targets
#[cfg(feature = "stealth-net")]
pub use l3_embodiment::nt_shield::nt_shield_stealth_net;
pub use l3_embodiment::nt_shield::nt_shield_traffic;
