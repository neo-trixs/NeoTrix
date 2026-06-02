#[cfg(test)]
mod tests {
    use crate::neotrix::nt_world_model::TaskType;
    use crate::neotrix::nt_mind::{
        ReasoningBank, ReasoningMemory,
    };
    use crate::neotrix::nt_mind::memory::{MemoryLifecycle, MemoryTier};
    use crate::neotrix::nt_mind::self_edit::MicroEdit;
    use chrono::Utc;

    #[test]
    fn test_memory_tier_promotion() {
        assert_eq!(MemoryTier::Working.promote(), Some(MemoryTier::Episodic));
        assert_eq!(MemoryTier::Episodic.promote(), Some(MemoryTier::Semantic));
        assert_eq!(MemoryTier::Semantic.promote(), Some(MemoryTier::Procedural));
        assert_eq!(MemoryTier::Procedural.promote(), None);
        assert_eq!(MemoryTier::Working.as_str(), "working");
        assert_eq!(MemoryTier::Procedural.as_str(), "procedural");
    }

    #[test]
    fn test_memory_lifecycle_ttl() {
        let mut life = MemoryLifecycle::with_ttl(0.8, 3600);
        assert!(!life.is_expired());
        life.created_at = 0;
        assert!(life.is_expired());
        life.touch();
        assert_eq!(life.access_count, 1);
    }

    #[test]
    fn test_reasoning_memory_has_tier_and_lifecycle() {
        let mem = ReasoningMemory::new("test task", TaskType::CodeGeneration, &[], 0.75);
        assert_eq!(mem.tier.as_str(), "working");
        assert!(mem.lifecycle.importance > 0.0);
    }

    #[test]
    fn test_prune_expired() {
        let mut bank = ReasoningBank::new(10);
        let micro_edits = vec![MicroEdit::AdjustDimension("typography".to_string(), 0.1)];

        let m1 = ReasoningMemory::new("keep me", TaskType::UIDesign, &micro_edits, 0.9)
            .with_ttl(999999);
        bank.store(m1);

        let m2 = ReasoningMemory::new("expire me", TaskType::CodeReview, &micro_edits, 0.8)
            .with_ttl(0);
        bank.store(m2);

        assert_eq!(bank.stats().total_memories, 2);

        let now = Utc::now().timestamp() + 1;
        let pruned = bank.prune_expired(now);
        assert_eq!(pruned, 1);

        let stats = bank.stats();
        assert_eq!(stats.total_memories, 1);
        assert!(bank.memories().iter().all(|m| m.task_description == "keep me"));
    }

    #[test]
    fn test_memory_lifecycle_confidence() {
        let lifecycle = MemoryLifecycle::new(0.8);
        assert!((lifecycle.confidence - 0.8).abs() < 0.001);
        let lifecycle = lifecycle.with_confidence(0.95);
        assert!((lifecycle.confidence - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_prune_expired_keeps_non_expired() {
        let mut bank = ReasoningBank::new(10);
        let micro_edits = vec![MicroEdit::AdjustDimension("typography".to_string(), 0.1)];
        let m1 = ReasoningMemory::new("keep me", TaskType::UIDesign, &micro_edits, 0.9)
            .with_ttl(999999);
        bank.store(m1);
        let m2 = ReasoningMemory::new("also keep", TaskType::CodeReview, &micro_edits, 0.8);
        bank.store(m2);
        assert_eq!(bank.stats().total_memories, 2);
        let pruned = bank.prune_expired(Utc::now().timestamp());
        assert_eq!(pruned, 0);
        assert_eq!(bank.stats().total_memories, 2);
    }

    #[test]
    fn test_iterate_memories_with_tier_and_expiry() {
        let mut bank = ReasoningBank::new(100);
        bank.store(ReasoningMemory::new("task 1", TaskType::CodeGeneration, &[], 0.9));
        bank.store(ReasoningMemory::new("task 2", TaskType::UIDesign, &[], 0.8));
        bank.store(ReasoningMemory::new("task 3", TaskType::CodeReview, &[], 0.95));

        let _result = bank.iterate_memories(0.8, 0.3);
    }
}
