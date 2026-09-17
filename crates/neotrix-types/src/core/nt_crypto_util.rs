//! Cross-domain cryptographic utilities.
//!
//! Shared helper functions used across L1–L6 layers to avoid
//! private duplicate definitions of common hash operations.

use sha2::{Digest, Sha256};

/// Compute SHA-256 hash and return it as a lowercase hex string.
///
/// Used by:
/// - L1 `coverage_ledger` (hash chain / Merkle tree)
/// - L3 `receipt` (verifiable replay receipts)
/// - L3 `adversarial_pipeline` (defense audit chain)
pub fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex::encode(hasher.finalize())
}

/// Convenience overload for `&str` callers.
pub fn sha256_hex_str(s: &str) -> String {
    sha256_hex(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_deterministic() {
        let a = sha256_hex(b"hello");
        let b = sha256_hex(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn sha256_known_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_str_matches_bytes() {
        let input = "test data";
        assert_eq!(sha256_hex(input.as_bytes()), sha256_hex_str(input));
    }
}
