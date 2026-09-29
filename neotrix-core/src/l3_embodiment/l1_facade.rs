//! L1 Facade — L3 具身层对 L1 共享类型的 re-export 门面
//!
//! L3 具身层通过此模块访问 L1 共享类型，避免散布 `use crate::l1_action::*`。
//! 单一事实源仍在 L1，此处仅 re-export 保持跨层引用集中可审计。

// NT-IO 共享类型
pub use crate::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;
pub use crate::l1_action::nt_io::nt_io_provider::types::{
    FinishReason, LlmRequest, LlmResponse, Message, Role, Tool, Usage,
};
pub use crate::l1_action::nt_io::nt_l1_error::{L1Error, L1Result};

// NT-MEMORY 共享类型
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_write_guard::{
    record_write_evidence, scan_write_guard_evidence, WriteGuardStats, WriteGuardVerdict,
};
pub use crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase;

// NT-ACT 共享类型
pub use crate::l1_action::nt_act::nt_act_cleanup::shared::*;

// NT-MEMORY 共享类型
pub use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;
pub use crate::l4_emotion::nt_memory::nt_memory_kb::NodeType;

// NT-IO 共享类型
pub use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;

// ─── L5 / L6 跨层引用收敛点（分层门 sanctioned channel）────────────────────────
// 以下符号此前由 L3 业务文件（`nt_shield_enforcer.rs` / `nt_sandboxed_shell.rs`）
// **直引** `crate::l5_cognition::…` / `crate::l6_meta::…`，层名出现在业务代码里
// ⇒ 门记违规。走**目标层** facade 无效（路径仍含层名），必须经**本层**门面。
//
// 路径一律沿用消费方**原本就在用**的路径（原代码已能编译 ⇒ 路径可证），
// 不追原始定义处；`ToolRegistry` 等在 `nt_core_gate/` 目录内，由其 mod.rs 转出。
// ALLOW: 类型/常量直访，trait-object 不可行（同 l1_facade.rs:125-128 既有说明）。
pub use crate::l6_meta::nt_approval::{ActionType, ApprovalEngine, ApprovalMode};
pub use crate::l6_meta::nt_laws::{LawViolation, ProjectLaws};
pub use crate::l5_cognition::nt_core_gate::{ToolRegistry, ToolReversibility, ToolSpec};
