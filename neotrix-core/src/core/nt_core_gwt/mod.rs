pub mod module_def;
pub mod workspace;
pub mod resonance;
pub mod monitor;
pub mod physics_attention;
pub mod ctm_verifier;
pub mod inner_speech;
pub mod modality_router;
pub mod cognitive_type;
pub mod cognitive_hub;
pub mod meta_workspace;
pub mod competition_gate;
pub mod compaction;
pub mod moe_router;
pub mod vsa_scorer;
pub mod pipeline;

pub use workspace::{AuditBlock, AuditEventType};
pub use inner_speech::InnerSpeech;
pub use modality_router::{Modality, ModalityRouter};
pub use cognitive_type::{CognitiveProfile, CognitiveType};
pub use cognitive_hub::{CognitiveHub, HUB_COUNT, HUB_TOPK};
pub use meta_workspace::{MetaObservation, MetaWorkspace, PrimaryObservation};
pub use ctm_verifier::{CtmAlignmentReport, CtmCheck, CtmVerifier, E8_STATE_COUNT};
pub use module_def::{
    OrchestratorAgent, OrchestratorPhase, SpecialistModule,
};

#[cfg(test)]
mod tests {
    use super::module_def::{SpecialistType, SpecialistModule};

    #[test]
    fn test_specialist_type_debug() {
        let st = SpecialistType::PatternMatcher;
        let s = format!("{:?}", st);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_specialist_module_new_with_type() {
        let m = SpecialistModule::new(SpecialistType::CodeAnalyzer, "ca-1".into());
        assert_eq!(m.name, "ca-1");
        assert_eq!(m.module_type, m.specialist_type);
    }

    #[test]
    fn test_specialist_module_activation() {
        let mut m = SpecialistModule::new(SpecialistType::RiskAssessor, "ra".into());
        m.activate(0.7);
        assert!((m.activation - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_specialist_type_variants_distinct() {
        use std::collections::HashSet;
        let variants = vec![
            SpecialistType::PatternMatcher,
            SpecialistType::AnomalyDetector,
            SpecialistType::KnowledgeRetriever,
            SpecialistType::CodeAnalyzer,
            SpecialistType::Planner,
            SpecialistType::KnowledgeIntegrator,
        ];
        let set: HashSet<_> = variants.iter().collect();
        assert_eq!(set.len(), variants.len());
    }
}
