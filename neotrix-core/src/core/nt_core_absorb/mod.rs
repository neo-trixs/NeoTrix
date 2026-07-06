pub mod spec_driven;

pub use spec_driven::{
    EvolutionSpec, SpecDiff, SpecDrivenPipeline, SpecPipelineConfig, SpecPipelineStats,
    SpecStatus, SpecVerification, SpecVerifier,
};

use crate::core::nt_core_cap::CapabilityVector;

pub trait AbsorbValidator {
    fn validate_absorb(&self, after: &CapabilityVector) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_cap::CapabilityVector;

    struct AlwaysValid;
    impl AbsorbValidator for AlwaysValid {
        fn validate_absorb(&self, _after: &CapabilityVector) -> bool { true }
    }

    struct NeverValid;
    impl AbsorbValidator for NeverValid {
        fn validate_absorb(&self, _after: &CapabilityVector) -> bool { false }
    }

    #[test]
    fn test_always_valid() {
        let v = AlwaysValid;
        let cv = CapabilityVector::default();
        assert!(v.validate_absorb(&cv));
    }

    #[test]
    fn test_never_valid() {
        let v = NeverValid;
        let cv = CapabilityVector::default();
        assert!(!v.validate_absorb(&cv));
    }
}
