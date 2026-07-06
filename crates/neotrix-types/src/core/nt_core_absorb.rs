//! AbsorbValidator — 吸收验证器 trait
//!
//! 对应 AIRecon 的 Critic Model 闭环：由独立模型/规则验证吸收效果
//! 迁移自 `reasoning_brain/self_iterating.rs`

use crate::core::nt_core_cap::CapabilityVector;

/// AbsorbValidator trait — 吸收验证器（独立于被验证方）
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
