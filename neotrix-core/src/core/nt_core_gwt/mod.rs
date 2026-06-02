pub mod module_def;
pub mod workspace;
pub mod resonance;
pub mod monitor;
pub mod physics_attention;
pub mod cls_buffer;
pub mod geometry_sync;
pub mod pipeline;

pub use geometry_sync::{
    CycleReport, CrossDimensionalResonator, DimensionLayer, GeometrySync, IntegratedPhi,
    LayerSnapshot, CONSCIOUS_PHI_THRESHOLD, DEFAULT_COUPLING, DEFAULT_SYNC_THRESHOLD,
    LAYER_COUNT,
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
