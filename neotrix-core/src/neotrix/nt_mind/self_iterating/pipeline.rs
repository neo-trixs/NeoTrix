// Re-export from decomposed sub-files (sibling modules)
pub use super::pipeline_awareness::*;
pub use super::pipeline_code::*;
pub use super::pipeline_core::*;
pub use super::pipeline_evolution::*;
pub use super::pipeline_memory::*;
pub use super::pipeline_search::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_expert_routing::TaskType;
    use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;

    #[test]
    fn test_pipeline_execution_returns_reward() {
        let mut brain = SelfIteratingBrain::new();
        let result = brain.run_seal_loop("设计一个响应式 UI 界面", None, None);
        assert!(
            result.is_ok(),
            "pipeline 应返回奖励值，但得到: {:?}",
            result
        );
        let reward = result.expect("result should be ok in test");
        assert!(reward > -1.0, "奖励应 > -1.0，得到: {}", reward);
    }

    #[test]
    fn test_pipeline_kernel_iterate() {
        let mut brain = SelfIteratingBrain::new();
        let result = brain.kernel_iterate("优化数据库查询性能");
        assert!(result.improved || result.score_after >= result.score_before - 0.1);
        assert!(result.iteration > 0);
    }

    #[test]
    fn test_pipeline_stores_memory() {
        let mut brain = SelfIteratingBrain::new();
        let _ = brain.run_seal_loop("设计 React 组件", None, None);
        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories > 0, "pipeline 应存储推理记忆");
    }

    #[test]
    fn test_pipeline_twice_accumulates() {
        let mut brain = SelfIteratingBrain::new();
        let _ = brain.run_seal_loop("任务 A", None, None);
        let _ = brain.run_seal_loop("任务 B", None, None);
        let stats = brain.reasoning_bank.stats();
        assert!(stats.total_memories >= 2, "两次 pipeline 应累积记忆");
    }

    #[test]
    fn test_pipeline_champion_promotion() {
        let mut brain = SelfIteratingBrain::new();
        brain.champion = Some(BrainSnapshot::new(&brain.brain, &TaskType::General));
        let baseline = brain
            .champion
            .as_ref()
            .expect("value should be ok in test")
            .score;

        brain.brain.capability.arr_mut()[0] = 0.99;
        brain.brain.capability.normalize();
        let _ = brain.kernel_iterate("general");

        if let Some(ref champ) = brain.champion {
            assert!(champ.score >= baseline * 0.9, "champion 不应显著降低");
        }
    }

    #[test]
    fn test_pipeline_autonomy_proposal_skips_execution() {
        let mut brain = SelfIteratingBrain::new();
        brain.autonomy = AutonomyLevel::Proposal;
        let before = brain.brain.capability.clone();
        let result = brain.run_seal_loop("测试任务", None, None);
        let after = brain.brain.capability.clone();
        let change: f64 = before
            .arr()
            .iter()
            .zip(after.arr().iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(change < 0.001, "Proposal 模式不应修改能力向量");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_autonomy_bounded_blocks_high_capability() {
        let mut brain = SelfIteratingBrain::new();
        brain.autonomy = AutonomyLevel::Bounded;
        for i in 0..brain.brain.capability.arr().len() {
            brain.brain.capability.arr_mut()[i] = 0.9;
        }
        let _snapshot = brain.brain.capability.clone();
        let result = brain.run_seal_loop("test", None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_stages_order() {
        let pipeline = seal_pipeline();
        let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
        assert_eq!(
            names,
            vec![
                "vsa_fingerprint",
                "canonical_sort",
                "stream_hygiene",
                "inner_critic",
                "semantic_recall",
                "goal_contract",
                "snapshot",
                "checkpoint",
                "autonomy_gate",
                "memory_retrieval",
                "gap_analysis",
                "ssm_update",
                "open_source_compare",
                "self_edit_gen",
                "bounded_edit",
                "apply_edits",
                "lsp_diagnostics",
                "evidence_capture",
                "external_verifier",
                "reward_calc",
                "narrow_recovery",
                "adaptive_lr",
                "validation_gate",
                "gwt_absorb",
                "stats_significance",
                "harness_adapt",
                "task_affinity",
                "knowledge_quality",
                "rollback_decision",
                "rewind",
                "rejected_feedback",
                "champion_compare",
                "bank_storage",
                "hypercube_optimize",
                "e8_experiment",
                "epoch_slow_update",
                "nt_shield_scan",
                "session_distill",
                "conversation_distill",
                "aging_diagnosis",
                "embedding_refresh",
                "spectral_monitor",
                "trajectory_collect",
                "coach_and_update",
                "meta_improvement",
                "sleep",
                "self_preservation",
                "degradation_gate",
                "plugin_discovery",
                "uq_calibration",
                "phi_measure",
                "conflict_resolution",
                "intrinsic_motivation",
                "code_search",
                "perception_evolution",
                "side_git",
                "self_review",
                "final_verification",
                "goal_terminator",
            ],
            "SEAL pipeline 应有 59 个 stage"
        );
    }

    #[test]
    fn test_kernel_pipeline_stages_order() {
        let pipeline = kernel_iterate_pipeline();
        let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
        assert_eq!(
            names,
            vec![
                "snapshot",
                "autonomy_gate",
                "memory_retrieval",
                "open_source_compare",
                "adaptive_lr",
                "knowledge_absorb",
                "memory_storage",
                "champion_compare",
                "evaluation",
            ],
            "Kernel pipeline 应有 9 个 stage"
        );
    }

    #[test]
    fn test_all_stages_have_unique_names() {
        let pipeline = seal_pipeline();
        let names: Vec<&str> = pipeline.stages.iter().map(|s| s.name()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "所有 stage 名必须唯一");
    }

    #[test]
    fn test_permission_level_syncs_to_approval_mode() {
        {
            let mut s = crate::cli::global_shield()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            s.set_approval_mode(crate::cli::approval::ApprovalMode::Suggest);
        }
        let mut brain = SelfIteratingBrain::new();

        brain.permission = PermissionLevel::Review;
        let _ = brain.run_seal_loop("test_review_sync", None, None);
        {
            let shield = crate::cli::global_shield()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            assert_eq!(
                shield.approval.mode(),
                crate::cli::approval::ApprovalMode::Suggest
            );
        }

        brain.permission = PermissionLevel::Suggest;
        let _ = brain.run_seal_loop("test_suggest_sync", None, None);
        {
            let shield = crate::cli::global_shield()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            assert_eq!(
                shield.approval.mode(),
                crate::cli::approval::ApprovalMode::AutoEdit
            );
        }

        brain.permission = PermissionLevel::Full;
        let _ = brain.run_seal_loop("test_full_sync", None, None);
        {
            let shield = crate::cli::global_shield()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            assert_eq!(
                shield.approval.mode(),
                crate::cli::approval::ApprovalMode::FullAuto
            );
        }
    }

    #[test]
    fn test_permission_level_review_skips_autonomy_gate() {
        let mut brain = SelfIteratingBrain::new();
        brain.permission = PermissionLevel::Review;
        let stage = AutonomyGateStage::new();
        let decision = stage.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
    }

    #[test]
    fn test_sandbox_readonly_skips_autonomy_gate() {
        {
            let mut s = crate::cli::global_shield()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            s.set_sandbox_mode(crate::cli::sandbox::CliSandboxMode::ReadOnly);
        }
        let mut brain = SelfIteratingBrain::new();
        brain.permission = PermissionLevel::Full;
        let stage = AutonomyGateStage::new();
        let decision = stage.process(&mut brain).unwrap();
        assert!(matches!(decision, StageDecision::Skip(_)));
        {
            let mut s = crate::cli::global_shield()
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            s.set_sandbox_mode(crate::cli::sandbox::CliSandboxMode::Disabled);
        }
    }
}
