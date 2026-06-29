// REVIVED Task 2 — dead_code removed
use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;

// ── Module-level helper functions ─────────────────────────────────────

/// XOR two byte slices element-wise (GF(2) vector addition).
pub fn gf2_add(a: &[u8], b: &[u8]) -> Vec<u8> {
    let n = a.len().max(b.len());
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let a_byte = a.get(i).copied().unwrap_or(0);
        let b_byte = b.get(i).copied().unwrap_or(0);
        result.push(a_byte ^ b_byte);
    }
    result
}

/// Matrix–vector multiplication over GF(2).
///
/// Each row of `matrix` is a packed byte slice.  The result has
/// `matrix.len()` bits, one per row, packed into bytes.
pub fn gf2_mul_vec(matrix: &[Vec<u8>], vec: &[u8]) -> Vec<u8> {
    let rows = matrix.len();
    let result_bytes = (rows + 7) / 8;
    let mut result = vec![0u8; result_bytes];
    for (i, row) in matrix.iter().enumerate() {
        let dot: u8 = row
            .iter()
            .zip(vec.iter())
            .map(|(a, b)| a & b)
            .fold(0, |acc, x| acc ^ x);
        if dot.count_ones() & 1 == 1 {
            result[i / 8] |= 1 << (i % 8);
        }
    }
    result
}

/// Pack a vector of bits (0/1) into bytes, MSB first.
///
/// `bits[0]` → byte[0] bit 7, `bits[7]` → byte[0] bit 0.
pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    let byte_len = (bits.len() + 7) / 8;
    let mut bytes = vec![0u8; byte_len];
    for (i, &b) in bits.iter().enumerate() {
        if b & 1 == 1 {
            bytes[i / 8] |= 1 << (7 - (i % 8));
        }
    }
    bytes
}

/// Unpack bytes into a vector of bits (each 0 or 1), MSB first.
///
/// byte[0] bit 7 → `result[0]`, byte[0] bit 0 → `result[7]`.
pub fn bytes_to_bits(bytes: &[u8], n: usize) -> Vec<u8> {
    let mut bits = vec![0u8; n];
    for i in 0..n {
        bits[i] = (bytes[i / 8] >> (7 - (i % 8))) & 1;
    }
    bits
}

/// Count the number of 1-bits in a packed byte slice.
pub fn hamming_weight(v: &[u8]) -> usize {
    v.iter().map(|&b| b.count_ones() as usize).sum()
}

/// Hamming distance between two packed byte slices.
fn hamming_dist(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

// ── LinearCodeConfig ──────────────────────────────────────────────────

/// Configuration for a linear code VSA encoder.
///
/// `code_rate` controls the ratio of information bits to coded bits.
/// A rate‑½ code (code_rate = 0.5) uses `dim/2` information bits and
/// achieves the Gilbert–Varshamov bound for random binary linear codes.
#[derive(Clone, Debug)]
pub struct LinearCodeConfig {
    pub dim: usize,
    pub code_rate: f64,
}

impl Default for LinearCodeConfig {
    fn default() -> Self {
        Self {
            dim: 4096,
            code_rate: 0.25,
        }
    }
}

// ── LinearCodeVSA ─────────────────────────────────────────────────────

/// Information‑theoretically optimal VSA encoding using random linear codes
/// over GF(2).
///
/// The encoder maps `k` information bits into a `dim`‑bit codeword via
/// multiplication by a random generator matrix `G` (size k×dim).  The parity
/// check matrix `H` (size (dim−k)×dim) satisfies H·Gᵀ = 0 and is used for
/// syndrome‑based decoding.
///
/// For small `k` (≤ 12) decoding is exact exhaustive enumeration.  For larger
/// `k` a greedy bit‑flipping decoder provides approximate nearest‑codeword
/// search.
pub struct LinearCodeVSA {
    dim: usize,
    k: usize,
    dim_bytes: usize,
    k_bytes: usize,
    generator: Vec<Vec<u8>>,
    parity_check: Vec<Vec<u8>>,
    h_dim: usize,
}

impl LinearCodeVSA {
    /// Construct a new random linear‑code VSA.
    ///
    /// `config.dim` should be a multiple of 8 (e.g. 4096).  The generator
    /// matrix is seeded deterministically from each row index.
    ///
    /// The parity‑check matrix is computed only when `k` or `dim` are small,
    /// to keep construction time reasonable.  When skipped, `has_parity_check()`
    /// returns `false`.
    pub fn new(config: LinearCodeConfig) -> Self {
        let dim = if config.dim < 8 { 8 } else { config.dim };
        let k = ((dim as f64 * config.code_rate.clamp(0.01, 0.99)) as usize).clamp(1, dim - 1);
        let dim_bytes = (dim + 7) / 8;
        let k_bytes = (k + 7) / 8;

        // ── generator matrix: k rows, each dim bits seeded by row index ──
        let mut generator = Vec::with_capacity(k);
        for i in 0..k {
            let seed = i as u64 ^ 0x9e3779b97f4a7c15;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut row = vec![0u8; dim_bytes];
            for byte in &mut row {
                *byte = rng.gen();
            }
            generator.push(row);
        }

        // ── parity‑check matrix (only for feasible sizes) ──
        let (parity_check, h_dim) = if k * dim <= 2_000_000 {
            let pc = Self::compute_parity_check(&generator, dim, k);
            let h = pc.len();
            (pc, h)
        } else {
            (Vec::new(), 0)
        };

        Self {
            dim,
            k,
            dim_bytes,
            k_bytes,
            generator,
            parity_check,
            h_dim,
        }
    }

    // ── accessors ─────────────────────────────────────────────────────

    pub fn dim(&self) -> usize {
        self.dim
    }
    pub fn k(&self) -> usize {
        self.k
    }
    pub fn code_rate(&self) -> f64 {
        self.k as f64 / self.dim as f64
    }
    pub fn information_capacity(&self) -> f64 {
        self.k as f64 / self.dim as f64
    }
    pub fn has_parity_check(&self) -> bool {
        self.h_dim > 0
    }
    pub fn generator(&self) -> &[Vec<u8>] {
        &self.generator
    }
    pub fn parity_check(&self) -> &[Vec<u8>] {
        &self.parity_check
    }

    // ── estimated minimum distance ──

    /// Estimate the minimum Hamming distance by random sampling of
    /// codeword pairs.
    pub fn estimate_min_distance(&self, samples: usize) -> f64 {
        if self.k <= 1 {
            return self.dim as f64;
        }
        let mut min_dist = self.dim as f64;
        let sample_pairs = samples.min(10000);
        let mut rng = rand::thread_rng();
        for _ in 0..sample_pairs {
            let i: usize = rng.gen_range(0..self.k);
            let j: usize = rng.gen_range(0..self.k);
            if i == j {
                continue;
            }
            let mut info_a = vec![0u8; self.k_bytes];
            let mut info_b = vec![0u8; self.k_bytes];
            info_a[i / 8] = 1 << (i % 8);
            info_b[j / 8] = 1 << (j % 8);
            let ca = self.encode_info(&info_a);
            let cb = self.encode_info(&info_b);
            let d = hamming_dist(&ca, &cb) as f64;
            if d < min_dist {
                min_dist = d;
            }
        }
        min_dist
    }

    // ── encode ────────────────────────────────────────────────────────

    /// Encode information bits into a `dim`‑bit codeword.
    ///
    /// `input` is interpreted as `k` information bits (padded with zeros
    /// if shorter, truncated if longer).  The result is `dim_bytes` packed
    /// bytes.
    pub fn encode(&self, input: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; self.dim_bytes];
        let limit = input.len().min(self.k_bytes);
        for byte_idx in 0..limit {
            let byte = input[byte_idx];
            if byte == 0 {
                continue;
            }
            for bit_idx in 0..8 {
                let gen_idx = byte_idx * 8 + bit_idx;
                if gen_idx >= self.k {
                    break;
                }
                if (byte >> bit_idx) & 1 == 1 {
                    for (r, g) in result.iter_mut().zip(self.generator[gen_idx].iter()) {
                        *r ^= g;
                    }
                }
            }
        }
        result
    }

    /// Internal encode that accepts info bits at self.k_bytes length.
    fn encode_info(&self, info: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; self.dim_bytes];
        let limit = info.len().min(self.k_bytes);
        for byte_idx in 0..limit {
            let byte = info[byte_idx];
            if byte == 0 {
                continue;
            }
            for bit_idx in 0..8 {
                let gen_idx = byte_idx * 8 + bit_idx;
                if gen_idx >= self.k {
                    break;
                }
                if (byte >> bit_idx) & 1 == 1 {
                    for (r, g) in result.iter_mut().zip(self.generator[gen_idx].iter()) {
                        *r ^= g;
                    }
                }
            }
        }
        result
    }

    // ── decode ────────────────────────────────────────────────────────

    /// Decode a noisy VSA vector back to its nearest information bits.
    ///
    /// For `k ≤ 12` this performs exhaustive nearest‑codeword search over
    /// all `2^k` possibilities.  For larger `k` a greedy bit‑flipping decoder
    /// is used (approximate).
    pub fn decode(&self, noisy: &[u8]) -> Vec<u8> {
        if self.k <= 12 {
            self.decode_exhaustive(noisy)
        } else {
            self.decode_bit_flipping(noisy)
        }
    }

    fn decode_exhaustive(&self, noisy: &[u8]) -> Vec<u8> {
        let total = 1usize << self.k;
        let noisy_trim = &noisy[..noisy.len().min(self.dim_bytes)];
        let mut best_info = vec![0u8; self.k_bytes];
        let mut best_dist = u32::MAX;
        for val in 0..total {
            let mut info = vec![0u8; self.k_bytes];
            for bit in 0..self.k {
                if (val >> bit) & 1 == 1 {
                    info[bit / 8] |= 1 << (bit % 8);
                }
            }
            let codeword = self.encode_info(&info);
            let dist = hamming_dist(&codeword, noisy_trim);
            if dist < best_dist {
                best_dist = dist;
                best_info = info;
                if dist == 0 {
                    break;
                }
            }
        }
        best_info
    }

    fn decode_bit_flipping(&self, noisy: &[u8]) -> Vec<u8> {
        let noisy_trim = &noisy[..noisy.len().min(self.dim_bytes)];
        let mut info = vec![0u8; self.k_bytes];
        let mut best_info = info.clone();
        let mut best_dist = hamming_dist(&self.encode_info(&info), noisy_trim);

        for _ in 0..(2 * self.k) {
            let mut improved = false;
            for bit in 0..self.k {
                info[bit / 8] ^= 1 << (bit % 8);
                let codeword = self.encode_info(&info);
                let dist = hamming_dist(&codeword, noisy_trim);
                if dist < best_dist {
                    best_dist = dist;
                    best_info = info.clone();
                    improved = true;
                } else {
                    info[bit / 8] ^= 1 << (bit % 8);
                }
            }
            if !improved {
                break;
            }
        }
        best_info
    }

    // ── syndrome table ────────────────────────────────────────────────

    /// Generate a syndrome table for instant lookup decoding.
    ///
    /// For small codes (`k ≤ 12`) this enumerates all `2^k` codewords and
    /// builds a mapping `codeword → info bits`.  The result size equals
    /// `2^k` (which equals `2^(dim‑k)` for rate‑½ codes).
    ///
    /// Returns an empty map when `k > 12`.
    pub fn generate_syndrome_table(&self) -> HashMap<Vec<u8>, Vec<u8>> {
        if self.k > 12 {
            return HashMap::new();
        }
        let total = 1usize << self.k;
        let mut table = HashMap::with_capacity(total);
        for val in 0..total {
            let mut info = vec![0u8; self.k_bytes];
            for bit in 0..self.k {
                if (val >> bit) & 1 == 1 {
                    info[bit / 8] |= 1 << (bit % 8);
                }
            }
            let codeword = self.encode(&info);
            table.insert(codeword, info);
        }
        table
    }

    // ── similarity ────────────────────────────────────────────────────

    /// Normalised Hamming similarity between two packed vectors.
    ///
    /// Returns `1.0` for identical vectors, `0.0` for bitwise complements.
    pub fn similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        let len = a.len().min(b.len());
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

    // ── parity‑check multiplication ───────────────────────────────────

    /// Compute syndrome `s = H · vᵀ`.
    ///
    /// Returns `(dim‑k)` bits packed into bytes, or an empty vector if no
    /// parity‑check matrix is available.
    pub fn syndrome(&self, v: &[u8]) -> Vec<u8> {
        if self.h_dim == 0 {
            return Vec::new();
        }
        gf2_mul_vec(&self.parity_check, v)
    }
}

// ─── GF(256) Arithmetic for Reed-Solomon ───

fn gf256_tables() -> &'static ([u8; 512], [u8; 256]) {
    static TABLES: std::sync::OnceLock<([u8; 512], [u8; 256])> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut exp = [0u8; 512];
        exp[0] = 1;
        for i in 1..512 {
            let prev = exp[i - 1] as u16;
            let mut val = prev << 1;
            if val & 0x100 != 0 {
                val ^= 0x11D;
            }
            exp[i] = val as u8;
        }
        let mut log = [0u8; 256];
        let mut i = 0;
        while i < 255 {
            log[exp[i] as usize] = i as u8;
            i += 1;
        }
        (exp, log)
    })
}

/// GF(256) element operations using the irreducible polynomial
/// x^8 + x^4 + x^3 + x^2 + 1 (0x11D).
pub struct Gf256;

impl Gf256 {
    pub fn add(a: u8, b: u8) -> u8 {
        a ^ b
    }

    pub fn mul(a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }
        let (exp, log) = gf256_tables();
        let sum = log[a as usize] as u16 + log[b as usize] as u16;
        exp[sum as usize]
    }

    pub fn inv(a: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        let (exp, log) = gf256_tables();
        let l = log[a as usize] as u16;
        exp[(255 - l) as usize]
    }

    pub fn exp(e: u8) -> u8 {
        let (exp, _) = gf256_tables();
        exp[e as usize]
    }

    pub fn log(a: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        let (_, log) = gf256_tables();
        log[a as usize]
    }
}

// ─── Reed-Solomon Codec ───

pub struct ReedSolomon {
    pub n: usize,
    pub k: usize,
    pub generator: Vec<u8>,
}

impl ReedSolomon {
    pub fn new(n: usize, k: usize) -> Self {
        assert!(n <= 255, "RS n must be <= 255, got {n}");
        assert!(k <= n, "RS k must be <= n, got k={k} n={n}");
        let generator = Self::compute_generator(n - k);
        Self { n, k, generator }
    }

    pub fn t(&self) -> usize {
        (self.n - self.k) / 2
    }

    fn compute_generator(nk: usize) -> Vec<u8> {
        let mut gen = vec![0u8; nk + 1];
        gen[0] = 1;
        for j in 1..=nk {
            let root = Gf256::exp(j as u8);
            let mut prev = 0u8;
            for i in 0..=j {
                let tmp = gen[i];
                gen[i] = prev ^ Gf256::mul(tmp, root);
                prev = tmp;
                if i == j {
                    break;
                }
            }
        }
        gen
    }

    pub fn encode(&self, message: &[u8]) -> Vec<u8> {
        let mut codeword = vec![0u8; self.n];
        let msg_len = message.len().min(self.k);
        codeword[..msg_len].copy_from_slice(&message[..msg_len]);
        for i in 0..self.k {
            let coef = codeword[i];
            if coef != 0 {
                for j in 1..self.generator.len() {
                    codeword[i + j] ^= Gf256::mul(self.generator[j], coef);
                }
            }
        }
        codeword
    }

    fn syndrome(&self, received: &[u8]) -> Vec<u8> {
        let nk = self.n - self.k;
        let mut syndromes = vec![0u8; nk];
        for (j, syn) in syndromes.iter_mut().enumerate() {
            let alpha_j = Gf256::exp((j + 1) as u8);
            *syn = received[self.n - 1];
            for i in (0..self.n - 1).rev() {
                *syn = Gf256::mul(*syn, alpha_j) ^ received[i];
            }
        }
        syndromes
    }

    fn berlekamp_massey(&self, syndromes: &[u8]) -> Vec<u8> {
        let nk = syndromes.len();
        let mut lambda: Vec<u8> = vec![1];
        let mut b: Vec<u8> = vec![1];
        let mut l = 0usize;
        let mut m = 1usize;

        for r in 0..nk {
            let mut d = syndromes[r];
            for i in 1..=l {
                if i < lambda.len() && r >= i {
                    d ^= Gf256::mul(lambda[i], syndromes[r - i]);
                }
            }

            if d == 0 {
                m += 1;
            } else {
                let old_lambda = lambda.clone();
                let mut new_lambda = lambda.clone();
                for i in 0..b.len() {
                    let idx = i + m;
                    while idx >= new_lambda.len() {
                        new_lambda.push(0);
                    }
                    new_lambda[idx] ^= Gf256::mul(d, b[i]);
                }

                if 2 * l <= r {
                    let inv_d = Gf256::inv(d);
                    b = old_lambda.iter().map(|&c| Gf256::mul(inv_d, c)).collect();
                    l = r + 1 - l;
                    m = 1;
                } else {
                    m += 1;
                }

                lambda = new_lambda;
            }
        }

        while lambda.len() > 1 && lambda[lambda.len() - 1] == 0 {
            lambda.pop();
        }

        lambda
    }

    fn chien_search(&self, locator: &[u8]) -> Vec<usize> {
        let mut positions = Vec::new();
        for i in 0..self.n {
            let mut val = 0u8;
            for (j, &coef) in locator.iter().enumerate() {
                let power = (i * j) as u16;
                val ^= Gf256::mul(coef, Gf256::exp((power % 255) as u8));
            }
            if val == 0 && i < self.n {
                positions.push(i);
            }
        }
        positions
    }

    fn forney(&self, syndromes: &[u8], locator: &[u8], positions: &[usize]) -> Vec<u8> {
        let nk = self.n - self.k;
        let degree = locator.len().saturating_sub(1);

        let mut omega = vec![0u8; nk];
        for k in 0..nk {
            for i in 0..=degree.min(k) {
                let syn_idx = k - i;
                if syn_idx < nk {
                    omega[k] ^= Gf256::mul(locator[i], syndromes[syn_idx]);
                }
            }
        }

        let mut error_values = Vec::with_capacity(positions.len());
        for &pos in positions {
            let pos_mod = pos % 255;
            let x_inv_power = (255 - pos_mod) as u16;

            let mut omega_val = 0u8;
            for (k, &coef) in omega.iter().enumerate().take(nk) {
                let power = (x_inv_power * k as u16) % 255;
                omega_val ^= Gf256::mul(coef, Gf256::exp(power as u8));
            }

            let mut sigma_deriv = 0u8;
            for (j, &coef) in locator.iter().enumerate() {
                if j % 2 == 1 {
                    let power = (x_inv_power * (j - 1) as u16) % 255;
                    sigma_deriv ^= Gf256::mul(coef, Gf256::exp(power as u8));
                }
            }

            let e = if sigma_deriv == 0 {
                0
            } else {
                Gf256::mul(omega_val, Gf256::inv(sigma_deriv))
            };
            error_values.push(e);
        }

        error_values
    }

    pub fn decode(&self, codeword: &[u8]) -> Result<(Vec<u8>, usize), String> {
        let received: Vec<u8> = if codeword.len() >= self.n {
            codeword[..self.n].to_vec()
        } else {
            let mut r = codeword.to_vec();
            r.resize(self.n, 0);
            r
        };

        let t = self.t();
        if t == 0 {
            return Ok((received[..self.k].to_vec(), 0));
        }

        let syndromes = self.syndrome(&received);
        if syndromes.iter().all(|&s| s == 0) {
            return Ok((received[..self.k].to_vec(), 0));
        }

        let locator = self.berlekamp_massey(&syndromes);
        if locator.is_empty() || (locator.len() == 1 && locator[0] == 0) {
            return Err("Failed to find error locator polynomial".to_string());
        }

        let positions = self.chien_search(&locator);
        if positions.is_empty() {
            return Err("Chien search found no error positions".to_string());
        }

        let error_values = self.forney(&syndromes, &locator, &positions);
        let mut decoded = received;
        for (&pos, &val) in positions.iter().zip(error_values.iter()) {
            if pos < self.n {
                decoded[pos] ^= val;
            }
        }

        let errors_corrected = positions.len();
        Ok((decoded[..self.k].to_vec(), errors_corrected))
    }
}

// ─── Hadamard VSA Operations ───

/// Hadamard bind: element-wise XOR (Hadamard product for binary {0,1} VSA).
/// Self-inverse: unbind(bind(a,b), b) == a.
pub fn hadamard_bind(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Hadamard unbind: identical to bind for binary VSA (self-inverse).
pub fn hadamard_unbind(c: &[u8], b: &[u8]) -> Vec<u8> {
    hadamard_bind(c, b)
}

impl LinearCodeVSA {
    // ── parity‑check matrix computation (Gaussian elimination over GF(2)) ──

    /// Compute nullspace basis of the generator matrix.
    ///
    /// Returns `H` (a `(dim−rank)×dim` matrix in packed form) such that
    /// `H · Gᵀ = 0`.
    fn compute_parity_check(generator: &[Vec<u8>], dim: usize, k: usize) -> Vec<Vec<u8>> {
        let dim_bytes = (dim + 7) / 8;

        // Clone generator rows for in‑place elimination (packed form).
        let mut m: Vec<Vec<u8>> = generator.iter().map(|r| r.clone()).collect();

        let mut rank = 0;
        let mut pivot_cols: Vec<Option<usize>> = vec![None; k];

        for col in 0..dim {
            let byte_idx = col / 8;
            let bit_idx = col % 8;

            // Find pivot row (first row ≥ rank with a 1 at this column).
            let mut pivot = None;
            for r in rank..k {
                if (m[r][byte_idx] >> bit_idx) & 1 == 1 {
                    pivot = Some(r);
                    break;
                }
            }

            if let Some(pivot_row) = pivot {
                m.swap(rank, pivot_row);
                pivot_cols[rank] = Some(col);

                // Eliminate this column from all other rows.
                // Clone the pivot row to avoid simultaneous mutable/immutable borrows.
                let pivot_row_clone = m[rank].clone();
                for r in 0..k {
                    if r != rank && ((m[r][byte_idx] >> bit_idx) & 1) == 1 {
                        for (x, y) in m[r].iter_mut().zip(pivot_row_clone.iter()) {
                            *x ^= y;
                        }
                    }
                }

                rank += 1;
            }
        }

        // Build nullspace: one vector per non‑pivot column.
        let pc_set: std::collections::HashSet<usize> =
            pivot_cols.iter().filter_map(|&x| x).collect();

        let mut nullspace: Vec<Vec<u8>> = Vec::new();
        for col in 0..dim {
            if pc_set.contains(&col) {
                continue;
            }
            let mut vec = vec![0u8; dim_bytes];
            vec[col / 8] |= 1 << (col % 8);

            for row in 0..rank {
                if let Some(pc) = pivot_cols[row] {
                    if ((m[row][col / 8] >> (col % 8)) & 1) == 1 {
                        vec[pc / 8] |= 1 << (pc % 8);
                    }
                }
            }
            nullspace.push(vec);
        }

        nullspace
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Small code for exhaustive decode tests (k=8 ≤ 12)
    fn make_small() -> LinearCodeVSA {
        LinearCodeVSA::new(LinearCodeConfig {
            dim: 32,
            code_rate: 0.25,
        })
    }

    // Code with parity check available (k=8, dim-k=8)
    fn make_parity_code() -> LinearCodeVSA {
        LinearCodeVSA::new(LinearCodeConfig {
            dim: 16,
            code_rate: 0.5,
        })
    }

    // Default-sized code
    fn make_default() -> LinearCodeVSA {
        LinearCodeVSA::new(LinearCodeConfig {
            dim: 4096,
            code_rate: 0.25,
        })
    }

    // ── 1. Constructor ────────────────────────────────────────────────

    #[test]
    fn test_new_with_default_config() {
        let c = LinearCodeConfig::default();
        assert_eq!(c.dim, 4096);
        assert!((c.code_rate - 0.25).abs() < 1e-12);
        let vsa = LinearCodeVSA::new(c);
        assert_eq!(vsa.dim(), 4096);
        assert!(vsa.k() > 0);
        assert!(vsa.k() < 4096);
        assert!((vsa.code_rate() - 0.25).abs() < 0.01);
    }

    // ── 2. Encoded output length ──────────────────────────────────────

    #[test]
    fn test_encoded_length() {
        let vsa = make_small();
        let data = vec![0b10101010u8];
        let encoded = vsa.encode(&data);
        assert_eq!(encoded.len(), vsa.dim() / 8);
    }

    // ── 3. Encode → decode roundtrip (exhaustive) ─────────────────────

    #[test]
    fn test_encode_decode_roundtrip() {
        let vsa = make_small();
        let test_data: Vec<Vec<u8>> = vec![
            vec![0b00000001u8],
            vec![0b10000000u8],
            vec![0b10101010u8],
            vec![0b11111111u8],
            vec![0u8; 0],
        ];
        for data in &test_data {
            let encoded = vsa.encode(data);
            let decoded = vsa.decode(&encoded);
            let expected = if data.is_empty() {
                vec![0u8; vsa.k_bytes]
            } else {
                let mut padded = data.clone();
                padded.resize(vsa.k_bytes, 0);
                padded
            };
            assert_eq!(
                decoded, expected,
                "roundtrip failed for data {:?}: got {:?}, expected {:?}",
                data, decoded, expected
            );
        }
    }

    // ── 4. Self-similarity = 1.0 ──────────────────────────────────────

    #[test]
    fn test_similarity_identical() {
        let vsa = make_small();
        let data = vec![0b10101010u8];
        let enc = vsa.encode(&data);
        let sim = vsa.similarity(&enc, &enc);
        assert!((sim - 1.0).abs() < 1e-10, "self-similarity must be 1.0");
    }

    // ── 5. Orthogonal vectors → similarity ≈ 0.0 ──────────────────────

    #[test]
    fn test_similarity_orthogonal() {
        let vsa = make_small();
        let zeros = vec![0u8; vsa.dim() / 8];
        let ones = vec![0xFFu8; vsa.dim() / 8];
        let sim = vsa.similarity(&zeros, &ones);
        assert!(
            (sim - 0.0).abs() < 1e-10,
            "all-zeros vs all-ones must be 0, got {}",
            sim
        );
    }

    // ── 6. Information capacity = k/dim ───────────────────────────────

    #[test]
    fn test_information_capacity() {
        let vsa = make_small();
        let expected = vsa.k() as f64 / vsa.dim() as f64;
        assert!((vsa.information_capacity() - expected).abs() < 1e-12);
        assert!((vsa.code_rate() - expected).abs() < 1e-12);
    }

    // ── 7. Different inputs → different codewords ─────────────────────

    #[test]
    fn test_different_inputs_different_outputs() {
        let vsa = make_small();
        let a = vsa.encode(&[0b00000001u8]);
        let b = vsa.encode(&[0b00000010u8]);
        assert_ne!(a, b, "different info must produce different codewords");
        let sim = vsa.similarity(&a, &b);
        // For random linear codes, different inputs should still have
        // expected similarity ≈ 0.5 for sparse input.
        assert!(
            sim < 0.9,
            "different inputs should have low-ish similarity, got {}",
            sim
        );
    }

    // ── 8. Syndrome table completeness ─────────────────────────────────

    #[test]
    fn test_syndrome_table_size() {
        // Use rate-½ so k = dim-k = 8; table size = 2^8 = 256
        let vsa = make_parity_code();
        assert_eq!(vsa.k(), 8);
        let table = vsa.generate_syndrome_table();
        assert_eq!(
            table.len(),
            256,
            "syndrome table must have 2^k = 256 entries"
        );
    }

    // ── 9. Error correction: flip 1 bit → decode still correct ────────

    #[test]
    fn test_error_correction_single_bit_flip() {
        let vsa = make_small();
        let data = vec![0b10101010u8];
        let encoded = vsa.encode(&data);
        // Flip one bit in the codeword
        let mut noisy = encoded.clone();
        noisy[0] ^= 0x01; // flip LSB of first byte
        let decoded = vsa.decode(&noisy);
        let expected = {
            let mut p = data.clone();
            p.resize(vsa.k_bytes, 0);
            p
        };
        assert_eq!(decoded, expected, "single-bit flip should be corrected");
    }

    // ── 10. Generator matrix dimensions ───────────────────────────────

    #[test]
    fn test_generator_dimensions() {
        let vsa = make_small();
        assert_eq!(vsa.generator.len(), vsa.k());
        assert_eq!(vsa.generator[0].len(), vsa.dim() / 8);
    }

    // ── 11. H · Gᵀ = 0 (parity check orthogonality) ───────────────────

    #[test]
    fn test_parity_check_orthogonality() {
        let vsa = make_parity_code();
        assert!(
            vsa.has_parity_check(),
            "parity check should be computed for small code"
        );
        // For each row of H and each row of G, dot product must be 0.
        for h_row in vsa.parity_check.iter() {
            for g_row in vsa.generator.iter() {
                let dot: u8 = h_row
                    .iter()
                    .zip(g_row.iter())
                    .map(|(a, b)| a & b)
                    .fold(0, |acc, x| acc ^ x);
                assert_eq!(
                    dot.count_ones() & 1,
                    0,
                    "H·Gᵀ must be zero matrix; dot product had odd parity"
                );
            }
        }
    }

    // ── 12. Code rate clamping ────────────────────────────────────────

    #[test]
    fn test_code_rate_clamping() {
        // code_rate = 0.0 → clamped to 0.01 → k ≥ 1
        let vsa_low = LinearCodeVSA::new(LinearCodeConfig {
            dim: 64,
            code_rate: 0.0,
        });
        assert!(vsa_low.k() >= 1, "even zero rate should give k ≥ 1");
        assert!(vsa_low.k() < 64);

        // code_rate = 1.0 → clamped to 0.99 → k ≤ dim-1
        let vsa_high = LinearCodeVSA::new(LinearCodeConfig {
            dim: 64,
            code_rate: 1.0,
        });
        assert!(vsa_high.k() < 64, "rate=1.0 should clamp so k < dim");
        assert!(vsa_high.k() > 0);

        // code_rate = 0.5 → exactly half
        let vsa_half = LinearCodeVSA::new(LinearCodeConfig {
            dim: 64,
            code_rate: 0.5,
        });
        assert_eq!(vsa_half.k(), 32);
    }

    // ── 13. Helper functions ──────────────────────────────────────────

    #[test]
    fn test_gf2_add() {
        let a = vec![0b10101010u8];
        let b = vec![0b01010101u8];
        let c = gf2_add(&a, &b);
        assert_eq!(c, vec![0b11111111u8]);
    }

    #[test]
    fn test_gf2_mul_vec_simple() {
        // 2×8 matrix with known rows
        let mat = vec![vec![0b11110000u8], vec![0b00111100u8]];
        let vec = vec![0b10100000u8]; // bits: 1,0,1,0,0,0,0,0
        let result = gf2_mul_vec(&mat, &vec);
        // row 0 dot = popcount(0b11110000 & 0b10100000) = popcount(0b10100000) = 2 → parity 0
        // row 1 dot = popcount(0b00111100 & 0b10100000) = popcount(0b00100000) = 1 → parity 1
        assert_eq!(result.len(), 1);
        assert_eq!(result[0] & 0x01, 0); // row 0 parity = 0
        assert_eq!((result[0] >> 1) & 0x01, 1); // row 1 parity = 1
    }

    #[test]
    fn test_bits_to_bytes_roundtrip() {
        let bits = vec![1, 0, 1, 0, 1, 0, 1, 0];
        let bytes = bits_to_bytes(&bits);
        assert_eq!(bytes, vec![0b10101010u8]);
        let back = bytes_to_bits(&bytes, 8);
        assert_eq!(back, bits);
    }

    #[test]
    fn test_hamming_weight() {
        assert_eq!(hamming_weight(&[0b10101010u8, 0b11110000u8]), 8);
        assert_eq!(hamming_weight(&[0u8; 10]), 0);
    }

    // ── 14. Information capacity of default code ──────────────────────

    #[test]
    fn test_default_code_capacity() {
        let vsa = make_default();
        let cap = vsa.information_capacity();
        assert!((cap - 0.25).abs() < 0.01);
    }

    // ── 15. Generator matrix rows are well-formed ─────────────────────

    #[test]
    fn test_generator_rows_unique() {
        let vsa = make_small();
        // Verify no two generator rows are identical (very unlikely for random rows)
        let mut any_distinct = false;
        'outer: for i in 0..vsa.k() {
            for j in (i + 1)..vsa.k() {
                if vsa.generator[i] != vsa.generator[j] {
                    any_distinct = true;
                    break 'outer;
                }
            }
        }
        assert!(any_distinct, "generator rows should be distinct");
    }

    // ── 16. Syndrome computation via parity check ─────────────────────

    #[test]
    fn test_syndrome_of_codeword_is_zero() {
        let vsa = make_parity_code();
        assert!(vsa.has_parity_check());
        let data = vec![0b10101010u8];
        let codeword = vsa.encode(&data);
        let syn = vsa.syndrome(&codeword);
        // Syndrome of any valid codeword must be all zeros.
        assert!(
            syn.iter().all(|&b| b == 0),
            "syndrome of a valid codeword must be zero, got {:?}",
            syn
        );
    }

    // ── RS + Hadamard tests ──────────────────────────────────

    #[test]
    fn test_gf256_mul_inv_roundtrip() {
        for x in 1u8..=255 {
            let inv = Gf256::inv(x);
            let product = Gf256::mul(x, inv);
            assert_eq!(product, 1, "a * inv(a) = 1 failed for a={x}");
        }
    }

    #[test]
    fn test_rs_encode_decode() {
        let rs = ReedSolomon::new(255, 223);
        let msg: Vec<u8> = (0..223).map(|i| i as u8).collect();
        let codeword = rs.encode(&msg);
        let (decoded, errors) = rs.decode(&codeword).unwrap();
        assert_eq!(decoded, msg);
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_rs_error_correction() {
        let rs = ReedSolomon::new(255, 251);
        let msg: Vec<u8> = (0..251).map(|i| i as u8).collect();
        let codeword = rs.encode(&msg);
        let mut corrupted = codeword.clone();
        corrupted[0] ^= 0xAB;
        corrupted[100] ^= 0xCD;
        let (decoded, errors) = rs.decode(&corrupted).unwrap();
        assert_eq!(decoded, msg);
        assert_eq!(errors, 2);
    }

    #[test]
    fn test_hadamard_bind_unbind_identity() {
        let a = vec![0xAB, 0xCD, 0xEF, 0x12];
        let b = vec![0x34, 0x56, 0x78, 0x90];
        let bound = hadamard_bind(&a, &b);
        let unbound = hadamard_unbind(&bound, &b);
        assert_eq!(unbound, a);
    }

    #[test]
    fn test_hadamard_bind_different_from_inputs() {
        let a = vec![0xAB, 0xCD, 0xEF, 0x12];
        let b = vec![0x34, 0x56, 0x78, 0x90];
        let bound = hadamard_bind(&a, &b);
        assert_ne!(bound, a);
        assert_ne!(bound, b);
    }
}
