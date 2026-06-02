use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_knowledge::{KnowledgeSource, KnowledgeProvider};
use crate::core::nt_core_absorb::AbsorbValidator;

#[derive(Debug, Clone)]
pub struct AbsorbResult {
    pub success: bool,
    pub source_name: String,
    pub weight: f64,
    pub before: CapabilityVector,
    pub after: CapabilityVector,
}

pub fn absorb_with_validation(
    cap: &mut CapabilityVector,
    source: KnowledgeSource,
    validator: &dyn AbsorbValidator,
) -> AbsorbResult {
    let before = cap.clone();
    let cv = source.capability_vector();
    cap.update_from_other(&cv, 0.01);
    cap.normalize();
    let after = cap.clone();
    let valid = validator.validate_absorb(&after);
    if !valid {
        *cap = before.clone();
    }
    let weight = if valid { 1.0 } else { 0.0 };
    AbsorbResult { success: valid, source_name: format!("{:?}", source), weight, before, after }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_absorb::AbsorbValidator;
    use crate::core::nt_core_cap::CapabilityVector;
    use crate::core::nt_core_knowledge::{KnowledgeSource, KnowledgeProvider};

    struct AlwaysValid;
    impl AbsorbValidator for AlwaysValid {
        fn validate_absorb(&self, _after: &CapabilityVector) -> bool { true }
    }

    struct NeverValid;
    impl AbsorbValidator for NeverValid {
        fn validate_absorb(&self, _after: &CapabilityVector) -> bool { false }
    }

    struct MockProvider {
        cv: CapabilityVector,
        name: String,
    }

    impl MockProvider {
        fn new(prefix: &str) -> Self {
            let mut cv = CapabilityVector::default();
            cv.arr[0] = 0.5;
            Self { cv, name: prefix.to_string() }
        }
    }

    impl KnowledgeProvider for MockProvider {
        fn capability_vector(&self) -> CapabilityVector { self.cv.clone() }
        fn name(&self) -> &str { &self.name }
        fn source_weight(&self) -> f64 { 0.8 }
    }

    #[test]
    fn test_absorb_with_validation_accepts() {
        let mut cap = CapabilityVector::default();
        let result = absorb_with_validation(&mut cap, KnowledgeSource::HeroUI, &AlwaysValid);
        assert!(result.success);
        assert!((result.weight - 1.0).abs() < 1e-10);
        assert_eq!(result.source_name, "HeroUI");
    }

    #[test]
    fn test_absorb_with_validation_rejects() {
        let mut cap = CapabilityVector::default();
        let before = cap.clone();
        let result = absorb_with_validation(&mut cap, KnowledgeSource::HeroUI, &NeverValid);
        assert!(!result.success);
        assert!((result.weight).abs() < 1e-10);
        // Vector should be rolled back
        assert_eq!(cap.arr, before.arr);
    }

    #[test]
    fn test_absorb_rollback_on_reject() {
        let mut cap = CapabilityVector::default();
        let initial_sum: f64 = cap.arr.iter().sum();
        let _ = absorb_with_validation(&mut cap, KnowledgeSource::BaseUI, &NeverValid);
        let after_sum: f64 = cap.arr.iter().sum();
        assert!((initial_sum - after_sum).abs() < 1e-10);
    }

    #[test]
    fn test_absorb_normalizes() {
        let mut cap = CapabilityVector::default();
        cap.arr[0] = 10.0;
        cap.arr[1] = 10.0;
        let _ = absorb_with_validation(&mut cap, KnowledgeSource::ArcUI, &AlwaysValid);
        let max_val = cap.arr.iter().cloned().fold(0.0f64, |acc, x| acc.max(x));
        assert!(max_val <= 1.0 + 1e-9, "max norm should be ≤ 1.0, got {}", max_val);
    }

    #[test]
    fn test_absorb_from_provider() {
        let mut cap = CapabilityVector::default();
        let provider = MockProvider::new("test_provider");
        let result = absorb_from_provider(&mut cap, &provider, 0.01);
        assert!(result.success);
        assert_eq!(result.source_name, "test_provider");
        assert!((result.weight - 0.008).abs() < 1e-10);
    }

    #[test]
    fn test_absorb_provider_normalizes() {
        let mut cap = CapabilityVector::default();
        cap.arr[0] = 100.0;
        let provider = MockProvider::new("normalizer");
        let _ = absorb_from_provider(&mut cap, &provider, 0.01);
        let norm: f64 = cap.arr.iter().map(|x| x * x).sum();
        assert!((norm - 1.0).abs() < 1e-6 || norm <= 1.0 + 1e-6);
    }

    #[test]
    fn test_absorb_provider_multiple_sources() {
        let mut cap = CapabilityVector::default();
        let p1 = MockProvider::new("p1");
        let p2 = MockProvider::new("p2");
        let r1 = absorb_from_provider(&mut cap, &p1, 0.01);
        let r2 = absorb_from_provider(&mut cap, &p2, 0.01);
        assert!(r1.success);
        assert!(r2.success);
        assert_ne!(r1.source_name, r2.source_name);
    }

    #[test]
    fn test_absorb_result_debug_and_clone() {
        let mut cap = CapabilityVector::default();
        let result = absorb_with_validation(&mut cap, KnowledgeSource::Botasaurus, &AlwaysValid);
        let _ = format!("{:?}", result);
        let cloned = result.clone();
        assert_eq!(result.success, cloned.success);
        assert_eq!(result.source_name, cloned.source_name);
    }
}

pub fn absorb_from_provider(
    cap: &mut CapabilityVector,
    provider: &dyn KnowledgeProvider,
    learning_rate: f64,
) -> AbsorbResult {
    let before = cap.clone();
    let cv = provider.capability_vector().clone();
    cap.update_from_other(&cv, learning_rate);
    cap.normalize();
    AbsorbResult {
        success: true,
        source_name: provider.name().to_string(),
        weight: provider.source_weight() * learning_rate,
        before,
        after: cap.clone(),
    }
}
