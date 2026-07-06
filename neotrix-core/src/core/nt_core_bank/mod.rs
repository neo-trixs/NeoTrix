mod tier;
mod mem;
mod stats;
mod iteration;
mod pipeline;
mod offload;
mod l1;
mod bank;

pub use tier::{MemoryTier, MemoryLifecycle};
pub use mem::{ReasoningMemory, T3ViewType, T3Views, TemporalContext};
pub use stats::{ReasoningBankStats, MemoryDetailedStats};
pub use bank::ReasoningBank;
pub use iteration::MemoryIterationResult;
pub use pipeline::{PipelineConfig, PipelineState};
pub use offload::OffloadManager;
pub use l1::{L1Memory, SceneBlock, Persona, ExtractionPrompt};

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
