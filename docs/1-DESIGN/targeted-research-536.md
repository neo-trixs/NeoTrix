# Targeted Research #536: Hardcoded/Fake Data Pain Points

**Date**: 2026-09-13
**Scope**: 4 files in `neotrix-core/src/` returning fabricated data as real results

## Summary

Found and fixed 4 hardcoded/fake-data pain points across NT-SHIELD, NT-CORE bank, and NT-META QA layers. Each fix replaces fabricated data with either explicit errors (stub) or honest low/zero scores that signal "not yet implemented."

---

## Fix 1: `nt_shield_pentest_agent.rs` (L3-Embodiment / NT-SHIELD)

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_pentest_agent.rs`

**Problem**: `detect_vulnerabilities()` returned a hardcoded `_VulnerabilityReport` with fabricated severity (`"critical"`), confidence (`0.87`), and an invented exploitation chain. `generate_exploitation_chain()` returned a canned 2-phase plan. These fake results would be indistinguishable from real scan output.

**Fix**:
- `detect_vulnerabilities()` → returns `Result<Vec<...>, String>` with explicit `Err("PentestGPT adapter not wired...")`
- `generate_exploitation_chain()` → returns `Result<..., String>` with explicit `Err("E8 exploitation-chain planner not wired...")`
- Updated caller in `nt_shield_impl/mod.rs` to handle `Result` and log a warning instead of silently trusting fabricated data

**Doc comments added**: What real implementation needs (E8 hexagram engine, PentestGPT LLM client, sandbox integration).

---

## Fix 2: `nt_shield_local_inference.rs` (L3-Embodiment / NT-SHIELD)

**File**: `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_impl/nt_shield_local_inference.rs`

**Problem**: `_load_or_create_profile()` hardcoded optimistic defaults: `expected_throughput_tok_s: 9.06`, `memory_requirements_gb: 8.0`, `e8_reasoning_score: 0.85`. These pretend the system has been benchmarked when it hasn't. Any downstream consumer would trust these numbers.

**Fix**:
- `expected_throughput_tok_s: 0.0` (UNCALIBRATED)
- `memory_requirements_gb: 0.0` (UNCALIBRATED)
- `e8_reasoning_score: 0.0` (UNCALIBRATED)

All three fields now emit zero with doc comments indicating `run optimize_model()` to populate real values.

---

## Fix 3: `bank/search.rs` (L1-Action / NT-CORE)

**File**: `neotrix-core/src/core/nt_core_bank/bank/search.rs`

**Problem (3a)**: `vector_search_by_text()` computed the **norm** (magnitude) of each memory's embedding vector and used that as the search score. It never embedded the query or computed query-document similarity. This means search results were ordered by embedding size, not relevance — returning semantically irrelevant results.

**Fix (3a)**:
- Now embeds the query text via `TextEmbedder::embed()`
- Computes real cosine similarity via `nt_core_math::cosine_similarity_f64(query_emb, mem_emb)`
- Returns empty when embedding is unavailable (no fake scores)

**Problem (3b)**: `index_memory()` had commented-out cosine similarity computation, leaving `strength = 0.0` permanently. Hypergraph edges were never created, making the hypergraph traversal dead code.

**Fix (3b)**:
- Uncommented and replaced with `nt_core_math::cosine_similarity_f64()` call
- Hypergraph edges now form with real similarity scores

---

## Fix 4: `layered_qa.rs` (L6-Meta / NT-META)

**File**: `neotrix-core/src/l6_meta/coordination/layered_qa.rs`

**Problem**: `execute_check()` default branch (the `else` clause) passed any non-empty output:
```rust
let has_output = !output.as_object().map_or(true, |m| m.is_empty());
```
This meant ALL check items that didn't match "技术"/"合规"/"视觉" in their name silently passed. The QA gate was a rubber stamp — any non-empty output passed quality checks.

**Fix**:
- Default branch now returns `(false, IssueSeverity::Warning, "UNIMPLEMENTED check type ...")`
- QA gate correctly rejects output when check logic isn't implemented
- Test updated to verify the gate blocks unimplemented checks

**Impact**: Tests that previously passed with fake QA results will now correctly fail until real check implementations are added.

---

## Pattern Summary

| File | Fake Data | Fix Strategy |
|------|-----------|-------------|
| pentest_agent.rs | Fabricated vulnerability reports | `Err("not wired")` |
| local_inference.rs | Optimistic benchmark numbers | Zero (uncalibrated) |
| search.rs (vector) | Embedding norm as relevance score | Real cosine similarity |
| search.rs (hypergraph) | strength=0.0 dead code | Uncommented real computation |
| layered_qa.rs | Non-empty = pass | Fail unimplemented checks |

## Remaining Work

These are stubs returning honest errors. The real implementations needed:

1. **PentestGPT adapter**: Wire LLM client, E8 attack-path reasoning, sandbox payload execution
2. **Inference profiling**: Run `optimize_model()` against real hardware benchmarks
3. **QA check implementations**: Add `RequiredField`, `DurationMatch`, `ResolutionMatch`, `EntityConsistency` logic
4. **KB embedding**: Ensure `TextEmbedder` is configured and available at call sites
