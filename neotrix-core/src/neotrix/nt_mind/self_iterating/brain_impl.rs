// brain_impl.rs — shim re-exporting from split files + tests

pub use super::brain_core::{ReasoningBrain, AbsorbValidator, DefaultAbsorbValidator, SelfIteration};
pub(crate) use super::brain_core::BrainMetadata;
pub use super::brain_ewc::{FisherMatrix, WeightUpdateRecord, RLAlgorithm, EvaluationRecord};
pub use super::brain_seal::{SealEditStrategy, DefaultSealStrategy, CapabilityDelta, ConservativeSealStrategy, AggressiveSealStrategy};

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::core::KnowledgeSource;

    #[test]
    fn test_absorb_tracks_access() {
        let mut brain = ReasoningBrain::new();
        let source = KnowledgeSource::HeroUI;
        brain.absorb(source);
        assert_eq!(brain.source_access_count(&KnowledgeSource::HeroUI), 1);
        assert!(!brain.is_source_hot(&KnowledgeSource::HeroUI));
    }

    #[test]
    fn test_multiple_absorb_makes_hot() {
        let mut brain = ReasoningBrain::new();
        let source = KnowledgeSource::BaseUI;
        brain.absorb(source);
        brain.absorb(KnowledgeSource::BaseUI);
        brain.absorb(KnowledgeSource::BaseUI);
        assert!(brain.is_source_hot(&KnowledgeSource::BaseUI));
        assert_eq!(brain.source_access_count(&KnowledgeSource::BaseUI), 3);
    }

    #[test]
    fn test_cold_sources_returns_unabsorbed() {
        let brain = ReasoningBrain::new();
        let cold = brain.cold_sources(1);
        assert!(cold.contains(&KnowledgeSource::HeroUI));
        assert!(cold.contains(&KnowledgeSource::BaseUI));
        assert!(cold.contains(&KnowledgeSource::ArcUI));
        assert!(cold.contains(&KnowledgeSource::CortexUI));
        assert!(cold.contains(&KnowledgeSource::AgenticDS));
        assert!(cold.contains(&KnowledgeSource::DesignPhilosophy));
        assert!(!cold.is_empty());
    }

    #[test]
    fn test_absorb_batch_tracks_all() {
        let mut brain = ReasoningBrain::new();
        let sources = [
            KnowledgeSource::HeroUI,
            KnowledgeSource::BaseUI,
            KnowledgeSource::ArcUI,
        ];
        brain.absorb_batch(&sources);
        assert_eq!(brain.source_access_count(&KnowledgeSource::HeroUI), 1);
        assert_eq!(brain.source_access_count(&KnowledgeSource::BaseUI), 1);
        assert_eq!(brain.source_access_count(&KnowledgeSource::ArcUI), 1);
    }

    #[test]
    fn test_absorbed_source_removed_from_cold() {
        let mut brain = ReasoningBrain::new();
        brain.absorb(KnowledgeSource::HeroUI);
        let cold = brain.cold_sources(1);
        assert!(!cold.contains(&KnowledgeSource::HeroUI));
    }

    #[test]
    fn test_strategy_default() {
        let brain = ReasoningBrain::new();
        let edits = brain.strategy.generate_edit(&brain, "design a UI");
        assert!(!edits.is_empty(), "default strategy should produce edits");
        for e in &edits {
            assert!(!e.dimension.is_empty());
            assert!((e.confidence - 0.8).abs() < 1e-6);
        }
        let micro = brain.generate_self_edit("design a UI");
        assert!(micro.len() >= edits.len() + 2);
    }

    #[test]
    fn test_strategy_conservative() {
        let brain = ReasoningBrain {
            strategy: Box::new(ConservativeSealStrategy { max_delta: 0.05 }),
            ..ReasoningBrain::new()
        };
        let edits = brain.strategy.generate_edit(&brain, "code review nt_shield");
        assert!(!edits.is_empty());
        for e in &edits {
            assert!(e.delta <= 0.05 + 1e-12, "conservative delta {} > 0.05", e.delta);
            assert!((e.confidence - 0.6).abs() < 1e-6);
        }
    }

    #[test]
    fn test_strategy_aggressive() {
        let brain = ReasoningBrain {
            strategy: Box::new(AggressiveSealStrategy { max_delta: 0.1 }),
            ..ReasoningBrain::new()
        };
        let edits = brain.strategy.generate_edit(&brain, "plan complex system");
        assert!(!edits.is_empty());
        for e in &edits {
            assert!(e.delta >= 0.1 - 1e-12, "aggressive delta {} < 0.1", e.delta);
            assert!((e.confidence - 0.9).abs() < 1e-6);
        }
    }

    #[test]
    fn test_strategy_swappable() {
        let brain_default = ReasoningBrain::new();
        assert_eq!(brain_default.strategy.name(), "default");

        let brain_conservative = ReasoningBrain {
            strategy: Box::new(ConservativeSealStrategy { max_delta: 0.1 }),
            ..ReasoningBrain::new()
        };
        assert_eq!(brain_conservative.strategy.name(), "conservative");

        let brain_aggressive = ReasoningBrain {
            strategy: Box::new(AggressiveSealStrategy { max_delta: 0.1 }),
            ..ReasoningBrain::new()
        };
        assert_eq!(brain_aggressive.strategy.name(), "aggressive");
    }

    #[test]
    fn test_fisher_basic() {
        let mut fisher = FisherMatrix::new(23);
        assert_eq!(fisher.values.len(), 23);
        assert!(fisher.values.iter().all(|v| *v == 0.0));

        let deltas = [0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                      0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                      0.0, 0.0, 0.0];
        fisher.update_raw(&deltas);
        assert_eq!(fisher.total_samples, 1);
        assert!((fisher.values[0] - 0.1).abs() < 1e-10);
        assert!((fisher.values[1] - 0.2).abs() < 1e-10);

        fisher.update_raw(&deltas);
        assert_eq!(fisher.total_samples, 2);
        assert!((fisher.values[0] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_default_fisher_enabled() {
        let brain = ReasoningBrain::new();
        assert!(brain.fisher.is_some());
        assert!((brain.ewc_lambda - 0.5).abs() < 1e-10);
        assert_eq!(brain.fisher.as_ref().unwrap().values.len(), 23);
    }

    #[test]
    fn test_save_load_ewc_roundtrip() {
        let mut brain = ReasoningBrain {
            fisher: Some(FisherMatrix::new(23)),
            ewc_lambda: 0.8,
            ..ReasoningBrain::new()
        };
        let deltas = [0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                      0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                      0.0, 0.0, 0.0];
        brain.fisher.as_mut().unwrap().update_raw(&deltas);

        let path = std::env::temp_dir().join("test_ewc_roundtrip.json");
        let path_str = path.to_str().unwrap().to_string();

        brain.save_ewc(&path_str).expect("save should succeed");

        let mut loaded = ReasoningBrain::new();
        loaded.load_ewc(&path_str).expect("load should succeed");

        assert!((loaded.ewc_lambda - 0.8).abs() < 1e-10);
        assert!(loaded.fisher.is_some());
        let f = loaded.fisher.unwrap();
        assert_eq!(f.values.len(), 23);
        assert!((f.values[0] - 0.1).abs() < 1e-10);
        assert!((f.values[1] - 0.2).abs() < 1e-10);
        assert_eq!(f.total_samples, 1);

        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_fisher_ewc_scales_down() {
        let mut brain = ReasoningBrain {
            fisher: Some(FisherMatrix::new(23)),
            ewc_lambda: 0.01,
            ..ReasoningBrain::new()
        };

        brain.absorb(KnowledgeSource::HeroUI);
        let _first_penalty = brain.fisher.as_ref().unwrap().ewc_penalty(
            &[0.0; 23],
            brain.capability.arr(),
        );

        let _cap_before = brain.capability.clone();
        brain.absorb(KnowledgeSource::AgenticDS);
        let has_importance = brain.fisher.as_ref().unwrap().values.iter().any(|v| *v > 0.0);
        assert!(has_importance, "Fisher should have non-zero importance after first absorb");
    }
}
