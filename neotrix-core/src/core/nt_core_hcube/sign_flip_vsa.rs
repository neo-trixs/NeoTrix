pub const SIGN_FLIP_DIM: usize = 4096;
pub const SIGN_FLIP_PACKED_BYTES: usize = 512;

/// A 4096-dimensional bipolar VSA vector with values in {-1, +1}.
///
/// Packed as 512 bytes (8 values per byte, one bit per value).
/// Bit = 0 → value -1, bit = 1 → value +1.
///
/// Sign-flip binding is element-wise multiplication, which maps to XNOR
/// in the packed bit representation. This is self-inverse and preserves
/// similarity better than XOR binding for compositional reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignFlipVsa {
    data: [u8; SIGN_FLIP_PACKED_BYTES],
}

impl SignFlipVsa {
    /// Identity vector: all +1
    pub fn identity() -> Self {
        SignFlipVsa {
            data: [0xFF; SIGN_FLIP_PACKED_BYTES],
        }
    }

    /// Zero vector (all -1)
    pub fn zero() -> Self {
        SignFlipVsa {
            data: [0x00; SIGN_FLIP_PACKED_BYTES],
        }
    }

    /// Create from a packed 512-byte binary VSA (bit=0→-1, bit=1→+1)
    pub fn from_binary(binary: &[u8; SIGN_FLIP_PACKED_BYTES]) -> Self {
        SignFlipVsa { data: *binary }
    }

    /// Convert to packed 512-byte binary VSA
    pub fn to_binary(&self) -> [u8; SIGN_FLIP_PACKED_BYTES] {
        self.data
    }

    /// Create from binary Vec<u8> (one byte per element, 0 or 1)
    pub fn from_binary_vec(binary: &[u8]) -> Self {
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for (i, &b) in binary.iter().enumerate().take(SIGN_FLIP_DIM) {
            if b != 0 {
                data[i / 8] |= 1 << (i % 8);
            }
        }
        SignFlipVsa { data }
    }

    /// Convert to binary Vec<u8> (one byte per element, 0 or 1)
    pub fn to_binary_vec(&self) -> Vec<u8> {
        let mut result = vec![0u8; SIGN_FLIP_DIM];
        for i in 0..SIGN_FLIP_DIM {
            result[i] = (self.data[i / 8] >> (i % 8)) & 1;
        }
        result
    }

    /// Generate a random sign-flip vector with seeded RNG (deterministic)
    pub fn random(seed: u64) -> Self {
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for i in 0..SIGN_FLIP_DIM {
            if seeded_random(seed, i) == 1 {
                data[i / 8] |= 1 << (i % 8);
            }
        }
        SignFlipVsa { data }
    }

    /// Generate a deterministic sign-flip vector from a string
    pub fn from_string(text: &str) -> Self {
        let seed: u64 = text
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        Self::random(seed)
    }

    /// Sign-flip binding: element-wise multiplication = XNOR in bit space.
    ///
    /// Self-inverse: `bind(x, x) = identity` (all +1).
    /// Commutative: `bind(x, y) = bind(y, x)`.
    pub fn bind(&self, other: &SignFlipVsa) -> SignFlipVsa {
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for i in 0..SIGN_FLIP_PACKED_BYTES {
            data[i] = !(self.data[i] ^ other.data[i]);
        }
        SignFlipVsa { data }
    }

    /// Self-inverse: bind IS unbind for sign-flip VSA
    pub fn unbind(&self, other: &SignFlipVsa) -> SignFlipVsa {
        self.bind(other)
    }

    /// Bundle: element-wise majority.
    /// For each bit position, the output is +1 if more vectors have +1 than -1.
    pub fn bundle(vectors: &[&SignFlipVsa]) -> SignFlipVsa {
        if vectors.is_empty() {
            return SignFlipVsa::zero();
        }
        let count = vectors.len();
        let half = (count / 2) as i32;
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for byte_idx in 0..SIGN_FLIP_PACKED_BYTES {
            let mut out_byte: u8 = 0;
            for bit_idx in 0..8 {
                let mut ones: i32 = 0;
                for v in vectors {
                    if (v.data[byte_idx] >> bit_idx) & 1 == 1 {
                        ones += 1;
                    }
                }
                if ones > half {
                    out_byte |= 1 << bit_idx;
                }
            }
            data[byte_idx] = out_byte;
        }
        SignFlipVsa { data }
    }

    /// Approximate bundle: sum bipolar values and threshold at 0.
    /// Equivalent to majority for odd-length inputs; ties resolve to -1.
    pub fn approximate_bundle(vectors: &[&SignFlipVsa]) -> SignFlipVsa {
        if vectors.is_empty() {
            return SignFlipVsa::zero();
        }
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for byte_idx in 0..SIGN_FLIP_PACKED_BYTES {
            let mut out_byte: u8 = 0;
            for bit_idx in 0..8 {
                let mut sum_val: i32 = 0;
                for v in vectors {
                    if (v.data[byte_idx] >> bit_idx) & 1 == 1 {
                        sum_val += 1;
                    } else {
                        sum_val -= 1;
                    }
                }
                if sum_val > 0 {
                    out_byte |= 1 << bit_idx;
                }
            }
            data[byte_idx] = out_byte;
        }
        SignFlipVsa { data }
    }

    /// Permute: rotate all 4096 bits by k positions (left rotation).
    pub fn permute(&self, k: usize) -> SignFlipVsa {
        let k = k % SIGN_FLIP_DIM;
        if k == 0 {
            return *self;
        }
        let mut bits = [0u8; SIGN_FLIP_DIM];
        for i in 0..SIGN_FLIP_DIM {
            bits[i] = (self.data[i / 8] >> (i % 8)) & 1;
        }
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for i in 0..SIGN_FLIP_DIM {
            let dst = (i + k) % SIGN_FLIP_DIM;
            if bits[i] != 0 {
                data[dst / 8] |= 1 << (dst % 8);
            }
        }
        SignFlipVsa { data }
    }

    /// Cosine similarity in [-1, 1].
    /// 1.0 = identical, -1.0 = completely opposite, 0.0 = orthogonal.
    pub fn cosine_similarity(&self, other: &SignFlipVsa) -> f64 {
        let dist: u32 = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        1.0 - 2.0 * (dist as f64 / SIGN_FLIP_DIM as f64)
    }

    /// Hamming similarity in [0, 1].
    /// 1.0 = identical, 0.0 = completely opposite.
    pub fn hamming_similarity(&self, other: &SignFlipVsa) -> f64 {
        let dist: u32 = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| (a ^ b).count_ones())
            .sum();
        1.0 - dist as f64 / SIGN_FLIP_DIM as f64
    }

    /// Negate: flip all signs (bitwise NOT)
    pub fn negate(&self) -> SignFlipVsa {
        let mut data = [0u8; SIGN_FLIP_PACKED_BYTES];
        for i in 0..SIGN_FLIP_PACKED_BYTES {
            data[i] = !self.data[i];
        }
        SignFlipVsa { data }
    }
}

/// Splitmix64-style deterministic PRNG that maps (seed, index) → {-1, +1}
fn seeded_random(seed: u64, index: usize) -> i8 {
    let mut z = seed.wrapping_add(index as u64);
    z = z.wrapping_mul(0x9e3779b97f4a7c15);
    z ^= z >> 30;
    z = z.wrapping_mul(0xbf58476d1ce4e5b9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    if (z & 1) == 0 {
        -1
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_seeded_deterministic() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(42);
        assert_eq!(a, b, "same seed must produce same vector");

        let c = SignFlipVsa::random(99);
        assert_ne!(
            a, c,
            "different seeds should produce different vectors (very unlikely collision)"
        );
    }

    #[test]
    fn test_bind_and_unbind_self_inverse() {
        let a = SignFlipVsa::random(42);
        let bound = a.bind(&a);
        let identity = SignFlipVsa::identity();
        assert_eq!(bound, identity, "bind(x, x) must be identity (all +1)");
    }

    #[test]
    fn test_bind_identity() {
        let a = SignFlipVsa::random(42);
        let ident = SignFlipVsa::identity();
        let bound = a.bind(&ident);
        assert_eq!(bound, a, "bind(x, identity) must equal x");
    }

    #[test]
    fn test_bind_commutative() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        let ab = a.bind(&b);
        let ba = b.bind(&a);
        assert_eq!(ab, ba, "sign-flip bind must be commutative");
    }

    #[test]
    fn test_unbind_same_as_bind() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        assert_eq!(a.bind(&b), a.unbind(&b), "bind == unbind for sign-flip");
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = SignFlipVsa::random(42);
        let sim = a.cosine_similarity(&a);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "self cosine must be 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = SignFlipVsa::random(42);
        let neg = a.negate();
        let sim = a.cosine_similarity(&neg);
        assert!(
            (sim + 1.0).abs() < 1e-10,
            "cosine(a, negate(a)) must be -1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_hamming_similarity_self() {
        let a = SignFlipVsa::random(42);
        let sim = a.hamming_similarity(&a);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "self hamming must be 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_bundle_simple() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        let bundled = SignFlipVsa::bundle(&[&a, &b]);
        let sim_a = bundled.cosine_similarity(&a);
        let sim_b = bundled.cosine_similarity(&b);
        assert!(
            sim_a > 0.3 || sim_b > 0.3,
            "bundle should be similar to at least one input; sim_a={}, sim_b={}",
            sim_a,
            sim_b
        );
    }

    #[test]
    fn test_bundle_empty_returns_zero() {
        let bundled = SignFlipVsa::bundle(&[]);
        assert_eq!(bundled, SignFlipVsa::zero(), "empty bundle must be zero");
    }

    #[test]
    fn test_permute_reversible() {
        let a = SignFlipVsa::random(42);
        let p = a.permute(257);
        let r = p.permute(SIGN_FLIP_DIM - 257);
        assert_eq!(a, r, "permute by k then by (dim-k) must restore original");
    }

    #[test]
    fn test_permute_zero_no_op() {
        let a = SignFlipVsa::random(42);
        assert_eq!(a, a.permute(0), "permute by 0 must be identity");
        assert_eq!(
            a,
            a.permute(SIGN_FLIP_DIM),
            "permute by dim must be identity"
        );
    }

    #[test]
    fn test_from_binary_roundtrip() {
        let a = SignFlipVsa::random(42);
        let binary = a.to_binary();
        let b = SignFlipVsa::from_binary(&binary);
        assert_eq!(a, b, "to_binary → from_binary must roundtrip");
    }

    #[test]
    fn test_from_binary_vec_roundtrip() {
        let a = SignFlipVsa::random(42);
        let binary_vec = a.to_binary_vec();
        assert_eq!(binary_vec.len(), SIGN_FLIP_DIM);
        let b = SignFlipVsa::from_binary_vec(&binary_vec);
        assert_eq!(a, b, "to_binary_vec → from_binary_vec must roundtrip");
    }

    #[test]
    fn test_from_string_deterministic() {
        let a = SignFlipVsa::from_string("hello");
        let b = SignFlipVsa::from_string("hello");
        assert_eq!(a, b, "same string must produce same vector");

        let c = SignFlipVsa::from_string("world");
        assert_ne!(
            a, c,
            "different strings should produce different vectors (very unlikely collision)"
        );
    }

    #[test]
    fn test_negate() {
        let a = SignFlipVsa::random(42);
        let neg = a.negate();
        assert_ne!(a, neg, "negate must produce a different vector");
        let reneg = neg.negate();
        assert_eq!(a, reneg, "double negate must restore original");
    }

    #[test]
    fn test_identity_is_all_plus_one() {
        let ident = SignFlipVsa::identity();
        for &byte in ident.data.iter() {
            assert_eq!(byte, 0xFF, "identity must be all bits 1");
        }
    }

    #[test]
    fn test_zero_is_all_minus_one() {
        let zero = SignFlipVsa::zero();
        for &byte in zero.data.iter() {
            assert_eq!(byte, 0x00, "zero must be all bits 0");
        }
    }

    #[test]
    fn test_cosine_orthogonal_avg() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        let sim = a.cosine_similarity(&b);
        // Random bipolar vectors of 4096 dims have expected cosine ~0
        assert!(
            sim.abs() < 0.3,
            "random vectors should have near-zero cosine, got {}",
            sim
        );
    }

    #[test]
    fn test_bind_unbind_three_way() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        let c = SignFlipVsa::random(123);
        // (a ⊗ b) ⊗ b = a (self-inverse)
        let ab = a.bind(&b);
        let abb = ab.bind(&b);
        assert_eq!(a, abb, "(a ⊗ b) ⊗ b must equal a");

        // (a ⊗ b) ⊗ (c ⊗ b) = a ⊗ c
        let cb = c.bind(&b);
        let ab_cb = ab.bind(&cb);
        let ac = a.bind(&c);
        assert_eq!(ac, ab_cb, "(a⊗b) ⊗ (c⊗b) must equal a ⊗ c");
    }

    #[test]
    fn test_approximate_bundle_matches_bundle_for_odd() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        let c = SignFlipVsa::random(123);
        let bundle = SignFlipVsa::bundle(&[&a, &b, &c]);
        let approx = SignFlipVsa::approximate_bundle(&[&a, &b, &c]);
        assert_eq!(
            bundle, approx,
            "bundle and approximate_bundle should match for odd-length inputs"
        );
    }

    #[test]
    fn test_permute_preserves_cosine() {
        let a = SignFlipVsa::random(42);
        let b = SignFlipVsa::random(99);
        let pa = a.permute(100);
        let pb = b.permute(100);
        let sim_orig = a.cosine_similarity(&b);
        let sim_perm = pa.cosine_similarity(&pb);
        assert!(
            (sim_orig - sim_perm).abs() < 1e-10,
            "cosine must be invariant under same permutation"
        );
    }
}
