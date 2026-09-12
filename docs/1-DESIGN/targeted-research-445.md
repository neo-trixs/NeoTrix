# Targeted Research 445: Internal Pain Points

Generated from codebase audit of 3 critical stubs.

---

## Pain Point 1: KB Semantic Search Returns Nothing (P0)

**File:** `neotrix-core/src/core/nt_core_bank/bank/search.rs:270`

**Problem:** `retrieve_relevant_by_embedding` hardcodes cosine similarity to `0.0_f64`, then filters `score > 0.0`. Every memory with an embedding is discarded. Semantic search is completely broken — it never returns results.

```rust
// Line 270 — the bug
.map(|_emb| (0.0_f64, m)) // TODO: compute cosine_similarity
```

Line 272 then kills all results:
```rust
.filter(|(score, _)| *score > 0.0)  // 0.0 > 0.0 is false → empty vec
```

**Proposed Fix:** Compute actual cosine similarity. Two options:

1. **Simple (no deps):** Manual dot / norm computation on `&[f64]`.
2. **Optimal (SIMD):** Use `innr` crate (`innr::dense_f64::cosine_f64`) — SIMD-dispatched (NEON on Apple Silicon, AVX2/AVX-512 on x86), ~8.5 Gelem/s.

```rust
// Option 1: zero-dependency inline
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

// Line 270 replacement:
.map(|emb| {
    let sim = cosine_similarity(task_embedding, emb);
    ((sim + 1.0) / 2.0, m)  // map [-1,1] → [0,1] so filter >0 works
})
```

Also remove the `.filter(|(score, _)| *score > 0.0)` — or change threshold to something meaningful like `> 0.3`.

**Priority:** P0 — This is a silent data loss bug. All embedding-based retrieval is dead.

---

## Pain Point 2: SelfModel.value_function Is a Placeholder (P1)

**File:** `neotrix-core/src/core/nt_core_self_model.rs:94-108`

**Problem:** The value function, which SEAL uses to rank candidate behavior mutations, ignores the action content entirely. It returns `(action.len() / 64.0).clamp(0,1)` — longer strings score higher regardless of semantic content.

```rust
// Line 99-107 — placeholder
pub fn value_function(&self, action: &str) -> f64 {
    let total_weight: f64 = self.value_weights.iter().map(|w| w.weight).sum();
    let _ = total_weight; // unused!
    let signal = (action.len() as f64).clamp(0.0, 64.0) / 64.0;
    signal.clamp(0.0, 1.0)
}
```

**Proposed Fix:** Project action onto value dimensions using keyword scoring (no LLM needed at this level):

```rust
pub fn value_function(&self, action: &str) -> f64 {
    if action.is_empty() { return 0.0; }
    let lower = action.to_lowercase();
    let mut score = 0.0;
    let mut total_weight = 0.0;
    for vw in &self.value_weights {
        let dim_score = match vw.dimension.as_str() {
            "coherence" => {
                let signals = ["because", "therefore", "consist", "align", "逻辑", "一致"];
                signals.iter().filter(|s| lower.contains(*s)).count() as f64 / signals.len() as f64
            }
            "safety" => {
                let risky = ["delete", "rm -rf", "drop table", "unsafe", "删除", "破坏"];
                let safe = ["backup", "validate", "check", "安全", "验证"];
                let risk = risky.iter().filter(|s| lower.contains(*s)).count() as f64;
                let saf = safe.iter().filter(|s| lower.contains(*s)).count() as f64;
                (1.0 - risk * 0.3 + saf * 0.2).clamp(0.0, 1.0)
            }
            "growth" => {
                let signals = ["learn", "adapt", "evolve", "new", "学习", "进化", "新增"];
                signals.iter().filter(|s| lower.contains(*s)).count() as f64 / signals.len() as f64
            }
            _ => 0.5,
        };
        score += vw.weight * dim_score;
        total_weight += vw.weight;
    }
    if total_weight > 0.0 { score / total_weight } else { 0.5 }
}
```

**Priority:** P1 — SEAL's mutation ranking is garbage-in-garbage-out. All evolution decisions based on this are random.

---

## Pain Point 3: ConceptEmergenceStage Is a No-Op (P2)

**File:** `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs:50-53`

**Problem:** The stage that should discover emergent concepts from KB node clusters does nothing:

```rust
fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
    // 从brain的KB中获取节点嵌入 (ReasoningBrain 无 KB 字段, 跳过)
    // TODO: 通过 MemoryOrchestrator 获取嵌入
    Ok(StageDecision::Continue)
}
```

**Proposed Fix:** Wire to KB embeddings and do simple centroid-based clustering:

```rust
fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
    // 1. Get recent memories with embeddings from KB
    let memories = brain.kb.retrieve_recent_with_embeddings(100);
    if memories.len() < 5 { return Ok(StageDecision::Continue); }

    // 2. Simple online K-means (k=3) to find clusters
    let embeddings: Vec<&[f64]> = memories.iter()
        .filter_map(|m| m.embedding.as_deref())
        .collect();

    // 3. If a cluster has >3 members with high intra-cluster similarity,
    //    emit a "concept" as a new KB node
    let clusters = simple_kmeans(&embeddings, 3, 10);
    for cluster in &clusters {
        if cluster.members.len() >= 3 && cluster.mean_similarity > 0.7 {
            brain.kb.store_concept(Node {
                label: format!("emergent_{}", cluster.centroid_hash),
                members: cluster.members.iter().map(|m| m.id.clone()).collect(),
                source: "concept_emergence".into(),
            });
        }
    }
    Ok(StageDecision::Continue)
}
```

**Priority:** P2 — Nice-to-have for self-evolution, but SEAL works without it (other stages compensate).

---

## Summary

| # | Pain Point | File:Line | Priority | Fix Complexity |
|---|-----------|-----------|----------|----------------|
| 1 | KB search cosine sim = 0.0 | search.rs:270 | **P0** | Low (5 lines) |
| 2 | SelfModel value_function placeholder | nt_core_self_model.rs:94 | **P1** | Medium (30 lines) |
| 3 | ConceptEmergenceStage no-op | pipeline.rs:50 | **P2** | Medium (40 lines) |

P0 should be fixed immediately — it's a silent correctness bug affecting all retrieval.
