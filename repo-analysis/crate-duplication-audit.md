# Crate Duplication Audit: neotrix-core vs neotrix-types

> Generated 2026-05-29 | Scope: `neotrix-core/src/core/` ↔ `crates/neotrix-types/src/core/`

---

## 1. Summary

| Metric | neotrix-core | neotrix-types |
|--------|-------------|---------------|
| `.rs` files in `core/` | 120 | 75 |
| Total LOC | 16,569 | 23,604 |
| Shared root files | 17 | 17 |
| Shared subdirs | 6 | 6 |

**Core finding**: The two crates maintain **largely identical copies** of shared type definitions, with neotrix-types acting as the "enriched" superset. Divergence is concentrated in 6 key areas.

---

## 2. Root-Level Files: Duplication Status

| File | Status | Divergence |
|------|--------|-----------|
| `absorb.rs` | **Divergent** (+56 lines in n-t) | n-t adds `MemoryProvider` impl for `ReasoningBank` (orphan-rule-safe), `get_brain_report()` trait method, imports `TaskType` + `ReasoningBank` |
| `accessor.rs` | **Divergent** (-67 lines in n-t) | n-core has `GitHubRepoAccessor` (clones repos via git) + `use std::path::PathBuf`; n-t lacks this type and import |
| `capability.rs` | **Divergent** (+2 lines in n-t) | n-t adds `pub const fn const_dim() -> usize { NUM_FIELDS }` |
| `crt_time.rs` | Identical | — |
| `e8.rs` | Identical | — |
| `e8_observer.rs` | Identical | — |
| `e8_reasoning.rs` | Identical | — |
| `edit.rs` | Identical | — |
| `event.rs` | Identical | — |
| `hypergraph.rs` | Identical | — |
| `iteration.rs` | Identical | — |
| `iteration_agent.rs` | Identical | — |
| `mod.rs` | **Divergent** | n-t declares 14+ extra modules and re-exports (skill, context, epoch, file_parser, layered_memory, skill_tree, self_org, hooks, task_types, panoramic, node_canvas, skills, tools, fs_util, wal, persist_envelope, governance, meta_rules, llm_timeout, context_strategy, self_model) |
| `rkyv_store.rs` | Identical | — |
| `signal.rs` | Identical | — |
| `traits.rs` | Identical | — |
| `walsh_memory.rs` | Identical | — |

**Summary**: 12 of 17 root files are identical. 5 are divergent (absorb, accessor, capability, mod.rs, and the mod.rs differences are structural/declarative only).

---

## 3. Shared Subdirectories

### 3.1 `knowledge/`

| File | Status |
|------|--------|
| `mod.rs` | **Divergent** — n-core includes `activation` module + re-exports; n-t does not |
| `activation.rs` | **n-core only** — has `KSActivationEngine`, `ActivationPolicy`, etc. |
| `provider.rs` | **n-t only** |
| `sources.rs` | **Divergent** — n-t has 30+ more `KnowledgeSource` match arms (SecurityAttacks, LiteParse, SmartSearch, AQBot, AionUi, CyberVerse, Hotpush, etc.) |
| `tracker.rs` | Identical |
| `types.rs` | **Divergent** — n-t has extra `KnowledgeSource` enum variants (SecurityAttacks..LettaMemory, Maigret..CarbonCode) + `MetaCognition = 50` |
| `vectors_group_a.rs` | Identical |
| `vectors_group_b` | **Different type** — n-core: single file (189 lines); n-t: directory with 6 files (cli_agent_tools, infrastructure_cosmology, memory_systems, mod, skills_and_misc, tool_ecosystem) |

### 3.2 `memory/`

| File | Status |
|------|--------|
| `mod.rs` | **Divergent** — n-t exports `LifecycleAction`, `LifecycleConfig`, `MemorySource`, `ConsolidationReport`, adds `seed_knowledge` module |
| `mem.rs` | **Divergent** — n-t adds `MemorySource` enum, extra fields on `ReasoningMemory` (confidence, source, last_used_at, conflict_group, verification_time) |
| `tier.rs` | **Divergent** — n-t adds `Eq + Hash` derives, `LifecycleAction` enum, `LifecycleConfig` struct (179 extra lines) |
| `iteration.rs` | **Divergent** — n-t adds `ConsolidationReport` struct, removes `#[allow(dead_code)]` (45 extra lines) |
| `offload.rs` | Identical |
| `l1.rs` | Identical |
| `pipeline.rs` | Identical |
| `stats.rs` | Identical |
| `compressor.rs` | **n-t only** |
| `seed_knowledge.rs` | **n-t only** |
| `bank/mod.rs` | **Divergent drastically** — n-t: 496 lines (tests + bank_impl refactor); n-core: 146 lines (separate files for store/search/seeds/maintenance/ext) |
| `bank/bank_impl/` | **n-t only** |
| `bank/store.rs` | **n-core only** |
| `bank/search.rs` | **n-core only** |
| `bank/seeds.rs` | **n-core only** |
| `bank/maintenance.rs` | **n-core only** |
| `bank/ext.rs` | **n-core only** |

### 3.3 `metacognition/`

| File | Status |
|------|--------|
| `mod.rs` | **Merged** — `meta_goal_bridge` contents consolidated into `planner` |
| `planner.rs` | **Extended** — now includes `MetaGoal`, `MetaGoalBridge` from former `meta_goal_bridge.rs` |
| All other files | Identical (metacognition_loop, monitor, scanner, self_model, weakness) |

### 3.4 `thinking_model/`

| File | Status |
|------|--------|
| `mod.rs` | Identical |
| `archive.rs` | **Divergent** — n-t uses `serde_json::to_string` for `to_json()`, n-core uses manual string formatting; n-t adds `Serialize/Deserialize` derives + `#[serde(rename)]` attributes (135 diff lines) |
| All other 10 files | Identical (attention_head, context_window, intrinsic_motivation, metacognitive_evaluator, reasoning_strategy, self_referential, silicon_self, skill_crystal, system_identity, thinking_trace) |

### 3.5 `hypercube/`

All 7 files **identical**.

### 3.6 `consciousness/`

| File | Status |
|------|--------|
| `mod.rs` | **Divergent** — n-t adds `pub mod recurrent;` |
| `module_def.rs` | **Divergent** — n-t adds `name()` and `short_name()` methods on `SpecialistType` |
| `recurrent.rs` | **n-t only** |
| `resonance.rs` | Identical |
| `workspace.rs` | Identical |

### 3.7 `epoch/`

| File | Status |
|------|--------|
| `mod.rs` | **Divergent totally** — n-core: 7-line re-export bridge to `neotrix_types`; n-t: full implementation with `definitions.rs` + `types.rs` |
| `definitions.rs` | **n-t only** |
| `types.rs` | **n-t only** |

---

## 4. Unique to `neotrix-core` (not in `neotrix-types`)

These represent capabilities that would need to be preserved if neotrix-types becomes the source of truth:

| Path | Type | Description |
|------|------|-------------|
| `architect_agent/` | Directory (5 files) | ArchitectAgent, ArchitectureDesigner, CodeImplementer, ChangeVerifier |
| `knowledge/activation.rs` | File | KSActivationEngine, ActivationPolicy, KsLifecycle, CascadeSelector, RegisteredSource |
| `knowledge/vectors_group_b.rs` | File (flat) | Capability vector fn for group_b sources |
| `metacognition/meta_goal_bridge.rs` | File | MetaGoal, MetaGoalBridge |
| `memory/bank/store.rs` | File | Separate store impl |
| `memory/bank/search.rs` | File | Separate search impl |
| `memory/bank/seeds.rs` | File | Seed knowledge initialization |
| `memory/bank/maintenance.rs` | File | Maintenance operations |
| `memory/bank/ext.rs` | File | Extension operations |
| `accessor.rs` (extended) | Extra impl | GitHubRepoAccessor (git clone) |

## 5. Unique to `neotrix-types` (not in `neotrix-core`)

These represent types that neotrix-types has developed independently:

| Path | Type | Description |
|------|------|-------------|
| `context/` | Directory (5 files) | ToolSandbox, SessionStore, TruncationStrategy, HookRegistry |
| `context_strategy.rs` | File | Context strategy types |
| `file_parser/` | Directory (5 files) | docx, pdf, text, xml parsers |
| `fs_util.rs` | File | Filesystem utilities |
| `governance.rs` | File | Governance types |
| `hooks.rs` | File | Hooks types |
| `layered_memory.rs` | File | LayeredMemory type |
| `llm_timeout.rs` | File | LLM timeout handling |
| `meta_rules.rs` | File | Meta rules |
| `node_canvas.rs` | File | Node canvas types |
| `panoramic.rs` | File | PanoramicInventory, ModuleEntry |
| `persist_envelope.rs` | File | Persistence envelope |
| `self_measure/` | Directory | Self-measurement types |
| `self_model.rs` | File | Self-model types |
| `self_org/` | Directory (2 files) | Self-org state |
| `skill_tree.rs` | File | Skill tree |
| `skill.rs` | File | Skill types |
| `skills/` | Directory (1 file) | Skills module |
| `task_types.rs` | File | Extended task types |
| `tools/` | Directory (1 file) | Tools module |
| `wal.rs` | File | Write-ahead log |
| `knowledge/provider.rs` | File | Knowledge provider trait |
| `knowledge/vectors_group_b/` | Directory (6 files) | Split group_b vectors |
| `memory/compressor.rs` | File | Memory compression |
| `memory/seed_knowledge.rs` | File | Seed knowledge injection |
| `memory/bank/bank_impl/` | Directory | Refactored bank impl |
| `consciousness/recurrent.rs` | File | ConsciousnessState, ConsciousnessLoop, RecurrentCell |

Plus **semantic additions** (embedded in shared files):
- 30+ `KnowledgeSource` variants
- `MemorySource` enum + extra `ReasoningMemory` fields
- `LifecycleAction`/`LifecycleConfig` for tier system
- `ConsolidationReport`
- `MemoryProvider` impl for `ReasoningBank`
- `SpecialistType::name()`/`short_name()` methods
- `serde_json` based serialization in `archive.rs`
- `const_dim()` on CapabilityVector

---

## 6. Recommended Unification Approach

### Option A: neotrix-types as source of truth (RECOMMENDED)

neotrix-core re-exports from neotrix-types via `pub use neotrix_types::core::...`.

- **Impact**: ~10 neotrix-core files must become thin re-exports
- **Effort**: Medium (2-4 hours)
- **Risk**: Low
- **Preserves**: All neotrix-types unique features (the "enriched" superset)
- **Requires**: Move `architect_agent/`, `meta_goal_bridge.rs`, `activation.rs`, `bank/store/search/seeds/maintenance/ext`, and `GitHubRepoAccessor` from neotrix-core into neotrix-types (or keep as neotrix-core-only extras)

**Key migration steps**:
1. Strip `knowledge/`, `memory/`, `metacognition/`, `thinking_model/`, `hypercube/`, `consciousness/`, `epoch/` from neotrix-core
2. Replace with `pub use neotrix_types::core::{...}`
3. Move architect_agent → neotrix-types (or keep as neotrix-core-only)
4. Merge `bank/store.rs` + `bank/search.rs` + `bank/seeds.rs` + `bank/maintenance.rs` + `bank/ext.rs` into `bank/bank_impl/` (n-t style)
5. Resolve `knowledge/vectors_group_b` file vs directory conflict (adopt directory approach)
6. Resolve `accessor.rs` — move `GitHubRepoAccessor` to n-t or n-core-exclusive

### Option B: neotrix-core as source of truth

neotrix-types re-exports from neotrix-core (reverse of Option A).

- **Effort**: High (8-12 hours)
- **Risk**: High
- **Problem**: neotrix-types is the "enriched" superset — reversing would require backporting all n-t unique features into n-core, including ~30 KnowledgeSource variants, MemorySource, LifecycleConfig, typed bank_impl, etc. This would balloon neotrix-core dramatically.

### Option C: Single shared types crate

Merge both into a single `neotrix-types` that both `neotrix-core` and downstream crates depend on.

- **Effort**: Medium (3-6 hours)
- **Risk**: Medium
- **Pro**: Eliminates duplication entirely
- **Con**: Major dependency graph surgery; both existing crates must be updated simultaneously
- **Note**: This could be Phase 2 after Option A is proven

---

## 7. Recommendation

**Option A — neotrix-types as source of truth**.

Rationale:
1. neotrix-types is the superset (23,604 LOC vs 16,569) — it has already added features
2. 14 of 17 shared root files are identical or near-identical — re-export is trivial
3. Subdirectory divergence is mostly neotrix-types additions (not neotrix-core removals)
4. Current `epoch/mod.rs` in neotrix-core already **is** a re-export bridge to neotrix-types — proof of concept works
5. neotrix-core can keep its exclusive modules (`architect_agent/`, `meta_goal_bridge.rs`, `activation.rs`, `GitHubRepoAccessor`) as crate-local, only re-exporting shared types
