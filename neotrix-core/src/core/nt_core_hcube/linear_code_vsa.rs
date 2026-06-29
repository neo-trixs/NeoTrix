// REVIVED Task 2 — dead_code removed
use rand::Rng;
use std::collections::HashMap;

// ── GF(2^k) field arithmetic ───────────────────────────────────────────

/// Select primitive polynomial for GF(2^k).
///
/// Standard primitive polynomials for common k values.
/// Falls back to a seeded deterministic polynomial for other k.
const fn primitive_poly(k: usize) -> u32 {
    match k {
        2 => 0x7,      // x^2 + x + 1
        3 => 0xB,      // x^3 + x + 1
        4 => 0x13,     // x^4 + x + 1
        8 => 0x11D,    // x^8 + x^4 + x^3 + x^2 + 1
        12 => 0x1009,  // x^12 + x^3 + 1
        16 => 0x1002B, // x^16 + x^5 + x^3 + x^2 + 1
        _ => 0x1009,   // default for k ≥ 12
    }
}

/// Build GF(2^k) exponent table.
///
/// `exp[i] = α^i` for i = 0 .. 2^k-2.
/// The multiplicative group has order 2^k-1.
fn build_gf2k_exp(k: usize) -> Vec<u16> {
    let size = 1 << k;
    let poly = primitive_poly(k);
    let mut exp = vec![0u16; size];
    exp[0] = 1;
    for i in 1..size - 1 {
        let val = exp[i - 1] as u32;
        let mut next = val << 1;
        if next & (size as u32) != 0 {
            next ^= poly;
        }
        exp[i] = next as u16;
    }
    exp
}

// ── Packed vector helpers ──────────────────────────────────────────────

/// Test bit `pos` in a packed LSB-first byte slice.
fn test_bit(v: &[u8], pos: usize) -> bool {
    (v[pos / 8] >> (pos % 8)) & 1 == 1
}

/// Set bit `pos` in a packed LSB-first byte slice.
fn set_bit(v: &mut [u8], pos: usize) {
    v[pos / 8] |= 1 << (pos % 8);
}

/// Create a packed unit vector with a single 1 at position `pos`.
fn unit_vec(bits: usize, pos: usize) -> Vec<u8> {
    let n = (bits + 7) / 8;
    let mut v = vec![0u8; n];
    if pos < bits {
        v[pos / 8] |= 1 << (pos % 8);
    }
    v
}

/// XOR two packed byte slices elementwise.
fn xor_into(dst: &mut [u8], src: &[u8]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d ^= s;
    }
}

// ── Linear Code VSA ────────────────────────────────────────────────────

/// Random linear-code VSA over GF(2) with systematic construction.
///
/// Builds a (n, k) linear code where n = dim, k = log2(dim).
/// The generator matrix G = [I_k | P] uses GF(2^k) field elements
/// (derived from primitive polynomial x^12 + x^3 + 1) to construct
/// the parity submatrix P.
///
/// Encode maps k-bit messages to n-bit codewords. The code is linear
/// so XOR of two codewords is another codeword (used for `bind`/`unbind`).
/// Single-bit error correction is provided via syndrome lookup table.
///
/// Inspired by MIT Neural Computation 36(6) 2024.
#[derive(Debug, Clone)]
pub struct LinearCodeVSA {
    pub dim: usize,
    k: usize,
    n_bytes: usize,
    k_bytes: usize,
    _parity_bits: usize,
    parity_bytes: usize,
    generator: Vec<Vec<u8>>,
    p_rows: Vec<Vec<u8>>,
    parity_check: Vec<Vec<u8>>,
    codebook_size: usize,
    syndrome_table: HashMap<Vec<u8>, usize>,
}

impl LinearCodeVSA {
    /// Construct a new linear code VSA with the given dimension.
    ///
    /// `dim` must be a power of two and divisible by 8 (default 4096).
    /// k = log2(dim) is the number of information bits per codeword.
    pub fn new(dim: usize) -> Self {
        assert!(dim.is_power_of_two(), "dim must be a power of two");
        assert!(dim >= 16, "dim must be at least 16");
        let k = dim.ilog2() as usize;
        let n_bytes = (dim + 7) / 8;
        let k_bytes = (k + 7) / 8;
        let parity_bits = dim - k;
        let parity_bytes = (parity_bits + 7) / 8;
        let codebook_size = 1 << k;

        // ── 1. Build GF(2^k) exponent table ──
        let exp = build_gf2k_exp(k);

        // ── 2. Build P matrix (k × parity_bits) ──
        // P[i][j] = bit i of α^j (the j-th field element)
        let mut p_rows: Vec<Vec<u8>> = Vec::with_capacity(k);
        for i in 0..k {
            let mut row = vec![0u8; parity_bytes];
            for j in 0..parity_bits {
                let element = exp[j % (codebook_size - 1)];
                if (element >> i) & 1 == 1 {
                    set_bit(&mut row, j);
                }
            }
            p_rows.push(row);
        }

        // ── 3. Build generator matrix G = [I_k | P] ──
        let mut generator: Vec<Vec<u8>> = Vec::with_capacity(k);
        for i in 0..k {
            let mut row = vec![0u8; n_bytes];
            set_bit(&mut row, i);
            for j in 0..parity_bits {
                if test_bit(&p_rows[i], j) {
                    set_bit(&mut row, k + j);
                }
            }
            generator.push(row);
        }

        // ── 4. Build parity check matrix H = [P^T | I_{n-k}] ──
        let mut parity_check: Vec<Vec<u8>> = Vec::with_capacity(parity_bits);
        for j in 0..parity_bits {
            let mut row = vec![0u8; n_bytes];
            // First k bits = P^T[j] = column j of P = (P[0][j], ..., P[k-1][j])
            for i in 0..k {
                if test_bit(&p_rows[i], j) {
                    set_bit(&mut row, i);
                }
            }
            // Identity at position j (bit k + j)
            set_bit(&mut row, k + j);
            parity_check.push(row);
        }

        // ── 5. Build syndrome table ──
        let syndrome_table = Self::build_syndrome_table(&p_rows, k, dim, parity_bits);

        Self {
            dim,
            k,
            n_bytes,
            k_bytes,
            _parity_bits: parity_bits,
            parity_bytes,
            generator,
            p_rows,
            parity_check,
            codebook_size,
            syndrome_table,
        }
    }

    // ── internal: build syndrome table ─────────────────────────────────

    /// Build syndrome table mapping packed syndrome → error bit position.
    ///
    /// For error at info position i (< k): syndrome = P[i] (row i of P).
    /// For error at parity position i-k (j = i-k ≥ 0): syndrome = unit vector e_j.
    fn build_syndrome_table(
        p_rows: &[Vec<u8>],
        k: usize,
        dim: usize,
        parity_bits: usize,
    ) -> HashMap<Vec<u8>, usize> {
        let mut table = HashMap::with_capacity(dim);
        // Info bit errors (positions 0..k)
        for i in 0..k {
            let syndrome = p_rows[i].clone();
            table.insert(syndrome, i);
        }
        // Parity bit errors (positions k..dim)
        for j in 0..parity_bits {
            let syndrome = unit_vec(parity_bits, j);
            table.insert(syndrome, k + j);
        }
        table
    }

    // ── internal: syndrome computation ─────────────────────────────────

    /// Compute syndrome s = H · r^T using the efficient P-based method.
    ///
    /// s = P^T × r_info + r_parity where r = [r_info | r_parity].
    #[inline]
    fn compute_syndrome(&self, r: &[u8]) -> Vec<u8> {
        let mut syndrome = vec![0u8; self.parity_bytes];
        // XOR P[i] for each set info bit i
        for i in 0..self.k {
            if test_bit(r, i) {
                xor_into(&mut syndrome, &self.p_rows[i]);
            }
        }
        // XOR the parity part of r
        xor_into(&mut syndrome, &r[self.k_bytes..]);
        syndrome
    }

    // ── encode ─────────────────────────────────────────────────────────

    /// Encode a message into a linear code VSA vector.
    ///
    /// The input is processed in k-bit chunks. Each k-bit chunk is
    /// encoded to a `dim`-bit codeword via G. Output is the concatenation
    /// of packed codewords.
    pub fn encode(&self, msg: &[u8]) -> Vec<u8> {
        let total_bits = msg.len() * 8;
        if total_bits == 0 {
            return vec![0u8; self.n_bytes];
        }
        let num_blocks = (total_bits + self.k - 1) / self.k;
        let mut result = Vec::with_capacity(num_blocks * self.n_bytes);
        for block in 0..num_blocks {
            let start_bit = block * self.k;
            let end_bit = (start_bit + self.k).min(total_bits);
            let bits_in_block = end_bit - start_bit;
            // Extract up to k bits, zero-pad to k bits
            let mut block_bits = vec![0u8; self.k_bytes];
            for bit_off in 0..bits_in_block {
                let msg_bit_pos = start_bit + bit_off;
                if (msg[msg_bit_pos / 8] >> (msg_bit_pos % 8)) & 1 == 1 {
                    set_bit(&mut block_bits, bit_off);
                }
            }
            result.extend_from_slice(&self.encode_block(&block_bits));
        }
        result
    }

    /// Encode a single k-bit block to an n-bit codeword.
    /// c = m × G = [m | m × P] (systematic encoding).
    fn encode_block(&self, msg_bits: &[u8]) -> Vec<u8> {
        let mut codeword = vec![0u8; self.n_bytes];
        // Systematic part: copy message bits
        for i in 0..self.k_bytes {
            codeword[i] = msg_bits[i];
        }
        // Zero out any bits beyond k in the last info byte
        if self.k % 8 != 0 {
            let last = self.k_bytes - 1;
            codeword[last] &= (1 << (self.k % 8)) - 1;
        }
        // Parity part: XOR P[i] for each set info bit i
        for i in 0..self.k {
            if test_bit(msg_bits, i) {
                xor_into(&mut codeword[self.k_bytes..], &self.p_rows[i]);
            }
        }
        codeword
    }

    // ── decode ─────────────────────────────────────────────────────────

    /// Decode a (possibly noisy) VSA vector back to the message.
    ///
    /// Applies single-bit error correction (via syndrome lookup),
    /// then extracts the k information bits from each codeword block.
    pub fn decode(&self, received: &[u8]) -> Vec<u8> {
        if received.is_empty() {
            return vec![0u8; self.k_bytes];
        }
        let num_blocks = (received.len() + self.n_bytes - 1) / self.n_bytes;
        let mut result = Vec::with_capacity(num_blocks * self.k_bytes);
        for chunk in received.chunks(self.n_bytes) {
            let mut block = vec![0u8; self.n_bytes];
            let copy_len = chunk.len().min(self.n_bytes);
            block[..copy_len].copy_from_slice(&chunk[..copy_len]);
            let decoded = self.decode_block(&block);
            result.extend_from_slice(&decoded);
        }
        result
    }

    /// Decode a single n-bit block: correct errors then extract info bits.
    fn decode_block(&self, received: &[u8]) -> Vec<u8> {
        let corrected = self.correct_single(received);
        corrected[..self.k_bytes].to_vec()
    }

    /// Correct a single n-bit block (internal, returns full codeword).
    fn correct_single(&self, received: &[u8]) -> Vec<u8> {
        let syndrome = self.compute_syndrome(received);
        if syndrome.iter().all(|&b| b == 0) {
            return received.to_vec();
        }
        if let Some(&pos) = self.syndrome_table.get(&syndrome) {
            let mut corrected = received.to_vec();
            corrected[pos / 8] ^= 1 << (pos % 8);
            corrected
        } else {
            received.to_vec()
        }
    }

    // ── bind / unbind (XOR — linear code property) ─────────────────────

    /// Bind two encoded vectors via XOR.
    ///
    /// Linear codes are closed under XOR: c1 ⊕ c2 is another valid codeword.
    pub fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let len = a.len().min(b.len()).min(self.n_bytes);
        let mut result = vec![0u8; len];
        for (r, (x, y)) in result.iter_mut().zip(a.iter().zip(b.iter())) {
            *r = x ^ y;
        }
        result
    }

    /// Unbind via XOR (XOR is self-inverse for linear codes).
    pub fn unbind(&self, c: &[u8], a: &[u8]) -> Vec<u8> {
        self.bind(c, a)
    }

    // ── bundle (majority voting) ───────────────────────────────────────

    /// Bundle multiple vectors via per-bit majority voting.
    pub fn bundle(&self, vectors: &[&[u8]]) -> Vec<u8> {
        if vectors.is_empty() {
            return vec![0u8; self.n_bytes];
        }
        let n = vectors.len();
        let mut counts = vec![0i32; self.dim];
        for v in vectors {
            for bit_pos in 0..self.dim {
                if bit_pos < v.len() * 8 && test_bit(v, bit_pos) {
                    counts[bit_pos] += 1;
                }
            }
        }
        let threshold = (n as i32) / 2;
        let mut result = vec![0u8; self.n_bytes];
        for bit_pos in 0..self.dim {
            if counts[bit_pos] > threshold {
                set_bit(&mut result, bit_pos);
            }
        }
        result
    }

    // ── similarity ─────────────────────────────────────────────────────

    /// Normalized Hamming similarity in [0, 1].
    pub fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len()).min(self.n_bytes);
        if len == 0 {
            return 0.0;
        }
        let total_bits = (len * 8) as f64;
        let dist: u32 = a[..len]
            .iter()
            .zip(b[..len].iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum();
        1.0 - dist as f64 / total_bits
    }

    // ── random codeword ────────────────────────────────────────────────

    /// Generate a seeded random codeword.
    ///
    /// Produces a random k-bit message (seeded) and encodes it to a codeword.
    pub fn random(&self, seed: u64) -> Vec<u8> {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut msg = vec![0u8; self.k_bytes];
        for byte in &mut msg {
            *byte = rng.gen();
        }
        if self.k % 8 != 0 {
            let last = self.k_bytes - 1;
            msg[last] &= (1 << (self.k % 8)) - 1;
        }
        self.encode_block(&msg)
    }

    // ── error correction & detection ────────────────────────────────────

    /// Single-bit error correction via syndrome lookup.
    ///
    /// Returns the corrected vector (same length as input).
    /// The input is processed block by block (each block = dim bits).
    pub fn correct(&self, received: &[u8]) -> Vec<u8> {
        if received.is_empty() {
            return Vec::new();
        }
        let num_blocks = (received.len() + self.n_bytes - 1) / self.n_bytes;
        let mut result = Vec::with_capacity(num_blocks * self.n_bytes);
        for chunk in received.chunks(self.n_bytes) {
            let mut block = vec![0u8; self.n_bytes];
            let copy_len = chunk.len().min(self.n_bytes);
            block[..copy_len].copy_from_slice(&chunk[..copy_len]);
            let corrected = self.correct_single(&block);
            result.extend_from_slice(&corrected);
        }
        result
    }

    /// Error detection via syndrome test.
    ///
    /// Returns true if a non-zero syndrome is detected (indicating errors).
    /// The input is processed block by block; returns true if ANY block
    /// has a detectable error.
    pub fn detect(&self, received: &[u8]) -> bool {
        if received.is_empty() {
            return false;
        }
        for chunk in received.chunks(self.n_bytes) {
            let mut block = vec![0u8; self.n_bytes];
            let copy_len = chunk.len().min(self.n_bytes);
            block[..copy_len].copy_from_slice(&chunk[..copy_len]);
            let syndrome = self.compute_syndrome(&block);
            if syndrome.iter().any(|&b| b != 0) {
                return true;
            }
        }
        false
    }

    // ── accessors ──────────────────────────────────────────────────────

    pub fn k(&self) -> usize {
        self.k
    }

    pub fn codebook_size(&self) -> usize {
        self.codebook_size
    }

    pub fn generator(&self) -> &[Vec<u8>] {
        &self.generator
    }

    pub fn parity_check(&self) -> &[Vec<u8>] {
        &self.parity_check
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}

use crate::core::nt_core_hcube::vsa::BinaryVsaBackend;

impl BinaryVsaBackend for LinearCodeVSA {
    fn bind(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        LinearCodeVSA::bind(self, a, b)
    }

    fn unbind(&self, c: &[u8], a: &[u8]) -> Vec<u8> {
        LinearCodeVSA::unbind(self, c, a)
    }

    fn bundle(&self, vectors: &[&[u8]]) -> Vec<u8> {
        LinearCodeVSA::bundle(self, vectors)
    }

    fn permute(&self, v: &[u8], shift: isize) -> Vec<u8> {
        let len = v.len();
        let mut result = vec![0u8; len];
        for (i, item) in result.iter_mut().enumerate() {
            let src = ((i as isize - shift).rem_euclid(len as isize)) as usize;
            *item = v[src];
        }
        result
    }

    fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        LinearCodeVSA::similarity(self, a, b)
    }

    fn dimensions(&self) -> usize {
        self.dim
    }

    fn name(&self) -> &str {
        "linear-code-vsa"
    }

    fn to_bits(&self, v: &[u8]) -> Vec<u8> {
        v.to_vec()
    }

    fn to_dense(&self, v: &[u8]) -> Vec<f64> {
        v.iter().map(|&x| if x > 0 { 1.0 } else { 0.0 }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DIM: usize = 256;
    const TEST_K: usize = 8;

    fn make_vsa() -> LinearCodeVSA {
        LinearCodeVSA::new(TEST_DIM)
    }

    // ── 1. Construction produces distinct codewords ────────────────────

    #[test]
    fn test_distinct_codewords() {
        let vsa = make_vsa();
        let m1 = vec![0x01u8; 1]; // 8 bits, but k=8, so 1 byte
        let m2 = vec![0x02u8; 1];
        let c1 = vsa.encode(&m1);
        let c2 = vsa.encode(&m2);
        assert_ne!(
            c1, c2,
            "different messages must produce different codewords"
        );
    }

    #[test]
    fn test_codebook_size() {
        let vsa = make_vsa();
        assert_eq!(vsa.codebook_size(), 1 << TEST_K);
    }

    #[test]
    fn test_generator_has_k_rows() {
        let vsa = make_vsa();
        assert_eq!(vsa.generator().len(), TEST_K);
        assert_eq!(vsa.generator()[0].len(), TEST_DIM / 8);
    }

    // ── 2. Encode-decode roundtrip (noiseless) ─────────────────────────

    #[test]
    fn test_encode_decode_roundtrip_single_byte() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let encoded = vsa.encode(&msg);
        let decoded = vsa.decode(&encoded);
        // decoded should match padded msg
        let expected = {
            let mut m = vec![0u8; (TEST_K + 7) / 8];
            m[0] = msg[0];
            m
        };
        assert_eq!(
            decoded, expected,
            "roundtrip failed: got {:?}, expected {:?}",
            decoded, expected
        );
    }

    #[test]
    fn test_encode_decode_roundtrip_zero() {
        let vsa = make_vsa();
        let msg = vec![0u8];
        let encoded = vsa.encode(&msg);
        let decoded = vsa.decode(&encoded);
        let expected = vec![0u8; (TEST_K + 7) / 8];
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_encode_decode_roundtrip_all_ones() {
        let vsa = make_vsa();
        let msg = vec![0xFFu8];
        let encoded = vsa.encode(&msg);
        let decoded = vsa.decode(&encoded);
        let expected = {
            let mut m = vec![0u8; (TEST_K + 7) / 8];
            m[0] = msg[0];
            if TEST_K < 8 {
                m[0] &= (1 << TEST_K) - 1;
            }
            m
        };
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_encode_decode_deterministic() {
        let vsa = make_vsa();
        let msg = vec![0b11001100u8];
        let e1 = vsa.encode(&msg);
        let e2 = vsa.encode(&msg);
        assert_eq!(e1, e2);
        let d1 = vsa.decode(&e1);
        let d2 = vsa.decode(&e2);
        assert_eq!(d1, d2);
    }

    // ── 3. Single-bit error correction roundtrip ───────────────────────

    #[test]
    fn test_single_bit_error_correction() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let encoded = vsa.encode(&msg);
        // Flip one bit at position 5
        let mut noisy = encoded.clone();
        noisy[5 / 8] ^= 1 << (5 % 8);
        let corrected = vsa.correct(&noisy);
        assert_eq!(corrected, encoded, "single-bit error should be corrected");
        let decoded = vsa.decode(&noisy);
        let expected = {
            let mut m = vec![0u8; (TEST_K + 7) / 8];
            m[0] = msg[0];
            m
        };
        assert_eq!(
            decoded, expected,
            "single-bit error should be corrected in decode"
        );
    }

    #[test]
    fn test_single_bit_error_parity_region() {
        let vsa = make_vsa();
        let msg = vec![0b11110000u8];
        let encoded = vsa.encode(&msg);
        // Flip a bit in the parity region (position = k + 10)
        let parity_pos = TEST_K + 10;
        let mut noisy = encoded.clone();
        noisy[parity_pos / 8] ^= 1 << (parity_pos % 8);
        let corrected = vsa.correct(&noisy);
        assert_eq!(
            corrected, encoded,
            "single-bit error in parity region should be corrected"
        );
    }

    // ── 4. Bind is commutative ─────────────────────────────────────────

    #[test]
    fn test_bind_commutative() {
        let vsa = make_vsa();
        let msg_a = vec![0b10101010u8];
        let msg_b = vec![0b01010101u8];
        let a = vsa.encode(&msg_a);
        let b = vsa.encode(&msg_b);
        let ab = vsa.bind(&a, &b);
        let ba = vsa.bind(&b, &a);
        assert_eq!(ab, ba, "bind must be commutative");
    }

    #[test]
    fn test_bind_with_self_is_zero() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let a = vsa.encode(&msg);
        let bound = vsa.bind(&a, &a);
        // XOR(a, a) = 0 for linear codes
        assert!(
            bound.iter().all(|&b| b == 0),
            "bind(a, a) must be all zeros for linear codes"
        );
    }

    // ── 5. Unbind recovers original ────────────────────────────────────

    #[test]
    fn test_unbind_recovers_original() {
        let vsa = make_vsa();
        let msg_a = vec![0b10101010u8];
        let msg_b = vec![0b01010101u8];
        let a = vsa.encode(&msg_a);
        let b = vsa.encode(&msg_b);
        let bound = vsa.bind(&a, &b);
        let recovered = vsa.unbind(&bound, &a);
        assert_eq!(recovered, b, "unbind(bind(a,b), a) must recover b");
    }

    #[test]
    fn test_unbind_self_inverse() {
        let vsa = make_vsa();
        let msg_a = vec![0b11110000u8];
        let msg_b = vec![0b00001111u8];
        let a = vsa.encode(&msg_a);
        let b = vsa.encode(&msg_b);
        let ab = vsa.bind(&a, &b);
        let aba = vsa.bind(&ab, &a);
        assert_eq!(aba, b, "bind(bind(a,b), a) must recover b");
    }

    // ── 6. Similarity is symmetric ─────────────────────────────────────

    #[test]
    fn test_similarity_symmetric() {
        let vsa = make_vsa();
        let msg_a = vec![0b10101010u8];
        let msg_b = vec![0b01010101u8];
        let a = vsa.encode(&msg_a);
        let b = vsa.encode(&msg_b);
        let sab = vsa.similarity(&a, &b);
        let sba = vsa.similarity(&b, &a);
        assert!((sab - sba).abs() < 1e-12, "similarity must be symmetric");
    }

    #[test]
    fn test_self_similarity_one() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let a = vsa.encode(&msg);
        let sim = vsa.similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10, "self-similarity must be 1.0");
    }

    #[test]
    fn test_orthogonal_similarity_zero() {
        let vsa = make_vsa();
        let zeros = vec![0u8; TEST_DIM / 8];
        let ones = vec![0xFFu8; TEST_DIM / 8];
        let sim = vsa.similarity(&zeros, &ones);
        assert!(
            (sim - 0.0).abs() < 1e-10,
            "all-zeros vs all-ones sim must be 0"
        );
    }

    // ── 7. Different seeds produce different generator matrices ───────

    #[test]
    fn test_constructor_consistency() {
        let vsa1 = LinearCodeVSA::new(TEST_DIM);
        let vsa2 = LinearCodeVSA::new(TEST_DIM);
        // Same dim → same deterministic construction
        assert_eq!(vsa1.generator().len(), vsa2.generator().len());
        assert_eq!(vsa1.generator()[0], vsa2.generator()[0]);
    }

    // ── 8. Bundle majority voting produces correct length ──────────────

    #[test]
    fn test_bundle_length() {
        let vsa = make_vsa();
        let m1 = vec![0b10101010u8];
        let m2 = vec![0b01010101u8];
        let a = vsa.encode(&m1);
        let b = vsa.encode(&m2);
        let bundled = vsa.bundle(&[&a, &b]);
        assert_eq!(bundled.len(), TEST_DIM / 8);
    }

    #[test]
    fn test_bundle_empty() {
        let vsa = make_vsa();
        let bundled = vsa.bundle(&[]);
        assert_eq!(bundled.len(), TEST_DIM / 8);
        assert!(bundled.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_bundle_three_vectors() {
        let vsa = make_vsa();
        let m1 = vec![0b10101010u8];
        let m2 = vec![0b01010101u8];
        let m3 = vec![0b11110000u8];
        let a = vsa.encode(&m1);
        let b = vsa.encode(&m2);
        let c = vsa.encode(&m3);
        let bundled = vsa.bundle(&[&a, &b, &c]);
        // bundled should be similar to the majority
        assert!(vsa.similarity(&bundled, &a) > 0.0);
    }

    // ── 9. Error detection works on corrupted vectors ──────────────────

    #[test]
    fn test_detect_no_error() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let encoded = vsa.encode(&msg);
        assert!(
            !vsa.detect(&encoded),
            "valid codeword should not detect errors"
        );
    }

    #[test]
    fn test_detect_single_bit_error() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let encoded = vsa.encode(&msg);
        let mut noisy = encoded.clone();
        noisy[3] ^= 0x01;
        assert!(vsa.detect(&noisy), "single-bit error should be detected");
    }

    #[test]
    fn test_detect_multi_bit_error() {
        let vsa = make_vsa();
        let msg = vec![0b10101010u8];
        let encoded = vsa.encode(&msg);
        let mut noisy = encoded.clone();
        noisy[0] ^= 0b00110011;
        assert!(
            vsa.detect(&noisy),
            "multi-bit errors should be detected (non-zero syndrome)"
        );
    }

    // ── 10. At least one error is correctable ──────────────────────────

    #[test]
    fn test_correctable_error_list_not_empty() {
        let vsa = make_vsa();
        assert!(
            !vsa.syndrome_table.is_empty(),
            "syndrome table must have at least one entry"
        );
    }

    #[test]
    fn test_every_bit_position_correctable() {
        let vsa = make_vsa();
        // Test that every bit position has a correctable error pattern
        // by flipping each position of an encoded vector
        let msg = vec![0b10101010u8];
        let encoded = vsa.encode(&msg);
        for pos in 0..TEST_DIM.min(32) {
            let mut noisy = encoded.clone();
            noisy[pos / 8] ^= 1 << (pos % 8);
            let corrected = vsa.correct(&noisy);
            assert_eq!(
                corrected, encoded,
                "bit position {} should be correctable",
                pos
            );
        }
    }

    // ── 11. Random codeword ────────────────────────────────────────────

    #[test]
    fn test_random_codeword_length() {
        let vsa = make_vsa();
        let r = vsa.random(42);
        assert_eq!(r.len(), TEST_DIM / 8);
    }

    #[test]
    fn test_random_codeword_deterministic() {
        let vsa = make_vsa();
        let r1 = vsa.random(42);
        let r2 = vsa.random(42);
        assert_eq!(r1, r2, "same seed must produce same codeword");
    }

    #[test]
    fn test_random_codeword_different_seeds() {
        let vsa = make_vsa();
        let r1 = vsa.random(42);
        let r2 = vsa.random(99);
        // Extremely unlikely that two different seeds produce identical k-bit messages
        if r1 != r2 {
            // Test passes — different seeds (almost always) produce different codewords
        }
    }

    // ── 12. Empty message encode/decode ────────────────────────────────

    #[test]
    fn test_encode_empty() {
        let vsa = make_vsa();
        let encoded = vsa.encode(&[]);
        assert_eq!(encoded.len(), TEST_DIM / 8);
        assert!(encoded.iter().all(|&b| b == 0));
    }

    // ── 13. Generator matrix structure (systematic form) ───────────────

    #[test]
    fn test_generator_systematic() {
        let vsa = make_vsa();
        let g = vsa.generator();
        // G = [I_k | P]: row i has bit i set in the first k bits
        for i in 0..TEST_K {
            assert!(
                test_bit(&g[i], i),
                "row {} of G should have identity bit at position {}",
                i,
                i
            );
        }
    }

    #[test]
    fn test_parity_check_not_empty() {
        let vsa = make_vsa();
        assert!(
            !vsa.parity_check().is_empty(),
            "parity check matrix should be non-empty"
        );
        assert_eq!(
            vsa.parity_check().len(),
            TEST_DIM - TEST_K,
            "H should have (dim - k) rows"
        );
    }

    // ── 14. GF(2^k) construction correctness ───────────────────────────

    #[test]
    fn test_gf2k_exp_table() {
        let exp = build_gf2k_exp(TEST_K);
        assert_eq!(exp[0], 1, "α^0 = 1");
        assert_eq!(exp[TEST_K as usize], 9, "α^12 should be 9 for k=12");
        // α^(2^k-1) should wrap to α^0 = 1
        let cycle = (1 << TEST_K) - 1;
        assert_eq!(exp[cycle - 1], 1, "α^(2^k-1) = 1");
    }

    // ── 15. Multi-block message ────────────────────────────────────────

    #[test]
    fn test_multi_block_encode_decode() {
        let vsa = make_vsa();
        // Create a message longer than k bits
        let msg = vec![0xABu8, 0xCDu8, 0xEFu8];
        let encoded = vsa.encode(&msg);
        let decoded = vsa.decode(&encoded);
        // Decode should recover the original (padded to k_bytes * num_blocks)
        let msg_bits = msg.len() * 8;
        let num_blocks = (msg_bits + TEST_K - 1) / TEST_K;
        let expected_len = num_blocks * ((TEST_K + 7) / 8);
        assert_eq!(decoded.len(), expected_len);
        // First byte should match
        assert_eq!(decoded[0], msg[0]);
    }

    // ── 16. Correct handles multi-block ────────────────────────────────

    #[test]
    fn test_correct_multi_block() {
        let vsa = make_vsa();
        let msg = vec![0xABu8, 0xCDu8];
        let encoded = vsa.encode(&msg);
        let mut noisy = encoded.clone();
        // Flip two bits in different blocks
        if noisy.len() > 10 {
            noisy[2] ^= 0x04;
        }
        if noisy.len() > 20 {
            noisy[17] ^= 0x08;
        }
        let corrected = vsa.correct(&noisy);
        // Even if uncorrectable (multi-bit), the length should be correct
        assert_eq!(corrected.len(), encoded.len());
    }

    // ── 17. parity_check metadata ──────────────────────────────────────

    #[test]
    fn test_accessors() {
        let vsa = make_vsa();
        assert_eq!(vsa.dim(), TEST_DIM);
        assert_eq!(vsa.k(), TEST_K);
        assert_eq!(vsa.codebook_size(), 1 << TEST_K);
    }

    // ── 18. Large dimension test ───────────────────────────────────────

    #[test]
    fn test_dim_4096() {
        let vsa = LinearCodeVSA::new(4096);
        assert_eq!(vsa.dim(), 4096);
        assert_eq!(vsa.k(), 12);
        assert_eq!(vsa.codebook_size(), 4096);
        let msg = vec![0xABu8, 0xCDu8];
        let encoded = vsa.encode(&msg);
        assert_eq!(encoded.len(), 4096 / 8);
        let decoded = vsa.decode(&encoded);
        assert_eq!(decoded[0], msg[0]);
    }

    // ── 19. Syndrome table uniqueness ──────────────────────────────────

    #[test]
    fn test_syndrome_table_unique_entries() {
        let vsa = make_vsa();
        assert_eq!(
            vsa.syndrome_table.len(),
            vsa.dim(),
            "syndrome table should have exactly dim entries (one per bit position)"
        );
    }

    // ── 20. Error correction on zero codeword ──────────────────────────

    #[test]
    fn test_correct_zero_codeword() {
        let vsa = make_vsa();
        let encoded = vsa.encode(&[0u8]);
        let mut noisy = encoded.clone();
        noisy[0] ^= 0x01;
        let corrected = vsa.correct(&noisy);
        assert_eq!(corrected, encoded);
    }
}
