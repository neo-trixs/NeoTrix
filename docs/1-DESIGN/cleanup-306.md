# Cleanup #306 — Cross-Layer Reference Scan

**Scan date**: 2026-09-11
**Scope**: `neotrix-core/src/` layers l1_action..l6_meta + neotrix/
**Exclusions**: test files, facade files, test directories

## Layer Map

| Layer | Directory | Role |
|-------|-----------|------|
| L1 Action | `l1_action/` | 工具/动作/记忆/IO |
| L2 Perception | `l2_perception/` | 世界感知/爬虫 |
| L3 Embodiment | `l3_embodiment/` | 具身/安全/情感具身 |
| L4 Emotion | `l4_emotion/` | 情感引擎 |
| L5 Cognition | `l5_cognition/` | 核心推理/自我进化 |
| L6 Meta | `l6_meta/` | 元认知/自愈/跨会话 |

## Summary

| Layer | Cross-layer refs | Status |
|-------|-----------------|--------|
| l1_action | 0 | ✅ Clean |
| l2_perception | 0 | ✅ Clean |
| l3_embodiment | 0 | ✅ Clean |
| l4_emotion | 0 | ✅ Clean |
| **l5_cognition** | **10** | ⚠️ Violations |
| l6_meta | 0 | ✅ Clean |

**Total violations: 10** (all from L5)

## Cross-Layer Reference Matrix

| From \ To | L1 Action | L2 Perception | L3 Embodiment | L4 Emotion | L5 Cognition | L6 Meta |
|-----------|-----------|---------------|---------------|------------|--------------|---------|
| **L1 Action** | - | ✅ | ✅ | ✅ | ✅ | ✅ |
| **L2 Perception** | ✅ | - | ✅ | ✅ | ✅ | ✅ |
| **L3 Embodiment** | ✅ | ✅ | - | ✅ | ✅ | ✅ |
| **L4 Emotion** | ✅ | ✅ | ✅ | - | ✅ | ✅ |
| **L5 Cognition** | ⚠️ 4 | ⚠️ 1 | ✅ | ✅ | - | ⚠️ 3 |
| **L6 Meta** | ✅ | ✅ | ✅ | ✅ | ✅ | - |

## Violation Details

### ⚠️ L5→L1 (Downward — Highest Priority)

Cognition 直接引用 Action 层，违反分层依赖方向。

| File | Line | Import | Type |
|------|------|--------|------|
| `l5_cognition/nt_mind/evolution/self_diagnose.rs` | 10 | `use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` | Direct |
| `l5_cognition/nt_mind/evolution/evolution_loop.rs` | 67 | `pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot` | Re-export |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 679 | `use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{CommunityDetector, CommunityAwareSearch}` | Direct |
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 680 | `use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{...}` | Direct |

**Fix**: Move `ProjectSnapshot` to a shared types crate or use trait abstraction. Community/KB access should go through `kb_facade` or `act_facade`.

### ⚠️ L5→L6 (Upward — Medium Priority)

Cognition 直接引用 Meta 层，可能形成循环依赖。

| File | Line | Import | Type |
|------|------|--------|------|
| `l5_cognition/nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | 940 | `use crate::l6_meta::coordination::self_improvement::SystemMetrics` | Direct |
| `l5_cognition/mod.rs` | 22 | `/// L6 类型通过此模块访问` (declaration) | Comment |
| `l5_cognition/traits.rs` | 137 | `// use crate::l6_meta::*` (commented out) | Commented |

**Fix**: `SystemMetrics` should be accessed via event bus or trait. L6 should provide metrics via EventBus, not direct import.

### ⚠️ L5→L2 (Upward — Low Priority)

| File | Line | Import | Type |
|------|------|--------|------|
| `l5_cognition/mod.rs` | 16 | `/// L2 类型通过此模块访问` (declaration) | Comment |

**Fix**: This is a facade declaration comment, not an actual import. Verify that `l2_facade.rs` properly abstracts L2 access.

## Facade Inventory

L5 uses facade modules to abstract cross-layer access:

| Facade | Location | Abstracts | Status |
|--------|----------|-----------|--------|
| `act_facade.rs` | `l5_cognition/` | L1 Action | ✅ Used |
| `io_facade.rs` | `l5_cognition/` | L1 IO | ✅ Used |
| `kb_facade.rs` | `l5_cognition/` | L1 Memory/KB | ✅ Used |
| `l2_facade.rs` | `l5_cognition/` | L2 Perception | ✅ Used |
| `l3_facade.rs` | `l5_cognition/` | L3 Embodiment | ✅ Used |
| `l6_facade.rs` | `l5_cognition/` | L6 Meta | ✅ Used |

**Facade violations** (refs that bypass facades):

| Source File | Should Use | Instead Uses |
|-------------|-----------|--------------|
| `handlers_maintenance.rs:679-680` | `kb_facade` | Direct `l1_action::nt_memory` |
| `self_diagnose.rs:10` | `act_facade` | Direct `l1_action::nt_act` |
| `evolution_loop.rs:67` | `act_facade` | Direct `l1_action::nt_act` |
| `handlers_maintenance.rs:940` | `l6_facade` | Direct `l6_meta` |

## Cross-Layer mod Declarations

| File | Declaration | Purpose |
|------|-------------|---------|
| `l2_perception/nt_world/mod.rs:10` | `pub mod l1_facade` | L2 exposes L1 facade |
| `l3_embodiment/mod.rs:5` | `pub mod l1_facade` | L3 exposes L1 facade |
| `l5_cognition/mod.rs:13` | `pub mod l3_facade` | L5 wraps L3 access |
| `l5_cognition/mod.rs:19` | `pub mod l2_facade` | L5 wraps L2 access |
| `l5_cognition/mod.rs:25` | `pub mod l6_facade` | L5 wraps L6 access |
| `l6_meta/mod.rs:15` | `pub mod l1_facade` | L6 exposes L1 facade |
| `l5_cognition/nt_mind/foundation/mod.rs:7` | `pub mod l1_wrappers` | L5 wraps L1 functions |

## neotrix/ Glue Layer

`neotrix/mod.rs` references all 6 layers — this is expected for the top-level glue module:

- L1 Action: 11 refs (module re-exports)
- L2 Perception: 2 refs
- L3 Embodiment: 4 refs
- L4 Emotion: 1 ref
- L5 Cognition: 4 refs
- L6 Meta: 3 refs

`neotrix/nt_shanhai_geo/geo_sync.rs:8` → L1 Action (direct, not facade)

## Recommendations

### Priority 1 — Fix L5→L1 Direct Violations (4 refs)

1. **`ProjectSnapshot`**: Move to shared types or define trait in `act_facade`
2. **CommunityDetector/CommunityAwareSearch**: Access through `kb_facade` trait
3. **nt_memory_store**: Access through `kb_facade` trait

### Priority 2 — Fix L5→L6 Direct Violations (1 active ref)

1. **`SystemMetrics`**: Access through event bus or define `MetricsProvider` trait in `l6_facade`

### Priority 3 — Verify Facade Coverage

1. Audit `l2_facade.rs` covers all L2 types needed by L5
2. Audit `l6_facade.rs` covers `SystemMetrics` access
3. Ensure no new direct cross-layer imports in CI (lint rule)

### Priority 4 — Optional Architecture Hardening

1. Add `#![deny(unused_imports)]` to catch dead cross-layer imports
2. Consider adding a `forbid_direct_cross_layer` lint via `cargo-deny` or custom script
3. Document facade pattern in `CONTEXT.md` as architecture rule

---
*Generated by cross-layer scanner — Cleanup #306*
