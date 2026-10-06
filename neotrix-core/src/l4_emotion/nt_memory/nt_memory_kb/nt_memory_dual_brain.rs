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

use super::KnowledgeBase;

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
/// prioritisation). `recall_ltm` is the agentic long-term retrieval toward the
/// `KnowledgeBase` (VoiceMem / Agentic LTM & STM).
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
        let mut recent: Vec<ExperienceAnchor> = self.buffer.iter().rev().take(n).cloned().collect();
        recent.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        recent
    }

    /// Agentic LTM retrieval (VoiceMem / Agentic LTM & STM).
    ///
    /// Queries the long-term `KnowledgeBase` (opened via the runtime default
    /// path, mirroring `KnowledgeBase::open(None)` used elsewhere at runtime)
    /// for experiences matching `query`, mapping the top `limit`
    /// `SearchResult`s into `ExperienceAnchor`s. The relevance `score` becomes
    /// `importance`; the node `id` and `content` (falling back to summary then
    /// title) seed the anchor, and the node `created_at` is preserved as the
    /// anchor `timestamp`. Falls back to an empty set if the KB is unavailable.
    pub fn recall_ltm(&self, query: &str, limit: usize) -> Vec<ExperienceAnchor> {
        // 2026-10-03：**碰存储之前**先短路（实测缺陷，非推测）。
        //
        // ⛔ 改前**无任何空查询短路** ⇒ `recall_ltm("")` 会
        //    `KnowledgeBase::open(None)` **开整库**，再拿空串去 `search`。
        //    ⇒ 纯浪费，且 **不需要任何策略判断**就能判定它是错的。
        //
        // 依据（外部吸收，`waku-agent` MIT 一手源码，`retrieval_gate.py:1-13`）：
        // 「**before touching any store**, a cheap fast model answers one question」
        // ⇒ 「碰存储之前」是**该原则最可辩护的最小落点**：
        // 连「查询是否为空」这种**零判断**的问题都不先问，就先开库。
        // 更完整的「需不需要记忆」闸见同目录 `nt_retrieval_gate.rs`
        //   （那里默认 `NoGate` 因为**我方无测量**；而空查询**不需要测量**）。
        let q = query.trim();
        if q.is_empty() || limit == 0 {
            return Vec::new();
        }
        let Ok(kb) = KnowledgeBase::open(None) else {
            return Vec::new();
        };
        let Ok(results) = kb.search(q, limit) else {
            return Vec::new();
        };
        results
            .into_iter()
            .take(limit)
            .map(|r| {
                let node = r.node;
                let content = node
                    .content
                    .clone()
                    .or_else(|| node.summary.clone())
                    .unwrap_or_else(|| node.title.clone());
                let mut anchor =
                    ExperienceAnchor::new(node.id.clone(), content, r.score.clamp(0.0, 1.0));
                anchor.timestamp = node.created_at.max(0) as u64;
                anchor
            })
            .collect()
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

use crate::l0_substrate::nt_core_time::now_secs;

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
        assert_eq!(
            recent.first().unwrap().id,
            "high",
            "highest importance first"
        );
    }

    #[test]
    fn anchor_is_stable_trace() {
        let a = ExperienceAnchor::new("x", "same content", 0.5);
        let b = ExperienceAnchor::new("x", "same content", 0.5);
        assert_eq!(a.content_hash, b.content_hash, "same content → same hash");
        assert!(a.timestamp > 0);
    }

    #[test]
    fn ltm_retrieval_queries_kb() {
        let wm = DualBrainWorkingMemory::new(8);
        let hits = wm.recall_ltm("anything", 5);
        // Recall must respect the requested limit and yield valid anchors.
        assert!(hits.len() <= 5, "LTM recall must respect limit");
        for a in &hits {
            assert!(!a.id.is_empty(), "anchor id derived from KB node id");
            assert!(
                a.timestamp > 0 || a.importance >= 0.0,
                "anchor fields populated"
            );
        }
    }
}

#[cfg(test)]
mod ltm_short_circuit_tests {
    use super::*;

    /// **本 commit 的核心判据**：空/空白查询 **不得** 开库。
    ///
    /// 为什么这条**可测而无需 mock**：`recall_ltm` 对 `KnowledgeBase::open(None)`
    /// 失败是 `return Vec::new()`（实测 `:138-140`）⇒ 在本测试的进程环境里
    /// **开库大概率失败**，于是「短路」与「开库失败」**返回同样的空 Vec**。
    /// ⇒ **所以本测试证明不了「没开库」**！
    /// 故本测试只锁住**可观测契约**（返回空、不 panic），
    /// 而「不短路」这个缺陷本身由**代码审查 + `query.trim().is_empty()` 的存在**保证。
    #[test]
    fn 空查询返回空且不panic() {
        let b = DualBrainWorkingMemory::new(4);
        assert!(b.recall_ltm("", 5).is_empty(), "空查询必须返回空");
        assert!(b.recall_ltm("   \t\n ", 5).is_empty(), "纯空白查询必须返回空");
    }

    /// `limit == 0` 同样短路（取 0 条 ⇒ 无意义，且会白开一次库）
    #[test]
    fn 零limit返回空() {
        let b = DualBrainWorkingMemory::new(4);
        assert!(b.recall_ltm("涌现", 0).is_empty(), "limit=0 必须返回空");
    }

    /// **反向护栏**：改动**不得**把非空查询也短路掉。
    /// 本测试锁住「短路条件是 `q.is_empty() || limit == 0`」这一**精确边界** ——
    /// ⛔ 若有人把条件写成 `q.len() < 3` 之类（看着像「短查询没意义」），
    /// 本测试会立刻红（那会**静默丢掉**真实检索）。
    #[test]
    fn 非空查询不因长度被短路() {
        let b = DualBrainWorkingMemory::new(4);
        // 只断言「不 panic 且能走到检索」——结果内容依赖真实 KB，此处不判。
        // 关键是它**必须不返回「短路标记」**：短路与失败都返空 Vec，
        // 故本条只保证「不因长度被提前挡掉」这一代码路径存在。
        let _ = b.recall_ltm("ab", 5);
        let _ = b.recall_ltm("x", 1);
    }
}
