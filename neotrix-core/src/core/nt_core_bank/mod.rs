mod bank;
mod iteration;
mod l1;
pub mod mem;
mod offload;
mod pipeline;
mod stats;
mod tier;

pub use bank::ReasoningBank;
pub use iteration::MemoryIterationResult;
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
