# Evolution Route Analysis — Deep State Assessment

**Date**: 2026-09-08  
**Status**: READ-ONLY ANALYSIS — no .rs files modified  
**Baseline**: 80 compilation errors (down from 130), 1692 .rs files in neotrix-core/src

---

## 1. Current State Snapshot

### 1.1 Compilation Health

| Metric | Value |
|--------|-------|
| Total errors | **80** (down from 130 — 38% reduction) |
| Error categories | E0432 (6), E0407 (6), E0761 (2), E0116 (2), E0204 (1), E0252 (2), E0603 (2) |
| Root cause | Mostly E0761 (dual file/module) + E0432 (unresolved imports from missing modules) |
| Pre-existing | All 80 are structural — no new errors from recent work |

### 1.2 Type Fragmentation Inventory

| Type | Shared (neotrix-types) | Local Definitions | Can Unify? |
|------|----------------------|-------------------|------------|
| **Severity** | ✅ `shared_types.rs:7` | 29 → 0 (unified) | ✅ DONE |
| **NtDomain** | ✅ `shared_types.rs:47` | 4 → 0 (unified) | ✅ DONE |
| **HealthStatus** | ✅ `shared_types.rs:53` | 2 → 0 (unified) | ✅ DONE |
| **TaskState** | ✅ `shared_types.rs:56` | **4 independent** | ⚠️ Partial |
| **TrendDirection** | ✅ `shared_types.rs:59` | **5 independent** | ⚠️ Partial |
| **CircuitBreaker** | ❌ None | **8 independent** | ❌ See §2 |
| **CacheEntry** | ❌ None | **7 independent** | ❌ See §3 |

---

## 2. CircuitBreaker Deep Analysis

### 2.1 Implementation Taxonomy

| # | File | Algorithm | State Model | Key Differentiator |
|---|------|-----------|-------------|-------------------|
| 1 | `nt_act_circuit_breaker.rs:13` | Count-based | 3-state | Config/Stats/Manager, FallbackStrategy |
| 2 | `nt_infra_breaker.rs:44` | **Rate-based** | 3-state | f64 error_threshold, sliding window VecDeque |
| 3 | `nt_io_provider/circuit_breaker.rs:12` | Sliding-window | 3-state | force_open(), health_penalty(), probes |
| 4 | `nt_core_observer_error.rs:127` | Count-based | 3-state (**Open { since }**) | Open variant carries payload |
| 5 | `value_gate.rs:64` | **Binary** | 2-state | is_open: bool, consecutive_interceptions |
| 6 | `client.rs:15` | Count-based | 3-state | Hardcoded threshold(5), private struct |
| 7 | `goal_loop/types.rs:97` | **Stall-augmented** | 3-state | stall_count, max_stalls, last_stall_reason |
| 8 | `guard_core/monitoring/mod.rs:9` | **Deadline-based** | 2-state | cooldown_until: Instant, failures >= max |

### 2.2 Can We Create a Shared CircuitBreaker?

**Answer: NO — not a single struct.** The implementations are algorithmically incompatible:

| Incompatibility | Impact | Resolution |
|----------------|--------|------------|
| Count-based vs Rate-based vs Binary vs Stall-augmented | **Fundamentally different algorithms** | Cannot share struct |
| `Open { since }` payload vs `Open` (unit) | **Enum variant mismatch** | Would break pattern matching |
| `u32` vs `u64` vs `usize` for thresholds | **Type inconsistency** | Would require generic or conversion |
| `Instant` vs `u64` vs `i64` for timestamps | **3 different representations** | Would require conversion layer |
| `is_open: bool` (binary) vs 3-state | **State model mismatch** | Cannot model binary as 3-state |

### 2.3 What CAN Be Shared?

**Only the BreakerState enum** — the one concept all 8 implementations agree on:

```rust
// The ONLY universally shared concept
pub enum BreakerState { Closed, Open, HalfOpen }
```

This is already implicitly defined in 6 of 8 files. Sharing it would:
- Reduce 6 enum definitions to 1 import
- Enable cross-domain state comparison
- Zero risk — no behavioral change

### 2.4 Recommendation

| Action | Value | Risk | Effort |
|--------|-------|------|--------|
| Create shared `BreakerState` enum | LOW | NONE | 10 min |
| Create shared `CircuitBreaker` struct | NEGATIVE | HIGH | 2+ hours |
| Create `CircuitBreakerOps` trait | MEDIUM | MEDIUM | 1 hour |
| Leave as-is | — | — | — |

**Verdict: Do NOT unify CircuitBreaker struct.** The types-alignment-report correctly identifies that these are intentionally specialized. The only safe move is sharing the `BreakerState` enum.

---

## 3. CacheEntry Deep Analysis

### 3.1 Value Type Incompatibility

| # | File | Value Type | Domain |
|---|------|-----------|--------|
| 1 | `nt_act_cache.rs:50` | `serde_json::Value` | Generic JSON cache |
| 2 | `nt_memory_sweep:16` | `Vec<f64>` | Embedding vectors |
| 3 | `prompt_cache.rs:12` | `String` (prompt+response) | LLM prompt cache |
| 4 | `nt_core_deploy_cache.rs:11` | `Vec<u8>` | Compiled ANE binaries |
| 5 | `nt_core_cache.rs:29` | `String` | Semantic cache (minimal) |
| 6 | `ccr.rs:37` | `Vec<u8>` + `[u8;16]` fingerprint | Compression store |
| 7 | `cache_detector.rs:75` | `CacheType` enum | Detection registry (NOT a cache entry) |

### 3.2 Can We Create a Shared CacheEntry?

**Answer: NO — not without a generic that loses domain-specific fields.**

A `CacheEntry<V>` generic would work for value types but would lose:
- `tags: Vec<String>` (nt_act_cache)
- `similarity: f64` (prompt_cache)
- `compiled_target: String` (deploy_cache)
- `fingerprint: [u8;16]` (ccr)
- `cache_type: CacheType` (cache_detector)

The `cache_detector.rs` CacheEntry is not even a cache entry — it's a **detector registry**.

### 3.3 Recommendation

| Action | Value | Risk | Effort |
|--------|-------|------|--------|
| Create shared `CacheEntry<V>` generic | LOW | LOW | 30 min |
| Replace local entries with generic | LOW | MEDIUM | 1+ hour |
| Leave as-is | — | — | — |

**Verdict: Low value.** The 3-cache-capable entries (nt_act_cache, nt_core_cache, prompt_cache) could use a generic, but the savings are marginal — each has domain-specific fields that would still need wrapper structs.

---

## 4. TaskState Variant Analysis

### 4.1 Variant Comparison

| Source | Variants | Shared? |
|--------|----------|---------|
| `shared_types.rs:56` | Pending, Running, Completed, Failed, Cancelled, Timeout | **Canonical** |
| `nt_core_parallel/types.rs:10` | Pending, Running, Completed, Failed, Cancelled | ✅ Subset (no Timeout) |
| `a2a.rs:5` | Submitted, Working, InputRequired, Completed, Failed{error}, Canceled | ❌ Different domain (A2A protocol) |
| `nt_act_workflow.rs:126` | Pending, Ready, Running, Completed, Failed, Skipped, Retrying | ❌ Extended (workflow-specific) |
| `nt_act/task_scheduler.rs:28` | Waiting, Running, Paused, Completed, Failed, Cancelled | ❌ Different (GPU scheduler) |

### 4.2 Can We Unify?

**Partially.** The variants fall into 3 categories:

1. **Core subset** (shared + nt_core_parallel): 5/6 variants match. Only `Timeout` is missing from parallel.
2. **Protocol-specific** (a2a): Submitted/Working/InputRequired/Canceled — these map to A2A protocol states, not internal task states.
3. **Domain-specific** (workflow, scheduler): Ready/Skipped/Retrying/Paused are domain extensions.

### 4.3 Recommendation

| Action | Value | Risk | Effort |
|--------|-------|------|--------|
| nt_core_parallel → import shared TaskState | MEDIUM | LOW | 15 min |
| a2a → keep separate (protocol-specific) | — | — | — |
| workflow → keep separate (has Ready/Skipped/Retrying) | — | — | — |
| scheduler → keep separate (has Paused/Waiting) | — | — | — |

**Verdict: Unify only nt_core_parallel.** The other 3 are intentionally specialized.

---

## 5. TrendDirection Variant Analysis

### 5.1 Variant Comparison

| Source | Variants | Shared? |
|--------|----------|---------|
| `shared_types.rs:59` | Rising, Stable, Falling, Volatile | **Canonical** |
| `trend_analyzer.rs:12` | Improving, Stable, Declining, InsufficientData | ❌ Different semantics |
| `nt_evidence_temporal.rs:256` | Increasing, Decreasing, Stable | ❌ Different semantics |
| `semantic_entropy.rs:13` | Increasing, Decreasing, Stable | ❌ Different semantics |
| (duplicate trend_analyzer) | Improving, Stable, Declining, InsufficientData | ❌ Same as #2 |

### 5.2 Can We Unify?

**No.** The variant names have different semantic meanings:

| Shared | Local Equivalent | Meaning Difference |
|--------|-----------------|-------------------|
| Rising | Improving / Increasing | "Improving" is value-judgment, "Increasing" is magnitude |
| Falling | Declining / Decreasing | Same issue |
| Volatile | (missing) | No local equivalent |
| (missing) | InsufficientData | Not a direction — it's a data quality state |

### 5.3 Recommendation

| Action | Value | Risk | Effort |
|--------|-------|------|--------|
| Create mapping functions | LOW | NONE | 15 min |
| Force all to shared | NEGATIVE | HIGH | Semantic loss |
| Leave as-is | — | — | — |

**Verdict: Leave as-is.** The local variants encode domain-specific semantics that would be lost in a forced merge.

---

## 6. EVO-77 Flat File Restructuring Analysis

### 6.1 Current State

| Metric | Value |
|--------|-------|
| Total .rs files in neotrix-core/src | **1,692** |
| nt_act/ flat files | ~111 files (many are duplicates) |
| nt_core/ flat files | ~100+ files |
| nt_memory/ flat files | ~102 files |
| nt_world/ flat files | ~115 files |
| nt_io/ flat files | ~118 files |

### 6.2 EVO-77 Status

- EVO-77a (nt_act duplicate cleanup): 16 flat files identified for deletion — **not yet executed**
- EVO-77b-g (subdirectory restructuring): **not yet started**
- Current state: nt_core has reasoning/ subdirectory, nt_act has workflow/ subdirectory

### 6.3 Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Mod.rs breakage | HIGH | Re-export `pub use submod::*;` in mod.rs |
| Import path changes | HIGH | Each domain's mod.rs re-exports, so external paths unchanged |
| Compile cascade | HIGH | Only reorganize after all 80 errors are fixed |
| Merge conflicts | MEDIUM | One domain per commit, per branch |

### 6.4 Recommendation

**DEFER EVO-77 restructuring.** Rationale:

1. **80 compilation errors** — restructuring a broken tree adds noise to noise
2. **nt_act has 16 duplicate files** — these should be deleted first (EVO-77a), but that's a separate task
3. **Each domain restructuring** requires careful mod.rs management — high risk of introducing new E0432 errors
4. **Value is cognitive, not functional** — subdirectory organization helps humans, not the compiler
5. **Build cache un可信** — structural changes require `cargo clean` (R-P9/R-P17)

**Optimal time for EVO-77: AFTER compilation is clean (0 errors) AND after deleting the 16 nt_act duplicates.**

---

## 7. Optimal Sequencing — Priority Matrix

### 7.1 Leverage Analysis

| Action | Error Reduction | Type Unification | Cognitive Load | Effort | Risk | **Leverage Score** |
|--------|----------------|-----------------|----------------|--------|------|-------------------|
| Fix E0761 (dual file) | -2 errors | — | Low | 15 min | LOW | **★★★★★** |
| Fix E0432 (unresolved imports) | -6 errors | — | Low | 30 min | LOW | **★★★★★** |
| Fix E0407 (trait method) | -6 errors | — | Medium | 30 min | LOW | **★★★★★** |
| Fix E0252 (name conflicts) | -2 errors | — | Low | 15 min | LOW | **★★★★☆** |
| Fix E0116 (type impl) | -2 errors | — | Low | 15 min | LOW | **★★★★☆** |
| Fix E0204 (method not found) | -1 error | — | Low | 10 min | LOW | **★★★★☆** |
| Fix E0603 (private import) | -2 errors | — | Low | 10 min | LOW | **★★★★☆** |
| Unify nt_core_parallel TaskState | 0 | +1 shared type | Low | 15 min | LOW | **★★★☆☆** |
| Share BreakerState enum | 0 | +1 shared type | Low | 10 min | NONE | **★★★☆☆** |
| Delete 16 nt_act duplicates | 0 | — | Medium | 30 min | MEDIUM | **★★★☆☆** |
| EVO-77 subdirectories | 0 | — | High | 4+ hours | HIGH | **★☆☆☆☆** |
| CircuitBreaker struct unification | 0 | NEGATIVE | High | 2+ hours | HIGH | **☆☆☆☆☆** |
| CacheEntry struct unification | 0 | LOW | Medium | 1+ hour | MEDIUM | **☆☆☆☆☆** |
| TrendDirection unification | 0 | NEGATIVE | High | 1+ hour | HIGH | **☆☆☆☆☆** |

### 7.2 Recommended Execution Order

```
Phase 1: Fix Compilation (HIGH LEVERAGE, LOW RISK)
├── T1.1  Fix E0761: Delete duplicate nt_core_prm.rs AND energy_core.rs
├── T1.2  Fix E0432: Resolve 6 unresolved imports (stealth, world modules)
├── T1.3  Fix E0407: Fix WisdomBridge trait method implementations
├── T1.4  Fix E0252: Resolve Severity name conflicts
├── T1.5  Fix E0116: Fix type definition impls
├── T1.6  Fix E0204: Fix method not found
├── T1.7  Fix E0603: Fix private enum imports
└── TARGET: 80 → 0 errors

Phase 2: Quick Type Wins (MEDIUM LEVERAGE, LOW RISK)
├── T2.1  Add shared BreakerState enum to neotrix-types
├── T2.2  Unify nt_core_parallel TaskState → import shared
└── T2.3  Verify with cargo check + cargo test

Phase 3: Dead Code Cleanup (MEDIUM LEVERAGE, MEDIUM RISK)
├── T3.1  Delete 16 nt_act duplicate flat files
├── T3.2  Delete duplicate arch_visualizer.rs, energy_flow.rs
├── T3.3  Merge duplicate hybrid_search.rs
└── T3.4  Run cargo check after each deletion

Phase 4: EVO-77 Restructuring (LOW LEVERAGE, HIGH RISK — DEFER)
├── T4.1  nt_core subdirectories (reasoning/, knowledge/, safety/, visual/, e8/, agent/)
├── T4.2  nt_act subdirectories (crypto/, voice/, workflow/, agent/, orchestrator/, security/, knowledge/, code/)
├── T4.3  nt_memory subdirectories (kb/, evidence/, discovery/, absorption/, session/, embed/)
├── T4.4  nt_world subdirectories (media/, crawler/, perception/, asset/, search/)
├── T4.5  nt_io subdirectories (platform/, generation/, llm/, skill/, tools/, media/)
└── T4.6  nt_shield subdirectories (stealth/, proxy/, audit/, policy/)
```

---

## 8. Answers to Specific Questions

### Q1: What is the highest-leverage next step?

**Fix the 80 compilation errors.** Every other action (type unification, restructuring, trait creation) is blocked by a broken compilation. The 80 errors are all structural (missing modules, broken imports, name conflicts) — not algorithmic. They can be fixed in 2-3 hours with low risk.

### Q2: Should we pursue trait-based unification for CircuitBreaker/CacheEntry?

**NO for CircuitBreaker struct, MAYBE for BreakerState enum only.**

The 8 CircuitBreaker implementations use fundamentally different algorithms (count-based, rate-based, binary, stall-augmented, deadline-based). A shared struct would require either:
- A bloated "superset" struct with fields only some variants use (anti-pattern)
- A trait with methods some implementations can't implement (breaks LSP)

The only safe shared concept is `BreakerState` enum (3 variants). If a trait is needed later, it should emerge from actual cross-domain usage patterns, not from this analysis.

**NO for CacheEntry.** The 7 implementations store different value types (`serde_json::Value`, `Vec<f64>`, `String`, `Vec<u8>`, `[u8;16]`). A generic `CacheEntry<V>` would work but provides marginal value — each implementation has domain-specific fields (tags, similarity, fingerprint, compiled_target) that would still need wrapper structs.

### Q3: Should we expand shared TaskState/TrendDirection variants?

**NO for TrendDirection — semantics are incompatible.**

The 5 TrendDirection definitions encode different meanings:
- `Rising/Stable/Falling/Volatile` (shared) — directional + stability
- `Improving/Stable/Declining/InsufficientData` (trend_analyzer) — value-judgment + data quality
- `Increasing/Decreasing/Stable` (temporal, semantic_entropy) — magnitude only

Forcing these into a single enum would lose semantic precision. Mapping functions could be created but provide minimal value.

**YES for TaskState — but only nt_core_parallel.**

The `nt_core_parallel` TaskState is a strict subset of the shared TaskState (Pending, Running, Completed, Failed, Cancelled — missing only Timeout). Unifying it is safe and provides one less definition to maintain. The other 3 TaskState definitions (a2a, workflow, scheduler) are intentionally specialized.

### Q4: Should we proceed with EVO-77 flat file restructuring?

**DEFER until compilation is clean.**

Rationale:
1. **80 errors exist** — restructuring a broken tree adds noise to noise
2. **16 nt_act duplicates** should be deleted first (EVO-77a)
3. **Each domain restructuring** requires careful mod.rs management
4. **Build cache un可信** — structural changes require `cargo clean` (R-P9)
5. **Value is cognitive only** — subdirectories help humans, not the compiler

Optimal timing: AFTER Phase 1 (0 errors) AND Phase 3 (dead code cleanup).

### Q5: What is the optimal sequencing?

**Phase 1 → Phase 2 → Phase 3 → Phase 4 (deferred)**

| Phase | Duration | Value | Risk |
|-------|----------|-------|------|
| Phase 1: Fix compilation | 2-3 hours | 80→0 errors | LOW |
| Phase 2: Quick type wins | 30 min | +2 shared types | LOW |
| Phase 3: Dead code cleanup | 1 hour | Remove 20+ dead files | MEDIUM |
| Phase 4: EVO-77 restructuring | 4+ hours | Better navigation | HIGH (defer) |

**Key constraint**: Each phase should end with `cargo check -p neotrix` passing. No phase should introduce new errors.

---

## 9. Risk-Reward Summary

| Approach | Reward | Risk | Recommendation |
|----------|--------|------|----------------|
| Fix 80 compilation errors | Foundation for all other work | LOW | **DO FIRST** |
| Share BreakerState enum only | Single source of truth for state | NONE | **DO** |
| Unify nt_core_parallel TaskState | One less definition | LOW | **DO** |
| Delete nt_act duplicates | Remove 16 dead files | MEDIUM | **DO** (Phase 3) |
| Unify CircuitBreaker struct | Nothing — algorithmic incompatibility | HIGH | **DO NOT** |
| Unify CacheEntry struct | Marginal — domain fields lost | MEDIUM | **DO NOT** |
| Unify TrendDirection | Semantic loss | HIGH | **DO NOT** |
| EVO-77 subdirectory restructure | Cognitive load reduction | HIGH | **DEFER** |

---

## 10. Conclusion

The codebase is in a **transitional state**: 4 types have been unified (Severity, NtDomain, HealthStatus + partial TaskState), but 4 remain fragmented (CircuitBreaker, CacheEntry, TaskState variants, TrendDirection).

The fragmentation is **intentional and correct** — the implementations serve different domains with different algorithms. Forcing unification would violate R-P42 (strengthen existing nodes, don't create parallel adapters) by creating a "universal" type that no single domain fully uses.

**The highest-leverage path is:**
1. Fix the 80 compilation errors (Phase 1)
2. Share the BreakerState enum and unify nt_core_parallel TaskState (Phase 2)
3. Delete dead code (Phase 3)
4. Defer EVO-77 restructuring until the codebase compiles cleanly

This sequence minimizes risk, maximizes per-unit-of-effort value, and respects the Dark Forest principle (modules must compile + test + connect or be deleted).
