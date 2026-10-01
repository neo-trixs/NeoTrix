//! # 与 `neotrix-types` 侧同名模块的关系（2026-09-30 加注）
//!
//! 低层契约库：`neotrix-types/src/core/nt_core_bank`（≈156 KB）。
//! 本模块是它的**上层实现**（`neotrix-core` 依赖 `neotrix-types`）。
//!
//! ⛔ **两者不是镜像冗余**：99 个同名函数里 26 个实现不同，
//! 本侧独有 35 个函数（对侧有、低层没有）、低层独有 51 个。
//! ⇒ **不要合并，也不要删除任何一侧**。完整取证：`docs/architecture/MIRROR-BANK-2026-09-30.md`。
//!
//! **规则**：新能力加到这里（实现层）；只有**跨后端共用**的契约/纯函数才下沉到 types。
//! 两侧同名函数若有实现差异，属**待定性的架构债**，改动前先配行为对位（`nt_parity_ref`）。

pub mod bank;
mod iteration;
mod l1;
mod mem;
mod offload;
mod pipeline;
mod stats;
mod tier;

pub use bank::ReasoningBank;
pub use iteration::MemoryIterationResult;
pub use iteration::{Bm25Document, Bm25Index};
pub use iteration::rrf_fuse;
pub use l1::{ExtractionPrompt, L1Memory, Persona, SceneBlock};
pub use mem::{ReasoningMemory, T3ViewType, T3Views, TemporalContext};
pub use offload::OffloadManager;
pub use pipeline::{PipelineConfig, PipelineState};
pub use stats::{MemoryDetailedStats, ReasoningBankStats};
pub use tier::{MemoryLifecycle, MemoryTier};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_module_re_exports_tier() {
        let tier = MemoryTier::Working;
        assert_eq!(tier.as_str(), "working");
    }

    #[test]
    fn test_memory_module_re_exports_lifecycle() {
        let lc = MemoryLifecycle::new(0.7);
        assert!((lc.importance - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_memory_module_memory_lifecycle_without_ttl() {
        let lc = MemoryLifecycle::new(0.5);
        assert!(!lc.is_expired());
    }

    #[test]
    fn test_memory_module_t3_view_type_all() {
        let all = T3ViewType::all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_memory_module_t3_view_type_default() {
        let views = T3Views::new();
        assert!(views.struct_view.is_none());
        assert!(views.semantic_view.is_none());
        assert!(views.reflect_view.is_none());
    }
}
