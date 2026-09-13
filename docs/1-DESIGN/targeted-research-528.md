# Targeted Research #528 — Internal Pain Points (Iteration 5)

**Date:** 2026-09-13
**Scope:** neotrix-core/src/ — hardcoded values, stub functions, dead code paths
**Method:** grep TODO/FIXME/HACK/XXX + hardcoded returns + dead branches

---

## Pain Point 1: `resource_budget.rs:287-296` — Hardcoded model pricing

**File:** `l1_action/nt_act/resource_budget.rs:287-296`
**Severity:** P1
**What's wrong:** `estimate_cost()` uses hardcoded cost-per-1k-tokens with no dynamic lookup. Missing models (e.g. `claude-3.5-sonnet`, `gemini-2.0-flash`) silently fall back to `$0.001/1k` — wildly inaccurate. The `gpt-4` price is stale (now $0.03 → $0.06 input / $0.12 output since 2024).
**Impact:** Cost budget enforcement gives wrong numbers; downstream `BudgetStats` and budget guards are unreliable.

**Fix sketch:**
```rust
pub fn estimate_cost(&self, token_count: u64, model: &str, is_output: bool) -> f64 {
    let cost_per_1k = crate::core::nt_core_resource_pool::MODEL_PRICING
        .get(model)
        .map(|p| if is_output { p.output } else { p.input })
        .unwrap_or_else(|| {
            log::warn!("[resource_budget] unknown model '{model}', using fallback $0.001/1k");
            0.001
        });
    (token_count as f64 / 1000.0) * cost_per_1k
}
```
The `MODEL_PRICING` map lives in `nt_core_resource_pool` (already has model registry infra). Add an `is_output` param to separate input/output pricing — current single-value approach can't distinguish.

---

## Pain Point 2: `dynamic_memory_bank.rs:250-267` — Fake semantic similarity

**File:** `l2_perception/nt_world/dynamic_memory_bank.rs:250-267`
**Severity:** P1
**What's wrong:** `calculate_semantic_similarity()` is a pure keyword-overlap stub. No embedding, no DINOv2, no cosine similarity. For a "dynamic memory bank" that tracks entities across shots, this means entity re-identification is broken — "a red-haired girl" and "the crimson-haired lass" score 0.0.
**Impact:** Entity tracking in multi-scene content fails silently; deduplication and continuity checks are meaningless.

**Fix sketch:**
```rust
fn calculate_semantic_similarity(&self, query: &str, description: &str) -> f32 {
    // Delegate to existing KB embedding infra (BM25 + cosine on stored embeddings)
    if let Some(kb) = &self.kb {
        let q_emb = kb.embed(query).unwrap_or_default();
        let d_emb = kb.embed(description).unwrap_or_default();
        return cosine_similarity(&q_emb, &d_emb);
    }
    // Fallback: BM25 lexical score only (not keyword-overlap)
    self.bm25_score(query, description)
}
```
`KnowledgeBase` already has `embed()` and BM25 index — wire through instead of reinventing with substring matching.

---

## Pain Point 3: `kb_cmds.rs:157-159` — `/kb consistency` always returns empty

**File:** `cli/commands/kb_cmds.rs:157-159`
**Severity:** P2
**What's wrong:** `cmd_consistency()` opens the KB, then returns `CommandOutput::ok("")` — the actual consistency check is commented out with `TODO: setting_consistency module not found; stub`. The CLI command exists, users call it, it always reports success with zero output.
**Impact:** Users trust the "all clear" result; data inconsistencies in KB go undetected. This is a silent false-negative in a safety-critical feature.

**Fix sketch:**
```rust
fn cmd_consistency(_args: &[String]) -> CommandOutput {
    let kb = match open_kb() {
        Some(kb) => kb,
        None => return CommandOutput::err("无法打开知识库 ~/.neotrix/knowledge.db"),
    };
    // Inline basic consistency checks (or re-add the module)
    let mut issues = Vec::new();
    // Check: orphan edges (source/target node missing)
    let orphan_count = kb.query_orphan_edges().unwrap_or(0);
    if orphan_count > 0 {
        issues.push(format!("{} orphan edges (missing source/target node)", orphan_count));
    }
    // Check: duplicate nodes
    let dup_count = kb.query_duplicate_nodes().unwrap_or(0);
    if dup_count > 0 {
        issues.push(format!("{} duplicate nodes", dup_count));
    }
    if issues.is_empty() {
        CommandOutput::ok("✅ KB consistency check passed")
    } else {
        CommandOutput::ok(&format!("⚠️ Found {} issues:\n{}", issues.len(), issues.join("\n")))
    }
}
```
Wire through `KnowledgeBase` methods that already exist (orphan detection, dedup scan). The empty-string ok() is worse than returning an error.

---

## Summary

| # | Location | Severity | Issue | Category |
|---|----------|----------|-------|----------|
| 1 | `resource_budget.rs:287` | P1 | Hardcoded stale model pricing, no input/output split | Hardcoded values |
| 2 | `dynamic_memory_bank.rs:250` | P1 | Fake semantic similarity (keyword overlap only) | Stub implementation |
| 3 | `kb_cmds.rs:157` | P2 | `/kb consistency` always returns empty success | Dead code path |

**Total new pain points:** 3
**P1:** 2 | **P2:** 1
