#[cfg(test)]
mod tests {
    use crate::neotrix::nt_mind::{
        ReasoningBrain, KnowledgeSource, CapabilityVector,
    };

    #[test]
    fn test_nt_mind_creation() {
        let brain = ReasoningBrain::new();
        assert_eq!(brain.total_absorb_count, 0);
    }

    #[test]
    fn test_absorb_knowledge() {
        let mut brain = ReasoningBrain::new();

        brain.absorb(KnowledgeSource::HeroUI);
        assert_eq!(brain.total_absorb_count, 1);

        brain.absorb(KnowledgeSource::BaseUI);
        assert_eq!(brain.total_absorb_count, 2);

        assert!(brain.capability.compound_composition() > 0.0);
        assert!(brain.capability.accessibility() > 0.0);
    }

    #[test]
    fn test_evaluate_capability() {
        let mut brain = ReasoningBrain::new();

        let initial_compound = brain.capability.compound_composition();
        let initial_accessibility = brain.capability.accessibility();

        brain.absorb_batch(&[
            KnowledgeSource::HeroUI,
            KnowledgeSource::BaseUI,
        ]);

        assert!(brain.capability.compound_composition() > initial_compound);
        assert!(brain.capability.accessibility() > initial_accessibility);
    }

    #[test]
    fn test_capability_vector_similarity() {
        let mut v1 = CapabilityVector::default();
        v1.set_typography(0.8);
        v1.set_grid(0.7);
        v1.set_color(0.6);

        let mut v2 = CapabilityVector::default();
        v2.set_typography(0.7);
        v2.set_grid(0.8);
        v2.set_color(0.5);

        let sim = v1.similarity(&v2);
        assert!(sim > 0.0 && sim <= 1.0);
    }

    #[test]
    fn test_brain_persistence() {
        let test_id = format!("neotrix_test_persist_{}", std::process::id());
        let temp_dir = std::env::temp_dir().join(&test_id);
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let mut brain = ReasoningBrain::new();
        let initial_typography = brain.capability.typography();

        brain.absorb(KnowledgeSource::HeroUI);
        brain.absorb(KnowledgeSource::BaseUI);

        assert!(brain.capability.typography() > initial_typography);
        assert_eq!(brain.total_absorb_count, 2);

        let save_result = brain.save_to_dir(Some(&temp_dir));
        assert!(save_result.is_ok());

        let metadata_path = temp_dir.join("brain_metadata.json");
        assert!(metadata_path.exists(), "元数据文件不存在: {:?}", metadata_path);

        let load_result = ReasoningBrain::load_from_dir(Some(&temp_dir));
        assert!(load_result.is_ok(), "加载失败: {:?}", load_result.err());

        let loaded_brain = load_result.expect("load_result should be ok in test");
        assert_eq!(loaded_brain.total_absorb_count, 2);
        assert!((loaded_brain.capability.typography() - brain.capability.typography()).abs() < 0.001);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multi_source_absorb() {
        let mut brain = ReasoningBrain::new();

        let initial_absorb_count = brain.total_absorb_count;
        let initial_typography = brain.capability.typography();
        let initial_accessibility = brain.capability.accessibility();

        let sources = vec![
            KnowledgeSource::HeroUI,
            KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI,
            KnowledgeSource::CortexUI,
            KnowledgeSource::AgenticDS,
        ];

        brain.absorb_batch(&sources);

        assert_eq!(brain.total_absorb_count, initial_absorb_count + 5);

        assert!(brain.capability.compound_composition() > 0.0);
        assert!(brain.capability.accessibility() > 0.0);

        assert!(brain.capability.typography() > initial_typography ||
                brain.capability.accessibility() > initial_accessibility);
    }

    #[test]
    fn test_capability_evolution() {
        let mut brain = ReasoningBrain::new();

        let initial_capability = brain.capability.clone();

        brain.absorb(KnowledgeSource::HeroUI);
        let after_heroui = brain.capability.clone();

        assert!(after_heroui.compound_composition() > initial_capability.compound_composition());
        assert!(after_heroui.tailwind_proficiency() > initial_capability.tailwind_proficiency());

        brain.absorb(KnowledgeSource::BaseUI);
        let after_baseui = brain.capability.clone();

        assert!(after_baseui.accessibility() > after_heroui.accessibility());
        assert!(after_baseui.react_aria_usage() > after_heroui.react_aria_usage());

        assert!(after_baseui.compound_composition() >= after_heroui.compound_composition() - 0.001);
    }
}
