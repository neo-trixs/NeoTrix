use crate::core::CapabilityVector;
use crate::core::knowledge::KnowledgeSource;

pub(super) fn handle_cli_agent_tools(s: &KnowledgeSource) -> Option<CapabilityVector> {
    match s {
        KnowledgeSource::DeepSeekTui => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.4,
                0.1, 0.2, 0.1, 0.2,
                0.92, 0.6, 0.9, 0.85, 0.88,
                0.5, 0.6, 0.4,
                0.3, 0.4, 0.3,
                0.6, 0.7, 0.92, 0.85,
            );
            cv.extend_named(&[
                ("sub_agent_concurrency".into(), 0.95),
                ("lsp_diagnostics".into(), 0.92),
                ("sandbox_isolation".into(), 0.9),
                ("session_save_resume".into(), 0.88),
                ("workspace_rollback".into(), 0.85),
                ("prefix_cache_tracking".into(), 0.8),
                ("rlm_batched_analysis".into(), 0.85),
                ("skills_system".into(), 0.9),
            ]);
            Some(cv)
        }
        KnowledgeSource::Codebuff => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.7, 0.92, 0.88, 0.85,
                0.4, 0.5, 0.3,
                0.3, 0.3, 0.2,
                0.6, 0.7, 0.9, 0.85,
            );
            cv.extend_named(&[
                ("file_picker_agent".into(), 0.93),
                ("planner_agent".into(), 0.9),
                ("editor_agent".into(), 0.92),
                ("reviewer_agent".into(), 0.88),
                ("custom_agent_definition".into(), 0.95),
                ("agent_sdk".into(), 0.85),
                ("free_tier".into(), 0.75),
            ]);
            Some(cv)
        }
        KnowledgeSource::OpenClaude => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.88, 0.65, 0.85, 0.82, 0.85,
                0.3, 0.4, 0.3,
                0.2, 0.2, 0.2,
                0.5, 0.6, 0.85, 0.8,
            );
            cv.extend_named(&[
                ("multi_provider_routing".into(), 0.95),
                ("duckduckgo_search".into(), 0.9),
                ("grpc_headless".into(), 0.88),
                ("provider_profile".into(), 0.92),
                ("model_routing_table".into(), 0.9),
                ("firecrawl_integration".into(), 0.82),
            ]);
            Some(cv)
        }
        KnowledgeSource::Cairn => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.85, 0.5, 0.88, 0.8, 0.9,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.1,
                0.5, 0.6, 0.85, 0.8,
            );
            cv.extend_named(&[
                ("blackboard_architecture".into(), 0.95),
                ("fact_intent_graph".into(), 0.93),
                ("stigmergy_coordination".into(), 0.92),
                ("ooda_loop".into(), 0.9),
                ("container_isolation".into(), 0.85),
                ("runtime_task_generation".into(), 0.88),
                ("hint_mechanism".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::Orca => {
            let mut cv = CapabilityVector::from_values(
                0.5, 0.4, 0.5, 0.4,
                0.3, 0.3, 0.3, 0.3,
                0.8, 0.6, 0.8, 0.75, 0.7,
                0.4, 0.3, 0.3,
                0.3, 0.3, 0.2,
                0.5, 0.6, 0.7, 0.6,
            );
            cv.extend_named(&[
                ("worktree_isolation".into(), 0.93),
                ("ssh_worktree".into(), 0.88),
                ("mobile_companion".into(), 0.85),
                ("design_mode".into(), 0.82),
                ("multi_agent_ui".into(), 0.9),
                ("github_integration".into(), 0.88),
            ]);
            Some(cv)
        }
        KnowledgeSource::RedRun => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.1, 0.2,
                0.1, 0.2, 0.1, 0.2,
                0.85, 0.5, 0.9, 0.78, 0.92,
                0.2, 0.2, 0.1,
                0.1, 0.2, 0.1,
                0.4, 0.5, 0.85, 0.85,
            );
            cv.extend_named(&[
                ("agent_team_orchestration".into(), 0.93),
                ("engagement_state_db".into(), 0.9),
                ("semantic_skill_routing".into(), 0.88),
                ("ml_technique_selection".into(), 0.85),
                ("domain_teammate".into(), 0.9),
                ("persistent_context".into(), 0.88),
                ("state_dashboard".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::AutonomousSpeedrunning => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.7, 0.92, 0.9, 0.85,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("multi_wave_experiment".into(), 0.92),
                ("run_tracking".into(), 0.9),
                ("novelty_gating".into(), 0.88),
                ("harness_framework".into(), 0.85),
                ("aggregated_analysis".into(), 0.82),
            ]);
            Some(cv)
        }
        KnowledgeSource::Synesis => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.2,
                0.88, 0.6, 0.85, 0.85, 0.9,
                0.4, 0.3, 0.2,
                0.3, 0.3, 0.2,
                0.65, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("ucb1_bandit_scoring".into(), 0.95),
                ("ml_reward_model".into(), 0.93),
                ("faiss_embedding_index".into(), 0.92),
                ("stale_rule_detection".into(), 0.88),
                ("contradiction_resolution".into(), 0.87),
                ("retrieval_param_optimization".into(), 0.9),
                ("consolidation_clustering".into(), 0.85),
            ]);
            Some(cv)
        }
        _ => None,
    }
}
