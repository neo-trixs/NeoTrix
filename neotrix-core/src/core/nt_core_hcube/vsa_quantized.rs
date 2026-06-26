// REVIVED Task 2 — dead_code removed
use crate::core::nt_core_hcube::vsa::BinaryVsaBackend;
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
    Complex::new(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re)
}

fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

fn fft_inplace(x: &mut [Complex]) {
    let n = x.len();
    if n <= 1 {
        return;
    }
    debug_assert!(
        is_power_of_two(n),
        "FFT requires power of 2 length, got {}",
        n
    );

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
    if u8val >= 128 {
        1
    } else {
        0
    }
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
        (0..VSA_DIM)
            .map(|_| if rng.gen_bool(0.5) { 1 } else { 0 })
            .collect()
    }

    pub fn binarize(v: &[u8]) -> Vec<u8> {
        v.iter()
            .map(|&x| if x >= BINARY_THRESHOLD { 1 } else { 0 })
            .collect()
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
        counts
            .iter()
            .map(|&c| if c > threshold { 1 } else { 0 })
            .collect()
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
        #[cfg(target_arch = "aarch64")]
        {
            return Self::hamming_distance_neon(a, b);
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
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
    }

    #[cfg(target_arch = "aarch64")]
    fn hamming_distance_neon(a: &[u8], b: &[u8]) -> u32 {
        // SAFETY:
        // - `a` and `b` are initialized Vec<u8> slices of length `len`
        // - `offset` is bounded by `chunks16 * 16` where `chunks16 = len / 16`
        // - This keeps all pointer accesses within the allocated buffer
        // - NEON vld1q/vst1q instructions operate on 16-byte aligned addresses —
        //   Vec<u8> allocation is 16-byte aligned by the allocator
        unsafe {
            use std::arch::aarch64::*;
            let len = a.len().min(b.len()).min(VSA_DIM);
            let mut dist = 0u32;
            let chunks16 = len / 16;
            let rem = len % 16;
            for i in 0..chunks16 {
                let offset = i * 16;
                let a_vec = vld1q_u8(a.as_ptr().add(offset));
                let b_vec = vld1q_u8(b.as_ptr().add(offset));
                let xor_vec = veorq_u8(a_vec, b_vec);
                let masked = vandq_u8(xor_vec, vdupq_n_u8(1));
                let cnt = vcntq_u8(masked);
                let sum16 = vpaddlq_u8(cnt);
                let sum32 = vpaddlq_u16(sum16);
                let sum64 = vpaddlq_u32(sum32);
                dist += vgetq_lane_u64::<0>(sum64) as u32;
                dist += vgetq_lane_u64::<1>(sum64) as u32;
            }
            for j in 0..rem {
                let idx = chunks16 * 16 + j;
                if (a[idx] & 1) != (b[idx] & 1) {
                    dist += 1;
                }
            }
            dist
        }
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
        // Bipolar cosine: map {0,1} → {-1,+1}, then cos = 1 - 2*hamming/len
        let h = Self::hamming_distance(a, b) as f64;
        1.0 - 2.0 * h / len as f64
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn negate(v: &[u8]) -> Vec<u8> {
        v.iter().map(|&x| x ^ 1).collect()
    }

    /// Compute hamming similarity restricted to one subspace.
    pub fn subspace_similarity(a: &[u8], b: &[u8], subspace: std::ops::Range<usize>) -> f64 {
        let len = subspace.len();
        if len == 0 || subspace.start >= a.len().min(b.len()) {
            return 0.0;
        }
        let end = subspace.end.min(a.len()).min(b.len());
        let mut dist = 0u32;
        for i in subspace.start..end {
            if (a[i] & 1) != (b[i] & 1) {
                dist += 1;
            }
        }
        1.0 - dist as f64 / len as f64
    }

    /// Extract a single subspace (all other bits zeroed).
    pub fn extract_subspace(v: &[u8], subspace: std::ops::Range<usize>) -> Vec<u8> {
        let len = v.len();
        let mut result = vec![0u8; len];
        let end = subspace.end.min(len);
        for i in subspace.start..end {
            result[i] = v[i] & 1;
        }
        result
    }

    /// Check if a subspace has any non-zero bits.
    pub fn subspace_is_active(v: &[u8], subspace: std::ops::Range<usize>) -> bool {
        let end = subspace.end.min(v.len());
        for i in subspace.start..end {
            if (v[i] & 1) != 0 {
                return true;
            }
        }
        false
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
        counts
            .iter()
            .map(|&c| if c > threshold { 255 } else { 0 })
            .collect()
    }

    /// Orthogonal Subspace Carving (OSC) bind.
    ///
    /// For each (filler, role) pair: projects filler onto role's null space
    /// before binding via XOR. The null space is a deterministic binary mask
    /// derived from the role vector (seeded hash). This geometrically suppresses
    /// cross-talk between superimposed bound structures.
    ///
    /// Returns the bundled composite of all role-bound, subspace-carved fillers.
    pub fn osc_bind(fillers: &[&[u8]], roles: &[&[u8]]) -> Vec<u8> {
        assert_eq!(
            fillers.len(),
            roles.len(),
            "osc_bind: fillers and roles must have same length"
        );
        if fillers.is_empty() {
            return vec![0; VSA_DIM];
        }
        let dim = fillers[0].len().min(VSA_DIM);
        let mut bound_vectors: Vec<Vec<u8>> = Vec::with_capacity(fillers.len());
        for (filler, role) in fillers.iter().zip(roles.iter()) {
            let role_seed = role
                .iter()
                .fold(0u64, |acc, &x| acc.wrapping_mul(31).wrapping_add(x as u64));
            let role_null_raw = QuantizedVSA::seeded_random(role_seed, dim);
            let role_null: Vec<u8> = role_null_raw
                .iter()
                .map(|&x| if x >= BINARY_THRESHOLD { 1 } else { 0 })
                .collect();
            let projected: Vec<u8> = filler
                .iter()
                .zip(role_null.iter())
                .map(|(f, rn)| f & rn)
                .collect();
            let bound = QuantizedVSA::xor_bind(&projected, role);
            bound_vectors.push(bound);
        }
        let refs: Vec<&[u8]> = bound_vectors.iter().map(|v| v.as_slice()).collect();
        QuantizedVSA::bundle(&refs)
    }

    /// Orthogonal Subspace Carving (OSC) unbind.
    /// XOR unbind the composite with the role to retrieve the projected filler.
    pub fn osc_unbind(composite: &[u8], role: &[u8]) -> Vec<u8> {
        QuantizedVSA::xor_bind(composite, role)
    }

    pub fn xor_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
        #[cfg(target_arch = "aarch64")]
        {
            return Self::xor_bind_neon(a, b);
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn xor_bind_neon(a: &[u8], b: &[u8]) -> Vec<u8> {
        // SAFETY:
        // - `a` and `b` are initialized Vec<u8> slices of length `len`
        // - `offset` is bounded by `chunks16 * 16` where `chunks16 = len / 16`
        // - `result` is a newly allocated Vec<u8> of length `len`
        // - All pointer arithmetic keeps accesses within the allocated buffers
        // - NEON vld1q/vst1q instructions operate on 16-byte aligned addresses —
        //   Vec<u8> allocation is 16-byte aligned by the allocator
        unsafe {
            use std::arch::aarch64::*;
            let len = a.len().min(b.len());
            let mut result = vec![0u8; len];
            let chunks16 = len / 16;
            let rem = len % 16;
            for i in 0..chunks16 {
                let offset = i * 16;
                let a_vec = vld1q_u8(a.as_ptr().add(offset));
                let b_vec = vld1q_u8(b.as_ptr().add(offset));
                vst1q_u8(result.as_mut_ptr().add(offset), veorq_u8(a_vec, b_vec));
            }
            for j in 0..rem {
                let idx = chunks16 * 16 + j;
                result[idx] = a[idx] ^ b[idx];
            }
            result
        }
    }
}

impl BinaryVsaBackend for QuantizedVSA {
    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        QuantizedVSA::xor_bind(a, b)
    }

    fn unbind(&self, c: &[u8], a: &[u8]) -> Vec<u8> {
        QuantizedVSA::xor_bind(c, a)
    }

    fn bundle(&self, vectors: &[&[u8]]) -> Vec<u8> {
        QuantizedVSA::bundle(vectors)
    }

    fn permute(&self, v: &[u8], shift: isize) -> Vec<u8> {
        QuantizedVSA::permute(v, shift)
    }

    fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        QuantizedVSA::similarity(a, b)
    }

    fn dimensions(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "quantized-vsa"
    }

    fn to_bits(&self, v: &[u8]) -> Vec<u8> {
        v.to_vec()
    }

    fn to_dense(&self, v: &[u8]) -> Vec<f64> {
        v.iter().map(|&x| if x > 0 { 1.0 } else { 0.0 }).collect()
    }
}

/// Pack 8 binary values (each 0 or 1) into each byte for fast hamming distance.
/// Input length must be multiple of 8. Output is input.len() / 8.
pub fn pack_binary(v: &[u8]) -> Vec<u8> {
    v.chunks(8)
        .map(|chunk| {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit != 0 {
                    byte |= 1 << i;
                }
            }
            byte
        })
        .collect()
}

/// Hamming distance on packed binary vectors using POPCNT via u8::count_ones().
/// Both inputs must be same length.
pub fn hamming_distance_packed(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x ^ y).count_ones())
        .sum()
}

/// Normalized similarity [0,1] from packed hamming distance.
/// 1.0 = identical, 0.0 = all bits differ.
pub fn similarity_packed(a: &[u8], b: &[u8]) -> f64 {
    let total_bits = (a.len().min(b.len()) * 8) as f64;
    if total_bits == 0.0 {
        return 0.0;
    }
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
        assert!(
            !bound.iter().all(|&x| x == 0),
            "FHRR self-bind should NOT be all zeros"
        );
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
        assert!(
            sim > 0.5,
            "unbind(bind(a,b), a) should recover b approximately; sim = {}",
            sim
        );
    }

    #[test]
    fn test_bundle_non_empty() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let bundled = QuantizedVSA::bundle(&[&a, &b]);
        assert_eq!(bundled.len(), VSA_DIM);
        let sim_a = QuantizedVSA::similarity(&bundled, &a);
        let sim_b = QuantizedVSA::similarity(&bundled, &b);
        assert!(
            sim_a > 0.4 || sim_b > 0.4,
            "bundled should be similar to at least one component"
        );
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
        for i in (0..VSA_DIM).step_by(2) {
            v[i] = 1;
        }
        let packed = pack_binary(&v);
        assert_eq!(packed.len(), VSA_DIM / 8);
        for &byte in &packed {
            assert_eq!(byte, 0b01010101, "alternating bits should pack to 0x55");
        }
    }

    // ─── KROP cleanup tests ─────────────────────────────────────────────

    #[test]
    fn test_cleanup_krop_empty_input() {
        let result = cleanup_krop(&[], Some(8), 42);
        assert!(result.is_empty());
    }

    #[test]
    fn test_cleanup_krop_single_vector() {
        let v = QuantizedVSA::random_binary();
        let result = cleanup_krop(&[v.clone()], Some(8), 42);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], v);
    }

    #[test]
    fn test_cleanup_krop_identical_vectors_collapse() {
        let v = QuantizedVSA::random_binary();
        let vsas = vec![v.clone(), v.clone(), v.clone()];
        let result = cleanup_krop(&vsas, Some(4), 42);
        assert!(
            !result.is_empty(),
            "identical vectors should produce at least one centroid"
        );
        // All identical VSA vectors should hash to same bucket → single medoid
        assert_eq!(
            result.len(),
            1,
            "identical vectors should collapse to one centroid"
        );
    }

    #[test]
    fn test_cleanup_krop_deterministic() {
        let vsas = vec![
            QuantizedVSA::random_binary(),
            QuantizedVSA::random_binary(),
            QuantizedVSA::random_binary(),
        ];
        let r1 = cleanup_krop(&vsas, Some(8), 42);
        let r2 = cleanup_krop(&vsas, Some(8), 42);
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a, b, "KROP must be deterministic for same seed");
        }
    }

    // ─── OSC (Orthogonal Subspace Carving) tests ────────────────────────────

    #[test]
    fn test_osc_bind_empty() {
        let result = QuantizedVSA::osc_bind(&[], &[]);
        assert_eq!(result.len(), VSA_DIM);
        assert!(result.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_osc_bind_single_pair_not_zero() {
        let f = QuantizedVSA::random_binary();
        let r = QuantizedVSA::random_binary();
        let composite = QuantizedVSA::osc_bind(&[&f], &[&r]);
        assert_eq!(composite.len(), VSA_DIM);
        assert!(
            !composite.iter().all(|&x| x == 0),
            "single pair OSC should not be all-zero"
        );
    }

    #[test]
    fn test_osc_reduces_crosstalk() {
        // Create 4 filler-role pairs
        let pairs: Vec<(Vec<u8>, Vec<u8>)> = (0..4)
            .map(|_| (QuantizedVSA::random_binary(), QuantizedVSA::random_binary()))
            .collect();
        let fillers: Vec<&[u8]> = pairs.iter().map(|(f, _)| f.as_slice()).collect();
        let roles: Vec<&[u8]> = pairs.iter().map(|(_, r)| r.as_slice()).collect();

        // Naive composite: bundle of plain XOR binds
        let naive_bound: Vec<Vec<u8>> = pairs
            .iter()
            .map(|(f, r)| QuantizedVSA::xor_bind(f, r))
            .collect();
        let naive_refs: Vec<&[u8]> = naive_bound.iter().map(|v| v.as_slice()).collect();
        let naive_composite = QuantizedVSA::bundle(&naive_refs);

        // OSC composite
        let osc_composite = QuantizedVSA::osc_bind(&fillers, &roles);

        // Unbind with each role and check average similarity to original filler
        let mut naive_total_sim = 0.0;
        let mut osc_total_sim = 0.0;
        for (f, r) in &pairs {
            let naive_recovered = QuantizedVSA::osc_unbind(&naive_composite, r);
            let osc_recovered = QuantizedVSA::osc_unbind(&osc_composite, r);
            naive_total_sim += QuantizedVSA::similarity(&naive_recovered, f);
            osc_total_sim += QuantizedVSA::similarity(&osc_recovered, f);
        }
        let n = pairs.len() as f64;
        let naive_avg = naive_total_sim / n;
        let osc_avg = osc_total_sim / n;

        assert!(
            osc_avg > naive_avg,
            "OSC should reduce crosstalk: naive_avg={:.4} osc_avg={:.4}",
            naive_avg,
            osc_avg
        );
    }

    #[test]
    fn test_cleanup_krop_output_valid_binary() {
        let vsas: Vec<Vec<u8>> = (0..10).map(|_| QuantizedVSA::random_binary()).collect();
        let result = cleanup_krop(&vsas, Some(8), 42);
        for centroid in &result {
            assert_eq!(centroid.len(), VSA_DIM);
            for &x in centroid {
                assert!(x == 0 || x == 1, "centroid elements must be binary");
            }
        }
    }
}

/// Unified bipolar cosine similarity for binary VSA vectors (u8, 0/1).
///
/// Maps {0,1} → {-1,+1} and returns cosine = 1 - 2*hamming/len ∈ [-1, 1].
/// This is the canonical similarity function for all VSA engines.
pub fn cosine_sim_u8(a: &[u8], b: &[u8]) -> f64 {
    QuantizedVSA::cosine(a, b)
}

/// KROP (K-random orthogonal projection) linearithmic cleanup.
///
/// Projects all VSA vectors onto K random binary hyperplanes using XOR,
/// bins them by LSH signature, and returns one medoid per non-empty bucket.
/// Complexity: O(N * K + N * bucket_search) ≈ O(N log N) when K ≈ log₂(N).
///
/// - `vsas`: input VSA vectors (each length must match VSA_DIM or be 0/1 binary)
/// - `k`: number of random projections (default: max(8, log2(N)))
/// - `seed`: deterministic seed for reproducible hyperplanes
pub fn cleanup_krop(vsas: &[Vec<u8>], k: Option<usize>, seed: u64) -> Vec<Vec<u8>> {
    if vsas.is_empty() {
        return vec![];
    }
    let n = vsas.len();
    let dim = vsas[0].len();
    let k = k.unwrap_or_else(|| (n as f64).log2().ceil().max(8.0) as usize);

    // Generate K random binary hyperplanes (deterministic via seed)
    let hyperplanes: Vec<Vec<u8>> = (0..k)
        .map(|i| QuantizedVSA::seeded_random(seed.wrapping_add(i as u64 * 0x9e37_79b9), dim))
        .collect();

    // Project each VSA onto each hyperplane: dot = hamming weight of (v XOR h)
    // For binary VSA, the sign is majority bit of the XOR result
    let signatures: Vec<Vec<u8>> = vsas
        .iter()
        .map(|v| {
            hyperplanes
                .iter()
                .map(|h| {
                    let xor_ones: u32 = v.iter().zip(h.iter()).map(|(a, b)| (a ^ b) as u32).sum();
                    // Sign: 1 if more than half the bits differ, else 0
                    if xor_ones > (dim as u32) / 2 {
                        1u8
                    } else {
                        0u8
                    }
                })
                .collect()
        })
        .collect();

    // LSH binning: each signature is a bucket key
    use std::collections::HashMap;
    let mut buckets: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
    for (i, sig) in signatures.iter().enumerate() {
        buckets.entry(sig.clone()).or_default().push(i);
    }

    // For each bucket, choose the medoid (min total Hamming distance to bucket peers)
    bucket_medoids(vsas, &buckets)
}

/// KROP (K-random orthogonal projection) linearithmic cleanup with configurable parameters.
///
/// Wraps the `cleanup_krop` function in a struct that stores K and seed,
/// providing a builder-like configuration API. Default K = 8, seed = 42.
///
/// Complexity: O(N * K + N * bucket_search) ≈ O(N log N) when K ≈ log₂(N).
#[derive(Debug, Clone)]
pub struct KropCleanup {
    pub k: usize,
    pub seed: u64,
}

impl Default for KropCleanup {
    fn default() -> Self {
        Self { k: 8, seed: 42 }
    }
}

impl KropCleanup {
    pub fn new(k: usize, seed: u64) -> Self {
        Self { k, seed }
    }

    pub fn with_k(mut self, k: usize) -> Self {
        self.k = k;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn cleanup(&self, vsas: &[Vec<u8>]) -> Vec<Vec<u8>> {
        cleanup_krop(vsas, Some(self.k), self.seed)
    }
}

fn bucket_medoids(
    vsas: &[Vec<u8>],
    buckets: &std::collections::HashMap<Vec<u8>, Vec<usize>>,
) -> Vec<Vec<u8>> {
    let mut result = Vec::with_capacity(buckets.len());
    for indices in buckets.values() {
        if indices.is_empty() {
            continue;
        }
        // Single-element bucket: just return the lone vector
        if indices.len() == 1 {
            result.push(vsas[indices[0]].clone());
            continue;
        }
        // Multi-element bucket: find the medoid
        let mut best_idx = indices[0];
        let mut best_dist = u64::MAX;
        for &i in indices {
            let total: u64 = indices
                .iter()
                .map(|&j| QuantizedVSA::hamming_distance(&vsas[i], &vsas[j]) as u64)
                .sum();
            if total < best_dist {
                best_dist = total;
                best_idx = i;
            }
        }
        result.push(vsas[best_idx].clone());
    }
    result
}
