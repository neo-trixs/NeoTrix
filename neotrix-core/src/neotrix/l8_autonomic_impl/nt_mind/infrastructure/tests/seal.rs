#[cfg(test)]
mod tests {
    use crate::neotrix::nt_world_model::TaskType;
    use crate::neotrix::nt_mind::{
        SelfIteratingBrain, ReasoningMemory,
    };

    #[test]
    fn test_self_iteration() {
        let mut system = SelfIteratingBrain::new_lightweight();

        let result = system.iterate(TaskType::UIDesign);
        assert!(result.iteration == 1);
        assert!(result.absorbed_count > 0);
        assert!(result.improved || !result.improved);
    }

    #[test]
    fn test_self_iterating_brain_creation() {
        let system = SelfIteratingBrain::new_lightweight();
        assert_eq!(system.iteration, 0);
        assert!(system.auto_absorb);
        assert_eq!(system.quality_threshold, 0.85);
        assert_eq!(system.reasoning_bank.stats().total_memories, 0);
    }

    #[test]
    fn test_self_iterating_brain_report() {
        let mut system = SelfIteratingBrain::new_lightweight();

        let _ = system.iterate(TaskType::UIDesign);

        let report = system.get_brain_report();
        assert!(report.iteration >= 1);
        let _ = report.total_absorbed;
    }

    #[test]
    fn test_kernel_iterate_basic() {
        let mut system = SelfIteratingBrain::new_lightweight();
        let result = system.kernel_iterate("implement user authentication");

        assert_eq!(result.iteration, 1);
        assert!(result.score_before >= 0.0);
        assert!(result.score_after >= 0.0);
    }

    #[test]
    fn test_kernel_iterate_multiple_times() {
        let mut system = SelfIteratingBrain::new_lightweight();

        let r1 = system.kernel_iterate("fix memory leak in data pipeline");
        assert_eq!(r1.iteration, 1);

        let r2 = system.kernel_iterate("add error handling for network requests");
        assert_eq!(r2.iteration, 2);

        let r3 = system.kernel_iterate("optimize query performance");
        assert_eq!(r3.iteration, 3);

        assert!(system.evaluation_history.len() == 0);
    }

    #[test]
    fn test_kernel_iterate_with_memory_storage() {
        let mut system = SelfIteratingBrain::new_lightweight();
        system.kernel_iterate("refactor database access layer");

        let stats = system.reasoning_bank.stats();
        assert_eq!(stats.total_memories, 0);
    }

    #[test]
    fn test_kernel_iterate_adaptive_lr() {
        let mut system = SelfIteratingBrain::new_lightweight();
        let _initial_lr = system.brain.learning_rate;

        let mem = ReasoningMemory::new("high reward task", TaskType::CodeGeneration, &[], 0.9);
        system.reasoning_bank.store(mem);

        system.kernel_iterate("implement feature with high reward memory");
        assert!(system.brain.learning_rate > 0.0 && system.brain.learning_rate <= 1.0);
    }
}
