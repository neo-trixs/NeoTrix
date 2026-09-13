// Re-export from neotrix-types (single source of truth for nt_core_bank types)
pub use neotrix_types::core::nt_core_bank::{PipelineConfig, PipelineState};

// Inline tests from the original definition
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_defaults() {
        let c = PipelineConfig::default();
        assert_eq!(c.l1_trigger_count, 5);
        assert_eq!(c.l2_trigger_count, 3);
        assert_eq!(c.l3_trigger_count, 5);
        assert_eq!(c.offload_threshold, 20);
    }

    #[test]
    fn test_pipeline_state_new() {
        let s = PipelineState::new();
        assert_eq!(s.total_memories, 0);
        assert_eq!(s.pending_memories, 0);
    }

    #[test]
    fn test_pipeline_state_record_memory() {
        let mut s = PipelineState::new();
        s.record_memory();
        assert_eq!(s.total_memories, 1);
        assert_eq!(s.pending_memories, 1);
    }

    #[test]
    fn test_should_trigger_l1_when_pending_reaches_threshold() {
        let config = PipelineConfig {
            l1_trigger_count: 3,
            ..Default::default()
        };
        let mut s = PipelineState::new();
        for _ in 0..2 {
            s.record_memory();
        }
        assert!(!s.should_trigger_l1(&config));
        s.record_memory();
        assert!(s.should_trigger_l1(&config));
    }

    #[test]
    fn test_should_not_trigger_l2_when_no_l1() {
        let config = PipelineConfig::default();
        let s = PipelineState::new();
        assert!(!s.should_trigger_l2(&config));
    }

    #[test]
    fn test_should_not_trigger_l3_when_no_l2() {
        let config = PipelineConfig::default();
        let s = PipelineState::new();
        assert!(!s.should_trigger_l3(&config));
    }
}
