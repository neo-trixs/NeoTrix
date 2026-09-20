//! KB 存储原语 — 单一事实源在 `l0_substrate::nt_core_kb_primitives`。
//!
//! 此处 re-export 保持 `crate::l6_meta::nt_core_kb_primitives::*` 路径不变,
//! 消除低层 (L1-L5) → L6 反向依赖。

pub use crate::l0_substrate::nt_core_kb_primitives::*;
