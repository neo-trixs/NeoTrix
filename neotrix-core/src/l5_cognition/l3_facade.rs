//! L3 Facade — L5 认知层对 L3 共享类型的 re-export 门面
//!
//! L5 认知层通过此模块访问 L3 共享类型，避免散布 `use crate::l3_embodiment::*`。
//! 单一事实源仍在 L3，此处仅 re-export 保持跨层引用集中可审计。

pub use crate::l3_embodiment::nt_shield::nt_shield_audit::{
    write_guard_check_result, CheckStatus,
};
