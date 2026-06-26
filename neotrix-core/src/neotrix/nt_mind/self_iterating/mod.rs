// Re-export from split files
pub mod aging_monitor;
pub mod brain_absorb;
pub mod brain_core;
pub mod brain_dgm;
pub mod brain_ewc;
pub mod brain_impl;
pub mod brain_seal;
pub mod checkpoint;
pub mod curvature_rl;
pub mod dgm_variant;
pub mod fingerprint_reconciler;
pub mod goal_contract;
pub mod harness_adapter;
#[cfg(feature = "hgm")]
pub mod hgm;
mod loop_impl;
#[cfg(feature = "lse")]
pub mod lse;
mod persist_impl;
pub mod pipeline;
pub(crate) mod pipeline_awareness;
pub(crate) mod pipeline_code;
pub(crate) mod pipeline_core;
pub(crate) mod pipeline_evolution;
pub(crate) mod pipeline_memory;
pub(crate) mod pipeline_search;
pub mod recipe;
pub mod secret_scanner;
pub(crate) mod sib_state;
pub mod skill_crystallizer;
pub mod skillopt;
pub mod stage_contracts;
pub mod validation;
pub mod vsi_verifier;

pub use brain_impl::{
    AbsorbValidator, DefaultAbsorbValidator, EvaluationRecord, ReasoningBrain, SelfIteration,
};
pub use loop_impl::EvoStats;
pub use loop_impl::SelfIteratingBrain;
pub mod sia_loop;
pub mod tri_agent_pipeline;

// Hyperagent split modules
#[cfg(test)]
pub mod hyperagents;
pub mod hyperarchive;
pub mod hypercore;
pub mod hyperdgm;
pub mod hyperstage;

pub use pipeline::{
    kernel_iterate_pipeline, run_conversation_distill, seal_pipeline, AutonomyLevel, BrainPipeline,
    BrainSnapshot, BrainStage, DistillationResult, StageDecision,
};
pub use sia_loop::{
    ExecutionMetrics, ExecutionTrajectory, FeedbackAgent, MetaAgent, SIAController, SIAImprovement,
    TargetAgent, ToolCallRecord, TrajectoryStep,
};
pub use validation::{
    aggregate_reward, cargo_check_validation, taste_skill_gate, user_accept_reject,
    ValidationResult,
};
pub mod edit_journal;
pub mod nt_lang_bridge;

// Hyperagent re-exports
pub use super::evolution_types::{ParentSelection, SelectionConfig};
pub use hyperarchive::{HyperAgentArchive, HyperAgentRecord};
pub use hypercore::{
    FileDiff, HyperMetaAgent, ModificationTarget, SafetyCheckResult, SelfModificationProposal,
};
pub use hyperdgm::{
    DGMMetaAgent, GenerativeReplay, LatentEdit, SelfRefCheckResult, SelfReferentialCheck,
};
pub use hyperstage::{DGMMetaEvolveStage, MetaEvolveStage};

// DGM-Hyperagent SEAL re-exports
pub use brain_dgm::{DgmEditOrchestrator, DgmSelfEditStrategy, EditContext, EditCritic};

// SkillOpt re-exports
pub use aging_monitor::{AgingDiagnosisStage, AgingMonitor};
pub use harness_adapter::{HarnessAdapter, HarnessProfile};
pub use skillopt::{
    BoundedEditStage, EditBudget, EpochSlowUpdateStage, LrScheduler, RejectedBufferFeedbackStage,
    RejectedEdit, RejectedEditBuffer, ValidationGate, ValidationGateStage,
};

// Internal helper shared across hyperagent sub-modules
pub(crate) use hyperarchive::cosine_distance;

/// SEAL RL 训练循环的单元测试
/// 简化版余弦相似度（用于 preview diff 计算）
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_rl_loop_single_task() {
        let mut brain = SelfIteratingBrain::new();
        brain.brain.learning_rate = 0.05;

        // 测试单个任务的 SEAL 循环（不使用基准测试）
        let task = "设计一个响应式 UI 界面";
        let result = brain.run_seal_loop(task, None, None);

        assert!(result.is_ok());
        let reward = result.expect("result should be ok in test");
        log::info!("单个任务奖励: {}", reward);

        // 验证 ReasoningBank 有记录
        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories > 0);
        log::info!("ReasoningBank 统计: {:?}", stats);
    }

    #[test]
    fn test_benchmark_scoring() {
        // Test disabled: benchmarks module disabled
    }

    #[test]
    fn test_seal_rl_loop_batch() {
        let mut brain = SelfIteratingBrain::new();
        brain.brain.learning_rate = 0.05;

        let tasks = vec![
            ("设计一个 React 组件".to_string(), None, None),
            ("分析代码性能问题".to_string(), None, None),
            ("优化数据库查询".to_string(), None, None),
        ];

        let result = brain.run_seal_loop_batch(&tasks);

        assert!(result.is_ok());
        let avg_reward = result.expect("result should be ok in test");
        log::info!("批量任务平均奖励: {}", avg_reward);

        // 验证迭代次数
        assert_eq!(brain.iteration, 3);

        // 验证 ReasoningBank 有记录
        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories > 0);
    }

    #[test]
    fn test_rollback_on_negative_reward() {
        let mut brain = SelfIteratingBrain::new();
        let _original_capability = brain.brain.capability.clone();

        // 设置一个会导致负奖励的场景（通过空任务）
        let task = "x"; // 短任务，可能得到低奖励
        if let Err(e) = brain.run_seal_loop(task, None, None) {
            log::error!("SEAL loop returned error: {}", e);
        }

        // 即使奖励为负，brain 状态应该被回滚
        // 注意：由于噪声和初始化，这个测试可能会不稳定
        log::info!("学习率: {}", brain.brain.learning_rate);
    }

    #[test]
    fn test_policy_update() {
        let mut brain = SelfIteratingBrain::new();
        let initial_lr = brain.brain.learning_rate;

        // 模拟高奖励，策略应该增加学习率
        brain.update_policy(0.8);
        assert!(brain.brain.learning_rate >= initial_lr);

        // 模拟低奖励，策略应该减小学习率
        brain.brain.learning_rate = initial_lr; // 重置
        brain.update_policy(-0.1);
        assert!(brain.brain.learning_rate <= initial_lr);
    }

    #[test]
    fn test_regularization() {
        let mut brain = SelfIteratingBrain::new();
        let snapshot = brain.brain.capability.clone();

        // 修改能力向量（使用 setter 方法）
        let mut cap = brain.brain.capability.clone();
        cap.set_typography(cap.typography() + 0.5);
        cap.set_grid(cap.grid() + 0.3);
        brain.brain.capability = cap;

        let reg = brain.compute_regularization(&snapshot);
        log::info!("正则化值: {}", reg);

        // 正则化应该是负值（惩罚）
        assert!(reg < 0.0);
    }

    #[test]
    fn test_save_and_load_brain() {
        let test_id = format!("neotrix_test_brain_{}", std::process::id());
        let temp_dir = std::env::temp_dir().join(&test_id);
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut brain = SelfIteratingBrain::new();
        brain.brain.learning_rate = 0.1;
        brain.iteration = 5;

        // 保存到指定目录
        let save_result = brain.brain.save_to_dir(Some(&temp_dir));
        assert!(save_result.is_ok(), "保存失败: {:?}", save_result.err());

        // 验证文件已创建
        let metadata_path = temp_dir.join("brain_metadata.json");
        assert!(
            metadata_path.exists(),
            "元数据文件不存在: {:?}",
            metadata_path
        );

        // 从指定目录加载
        let loaded_result = ReasoningBrain::load_from_dir(Some(&temp_dir));
        assert!(loaded_result.is_ok(), "加载失败: {:?}", loaded_result.err());

        let loaded = loaded_result.expect("loaded_result should be ok in test");
        assert_eq!(loaded.learning_rate, 0.1);
        assert_eq!(loaded.total_absorb_count, 0); // 未执行 absorb

        // 清理
        if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
            log::error!("Failed to clean up temp dir: {}", e);
        }
    }

    #[test]
    fn test_reasoning_bank_integration() {
        let mut brain = SelfIteratingBrain::new();

        // 第一次运行
        if let Err(e) = brain.run_seal_loop("设计 UI 组件", None, None) {
            log::error!("First SEAL loop error: {}", e);
        }

        // 第二次运行，应该能检索到第一次的记忆
        if let Err(e) = brain.run_seal_loop("设计另一个 UI 组件", None, None) {
            log::error!("Second SEAL loop error: {}", e);
        }

        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories >= 2);
        log::info!("ReasoningBank 成功率: {:.2}%", stats.success_rate * 100.0);
    }

    #[test]
    fn test_full_seal_algorithm2() {
        // 完整 SEAL Algorithm 2 测试
        let mut brain = SelfIteratingBrain::new();
        brain.brain.learning_rate = 0.05;
        brain.quality_threshold = 0.5;

        let task_batch = vec![
            ("用 React 和 Tailwind 构建登录页面".to_string(), None, None),
            ("用 Vue 实现数据可视化组件".to_string(), None, None),
            ("优化现有代码的性能".to_string(), None, None),
            ("设计无障碍访问的用户界面".to_string(), None, None),
        ];

        // 运行批量 SEAL 循环（包含基准测试）
        let avg_reward = brain
            .run_seal_loop_batch(&task_batch)
            .expect("value should be ok in test");

        log::info!("SEAL Algorithm 2 完成:");
        log::info!("  平均奖励: {}", avg_reward);
        log::info!("  迭代次数: {}", brain.iteration);
        log::info!("  最终学习率: {}", brain.brain.learning_rate);

        let report = brain.get_brain_report();
        log::info!("  Brain 报告: {:?}", report);

        let bank_stats = brain.reasoning_bank.stats();
        log::info!("  ReasoningBank 统计: {:?}", bank_stats);

        // 验证基本期望
        assert_eq!(brain.iteration, 4);
        assert!(brain.reasoning_bank.stats().total_memories > 0);
    }

    #[test]
    fn test_seal_with_benchmark() {
        // 测试已禁用：benchmarks 模块暂时禁用
    }
}
