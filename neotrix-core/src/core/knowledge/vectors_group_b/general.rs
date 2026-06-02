use crate::core::{CapabilityVector, knowledge::KnowledgeSource};
pub(super) fn cap_vec_general(s: &KnowledgeSource) -> CapabilityVector {
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
            cv
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
            cv
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
            cv
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
            cv
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
            cv
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
            cv
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
            cv
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
            cv
        }
        KnowledgeSource::MemOS => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.3, 0.2, 0.2,
                0.9, 0.6, 0.88, 0.9, 0.85,
                0.5, 0.4, 0.3,
                0.3, 0.3, 0.2,
                0.7, 0.8, 0.9, 0.88,
            );
            cv.extend_named(&[
                ("multi_agent_memory_sharing".into(), 0.93),
                ("skill_evolution".into(), 0.92),
                ("hybrid_search_fts5".into(), 0.9),
                ("memory_scheduler".into(), 0.88),
                ("multi_cube_kb".into(), 0.85),
                ("memory_feedback_correction".into(), 0.87),
                ("cross_task_skill_reuse".into(), 0.9),
            ]);
            cv
        }
        KnowledgeSource::Reflexio => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.92, 0.7, 0.9, 0.88, 0.85,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.92, 0.85,
            );
            cv.extend_named(&[
                ("playbook_extraction".into(), 0.95),
                ("error_avoidance".into(), 0.93),
                ("expert_learning".into(), 0.9),
                ("success_path_persistence".into(), 0.92),
                ("transfer_learning_across_users".into(), 0.88),
                ("hybrid_search_retrieval".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::Mem0 => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.65, 0.92, 0.9, 0.88,
                0.4, 0.3, 0.2,
                0.3, 0.3, 0.2,
                0.7, 0.8, 0.9, 0.88,
            );
            cv.extend_named(&[
                ("single_pass_add_extraction".into(), 0.94),
                ("entity_linking".into(), 0.93),
                ("multi_signal_retrieval".into(), 0.92),
                ("agent_generated_facts".into(), 0.9),
                ("multi_level_memory".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::Mnemosyne => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.3, 0.3, 0.3,
                0.88, 0.6, 0.85, 0.88, 0.85,
                0.4, 0.3, 0.2,
                0.3, 0.3, 0.2,
                0.65, 0.75, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("five_layer_cognitive".into(), 0.93),
                ("rl_consolidation".into(), 0.92),
                ("theory_of_mind_for_agents".into(), 0.9),
                ("flash_reasoning_bfs".into(), 0.88),
                ("cross_agent_synthesis".into(), 0.9),
                ("proactive_recall".into(), 0.87),
                ("active_consolidation".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::OriMnemos => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.6, 0.88, 0.85, 0.88,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("actr_decay".into(), 0.93),
                ("hebbian_cooccurrence".into(), 0.92),
                ("recursive_harness".into(), 0.9),
                ("spreading_activation".into(), 0.88),
                ("hub_dampening".into(), 0.85),
                ("resolution_boost".into(), 0.87),
                ("zero_cloud_dependency".into(), 0.9),
            ]);
            cv
        }
        KnowledgeSource::OPSD => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.95, 0.65, 0.92, 0.9, 0.88,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.55, 0.6, 0.85, 0.85,
            );
            cv.extend_named(&[
                ("on_policy_self_distillation".into(), 0.95),
                ("token_level_jsd".into(), 0.92),
                ("fixed_teacher_lora".into(), 0.88),
                ("kl_clipping_stabilization".into(), 0.9),
                ("student_teacher_context".into(), 0.87),
            ]);
            cv
        }
        KnowledgeSource::AttentionMechanism => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.6,
                0.85, 0.6, 0.9, 0.8, 0.7,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.7, 0.5,
            );
            cv.extend_named(&[
                ("global_workspace_attention".into(), 0.92),
                ("vsa_binding".into(), 0.9),
                ("salience_competition".into(), 0.88),
                ("broadcast_dynamics".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::PatchFile => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.5, 0.5, 0.6, 0.6, 0.4,
                0.3, 0.7, 0.3,
                0.3, 0.2, 0.2,
                0.85, 0.4, 0.7, 0.5,
            );
            cv.extend_named(&[
                ("surgical_edit".into(), 0.95),
                ("patch_application".into(), 0.92),
                ("line_precision".into(), 0.9),
                ("context_aware_hunk".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::KeyVault => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.4, 0.3, 0.5, 0.4, 0.3,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.5, 0.8, 0.95,
            );
            cv.extend_named(&[
                ("dual_layer_secret".into(), 0.95),
                ("key_rotation".into(), 0.92),
                ("access_audit".into(), 0.9),
                ("encryption_at_rest".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::SealLoop => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.65, 0.85, 0.75, 0.8,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.8, 0.6, 0.8, 0.7,
            );
            cv.extend_named(&[
                ("seal_iteration".into(), 0.95),
                ("self_edit_generation".into(), 0.92),
                ("external_reward".into(), 0.9),
                ("capability_absorption".into(), 0.88),
                ("iteration_threshold".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::HashCortxAgents => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.3, 0.3,
                0.85, 0.6, 0.88, 0.85, 0.8,
                0.3, 0.5, 0.2,
                0.2, 0.2, 0.2,
                0.8, 0.6, 0.5, 0.4,
            );
            cv.extend_named(&[
                ("agent_template_system".into(), 0.95),
                ("concrete_agent_presets".into(), 0.93),
                ("template_composition".into(), 0.9),
                ("role_specialization".into(), 0.88),
                ("configuration_reuse".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::HashCortxSecurity => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.1, 0.1, 0.2, 0.2,
                0.7, 0.3, 0.9, 0.7, 0.95,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.4, 0.5, 0.95, 0.9,
            );
            cv.extend_named(&[
                ("interactive_guard".into(), 0.95),
                ("deny_list_management".into(), 0.93),
                ("audit_log".into(), 0.9),
                ("permission_scoping".into(), 0.88),
                ("security_workflow".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::HashCortxSwarm => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.88, 0.6, 0.88, 0.85, 0.8,
                0.2, 0.7, 0.2,
                0.2, 0.2, 0.2,
                0.9, 0.7, 0.4, 0.4,
            );
            cv.extend_named(&[
                ("swarm_execution".into(), 0.95),
                ("agent_coordination".into(), 0.93),
                ("parallel_task_distribution".into(), 0.9),
                ("result_aggregation".into(), 0.88),
                ("swarm_orchestration".into(), 0.92),
            ]);
            cv
        }
        KnowledgeSource::HashCortxFailover => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.1, 0.1, 0.2, 0.2,
                0.75, 0.3, 0.8, 0.7, 0.75,
                0.1, 0.3, 0.1,
                0.1, 0.1, 0.1,
                0.85, 0.6, 0.7, 0.6,
            );
            cv.extend_named(&[
                ("tier_aware_failover".into(), 0.95),
                ("provider_routing".into(), 0.93),
                ("cost_aware_selection".into(), 0.9),
                ("model_fallback_chain".into(), 0.88),
                ("health_check".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::HetuLuoshu => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.2, 0.1,
                0.3, 0.3, 0.1, 0.1,
                0.9, 0.8, 0.85, 0.85, 0.9,
                0.1, 0.3, 0.1,
                0.1, 0.1, 0.1,
                0.4, 0.5, 0.85, 0.6,
            );
            cv.extend_named(&[
                ("he_tu_luo_shu".into(), 0.98),
                ("archaeological_cosmology".into(), 0.95),
                ("pattern_matrix".into(), 0.93),
                ("magic_square".into(), 0.9),
                ("prehistoric_mathematics".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::YijingBinary => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.2, 0.1,
                0.2, 0.3, 0.1, 0.15,
                0.95, 0.85, 0.9, 0.85, 0.92,
                0.1, 0.25, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.6, 0.8, 0.55,
            );
            cv.extend_named(&[
                ("binary_encoding".into(), 0.98),
                ("xiantian_sequence".into(), 0.95),
                ("leibniz_correspondence".into(), 0.93),
                ("i_ching_table".into(), 0.95),
                ("hexagram_circulation".into(), 0.9),
            ]);
            cv
        }
        KnowledgeSource::FivePhasesGauge => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.15, 0.1,
                0.2, 0.25, 0.1, 0.1,
                0.92, 0.85, 0.88, 0.95, 0.9,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.85, 0.82, 0.7,
            );
            cv.extend_named(&[
                ("five_phases_gauge".into(), 0.95),
                ("wu_xing_symmetry".into(), 0.93),
                ("standard_model_mapping".into(), 0.92),
                ("generation_restriction".into(), 0.9),
                ("su3_root_system".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::ThreeCosmologies => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.2, 0.1,
                0.3, 0.35, 0.1, 0.2,
                0.88, 0.8, 0.85, 0.9, 0.88,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.4, 0.7, 0.8, 0.65,
            );
            cv.extend_named(&[
                ("gaitian_geometry".into(), 0.95),
                ("huntian_sphere".into(), 0.93),
                ("xuanye_void".into(), 0.95),
                ("modern_physics_mapping".into(), 0.92),
                ("ancient_cosmology".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::HuainanziCalendar => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.2, 0.1,
                0.25, 0.3, 0.1, 0.15,
                0.9, 0.8, 0.92, 0.93, 0.85,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.45, 0.65, 0.85, 0.6,
            );
            cv.extend_named(&[
                ("universal_embedding_matrix".into(), 0.95),
                ("solar_term_calendar".into(), 0.93),
                ("twenty_four_qi".into(), 0.92),
                ("huainanzi_astronomy".into(), 0.9),
                ("celestial_chronology".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::ZhangHengSeismoscope => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.15, 0.1,
                0.2, 0.3, 0.1, 0.2,
                0.88, 0.75, 0.95, 0.85, 0.9,
                0.1, 0.15, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.8, 0.82, 0.75,
            );
            cv.extend_named(&[
                ("resonance_detection".into(), 0.95),
                ("mechanical_amplification".into(), 0.93),
                ("ligo_prototype".into(), 0.9),
                ("inverted_pendulum".into(), 0.92),
                ("2025_restoration".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::MawangduiAstronomy => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.2, 0.1,
                0.3, 0.35, 0.15, 0.15,
                0.85, 0.8, 0.92, 0.88, 0.9,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.4, 0.7, 0.8, 0.65,
            );
            cv.extend_named(&[
                ("comet_atlas".into(), 0.95),
                ("wu_xing_zhan".into(), 0.93),
                ("planetary_chronology".into(), 0.9),
                ("daoyin_chart".into(), 0.88),
                ("han_dynasty_astronomy".into(), 0.9),
            ]);
            cv
        }
        KnowledgeSource::ShaoYongCosmology => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.15, 0.1,
                0.25, 0.35, 0.1, 0.2,
                0.92, 0.85, 0.88, 0.92, 0.9,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.35, 0.75, 0.82, 0.7,
            );
            cv.extend_named(&[
                ("cosmic_cycle_129600".into(), 0.95),
                ("huangji_jingshi".into(), 0.93),
                ("penrose_ccc_mapping".into(), 0.9),
                ("time_fractal".into(), 0.92),
                ("yuan_hui_yun_shi".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::DayanNumber => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.15, 0.1,
                0.2, 0.3, 0.1, 0.1,
                0.95, 0.85, 0.95, 0.9, 0.92,
                0.1, 0.2, 0.1,
                0.1, 0.1, 0.1,
                0.3, 0.8, 0.85, 0.7,
            );
            cv.extend_named(&[
                ("dayan_number_50".into(), 0.95),
                ("zhao_shuang_proof".into(), 0.93),
                ("quantum_measurement_foundation".into(), 0.92),
                ("jiao_weifang_binomial".into(), 0.9),
                ("yarrow_stalk_algorithm".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::AdamsLaw => {
            let mut cv = CapabilityVector::from_values(
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0,
                0.0, 0.3, 0.5,
                0.0, 0.7, 0.6, 0.4,
            );
            cv.extend_named(&[
                ("text_frequency".into(), 0.95),
                ("prompt_paraphrase".into(), 0.85),
                ("curriculum_learning".into(), 0.80),
                ("distillation".into(), 0.75),
                ("cross_model".into(), 0.90),
                ("cross_task".into(), 0.88),
            ]);
            cv
        }
        _ => unreachable!("specialized sources handled by specialized helper"),
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
