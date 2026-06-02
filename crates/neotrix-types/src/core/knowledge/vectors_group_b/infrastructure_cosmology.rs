use crate::core::CapabilityVector;
use crate::core::knowledge::KnowledgeSource;

pub(super) fn handle_infrastructure_cosmology(s: &KnowledgeSource) -> Option<CapabilityVector> {
    match s {
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
        }
        KnowledgeSource::SecurityAttacks => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.3, 0.3, 0.2, 0.4,
                0.95, 0.85, 0.98, 0.9, 0.95,
                0.3, 0.3, 0.4,
                0.4, 0.3, 0.2,
                0.6, 0.7, 0.92, 0.95,
            );
            cv.extend_named(&[
                ("http_proxy_intercept".into(), 0.98),
                ("poc_validation".into(), 0.96),
                ("browser_security_test".into(), 0.95),
                ("vulnerability_scanning".into(), 0.94),
                ("osint_reconnaissance".into(), 0.9),
                ("web_security_knowledge".into(), 0.95),
                ("attack_surface_mapping".into(), 0.88),
                ("credential_testing".into(), 0.85),
                ("csrf_detection".into(), 0.92),
                ("xss_detection".into(), 0.94),
                ("sql_injection_test".into(), 0.93),
                ("ssrf_detection".into(), 0.9),
            ]);
            Some(cv)
        }
        _ => None,
    }
}
