#[cfg(test)]
mod tests {
    use crate::neotrix::nt_world_model::TaskType;
    use crate::neotrix::nt_mind::{
        ReasoningBank, ReasoningMemory,
    };
    use crate::neotrix::nt_mind::self_edit::MicroEdit;

    #[test]
    fn test_bm25_basic_search() {
        use crate::neotrix::nt_mind::bm25::{Bm25Index, Bm25Document};
        let docs = vec![
            Bm25Document { id: "1".into(), text: "Rust memory safety ownership borrowing lifetimes".into() },
            Bm25Document { id: "2".into(), text: "async await tokio async runtime concurrency".into() },
            Bm25Document { id: "3".into(), text: "React hooks useState useEffect component lifecycle".into() },
        ];
        let index = Bm25Index::build(&docs);
        let results = index.search("async concurrency", 3);
        assert!(!results.is_empty(), "should find async docs");
        assert_eq!(results[0].1, "2", "doc 2 should be top for async concurrency");
    }

    #[test]
    fn test_bm25_empty_index() {
        use crate::neotrix::nt_mind::bm25::Bm25Index;
        let index = Bm25Index::empty();
        let results = index.search("anything", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_fusion() {
        use crate::neotrix::nt_mind::bm25::rrf_fuse;
        let v1: Vec<(f64, String)> = vec![(0.9, "a".into()), (0.8, "b".into()), (0.7, "c".into())];
        let v2: Vec<(f64, String)> = vec![(0.95, "b".into()), (0.85, "a".into()), (0.6, "d".into())];
        let fused = rrf_fuse(&[v1, v2]);
        assert!(!fused.is_empty());
        let top = fused[0].1.clone();
        assert!(top == "a" || top == "b", "a or b should be top, got {}", top);
    }

    #[test]
    fn test_bank_bm25_hybrid_retrieval() {
        let mut bank = ReasoningBank::new(100);
        let mem1 = ReasoningMemory::new("async Rust tokio concurrency", TaskType::CodeGeneration, &[], 0.9);
        let mem2 = ReasoningMemory::new("React hooks useState", TaskType::UIDesign, &[], 0.8);
        let mem3 = ReasoningMemory::new("Python async asyncio event loop", TaskType::CodeGeneration, &[], 0.7);
        bank.store(mem1);
        bank.store(mem2);
        bank.store(mem3);

        let results = bank.retrieve_relevant("async concurrency", None, 5);
        assert!(!results.is_empty(), "BM25 hybrid should find async docs");
        let descs: Vec<&str> = results.iter().map(|m| m.task_description.as_str()).collect();
        let any_async = descs.iter().any(|d| d.contains("async"));
        assert!(any_async, "should contain async-related memory");
    }

    #[test]
    fn test_bank_bm25_type_filtered() {
        let mut bank = ReasoningBank::new(100);
        let mem1 = ReasoningMemory::new("Rust ownership borrow checker", TaskType::CodeReview, &[], 0.9);
        let mem2 = ReasoningMemory::new("design system color palette", TaskType::UIDesign, &[], 0.85);
        bank.store(mem1);
        bank.store(mem2);

        let results = bank.retrieve_relevant("ownership", Some(TaskType::CodeReview), 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].task_type, TaskType::CodeReview);
    }

    #[test]
    fn test_multi_modal_search() {
        let mut bank = ReasoningBank::new(10);
        let micro_edits = vec![MicroEdit::AdjustDimension("typography".to_string(), 0.1)];

        let mut m1 = ReasoningMemory::new("async tokio runtime concurrency", TaskType::CodeGeneration, &micro_edits, 0.9);
        m1.embedding = Some(vec![1.0, 0.0, 0.0]);
        bank.store(m1);

        let mut m2 = ReasoningMemory::new("memory safety ownership", TaskType::CodeReview, &micro_edits, 0.8);
        m2.embedding = Some(vec![0.0, 1.0, 0.0]);
        bank.store(m2);

        let mut m3 = ReasoningMemory::new("React hooks useEffect", TaskType::UIDesign, &micro_edits, 0.7);
        m3.embedding = Some(vec![0.0, 0.0, 1.0]);
        bank.store(m3);

        let results = bank.multi_modal_search("async tokio concurrency", 1.0, 0.8, 0.5);
        assert!(!results.is_empty(), "should find async-related memories");
        assert!(results[0].task_description.contains("async"), "top result should match async query");
    }

    #[test]
    fn test_graph_search_by_entities() {
        let mut bank = ReasoningBank::new(10);
        let micro_edits = vec![MicroEdit::AdjustDimension("typography".to_string(), 0.1)];
        bank.store(ReasoningMemory::new("async tokio runtime concurrency", TaskType::CodeGeneration, &micro_edits, 0.9));
        bank.store(ReasoningMemory::new("Rust memory safety ownership borrowing", TaskType::CodeReview, &micro_edits, 0.8));
        bank.store(ReasoningMemory::new("React hooks useState useEffect", TaskType::UIDesign, &micro_edits, 0.7));
        let results = bank.multi_modal_search("memory ownership safety", 0.5, 0.5, 1.0);
        assert!(!results.is_empty(), "graph search should find memory-related results");
    }
}
