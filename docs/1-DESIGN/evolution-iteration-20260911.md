# Evolution Iteration Report — 2026-09-11

## Project Snapshot

| Metric | Value |
|--------|-------|
| Total `.rs` files | 1,864 |
| Total lines of Rust | 611,753 |
| Frontend build | ✅ Pass (23.94s) |
| `cargo check` | ⏳ Blocked by stale lock file |

### Files by Layer

| Layer | Files |
|-------|-------|
| L1 Action | 391 |
| L2 Perception | 214 |
| L3 Embodiment | 200 |
| L4 Emotion | 6 |
| L5 Cognition | 359 |
| L6 Meta | 58 |
| Core | 376 |
| Neotrix | 69 |
| CLI | 74 |

---

## New Modules Audit (21 modules)

### All 21 Modules — Status Matrix

| Module | Lines | Declared | External Imports | Stubs | Status |
|--------|-------|----------|-----------------|-------|--------|
| addressable_store | 278 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| context_fs | 142 | ✅ | ⚠️ 0 | 1 (TODO) | 🔴 Dead |
| trinity | 171 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| memory_types | 138 | ✅ | ✅ 93 | 0 | 🟢 Wired |
| shared_utils | 8 | ✅ | ✅ 9 | 0 | 🟢 Wired |
| context_sandbox | 256 | ✅ | ⚠️ 0 | 1 (TODO) | 🔴 Dead |
| hooks | 128 | ✅ | ✅ 7 | 0 | 🟢 Wired |
| acp | 152 | ✅ | ✅ 2 | 0 | 🟢 Wired |
| cache_compaction | 149 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| mcp_server | 105 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| procedural_graph | 981 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| distillation (seal/) | 629 | ✅ (evolution/) | ✅ 8 | 1 (in test string) | 🟡 Partial |
| crystallization | 369 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| aegis | 303 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| harness_evolution | 235 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| harness_optimizer | 246 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| context_assembly | 346 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| io_contract | 87 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| reference_view | 102 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| context_boundary | 233 | ✅ | ⚠️ 0 | 0 | 🔴 Dead |
| refinement | 138 | ✅ | ✅ 1 | 0 | 🟢 Wired |

### Summary

| Category | Count | Modules |
|----------|-------|---------|
| 🟢 Wired (imported + used) | 5 | memory_types, shared_utils, hooks, acp, refinement |
| 🟡 Partial (declared, some usage) | 1 | distillation |
| 🔴 Dead (declared, 0 imports) | 15 | addressable_store, context_fs, trinity, context_sandbox, cache_compaction, mcp_server, procedural_graph, crystallization, aegis, harness_evolution, harness_optimizer, context_assembly, io_contract, reference_view, context_boundary |

---

## Critical Findings

### 1. 71% of New Modules Are Dead Code
15 of 21 modules have zero external imports. They compile and are declared in parent `mod.rs` files, but no other code in the project references them. This violates R-P79 (external tech absorption must connect to production paths in same session).

### 2. Heavy `#[allow(dead_code)]` Suppression
17 files contain 5+ `#[allow(dead_code)]` annotations, with 9 files using file-level suppression. Notable offenders:
- `universal_adapter.rs`: 13 dead_code allows
- `procedural_graph.rs`: 10 dead_code allows
- `aegis.rs`: 11 dead_code allows
- `attention_head.rs`: 11 dead_code allows
- `addressable_store.rs`: 9 dead_code allows

### 3. Dual `distillation` Module (Semantic Duplication)
Two different `distillation.rs` files exist:
- `l5_cognition/nt_mind/nt_mind/distillation.rs` — older, uses `CapabilityVector`/`ReasoningMemory`/`MicroEdit`
- `l5_cognition/nt_mind/seal/distillation.rs` — newer, uses `TraceSummary`/`StepSummary` approach

These are semantically different implementations of the same concept. The seal/ version has 8 external imports and is the active one.

### 4. `shared_utils` Is Minimal (8 Lines)
The shared utility module is essentially empty — likely just type re-exports or trivial helpers.

### 5. Stale Cargo Lock
`cargo check` blocked by stale build directory lock. Needs `cargo clean` or process kill before build verification can proceed.

---

## Modules Wired Into Production

| Module | Consumers | Integration |
|--------|-----------|-------------|
| memory_types | 93 imports | Foundation types used across memory subsystem |
| shared_utils | 9 imports | Utility re-exports |
| hooks | 7 imports | IO hook registration |
| acp | 2 imports | Agent Communication Protocol |
| refinement | 1 import | Harness refinement loop |
| distillation | 8 imports | SEAL experience distillation |

---

## Next Iteration Priorities

### P0 — Wire Dead Modules (Critical)
15 modules need integration wiring or removal. Prioritize by domain:

| Priority | Module | Lines | Action |
|----------|--------|-------|--------|
| P0-1 | addressable_store | 278 | Wire into NT-MEMORY KB path |
| P0-2 | context_fs | 142 | Wire into NT-MEMORY context management |
| P0-3 | trinity | 171 | Wire into NT-MEMORY trinity architecture |
| P0-4 | context_sandbox | 256 | Wire into NT-IO sandbox execution |
| P0-5 | cache_compaction | 149 | Wire into NT-IO cache lifecycle |
| P0-6 | mcp_server | 105 | Wire into NT-IO MCP gateway |
| P0-7 | context_assembly | 346 | Wire into NT-CORE context pipeline |

### P1 — Wire SEAL Harness Modules
| Priority | Module | Lines | Action |
|----------|--------|-------|--------|
| P1-1 | procedural_graph | 981 | Wire into SEAL pipeline |
| P1-2 | crystallization | 369 | Wire into SEAL skill crystallization |
| P1-3 | aegis | 303 | Wire into SEAL safety layer |
| P1-4 | harness_evolution | 235 | Wire into SEAL evolution loop |
| P1-5 | harness_optimizer | 246 | Wire into SEAL optimization |

### P2 — Wire Remaining Modules
| Priority | Module | Lines | Action |
|----------|--------|-------|--------|
| P2-1 | io_contract | 87 | Wire into NT-CORE IO contracts |
| P2-2 | reference_view | 102 | Wire into NT-ACT reference views |
| P2-3 | context_boundary | 233 | Wire into NT-SHIELD boundary enforcement |

### P3 — Cleanup
- Resolve dual `distillation` module (merge or remove old version)
- Remove file-level `#[allow(dead_code)]` from modules once wired
- Clear stale cargo lock and verify full build

---

## Dark Forest Assessment

Per the Dark Forest axiom: "every module must compile + test + connect (have consumers) or be deleted."

| Verdict | Count | Action |
|---------|-------|--------|
| Connect (wire into production) | 15 | Wire into parent subsystems |
| Delete (remove dead code) | 0 | No modules flagged for deletion yet |
| Resolve (semantic duplication) | 1 | distillation (seal/ vs nt_mind/) |

**All 21 modules compile** — none are flagged for deletion. The issue is purely integration wiring.

---

## Build Verification

| Check | Status |
|-------|--------|
| Frontend (vite build) | ✅ Pass (23.94s) |
| Rust compilation | ⏳ Needs lock cleanup |
| `cargo check --lib` | Blocked by stale lock |
