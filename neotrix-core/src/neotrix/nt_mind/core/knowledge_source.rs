//! Reverse bridge: V1 → core/
//! All types re-exported from `crate::core::knowledge`.

pub use crate::core::nt_core_knowledge::{
    AbsorptionRecord, KnowledgeProvider, KnowledgeSource, MaturityLevel, RewardSource, TaskType,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_source_name_known_variant() {
        let name = KnowledgeSource::HeroUI.name();
        assert!(!name.is_empty());
        assert_eq!(name, "heroui-inc/heroui");
    }

    #[test]
    fn test_knowledge_source_name_fallback() {
        let name = KnowledgeSource::HyperAgents.name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_knowledge_source_source_weight_range() {
        let sources = KnowledgeSource::all();
        for src in &sources {
            let w = src.source_weight();
            assert!(w > 0.0 && w <= 1.0, "source_weight for {:?} = {}", src, w);
        }
    }

    #[test]
    fn test_knowledge_source_all_nonempty() {
        let all = KnowledgeSource::all();
        assert!(!all.is_empty());
        assert!(all.len() > 50);
    }

    #[test]
    fn test_knowledge_source_capability_vector_has_positive_dims() {
        let cv = KnowledgeSource::HeroUI.capability_vector();
        assert!(cv.arr().iter().any(|&v| v > 0.0));
        assert_eq!(cv.dim(), 23);
    }

    #[test]
    fn test_knowledge_source_capability_vector_extension() {
        let cv = KnowledgeSource::MemOS.capability_vector();
        assert!(!cv.extension().is_empty());
    }

    #[test]
    fn test_knowledge_provider_trait_coherence() {
        let src = KnowledgeSource::BaseUI;
        assert_eq!(KnowledgeProvider::name(&src), KnowledgeSource::name(&src));
        assert_eq!(
            KnowledgeProvider::source_weight(&src),
            KnowledgeSource::source_weight(&src)
        );
    }

    #[test]
    fn test_task_type_variants() {
        assert_ne!(TaskType::Design as u8, TaskType::CodeAnalysis as u8);
        assert_ne!(TaskType::General as u8, TaskType::Reflection as u8);
    }

    #[test]
    fn test_reward_source_priority_multiplier() {
        assert!((RewardSource::External.priority_multiplier() - 2.0).abs() < 1e-9);
        assert!((RewardSource::Internal.priority_multiplier() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_maturity_level_ordering() {
        assert!(MaturityLevel::Candidate < MaturityLevel::Reviewed);
        assert!(MaturityLevel::Reviewed < MaturityLevel::Validated);
        assert!(MaturityLevel::Validated < MaturityLevel::GroundTruth);
    }

    #[test]
    fn test_maturity_level_promote_chain() {
        assert_eq!(
            MaturityLevel::Candidate.promote(),
            Some(MaturityLevel::Reviewed)
        );
        assert_eq!(
            MaturityLevel::Reviewed.promote(),
            Some(MaturityLevel::Validated)
        );
        assert_eq!(
            MaturityLevel::Validated.promote(),
            Some(MaturityLevel::GroundTruth)
        );
        assert_eq!(MaturityLevel::GroundTruth.promote(), None);
    }

    #[test]
    fn test_maturity_level_confidence() {
        assert!((MaturityLevel::Candidate.confidence() - 0.25).abs() < 1e-9);
        assert!((MaturityLevel::Reviewed.confidence() - 0.5).abs() < 1e-9);
        assert!((MaturityLevel::Validated.confidence() - 0.75).abs() < 1e-9);
        assert!((MaturityLevel::GroundTruth.confidence() - 1.0).abs() < 1e-9);
    }
}
