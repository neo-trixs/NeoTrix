//! KB schema 初始化 — 实现已下沉至 core `nt_core_kb_primitives` (D3 架构倒置)。
//! 此处 re-export 保持 `nt_memory_schema::initialize` 调用方路径不变。
//! 单一事实源: `crate::core::nt_core_kb_primitives::schema_initialize`。

pub use crate::core::nt_core_kb_primitives::{SCHEMA_VERSION, schema_initialize as initialize};