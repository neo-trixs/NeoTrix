# Cross-Layer Import Scan — cleanup-297

**Date**: 2026-09-11
**Scope**: `neotrix-core/src/` — 6 layers (L1-L6), excluding `*test*` and `*facade*` files
**Method**: `rg` regex scan for `use crate::l[1-6]_` patterns, diff'd source vs target layer

---

## Architecture Rule

Six-Layer Architecture requires **unidirectional dependency**: L1 ← L2 ← L3 ← L4 ← L5 ← L6. Lower layers must NOT import from higher layers. Cross-layer access should go through `l*_facade.rs` files (official inter-layer interfaces).

---

## Summary

| Category | Count | Status |
|----------|-------|--------|
| Facade files (official cross-layer) | 10 | ✅ By design |
| Direct cross-layer imports (non-facade) | 2 | ⚠️ Violation |
| Upward dependency (L5→L6) | 0 direct | ✅ Clean |
| L4 (Emotion) cross-layer | 0 | ✅ Isolated |

---

## Facade Files (Official Inter-Layer Interfaces) — ✅ By Design

These files are the sanctioned crossing points. Each re-exports types from a lower layer for controlled consumption.

| Facade File | Imports From | Items |
|-------------|-------------|-------|
| `l2_perception/nt_world/l1_facade.rs` | L1 | KnowledgeBase, NodeType, HTTP utils, proxy_from_env |
| `l3_embodiment/l1_facade.rs` | L1 | L1Error, L1Result, GatewayV2, Provider types, WriteGuardStats, cleanup |
| `l5_cognition/act_facade.rs` | L1 | trade, crypto, code modules (recipe_refactor, CryptoAgent, etc.) |
| `l5_cognition/io_facade.rs` | L1 | estimate_tokens, ReasoningKernel, standalone IO |
| `l5_cognition/io_skills_facade.rs` | L1 | 11 IO skill modules (AI image, excalidraw, video, unslop, etc.) |
| `l5_cognition/kb_facade.rs` | L1 | KnowledgeBase, bm25, memory types, crawl, store, confidence |
| `l5_cognition/l2_facade.rs` | L2 | WorldModelV2, UnifiedSearch, NovelQueue |
| `l5_cognition/l3_facade.rs` | L3 | Shield audit types |
| `l5_cognition/l6_facade.rs` | L6 | ConsciousnessGoldStandard, ConsciousnessMonitor, EvalHarness, EvolutionHarness |
| `l6_meta/l1_facade.rs` | L1 | Cleanup shared types |

---

## Direct Cross-Layer Imports (Non-Facade) — ⚠️ Violations

### Violation 1: L5 → L1 (ProjectSnapshot)

```
neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:67
  pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:10
  use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```

**Impact**: L5 (Cognition) directly imports from L1 (Action) bypassing the facade pattern. `ProjectSnapshot` is used in the SEAL evolution loop for self-diagnosis.

**Fix**: Add `ProjectSnapshot` to `l5_cognition/act_facade.rs` and import from there instead.

### Violation 2: L5 → L1 (commented, no impact)

```
neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_loop.rs:21
  // pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;

neotrix-core/src/l5_cognition/nt_mind/evolution/self_diagnose.rs:33
  // pub use crate::l1_action::nt_act::nt_l1_shared_types::{
```

**Impact**: Commented out, no runtime effect. Historical artifact.

---

## Layer Isolation Status

| Layer | Imports FROM lower layers (non-facade) | Status |
|-------|---------------------------------------|--------|
| L1 Action | None | ✅ Clean |
| L2 Perception | None | ✅ Clean |
| L3 Embodiment | None | ✅ Clean |
| L4 Emotion | None (no files found) | ✅ Isolated |
| L5 Cognition | L1 via 2 direct imports | ⚠️ 2 violations |
| L6 Meta | None | ✅ Clean |

---

## Upward Dependency Check

| Pattern | Found | Status |
|---------|-------|--------|
| L1 → L2/L3/L4/L5/L6 | 0 | ✅ |
| L2 → L3/L4/L5/L6 | 0 | ✅ |
| L3 → L4/L5/L6 | 0 | ✅ |
| L4 → L5/L6 | 0 | ✅ |
| L5 → L6 | 0 direct (via l6_facade only) | ✅ |
| L6 → L5/L4/L3/L2 | 0 | ✅ |

---

## Remediation Plan

| # | Action | Priority | File |
|---|--------|----------|------|
| 1 | Move `ProjectSnapshot` re-export to `act_facade.rs` | P1 | `l5_cognition/act_facade.rs` |
| 2 | Update `evolution_loop.rs:67` to use `act_facade` | P1 | `l5_cognition/nt_mind/evolution/evolution_loop.rs` |
| 3 | Update `self_diagnose.rs:10` to use `act_facade` | P1 | `l5_cognition/nt_mind/evolution/self_diagnose.rs` |
| 4 | Remove commented L1 imports in evolution_loop/self_diagnose | P2 | cleanup |

---

## Commands Used

```bash
# Cross-layer import scan (excluding test/facade)
rg --no-heading -n 'use crate::l[1-6]_' \
  neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}/ \
  --glob '!*test*' --glob '!*facade*' --glob '!test/**' --glob '!facade/**'
```
