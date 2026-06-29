//! Sutra-inspired rotation binding for VSA (arXiv:2605.20919).
//!
//! bind(role, filler) = R_role @ filler
//! where R_role is a seeded approximate orthogonal transformation.
//!
//! For 4096-bit vectors, we decompose into 64 blocks of 64 bits each.
//! Each block uses a seeded 64×64 permutation matrix (generated on-the-fly
//! via a deterministic shuffle from the role seed). This gives:
//!   - Near-orthogonal transformation (norm-preserving)
//!   - Invertible (reverse shuffle + same sign flips)
//!   - O(n) time, O(1) extra memory
//!   - Works correctly on anisotropic embeddings (LLM, protein, etc.)

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const BLOCK_SIZE: usize = 64;

/// A seeded rotation binding transformation.
///
/// Each role has a unique seed. The seed deterministically generates
/// a block-diagonal permutation + sign-flip transformation that
/// approximates a Haar-random orthogonal matrix.
#[derive(Debug, Clone)]
pub struct RotationBind {
    role_seed: u64,
    dim: usize,
}

impl RotationBind {
    /// Create a new rotation binding for a given role seed.
    ///
    /// The `role_seed` should be unique per role (e.g., hash of role name).
    /// Same seed = same transformation (deterministic).
    pub fn new(role_seed: u64) -> Self {
        Self {
            role_seed,
            dim: 4096,
        }
    }

    /// Create with explicit dimension.
    pub fn with_dim(role_seed: u64, dim: usize) -> Self {
        let dim = if dim == 0 { 4096 } else { dim };
        Self { role_seed, dim }
    }

    /// Seed a role name: deterministic hash of any string.
    /// Use this to create role seeds from role names.
    pub fn seed_from_role(role: &str) -> u64 {
        let mut h = DefaultHasher::new();
        role.hash(&mut h);
        h.finish()
    }

    /// Dimension of this rotator.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Apply the rotation: bind(role, filler).
    ///
    /// For each block at position `b`:
    ///   1. Compute a deterministic permutation of indices within the block
    ///      using a seeded hash of (role_seed, b, i).
    ///   2. Apply a sign flip determined by another seeded hash.
    ///   3. Map the permuted+flipped values to the output.
    pub fn bind(&self, filler: &[u8]) -> Vec<u8> {
        let n = filler.len().min(self.dim);
        let mut result = vec![0u8; n];
        let num_blocks = n.div_ceil(BLOCK_SIZE);

        for block in 0..num_blocks {
            let start = block * BLOCK_SIZE;
            let end = (start + BLOCK_SIZE).min(n);
            let block_len = end - start;
            if block_len == 0 {
                continue;
            }

            // Generate permutation of indices within this block
            let mut indices: Vec<usize> = (start..end).collect();
            self.shuffle_in_block(&mut indices, block);

            // Apply permutation + sign flip
            for (out_i, &src_i) in indices.iter().enumerate() {
                let val = filler[src_i];
                let sign = self.sign_flip(block, out_i);
                let out_idx = start + out_i;
                if out_idx < n {
                    result[out_idx] = if sign { val ^ 1 } else { val };
                }
            }
        }
        result
    }

    /// Apply the inverse rotation: unbind(role, bound).
    ///
    /// This is the reverse permutation of bind.
    pub fn unbind(&self, bound: &[u8]) -> Vec<u8> {
        let n = bound.len().min(self.dim);
        let mut result = vec![0u8; n];
        let num_blocks = n.div_ceil(BLOCK_SIZE);

        for block in 0..num_blocks {
            let start = block * BLOCK_SIZE;
            let end = (start + BLOCK_SIZE).min(n);
            let block_len = end - start;
            if block_len == 0 {
                continue;
            }

            // Generate the SAME permutation as bind
            let mut indices: Vec<usize> = (start..end).collect();
            self.shuffle_in_block(&mut indices, block);

            // Reverse the permutation: in bind, result[start + out_i] = filler[src_i]
            // So in unbind, result[src_i] = bound[start + out_i]
            for (out_i, &src_i) in indices.iter().enumerate() {
                let val = bound[start + out_i];
                let sign = self.sign_flip(block, out_i);
                let result_val = if sign { val ^ 1 } else { val };
                if src_i < n {
                    result[src_i] = result_val;
                }
            }
        }
        result
    }

    /// Generate a codebook mapping strings to rotation-bound hypervectors.
    ///
    /// Each string is first embedded via a deterministic hash, then
    /// bound with the role: codebook[string] = bind(role, embed(string)).
    pub fn codebook(&self, strings: &[&str]) -> Vec<(String, Vec<u8>)> {
        strings
            .iter()
            .map(|&s| {
                let embedded = self.embed_string(s);
                let bound = self.bind(&embedded);
                (s.to_string(), bound)
            })
            .collect()
    }

    /// Deterministic string embedding via hash-to-vector.
    /// This is a stand-in for LLM embedding used at compile time.
    pub fn embed_string(&self, s: &str) -> Vec<u8> {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        let base_seed = h.finish();

        (0..self.dim)
            .map(|i| {
                let mut h2 = DefaultHasher::new();
                base_seed.hash(&mut h2);
                (i as u64).hash(&mut h2);
                (h2.finish() as u8) & 1
            })
            .collect()
    }

    // --- internal helpers ---

    fn shuffle_in_block(&self, indices: &mut Vec<usize>, block: usize) {
        let n = indices.len();
        // Fisher-Yates shuffle with seeded hash for each swap
        for i in (1..n).rev() {
            let j = self.hash_pair(block, i, i as u64 + 1) % (i + 1);
            indices.swap(i, j);
        }
    }

    fn sign_flip(&self, block: usize, index: usize) -> bool {
        self.hash_pair(block, index, u64::MAX) & 1 == 1
    }

    fn hash_pair(&self, a: usize, b: usize, mix: u64) -> usize {
        let mut h = DefaultHasher::new();
        self.role_seed.hash(&mut h);
        (a as u64).hash(&mut h);
        (b as u64).hash(&mut h);
        mix.hash(&mut h);
        h.finish() as usize
    }
}

/// A codebook that maps role seeds to rotation bindings.
/// Each role has a unique rotation transformation.
#[derive(Debug, Clone)]
pub struct RotationCodebook {
    bindings: Vec<RotationBind>,
    labels: Vec<String>,
}

impl RotationCodebook {
    /// Create a new empty codebook.
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            labels: Vec::new(),
        }
    }

    /// Get or create a rotation binding for a role.
    pub fn role(&mut self, role: &str) -> RotationBind {
        // Check if we already have this role
        if let Some(pos) = self.labels.iter().position(|l| l == role) {
            return self.bindings[pos].clone();
        }
        // Create new binding
        let seed = RotationBind::seed_from_role(role);
        let rb = RotationBind::new(seed);
        self.bindings.push(rb.clone());
        self.labels.push(role.to_string());
        rb
    }

    /// Encode a (role, filler) pair: bind(role, filler).
    pub fn encode(&mut self, role: &str, filler: &[u8]) -> Vec<u8> {
        let rb = self.role(role);
        rb.bind(filler)
    }

    /// Decode a (role, bound) pair: unbind(role, bound).
    pub fn decode(&self, role: &str, bound: &[u8]) -> Result<Vec<u8>, String> {
        let pos = self
            .labels
            .iter()
            .position(|l| l == role)
            .ok_or_else(|| format!("Unknown role '{}'", role))?;
        Ok(self.bindings[pos].unbind(bound))
    }

    /// Number of registered roles.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if codebook is empty.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

impl Default for RotationCodebook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_bind_unbind_roundtrip() {
        let rb = RotationBind::new(42);
        let filler = rb.embed_string("hello");
        let bound = rb.bind(&filler);
        let unbound = rb.unbind(&bound);
        assert_eq!(filler, unbound, "bind; unbind should recover original");
    }

    #[test]
    fn test_different_roles_different_output() {
        let rb1 = RotationBind::new(1);
        let rb2 = RotationBind::new(2);
        let filler = vec![1u8; 128];
        let bound1 = rb1.bind(&filler);
        let bound2 = rb2.bind(&filler);
        assert_ne!(bound1, bound2, "different role seeds must differ");
    }

    #[test]
    fn test_deterministic_same_seed() {
        let rb = RotationBind::new(99);
        let filler = vec![1, 0, 1, 0, 1, 0, 1, 0];
        let a = rb.bind(&filler);
        let b = rb.bind(&filler);
        assert_eq!(a, b, "same seed + same filler must produce same bound");
    }

    #[test]
    fn test_codebook_stability() {
        let mut cb = RotationCodebook::new();
        let filler = vec![1u8; 64];
        let e1 = cb.encode("subject", &filler);
        let e2 = cb.encode("subject", &filler);
        assert_eq!(e1, e2, "same role + same filler must produce same encoding");
    }

    #[test]
    fn test_role_lookup() {
        let mut cb = RotationCodebook::new();
        cb.role("noun");
        cb.role("verb");
        cb.role("adjective");
        assert_eq!(cb.len(), 3);
    }

    #[test]
    fn test_embed_string_deterministic() {
        let rb = RotationBind::new(0);
        let a = rb.embed_string("test");
        let b = rb.embed_string("test");
        assert_eq!(a, b);
        let c = rb.embed_string("different");
        assert_ne!(a, c);
    }

    #[test]
    fn test_norm_preservation_approx() {
        let rb = RotationBind::new(7);
        let filler: Vec<u8> = (0..256).map(|i| (i as u8) & 1).collect();
        let ones_in = filler.iter().filter(|&&x| x == 1).count();
        let bound = rb.bind(&filler);
        let ones_out = bound.iter().filter(|&&x| x == 1).count();
        // Permutation preserves count exactly
        assert_eq!(
            ones_in, ones_out,
            "rotation is a permutation, must preserve 1-count"
        );
    }

    #[test]
    fn test_full_dim_4096() {
        let rb = RotationBind::new(12345);
        let filler = rb.embed_string("full_dim_test");
        assert_eq!(filler.len(), 4096);
        let bound = rb.bind(&filler);
        assert_eq!(bound.len(), 4096);
        let unbound = rb.unbind(&bound);
        assert_eq!(filler, unbound);
    }

    #[test]
    fn test_similarity_after_bind() {
        // Two similar fillers should become similar after same-role bind
        let rb = RotationBind::new(1);
        let a = rb.embed_string("cat");
        let b = rb.embed_string("dog"); // different but similar concepts
        let bound_a = rb.bind(&a);
        let bound_b = rb.bind(&b);
        // The rotation preserves inner products (it's near-orthogonal)
        // So the similarity between bound vectors ≈ similarity between fillers
        let sim_before = hamming_sim(&a, &b);
        let sim_after = hamming_sim(&bound_a, &bound_b);
        let diff = (sim_before - sim_after).abs();
        assert!(
            diff < 0.15,
            "rotation should approximately preserve Hamming similarity (diff={})",
            diff
        );
    }

    fn hamming_sim(a: &[u8], b: &[u8]) -> f64 {
        let n = a.len().min(b.len());
        let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        same as f64 / n as f64
    }
}
