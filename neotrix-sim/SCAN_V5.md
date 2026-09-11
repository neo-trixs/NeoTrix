# neotrix-sim Scan Report (V5)

**Date**: 2026-09-11
**Status**: GREEN (compiles, 325/325 tests pass)

---

## 1. Build & Test Summary

| Metric | Result |
|--------|--------|
| `cargo check -p neotrix-sim` | 0 errors, 0 warnings |
| `cargo test -p neotrix-sim --lib` | 325 passed, 0 failed, 0 ignored |
| Total `.rs` files | 65 |
| Total lines | 16,814 |
| Agent module files | 16 |

---

## 2. File Size Audit (>500 lines = SPLIT CANDIDATE)

| File | Lines | Status | Split Recommendation |
|------|-------|--------|---------------------|
| `world_sim.rs` | 1,550 | CRITICAL | Split into 4 submodules |
| `agents/graph_memory.rs` | 1,207 | HIGH | Split into 2 submodules |
| `consciousness/emergence_detector.rs` | 655 | MEDIUM | Split tests to separate file |
| `feel/mod.rs` | 552 | MEDIUM | Split into 2 submodules |
| `agents/memory_stream.rs` | 515 | LOW | Split tests to separate file |
| `agents/planning.rs` | 483 | LOW | Below threshold |
| `safety/safety_monitor.rs` | 469 | LOW | Below threshold |
| `agents/action_awareness.rs` | 462 | LOW | Below threshold |
| `agents/pheromone.rs` | 447 | LOW | Below threshold |
| `agents/event_reactive.rs` | 442 | LOW | Below threshold |
| `foundation/math_bridge.rs` | 413 | LOW | Below threshold |

---

## 3. Critical Issues

### 3.1. `cosine_sim` Duplicated 4 Times

Identical implementations exist in 4 files (varying only in signature: `&[f32; 16]` vs `&[f32]`):

| Location | Signature | Lines |
|----------|-----------|-------|
| `foundation/math_bridge.rs:318` | `pub fn cosine_sim(a: &[f32], b: &[f32]) -> f32` | Canonical |
| `agents/graph_memory.rs:943` | `fn cosine_sim(a: &[f32; 16], b: &[f32; 16]) -> f32` | Duplicate |
| `agents/memory_stream.rs:289` | `fn cosine_sim(a: &[f32; 16], b: &[f32; 16]) -> f32` | Duplicate |
| `consciousness/dual_representation.rs:129` | `fn cosine_sim(a: &[f32], b: &[f32]) -> f32` | Duplicate |

**Fix**: Delete the 3 private duplicates, make `math_bridge::cosine_sim` public, and import it everywhere. The fixed-size `[f32; 16]` variants can call the slice version via `cosine_sim(a.as_slice(), b.as_slice())`.

### 3.2. `println!` Debug Logging in Production Code

Two `println!` calls in `world_sim.rs` pollute stdout:

- **Line 1279**: `println!("[Tick {}] GlobalCoherence: ...")`
- **Line 1373**: `println!("[Evolution Gen {}] ...")`

**Fix**: Replace with `tracing::info!` or `log::info!` to integrate with the NeoTrix structured logging pipeline.

---

## 4. Structural Split Recommendations

### 4.1. `world_sim.rs` (1,550 lines) --> 4 submodules

This file is a monolith containing: config, state, tick loop, decision layers, observation building, action execution, evolution, consciousness metrics, event collection, and snapshot.

**Proposed split**:

```
world_sim/
  mod.rs            (~100 lines)  -- WorldSim struct + new() + tick() orchestration
  config.rs         (~100 lines)  -- WorldSimConfig + EvolutionRecord + WorldSnapshot
  observation.rs    (~100 lines)  -- build_observation()
  decision.rs       (~400 lines)  -- decide_action + layer_survival/goals/social/stigmergy/personality/default
  execution.rs      (~200 lines)  -- execute_action (pheromone deposit, memory recording, etc.)
  evolution.rs      (~200 lines)  -- evolution_cycle()
  consciousness.rs  (~50 lines)   -- compute_consciousness_metrics()
  events.rs         (~50 lines)   -- collect_world_events()
```

### 4.2. `agents/graph_memory.rs` (1,207 lines) --> 2 submodules

Contains two distinct systems: legacy `GraphMemory` (lines 1-276) and SYNAPSE-inspired `UnifiedMemoryGraph` (lines 278-937), plus tests (lines 957-1207).

**Proposed split**:

```
agents/graph_memory/
  mod.rs              (~30 lines)   -- re-exports + cosine_sim
  legacy.rs           (~275 lines)  -- GraphMemory, NodeKind, EdgeKind, MemNode, Edge
  unified.rs          (~660 lines)  -- UnifiedMemoryGraph, EpisodeNode, SemanticNode, configs
  tests.rs            (~250 lines)  -- all tests
```

### 4.3. `consciousness/emergence_detector.rs` (655 lines)

Implementation: 358 lines. Tests: 297 lines (45% of file is tests).

**Fix**: Extract `#[cfg(test)] mod tests` to `emergence_detector/tests.rs`.

### 4.4. `feel/mod.rs` (552 lines) --> 2 submodules

Contains: EmotionType (enum + methods), EmotionEngine, PadVector, SystemEvent, ConflictRule, helper types, tests.

**Proposed split**:

```
feel/
  mod.rs           (~20 lines)   -- re-exports
  emotion_type.rs  (~190 lines)  -- EmotionType enum, GWT/E8/CT modulations, iter
  engine.rs        (~340 lines)  -- EmotionEngine, PadVector, SystemEvent, ConflictRule, tests
```

### 4.5. `agents/memory_stream.rs` (515 lines)

Implementation: 279 lines. Tests: 236 lines (46% of file is tests).

**Fix**: Extract tests to `memory_stream/tests.rs`.

---

## 5. Redundant `pub use` Re-exports

`agents/mod.rs` and `consciousness/mod.rs` both have 7 `pub use` re-exports that flatten module internals into the parent namespace. These are not inherently wrong but can cause name collisions as the crate grows. No action needed now, but worth noting for future module expansion.

---

## 6. Recommendations (Priority Order)

| # | Action | Impact | Effort |
|---|--------|--------|--------|
| 1 | Deduplicate `cosine_sim` into `math_bridge` | Eliminates 3 identical functions | Low |
| 2 | Replace `println!` with structured logging | Production-ready output | Low |
| 3 | Split `world_sim.rs` into submodules | Maintainability, compile times | Medium |
| 4 | Split `graph_memory.rs` into legacy/unified | Two distinct systems in one file | Medium |
| 5 | Extract tests from `emergence_detector.rs` | Cleaner source files | Low |
| 6 | Split `feel/mod.rs` into type + engine | Separation of concerns | Low |
| 7 | Extract tests from `memory_stream.rs` | Cleaner source files | Low |

---

## 7. Healthy Patterns (No Action Needed)

- All 65 source files compile with zero warnings
- 325 tests pass with zero flaky failures
- Module structure follows consistent `mod.rs` re-export pattern
- Agent subsystem initialization uses `or_insert_with` (lazy, correct)
- Memory management: eviction, pruning, consolidation all present
- Safety guardrails: capability tracker, safety monitor, audit trail, evolution constraints all integrated
- Multi-tier tick schedule (Reflex/Fast/Medium/Slow/Background) is clean and well-documented

---

*Generated by neotrix-sim SCAN_V5*
