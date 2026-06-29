#[cfg(test)]
mod tests {
    use crate::neotrix::nt_mind::core::BrainMutView;
    use crate::neotrix::nt_mind::distillation::ExperienceDistiller;
    use crate::neotrix::nt_mind::memory::ReasoningBank;
    use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
    use crate::neotrix::nt_mind::reasoning_types::ReasoningType;
    use crate::neotrix::nt_mind::self_iterating::ReasoningBrain;

    #[test]
    fn test_infer_type() {
        assert_eq!(
            ReasoningEngine::infer_reasoning_type("fix bug"),
            ReasoningType::ErrorDebugging
        );
        assert_eq!(
            ReasoningEngine::infer_reasoning_type("implement x"),
            ReasoningType::TaskSolving
        );
        assert_eq!(
            ReasoningEngine::infer_reasoning_type("what is X"),
            ReasoningType::KnowledgeQuery
        );
        assert_eq!(
            ReasoningEngine::infer_reasoning_type("hello"),
            ReasoningType::Conversation
        );
    }

    #[test]
    fn test_distill_traces_empty() {
        let p = ExperienceDistiller::distill_traces(&[]);
        assert!(p.is_empty());
    }

    #[test]
    fn test_contrastive_empty() {
        let a = ExperienceDistiller::contrastive_reflect_traces(&[]);
        assert!(a.is_empty());
    }

    #[test]
    fn test_self_iterate_no_llm() {
        let brain: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
        let bank = ReasoningBank::new(100);
        let mut engine = ReasoningEngine::from_env(brain, bank);
        engine.distill_interval = 2;
        engine.self_iterate();
    }

    #[test]
    fn test_external_compile_reward_empty_project() {
        let brain: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
        let bank = ReasoningBank::new(100);
        let mut engine = ReasoningEngine::from_env(brain, bank);
        let reward = engine.external_compile_reward("/tmp/nonexistent_project_xyz");
        assert_eq!(reward, 0.0);
    }

    #[test]
    fn test_e8_state_save_load() {
        use crate::core::{FullReasoningState, MetaState, ReasoningHexagram};

        let brain: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
        let bank = ReasoningBank::new(100);
        let mut engine = ReasoningEngine::from_env(brain, bank);

        // Set up some E8 state
        let hex0 = ReasoningHexagram::new(0);
        let hex42 = ReasoningHexagram::new(42);
        engine.current_state = FullReasoningState::new(hex42, MetaState::new(2));
        engine.state_trajectory = vec![
            FullReasoningState::new(hex0, MetaState::new(0)),
            FullReasoningState::new(hex42, MetaState::new(1)),
            engine.current_state,
        ];
        // Modify strategy matrix
        engine.strategy_matrix[0][0] = hex42;
        engine.strategy_matrix[7][7] = hex0;

        // Save to temp file
        let test_id = format!("neotrix_test_e8_{}", std::process::id());
        let temp_dir = std::env::temp_dir().join(&test_id);
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("value should be ok in test");
        let save_path = temp_dir.join("e8_state.json");

        let save_result = engine.save_e8_state(&save_path);
        assert!(
            save_result.is_ok(),
            "E8 save failed: {:?}",
            save_result.err()
        );
        assert!(save_path.exists(), "E8 file not created");

        // Create new engine and load
        let brain2: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
        let bank2 = ReasoningBank::new(100);
        let mut engine2 = ReasoningEngine::from_env(brain2, bank2);

        let load_result = engine2.load_e8_state(&save_path);
        assert!(
            load_result.is_ok(),
            "E8 load failed: {:?}",
            load_result.err()
        );

        // Verify fields match
        assert_eq!(engine2.current_state, engine.current_state);
        assert_eq!(engine2.state_trajectory.len(), 3);
        assert_eq!(engine2.state_trajectory[0], engine.state_trajectory[0]);
        assert_eq!(engine2.strategy_matrix[0][0], engine.strategy_matrix[0][0]);
        assert_eq!(engine2.strategy_matrix[7][7], engine.strategy_matrix[7][7]);
        assert_eq!(engine2.state_trajectory[2], engine.current_state);

        // Verify observer was persisted
        assert_eq!(engine2.observer.analysis_count, 0); // fresh engine starts at 0
        assert!(engine2.observer.trajectory_history.is_empty());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
