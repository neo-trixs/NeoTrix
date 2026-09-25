//! nt_memory_resource_ingest — 资源摄取门面：子模块重导出，行为零变更。
//! 拆分：nt_resource_types（描述子 Builder）/ nt_resource_ingester（ingest/relate+批量吸收）
//!      / nt_resource_cortex（Cortex-Brain 编目：upsert/register+卷路径）
//!      / nt_resource_corpus（语料迁移/续传/回收/剪枝）+ tests。

pub mod nt_resource_corpus;
pub mod nt_resource_cortex;
pub mod nt_resource_ingester;
pub mod nt_resource_types;

pub use nt_resource_cortex::{corpus_archive_path, register_cortex_brain};
pub use nt_resource_corpus::{migrate_cortex_corpus, prune_cortex_orphans, reclaim_nt_target_tmp};
pub use nt_resource_ingester::{ingest_session_resources, ResourceIngester};
pub use nt_resource_types::{ResourceDescriptor, ResourceIngestResult, ResourceSource};

#[cfg(test)]
mod tests;
