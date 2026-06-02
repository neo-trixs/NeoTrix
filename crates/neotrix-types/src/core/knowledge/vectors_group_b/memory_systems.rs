use crate::core::CapabilityVector;
use crate::core::knowledge::KnowledgeSource;

pub(super) fn handle_memory_systems(s: &KnowledgeSource) -> Option<CapabilityVector> {
    match s {
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
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
            Some(cv)
        }
        KnowledgeSource::ZepMemory => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.92, 0.6, 0.9, 0.88, 0.85,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.65, 0.7, 0.9, 0.88,
            );
            cv.extend_named(&[
                ("temporal_knowledge_graph".into(), 0.95),
                ("entity_relationship_tracking".into(), 0.93),
                ("non_lossy_incremental_update".into(), 0.94),
                ("provenance_maintenance".into(), 0.9),
                ("community_subgraph_summary".into(), 0.88),
                ("temporal_reasoning".into(), 0.92),
                ("cross_session_synthesis".into(), 0.91),
                ("graphiti_engine".into(), 0.95),
            ]);
            Some(cv)
        }
        KnowledgeSource::HindsightMemory => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.93, 0.65, 0.92, 0.9, 0.88,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.7, 0.75, 0.92, 0.9,
            );
            cv.extend_named(&[
                ("multi_strategy_retrieval".into(), 0.96),
                ("semantic_vector_search".into(), 0.93),
                ("bm25_keyword_match".into(), 0.9),
                ("graph_entity_traversal".into(), 0.92),
                ("temporal_range_filtering".into(), 0.88),
                ("cross_encoder_reranking".into(), 0.94),
                ("rrf_fusion".into(), 0.91),
                ("retain_recall_reflect".into(), 0.95),
                ("biomimetic_memory_banks".into(), 0.93),
            ]);
            Some(cv)
        }
        KnowledgeSource::CogneeMemory => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.85, 0.6, 0.88, 0.85, 0.82,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.65, 0.85, 0.82,
            );
            cv.extend_named(&[
                ("pre_build_knowledge_graph".into(), 0.95),
                ("graph_rag_engine".into(), 0.93),
                ("multi_modal_ingestion".into(), 0.9),
                ("entity_extraction".into(), 0.92),
                ("relationship_inference".into(), 0.88),
                ("graph_vector_hybrid".into(), 0.91),
            ]);
            Some(cv)
        }
        KnowledgeSource::SageMemory => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.3,
                0.9, 0.6, 0.88, 0.85, 0.85,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.65, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("self_evolving_graph_memory".into(), 0.96),
                ("memory_writer_reader_loop".into(), 0.95),
                ("graph_foundation_model".into(), 0.93),
                ("evidence_chain_recovery".into(), 0.92),
                ("structural_gating".into(), 0.88),
                ("context_schema_decomposition".into(), 0.9),
                ("evolution_stability_bounds".into(), 0.87),
            ]);
            Some(cv)
        }
        KnowledgeSource::ApexMem => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.9, 0.55, 0.88, 0.85, 0.82,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("property_graph_ontology".into(), 0.95),
                ("append_only_temporal_store".into(), 0.94),
                ("graph_qna_agent".into(), 0.93),
                ("entity_linking_resolution".into(), 0.92),
                ("graph_sql_temporal_traversal".into(), 0.9),
                ("hybrid_search_retrieval".into(), 0.88),
                ("conflict_resolution_query_time".into(), 0.91),
                ("temporal_validity_intervals".into(), 0.89),
            ]);
            Some(cv)
        }
        KnowledgeSource::LangMem => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.3,
                0.2, 0.2, 0.2, 0.3,
                0.85, 0.7, 0.85, 0.88, 0.82,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.6, 0.7, 0.85, 0.82,
            );
            cv.extend_named(&[
                ("procedural_memory".into(), 0.95),
                ("agent_self_edit_prompt".into(), 0.94),
                ("semantic_fact_extraction".into(), 0.92),
                ("episodic_memory".into(), 0.88),
                ("background_compaction".into(), 0.85),
                ("supersession_dedup".into(), 0.9),
                ("namespace_memory_organization".into(), 0.87),
            ]);
            Some(cv)
        }
        KnowledgeSource::LettaMemory => {
            let mut cv = CapabilityVector::from_values(
                0.2, 0.2, 0.2, 0.2,
                0.2, 0.2, 0.2, 0.2,
                0.88, 0.6, 0.9, 0.85, 0.8,
                0.3, 0.2, 0.2,
                0.2, 0.2, 0.2,
                0.65, 0.7, 0.88, 0.85,
            );
            cv.extend_named(&[
                ("os_inspired_memory_tiers".into(), 0.95),
                ("core_recall_archival".into(), 0.94),
                ("agent_managed_memory".into(), 0.93),
                ("memory_block_self_edit".into(), 0.92),
                ("continual_learning".into(), 0.9),
                ("model_agnostic_runtime".into(), 0.88),
            ]);
            Some(cv)
        }
        _ => None,
    }
}
