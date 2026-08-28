//! E5 — 双脑流式记忆 (dual-brain streaming working memory, C2).
//!
//! Fused consciousness-evolution dimension E5: the "short-term / streaming" brain
//! that feeds the Self-Model (FEP/IIT) and the Self-Improvement loop
//! (SEAL + RedQueen + JIT). Absorbed concepts:
//!
//! - **VoiceMem** (src8): dual-brain split of working vs long-term memory — this
//!   module is the *short-term* half; LTM lives in the KB (`KnowledgeBase`).
//! - **SinkTrack** (src11): anchor recent experience into a stable trace — every
//!   item is an `ExperienceAnchor` (id + content hash + timestamp + importance).
//! - **Agentic LTM/STM** (src12): long-term vs short-term memory with agentic
//!   retrieval — `recall_ltm` is the agentic retrieval stub toward the KB.
//!
//! The buffer is intentionally bounded (a ring) and does NOT persist; it is the
//! volatile STM that the long-term KB can be queried into via `recall_ltm`.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default ring capacity for the streaming buffer (STM half of the dual brain).
pub const DEFAULT_WORKING_CAPACITY: usize = 64;

/// A single anchored working-memory item (SinkTrack-style stable trace).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperienceAnchor {
    /// Stable id for the anchor (SinkTrack: a stable trace pointer).
    pub id: String,
    /// Content hash of the anchored experience (cheap dedup / integrity check).
    pub content_hash: String,
    /// Unix-epoch seconds when the anchor was created.
    pub timestamp: u64,
    /// Importance score (0..1) — drives agentic LTM retrieval prioritisation.
    pub importance: f64,
    /// Short textual content for echo / debugging / handoff.
    pub content: String,
}

impl ExperienceAnchor {
    /// Build an anchor from content; the hash and timestamp are derived.
    pub fn new(id: impl Into<String>, content: impl Into<String>, importance: f64) -> Self {
        let content = content.into();
        let content_hash = fnv1a_hex(content.as_bytes());
        Self {
            id: id.into(),
            content_hash,
            timestamp: now_secs(),
            importance: importance.clamp(0.0, 1.0),
            content,
        }
    }
}

/// Dual-brain streaming working-memory buffer (short-term brain, E5).
///
/// Bounded ring of recent `ExperienceAnchor`s. `anchor` pushes onto the ring;
/// `recall_recent` returns the most recent `n` sorted by importance (agentic
/// prioritisation). `recall_ltm` is the agentic long-term retrieval stub
/// (VoiceMem / Agentic LTM & STM) — it is a no-op until wired to `KnowledgeBase`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualBrainWorkingMemory {
    /// Bounded ring buffer of recent anchors (STM).
    buffer: VecDeque<ExperienceAnchor>,
    /// Max retained anchors (ring capacity).
    capacity: usize,
}

impl Default for DualBrainWorkingMemory {
    fn default() -> Self {
        Self::new(DEFAULT_WORKING_CAPACITY)
    }
}

impl DualBrainWorkingMemory {
    /// Create a streaming buffer with the given ring capacity (clamped to ≥1).
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Max retained anchors.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Current number of anchored items.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Anchor an experience into the stable trace (SinkTrack). Returns the anchor id.
    /// Evicts the oldest anchor when the ring is full.
    pub fn anchor(&mut self, item: ExperienceAnchor) -> String {
        let id = item.id.clone();
        self.buffer.push_back(item);
        while self.buffer.len() > self.capacity {
            self.buffer.pop_front();
        }
        id
    }

    /// Convenience anchor from text content with default importance (0.5).
    pub fn anchor_text(&mut self, id: impl Into<String>, content: impl Into<String>) -> String {
        let a = ExperienceAnchor::new(id, content, 0.5);
        self.anchor(a)
    }

    /// Recall the `n` most recent anchors, highest-importance first.
    /// Agentic LTM prioritisation begins with importance ordering (src12).
    pub fn recall_recent(&self, n: usize) -> Vec<ExperienceAnchor> {
        let mut recent: Vec<ExperienceAnchor> =
            self.buffer.iter().rev().take(n).cloned().collect();
        recent.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        recent
    }

    /// Agentic LTM retrieval stub (VoiceMem / Agentic LTM & STM).
    ///
    /// TODO(E5): wire to `KnowledgeBase` semantic search /
    /// `MemoryPrimitives::mem_retrieve` so the streaming brain can pull relevant
    /// long-term traces into the ring. Currently returns an empty set.
    pub fn recall_ltm(&self, _query: &str, _limit: usize) -> Vec<ExperienceAnchor> {
        Vec::new()
    }

    /// Drain all anchors (used at session close / handoff to LTM).
    pub fn drain(&mut self) -> Vec<ExperienceAnchor> {
        self.buffer.drain(..).collect()
    }
}

/// FNV-1a 64-bit hash rendered as a hex string (no external deps).
fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Current unix-epoch seconds (saturating).
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_and_recall_respect_capacity() {
        let mut wm = DualBrainWorkingMemory::new(3);
        for i in 0..5 {
            wm.anchor_text(format!("a{i}"), format!("content {i}"));
        }
        assert_eq!(wm.len(), 3, "ring must evict oldest");
        let recent = wm.recall_recent(3);
        assert_eq!(recent.len(), 3);
        assert!(recent.iter().all(|a| a.id.starts_with("a")));
        // Oldest anchors (a0, a1) must have been evicted.
        assert!(!recent.iter().any(|a| a.id == "a0" || a.id == "a1"));
    }

    #[test]
    fn recall_recent_sorts_by_importance() {
        let mut wm = DualBrainWorkingMemory::new(8);
        wm.anchor(ExperienceAnchor::new("low", "low-importance", 0.1));
        wm.anchor(ExperienceAnchor::new("high", "high-importance", 0.9));
        let recent = wm.recall_recent(8);
        assert_eq!(recent.first().unwrap().id, "high", "highest importance first");
    }

    #[test]
    fn anchor_is_stable_trace() {
        let a = ExperienceAnchor::new("x", "same content", 0.5);
        let b = ExperienceAnchor::new("x", "same content", 0.5);
        assert_eq!(a.content_hash, b.content_hash, "same content → same hash");
        assert!(a.timestamp > 0);
    }

    #[test]
    fn ltm_retrieval_is_stubbed() {
        let wm = DualBrainWorkingMemory::new(8);
        assert!(wm.recall_ltm("anything", 5).is_empty(), "LTM stub returns empty");
    }
}
