//! L1 Facade — L2 感知层对 L1 共享类型的 re-export 门面
//!
//! L2 感知层通过此模块访问 L1 共享类型，避免散布 `use crate::l1_action::*`。
//! 单一事实源仍在 L1，此处仅 re-export 保持跨层引用集中可审计。

pub use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
pub use crate::l1_action::nt_memory::nt_memory_kb::NodeType;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_discovery_github_topics::DiscoveryPipelineConfig;

// HTTP 集中门面 — nt_http 类型与函数
// NOTE: download_to_file, shared_blocking_client, run_blocking 在 nt_http 中为 pub(crate)
//       因此这里也必须用 pub(crate) re-export，不能 pub use。
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::DownloadOptions;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::download_to_file;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::shared_blocking_client;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_http::run_blocking;

// HTTP 工厂门面 — proxy_from_env
pub use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;

// 共享出口类型门面 — EgressRule/EgressPolicy (避免 L2→L3 向上依赖)
pub use crate::l1_action::nt_io::nt_io_provider::common::egress_types::{
    SandboxEgressRule as EgressRule,
    SandboxEgressPolicy as EgressPolicy,
};
