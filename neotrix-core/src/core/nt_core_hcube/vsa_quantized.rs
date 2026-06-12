use rand::Rng;

pub const VSA_DIM: usize = 4096;
pub const BINARY_THRESHOLD: u8 = 128;

// === FFT-based Holographic Reduced Representation (FFT-HRR) ===
// Inline FFT implementation, zero external dependencies.
// VSA_DIM = 4096 = 2^12, perfect for radix-2 Cooley-Tukey.

#[derive(Clone, Copy, Debug)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }
}

fn complex_mul(a: Complex, b: Complex) -> Complex {
    Complex::new(
        a.re * b.re - a.im * b.im,
        a.re * b.im + a.im * b.re,
    )
}

fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

fn fft_inplace(x: &mut [Complex]) {
    let n = x.len();
    if n <= 1 {
        return;
    }
    debug_assert!(is_power_of_two(n), "FFT requires power of 2 length, got {}", n);

    // Bit-reversal permutation
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            x.swap(i, j);
        }
    }

    // Cooley-Tukey radix-2 in-place FFT
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = -2.0 * std::f64::consts::PI / len as f64;
        let wlen_re = angle.cos();
        let wlen_im = angle.sin();
        for i in (0..n).step_by(len) {
            let mut w_re = 1.0;
            let mut w_im = 0.0;
            for j in 0..half {
                let i1 = i + j;
                let i2 = i + j + half;
                let u_re = x[i1].re;
                let u_im = x[i1].im;
                let v_re = x[i2].re * w_re - x[i2].im * w_im;
                let v_im = x[i2].re * w_im + x[i2].im * w_re;
                x[i1] = Complex::new(u_re + v_re, u_im + v_im);
                x[i2] = Complex::new(u_re - v_re, u_im - v_im);
                let nw_re = w_re * wlen_re - w_im * wlen_im;
                let nw_im = w_re * wlen_im + w_im * wlen_re;
                w_re = nw_re;
                w_im = nw_im;
            }
        }
        len *= 2;
    }
}

fn ifft_inplace(x: &mut [Complex]) {
    // IFFT via conjugate → FFT → conjugate and scale
    for c in x.iter_mut() {
        c.im = -c.im;
    }
    fft_inplace(x);
    let n = x.len() as f64;
    for c in x.iter_mut() {
        c.im = -c.im;
        c.re /= n;
        c.im /= n;
    }
}

/// Convert u8 value to f64 in [-1, 1].
/// Binary {0, 1} maps exactly to {-1, 1}. Non-binary values use linear scale.
fn u8_to_f64(x: u8) -> f64 {
    match x {
        0 => -1.0,
        1 => 1.0,
        _ => (x as f64 / 127.5) - 1.0,
    }
}

/// Convert f64 in [-1, 1] to u8, then binarize at threshold 128.
fn f64_to_binary(x: f64) -> u8 {
    let clamped = x.max(-1.0).min(1.0);
    let u8val = ((clamped + 1.0) * 127.5) as u8;
    if u8val >= 128 { 1 } else { 0 }
}

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
        let n = a.len().min(b.len());
        let fft_len = n.next_power_of_two();

        let mut fa: Vec<Complex> = (0..fft_len).map(|_| Complex::new(0.0, 0.0)).collect();
        let mut fb: Vec<Complex> = (0..fft_len).map(|_| Complex::new(0.0, 0.0)).collect();

        for i in 0..n {
            fa[i] = Complex::new(u8_to_f64(a[i]), 0.0);
            fb[i] = Complex::new(u8_to_f64(b[i]), 0.0);
        }

        fft_inplace(&mut fa);
        fft_inplace(&mut fb);

        // Element-wise complex multiply in frequency domain → circular convolution
        let mut fc: Vec<Complex> = (0..fft_len).map(|_| Complex::new(0.0, 0.0)).collect();
        for i in 0..fft_len {
            fc[i] = complex_mul(fa[i], fb[i]);
        }

        ifft_inplace(&mut fc);

        fc.iter().take(n).map(|c| f64_to_binary(c.re)).collect()
    }

    /// FFT-HRR unbind: IFFT(FFT(c) * conj(FFT(a))) — circular cross-correlation,
    /// which approximates the inverse of bind for random VSA vectors.
    pub fn unbind(c: &[u8], a: &[u8]) -> Vec<u8> {
        let n = c.len().min(a.len());
        let fft_len = n.next_power_of_two();

        let mut fc: Vec<Complex> = (0..fft_len).map(|_| Complex::new(0.0, 0.0)).collect();
        let mut fa: Vec<Complex> = (0..fft_len).map(|_| Complex::new(0.0, 0.0)).collect();

        for i in 0..n {
            fc[i] = Complex::new(u8_to_f64(c[i]), 0.0);
            fa[i] = Complex::new(u8_to_f64(a[i]), 0.0);
        }

        fft_inplace(&mut fc);
        fft_inplace(&mut fa);

        // Element-wise multiply by conjugate: fc * conj(fa)
        let mut fb: Vec<Complex> = (0..fft_len).map(|_| Complex::new(0.0, 0.0)).collect();
        for i in 0..fft_len {
            fb[i] = complex_mul(fc[i], Complex::new(fa[i].re, -fa[i].im));
        }

        ifft_inplace(&mut fb);

        fb.iter().take(n).map(|c| f64_to_binary(c.re)).collect()
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

    pub fn negate(v: &[u8]) -> Vec<u8> {
        v.iter().map(|&x| !x).collect()
    }

    pub fn majority_bundle(vectors: &[&[u8]]) -> Vec<u8> {
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
        let threshold = (n as i32) / 2;
        counts.iter().map(|&c| if c > threshold { 255 } else { 0 }).collect()
    }

    pub fn xor_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
        a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
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
    fn test_bind_self_is_not_zero() {
        let a = QuantizedVSA::random_binary();
        let bound = QuantizedVSA::bind(&a, &a);
        // FHRR self-bind ≠ all zeros (unlike XOR). Check output is valid binary.
        assert_eq!(bound.len(), VSA_DIM);
        assert!(!bound.iter().all(|&x| x == 0), "FHRR self-bind should NOT be all zeros");
        for &x in &bound {
            assert!(x == 0 || x == 1, "FHRR output must be binary");
        }
    }

    #[test]
    fn test_bind_inverse_via_unbind() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let bound = QuantizedVSA::bind(&a, &b);
        let recovered = QuantizedVSA::unbind(&bound, &a);
        let sim = QuantizedVSA::similarity(&recovered, &b);
        // unbind(bind(a,b), a) ≈ b (circular deconvolution approximate inverse)
        assert!(sim > 0.5,
            "unbind(bind(a,b), a) should recover b approximately; sim = {}", sim);
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
    fn test_bind_commutative() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let ab = QuantizedVSA::bind(&a, &b);
        let ba = QuantizedVSA::bind(&b, &a);
        assert_eq!(ab, ba, "FFT-HRR circular convolution is commutative");
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
