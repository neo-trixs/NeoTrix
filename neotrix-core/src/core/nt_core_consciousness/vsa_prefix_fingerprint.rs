/// VSA prefix fingerprint — FirstPersonRef + system constraints → sha256
/// Verified on each pipeline run for drift detection.
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixFingerprint([u8; 32]);

impl PrefixFingerprint {
    pub fn compute(first_person: &[u8], system_constraints: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"neotrix-vsa-prefix-v1");
        hasher.update(first_person);
        hasher.update(system_constraints);
        PrefixFingerprint(hasher.finalize().into())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Debug)]
pub struct DriftError {
    pub expected: PrefixFingerprint,
    pub actual: PrefixFingerprint,
    pub cycle: u64,
}

impl std::fmt::Display for DriftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VSA prefix drift at cycle {}: expected {} got {}",
            self.cycle,
            self.expected.hex(),
            self.actual.hex()
        )
    }
}
impl std::error::Error for DriftError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_fingerprint() {
        let fp = PrefixFingerprint::compute(b"self", b"constraints");
        assert_eq!(fp.as_bytes().len(), 32);
    }

    #[test]
    fn test_deterministic() {
        let a = PrefixFingerprint::compute(b"hello", b"world");
        let b = PrefixFingerprint::compute(b"hello", b"world");
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_inputs_differ() {
        let a = PrefixFingerprint::compute(b"hello", b"world");
        let b = PrefixFingerprint::compute(b"hello", b"vsa");
        assert_ne!(a, b);
    }

    #[test]
    fn test_hex_format() {
        let fp = PrefixFingerprint::compute(b"test", b"data");
        assert_eq!(fp.hex().len(), 64);
    }

    #[test]
    fn test_drift_error_display() {
        let expected = PrefixFingerprint::compute(b"a", b"b");
        let actual = PrefixFingerprint::compute(b"c", b"d");
        let err = DriftError {
            expected,
            actual,
            cycle: 42,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("cycle 42"));
    }
}
