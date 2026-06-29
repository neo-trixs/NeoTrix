use std::sync::OnceLock;

use super::e8_lattice::E8Lattice;

const VSA_DIM: usize = 4096;
pub(crate) const VSABYTES: usize = VSA_DIM / 8;
const CHUNKS: usize = 16;
const BITS_PER_CHUNK: usize = VSA_DIM / CHUNKS;
const BYTES_PER_CHUNK: usize = BITS_PER_CHUNK / 8;

/// A 4096-bit VSA vector compressed through E8 lattice quantization.
///
/// Encoding pipeline:
///   1. Split 4096 bits → 16 × 256-bit chunks
///   2. Each chunk → popcount → f64 ∈ [-1, 1]
///   3. Concatenate → 16D vector, split into two 8D halves
///   4. Each 8D half → nearest E8 root index + coefficient
///
/// Storage: 2 × u16 (indices) + 2 × f32 (coefficients) = **12 bytes**
/// vs 512 bytes raw = **~42× compression**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct E8Quantized {
    pub idx_a: u16,
    pub idx_b: u16,
    pub coef_a: f32,
    pub coef_b: f32,
}

impl E8Quantized {
    fn lattice() -> &'static E8Lattice {
        static LATTICE: OnceLock<E8Lattice> = OnceLock::new();
        LATTICE.get_or_init(E8Lattice::new)
    }

    /// Quantize a 4096-bit VSA vector into 12-byte E8-compressed form.
    pub fn quantize(bits: &[u8; VSABYTES]) -> Self {
        // Step 1: 512 bytes → 16 × f64 in [-1, 1]
        let mut vec16 = [0.0f64; 16];
        for chunk in 0..CHUNKS {
            let offset = chunk * BYTES_PER_CHUNK;
            let mut pop = 0u32;
            for i in 0..BYTES_PER_CHUNK {
                pop += bits[offset + i].count_ones();
            }
            vec16[chunk] = (pop as f64 / 128.0) - 1.0;
        }

        // Step 2: split into two 8D halves, quantize each to E8 root
        let mut half_a = [0.0f64; 8];
        let mut half_b = [0.0f64; 8];
        for i in 0..8 {
            half_a[i] = vec16[i];
            half_b[i] = vec16[i + 8];
        }

        let lat = Self::lattice();
        let (ia, _) = lat.nearest_root(&half_a);
        let (ib, _) = lat.nearest_root(&half_b);

        let root_a = lat.root(ia).as_f64();
        let root_b = lat.root(ib).as_f64();
        let coef_a = dot8(&half_a, &root_a) / 2.0;
        let coef_b = dot8(&half_b, &root_b) / 2.0;

        Self {
            idx_a: ia as u16,
            idx_b: ib as u16,
            coef_a: coef_a as f32,
            coef_b: coef_b as f32,
        }
    }

    /// Dequantize back to a 4096-bit VSA vector.
    ///
    /// Reconstruction: each 256-bit chunk is filled with `0x00` or `0xFF`
    /// based on the sign of the corresponding f64 coefficient.
    /// Returns a zero vector if either root index is out of bounds.
    pub fn dequantize(&self) -> [u8; VSABYTES] {
        let lat = Self::lattice();

        // Decode each half independently
        let half_a = match lat.decode_single(self.idx_a as usize, self.coef_a as f64) {
            Some(h) => h,
            None => return [0u8; VSABYTES],
        };
        let half_b = match lat.decode_single(self.idx_b as usize, self.coef_b as f64) {
            Some(h) => h,
            None => return [0u8; VSABYTES],
        };

        // Reconstruct 16D vector
        let mut vec16 = [0.0f64; 16];
        for i in 0..8 {
            vec16[i] = half_a[i];
            vec16[i + 8] = half_b[i];
        }

        // Expand back to 4096 bits: threshold at 0
        let mut bits = [0u8; VSABYTES];
        for chunk in 0..CHUNKS {
            let offset = chunk * BYTES_PER_CHUNK;
            let fill = if vec16[chunk] > 0.0 { 0xFFu8 } else { 0x00u8 };
            for i in 0..BYTES_PER_CHUNK {
                bits[offset + i] = fill;
            }
        }
        bits
    }

    /// Size in bytes of the compressed form.
    pub const fn compressed_size() -> usize {
        2 + 2 + 4 + 4 // 2 × u16 + 2 × f32
    }
}

fn dot8(a: &[f64; 8], b: &[f64; 8]) -> f64 {
    let mut s = 0.0f64;
    for i in 0..8 {
        s += a[i] * b[i];
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_vsa() -> [u8; VSABYTES] {
        let mut v = [0u8; VSABYTES];
        for i in 0..VSABYTES {
            v[i] = (i.wrapping_mul(17) ^ 0xAB) as u8;
        }
        v
    }

    #[test]
    fn test_quantize_roundtrip() {
        let v = random_vsa();
        let q = E8Quantized::quantize(&v);
        let _v2 = q.dequantize();
        // Structural preservation: dequantized vector should have
        // same chunk-level density pattern as original
        for chunk in 0..CHUNKS {
            let offset = chunk * BYTES_PER_CHUNK;
            let orig_pop: u32 = v[offset..offset + BYTES_PER_CHUNK]
                .iter()
                .map(|b| b.count_ones())
                .sum();
            let orig_sign = (orig_pop as f64 / 128.0) - 1.0;
            let dec_pop: u32 = _v2[offset..offset + BYTES_PER_CHUNK]
                .iter()
                .map(|b| b.count_ones())
                .sum();
            let dec_sign = (dec_pop as f64 / 128.0) - 1.0;
            assert!(
                (orig_sign > 0.0) == (dec_sign > 0.0),
                "chunk {} sign mismatch: orig={}, dec={}",
                chunk,
                orig_sign,
                dec_sign
            );
        }
    }

    #[test]
    fn test_compressed_size() {
        assert_eq!(E8Quantized::compressed_size(), 12);
    }

    #[test]
    fn test_quantize_all_zero() {
        let v = [0u8; VSABYTES];
        let q = E8Quantized::quantize(&v);
        let v2 = q.dequantize();
        // All-zero input → all chunks have popcount 0 → vec16 = -1.0
        // → threshold at 0 → all zeros
        assert!(v2.iter().all(|b| *b == 0));
    }

    #[test]
    fn test_quantize_all_ones() {
        let v = [0xFFu8; VSABYTES];
        let q = E8Quantized::quantize(&v);
        let v2 = q.dequantize();
        // All-ones → each chunk popcount = 256 → vec16 = 1.0
        // → threshold at 0 → all ones
        assert!(v2.iter().all(|b| *b == 0xFF));
    }

    #[test]
    fn test_similarity_preservation() {
        // Create two similar vectors: v2 flips only 10 bits in v1
        let mut v1 = [0u8; VSABYTES];
        for i in 0..VSABYTES {
            v1[i] = 0b1010_1010;
        }
        let mut v2 = v1;
        for i in 0..5 {
            v2[i] ^= 0b1100_0011; // flip 4 bits each in first 5 bytes
        }

        let q1 = E8Quantized::quantize(&v1);
        let q2 = E8Quantized::quantize(&v2);

        // Similar inputs should produce same root indices
        // (since only 20 bits out of 4096 differ → chunk pops change by <2)
        // Similar vectors should produce identical quantized indices
        assert!(
            q1.idx_a == q2.idx_a && q1.idx_b == q2.idx_b,
            "similar vectors should roundtrip to same E8 lattice point"
        );
    }

    #[test]
    fn test_lazy_lattice_init() {
        // Multiple calls should not panic
        let v = random_vsa();
        let _q1 = E8Quantized::quantize(&v);
        let _q2 = E8Quantized::quantize(&v);
    }

    #[test]
    fn test_deterministic() {
        let v = random_vsa();
        let q1 = E8Quantized::quantize(&v);
        let q2 = E8Quantized::quantize(&v);
        assert_eq!(q1, q2);
    }
}
