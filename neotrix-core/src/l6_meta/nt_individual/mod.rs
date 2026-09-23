//! nt_individual — 个体（一等公民）：身份 × 宪法 × 记忆.
//!
//! 分身变个体的三件套：
//! - [`constitution`] 行为宪法（红线不可绕过）
//! - [`judgment`] 独立裁决（Allow / Deny / NeedHuman）
//! - [`memory`] 记忆连续性（KB experience 指针，不存正文）
//! - [`individual`] 组装体 + 进化入口
//! - [`presets`] 首个个体：WSD 外贸询价助手

#![forbid(unsafe_code)]

pub mod constitution;
pub mod individual;
pub mod judgment;
pub mod memory;
pub mod presets;
pub mod registry;

pub use constitution::{Constitution, Rule, ViolationAction};
pub use individual::{IndividualError, NtIndividual};
pub use judgment::{judge, ActionKind, Judgment, JudgmentCtx};
pub use memory::MemoryLink;
pub use registry::{IndividualRegistry, RegistryError};
