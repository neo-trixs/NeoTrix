//! # L1 — Shield 安全基座 (核心数据模型)
//!
//! ## 规则
//! - L1 是唯一可以并行执行多个能力的层
//! - L1 的执行必须通过模式链验证
//! - L1 不产生推理 — 只执行
//!
//! 实现在 `neotrix/l1_body_impl/nt_shield_*` 中。

pub mod immunity;

pub use immunity::{
    AntiPattern, FailureClass, Vaccine,
    ImmuneMemory, ImmuneSystem, ImmuneStats,
    AdversarialReview,
};

/// Shield guard: convenience function to check an action against the immune system.
pub fn shield_check_action<'a>(immune: &'a ImmuneSystem, action_desc: &str) -> Option<&'a AntiPattern> {
    immune.check_action(action_desc)
}
