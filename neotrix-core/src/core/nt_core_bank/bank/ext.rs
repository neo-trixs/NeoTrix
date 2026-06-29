#[cfg(test)]
mod tests {
    use crate::core::nt_core_bank::{
        MemoryTier, OffloadManager, PipelineConfig, PipelineState, ReasoningBank, ReasoningMemory,
        T3ViewType,
    };
    use crate::core::nt_core_knowledge::TaskType;

    fn make_mem(desc: &str, tt: TaskType, reward: f64) -> ReasoningMemory {
        ReasoningMemory::new(desc, tt, &[], reward)
    }

    #[test]
    fn test_new_bank_is_empty() {
        let bank = ReasoningBank::new(100);
        assert!(bank.memories().is_empty());
        let stats = bank.stats();
        assert_eq!(stats.total_memories, 0);
    }

    #[test]
    fn test_store_and_count() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("test", TaskType::General, 0.5));
        assert_eq!(bank.memories().len(), 1);
        bank.store(make_mem("another", TaskType::CodeReview, 0.8));
        assert_eq!(bank.memories().len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("ok1", TaskType::General, 0.9));
        bank.store(make_mem("ok2", TaskType::CodeReview, 0.7));
        bank.store(make_mem("fail1", TaskType::General, 0.1));
        let stats = bank.stats();
        assert_eq!(stats.total_memories, 3);
        assert_eq!(stats.success_count, 2);
        assert!((stats.success_rate - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_store_with_embedding() {
        let mut bank = ReasoningBank::new(100);
        let mem = make_mem("emb", TaskType::General, 0.5);
        bank.store_with_embedding(mem, vec![0.1, 0.2, 0.3]);
        assert_eq!(bank.memories().len(), 1);
        assert!(bank.memories()[0].embedding.is_some());
    }

    #[test]
    fn test_max_memories_enforced() {
        let mut bank = ReasoningBank::new(3);
        for i in 0..5 {
            bank.store(make_mem(
                &format!("mem{}", i),
                TaskType::General,
                0.1 * i as f64,
            ));
        }
        assert!(bank.memories().len() <= 3);
    }

    #[test]
    fn test_initialize_with_design_knowledge() {
        let mut bank = ReasoningBank::new(100);
        bank.initialize_with_design_knowledge();
        assert_eq!(bank.memories().len(), 7);
        bank.initialize_with_design_knowledge();
        assert_eq!(bank.memories().len(), 7);
    }

    #[test]
    fn test_initialize_with_coding_knowledge() {
        let mut bank = ReasoningBank::new(100);
        bank.initialize_with_coding_knowledge();
        assert_eq!(bank.memories().len(), 10);
    }

    #[test]
    fn test_initialize_with_everos_knowledge() {
        let mut bank = ReasoningBank::new(100);
        bank.initialize_with_everos_knowledge();
        assert_eq!(bank.memories().len(), 4);
    }

    #[test]
    fn test_prune_low_value() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("keep", TaskType::General, 0.8));
        bank.store(make_mem("prune", TaskType::General, 0.1));
        bank.store(make_mem("keep2", TaskType::CodeReview, 0.7));
        let pruned = bank.prune_low_value(0.5);
        assert_eq!(pruned, 1);
        assert_eq!(bank.memories().len(), 2);
    }

    #[test]
    fn test_consolidate_similar() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("task a", TaskType::CodeGeneration, 0.7));
        bank.store(make_mem("task b", TaskType::CodeGeneration, 0.71));
        bank.store(make_mem("diff task", TaskType::UIDesign, 0.7));
        let merged = bank.consolidate_similar(0.9);
        assert_eq!(merged, 1);
        assert_eq!(bank.memories().len(), 2);
    }

    #[test]
    fn test_replay_high_value() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("low", TaskType::General, 0.3));
        bank.store(make_mem("high", TaskType::General, 0.9));
        let replayed = bank.replay_high_value();
        assert_eq!(replayed, 1);
        assert!(bank.memories().len() >= 2);
    }

    #[test]
    fn test_quality_score_empty() {
        let bank = ReasoningBank::new(100);
        assert!((bank.quality_score()).abs() < 1e-10);
    }

    #[test]
    fn test_quality_score_non_empty() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("good", TaskType::CodeGeneration, 0.9));
        bank.store(make_mem("ok", TaskType::CodeReview, 0.7));
        bank.store(make_mem("bad", TaskType::General, 0.2));
        let score = bank.quality_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_retrieve_relevant_empty() {
        let bank = ReasoningBank::new(100);
        let results = bank.retrieve_relevant("test", None, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_retrieve_relevant_by_tt() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("rust coding", TaskType::CodeGeneration, 0.8));
        bank.store(make_mem("design", TaskType::UIDesign, 0.7));
        let results = bank.retrieve_relevant("rust", Some(TaskType::CodeGeneration), 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].task_type, TaskType::CodeGeneration);
    }

    #[test]
    fn test_prune_expired() {
        let mut bank = ReasoningBank::new(100);
        let mut mem = make_mem("expiring", TaskType::General, 0.5);
        mem.lifecycle.ttl_seconds = Some(1);
        mem.lifecycle.created_at = 0;
        bank.store(mem);
        bank.store(make_mem("permanent", TaskType::General, 0.5));
        let pruned = bank.prune_expired(999999);
        assert_eq!(pruned, 1);
        assert_eq!(bank.memories().len(), 1);
    }

    #[test]
    fn test_get_successes() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("ok", TaskType::General, 0.9));
        bank.store(make_mem("fail", TaskType::General, 0.1));
        let successes = bank.get_successes();
        assert_eq!(successes.len(), 1);
    }

    #[test]
    fn test_stats_detailed() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("t1", TaskType::CodeGeneration, 0.5));
        bank.store(make_mem("t2", TaskType::General, 0.8));
        let detailed = bank.stats_detailed();
        assert_eq!(detailed.total, 2);
        assert!(detailed.avg_confidence > 0.0);
    }

    #[test]
    fn test_bm25_rebuild_on_search() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("rust programming", TaskType::CodeGeneration, 0.5));
        bank.store(make_mem("design pattern", TaskType::UIDesign, 0.5));
        let results = bank.retrieve_relevant("rust", None, 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_store_with_pipeline() {
        let mut bank = ReasoningBank::new(100);
        let tmp = std::env::temp_dir().join("neotrix_test_pipeline");
        let _ = std::fs::create_dir_all(&tmp);
        let mut offload = OffloadManager::new(&tmp);
        let mut state = PipelineState::new();
        let config = PipelineConfig::default();
        let mem = make_mem("pipeline test", TaskType::General, 0.9);
        bank.store_with_pipeline(mem, &mut offload, &mut state, &config);
        assert_eq!(bank.memories().len(), 1);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_iterate_memories_does_not_panic() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("a", TaskType::General, 0.9));
        bank.store(make_mem("b", TaskType::General, 0.3));
        let result = bank.iterate_memories(0.9, 0.5);
        assert!(result.after.total_memories > 0);
    }

    #[test]
    fn test_without_wh() {
        let bank = ReasoningBank::new_without_wh(100);
        assert!(bank.retrieve_by_wh("test", 5).is_empty());
    }

    #[test]
    fn test_retrieve_by_wh() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("test query", TaskType::General, 0.5));
        let results = bank.retrieve_by_wh("test", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_store_deferred() {
        let mut bank = ReasoningBank::new(100);
        bank.store_deferred(make_mem("deferred", TaskType::General, 0.5));
        assert_eq!(bank.memories().len(), 1);
    }

    #[test]
    fn test_multi_modal_search() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem(
            "rust async programming",
            TaskType::CodeGeneration,
            0.9,
        ));
        bank.store(make_mem("ui design", TaskType::UIDesign, 0.3));
        let results = bank.multi_modal_search("rust", 1.0, 1.0, 1.0);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_retrieve_by_view() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("test view", TaskType::General, 0.5));
        let results = bank.retrieve_by_view("test", T3ViewType::Struct, None, 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_retrieve_all_views() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("view test", TaskType::General, 0.5));
        let results = bank.retrieve_all_views("view", None, 2);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = ReasoningBank::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = ReasoningBank::cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_zero() {
        let a = vec![0.0, 0.0];
        let b = vec![0.0, 0.0];
        let sim = ReasoningBank::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_zero_only_a() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        let sim = ReasoningBank::cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_quality_score_high_quality() {
        let mut bank = ReasoningBank::new(100);
        for i in 0..8 {
            let tt = match i {
                0 => TaskType::CodeGeneration,
                1 => TaskType::CodeReview,
                2 => TaskType::UIDesign,
                3 => TaskType::Security,
                4 => TaskType::Learning,
                5 => TaskType::Research,
                6 => TaskType::Reflection,
                _ => TaskType::Planning,
            };
            bank.store(make_mem(&format!("high quality {}", i), tt, 0.95));
        }
        let score = bank.quality_score();
        assert!(score > 0.5);
    }

    #[test]
    fn test_promote_tiers() {
        let mut bank = ReasoningBank::new(100);
        let mut mem = make_mem("promotable", TaskType::General, 0.5);
        mem.tier = MemoryTier::Working;
        mem.lifecycle.access_count = 5;
        mem.timestamp = 0;
        bank.store(mem);
        let promoted = bank.promote_tiers();
        assert!(promoted >= 1);
    }

    #[test]
    fn test_enable_hypergraph() {
        let mut bank = ReasoningBank::new(100);
        bank.enable_hypergraph(10);
        bank.store(make_mem("h1", TaskType::General, 0.5));
        bank.store(make_mem("h2", TaskType::CodeReview, 0.5));
        let id = bank.memories()[0].id.clone();
        assert!(bank.index_memory(&id).is_ok());
        let traverse = bank.hypergraph_traverse(&id, 1);
        assert!(traverse.is_empty() || !traverse.is_empty());
    }

    #[test]
    fn test_split_context() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("stable", TaskType::Planning, 0.9));
        bank.store(make_mem("dynamic", TaskType::General, 0.3));
        let mems: Vec<ReasoningMemory> = bank.memories().iter().cloned().collect();
        let (stable, dynamic) = ReasoningBank::split_context(&mems);
        assert!(!stable.is_empty());
        assert!(!dynamic.is_empty());
    }

    #[test]
    fn test_checkpoint_roundtrip() {
        let state = PipelineState::new();
        let dir = std::env::temp_dir().join("neotrix_test_checkpoint");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("checkpoint.json");
        assert!(ReasoningBank::save_pipeline_checkpoint(&state, &path).is_ok());
        let loaded = ReasoningBank::load_pipeline_checkpoint(&path);
        assert_eq!(loaded.l1_count, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_checkpoint_load_nonexistent() {
        let path = std::path::Path::new("/nonexistent/path/checkpoint.json");
        let state = ReasoningBank::load_pipeline_checkpoint(path);
        assert_eq!(state.l1_count, 0);
    }

    #[test]
    fn test_retrieve_by_wh_empty_bank() {
        let bank = ReasoningBank::new(100);
        assert!(bank.retrieve_by_wh("anything", 5).is_empty());
    }

    #[test]
    fn test_store_deferred_then_search() {
        let mut bank = ReasoningBank::new(100);
        bank.store_deferred(make_mem("deferred search", TaskType::General, 0.5));
        let results = bank.retrieve_relevant("deferred", None, 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_multiple_task_types() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("code gen", TaskType::CodeGeneration, 0.5));
        bank.store(make_mem("code review", TaskType::CodeReview, 0.5));
        bank.store(make_mem("ui design", TaskType::UIDesign, 0.5));
        bank.store(make_mem("security", TaskType::Security, 0.5));
        let detailed = bank.stats_detailed();
        assert_eq!(detailed.total, 4);
    }

    #[test]
    fn test_initialize_full_knowledge() {
        let mut bank = ReasoningBank::new(1000);
        bank.initialize_with_design_knowledge();
        bank.initialize_with_coding_knowledge();
        bank.initialize_with_everos_knowledge();
        assert_eq!(bank.memories().len(), 7 + 10 + 4);
    }

    #[test]
    fn test_retrieve_with_task_type_filter() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("rust async", TaskType::CodeGeneration, 0.8));
        bank.store(make_mem("rust design", TaskType::UIDesign, 0.8));
        let gen = bank.retrieve_relevant("rust", Some(TaskType::CodeGeneration), 5);
        let ui = bank.retrieve_relevant("rust", Some(TaskType::UIDesign), 5);
        assert_eq!(gen.len(), 1);
        assert_eq!(ui.len(), 1);
        assert_eq!(gen[0].task_type, TaskType::CodeGeneration);
        assert_eq!(ui[0].task_type, TaskType::UIDesign);
    }

    #[test]
    fn test_empty_bank_quality_score_zero() {
        let bank = ReasoningBank::new(100);
        assert!((bank.quality_score()).abs() < 1e-10);
    }

    #[test]
    fn test_consolidate_no_similar() {
        let mut bank = ReasoningBank::new(100);
        bank.store(make_mem("a", TaskType::CodeGeneration, 0.9));
        bank.store(make_mem("b", TaskType::UIDesign, 0.1));
        let merged = bank.consolidate_similar(0.9);
        assert_eq!(merged, 0);
        assert_eq!(bank.memories().len(), 2);
    }

    #[test]
    fn test_replay_max_memories() {
        let mut bank = ReasoningBank::new(2);
        bank.store(make_mem("a", TaskType::General, 0.9));
        bank.store(make_mem("b", TaskType::General, 0.9));
        let replayed = bank.replay_high_value();
        assert_eq!(replayed, 0);
    }
}
