//! L1 Facade — L3 具身层对 L1 共享类型的 re-export 门面
//!
//! L3 具身层通过此模块访问 L1 共享类型，避免散布 `use crate::l1_action::*`。
//! 单一事实源仍在 L1，此处仅 re-export 保持跨层引用集中可审计。

// NT-IO 共享类型
pub use crate::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;
pub use crate::l1_action::nt_io::nt_io_provider::types::{
    FinishReason, LlmRequest, Message, Role, Tool,
};
pub use crate::l1_action::nt_io::nt_l1_error::{L1Error, L1Result};

// NT-MEMORY 共享类型
pub use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::{
    record_write_evidence, scan_write_guard_evidence, WriteGuardStats, WriteGuardVerdict,
};

// NT-ACT 共享类型
pub use crate::l1_action::nt_act::nt_act_cleanup::shared::*;

// NT-MEMORY 共享类型
pub use crate::l1_action::nt_memory::nt_memory_kb::NodeType;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;

// NT-IO 共享类型
pub use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;
