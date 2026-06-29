use crate::core::CapabilityVector;
use crate::core::nt_core_knowledge::KnowledgeSource;

pub(super) fn capability_vector_group_a(s: &KnowledgeSource) -> Option<CapabilityVector> {
    match s {
        KnowledgeSource::HeroUI => Some(CapabilityVector::from_values(
            0.6, 0.7, 0.8, 0.6,
            0.4, 0.5, 0.5, 0.3,
            0.8, 0.7, 0.7, 0.8,
            0.9,
            0.7, 0.98, 0.7,
            0.6, 0.9, 0.7,
            0.5, 0.5, 0.6, 0.6,
        )),
        KnowledgeSource::BaseUI => Some(CapabilityVector::from_values(
            0.5, 0.6, 0.5, 0.7,
            0.3, 0.4, 0.6, 0.2,
            0.9, 0.5, 0.8, 0.7,
            0.95,
            0.98, 0.7, 0.3,
            0.99, 0.6, 0.3,
            0.4, 0.5, 0.7, 0.8,
        )),
        KnowledgeSource::ArcUI => Some(CapabilityVector::from_values(
            0.5, 0.5, 0.4, 0.6,
            0.3, 0.6, 0.5, 0.4,
            0.8, 0.6, 0.7, 0.8,
            0.85,
            0.8, 0.9, 0.4,
            0.7, 0.5, 0.3,
            0.98, 0.6, 0.7, 0.7,
        )),
        KnowledgeSource::CortexUI => Some(CapabilityVector::from_values(
            0.5, 0.6, 0.4, 0.6,
            0.3, 0.5, 0.6, 0.5,
            0.85, 0.6, 0.8, 0.8,
            0.9,
            0.9, 0.8, 0.4,
            0.8, 0.5, 0.3,
            0.7, 0.99, 0.8, 0.8,
        )),
        KnowledgeSource::AgenticDS => Some(CapabilityVector::from_values(
            0.6, 0.7, 0.5, 0.6,
            0.4, 0.5, 0.6, 0.4,
            0.9, 0.6, 0.85, 0.8,
            0.95,
            0.85, 0.8, 0.5,
            0.7, 0.6, 0.4,
            0.7, 0.8, 0.99, 0.99,
        )),
        KnowledgeSource::DesignPhilosophy => Some(CapabilityVector::from_values(
            0.8, 0.85, 0.6, 0.7,
            0.7, 0.8, 0.75, 0.7,
            0.8, 0.85, 0.75, 0.8,
            0.95,
            0.5, 0.6, 0.5,
            0.4, 0.6, 0.5,
            0.3, 0.4, 0.5, 0.5,
        )),
        KnowledgeSource::Hyperframes => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.5, 0.3,
                0.2, 0.6, 0.2, 0.5,
                0.85, 0.7, 0.8, 0.7, 0.6,
                0.3, 0.3, 0.2,
                0.2, 0.9, 0.3,
                0.4, 0.5, 0.6, 0.0,
            );
            cv.extend_named(&[
                ("video_rendering".into(), 0.95),
                ("html_composition".into(), 0.9),
                ("deterministic_capture".into(), 0.85),
                ("frame_adapter".into(), 0.8),
                ("puppeteer_ffmpeg".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::Betterleaks => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.1, 0.1, 0.1, 0.1,
                0.7, 0.3, 0.9, 0.6, 0.95,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.5, 0.8, 0.95, 0.0,
            );
            cv.extend_named(&[
                ("secret_detection".into(), 0.95),
                ("cel_filtering".into(), 0.9),
                ("bpe_entropy".into(), 0.8),
                ("scan_parallelism".into(), 0.85),
                ("secret_validation".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::YaoWebsecurity => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.1, 0.2,
                0.1, 0.2, 0.2, 0.2,
                0.8, 0.5, 0.85, 0.7, 0.9,
                0.1, 0.2, 0.1,
                0.1, 0.2, 0.1,
                0.3, 0.4, 0.8, 0.0,
            );
            cv.extend_named(&[
                ("security_audit".into(), 0.95),
                ("vulnerability_knowledge".into(), 0.9),
                ("report_generation".into(), 0.85),
                ("review_workflow".into(), 0.9),
                ("attack_surface_analysis".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::Botasaurus => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.6, 0.5, 0.7, 0.5, 0.6,
                0.2, 0.7, 0.4,
                0.2, 0.3, 0.2,
                0.3, 0.5, 0.7, 0.0,
            );
            cv.extend_named(&[
                ("anti_detection".into(), 0.95),
                ("web_scraping".into(), 0.9),
                ("human_mouse_trajectory".into(), 0.85),
                ("ui_builder".into(), 0.8),
                ("desktop_extractor".into(), 0.7),
            ]);
            Some(cv)
        }
        KnowledgeSource::ReactDoctor => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.85, 0.4, 0.85, 0.7, 0.8,
                0.6, 0.8, 0.7,
                0.3, 0.4, 0.2,
                0.7, 0.9, 0.9, 0.0,
            );
            cv.extend_named(&[
                ("react_lint".into(), 0.95),
                ("health_scoring".into(), 0.9),
                ("agent_skill_integration".into(), 0.85),
                ("diff_scanning".into(), 0.8),
                ("dead_code_detection".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::OpenPencil => {
            let mut cv = CapabilityVector::from_values(
                0.7, 0.8, 0.6, 0.7,
                0.5, 0.7, 0.5, 0.8,
                0.8, 0.9, 0.7, 0.8, 0.7,
                0.5, 0.6, 0.5,
                0.4, 0.8, 0.9,
                0.6, 0.9, 0.7, 0.0,
            );
            cv.extend_named(&[
                ("vector_design_canvas".into(), 0.95),
                ("mcp_design_tools".into(), 0.9),
                ("concurrent_agent_teams".into(), 0.85),
                ("design_as_code".into(), 0.9),
                ("multi_platform_export".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::AiTrader => {
            let mut cv = CapabilityVector::from_values(
                0.1, 0.1, 0.1, 0.1,
                0.3, 0.1, 0.1, 0.1,
                0.7, 0.5, 0.8, 0.6, 0.8,
                0.1, 0.1, 0.1,
                0.1, 0.1, 0.1,
                0.2, 0.3, 0.5, 0.0,
            );
            cv.extend_named(&[
                ("agent_trading".into(), 0.95),
                ("signal_sync".into(), 0.85),
                ("copy_trading".into(), 0.8),
                ("market_data_feeds".into(), 0.85),
                ("reward_system".into(), 0.7),
            ]);
            Some(cv)
        }
        KnowledgeSource::SesameRobot => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.6, 0.2, 0.7,
                0.5, 0.6, 0.5, 0.4, 0.7,
                0.2, 0.2, 0.1,
                0.1, 0.2, 0.1,
                0.2, 0.2, 0.3, 0.0,
            );
            cv.extend_named(&[
                ("esp32_firmware".into(), 0.9),
                ("quadruped_kinematics".into(), 0.85),
                ("oled_expression".into(), 0.8),
                ("servo_control".into(), 0.85),
                ("json_api".into(), 0.75),
            ]);
            Some(cv)
        }
        KnowledgeSource::EverOS => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.3, 0.3,
                0.2, 0.3, 0.2, 0.3,
                0.9, 0.6, 0.85, 0.9, 0.85,
                0.7, 0.3, 0.2,
                0.3, 0.3, 0.2,
                0.7, 0.9, 0.75, 0.8,
            );
            cv.extend_named(&[
                ("long_term_memory".into(), 0.95),
                ("hypergraph_memory".into(), 0.9),
                ("memory_extraction".into(), 0.93),
                ("self_evolution".into(), 0.9),
                ("continual_learning".into(), 0.88),
            ]);
            Some(cv)
        }
        KnowledgeSource::MattPocockSkills => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.7, 0.9, 0.85, 0.8,
                0.2, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.5, 0.6, 0.95, 0.9,
            );
            cv.extend_named(&[
                ("agent_skill_protocol".into(), 0.95),
                ("engineering_discipline".into(), 0.9),
                ("tdd_protocol".into(), 0.9),
                ("diagnostic_protocol".into(), 0.88),
                ("codebase_architecture".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::NestedLearning => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.95, 0.6, 0.9, 0.95, 0.9,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.85, 0.8, 0.8, 0.85,
            );
            cv.extend_named(&[
                ("nested_optimization".into(), 0.95),
                ("multi_timescale_update".into(), 0.92),
                ("continuum_memory".into(), 0.93),
                ("self_modifying".into(), 0.88),
                ("associative_memory".into(), 0.9),
            ]);
            Some(cv)
        }
        KnowledgeSource::AutonomousGoal => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.3, 0.2, 0.3,
                0.9, 0.85, 0.9, 0.85, 0.85,
                0.4, 0.3, 0.2,
                0.2, 0.3, 0.2,
                0.8, 0.9, 0.95, 0.85,
            );
            cv.extend_named(&[
                ("goal_lifecycle".into(), 0.95),
                ("exit_gate_detection".into(), 0.93),
                ("budget_management".into(), 0.95),
                ("session_continuity".into(), 0.9),
                ("circuit_breaker".into(), 0.88),
            ]);
            Some(cv)
        }
        KnowledgeSource::AwesomeDesignSkills => {
            let mut cv = CapabilityVector::from_values(
                0.5, 0.6, 0.4, 0.5,
                0.4, 0.5, 0.5, 0.4,
                0.85, 0.6, 0.85, 0.8, 0.8,
                0.4, 0.3, 0.2,
                0.3, 0.4, 0.3,
                0.5, 0.6, 0.7, 0.5,
            );
            cv.extend_named(&[
                ("figma_tokens".into(), 0.85),
                ("color_system".into(), 0.9),
                ("typography_scale".into(), 0.88),
                ("spacing_grid".into(), 0.85),
                ("motion_design".into(), 0.82),
                ("icon_system".into(), 0.8),
                ("component_anatomy".into(), 0.87),
                ("state_machine".into(), 0.83),
                ("accessibility_patterns".into(), 0.85),
                ("responsive_breakpoints".into(), 0.8),
                ("design_tokens".into(), 0.9),
                ("theme_engine".into(), 0.85),
                ("prototyping".into(), 0.78),
                ("design_system_docs".into(), 0.82),
                ("version_control_design".into(), 0.75),
                ("design_review".into(), 0.8),
                ("handoff".into(), 0.85),
                ("design_sprint".into(), 0.78),
                ("user_research".into(), 0.8),
                ("usability_testing".into(), 0.82),
                ("information_architecture".into(), 0.85),
                ("interaction_design".into(), 0.88),
            ]);
            Some(cv)
        }
        _ => None,
    }
}
