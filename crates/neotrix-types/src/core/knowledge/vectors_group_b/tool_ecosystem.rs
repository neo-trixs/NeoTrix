use crate::core::CapabilityVector;
use crate::core::knowledge::KnowledgeSource;

pub(super) fn handle_tool_ecosystem(s: &KnowledgeSource) -> Option<CapabilityVector> {
    match s {
        KnowledgeSource::LiteParse => {
            let mut cv = CapabilityVector::from_values(
                0.9, 0.85, 0.8, 0.75,
                0.3, 0.4, 0.5, 0.6,
                0.7, 0.65, 0.6, 0.55, 0.5,
                0.8, 0.75, 0.7,
                0.4, 0.5, 0.3,
                0.6, 0.5, 0.88, 0.82,
            );
            cv.extend_named(&[
                ("pdf_spatial_parsing".into(), 0.95),
                ("docx_xml_analysis".into(), 0.92),
                ("layout_detection".into(), 0.9),
                ("multi_format_extraction".into(), 0.88),
            ]);
            Some(cv)
        }
        KnowledgeSource::SmartSearch => {
            let mut cv = CapabilityVector::from_values(
                0.6, 0.7, 0.75, 0.8,
                0.5, 0.6, 0.4, 0.5,
                0.85, 0.8, 0.88, 0.9, 0.92,
                0.5, 0.4, 0.6,
                0.7, 0.6, 0.5,
                0.8, 0.85, 0.82, 0.88,
            );
            cv.extend_named(&[
                ("deep_research_pipeline".into(), 0.95),
                ("evidence_grading".into(), 0.92),
                ("provider_health_check".into(), 0.88),
                ("search_fallback_routing".into(), 0.9),
            ]);
            Some(cv)
        }
        KnowledgeSource::AQBot => {
            let mut cv = CapabilityVector::from_values(
                0.5, 0.6, 0.7, 0.8,
                0.6, 0.5, 0.4, 0.3,
                0.75, 0.7, 0.8, 0.85, 0.9,
                0.4, 0.5, 0.6,
                0.7, 0.6, 0.8,
                0.85, 0.8, 0.78, 0.82,
            );
            cv.extend_named(&[
                ("openai_compat_gateway".into(), 0.95),
                ("session_branching".into(), 0.9),
                ("conversation_compression".into(), 0.88),
                ("multi_key_rotation".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::AionUi => {
            let mut cv = CapabilityVector::from_values(
                0.4, 0.5, 0.6, 0.7,
                0.5, 0.4, 0.6, 0.5,
                0.7, 0.65, 0.72, 0.78, 0.8,
                0.6, 0.7, 0.5,
                0.6, 0.7, 0.5,
                0.75, 0.7, 0.72, 0.68,
            );
            cv.extend_named(&[
                ("cron_scheduler".into(), 0.95),
                ("webui_remote_access".into(), 0.9),
                ("job_persistence".into(), 0.88),
                ("interval_one_time_jobs".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::CyberVerse => {
            let mut cv = CapabilityVector::from_values(
                0.6, 0.5, 0.7, 0.8,
                0.7, 0.6, 0.8, 0.4,
                0.7, 0.65, 0.72, 0.68, 0.7,
                0.3, 0.4, 0.5,
                0.8, 0.7, 0.9,
                0.6, 0.7, 0.5, 0.4,
            );
            cv.extend_named(&[
                ("webrtc_voice_session".into(), 0.95),
                ("persona_memory".into(), 0.9),
                ("digital_human_avatar".into(), 0.85),
                ("subagent_delegation".into(), 0.88),
            ]);
            Some(cv)
        }
        KnowledgeSource::Hotpush => {
            let mut cv = CapabilityVector::from_values(
                0.5, 0.6, 0.4, 0.7,
                0.6, 0.5, 0.3, 0.4,
                0.75, 0.7, 0.8, 0.85, 0.82,
                0.4, 0.5, 0.6,
                0.7, 0.8, 0.6,
                0.85, 0.9, 0.78, 0.8,
            );
            cv.extend_named(&[
                ("topic_aggregation".into(), 0.95),
                ("trend_analysis".into(), 0.9),
                ("multi_channel_push".into(), 0.88),
                ("rule_based_filtering".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::InfiniteCanvas => {
            let mut cv = CapabilityVector::from_values(
                0.5, 0.7, 0.8, 0.6,
                0.4, 0.5, 0.7, 0.3,
                0.72, 0.68, 0.7, 0.75, 0.78,
                0.7, 0.65, 0.6,
                0.5, 0.6, 0.4,
                0.55, 0.6, 0.7, 0.65,
            );
            cv.extend_named(&[
                ("node_canvas_graph".into(), 0.95),
                ("drag_zoom_pan".into(), 0.9),
                ("node_connections".into(), 0.88),
                ("ai_image_generation".into(), 0.82),
            ]);
            Some(cv)
        }
        KnowledgeSource::AutoDocxProofread => {
            let mut cv = CapabilityVector::from_values(
                0.4, 0.5, 0.3, 0.6,
                0.3, 0.4, 0.2, 0.3,
                0.65, 0.6, 0.7, 0.72, 0.68,
                0.5, 0.4, 0.3,
                0.4, 0.5, 0.3,
                0.6, 0.5, 0.55, 0.5,
            );
            cv.extend_named(&[
                ("docx_proofreading".into(), 0.92),
                ("style_consistency".into(), 0.88),
                ("grammar_check".into(), 0.85),
                ("format_normalization".into(), 0.82),
            ]);
            Some(cv)
        }
        KnowledgeSource::OpenSwe => {
            let mut cv = CapabilityVector::from_values(
                0.6, 0.7, 0.8, 0.9,
                0.5, 0.6, 0.4, 0.5,
                0.82, 0.78, 0.85, 0.88, 0.9,
                0.3, 0.4, 0.5,
                0.8, 0.7, 0.9,
                0.7, 0.8, 0.75, 0.78,
            );
            cv.extend_named(&[
                ("sandbox_isolation".into(), 0.95),
                ("subagent_orchestration".into(), 0.92),
                ("middleware_hooks".into(), 0.9),
                ("slack_linear_integration".into(), 0.88),
            ]);
            Some(cv)
        }
        KnowledgeSource::PiMonolith => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.7, 0.92, 0.88, 0.85,
                0.4, 0.5, 0.3,
                0.3, 0.3, 0.2,
                0.6, 0.7, 0.9, 0.85,
            );
            cv.extend_named(&[
                ("lazy_skill_loading".into(), 0.95),
                ("minimalist_agent_core".into(), 0.93),
                ("skill_registration".into(), 0.9),
                ("capability_discovery".into(), 0.88),
                ("on_demand_module_load".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::ClawCode => {
            let mut cv = CapabilityVector::from_values(
                0.4, 0.4, 0.3, 0.4,
                0.3, 0.3, 0.2, 0.3,
                0.92, 0.7, 0.95, 0.9, 0.92,
                0.5, 0.6, 0.4,
                0.3, 0.4, 0.3,
                0.7, 0.8, 0.92, 0.9,
            );
            cv.extend_named(&[
                ("agent_orchestration".into(), 0.95),
                ("multi_provider_routing".into(), 0.93),
                ("tool_registry_40".into(), 0.92),
                ("provider_health_check".into(), 0.9),
                ("model_fallback".into(), 0.88),
                ("prompt_tiering".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::HermesAgent => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.7, 0.88, 0.92, 0.85,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.5, 0.6, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("llm_to_docs".into(), 0.95),
                ("self_improvement".into(), 0.93),
                ("documentation_generation".into(), 0.9),
                ("code_doc_sync".into(), 0.88),
                ("auto_docs_evolve".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::Bernstein => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.3, 0.2, 0.2, 0.2,
                0.88, 0.65, 0.9, 0.85, 0.8,
                0.3, 0.4, 0.3,
                0.2, 0.2, 0.2,
                0.85, 0.7, 0.85, 0.8,
            );
            cv.extend_named(&[
                ("deterministic_scheduling".into(), 0.95),
                ("orchestrator_pattern".into(), 0.93),
                ("step_execution".into(), 0.9),
                ("workflow_definition".into(), 0.88),
                ("subtask_decomposition".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::Mastra => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.88, 0.7, 0.9, 0.92, 0.85,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.9, 0.85,
            );
            cv.extend_named(&[
                ("observer_pattern".into(), 0.95),
                ("reflector_semantic_compression".into(), 0.92),
                ("event_driven".into(), 0.9),
                ("execution_trace".into(), 0.88),
                ("semantic_compression".into(), 0.85),
            ]);
            Some(cv)
        }
        KnowledgeSource::Omi => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.4, 0.2, 0.5,
                0.7, 0.5, 0.75, 0.7, 0.6,
                0.2, 0.2, 0.1,
                0.1, 0.2, 0.1,
                0.3, 0.4, 0.5, 0.4,
            );
            cv.extend_named(&[
                ("ble_wearable".into(), 0.95),
                ("hardware_agent".into(), 0.93),
                ("bluetooth_communication".into(), 0.9),
                ("low_power_protocol".into(), 0.88),
                ("embedded_llm".into(), 0.8),
            ]);
            Some(cv)
        }
        KnowledgeSource::Crush => {
            let mut cv = CapabilityVector::from_values(
                0.3, 0.3, 0.3, 0.3,
                0.2, 0.2, 0.2, 0.4,
                0.8, 0.6, 0.85, 0.8, 0.7,
                0.3, 0.4, 0.3,
                0.2, 0.3, 0.2,
                0.75, 0.6, 0.7, 0.6,
            );
            cv.extend_named(&[
                ("tui_interface".into(), 0.95),
                ("lsp_integration".into(), 0.93),
                ("mcp_protocol".into(), 0.9),
                ("go_runtime".into(), 0.85),
                ("terminal_ui".into(), 0.88),
            ]);
            Some(cv)
        }
        _ => None,
    }
}
