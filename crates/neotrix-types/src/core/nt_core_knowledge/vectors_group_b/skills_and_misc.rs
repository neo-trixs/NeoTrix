use crate::core::CapabilityVector;
use crate::core::nt_core_knowledge::KnowledgeSource;

pub(super) fn handle_skills_and_misc(s: &KnowledgeSource) -> CapabilityVector {
    match s {
        KnowledgeSource::LlmWiki => {
            let mut cv = CapabilityVector::from_values(
                0.4, 0.4, 0.3, 0.4,
                0.3, 0.4, 0.3, 0.3,
                0.88, 0.65, 0.85, 0.88, 0.92,
                0.5, 0.4, 0.3,
                0.3, 0.3, 0.2,
                0.7, 0.75, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("incremental_wiki_build".into(), 0.95),
                ("knowledge_graph_4signal".into(), 0.93),
                ("louvain_clustering".into(), 0.9),
                ("multi_provider_llm_routing".into(), 0.88),
                ("agent_skill_api".into(), 0.92),
                ("cascade_file_deletion".into(), 0.85),
                ("two_step_cot_ingest".into(), 0.9),
                ("deep_research_pipeline".into(), 0.87),
            ]);
            cv
        }
        KnowledgeSource::DarwinSkill => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.15, 0.2,
                0.15, 0.15, 0.2, 0.15,
                0.92, 0.88, 0.95, 0.9, 0.96,
                0.95, 0.85, 0.98,
                0.8, 0.8, 0.7,
                0.7, 0.3, 0.3, 0.3,
            );
            cv.extend_named(&[
                ("skill_rubric_evaluation".into(), 0.98),
                ("ratchet_mechanism".into(), 0.97),
                ("multi_judge_independent".into(), 0.95),
                ("early_stopping".into(), 0.93),
                ("runtime_neutrality".into(), 0.9),
                ("anti_pattern_blacklist".into(), 0.92),
                ("skill_lens_rubric_9dim".into(), 0.96),
                ("optimization_5phase".into(), 0.94),
            ]);
            cv
        }
        KnowledgeSource::QwenCode => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.7, 0.92, 0.88, 0.9,
                0.4, 0.5, 0.3,
                0.3, 0.3, 0.2,
                0.7, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("gemini_style_cli".into(), 0.95),
                ("multi_model_routing".into(), 0.92),
                ("agent_fork_capability".into(), 0.9),
                ("session_management".into(), 0.88),
                ("tool_abstraction".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::SkillOpt => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.4,
                0.95, 0.7, 0.92, 0.88, 0.85,
                0.4, 0.3, 0.2,
                0.3, 0.3, 0.2,
                0.7, 0.75, 0.9, 0.95,
            );
            cv.extend_named(&[
                ("textual_learning_rate".into(), 0.95),
                ("rejected_edit_buffer".into(), 0.93),
                ("validation_score_gate".into(), 0.94),
                ("bounded_edit_space".into(), 0.92),
                ("scored_rollout_analysis".into(), 0.9),
                ("epoch_wise_meta_update".into(), 0.88),
                ("text_space_optimizer".into(), 0.96),
                ("held_out_validation".into(), 0.91),
            ]);
            cv
        }
        KnowledgeSource::MuseAutoskill => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.9, 0.65, 0.88, 0.92, 0.85,
                0.3, 0.3, 0.2,
                0.2, 0.2, 0.2,
                0.65, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("skill_level_memory".into(), 0.95),
                ("skill_lifecycle_management".into(), 0.93),
                ("cross_agent_skill_transfer".into(), 0.88),
                ("on_demand_skill_creation".into(), 0.9),
                ("unit_test_feedback".into(), 0.85),
                ("runtime_feedback_loop".into(), 0.87),
                ("skill_version_tracking".into(), 0.92),
                ("skill_reuse_optimization".into(), 0.89),
            ]);
            cv
        }
        KnowledgeSource::FeynmanAgent => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.88, 0.65, 0.9, 0.85, 0.82,
                0.3, 0.3, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.65, 0.85, 0.8,
            );
            cv.extend_named(&[
                ("harness_decomposition".into(), 0.95),
                ("provider_router".into(), 0.93),
                ("credential_vault".into(), 0.9),
                ("policy_engine".into(), 0.92),
                ("approval_gate".into(), 0.88),
                ("model_catalog".into(), 0.9),
                ("session_storage".into(), 0.85),
                ("budget_tracker".into(), 0.87),
                ("turn_loop".into(), 0.86),
                ("iii_trigger_primitive".into(), 0.94),
            ]);
            cv
        }
        KnowledgeSource::AwesomeArchitecture => {
            let mut cv = CapabilityVector::from_values(
                0.5, 0.85, 0.6, 0.7,
                0.7, 0.6, 0.7, 0.5,
                0.8, 0.7, 0.88, 0.85, 0.9,
                0.5, 0.4, 0.3,
                0.4, 0.4, 0.3,
                0.6, 0.7, 0.8, 0.75,
            );
            cv.extend_named(&[
                ("system_design_templates".into(), 0.95),
                ("architectural_thinking".into(), 0.93),
                ("c4_model_diagrams".into(), 0.9),
                ("evolution_design".into(), 0.88),
                ("quality_tradeoff_analysis".into(), 0.92),
                ("distributed_system_patterns".into(), 0.9),
            ]);
            cv
        }
        KnowledgeSource::VulnGym => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.88, 0.5, 0.9, 0.8, 0.92,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.5, 0.6, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("vulnerability_benchmark".into(), 0.95),
                ("business_logic_security".into(), 0.94),
                ("entry_point_trace".into(), 0.93),
                ("critical_operation_identification".into(), 0.92),
                ("cross_module_reasoning".into(), 0.9),
                ("vulnerability_taxonomy_12class".into(), 0.91),
                ("white_box_vuln_hunting".into(), 0.93),
                ("verify_annotation".into(), 0.85),
            ]);
            cv
        }
        KnowledgeSource::Maigret => {
            let mut cv = CapabilityVector::from_values(
                0.15, 0.15, 0.15, 0.15,
                0.2, 0.15, 0.15, 0.3,
                0.85, 0.5, 0.88, 0.75, 0.8,
                0.15, 0.15, 0.15,
                0.15, 0.15, 0.15,
                0.3, 0.4, 0.85, 0.9,
            );
            cv.extend_named(&[
                ("github_osint".into(), 0.95),
                ("username_search".into(), 0.93),
                ("social_media_lookup".into(), 0.9),
                ("profile_analysis".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::TasteSkill => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.6,
                0.88, 0.7, 0.92, 0.9, 0.85,
                0.2, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.4, 0.5, 0.92, 0.95,
            );
            cv.extend_named(&[
                ("multi_judge_rubric".into(), 0.95),
                ("skill_quality_eval".into(), 0.93),
                ("lens_evaluation".into(), 0.9),
            ]);
            cv
        }
        KnowledgeSource::UnderstandAnything => {
            let mut cv = CapabilityVector::from_values(
                0.15, 0.15, 0.15, 0.15,
                0.15, 0.2, 0.2, 0.7,
                0.92, 0.6, 0.88, 0.85, 0.9,
                0.15, 0.15, 0.15,
                0.15, 0.15, 0.15,
                0.5, 0.55, 0.85, 0.82,
            );
            cv.extend_named(&[
                ("self_understanding".into(), 0.95),
                ("introspection_depth".into(), 0.92),
                ("gap_analysis".into(), 0.9),
                ("cognitive_mapping".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::CarbonCode => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.3, 0.2, 0.2, 0.5,
                0.85, 0.6, 0.9, 0.8, 0.88,
                0.2, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.35, 0.5, 0.85, 0.88,
            );
            cv.extend_named(&[
                ("carbon_optimization".into(), 0.95),
                ("energy_efficiency".into(), 0.93),
                ("green_computing".into(), 0.9),
                ("code_profiling".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::LlmArch => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.55,
                0.88, 0.6, 0.92, 0.85, 0.9,
                0.2, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.45, 0.5, 0.82, 0.85,
            );
            cv.extend_named(&[
                ("llm_architecture_knowledge".into(), 0.95),
                ("model_taxonomy".into(), 0.93),
                ("architecture_patterns".into(), 0.9),
                ("alignment_mapping".into(), 0.88),
            ]);
            cv
        }
        KnowledgeSource::Spear => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.3, 0.3,
                0.3, 0.4, 0.3, 0.5,
                0.88, 0.75, 0.92, 0.85, 0.85,
                0.4, 0.3, 0.3,
                0.3, 0.3, 0.3,
                0.7, 0.8, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("codeact_tools".into(), 0.95),
                ("prompt_optimization".into(), 0.92),
                ("evaluate_tool".into(), 0.93),
                ("set_prompt_tool".into(), 0.90),
                ("guard_metric_floor".into(), 0.88),
                ("auto_rollback".into(), 0.92),
            ]);
            cv
        }
        KnowledgeSource::Sia => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.15, 0.2, 0.2, 0.75,
                0.75, 0.65, 0.78, 0.8, 0.7,
                0.2, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.4, 0.5, 0.8, 0.85,
            );
            cv.extend_named(&[
                ("scaffold_rewrite".into(), 0.90),
                ("rl_training".into(), 0.88),
                ("trajectory_analysis".into(), 0.92),
                ("improvement_rationale".into(), 0.85),
                ("lora_adaptation".into(), 0.82),
            ]);
            cv
        }
        KnowledgeSource::SkillsGate => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.3, 0.2, 0.2, 0.6,
                0.85, 0.55, 0.82, 0.8, 0.78,
                0.2, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.5, 0.6, 0.88, 0.82,
            );
            cv.extend_named(&[
                ("skill_discovery".into(), 0.95),
                ("per_agent_management".into(), 0.93),
                ("skill_catalog_91k".into(), 0.92),
                ("remote_ssh_sync".into(), 0.85),
                ("private_skills".into(), 0.85),
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
        _ => unreachable!("group A variants handled by group_a helper"),
    }
}
