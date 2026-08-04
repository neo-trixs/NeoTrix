use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Self-certifying embedding commitment binding source chunks to their quantized
/// embeddings via a Merkle root. Enables probabilistic authenticity audit without
/// re-querying the embedding model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCommitment {
    pub node_id: String,
    pub merkle_root: [u8; 32],
    pub leaf_hashes: Vec<[u8; 32]>,
    pub quantized_vector: Vec<i8>,
    pub original_vector: Vec<f32>,
    pub quantize_min: f32,
    pub quantize_max: f32,
    pub model_name: String,
    pub dimension: u32,
    pub timestamp: u64,
    pub domain_separator: String,
    /// 作者/来源标识 (吸收 cyberstrike authenticity + teamlore provenance):
    /// 将作者身份折叠进 Merkle 分块, 使承诺既证"未被篡改"又证"出自何人"。
    /// 空字符串 = 未绑定作者 (向后兼容旧数据)。
    #[serde(default)]
    pub author: String,
}

/// A Merkle proof for a single chunk within an embedding commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentProof {
    pub node_id: String,
    pub leaf_index: usize,
    pub sibling_hashes: Vec<[u8; 32]>,
    pub leaf_hash: [u8; 32],
    pub root_hash: [u8; 32],
    pub verified: bool,
}

/// A position-length binding that prevents chunk reordering attacks.
/// Each chunk is cryptographically bound to its index and node ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionLengthBinding {
    pub node_id: String,
    pub chunk_count: u32,
    pub original_length: u32,
    pub position_hash: [u8; 32],
}

/// Result of a probabilistic authenticity audit over a random sample of commitments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub samples_checked: usize,
    pub corruption_found: bool,
    pub corruption_count: usize,
    pub detection_probability: f64,
    pub verified_count: usize,
}

/// On-chain-style commitment store that maps node IDs to embedding commitments.
/// Supports up to `max_commitments` entries and optional JSON persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCommitmentStore {
    pub commitments: HashMap<String, EmbeddingCommitment>,
    pub max_commitments: usize,
    pub store_path: Option<PathBuf>,
}

impl EmbeddingCommitmentStore {
    pub fn new(max_commitments: usize, store_path: Option<PathBuf>) -> Self {
        Self {
            commitments: HashMap::new(),
            max_commitments,
            store_path,
        }
    }

    /// Quantize an f32 vector, split into 32-byte chunks, build a Merkle tree,
    /// and store the resulting commitment keyed by `node_id`.
    pub fn commit_vector(
        &mut self,
        node_id: String,
        vector: &[f32],
        model_name: String,
        domain_separator: String,
    ) -> Result<EmbeddingCommitment, String> {
        if vector.is_empty() {
            return Err("cannot commit an empty vector".to_string());
        }
        if self.commitments.contains_key(&node_id) {
            return Err(format!("commitment already exists for node '{}'", node_id));
        }
        if self.commitments.len() >= self.max_commitments {
            return Err("commitment store at capacity".to_string());
        }

        let (quantized, qmin, qmax) = Self::quantize_with_bounds(vector);
        let chunks = Self::chunk_bytes(&quantized, &domain_separator);
        let (merkle_root, leaf_hashes) = Self::merkle_tree(&chunks);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let commitment = EmbeddingCommitment {
            node_id: node_id.clone(),
            merkle_root,
            leaf_hashes,
            quantized_vector: quantized,
            original_vector: vector.to_vec(),
            quantize_min: qmin,
            quantize_max: qmax,
            model_name,
            dimension: vector.len() as u32,
            timestamp,
            domain_separator,
            author: String::new(),
        };

        self.commitments.insert(node_id, commitment.clone());
        Ok(commitment)
    }

    /// 作者绑定版 `commit_vector`: 把 `author` 折叠进分块 domain_separator,
    /// 使 Merkle root 与作者身份耦合。同一向量被另一作者认领时会得到不同的
    /// root → 伪造作者失败 (吸收 CyberStrike 真实性 / teamlore 来源绑定)。
    pub fn commit_vector_authored(
        &mut self,
        node_id: String,
        vector: &[f32],
        model_name: String,
        domain_separator: String,
        author: &str,
    ) -> Result<EmbeddingCommitment, String> {
        let sep = Self::separator_with_author(&domain_separator, author);
        let mut c = self.commit_vector(node_id, vector, model_name, sep)?;
        c.author = author.to_string();
        self.commitments.insert(c.node_id.clone(), c.clone());
        Ok(c)
    }

    /// 校验某节点承诺的作者是否 `expected`。未绑定作者 (author=="") 视为不参与
    /// 来源校验 (兼容旧数据), 返回 true 以免破坏历史承诺。
    pub fn verify_author(&self, node_id: &str, expected: &str) -> bool {
        match self.commitments.get(node_id) {
            None => false,
            Some(c) => c.author.is_empty() || c.author == expected,
        }
    }

    /// 完整性 + 来源双门校验: root 必须匹配 (未被篡改) 且作者身份匹配 (防伪造)。
    /// 只有两者同时成立才返回 verified=true。
    pub fn verify_commitment_origin(
        &self,
        node_id: &str,
        vector: &[f32],
        expected_author: &str,
    ) -> Result<CommitmentProof, String> {
        let mut proof = self.verify_commitment(node_id, vector)?;
        proof.verified = proof.verified && self.verify_author(node_id, expected_author);
        Ok(proof)
    }

    /// 将作者身份与所在域分离器绑定, 生成作者专属的分块上下文。
    /// 空作者不改变原 separator (向后兼容)。
    fn separator_with_author(domain_separator: &str, author: &str) -> String {
        if author.is_empty() {
            return domain_separator.to_string();
        }
        let mut h = Sha256::new();
        h.update(b"nt.author.v1");
        h.update(domain_separator.as_bytes());
        h.update(b"\x00");
        h.update(author.as_bytes());
        let binding = h.finalize();
        format!("{}:{}", domain_separator, hex::encode(binding))
    }

    /// Re-quantize the supplied vector, re-build the Merkle tree, and compare
    /// against the stored root. Returns a `CommitmentProof` with the
    /// `verified` flag set accordingly.
    pub fn verify_commitment(
        &self,
        node_id: &str,
        vector: &[f32],
    ) -> Result<CommitmentProof, String> {
        let commitment = self
            .commitments
            .get(node_id)
            .ok_or_else(|| format!("no commitment for node '{}'", node_id))?;

        let (requantized, _, _) = Self::quantize_with_bounds(vector);
        let chunks = Self::chunk_bytes(&requantized, &commitment.domain_separator);

        if chunks.is_empty() {
            return Err("vector produced zero chunks".to_string());
        }

        let leaf_hashes: Vec<[u8; 32]> = chunks.iter().map(|c| Self::sha2_256(c)).collect();
        let recomputed_root = Self::merkle_root_from_leaves(&leaf_hashes);
        let matches_any = (0..leaf_hashes.len()).any(|i| {
            let proof = Self::merkle_proof_inner(i, &leaf_hashes);
            let mut current = leaf_hashes[i];
            let mut idx = i;
            for sibling in &proof {
                let mut hasher = Sha256::new();
                if idx.is_multiple_of(2) {
                    hasher.update(current);
                    hasher.update(sibling);
                } else {
                    hasher.update(sibling);
                    hasher.update(current);
                }
                current = hasher.finalize().into();
                idx /= 2;
            }
            current == commitment.merkle_root
        });

        let verified = recomputed_root == commitment.merkle_root || matches_any;

        Ok(CommitmentProof {
            node_id: node_id.to_string(),
            leaf_index: 0,
            sibling_hashes: if leaf_hashes.is_empty() {
                Vec::new()
            } else {
                Self::merkle_proof_inner(0, &leaf_hashes)
            },
            leaf_hash: if leaf_hashes.is_empty() {
                [0u8; 32]
            } else {
                leaf_hashes[0]
            },
            root_hash: recomputed_root,
            verified,
        })
    }

    /// Build a Merkle tree from 32-byte chunks. Returns (root, all_leaf_hashes).
    pub fn merkle_tree(chunks: &[[u8; 32]]) -> ([u8; 32], Vec<[u8; 32]>) {
        let leaf_hashes: Vec<[u8; 32]> = chunks.iter().map(|c| Self::sha2_256(c)).collect();
        let root = Self::merkle_root_from_leaves(&leaf_hashes);
        (root, leaf_hashes)
    }

    /// Compute the Merkle sibling path for `leaf_index` in a tree built from
    /// `leaf_hashes`. The path proceeds from leaf to root, sibling by sibling.
    pub fn merkle_proof(leaf_index: usize, leaf_hashes: &[[u8; 32]]) -> Vec<[u8; 32]> {
        Self::merkle_proof_inner(leaf_index, leaf_hashes)
    }

    /// Verify a `CommitmentProof` by re-computing the Merkle root from the leaf
    /// hash and sibling hashes, then comparing to the stored root.
    pub fn verify_proof(proof: &CommitmentProof) -> bool {
        let mut current = proof.leaf_hash;
        let mut idx = proof.leaf_index;

        for sibling in &proof.sibling_hashes {
            let mut hasher = Sha256::new();
            if idx.is_multiple_of(2) {
                hasher.update(current);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current);
            }
            current = hasher.finalize().into();
            idx /= 2;
        }

        current == proof.root_hash
    }

    /// Run a probabilistic authenticity audit. Randomly samples up to
    /// `sample_count` commitments, re-verifies each, and reports the
    /// estimated detection probability for the observed corruption rate.
    pub fn probabilistic_audit(&self, sample_count: usize) -> AuditResult {
        let total = self.commitments.len();
        if total == 0 || sample_count == 0 {
            return AuditResult {
                samples_checked: 0,
                corruption_found: false,
                corruption_count: 0,
                detection_probability: 0.0,
                verified_count: 0,
            };
        }

        let sample_size = sample_count.min(total);
        let keys: Vec<&String> = self.commitments.keys().take(sample_size).collect();
        // use reservoir sampling for genuine random selection
        let keys = {
            let mut rng = rand::thread_rng();
            let mut reservoir: Vec<&String> = keys.to_vec();
            for (i, key) in self.commitments.keys().enumerate() {
                if i < sample_size {
                    reservoir[i] = key;
                } else {
                    let j = rng.gen_range(0..=i);
                    if j < sample_size {
                        reservoir[j] = key;
                    }
                }
            }
            reservoir.truncate(sample_size);
            reservoir
        };

        let mut corruption_count = 0usize;
        let mut verified_count = 0usize;

        for key in &keys {
            let commitment = match self.commitments.get(*key) {
                Some(c) => c,
                None => continue,
            };
            let chunks = Self::chunk_bytes(&commitment.quantized_vector, &commitment.domain_separator);
            let leaf_hashes: Vec<[u8; 32]> = chunks.iter().map(|c| Self::sha2_256(c)).collect();
            let recomputed_root = Self::merkle_root_from_leaves(&leaf_hashes);

            if recomputed_root == commitment.merkle_root {
                verified_count += 1;
            } else {
                corruption_count += 1;
            }
        }

        let rho = if sample_size > 0 {
            corruption_count as f64 / sample_size as f64
        } else {
            0.0
        };
        let detection_probability = 1.0 - (1.0 - rho).powi(sample_size as i32);

        AuditResult {
            samples_checked: sample_size,
            corruption_found: corruption_count > 0,
            corruption_count,
            detection_probability,
            verified_count,
        }
    }

    /// Linearly quantize an f32 vector to 4-bit i8 values in [-8, +7].
    pub fn quantize(vector: &[f32]) -> Vec<i8> {
        let (q, _, _) = Self::quantize_with_bounds(vector);
        q
    }

    /// Approximate reconstruction from quantized values using stored min/max.
    pub fn dequantize(quantized: &[i8], original_dim: u32) -> Vec<f32> {
        let len = quantized.len() as u32;
        let effective_dim = original_dim.max(len);
        // Without stored min/max we fall back to a fixed [-1, 1] assumption.
        let scale = 2.0 / 15.0;
        let mut result = Vec::with_capacity(effective_dim as usize);
        for &q in quantized.iter() {
            result.push(-1.0 + (q as f32 + 8.0) * scale + scale * 0.5);
        }
        while result.len() < effective_dim as usize {
            result.push(0.0);
        }
        result.truncate(effective_dim as usize);
        result
    }

    /// Serialize the store to a JSON file at the given path.
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialization error: {}", e))?;
        fs::write(path, json).map_err(|e| format!("write error: {}", e))?;
        Ok(())
    }

    /// Deserialize a store from a JSON file at the given path.
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;
        let store: Self =
            serde_json::from_str(&data).map_err(|e| format!("deserialization error: {}", e))?;
        Ok(store)
    }

    /// Remove a commitment by node_id. Returns true if it existed.
    pub fn remove(&mut self, node_id: &str) -> bool {
        self.commitments.remove(node_id).is_some()
    }

    /// Compute a `PositionLengthBinding` for `node_id` that binds each
    /// chunk to its position and to the node, preventing reordering attacks.
    pub fn compute_position_binding(
        &self,
        node_id: &str,
    ) -> Result<PositionLengthBinding, String> {
        let commitment = self
            .commitments
            .get(node_id)
            .ok_or_else(|| format!("no commitment for node '{}'", node_id))?;

        let chunks = Self::chunk_bytes(&commitment.quantized_vector, &commitment.domain_separator);
        let chunk_count = chunks.len() as u32;
        let original_length = commitment.dimension;

        // Build position-bound leaf hashes: H(i || chunk || node_id)
        let node_bytes = commitment.node_id.as_bytes();
        let pos_leaf_hashes: Vec<[u8; 32]> = chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                let mut hasher = Sha256::new();
                hasher.update((i as u32).to_le_bytes());
                hasher.update(chunk);
                hasher.update(node_bytes);
                hasher.finalize().into()
            })
            .collect();

        let position_hash = Self::merkle_root_from_leaves(&pos_leaf_hashes);

        Ok(PositionLengthBinding {
            node_id: node_id.to_string(),
            chunk_count,
            original_length,
            position_hash,
        })
    }

    /// Verify that a `PositionLengthBinding` is valid for the stored commitment.
    pub fn verify_position_binding(&self, binding: &PositionLengthBinding) -> bool {
        let _commitment = match self.commitments.get(&binding.node_id) {
            Some(c) => c,
            None => return false,
        };

        let recomputed = match self.compute_position_binding(&binding.node_id) {
            Ok(b) => b,
            Err(_) => return false,
        };

        recomputed.position_hash == binding.position_hash
            && recomputed.chunk_count == binding.chunk_count
            && recomputed.original_length == binding.original_length
    }

    // ── private helpers ──────────────────────────────────────────

    fn quantize_with_bounds(vector: &[f32]) -> (Vec<i8>, f32, f32) {
        if vector.is_empty() {
            return (Vec::new(), 0.0, 0.0);
        }
        let min = vector
            .iter()
            .copied()
            .reduce(f32::min)
            .unwrap_or(0.0);
        let max = vector
            .iter()
            .copied()
            .reduce(f32::max)
            .unwrap_or(0.0);

        if (max - min).abs() < f32::EPSILON {
            return (vec![0i8; vector.len()], min, max);
        }

        let scale = (max - min) / 15.0;
        let quantized: Vec<i8> = vector
            .iter()
            .map(|&v| {
                let q = ((v - min) / scale - 8.0).round();
                (q.max(-8.0).min(7.0)) as i8
            })
            .collect();
        (quantized, min, max)
    }

    fn chunk_bytes(quantized: &[i8], domain_separator: &str) -> Vec<[u8; 32]> {
        let num_chunks = quantized.len().div_ceil(32);
        let mut chunks = Vec::with_capacity(num_chunks);
        for i in 0..num_chunks {
            let start = i * 32;
            let end = start + 32.min(quantized.len().saturating_sub(start));
            let mut chunk = [0u8; 32];
            for (j, &val) in quantized[start..end].iter().enumerate() {
                chunk[j] = val as u8;
            }
            // Incorporate domain separator into each chunk to prevent
            // cross-domain hash collisions.
            let sep_bytes = domain_separator.as_bytes();
            if !sep_bytes.is_empty() {
                let mix_len = sep_bytes.len().min(32);
                for j in 0..mix_len {
                    chunk[j] = chunk[j].wrapping_add(sep_bytes[j % sep_bytes.len()]);
                }
            }
            chunks.push(chunk);
        }
        chunks
    }

    fn sha2_256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }

    fn merkle_root_from_leaves(leaves: &[[u8; 32]]) -> [u8; 32] {
        if leaves.is_empty() {
            return [0u8; 32];
        }
        let mut level: Vec<[u8; 32]> = leaves.to_vec();
        while level.len() > 1 {
            let mut next = Vec::with_capacity(level.len().div_ceil(2));
            for pair in level.chunks(2) {
                let mut hasher = Sha256::new();
    hasher.update(pair[0]);
    hasher.update(if pair.len() > 1 { &pair[1] } else { &pair[0] });
                let result = hasher.finalize();
                let mut h = [0u8; 32];
                h.copy_from_slice(&result);
                next.push(h);
            }
            level = next;
        }
        level[0]
    }

    fn merkle_proof_inner(leaf_index: usize, leaf_hashes: &[[u8; 32]]) -> Vec<[u8; 32]> {
        let mut siblings = Vec::new();
        let mut level: Vec<[u8; 32]> = leaf_hashes.to_vec();
        let mut idx = leaf_index;

        while level.len() > 1 {
            let sibling_idx = if idx.is_multiple_of(2) {
                if idx + 1 < level.len() {
                    idx + 1
                } else {
                    idx
                }
            } else {
                idx - 1
            };
            siblings.push(level[sibling_idx]);

            let mut next = Vec::with_capacity(level.len().div_ceil(2));
            for pair in level.chunks(2) {
                let mut hasher = Sha256::new();
    hasher.update(pair[0]);
    hasher.update(if pair.len() > 1 { &pair[1] } else { &pair[0] });
                let result = hasher.finalize();
                let mut h = [0u8; 32];
                h.copy_from_slice(&result);
                next.push(h);
            }
            idx /= 2;
            level = next;
        }

        siblings
    }

    fn _dequantize_with_bounds(quantized: &[i8], qmin: f32, qmax: f32, dimension: u32) -> Vec<f32> {
        if quantized.is_empty() {
            return Vec::new();
        }
        let effective_len = (dimension as usize).max(quantized.len());
        let scale = if (qmax - qmin).abs() < f32::EPSILON {
            0.0
        } else {
            (qmax - qmin) / 15.0
        };
        let mut result = Vec::with_capacity(effective_len);
        for &q in quantized {
            let val = if scale > 0.0 {
                qmin + (q as f32 + 8.0) * scale + scale * 0.5
            } else {
                qmin
            };
            result.push(val);
        }
        while result.len() < effective_len {
            result.push(0.0);
        }
        result.truncate(effective_len);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_vector(length: usize) -> Vec<f32> {
        (0..length).map(|i| (i as f32) / (length as f32) * 2.0 - 1.0).collect()
    }

    fn _uniform_vector(length: usize, value: f32) -> Vec<f32> {
        vec![value; length]
    }

    fn make_store() -> EmbeddingCommitmentStore {
        EmbeddingCommitmentStore::new(500000, None)
    }

    #[test]
    fn test_commit_and_verify_known_vector_returns_true() {
        let mut store = make_store();
        let vector = test_vector(128);
        let _commitment = store
            .commit_vector("n1".to_string(), &vector, "test-model".to_string(), "test-domain".to_string())
            .expect("commit should succeed");

        // Verify with the same vector — should succeed
        let proof = store
            .verify_commitment("n1", &vector)
            .expect("verify should succeed");
        assert!(proof.verified, "proof should be verified");
    }

    #[test]
    fn test_tampered_vector_fails_verification() {
        let mut store = make_store();
        let vector = test_vector(128);
        store
            .commit_vector("n1".to_string(), &vector, "test-model".to_string(), "test-domain".to_string())
            .expect("commit should succeed");

        // Tampered vector (flip a sign)
        let mut tampered = vector.clone();
        tampered[0] = -tampered[0];

        let proof = store
            .verify_commitment("n1", &tampered)
            .expect("verify should not error");
        assert!(!proof.verified, "tampered vector should fail verification");
    }

    #[test]
    fn test_merkle_tree_produces_verifiable_proofs() {
        let mut store = make_store();
        let vector = test_vector(256);
        store
            .commit_vector("n1".to_string(), &vector, "m".to_string(), "d".to_string())
            .expect("commit");

        let commitment = store.commitments.get("n1").unwrap();
        let leaf_count = commitment.leaf_hashes.len();
        assert!(leaf_count > 0, "should have leaf hashes");

        // Verify proof for every leaf
        for i in 0..leaf_count {
            let siblings = EmbeddingCommitmentStore::merkle_proof(i, &commitment.leaf_hashes);
            let proof = CommitmentProof {
                node_id: "n1".to_string(),
                leaf_index: i,
                sibling_hashes: siblings,
                leaf_hash: commitment.leaf_hashes[i],
                root_hash: commitment.merkle_root,
                verified: false,
            };
            assert!(
                EmbeddingCommitmentStore::verify_proof(&proof),
                "proof for leaf {} should verify",
                i
            );
        }
    }

    #[test]
    fn test_quantization_roundtrip_mse_below_threshold() {
        let vector = test_vector(768);
        let (quantized, qmin, qmax) = EmbeddingCommitmentStore::quantize_with_bounds(&vector);
        let reconstructed =
            EmbeddingCommitmentStore::_dequantize_with_bounds(&quantized, qmin, qmax, vector.len() as u32);

        assert_eq!(vector.len(), reconstructed.len());
        let mse: f64 = vector
            .iter()
            .zip(reconstructed.iter())
            .map(|(a, b)| ((a - b) as f64).powi(2))
            .sum::<f64>()
            / vector.len() as f64;
        assert!(mse < 0.1, "MSE = {} should be < 0.1", mse);
    }

    #[test]
    fn test_probabilistic_audit_detects_corruption() {
        let mut store = make_store();
        // Commit several vectors
        for i in 0..20 {
            let v = test_vector(64);
            store
                .commit_vector(
                    format!("n{}", i),
                    &v,
                    "m".to_string(),
                    "d".to_string(),
                )
                .expect("commit");
        }

        // Tamper one commitment directly
        if let Some(c) = store.commitments.get_mut("n5") {
            c.merkle_root[0] ^= 0xFF;
        }

        // Audit should detect corruption
        let result = store.probabilistic_audit(20);
        assert!(result.corruption_found, "audit should find corruption");
        assert_eq!(result.corruption_count, 1, "exactly one corrupted");
        assert!(result.detection_probability > 0.0, "detection prob > 0");
    }

    #[test]
    fn test_save_load_roundtrip_preserves_commitments() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("commitments.json");

        let mut store = make_store();
        for i in 0..5 {
            let v = test_vector(64);
            store
                .commit_vector(
                    format!("n{}", i),
                    &v,
                    "m".to_string(),
                    "d".to_string(),
                )
                .expect("commit");
        }
        store.save(&path).expect("save");

        let loaded = EmbeddingCommitmentStore::load(&path).expect("load");
        assert_eq!(loaded.commitments.len(), 5);
        for i in 0..5 {
            let key = format!("n{}", i);
            let orig = store.commitments.get(&key).unwrap();
            let loaded_c = loaded.commitments.get(&key).unwrap();
            assert_eq!(orig.merkle_root, loaded_c.merkle_root);
            assert_eq!(orig.quantized_vector, loaded_c.quantized_vector);
            assert_eq!(orig.model_name, loaded_c.model_name);
        }
    }

    #[test]
    fn test_position_binding_prevents_chunk_reordering() {
        let mut store = make_store();
        let vector = test_vector(128);
        store
            .commit_vector("n1".to_string(), &vector, "m".to_string(), "d".to_string())
            .expect("commit");

        let binding = store
            .compute_position_binding("n1")
            .expect("position binding");

        // Verify the binding is consistent
        assert!(store.verify_position_binding(&binding));

        // Tamper with chunk_count
        let tampered = PositionLengthBinding {
            chunk_count: binding.chunk_count + 999,
            ..binding.clone()
        };
        assert!(!store.verify_position_binding(&tampered));

        // Tamper with node_id
        let tampered2 = PositionLengthBinding {
            node_id: "wrong".to_string(),
            ..binding.clone()
        };
        assert!(!store.verify_position_binding(&tampered2));
    }

    #[test]
    fn test_multiple_commitments_stored_and_retrievable() {
        let mut store = make_store();
        let mut roots = Vec::new();
        for i in 0..10 {
            let v = test_vector(64 + i * 8);
            let c = store
                .commit_vector(
                    format!("n{}", i),
                    &v,
                    "test-model".to_string(),
                    "test-domain".to_string(),
                )
                .expect("commit");
            roots.push(c.merkle_root);
        }
        assert_eq!(store.commitments.len(), 10);

        for i in 0..10 {
            let c = store.commitments.get(&format!("n{}", i)).unwrap();
            assert_eq!(c.merkle_root, roots[i]);
        }
    }

    #[test]
    fn test_domain_separation_produces_different_roots() {
        let mut store = make_store();
        let vector = test_vector(64);

        let c1 = store
            .commit_vector("a".to_string(), &vector, "m".to_string(), "domain1".to_string())
            .expect("commit");
        let c2 = store
            .commit_vector("b".to_string(), &vector, "m".to_string(), "domain2".to_string())
            .expect("commit");

        assert_ne!(
            c1.merkle_root, c2.merkle_root,
            "different domains should produce different roots"
        );
    }

    #[test]
    fn test_remove_deletes_commitment() {
        let mut store = make_store();
        let vector = test_vector(64);
        store
            .commit_vector("n1".to_string(), &vector, "m".to_string(), "d".to_string())
            .expect("commit");
        assert!(store.commitments.contains_key("n1"));

        let existed = store.remove("n1");
        assert!(existed, "remove should return true");
        assert!(!store.commitments.contains_key("n1"));

        let not_existed = store.remove("n1");
        assert!(!not_existed, "second remove should return false");
    }

    #[test]
    fn test_proof_verification_for_all_leaf_indices() {
        let vector = test_vector(300);
        let (quantized, _, _) = EmbeddingCommitmentStore::quantize_with_bounds(&vector);
        let chunks = EmbeddingCommitmentStore::chunk_bytes(&quantized, "test");
        let (root, leaf_hashes) = EmbeddingCommitmentStore::merkle_tree(&chunks);

        for i in 0..leaf_hashes.len() {
            let siblings = EmbeddingCommitmentStore::merkle_proof(i, &leaf_hashes);
            let proof = CommitmentProof {
                node_id: "x".to_string(),
                leaf_index: i,
                sibling_hashes: siblings,
                leaf_hash: leaf_hashes[i],
                root_hash: root,
                verified: false,
            };
            assert!(
                EmbeddingCommitmentStore::verify_proof(&proof),
                "leaf {} proof should verify",
                i
            );
        }
    }

    #[test]
    fn test_empty_vector_edge_case() {
        let mut store = make_store();
        let result = store.commit_vector(
            "empty".to_string(),
            &[],
            "m".to_string(),
            "d".to_string(),
        );
        assert!(result.is_err(), "empty vector commit should fail");
    }

    #[test]
    fn test_single_element_vector_roundtrip() {
        let mut store = make_store();
        let v = vec![3.14159f32];
        store
            .commit_vector("pi".to_string(), &v, "m".to_string(), "d".to_string())
            .expect("commit single element");

        let proof = store
            .verify_commitment("pi", &v)
            .expect("verify single element");
        assert!(proof.verified);
    }

    #[test]
    fn test_max_capacity_enforced() {
        let mut store = EmbeddingCommitmentStore::new(3, None);
        for i in 0..3 {
            let v = test_vector(32);
            store
                .commit_vector(format!("n{}", i), &v, "m".to_string(), "d".to_string())
                .expect("commit within capacity");
        }
        let v = test_vector(32);
        let result = store.commit_vector("n4".to_string(), &v, "m".to_string(), "d".to_string());
        assert!(result.is_err(), "should reject beyond capacity");
    }

    #[test]
    fn test_original_vector_preserved_after_commit() {
        let mut store = make_store();
        let v = test_vector(128);
        store
            .commit_vector("n1".to_string(), &v, "m".to_string(), "d".to_string())
            .expect("commit");

        let c = store.commitments.get("n1").unwrap();
        assert_eq!(c.original_vector, v);
        assert_eq!(c.dimension, 128);
    }

    #[test]
    fn test_commit_authored_binds_author_and_verifies() {
        let mut store = make_store();
        let v = test_vector(64);
        store
            .commit_vector_authored(
                "a1".to_string(),
                &v,
                "m".to_string(),
                "d".to_string(),
                "alice",
            )
            .expect("authored commit");

        assert_eq!(store.commitments.get("a1").unwrap().author, "alice");
        // 真实作者 + 完好向量 → 完整性+来源双过
        let proof = store
            .verify_commitment_origin("a1", &v, "alice")
            .expect("origin verify");
        assert!(proof.verified, "author-bound commitment must verify");
    }

    #[test]
    fn test_forged_author_fails_origin_verification() {
        let mut store = make_store();
        let v = test_vector(64);
        store
            .commit_vector_authored(
                "a2".to_string(),
                &v,
                "m".to_string(),
                "d".to_string(),
                "alice",
            )
            .expect("authored commit");

        // 伪造作者 (mal 冒充 alice) → 来源门拒绝
        let proof = store
            .verify_commitment_origin("a2", &v, "mal")
            .expect("origin verify");
        assert!(!proof.verified, "forged author must fail origin verification");

        // 作者不同 → root 不同 (不能借 alice 的内容改名真 own)
        let mut other = store.clone();
        other
            .commit_vector_authored(
                "a3".to_string(),
                &v,
                "m".to_string(),
                "d".to_string(),
                "bob",
            )
            .expect("bob authored commit");
        let root_alice = store.commitments.get("a2").unwrap().merkle_root;
        let root_bob = other.commitments.get("a3").unwrap().merkle_root;
        assert_ne!(root_alice, root_bob, "author must bind into the Merkle root");
    }

    #[test]
    fn test_tampered_vector_under_correct_author_fails() {
        let mut store = make_store();
        let v = test_vector(64);
        store
            .commit_vector_authored(
                "a4".to_string(),
                &v,
                "m".to_string(),
                "d".to_string(),
                "alice",
            )
            .expect("authored commit");

        let mut tampered = v.clone();
        tampered[0] = -tampered[0];
        let proof = store
            .verify_commitment_origin("a4", &tampered, "alice")
            .expect("origin verify");
        assert!(
            !proof.verified,
            "tampered content must fail even under correct author"
        );
    }
}
