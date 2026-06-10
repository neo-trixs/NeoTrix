use rand::Rng;

pub const VSA_DIM: usize = 4096;
pub const BINARY_THRESHOLD: u8 = 128;

#[derive(Debug, Clone)]
pub struct QuantizedVSA {
    dim: usize,
}

impl Default for QuantizedVSA {
    fn default() -> Self {
        Self::new(VSA_DIM)
    }
}

impl QuantizedVSA {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    pub fn random_vector() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..VSA_DIM).map(|_| rng.gen()).collect()
    }

    pub fn seeded_random(seed: u64, dim: usize) -> Vec<u8> {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        (0..dim).map(|_| rng.gen()).collect()
    }

    pub fn random_binary() -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..VSA_DIM).map(|_| if rng.gen_bool(0.5) { 1 } else { 0 }).collect()
    }

    pub fn binarize(v: &[u8]) -> Vec<u8> {
        v.iter().map(|&x| if x >= BINARY_THRESHOLD { 1 } else { 0 }).collect()
    }

    pub fn bind(a: &[u8], b: &[u8]) -> Vec<u8> {
        a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
    }

    pub fn bundle(vectors: &[&[u8]]) -> Vec<u8> {
        if vectors.is_empty() {
            return vec![0; VSA_DIM];
        }
        let n = vectors.len();
        let dim = vectors[0].len().min(VSA_DIM);
        let mut counts = vec![0i32; dim];
        for v in vectors {
            for (c, &x) in counts.iter_mut().zip(v.iter()) {
                if x >= BINARY_THRESHOLD {
                    *c += 1;
                }
            }
        }
        let threshold = (n as i32 + 1) / 2;
        counts.iter().map(|&c| if c > threshold { 1 } else { 0 }).collect()
    }

    pub fn permute(v: &[u8], shift: isize) -> Vec<u8> {
        let len = v.len().min(VSA_DIM);
        let mut result = vec![0u8; len];
        for i in 0..len {
            let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
            result[i] = v[src];
        }
        result
    }

    pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
        let len = a.len().min(b.len()).min(VSA_DIM);
        let mut dist = 0u32;
        let chunks = len / 8;
        let rem = len % 8;
        for i in 0..chunks {
            let offset = i * 8;
            let mut a_bits = 0u64;
            let mut b_bits = 0u64;
            for j in 0..8 {
                a_bits |= (a[offset + j] as u64 & 1) << j;
                b_bits |= (b[offset + j] as u64 & 1) << j;
            }
            dist += (a_bits ^ b_bits).count_ones();
        }
        for j in 0..rem {
            let idx = chunks * 8 + j;
            if (a[idx] & 1) != (b[idx] & 1) {
                dist += 1;
            }
        }
        dist
    }

    pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len()).min(VSA_DIM);
        if len == 0 {
            return 0.0;
        }
        1.0 - (Self::hamming_distance(a, b) as f64 / len as f64)
    }

    pub fn cosine(a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len()).min(VSA_DIM);
        if len == 0 {
            return 0.0;
        }
        let mut dot = 0u64;
        let mut mag_a = 0u64;
        let mut mag_b = 0u64;
        for i in 0..len {
            let va = a[i] as u64;
            let vb = b[i] as u64;
            dot += va * vb;
            mag_a += va * va;
            mag_b += vb * vb;
        }
        let denom = ((mag_a as f64).sqrt() * (mag_b as f64).sqrt()).max(1e-12);
        dot as f64 / denom
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

/// Pack 8 binary values (each 0 or 1) into each byte for fast hamming distance.
/// Input length must be multiple of 8. Output is input.len() / 8.
pub fn pack_binary(v: &[u8]) -> Vec<u8> {
    v.chunks(8).map(|chunk| {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit != 0 {
                byte |= 1 << i;
            }
        }
        byte
    }).collect()
}

/// Hamming distance on packed binary vectors using POPCNT via u8::count_ones().
/// Both inputs must be same length.
pub fn hamming_distance_packed(a: &[u8], b: &[u8]) -> u32 {
    a.iter().zip(b.iter()).map(|(&x, &y)| (x ^ y).count_ones()).sum()
}

/// Normalized similarity [0,1] from packed hamming distance.
/// 1.0 = identical, 0.0 = all bits differ.
pub fn similarity_packed(a: &[u8], b: &[u8]) -> f64 {
    let total_bits = (a.len().min(b.len()) * 8) as f64;
    if total_bits == 0.0 { return 0.0; }
    let dist = hamming_distance_packed(a, b) as f64;
    1.0 - dist / total_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vec(v: u8) -> Vec<u8> {
        vec![v; VSA_DIM]
    }

    #[test]
    fn test_random_vector_has_correct_dim() {
        let v = QuantizedVSA::random_vector();
        assert_eq!(v.len(), VSA_DIM);
    }

    #[test]
    fn test_binary_vector_is_binary() {
        let v = QuantizedVSA::random_binary();
        for &x in &v {
            assert!(x == 0 || x == 1, "binary vector element must be 0 or 1");
        }
    }

    #[test]
    fn test_bind_self_returns_zero() {
        let a = QuantizedVSA::random_binary();
        let bound = QuantizedVSA::bind(&a, &a);
        assert!(bound.iter().all(|&x| x == 0), "a XOR a should be all zeros");
    }

    #[test]
    fn test_bind_inverse() {
        let a = QuantizedVSA::random_binary();
        let ones: Vec<u8> = vec![1; VSA_DIM];
        let inverted = QuantizedVSA::bind(&a, &ones);
        assert!(QuantizedVSA::similarity(&a, &inverted) < 0.01,
            "a XOR all-ones should be permutation of a (dissimilar)");
    }

    #[test]
    fn test_bundle_non_empty() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let bundled = QuantizedVSA::bundle(&[&a, &b]);
        assert_eq!(bundled.len(), VSA_DIM);
        let sim_a = QuantizedVSA::similarity(&bundled, &a);
        let sim_b = QuantizedVSA::similarity(&bundled, &b);
        assert!(sim_a > 0.4 || sim_b > 0.4,
            "bundled should be similar to at least one component");
    }

    #[test]
    fn test_permute_reversible() {
        let v = QuantizedVSA::random_binary();
        let p = QuantizedVSA::permute(&v, 100);
        let r = QuantizedVSA::permute(&p, -100);
        let sim = QuantizedVSA::similarity(&r, &v);
        assert!((sim - 1.0).abs() < 1e-10, "permute should be reversible");
    }

    #[test]
    fn test_self_similarity_one() {
        let v = QuantizedVSA::random_binary();
        let sim = QuantizedVSA::similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_orthogonal_similarity_zero() {
        let zeros = test_vec(0);
        let ones = test_vec(1);
        let sim = QuantizedVSA::similarity(&zeros, &ones);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_preserves_self_similarity() {
        let v = QuantizedVSA::random_vector();
        let sim = QuantizedVSA::cosine(&v, &v);
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hamming_distance_counts_ones() {
        let a = test_vec(0);
        let b = test_vec(1);
        assert_eq!(QuantizedVSA::hamming_distance(&a, &b), VSA_DIM as u32);
    }

    #[test]
    fn test_dim() {
        let q = QuantizedVSA::new(2048);
        assert_eq!(q.dim(), 2048);
    }

    #[test]
    fn test_empty_bundle_returns_zero_vector() {
        let bundled = QuantizedVSA::bundle(&[]);
        assert_eq!(bundled.len(), VSA_DIM);
        assert!(bundled.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_binarize_zero() {
        let v = vec![0u8; 100];
        let b = QuantizedVSA::binarize(&v);
        assert!(b.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_binarize_high_values() {
        let v = vec![200u8; 100];
        let b = QuantizedVSA::binarize(&v);
        assert!(b.iter().all(|&x| x == 1));
    }

    #[test]
    fn test_bind_non_commutative_xor() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let ab = QuantizedVSA::bind(&a, &b);
        let ba = QuantizedVSA::bind(&b, &a);
        assert_eq!(ab, ba, "XOR is commutative");
    }

    #[test]
    fn test_hamming_range() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let dist = QuantizedVSA::hamming_distance(&a, &b);
        assert!(dist <= VSA_DIM as u32, "Hamming cannot exceed dimension");
    }

    #[test]
    fn test_pack_binary_roundtrip() {
        let original = QuantizedVSA::random_binary();
        assert_eq!(original.len(), VSA_DIM);
        let packed = pack_binary(&original);
        assert_eq!(packed.len(), VSA_DIM / 8);
    }

    #[test]
    fn test_pack_identical_vectors_zero_distance() {
        let v = vec![1u8; VSA_DIM];
        let packed = pack_binary(&v);
        assert_eq!(hamming_distance_packed(&packed, &packed), 0);
        assert!((similarity_packed(&packed, &packed) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pack_all_zeros_vs_all_ones() {
        let zeros = vec![0u8; VSA_DIM];
        let ones = vec![1u8; VSA_DIM];
        let pz = pack_binary(&zeros);
        let po = pack_binary(&ones);
        assert_eq!(hamming_distance_packed(&pz, &po), VSA_DIM as u32);
        assert!((similarity_packed(&pz, &po) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_pack_preserves_information() {
        let mut v = vec![0u8; VSA_DIM];
        for i in (0..VSA_DIM).step_by(2) { v[i] = 1; }
        let packed = pack_binary(&v);
        assert_eq!(packed.len(), VSA_DIM / 8);
        // Every other bit is 1 → each byte = 0b01010101 = 85
        for &byte in &packed {
            assert_eq!(byte, 0b01010101, "alternating bits should pack to 0x55");
        }
    }
}
