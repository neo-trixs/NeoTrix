//! L1 Facade — L2 感知层对 L1 共享类型的 re-export 门面
//!
//! L2 感知层通过此模块访问 L1 共享类型，避免散布 `use crate::l1_action::*`。
//! 单一事实源仍在 L1，此处仅 re-export 保持跨层引用集中可审计。

pub use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
pub use crate::l1_action::nt_memory::nt_memory_kb::NodeType;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::DownloadOptions;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_discovery_github_topics::DiscoveryPipelineConfig;
