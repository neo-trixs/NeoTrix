//! Reverse bridge: V1 → core/
//! All types re-exported from `crate::core::nt_core_cap`.

pub use crate::core::nt_core_cap::{
    CapabilityVector, FIELD_NAMES, IDX_ACCESSIBILITY, IDX_AI_NATIVE_STATES, IDX_ANALYSIS,
    IDX_BEM_NAMING, IDX_COLOR, IDX_COMPOUND_COMPOSITION, IDX_CREATIVITY, IDX_DATA_VIZ,
    IDX_DOMAIN_SPECIFICITY, IDX_EMOTION, IDX_EXPERIMENTAL, IDX_FIGMA_INTEGRATION, IDX_GRID,
    IDX_INFERENCE_DEPTH, IDX_MINIMALISM, IDX_QUALITY_GATES, IDX_REACT_ARIA_USAGE,
    IDX_SEMANTIC_LAYER, IDX_SYNTHESIS, IDX_TAILWIND_PROFICIENCY, IDX_TYPOGRAPHY, IDX_VERIFICATION,
    IDX_WHITESPACE, NUM_FIELDS,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_vector_default_dim() {
        let cv = CapabilityVector::default();
        assert_eq!(cv.total_dim(), 23);
        assert_eq!(cv.dim(), 23);
        assert_eq!(cv.to_full_vector().len(), 23);
    }

    #[test]
    fn test_capability_vector_from_values_accessors() {
        let cv = CapabilityVector::from_values(
            0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2,
            0.1, 0.9, 0.8, 0.7, 0.6, 0.5,
        );
        assert!((cv.typography() - 0.9).abs() < 1e-10);
        assert!((cv.grid() - 0.8).abs() < 1e-10);
        assert!((cv.verification() - 0.5).abs() < 1e-10);
        assert!((cv.creativity() - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_similarity_identical() {
        let a = CapabilityVector::from_values(
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        let b = a.clone();
        assert!((a.similarity(&b) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_similarity_orthogonal() {
        let a = CapabilityVector::from_values(
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        let b = CapabilityVector::from_values(
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        assert!((a.similarity(&b) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_similarity_zero_vector() {
        let a = CapabilityVector::default();
        let b = CapabilityVector::from_values(
            0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        assert!((a.similarity(&b) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_normalize_clamps_above_one() {
        let mut cv = CapabilityVector::from_values(
            2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        cv.normalize();
        assert!((cv.typography() - 2.0 / 3.0).abs() < 1e-10);
        assert!((cv.grid() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_normalize_noop_when_under_one() {
        let mut cv = CapabilityVector::from_values(
            0.5, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        cv.normalize();
        assert!((cv.typography() - 0.5).abs() < 1e-10);
        assert!((cv.grid() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_extension_dim() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("custom_dim", 0.95);
        assert_eq!(cv.total_dim(), 24);
        assert_eq!(cv.extension_values().len(), 1);
        assert!((cv.extension_values()[0] - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_extension_update_existing() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("dim_x", 0.5);
        cv.add_extension_dim("dim_x", 0.9);
        assert_eq!(cv.extension().len(), 1);
        assert!((cv.extension_values()[0] - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_provenance() {
        let mut cv = CapabilityVector::default();
        assert!(cv.provenance().is_none());
        cv.set_provenance("test_source".into());
        assert_eq!(cv.provenance(), Some(&"test_source".to_string()));
    }

    #[test]
    fn test_capability_vector_extension_similarity() {
        let mut a = CapabilityVector::default();
        a.add_extension_dim("d1", 1.0);
        a.add_extension_dim("d2", 0.0);
        let mut b = CapabilityVector::default();
        b.add_extension_dim("d1", 1.0);
        b.add_extension_dim("d2", 1.0);
        let sim = a.extension_similarity(&b);
        assert!(sim > 0.7);
    }

    #[test]
    fn test_capability_vector_merge_similar() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("alpha", 0.9);
        cv.add_extension_dim("alpha", 0.5);
        assert_eq!(cv.extension().len(), 1);
    }

    #[test]
    fn test_capability_vector_update_from_other() {
        let mut a = CapabilityVector::default();
        let b = CapabilityVector::from_values(
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        );
        a.update_from_other(&b, 0.5);
        assert!((a.typography() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_set_field_by_name() {
        let mut cv = CapabilityVector::default();
        assert!(cv.set_field_by_name("analysis", 0.85));
        assert!((cv.analysis() - 0.85).abs() < 1e-10);
        assert!(!cv.set_field_by_name("nonexistent", 0.5));
    }

    #[test]
    fn test_capability_vector_to_full_includes_extension() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("extra", 0.77);
        let full = cv.to_full_vector();
        assert_eq!(full.len(), 24);
        assert!((full[23] - 0.77).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_from_array_valid() {
        let arr = vec![0.5; 23];
        let cv = CapabilityVector::from_array(&arr).expect("valid array");
        assert!((cv.typography() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_from_array_invalid() {
        let arr = vec![0.5; 22];
        assert!(CapabilityVector::from_array(&arr).is_err());
    }

    #[test]
    fn test_capability_vector_to_array() {
        let cv = CapabilityVector::from_values(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        );
        assert_eq!(cv.to_array().len(), 23);
    }

    #[test]
    fn test_capability_vector_add_simd() {
        let mut cv = CapabilityVector::default();
        cv.add_simd(&[1.0, 2.0], 0);
        assert!((cv.typography() - 1.0).abs() < 1e-10);
        assert!((cv.grid() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_capability_vector_prune_extension() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("zero", 0.0);
        cv.add_extension_dim("nonzero", 0.5);
        cv.prune_extension();
        assert_eq!(cv.extension().len(), 1);
        assert_eq!(cv.extension()[0].0, "nonzero");
    }

    #[test]
    fn test_capability_vector_extend_named() {
        let mut cv = CapabilityVector::default();
        cv.extend_named(&[("e1".into(), 0.3), ("e2".into(), 0.7)]);
        assert_eq!(cv.extension().len(), 2);
    }
}
