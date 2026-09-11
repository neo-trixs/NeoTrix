# Redundancy Cleanup Analysis — 2026-09-11

**Scope**: `neotrix-core/src/` full scan (622,939 lines, ~1,200+ .rs files)

---

## 1. unified_archive Dead Code

### Verdict: **DEAD CODE — Safe to delete**

- **98 `.rs` files**, 4,769 lines total
- **Not declared** in `lib.rs` (no `pub mod unified_archive;`)
- **No imports** anywhere in the active codebase (`grep mod unified_archive` returns 0 hits)
- Contains duplicated snapshots of active `l2_perception/nt_world` files (bilibili.rs, bandcamp.rs, arxiv.rs, etc.)

| Stat | Value |
|------|-------|
| Files | 98 |
| Total lines | 4,769 |
| Largest file | `core/multi_cache.rs` (162 lines) |
| Average size | ~49 lines |

**Action**: `rm -rf neotrix-core/src/unified_archive/`

---

## 2. Duplicate Type Consolidation

### 2a. SearchResult — 13 definitions (12 active + 1 archive)

| # | File | Layer | Status |
|---|------|-------|--------|
| 1 | `core/nt_core_answer_engine.rs:21` | L5 | **Candidate fact source** |
| 2 | `core/nt_core_vector_store/types.rs:45` | L5 | Duplicate |
| 3 | `l2_perception/nt_world/nt_world_search.rs:60` | L2 | Duplicate |
| 4 | `l2_perception/nt_world/source/types.rs:86` | L2 | Duplicate |
| 5 | `l1_action/traits.rs:239` | L1 | Duplicate |
| 6 | `l1_action/nt_act/types.rs:162` | L1 | Duplicate |
| 7 | `l1_action/nt_memory/nt_memory_kb/nt_memory_types.rs:83` | L1 | Duplicate |
| 8 | `l1_action/nt_memory/nt_memory_kb/kb_cognition.rs:25` | L1 | Duplicate |
| 9 | `l1_action/nt_memory/nt_memory_kb/ntx/vec_segment.rs:305` | L1 | Duplicate |
| 10 | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/search_skill_stage.rs:68` | L5 | Duplicate |
| 11 | `l5_cognition/nt_mind/nt_mind/evolution/casebase.rs:521` | L5 | Duplicate |
| 12 | `l3_embodiment/nt_shield/nt_shield_stealth_net/crawler_core.rs:100` | L3 | Duplicate |
| 13 | `unified_archive/.../core/types.rs:86` | Archive | Dead |

**Recommendation**: Create `core::nt_core_types::SearchResult` as single fact source. All 12 active modules should `use` the canonical type.

### 2b. RiskLevel — 13 definitions (0 archive)

| # | File | Layer | Semantic Domain |
|---|------|-------|-----------------|
| 1 | `core/nt_core_meta/planner.rs:220` | L5 | Planning |
| 2 | `core/nt_core_self/human_approval.rs:24` | L5 | Approval |
| 3 | `l2_perception/nt_world/explore/system_scanner.rs:19` | L2 | Cleanup |
| 4 | `l1_action/nt_act/nt_act_cleanup/shared.rs:15` | L1 | Cleanup |
| 5 | `l1_action/nt_act/nt_act_trade/trade_core.rs:385` | L1 | Trade |
| 6 | `l1_action/nt_act/nt_act_trade/finance_compliance.rs:114` | L1 | Trade |
| 7 | `l1_action/nt_act/nt_act_trade/full_cycle.rs:509` | L1 | Trade |
| 8 | `l1_action/nt_act/nt_act_trade/production_logistics.rs:324` | L1 | Trade |
| 9 | `l1_action/nt_act/actions/disk_guard.rs:24` | L1 | Safety |
| 10 | `l6_meta/evolution/nt_act_human_approval.rs:66` | L6 | Approval |
| 11 | `l5_cognition/nt_mind/foundation/cleanup_engine.rs:639` | L5 | Cleanup |
| 12 | `l5_cognition/nt_mind/foundation/repair.rs:77` | L5 | Repair |
| 13 | `l3_embodiment/nt_shield/nt_shield/redaction.rs:19` | L3 | Security |

**Recommendation**: Merge into 3 semantic types:
- `GuardRiskLevel` (security/approval): items 2, 9, 10, 13
- `CleanupRiskLevel` (cleanup/repair): items 3, 4, 11, 12
- `TradeRiskLevel` (trade/finance): items 5, 6, 7, 8
- `PlanningRiskLevel` (meta): item 1

### 2c. GraphNode / GraphEdge — 8 definitions (1 fact source + 7 duplicates)

| # | File | Status |
|---|------|--------|
| 1 | `core/nt_core_graph_types.rs:6,37` | **Canonical fact source** |
| 2 | `core/nt_core_capability/mod.rs:577,586` | Duplicate |
| 3 | `neotrix/nt_unified_api/mod.rs:192,201` | Duplicate |
| 4 | `l1_action/nt_memory/nt_memory_openknowledge.rs:97,106` | Duplicate |
| 5 | `l1_action/nt_memory/nt_memory_kb/ntx/graph_segment.rs:30,20` | Duplicate |
| 6 | `l1_action/nt_memory/nt_memory_kb/knowledge_storage.rs:313,321` | Duplicate |
| 7 | `l1_action/nt_memory/nt_memory_leann_store.rs:9,20` | Duplicate |
| 8 | `l5_cognition/nt_mind/nt_mind/graph_types.rs:56,67` | Duplicate |
| 9 | `l5_cognition/nt_mind/nt_mind/evolution/deliberation.rs:83` (GraphEdge only) | Duplicate |

**Recommendation**: All 7 duplicate modules should `use core::nt_core_graph_types::{GraphNode, GraphEdge}`.

### 2d. Domain enum — 6+ definitions

| # | File | Variants |
|---|------|----------|
| 1 | `core/nt_core_capability/mod.rs:48` | 7 domains |
| 2 | `core/l7_capability/nt_core_grounded_gate.rs:172` | — |
| 3 | `l2_perception/nt_world/sense/nt_world_model_types.rs:144` | — |
| 4 | `neotrix/nt_core_capability_tree/src/node.rs:9` | — |
| 5 | `l1_action/nt_io/nt_io_provider/generation_classifier.rs:72` | — |
| 6 | `l5_cognition/nt_mind/nt_mind/experience_tree/mod.rs:62` | — |
| 7 | `l3_embodiment/nt_shield/slang_norm.rs:9` | — |
| 8 | `architecture/mod.rs:453` | — |

**Recommendation**: Define `core::nt_core_domain::Domain` as single fact source.

---

## 3. Large File Detection (>3000 lines)

| Lines | File | Recommendation |
|-------|------|----------------|
| 4,118 | `bin/experience.rs` | 🔴 Split into command modules |
| 3,349 | `core/nt_core_consciousness_core.rs` | 🔴 Split into submodules by responsibility |
| 3,219 | `l5_cognition/.../seal_core/.../pipeline.rs` | 🔴 Split pipeline stages |
| 3,192 | `entry/mod.rs` | 🔴 Extract dispatch logic |
| 3,091 | `l1_action/nt_memory/nt_memory_kb/mod.rs` | 🔴 Split into submodules |

**Files 2000-3000 lines (split recommended)**:

| Lines | File |
|-------|------|
| 2,628 | `l5_cognition/nt_mind/nt_mind_skill_engine.rs` |
| 2,509 | `l5_cognition/.../reasoning_engine/engine_core.rs` |
| 2,327 | `l5_cognition/.../evolution/evolution_loop.rs` |
| 2,270 | `l2_perception/.../nt_world_video_pipeline.rs` |
| 2,228 | `l1_action/.../nt_memory_graphrag/mod.rs` |
| 2,178 | `core/nt_core_mcp.rs` |
| 2,149 | `core/nt_core_gate/mod.rs` |
| 2,142 | `l6_meta/healing/nt_mind_eval_harness.rs` |
| 2,140 | `l5_cognition/.../handlers_consciousness.rs` |
| 2,134 | `l1_action/.../nt_memory_search.rs` |
| 2,055 | `l1_action/.../nt_memory_geo.rs` |
| 2,029 | `core/nt_core_e8/nt_core_community_ingester.rs` |
| 2,016 | `neotrix/nt_file_ability.rs` |
| 1,995 | `core/nt_core_deploy.rs` |
| 1,970 | `l5_cognition/.../cleanup_engine.rs` |

---

## 4. Empty Module Detection (<20 lines)

### Tier 1: Likely remnants (1-6 lines, non-re-export)

| Lines | File | Verdict |
|-------|------|---------|
| 1 | `l1_action/nt_io/nt_io_plugin/builtin/mod.rs` | 🔴 Residue |
| 1 | `l5_cognition/nt_core/cuda/mod.rs` | 🔴 Residue |
| 1 | `l5_cognition/nt_core/safety/mod.rs` | 🔴 Residue |
| 1 | `l5_cognition/nt_mind/harness/mod.rs` | 🔴 Residue |
| 1 | `l6_meta/nt_meta/mod.rs` | 🔴 Residue |
| 6 | `core/energy_core/seal_pipeline.rs` | 🔴 Residue |

### Tier 2: Possible re-exports (could be valid)

| Lines | File | Verdict |
|-------|------|---------|
| 2 | `l2_perception/nt_world/source/text/book/mod.rs` | 🟡 Check consumers |
| 2 | `l2_perception/nt_world/source/text/document/mod.rs` | 🟡 Check consumers |
| 3 | `l2_perception/nt_world/source/text/lyrics/mod.rs` | 🟡 Check consumers |
| 3 | `l2_perception/nt_world/source/video/mod.rs` | 🟡 Check consumers |
| 4 | `l2_perception/nt_world/source/text/social/mod.rs` | 🟡 Check consumers |
| 5 | `server/mod.rs` | 🟡 Entry point |
| 8 | `cli/tui/app/mod.rs` | 🟡 Entry point |

### Tier 3: Layer facade modules (valid pattern, keep)

| Lines | File | Verdict |
|-------|------|---------|
| 5-19 | `l1_action/mod.rs`, `l2_perception/mod.rs`, etc. | ✅ Layer re-exports |
| 11-19 | `core/l2_perception/mod.rs`, facade modules | ✅ Valid pattern |

---

## 5. Consolidated Action Plan

### P0 — Type Unification (compile-time impact)

| Action | Scope | Files affected |
|--------|-------|----------------|
| Create `core::nt_core_types::SearchResult` | 12 modules | 12 |
| Create `core::nt_core_domain::Domain` | 6+ modules | 6+ |
| Unify `RiskLevel` into 3 semantic types | 13 modules | 13 |
| Re-export `GraphNode`/`GraphEdge` from `core::nt_core_graph_types` | 7 modules | 7 |

### P1 — Dead Code Removal

| Action | Impact |
|--------|--------|
| Delete `unified_archive/` directory | -4,769 lines, -98 files |
| Remove 6 Tier-1 remnant modules | -11 lines |

### P2 — Large File Splitting

| Target | Strategy |
|--------|----------|
| `bin/experience.rs` (4,118L) | Extract subcommands into separate files |
| `core/nt_core_consciousness_core.rs` (3,349L) | Split into `cognition/`, `meta/`, `emotion/` |
| `l5_cognition/.../pipeline.rs` (3,219L) | Split by pipeline stage |
| `entry/mod.rs` (3,192L) | Extract dispatch + commands |
| `l1_action/.../mod.rs` (3,091L) | Split KB into submodules |

### P3 — Empty Module Merge

Merge 6 Tier-1 remnants into parent modules or delete if no consumers.

---

## 6. Estimated Impact

| Metric | Before | After (projected) |
|--------|--------|-------------------|
| Total lines | 622,939 | ~618,170 (-4,769) |
| .rs files | ~1,200+ | ~1,100 (-98 archive) |
| SearchResult defs | 13 | 1 |
| RiskLevel defs | 13 | 3 |
| GraphNode defs | 8 | 1 |
| Domain defs | 6+ | 1 |
| Files >3000L | 5 | 0 (after split) |
