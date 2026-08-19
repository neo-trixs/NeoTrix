//! Token 预算引擎 (D3 分层修复: 已下沉至 core::nt_core_llm)
//!
//! 本文件保留为 re-export 兼容层 — 所有 `nt_io_provider::context_budget::*`
//! 路径不变, 实体定义于 `crate::core::nt_core_llm` (NT-CORE 域)。
//!
//! 文献依据 (2026 token optimization 主线):
//! - 工具输出在进入 LLM 上下文前压缩可省 60-95% token (Headroom / RTK)
//! - agent 重发上下文占推理账单 ~62% (Cockroach Labs, 2026)
//! - 选择性压缩/裁剪历史省 20-40% 且不损连贯性 (Adaline, 2026)

pub use crate::core::nt_core_llm::{
    apply_context_budget, estimate_messages_tokens, estimate_tokens, truncate_preserving,
    BudgetResult,
};
