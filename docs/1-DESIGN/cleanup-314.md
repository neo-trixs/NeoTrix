# Cross-Layer Import Scan — cleanup-314

**Date**: 2026-09-11 19:57
**Scope**: `neotrix-core/src/` L1-L6 six-layer architecture
**Exclusions**: facade files (`*facade*`), test directories (`tests/`)
**Detection**: Three import paths — `use crate::l{N}_` (direct), `use crate::neotrix::` (facade bypass), `use crate::core::` (legacy)

---

## 1. Direct Cross-Layer Violations (`use crate::l{N}_`)

True layer boundary breaches. **11 total.**

### L1 → L5 (1 violation)

| File | Line | Import |
|------|------|--------|
| `l1_action/nt_act/nt_act_orchestrator/pm_integration_test.rs` | 4 | `use crate::l5_cognition::nt_mind::nt_mind::goal_loop::priority::{PriorityEngine, MoscowClass}` |

> Note: file name contains `test` — likely a test file. Should be excluded from layering rules.

### L5 → L1 (6 violations)

| File | Line | Import |
|------|------|--------|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 679 | `use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch}` |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 680 | `use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{...}` |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` |
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 33 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::{...}` (comment) |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 21 | `// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType` (comment) |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` |

**Real code violations**: 4 (lines 679, 680, 10, 67)

### L5 → L6 (1 violation)

| File | Line | Import |
|------|------|--------|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 940 | `use crate::l6_meta::coordination::self_improvement::SystemMetrics` |

### L5 → L2 (0 code violations)

Lines 16 and 22 in `l5_cognition/mod.rs` are doc comments (`///`), not imports.

### Matrix

| From \ To | L1 | L2 | L3 | L4 | L5 | L6 |
|-----------|----|----|----|----|----|----|
| **L1** | - | 0 | 0 | 0 | **1** | 0 |
| **L2** | 0 | - | 0 | 0 | 0 | 0 |
| **L3** | 0 | 0 | - | 0 | 0 | 0 |
| **L4** | 0 | 0 | 0 | - | 0 | 0 |
| **L5** | **4** | 0 | 0 | 0 | - | **1** |
| **L6** | 0 | 0 | 0 | 0 | 0 | - |

---

## 2. Facade Bypass (`use crate::neotrix::`)

Modules importing via the `neotrix` re-export layer instead of direct crate paths. **~74 occurrences.**

| Layer | Count | Top offenders |
|-------|-------|---------------|
| L1_action | 20 | `nt_act_code/pipeline_autofixer.rs`, `nt_act_autonomy/meta_goal_generator.rs`, `nt_io_provider/free_providers.rs` |
| L2_perception | 15 | `nt_world_sense_hub.rs`, `world_consciousness.rs`, `crawl/classifier.rs` |
| L3_embodiment | 15 | `nt_shield_stealth_net/http_client/mod.rs`, `bandit.rs`, `video_post_processor.rs` |
| L4_emotion | 0 | — |
| L5_cognition | 20 | `handlers_core.rs`, `run.rs`, `mod.rs`, `web_miner.rs`, `knowledge_chain.rs` |
| L6_meta | 4 | `transcendent_loop.rs`, `nt_mind_eval_harness.rs` |

### Key facade paths used

- `crate::neotrix::nt_memory_kb::` — L2/L5 access to KB (should use L1 path)
- `crate::neotrix::nt_io_http_factory::` — L3 shield accessing L1 HTTP infra
- `crate::neotrix::nt_shield_stealth_net::` — L5 cognition accessing L3 shield
- `crate::neotrix::nt_world_sense::` — L5 cognition accessing L2 perception
- `crate::neotrix::nt_act_voice::` — L5 cognition accessing L1 voice

---

## 3. Legacy Path (`use crate::core::`)

Old `core/` module path still widely used. **~101+ occurrences** across all layers.

| Layer | Count | Primary targets |
|-------|-------|-----------------|
| L1_action | 20+ | `nt_core_cap`, `nt_core_self_test`, `nt_core_sense`, `nt_core_reasoning` |
| L2_perception | 20+ | `nt_core_sense`, `nt_core_hcube`, `nt_core_td`, `nt_core_math` |
| L3_embodiment | 20+ | `nt_core_resource_pool`, `nt_core_self_test`, `nt_core_capability` |
| L4_emotion | 1 | `nt_core_self::emotion_state` |
| L5_cognition | 20+ | `nt_core_gate`, `nt_core_event`, `nt_core_meta`, `nt_core_hcube` |
| L6_meta | 20+ | `nt_core_self_test`, `nt_core_traits`, `nt_core_consciousness_core` |

---

## 4. Summary

| Category | Count | Severity |
|----------|-------|----------|
| Direct cross-layer `use crate::l{N}_` | **5** (excl. comments) | HIGH — violates layer isolation |
| Facade bypass `use crate::neotrix::` | **~74** | MEDIUM — works but hides dependency direction |
| Legacy `use crate::core::` | **~101+** | LOW — legacy path, functional but deprecated |
| **Total** | **~180+** | |

### Top Priority Fixes

1. **L5→L1 direct**: `handlers_maintenance.rs:679-680` — cognition importing memory KB types
2. **L5→L1 direct**: `self_diagnose.rs:10` + `evolution_loop.rs:67` — cognition importing action types (`ProjectSnapshot`)
3. **L5→L6 direct**: `handlers_maintenance.rs:940` — cognition importing meta governance types
4. **L1→L5 direct**: `pm_integration_test.rs:4` — action importing cognition priority engine (test file, lower priority)

### Recommendations

- **Direct violations (5)**: Route through trait abstractions in `traits.rs` or move shared types to a lower layer
- **Facade bypass (~74)**: Replace `crate::neotrix::X` with `crate::l{N}::X` for explicit layer attribution
- **Legacy path (~101)**: Gradually migrate `crate::core::X` → `crate::l{N}::X` or keep as-is if `core/` is treated as shared foundation
