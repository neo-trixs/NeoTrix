//! L2 Facade — L5 对 L2 感知层类型的集中 re-export 门面
//!
//! L5 认知层通过此模块访问 L2 感知层提供的具体类型,
//! 避免散布 `use crate::l2_perception::*` 造成分散的向下依赖。
//! 单一事实源仍在 L2, 此处仅 re-export 保持跨层引用集中可审计。
//!
//! ⚠️ 向下依赖: L5 → L2, 已通过 facade 集中化。
//! 对于仅需接口而非实现的场景, 优先使用 `crate::l2_perception::traits` 中的 trait 抽象。

// ── nt_world_model_v2 ──
pub use crate::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;

// ── nt_world_search ──
pub use crate::l2_perception::nt_world::nt_world_search::{SearchResult, UnifiedSearch};

// ── nt_world_novel ──
pub use crate::l2_perception::nt_world::nt_world_novel::{
    drain_novel_queue, classify_unanalyzed_books, QidianIngestReport,
};
