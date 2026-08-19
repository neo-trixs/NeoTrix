//! Slow iterative SEAL loop integration tests (C2).
//!
//! 拆分自 `tests/seal.rs`: 这些用例调用 `run_seal_loop` 跑完整 pipeline,
//! 单轮 30~150s, 拖垮默认 `cargo test`。默认 `#[ignore]`, 需要时以
//! `cargo test -p neotrix --lib -- --ignored seal_slow` 单独跑。

#[cfg(test)]
mod tests {
    use crate::neotrix::nt_mind::{SelfIteratingBrain, ReasoningMemory};
    use crate::neotrix::nt_world_model::TaskType;

    #[test]
    #[ignore = "slow iterative SEAL loop integration (30~150s); run with --ignored"]
    fn test_seal_loop_basic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut system = SelfIteratingBrain::new_lightweight();

        let reward = system.run_seal_loop("design a UI component", None, None);
        assert!(reward.is_ok());
        let r = reward.expect("reward should be ok in test");
        assert!(r >= 0.0 && r <= 1.0);

        let stats = system.reasoning_bank.stats();
        assert_eq!(stats.total_memories, 0);
    }

    #[test]
    #[ignore = "slow iterative SEAL loop integration (30~150s); run with --ignored"]
    fn test_seal_loop_with_embedding() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut system = SelfIteratingBrain::new_lightweight();

        let embedding = vec![0.5, 0.3, 0.8, 0.2, 0.6];
        let embedding_clone = embedding.clone();
        let reward = system.run_seal_loop("implement code generation", Some(embedding), None);
        assert!(reward.is_ok());
        let r = reward.expect("reward should be ok in test");
        assert!(r.is_finite(), "reward should be finite, got {}", r);

        let memories = system.reasoning_bank.retrieve_relevant_by_embedding(&embedding_clone, None, 1);
        assert!(memories.is_empty());
    }

    #[test]
    #[ignore = "slow iterative SEAL loop integration (30~150s); run with --ignored"]
    fn test_seal_loop_multiple_iterations() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut system = SelfIteratingBrain::new_lightweight();

        let tasks = ["design UI", "analyze code", "review nt_shield"];
        for task in &tasks {
            let _ = system.run_seal_loop(task, None, None);
        }

        let stats = system.reasoning_bank.stats();
        assert!(
            stats.total_memories > 0,
            "3-task SEAL 循环后记忆应已落 bank (ReasoningBankStorageStage freq=2), got {}",
            stats.total_memories
        );
    }

    #[test]
    #[ignore = "slow iterative SEAL loop integration (30~150s); run with --ignored"]
    fn test_seal_loop_reward_threshold() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut system = SelfIteratingBrain::new_lightweight();
        system.quality_threshold = 0.5;

        let reward = system.run_seal_loop("test task for reward", None, None);
        assert!(reward.is_ok());
        let r = reward.expect("reward should be ok in test");

        assert!(r.is_finite());

        assert!(system.quality_threshold > 0.0);
    }

    #[test]
    #[ignore = "slow iterative SEAL loop integration (30~150s); run with --ignored"]
    fn test_seal_loop_reward_bounded() {
        // 契约: SEAL reward 是归一化信号, 返回必须 ∈ [0,1]。
        // RDR 乘法 (ratio=depth/total_steps 可>1) + 4 个加法 bonus
        // (curiosity/goal/depth/tool) 可能把 reward 推出上界, 返回点必须 clamp。
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut system = SelfIteratingBrain::new_lightweight();

        // 多轮迭代让 state_trajectory 增长, 提高 depth/total_steps ratio
        let tasks = [
            "design a UI component",
            "implement code generation",
            "analyze performance bottleneck",
            "refactor data pipeline",
            "optimize query path",
        ];
        for task in &tasks {
            let reward = system.run_seal_loop(task, None, None);
            assert!(reward.is_ok());
            let r = reward.expect("reward should be ok");
            assert!(
                r >= 0.0 && r <= 1.0,
                "SEAL reward must stay in [0,1] (normalized signal), got {r}"
            );
        }
    }

    #[test]
    #[ignore = "slow iterative SEAL loop integration (30~150s); run with --ignored"]
    fn test_seal_with_memory() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut system = SelfIteratingBrain::new_lightweight();

        let _ = system.run_seal_loop("design a UI component with accessibility", None, None);

        let stats_before = system.reasoning_bank.stats();

        let _ = system.run_seal_loop("improve UI accessibility", None, None);

        let stats_after = system.reasoning_bank.stats();
        assert!(
            stats_after.total_memories >= stats_before.total_memories,
            "第二次循环后记忆只增不减, before={} after={}",
            stats_before.total_memories,
            stats_after.total_memories
        );

        // 显式写入一条记忆验证检索路径 (ReasoningBankStorageStage 按 freq=2 触发,
        // 单次循环可能 0 次迭代命中, 不依赖时序)。
        system
            .reasoning_bank
            .store(ReasoningMemory::new("improve UI accessibility", TaskType::UIDesign, &[], 0.8));

        let memories = system.reasoning_bank.retrieve_relevant("accessibility", None, 5);
        assert!(
            !memories.is_empty(),
            "写入的记忆应能被语义检索命中文案 'accessibility'"
        );
    }
}
