//! Phase 10.2 — End-to-End Latent Reasoning (端到端潜在推理 · LatentUM §5).
//!
//! Replaces the text round-trip `E8 → text → LLM → text → GWT` with a direct
//! latent path:
//!
//!   E8 latent thought → latent nearest-neighbor query → GWT broadcast (direct)
//!   expert response → update E8 state (as next thought)
//!
//! The pipeline maintains a latent episodic buffer: each recorded reasoning
//! step stores its E8 state's unified-space embedding + outcome. When a new
//! latent thought arrives, the top-k nearest neighbors (cosine in the unified
//! latent space) are retrieved without any text serialization, and the
//! neighbors' E8 modes are emitted as a direct GWT attention bias — so the
//! broadcast is driven by latent continuity rather than text.

use crate::core::nt_core_e8::unified_latent::UnifiedLatentSpace;
use crate::core::nt_core_hex::ReasoningHexagram;
use serde::{Deserialize, Serialize};

/// Maximum number of latent episodic entries retained.
pub const LATENT_MEMORY_SIZE: usize = 256;
/// Number of nearest neighbors retrieved per query.
pub const TOP_K_NEIGHBORS: usize = 4;

/// One latent episodic entry: an E8 state's unified embedding + outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentEpisodicEntry {
    /// E8 hexagram mode this entry corresponds to.
    pub mode: u8,
    /// Unified-space embedding of the state (UNIFIED_LATENT_DIM).
    pub embedding: Vec<f64>,
    /// Outcome score (e.g. SEAL reward / PRM score) recorded with the step.
    pub outcome: f64,
    /// Source: "e8" | "gwt" | "hypercube".
    pub source: String,
}

/// Result of a latent nearest-neighbor query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentRetrieval {
    /// E8 modes of the top-k neighbors (in descending similarity).
    pub neighbor_modes: Vec<u8>,
    /// Cosine similarities of the neighbors.
    pub similarities: Vec<f64>,
    /// Outcome-weighted attention distribution over the 64 E8 states:
    /// each neighbor mode accumulates its outcome × similarity mass.
    pub attention: Vec<f64>,
}

/// Phase 10.2 — latent reasoning pipeline over the unified space.
///
/// A bounded latent episodic memory queried by nearest-neighbor retrieval,
/// emitting a direct E8→GWT attention vector (no text in the hot path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentReasoningPipeline {
    /// Unified latent space used for all embeddings and comparisons.
    pub unified: UnifiedLatentSpace,
    /// Episodic buffer of latent entries (most recent first).
    pub memory: Vec<LatentEpisodicEntry>,
    /// Maximum memory length.
    pub capacity: usize,
    /// Number of neighbors retrieved per query.
    pub top_k: usize,
    /// Total queries served.
    pub queries_served: u64,
}

impl Default for LatentReasoningPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl LatentReasoningPipeline {
    /// Create a pipeline with default capacity and top-k.
    pub fn new() -> Self {
        Self::new_with(LATENT_MEMORY_SIZE, TOP_K_NEIGHBORS)
    }

    /// Create a pipeline with explicit capacity / top-k.
    pub fn new_with(capacity: usize, top_k: usize) -> Self {
        Self {
            unified: UnifiedLatentSpace::new(),
            memory: Vec::with_capacity(capacity),
            capacity,
            top_k: top_k.max(1),
            queries_served: 0,
        }
    }

    /// Record a latent episodic entry.
    ///
    /// `state` is the E8 hexagram; `outcome` is its observed quality (SEAL/PRM).
    pub fn record(&mut self, state: ReasoningHexagram, outcome: f64, source: &str) {
        let embedding = self.unified.project_e8_state(state);
        self.memory.insert(
            0,
            LatentEpisodicEntry {
                mode: state.0,
                embedding,
                outcome,
                source: source.to_string(),
            },
        );
        while self.memory.len() > self.capacity {
            self.memory.pop();
        }
    }

    /// Record with an explicit embedding (avoids re-projecting when the caller
    /// already has the unified-space vector).
    pub fn record_embedded(&mut self, mode: u8, embedding: Vec<f64>, outcome: f64, source: &str) {
        self.memory.insert(
            0,
            LatentEpisodicEntry {
                mode,
                embedding,
                outcome,
                source: source.to_string(),
            },
        );
        while self.memory.len() > self.capacity {
            self.memory.pop();
        }
    }

    /// Query the latent memory with a unified-space vector.
    ///
    /// Returns the top-k nearest neighbors (by cosine) and a direct attention
    /// vector over the 64 E8 states, outcome-weighted. When memory is empty,
    /// returns a zero attention vector and empty neighbors (no text fallback).
    pub fn query(&mut self, query_embedding: &[f64]) -> LatentRetrieval {
        self.queries_served += 1;
        let mut scored: Vec<(f64, &LatentEpisodicEntry)> = self
            .memory
            .iter()
            .filter_map(|e| {
                let sim = self.unified.cosine(query_embedding, &e.embedding);
                if sim > 0.0 {
                    Some((sim, e))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let n = scored.len().min(self.top_k);
        let mut attention = vec![0.0f64; 64];
        let mut neighbor_modes = Vec::with_capacity(n);
        let mut similarities = Vec::with_capacity(n);
        let mut mass = 0.0f64;
        for (sim, entry) in scored.iter().take(n) {
            neighbor_modes.push(entry.mode);
            similarities.push(*sim);
            let w = (*sim).max(0.0) * entry.outcome.clamp(0.0, 1.0).max(0.1);
            attention[entry.mode as usize] += w;
            mass += w;
        }
        if mass > 0.0 {
            for a in attention.iter_mut() {
                *a /= mass;
            }
        }
        LatentRetrieval {
            neighbor_modes,
            similarities,
            attention,
        }
    }

    /// Query by an E8 state (embeds on the fly).
    pub fn query_state(&mut self, state: ReasoningHexagram) -> LatentRetrieval {
        let emb = self.unified.project_e8_state(state);
        self.query(&emb)
    }

    /// Emit the retrieval as a direct GWT attention vector.
    ///
    /// Returns `(weights, bias)` ready for `GlobalWorkspace::set_e8_attention_weights`.
    pub fn to_gwt_attention(&self, retrieval: &LatentRetrieval, base_bias: f64) -> ([f64; 64], f64) {
        let bias = base_bias + (1.0 - retrieval.similarities.first().copied().unwrap_or(0.0)) * 0.2;
        let mut arr = [0.0f64; 64];
        for (i, a) in retrieval.attention.iter().enumerate().take(64) {
            arr[i] = *a;
        }
        (arr, bias.clamp(0.0, 1.0))
    }

    /// Memory fill ratio (0..1) for telemetry.
    pub fn fill_ratio(&self) -> f64 {
        self.memory.len() as f64 / self.capacity.max(1) as f64
    }

    /// Clear all episodic memory.
    pub fn clear(&mut self) {
        self.memory.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_memory_returns_empty() {
        let mut p = LatentReasoningPipeline::new();
        let r = p.query_state(ReasoningHexagram::new(0));
        assert!(r.neighbor_modes.is_empty());
        assert!(r.attention.iter().all(|&a| a == 0.0));
    }

    #[test]
    fn test_record_and_query_finds_same_state() {
        let mut p = LatentReasoningPipeline::new();
        p.record(ReasoningHexagram::new(8), 0.9, "seal");
        p.record(ReasoningHexagram::new(56), 0.5, "seal");
        let r = p.query_state(ReasoningHexagram::new(9)); // one line flip from 8
        assert!(!r.neighbor_modes.is_empty());
        // Nearest neighbor should be mode 8 (closest to mode 9).
        assert_eq!(r.neighbor_modes[0], 8);
    }

    #[test]
    fn test_query_attention_is_normalized() {
        let mut p = LatentReasoningPipeline::new();
        for mode in [0u8, 8, 16, 24, 32, 40, 48, 56] {
            p.record(ReasoningHexagram::new(mode), 0.8, "seal");
        }
        let r = p.query_state(ReasoningHexagram::new(0));
        let sum: f64 = r.attention.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "attention should sum to 1, got {sum}");
        assert!(r.attention[0] > 0.0);
    }

    #[test]
    fn test_outcome_weights_attention() {
        let mut p = LatentReasoningPipeline::new();
        // Same mode, different outcomes: high-outcome dominates.
        p.record(ReasoningHexagram::new(32), 0.95, "seal");
        p.record(ReasoningHexagram::new(32), 0.1, "seal");
        p.record(ReasoningHexagram::new(33), 0.1, "seal");
        let r = p.query_state(ReasoningHexagram::new(32));
        // Mode 32 should carry the largest attention mass.
        let m32 = r.attention[32];
        let m33 = r.attention[33];
        assert!(
            m32 > m33,
            "high-outcome mode should dominate: 32={m32:.3} vs 33={m33:.3}"
        );
    }

    #[test]
    fn test_memory_bounded() {
        let mut p = LatentReasoningPipeline::new_with(16, 3);
        for i in 0..100u8 {
            p.record(ReasoningHexagram::new(i % 64), 0.5, "seal");
        }
        assert!(p.memory.len() <= 16);
        assert!(p.fill_ratio() <= 1.0);
    }

    #[test]
    fn test_to_gwt_attention_returns_valid() {
        let mut p = LatentReasoningPipeline::new();
        p.record(ReasoningHexagram::new(40), 0.8, "seal");
        let r = p.query_state(ReasoningHexagram::new(40));
        let (weights, bias) = p.to_gwt_attention(&r, 0.3);
        assert_eq!(weights.len(), 64);
        assert!(bias >= 0.0 && bias <= 1.0);
    }

    #[test]
    fn test_clear_resets_memory() {
        let mut p = LatentReasoningPipeline::new();
        p.record(ReasoningHexagram::new(8), 0.9, "seal");
        assert!(p.memory.len() == 1);
        p.clear();
        assert!(p.memory.is_empty());
    }

    #[test]
    fn test_query_returns_similarities_sorted() {
        let mut p = LatentReasoningPipeline::new();
        for mode in [0u8, 16, 32, 48] {
            p.record(ReasoningHexagram::new(mode), 0.7, "seal");
        }
        let r = p.query_state(ReasoningHexagram::new(0));
        // Similarities must be non-increasing.
        for w in r.similarities.windows(2) {
            assert!(w[0] >= w[1]);
        }
    }
}
