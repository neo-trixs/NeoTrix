//! KB Facade — L5 对 L1 NT-MEMORY KB 共享类型的 re-export 门面
//!
//! L5 认知层通过此模块访问 KB 共享类型，避免散布 `use crate::l1_action::nt_memory::*`。
//! 单一事实源仍在 L1，此处仅 re-export 保持跨层引用集中可审计。

pub use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::{
    SearchResult, ProceduralMemoryRecord, ConversationRecord,
};
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_unify::{
    skill_upsert, skill_list_all, skill_content_hash, SkillRecord,
};
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_coeffect::*;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_write_guard::*;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::{
    extract_html_content, is_safe_fetch_url, enqueue_seed_urls,
};
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
    claim_next_crawl_url, mark_crawl_complete, count_nodes_by_domain,
};
pub use crate::l1_action::nt_memory::nt_memory_kb::bm25;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_confidence::RetrievalStrategy;
