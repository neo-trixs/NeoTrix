pub use neotrix_types::core::nt_core_cap::*;

#[cfg(test)]
mod tests {
    use super::CapabilityVector;

    #[test]
    fn test_capability_vector_default() {
        let cv = CapabilityVector::default();
        assert_eq!(cv.total_dim(), 23);
    }

    #[test]
    fn test_capability_vector_from_values() {
        let cv = CapabilityVector::from_values(
            0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.9,
            0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.9, 0.8, 0.7, 0.6, 0.5,
        );
        assert_eq!(cv.total_dim(), 23);
    }

    #[test]
    fn test_capability_vector_set_provenance() {
        let mut cv = CapabilityVector::default();
        cv.set_provenance("test_source".into());
        assert_eq!(cv.provenance(), Some(&"test_source".to_string()));
    }

    #[test]
    fn test_capability_vector_extension() {
        let mut cv = CapabilityVector::default();
        cv.add_extension_dim("custom", 0.95);
        assert_eq!(cv.total_dim(), 24);
        assert_eq!(cv.extension_values().len(), 1);
    }

    #[test]
    fn test_capability_vector_to_full_vector() {
        let cv = CapabilityVector::default();
        let full = cv.to_full_vector();
        assert_eq!(full.len(), 23);
    }
}
