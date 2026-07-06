//! NeoTrix 统一错误类型 — 定义在 core/nt_core_error（防 L7/L5/L4 反向依赖）

pub use crate::core::nt_core_error::{NeoTrixError, NeoTrixResult, from_string_result};
