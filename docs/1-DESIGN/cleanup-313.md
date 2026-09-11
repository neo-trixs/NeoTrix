# Cleanup-313: Cross-Layer Import Scan

**Date**: 2026-09-11
**Scope**: `neotrix-core/src/l1_action` ~ `l6_meta` (6 layers)
**Method**: `grep -rn 'use crate::l[1-6]_'` on non-test, non-facade `.rs` files
**Files scanned**: 1,179

---

## 1. Executive Summary

**Total direct cross-layer imports: 10** — all originating from **L5-Cognition**.

L5-Cognition is the only layer with direct cross-layer `use crate::l*_*` imports. The other 5 layers have **zero** direct cross-layer imports, meaning the layering discipline is well-maintained outside of L5.

However, L5 has **extensive indirect coupling** via `crate::core::` (317 imports) and `crate::neotrix::` (104 imports), totaling **421 coupling imports** — by far the highest of any layer.

---

## 2. Cross-Layer Import Matrix (Direct `use crate::l*_*`)

| Source \ Target | L1-Action | L2-Perception | L3-Embodiment | L4-Emotion | L5-Cognition | L6-Meta |
|---|---|---|---|---|---|---|
| **L1-Action** | -- | 0 | 0 | 0 | 0 | 0 |
| **L2-Perception** | 0 | -- | 0 | 0 | 0 | 0 |
| **L3-Embodiment** | 0 | 0 | -- | 0 | 0 | 0 |
| **L4-Emotion** | 0 | 0 | 0 | -- | 0 | 0 |
| **L5-Cognition** | **6** | **1** | 0 | 0 | -- | **3** |
| **L6-Meta** | 0 | 0 | 0 | 0 | 0 | -- |

---

## 3. Indirect Coupling via `crate::core::` / `crate::neotrix::`

| Layer | `crate::core::` | `crate::neotrix::` | Total |
|---|---|---|---|
| L1-Action | 98 | 31 | **129** |
| L2-Perception | 49 | 15 | **64** |
| L3-Embodiment | 32 | 15 | **47** |
| L4-Emotion | 1 | 0 | **1** |
| **L5-Cognition** | **317** | **104** | **421** |
| L6-Meta | 22 | 4 | **26** |

---

## 4. Top 15 Most-Coupled Files

| # | File | `core::` | `neotrix::` | Total |
|---|---|---|---|---|
| 1 | `l5_cognition/nt_mind/nt_mind/reason/reasoning_engine/engine_core.rs` | 34 | 6 | **40** |
| 2 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | 19 | 10 | **29** |
| 3 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs` | 20 | 5 | **25** |
| 4 | `l5_cognition/nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | 18 | 0 | **18** |
| 5 | `l5_cognition/nt_mind/nt_mind/consciousness/panorama_pipeline.rs` | 9 | 2 | **11** |
| 6 | `l5_cognition/nt_mind/nt_mind_background_loop/run.rs` | 9 | 1 | **10** |
| 7 | `l5_cognition/nt_mind/nt_mind_background_loop/mod.rs` | 6 | 3 | **9** |
| 8 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/loop_impl/core.rs` | 5 | 4 | **9** |
| 9 | `l5_cognition/nt_mind/nt_mind/reason/attention_router.rs` | 6 | 3 | **9** |
| 10 | `l5_cognition/nt_mind/nt_mind/consciousness/hypercube_bridge.rs` | 9 | 0 | **9** |
| 11 | `l5_cognition/nt_mind/nt_mind/reason/sleep/consolidation.rs` | 8 | 0 | **8** |
| 12 | `l2_perception/nt_world/crawl/mapper.rs` | 7 | 1 | **8** |
| 13 | `l5_cognition/nt_mind/nt_mind/reason/sleep/hebbian.rs` | 7 | 0 | **7** |
| 14 | `l5_cognition/nt_mind/nt_mind/reason/sleep/engine.rs` | 6 | 1 | **7** |
| 15 | `l5_cognition/nt_mind/nt_mind/evolution/goal_loop/loop_impl/core.rs` | 4 | 3 | **7** |

---

## 5. Direct Cross-Layer Import Details (L5 only)

### L5-Cognition → L1-Action (6 imports)

| File | Line | Import |
|---|---|---|
| `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 679 | `crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch}` |
| `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 680 | `crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{...}` |
| `nt_mind/evolution/self_diagnose.rs` | 10 | `crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` |
| `nt_mind/evolution/self_diagnose.rs` | 33 | `crate::l1_action::nt_act::nt_l1_shared_types::{...}` (commented) |
| `nt_mind/evolution/evolution_loop.rs` | 21 | `crate::l1_action::nt_act::nt_l1_shared_types::IssueType` (commented) |
| `nt_mind/evolution/evolution_loop.rs` | 67 | `crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` |

**Nature**: L5 needs `ProjectSnapshot`, `IssueType`, `CommunityDetector`, `KnowledgeBase` types from L1.

### L5-Cognition → L2-Perception (1 import)

| File | Line | Import |
|---|---|---|
| `mod.rs` | 16 | `use crate::l2_perception::*` — **comment** (doc only, facade already exists) |

**Status**: No actual wildcard import in code. `l2_facade.rs` already centralizes this.

### L5-Cognition → L6-Meta (3 imports)

| File | Line | Import |
|---|---|---|
| `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 940 | `crate::l6_meta::coordination::self_improvement::SystemMetrics` |
| `mod.rs` | 22 | `use crate::l6_meta::*` — **comment** (doc only, facade already exists) |
| `traits.rs` | 137 | `use crate::l6_meta::*` — **comment** (doc only, discussing facade pattern) |

**Status**: Only 1 real import (`SystemMetrics`). The other 2 are doc comments.

---

## 6. Facade Layer Status

L5 already has facade modules to centralize cross-layer references:

| Facade | Purpose | Lines in `mod.rs` |
|---|---|---|
| `kb_facade` | L5 → L1 NT-MEMORY KB types re-export | 4-5 |
| `io_facade` | L5 → L1 NT-IO shared types re-export | 6-7 |
| `io_skills_facade` | L5 → L1 NT-IO skill modules re-export | 8-9 |
| `act_facade` | L5 → L1 NT-ACT shared types re-export | 10-11 |
| `l3_facade` | L5 → L3 shared types re-export | 12-13 |
| `l2_facade` | L5 → L2 perception types re-export | 14-19 |
| `l6_facade` | L5 → L6 meta-cognition types re-export | 20-25 |

**Observation**: Facades exist but **not all L5 files use them**. The 6 direct L1 imports bypass `act_facade`/`kb_facade`.

---

## 7. Violations & Recommendations

### Critical: L5→L1 bypass of facades (6 imports)

`handlers_maintenance.rs`, `self_diagnose.rs`, `evolution_loop.rs` import directly from `crate::l1_action::*` instead of through `act_facade`/`kb_facade`.

**Fix**: Route through `crate::l5_cognition::act_facade` / `kb_facade`.

### Medium: Top-3 coupled files (>25 imports each)

`engine_core.rs` (40), `pipeline.rs` (29), `seal_loop.rs` (25) — these files have excessive coupling to `crate::core::*`. Consider:
- Extract shared types into a `l5_cognition::shared_types` module
- Use facade pattern for `crate::core::nt_core_*` imports
- Split large files into smaller, focused modules

### Low: L6-Meta import of `SystemMetrics` from L5→L6

Only 1 real cross-layer import (upward dependency L5→L6). Already documented as acceptable via `l6_facade`.

### Informational: `crate::core::` is the dominant coupling vector

317 imports from `crate::core::*` across L5 alone — this is the **primary coupling surface** between L5 and the foundation layer. The `core/` directory acts as a shared type library. This is architecturally acceptable but should be monitored for drift.

---

## 8. Per-Layer Cross-Layer File Counts

| Layer | Files scanned | Cross-layer out | Files with cross-layer |
|---|---|---|---|
| L1-Action | 389 | 0 | 0 |
| L2-Perception | 211 | 0 | 0 |
| L3-Embodiment | 177 | 0 | 0 |
| L4-Emotion | 6 | 0 | 0 |
| L5-Cognition | 337 | 10 | 4 |
| L6-Meta | 59 | 0 | 0 |
