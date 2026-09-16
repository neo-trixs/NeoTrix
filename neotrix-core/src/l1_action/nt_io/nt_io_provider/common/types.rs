//! LLM Provider 核心类型定义 (D3 分层修复: 类型已下沉至 core::nt_core_llm)
//!
//! 本文件保留为 re-export 兼容层 — 所有 `nt_io_provider::types::*` 路径不变,
//! 类型实体定义于 `crate::l1_action::nt_io::nt_io_llm` (统一 LLM 接口层)。
//!
//! 更新说明 (2026-07-01):
//! - `temperature` 改为 `Option<f32>` — 部分新模型 (如 Claude Sonnet 5) 不再支持采样参数
//! - 新增 `thinking_budget` — 支持模型自适应思考 (extended thinking)
//! - 新增 `provider_params` — 额外 provider 专用参数映射
//!
//! 2026-07-04: 新增 ProviderCategory (自我/客体分离)
//! 2026-09-16: 类型统一到 neotrix-types::llm_types

pub use crate::l1_action::nt_io::nt_io_llm::*;

/// Backward-compatible: LlmProvider trait (old interface, defined in core::nt_core_llm)
pub use crate::core::nt_core_llm::LlmProvider;

/// Backward-compatible: re-export ToolCallFunction from neotrix-types (single source of truth)
pub use neotrix_types::llm_types::ToolCallFunction;
