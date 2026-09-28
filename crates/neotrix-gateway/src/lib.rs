//! NeoTrix Gateway Crate
//!
//! L5 Gateway systems — model routing, skill registry, search,
//! BYOA (Bring Your Own Agent), capability system, Hive coordination,
//! gate control, context management, and awareness core bridging.

#![forbid(unsafe_code)]

pub mod model_gateway;
pub mod model_router;
pub mod skill_registry;

pub mod hive;
pub mod mind_modules;
pub mod context_mgmt;
pub mod gate;

// 2026-09-28: `decision-engine` feature 与 `pub use neotrix_decision_engine as
// decision_engine` 一并移除 —— 该 crate 已归档至
// ~/Downloads/Neo/neotrix-archive/crates/neotrix-decision-engine/。
// 移除前实测：全仓 0 处启用该 feature，此 re-export 别名从未被解引用
// （`OWNERSHIP.md:59` 与 `DIR-AUDIT-2026-09-27.md:221` 均已标 ⛔ 死）。
// 其中唯一有价值的确定性采样器（NtSampler）已萃取至
// `neotrix-core/src/l0_substrate/nt_sampler.rs`。
