use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::core::nt_core_consciousness::FirstPersonRef;

fn hash_first_person(first_person: &FirstPersonRef) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(first_person.self_vector());
    hasher.write_u64(first_person.birth_step());
    hasher.finish()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VsaPrefixFingerprint {
    pub fingerprint: u64,
    pub first_person_hash: u64,
    pub constraint_hash: u64,
    pub constraints: Vec<String>,
}

impl VsaPrefixFingerprint {
    pub fn new(first_person: &FirstPersonRef, constraints: Vec<String>) -> Self {
        let first_person_hash = hash_first_person(first_person);

        let mut constraint_hasher = DefaultHasher::new();
        for c in &constraints {
            c.hash(&mut constraint_hasher);
        }
        let constraint_hash = constraint_hasher.finish();

        let mut combined = DefaultHasher::new();
        first_person_hash.hash(&mut combined);
        constraint_hash.hash(&mut combined);
        let fingerprint = combined.finish();

        Self {
            fingerprint,
            first_person_hash,
            constraint_hash,
            constraints,
        }
    }

    pub fn verify(&self, first_person: &FirstPersonRef, constraints: &[String]) -> Result<(), DriftError> {
        let current = Self::new(first_person, constraints.to_vec());
        if self.fingerprint != current.fingerprint {
            return Err(DriftError {
                expected: self.fingerprint,
                actual: current.fingerprint,
                first_person_mismatch: self.first_person_hash != current.first_person_hash,
                constraint_mismatch: self.constraint_hash != current.constraint_hash,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DriftError {
    pub expected: u64,
    pub actual: u64,
    pub first_person_mismatch: bool,
    pub constraint_mismatch: bool,
}

impl std::fmt::Display for DriftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VSA prefix drift: expected={:x}, actual={:x}", self.expected, self.actual)
    }
}

impl std::error::Error for DriftError {}

pub fn default_constraints() -> Vec<String> {
    vec![
        "system:neotrix".to_string(),
        "version:0.18.0".to_string(),
        "vsa_dim:4096".to_string(),
        "e8_mode:adaptive".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_creation() {
        let fp = FirstPersonRef::bootstrap(0);
        let constraints = default_constraints();
        let vfp = VsaPrefixFingerprint::new(&fp, constraints);
        assert_ne!(vfp.fingerprint, 0);
        assert_ne!(vfp.first_person_hash, 0);
        assert_ne!(vfp.constraint_hash, 0);
    }

    #[test]
    fn test_fingerprint_verify_success() {
        let fp = FirstPersonRef::bootstrap(0);
        let constraints = default_constraints();
        let vfp = VsaPrefixFingerprint::new(&fp, constraints.clone());
        assert!(vfp.verify(&fp, &constraints).is_ok());
    }

    #[test]
    fn test_fingerprint_verify_fail_on_constraint_change() {
        let fp = FirstPersonRef::bootstrap(0);
        let constraints = default_constraints();
        let vfp = VsaPrefixFingerprint::new(&fp, constraints);
        let different = vec!["different".to_string()];
        let result = vfp.verify(&fp, &different);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.constraint_mismatch);
    }

    #[test]
    fn test_fingerprint_verify_fail_on_first_person_change() {
        let fp1 = FirstPersonRef::bootstrap(0);
        let fp2 = FirstPersonRef::bootstrap(1);
        let constraints = default_constraints();
        let vfp = VsaPrefixFingerprint::new(&fp1, constraints.clone());
        let result = vfp.verify(&fp2, &constraints);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.first_person_mismatch);
    }

    #[test]
    fn test_default_constraints_are_stable() {
        let c1 = default_constraints();
        let c2 = default_constraints();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_drift_error_display() {
        let err = DriftError {
            expected: 0xabc,
            actual: 0xdef,
            first_person_mismatch: true,
            constraint_mismatch: false,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("abc"));
        assert!(msg.contains("def"));
    }
}
