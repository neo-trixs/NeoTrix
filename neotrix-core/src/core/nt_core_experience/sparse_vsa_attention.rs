use std::collections::VecDeque;

use crate::core::nt_core_hcube::sparse_vsa::SparseBinaryVSA;
use crate::core::nt_core_hcube::vsa_bridge::VsaBridge;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

/// Zamba2-VL inspired sparse shared attention block for VSA pipeline.
///
/// Every N cycles, this module performs a "shared attention" computation over
/// evidence tokens retrieved from MemoryPalace / hypergraph, using VSA
/// bind/unbind as the attention mechanism (inspired by "Attention as Binding",
/// AAAI 2026). Between attention cycles, the pipeline runs "Mamba2-style"
/// fast VSA transformations (bundling + permutation) without external retrieval.
///
/// Architecture mirrors Zamba2-VL's hybrid design:
/// - ~90% fast VSA cycles (bundling / permutation — "SSM-like")
/// - ~10% shared attention blocks (bind/unbind over evidence — "Transformer-like")
pub const SHARED_ATTENTION_INTERVAL: u64 = 10;
pub const MAX_EVIDENCE_TOKENS: usize = 8;

#[derive(Debug, Clone)]
pub struct VsaAttentionBlock {
    pub query_vsa: Vec<u8>,
    pub key_vsas: Vec<Vec<u8>>,
    pub value_texts: Vec<String>,
    pub source_labels: Vec<String>,
    pub attention_output: Vec<u8>,
    pub attended_sources: Vec<String>,
}

impl VsaAttentionBlock {
    pub fn new(vsa_dim: usize) -> Self {
        Self {
            query_vsa: vec![0u8; vsa_dim],
            key_vsas: Vec::new(),
            value_texts: Vec::new(),
            source_labels: Vec::new(),
            attention_output: vec![0u8; vsa_dim],
            attended_sources: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key_vsas.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SparseVsaAttentionEngine {
    pub vsa_dim: usize,
    pub cycle_since_last_attention: u64,
    pub share_interval: u64,
    pub last_attention: Option<VsaAttentionBlock>,
    pub attention_history: VecDeque<VsaAttentionBlock>,
    pub max_history: usize,
    pub total_attention_cycles: u64,
    pub total_fast_cycles: u64,
}

impl SparseVsaAttentionEngine {
    pub fn new(vsa_dim: usize) -> Self {
        Self {
            vsa_dim,
            cycle_since_last_attention: 0,
            share_interval: SHARED_ATTENTION_INTERVAL,
            last_attention: None,
            attention_history: VecDeque::new(),
            max_history: 10,
            total_attention_cycles: 0,
            total_fast_cycles: 0,
        }
    }

    pub fn should_run_attention(&self) -> bool {
        self.cycle_since_last_attention >= self.share_interval
    }

    pub fn run_fast_cycle(&mut self, attractor: &mut [u8]) {
        self.cycle_since_last_attention += 1;
        self.total_fast_cycles += 1;
        // "SSM-like" fast transform: bundle attractor with a rotation of itself.
        // This is the "Mamba2-style" local transformation — no external retrieval.
        let mut rotated = attractor.to_vec();
        rotated.rotate_right(1);
        for (a, r) in attractor.iter_mut().zip(rotated.iter()) {
            *a = a.wrapping_add(*r);
        }
    }

    pub fn run_attention_cycle(
        &mut self,
        attractor: &[u8],
        evidence_keys: &[Vec<u8>],
        evidence_values: &[String],
        evidence_sources: &[String],
    ) -> Vec<u8> {
        self.cycle_since_last_attention = 0;
        self.total_attention_cycles += 1;

        let attended = if evidence_keys.is_empty() {
            // No evidence: attention is identity (zero-overhead fallback)
            attractor.to_vec()
        } else {
            // VSA-based attention: bind attractor (query) with each evidence key,
            // then bundle the bound results weighted by similarity.
            // This replaces softmax(QK^T)V with VSA bind/bundle operations.
            let mut accumulated = vec![0u8; self.vsa_dim];
            let mut attended_sources: Vec<String> = Vec::new();

            let max_tokens = evidence_keys.len().min(MAX_EVIDENCE_TOKENS);
            for i in 0..max_tokens {
                let key = &evidence_keys[i];
                let value_text = &evidence_values[i];
                let source = &evidence_sources[i];

                // VSA "attention": query ⊕ key (bind) measures compatibility
                let mut bound = vec![0u8; self.vsa_dim];
                for (b, (q, k)) in bound.iter_mut().zip(attractor.iter().zip(key.iter())) {
                    *b = q ^ k; // XOR binding = similarity in binary VSA
                }
                // Popcount-based attention weight: more 1s = less similar
                let sim: f64 = bound
                    .iter()
                    .map(|&b| (b.count_ones() as f64) / 8.0)
                    .sum::<f64>()
                    / self.vsa_dim as f64;
                // Normalize to [0,1]: XOR similarity, 0 = identical, 0.5 = random, 1 = opposite
                // Convert to attention weight: closer to 0 → higher weight
                let attn_weight = 1.0 - (sim * 2.0 - 1.0).abs();

                // Accumulate weighted evidence: bundle value VSA weighted by attention
                // Use seeded hash of value text as "VSA value vector"
                let value_vsa = self.value_to_vsa(value_text);
                for (a, v) in accumulated.iter_mut().zip(value_vsa.iter()) {
                    if attn_weight > 0.3 {
                        *a = a.wrapping_add((*v as f64 * attn_weight) as u8);
                    }
                }

                if attn_weight > 0.3 {
                    attended_sources.push(source.clone());
                }
            }

            // Bundle accumulated evidence with original attractor (residual connection)
            let mut output = attractor.to_vec();
            for (o, a) in output.iter_mut().zip(accumulated.iter()) {
                *o = o.wrapping_add(*a);
            }

            let block = VsaAttentionBlock {
                query_vsa: attractor.to_vec(),
                key_vsas: evidence_keys.iter().take(max_tokens).cloned().collect(),
                value_texts: evidence_values.iter().take(max_tokens).cloned().collect(),
                source_labels: evidence_sources.iter().take(max_tokens).cloned().collect(),
                attention_output: output.clone(),
                attended_sources,
            };
            self.last_attention = Some(block.clone());
            self.attention_history.push_back(block);
            if self.attention_history.len() > self.max_history {
                self.attention_history.pop_front();
            }

            output
        };

        attended
    }

    /// Sparse VSA attention cycle — uses SparseBinaryVSA with Jaccard similarity.
    /// Converts dense attractor/keys to sparse, scores via Jaccard, returns top-k evidence indices.
    pub fn run_sparse_attention_cycle(
        &mut self,
        attractor: &[u8],
        evidence_keys: &[Vec<u8>],
        evidence_values: &[String],
        evidence_sources: &[String],
        top_k: usize,
    ) -> Vec<usize> {
        self.cycle_since_last_attention = 0;
        self.total_attention_cycles += 1;

        if evidence_keys.is_empty() {
            return Vec::new();
        }

        let query_sparse: SparseBinaryVSA<VSA_DIM, 32> =
            VsaBridge::dense_to_sparse::<32>(attractor);
        let max_tokens = evidence_keys.len().min(MAX_EVIDENCE_TOKENS);
        let mut scored: Vec<(usize, f64)> = Vec::with_capacity(max_tokens);

        for i in 0..max_tokens {
            let key_sparse: SparseBinaryVSA<VSA_DIM, 32> =
                VsaBridge::dense_to_sparse::<32>(&evidence_keys[i]);
            let jaccard = SparseBinaryVSA::<VSA_DIM, 32>::similarity(&query_sparse, &key_sparse);
            scored.push((i, jaccard));
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let k = top_k.min(scored.len());
        let top_indices: Vec<usize> = scored.iter().take(k).map(|(idx, _)| *idx).collect();

        let attended_sources: Vec<String> = top_indices
            .iter()
            .filter_map(|&i| evidence_sources.get(i).cloned())
            .collect();
        let attended_values: Vec<String> = top_indices
            .iter()
            .filter_map(|&i| evidence_values.get(i).cloned())
            .collect();

        let block = VsaAttentionBlock {
            query_vsa: attractor.to_vec(),
            key_vsas: evidence_keys.iter().take(max_tokens).cloned().collect(),
            value_texts: attended_values,
            source_labels: evidence_sources.iter().take(max_tokens).cloned().collect(),
            attention_output: vec![0u8; self.vsa_dim],
            attended_sources,
        };
        self.last_attention = Some(block.clone());
        self.attention_history.push_back(block);
        if self.attention_history.len() > self.max_history {
            self.attention_history.pop_front();
        }

        log::debug!(
            "SVSA: sparse_attn top_k={}/{} jaccard_scores={:?}",
            k,
            max_tokens,
            scored
                .iter()
                .take(k)
                .map(|(_, s)| format!("{:.4}", s))
                .collect::<Vec<_>>()
                .join(",")
        );

        top_indices
    }

    /// Convert a text string to a deterministic VSA vector (seeded hash).
    fn value_to_vsa(&self, text: &str) -> Vec<u8> {
        use std::hash::{DefaultHasher, Hasher};
        let mut hasher = DefaultHasher::new();
        hasher.write(text.as_bytes());
        let seed = hasher.finish();
        let mut vsa = vec![0u8; self.vsa_dim];
        let mut rng = seed;
        for byte in vsa.iter_mut() {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *byte = (rng >> 40) as u8;
        }
        vsa
    }

    pub fn stats(&self) -> String {
        let ratio = if self.total_fast_cycles + self.total_attention_cycles > 0 {
            self.total_attention_cycles as f64
                / (self.total_fast_cycles + self.total_attention_cycles) as f64
        } else {
            0.0
        };
        format!(
            "svsa:attn={}_fast={}_ratio={:.3}_interval={}",
            self.total_attention_cycles, self.total_fast_cycles, ratio, self.share_interval,
        )
    }

    pub fn attention_summary(&self) -> String {
        if let Some(ref last) = self.last_attention {
            if last.attended_sources.is_empty() {
                "svsa_attn:no_evidence".to_string()
            } else {
                format!(
                    "svsa_attn:{}_sources=[{}]",
                    last.attended_sources.len(),
                    last.attended_sources.join(",")
                )
            }
        } else {
            "svsa_attn:never".to_string()
        }
    }
}
