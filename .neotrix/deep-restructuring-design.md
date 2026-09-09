# Deep Restructuring Design: nt_mind & nt_act Flat Files

## Executive Summary

Analysis of 29 nt_mind flat files and ~50 nt_act flat files reveals **two structural layers** of dependency risk:

1. **`pub use` re-export chains** in mod.rs create implicit public API paths — moving files breaks downstream consumers
2. **`super::` imports** between flat files create intra-module coupling — moving files breaks sibling imports

**Safe approach:** Move isolated flat files first (no cross-file `super::`, no `pub use` chain), verify with `cargo check` after each batch, then tackle clusters as coordinated groups.

---

## Module Path Overview

| Module | Actual Path | Mod.rs Location |
|--------|-------------|-----------------|
| nt_mind | `src/l5_cognition/nt_mind/` | `l5_cognition/nt_mind/mod.rs` |
| nt_act | `src/l1_action/nt_act/` | `l1_action/nt_act/mod.rs` |
| nt_mind (inner) | `src/l5_cognition/nt_mind/nt_mind/` | `l5_cognition/nt_mind/nt_mind/mod.rs` |
| neotrix (wrapper) | `src/neotrix/mod.rs` | Re-exports specific modules from both |

---

## Part 1: nt_mind Analysis

### 1.1 `pub use` Re-export Chains (CRITICAL — breaks public API)

```
l5_cognition/nt_mind/mod.rs:
  Line 40: pub use super::nt_mind::reason::*;
  Line 43: pub use super::nt_mind::infrastructure::*;
  Line 46: pub use super::nt_mind_benchmark::*;
```

**Implication:** These create the following public API paths:
- `crate::l5_cognition::nt_mind::reason::*` → actually comes from `nt_mind/nt_mind/reason/`
- `crate::l5_cognition::nt_mind::infrastructure::*` → actually comes from `nt_mind/nt_mind/infrastructure/`
- `crate::l5_cognition::nt_mind::*` (benchmark types) → actually comes from `nt_mind_benchmark.rs`

**Breaking rule:** Moving `nt_mind_benchmark.rs` into a subdirectory would break `pub use super::nt_mind_benchmark::*` unless the re-export is also updated.

### 1.2 neotrix/mod.rs Re-exports

```
pub use crate::l5_cognition::nt_mind::{
    nt_mind_autofixer, nt_mind_background_config, nt_mind_background_loop,
    nt_mind_benchmark, nt_mind_cleanup, nt_mind_distiller, nt_mind_evolution_daemon,
    nt_mind_evolution_loop, nt_mind_hook, nt_mind_knowledge_pipeline, nt_mind_memory,
    nt_mind_skill_engine,
};
```

**Implication:** These 12 modules are publicly re-exported through `crate::neotrix::*`. Moving them changes their crate-level path.

### 1.3 Cross-File `super::` Import Map (Production Code Only)

Files with **production-level** `use super::` imports (excluding `use super::*` in `#[cfg(test)]` blocks):

| Flat File | Imports From (via `super::`) | Target Module |
|-----------|------------------------------|---------------|
| `nt_mind_background_loop/mod.rs` | `nt_mind_cleanup` | CleanupEngine |
| | `nt_mind_evolution_daemon` | EvolutionDaemon |
| | `nt_mind_skill_engine` | SkillEngine |
| | `nt_mind_hook` | HookEvent, MindHookRegistry |
| | `nt_mind_knowledge_pipeline` | KnowledgeAbsorptionPipeline |
| | `nt_mind_background_config` | BackgroundConfig |
| `nt_mind_evolution_daemon` | `nt_mind_autofixer` | AutoFixer |
| | `nt_mind_evolution_loop` | EvolutionLoop |
| `nt_mind_evolution_loop` | `nt_mind_autofixer` | AutoFixer |
| | `nt_mind_self_diagnose` | (types) |
| `nt_mind_self_diagnose` | `nt_mind_autofixer` | AutoFixer |
| | `nt_mind_evolution_loop` | (types) |
| `nt_mind_recovery_verify` | `nt_mind_repair` | (types) |
| `nt_mind_hook` | inner `nt_mind::self_iterating` | SelfIteratingBrain |
| `nt_mind_skill_engine` | `nt_mind_hook` | HookEvent, etc. |
| `nt_mind_benchmark` | inner `nt_mind::memory` | ReasoningBank |
| | inner `nt_mind::` | ReasoningBrain |

### 1.4 Dependency Clusters

**Cluster A: Evolution Loop (tightly coupled)**
```
nt_mind_autofixer ← nt_mind_evolution_loop ← nt_mind_evolution_daemon
                         ↑
                   nt_mind_self_diagnose
```
All 4 files cross-reference each other. Must move as a group.

**Cluster B: Background Loop (hub-and-spoke)**
```
nt_mind_background_loop (hub)
  ├── nt_mind_cleanup
  ├── nt_mind_evolution_daemon
  ├── nt_mind_skill_engine
  ├── nt_mind_hook
  ├── nt_mind_knowledge_pipeline
  └── nt_mind_background_config
```
Background loop imports from 6 flat files. Moving any spoke requires updating background_loop imports.

**Cluster C: Inner nt_mind/nt_mind/ (dense internal mesh)**
The inner `nt_mind/nt_mind/` directory has ~60 files with extensive `super::` imports referencing siblings. These are ALREADY organized in subdirectories (consciousness/, reason/, seal_core/, knowledge/, etc.) and should NOT be restructured — they form a self-contained unit.

### 1.5 Safe-to-Move Files (nt_mind)

**No cross-file `super::` imports (production code):**

| File | Notes |
|------|-------|
| `nt_mind_absorption_registry.rs` | Only `use super::*` in tests |
| `nt_mind_build_runner.rs` | Only `use super::*` in tests |
| `nt_mind_bpco.rs` | Only `use super::*` in tests |
| `nt_mind_gasp.rs` | Only `use super::*` in tests |
| `nt_mind_git_learning.rs` | No super imports at all |
| `nt_mind_jit_agent.rs` | No super imports at all |
| `nt_mind_memory_consolidation.rs` | No super imports at all |
| `nt_mind_new_cleanup.rs` | ORPHAN (not in mod.rs) — can delete |
| `nt_mind_pilot_failure.rs` | No super imports at all |
| `nt_mind_recuris.rs` | Only `use super::*` in tests |
| `nt_mind_research.rs` | No super imports at all |
| `nt_mind_rsi_exam.rs` | Only `use super::*` in tests |
| `nt_mind_seal_enhanced.rs` | No super imports at all |
| `nt_mind_skill_chain.rs` | No super imports at all |
| `nt_mind_wordpecker.rs` | Only `use super::*` in tests |
| `nt_mind_yoyo_evolve.rs` | Only `use super::*` in tests |
| `nt_mind_yoyo_gasp.rs` | Only `use super::*` in tests |
| `nt_mind_yoyo_gasp_site.rs` | Only `use super::*` in tests |
| `nt_mind_yoyobook.rs` | Only `use super::*` in tests |

**CANNOT be moved without updating importers:**

| File | Importers |
|------|-----------|
| `nt_mind_autofixer.rs` | nt_mind_evolution_loop, nt_mind_evolution_daemon, nt_mind_self_diagnose |
| `nt_mind_evolution_loop.rs` | nt_mind_evolution_daemon, nt_mind_background_loop |
| `nt_mind_evolution_daemon.rs` | nt_mind_background_loop |
| `nt_mind_self_diagnose.rs` | nt_mind_evolution_loop, nt_mind_background_loop |
| `nt_mind_background_config.rs` | nt_mind_background_loop |
| `nt_mind_cleanup.rs` | nt_mind_background_loop |
| `nt_mind_hook.rs` | nt_mind_skill_engine, nt_mind_background_loop |
| `nt_mind_skill_engine.rs` | nt_mind_background_loop |
| `nt_mind_knowledge_pipeline.rs` | nt_mind_background_loop |
| `nt_mind_benchmark.rs` | pub use re-export chain in mod.rs |
| `nt_mind_memory.rs` | inner nt_mind uses it |
| `nt_mind_repair.rs` | nt_mind_recovery_verify |
| `nt_mind_recovery_verify.rs` | (depends on nt_mind_repair) |
| `nt_mind_guard.rs` | (has `use super::*` — implicit dependency) |

---

## Part 2: nt_act Analysis

### 2.1 `pub use` Re-export Chains (CRITICAL)

```
l1_action/nt_act/mod.rs:
  Line 76: pub use resource_budget::CostManager;
  Line 77: pub use temporal_continuity::ShotContinuityChecker;
  Line 78: pub use parallel_task::TaskScheduler;
  Line 79: pub use production_orchestrator::BatchProductionManager;
```

**Implication:** These 4 `pub use` create backward-compatibility aliases. Moving these files requires updating the re-export paths.

### 2.2 neotrix/mod.rs Re-exports

```
pub use crate::l1_action::nt_act::{
    nt_act_autonomy, nt_act_code, nt_act_crypto, nt_act_goal,
    nt_act_orchestrator, nt_act_sandbox, nt_act_voice,
};
```

**Implication:** These 7 modules are publicly re-exported through `crate::neotrix::*`.

### 2.3 Cross-File `super::` Import Map (Production Code)

**Cluster A: Crypto (tightly coupled — 18 files)**
```
nt_act_crypto/mod.rs re-exports from:
  chain, evm, token, wallet, wallet_store, cipher, tx, gas,
  dex, bridge, yields, portfolio, monitor, airdrop,
  security, self_evolve, collector, opportunity

Internal imports:
  chain ← evm, token, wallet, tx, gas, dex, bridge, yields,
          portfolio, monitor, airdrop, security, collector, opportunity
  evm ← token, dex, security
  wallet ← wallet_store
  cipher ← wallet_store
  opportunity ← collector, self_evolve
```

**Cluster B: Orchestrator (tightly coupled — 10 files)**
```
nt_act_orchestrator/mod.rs re-exports from:
  planner, types, worker, state_graph, task_state_dag,
  pm_workflow, critic, harness_scaffold, group_integration_test,
  pm_integration_test

Internal imports:
  types ← planner, worker
  state_graph ← task_state_dag
  planner ← pm_integration_test
```

**Cluster C: Code (tightly coupled — 10 files)**
```
nt_act_code/mod.rs re-exports from:
  ast_searcher, code_writer, edit_history, pattern_extractor,
  pipeline_autofixer, recipe_refactor, safe_applier,
  semantic_entropy, template_registry

Internal imports:
  edit_history ← safe_applier, pattern_extractor
  semantic_entropy ← safe_applier, code_writer
  template_registry ← code_writer
```

**Cluster D: Goal (tightly coupled — 7 files)**
```
nt_act_goal/mod.rs re-exports from:
  behavioral_verifier, conflict_resolver, coverage_analyzer,
  goal_generator, neotrix_bench, rl_feedback, test_writer

Internal imports:
  goal_generator ← conflict_resolver, meta_goal_generator (flat)
  behavioral_verifier ← rl_feedback
```

**Cluster E: Voice (tightly coupled — 3 files)**
```
nt_act_voice/mod.rs re-exports from:
  command, transcribe, trigger

Internal imports:
  command ← trigger
  (VoiceError, VoiceSample) ← transcribe, trigger
```

**Cluster F: Cross-cluster dependencies**
```
nt_act flat files → nt_act_code:
  pipeline_autofixer → code_writer, edit_history, safe_applier
  test_writer → code_writer (CodeGenResult)

nt_act flat files → nt_act_goal:
  rl_feedback → behavioral_verifier
  knowledge_distiller → rl_feedback

nt_act flat files → nt_act_autonomy:
  oracle_gate → awareness_monitor
  arch_optimizer → awareness_monitor
  meta_goal_generator → trend_analyzer, goal_generator
```

### 2.4 Safe-to-Move Files (nt_act)

**No cross-file `super::` imports (production code):**

| File | Notes |
|------|-------|
| `nt_act_3d_dev.rs` | No super imports |
| `nt_act_3d_render.rs` | No super imports |
| `nt_act_action_cache.rs` | No super imports |
| `nt_act_ai_assistant.rs` | No super imports |
| `nt_act_cache.rs` | No super imports |
| `nt_act_circuit_breaker.rs` | No super imports |
| `nt_act_disk_guard.rs` | No super imports |
| `nt_act_eventbus.rs` | No super imports |
| `nt_act_media.rs` | No super imports |
| `nt_act_rate_limiter.rs` | No super imports |
| `nt_act_sandbox.rs` | Uses `nt_act_disk_guard` via `crate::` (safe) |
| `nt_act_seo.rs` | No super imports |
| `nt_act_security.rs` | No super imports |
| `nt_act_types.rs` | No super imports |
| `nt_act_workflow.rs` | No super imports |
| `audio_orchestrator.rs` | No super imports |
| `checkpoint_persistence.rs` | No super imports |
| `cost_tracker.rs` | No super imports |
| `error_classifier.rs` | No super imports |
| `gpu_scheduler.rs` | No super imports |
| `model_router.rs` | No super imports |
| `multi_region_scheduler.rs` | No super imports |
| `nt_trade_finance_compliance.rs` | No super imports |
| `nt_trade_full_cycle.rs` | No super imports |
| `nt_trade_mock_adapters.rs` | No super imports |
| `nt_trade_orchestrator.rs` | No super imports |
| `nt_trade_production_logistics.rs` | No super imports |
| `nt_trade_quote_negotiation.rs` | No super imports |
| `operator_runbook.rs` | No super imports |
| `observability_stack.rs` | No super imports |
| `pipeline_checkpointing.rs` | No super imports |
| `production_pipeline.rs` | No super imports |
| `provider_migration_router.rs` | No super imports |
| `publish_gateway.rs` | No super imports |
| `recipe_refactor.rs` | No super imports |
| `video_job_pipeline.rs` | No super imports |
| `video_object_storage.rs` | No super imports |
| `video_spec.rs` | No super imports |
| `video_stitcher.rs` | No super imports |
| `yagni_ladder.rs` | No super imports |

**CANNOT be moved without updating importers:**

| File | Importers |
|------|-----------|
| `chain.rs` | 14 crypto files |
| `evm.rs` | 6 crypto files |
| `token.rs` | 3 crypto files |
| `wallet.rs` | wallet_store, tx |
| `cipher.rs` | wallet_store |
| `wallet_store.rs` | (depends on cipher, wallet) |
| `opportunity.rs` | collector, self_evolve |
| `collector.rs` | (depends on chain, opportunity) |
| `edit_history.rs` | safe_applier, pattern_extractor |
| `semantic_entropy.rs` | safe_applier, code_writer |
| `safe_applier.rs` | (depends on edit_history, semantic_entropy) |
| `code_writer.rs` | pipeline_autofixer, test_writer |
| `template_registry.rs` | code_writer |
| `state_graph.rs` | task_state_dag |
| `task_state_dag.rs` | (depends on state_graph) |
| `planner.rs` | pm_integration_test |
| `types.rs` | planner, worker |
| `goal_generator.rs` | conflict_resolver, meta_goal_generator |
| `behavioral_verifier.rs` | rl_feedback |
| `command.rs` | trigger |
| `awareness_monitor.rs` | oracle_gate, arch_optimizer |
| `trend_analyzer.rs` | meta_goal_generator |

---

## Part 3: Safe Restructuring Strategy

### Phase 0: Pre-flight (Before Any Moves)

```bash
# Establish clean baseline
cargo clean && cargo check -p neotrix --lib 2>&1 | tee /tmp/baseline-errors.txt
# Record error count (78 pre-existing)
```

### Phase 1: Delete Orphans (Low Risk)

| Action | File | Risk |
|--------|------|------|
| Delete | `nt_mind_new_cleanup.rs` | None — not in mod.rs, dead code |

**Verify:** `cargo check -p neotrix --lib`

### Phase 2: Move Isolated nt_mind Flat Files (Batch of 5-8)

**Target subdirectory:** `nt_mind/mind_modules/` (new)

**Batch 2a — Research & Learning cluster:**
```
nt_mind_research.rs      → mind_modules/research.rs
nt_mind_git_learning.rs  → mind_modules/git_learning.rs
nt_mind_pilot_failure.rs → mind_modules/pilot_failure.rs
nt_mind_jit_agent.rs     → mind_modules/jit_agent.rs
nt_mind_skill_chain.rs   → mind_modules/skill_chain.rs
```

**Update mod.rs:**
```rust
pub mod mind_modules;  // ADD

// Replace individual mods:
// pub mod nt_mind_research;  → REMOVE (now in mind_modules)
// ... etc
```

**Batch 2b — Auxiliary cluster:**
```
nt_mind_absorption_registry.rs → mind_modules/absorption_registry.rs
nt_mind_build_runner.rs        → mind_modules/build_runner.rs
nt_mind_bpco.rs                → mind_modules/bpco.rs
nt_mind_memory_consolidation.rs → mind_modules/memory_consolidation.rs
nt_mind_recuris.rs             → mind_modules/recuris.rs
nt_mind_rsi_exam.rs            → mind_modules/rsi_exam.rs
nt_mind_seal_enhanced.rs       → mind_modules/seal_enhanced.rs
```

**Batch 2c — Yoyo cluster:**
```
nt_mind_yoyobook.rs       → mind_modules/yoyobook.rs
nt_mind_yoyo_evolve.rs    → mind_modules/yoyo_evolve.rs
nt_mind_yoyo_gasp.rs      → mind_modules/yoyo_gasp.rs
nt_mind_yoyo_gasp_site.rs → mind_modules/yoyo_gasp_site.rs
nt_mind_gasp.rs           → mind_modules/gasp.rs
nt_mind_wordpecker.rs     → mind_modules/wordpecker.rs
```

**After each batch:**
```bash
cargo check -p neotrix --lib 2>&1 | grep "^error" | wc -l
# Must remain at 78 (pre-existing count)
```

### Phase 3: Move Isolated nt_act Flat Files (Batch of 5-8)

**Target subdirectory:** `nt_act/actions/` (new)

**Batch 3a — 3D & Media cluster:**
```
nt_act_3d_dev.rs     → actions/3d_dev.rs
nt_act_3d_render.rs  → actions/3d_render.rs
nt_act_media.rs      → actions/media.rs
nt_act_seo.rs        → actions/seo.rs
```

**Batch 3b — Infrastructure cluster:**
```
nt_act_cache.rs           → actions/cache.rs
nt_act_circuit_breaker.rs → actions/circuit_breaker.rs
nt_act_rate_limiter.rs    → actions/rate_limiter.rs
nt_act_eventbus.rs        → actions/eventbus.rs
nt_act_disk_guard.rs      → actions/disk_guard.rs
```

**Batch 3c — Security & Sandbox:**
```
nt_act_sandbox.rs   → actions/sandbox.rs
nt_act_security.rs  → actions/security.rs
```

**Batch 3d — Production infrastructure:**
```
observability_stack.rs    → actions/observability_stack.rs
cost_tracker.rs           → actions/cost_tracker.rs
error_classifier.rs       → actions/error_classifier.rs
gpu_scheduler.rs          → actions/gpu_scheduler.rs
model_router.rs           → actions/model_router.rs
multi_region_scheduler.rs → actions/multi_region_scheduler.rs
```

**Batch 3e — Video pipeline:**
```
video_job_pipeline.rs  → actions/video_job_pipeline.rs
video_object_storage.rs → actions/video_object_storage.rs
video_spec.rs          → actions/video_spec.rs
video_stitcher.rs      → actions/video_stitcher.rs
audio_orchestrator.rs  → actions/audio_orchestrator.rs
```

**Batch 3f — Remaining isolated:**
```
nt_act_action_cache.rs     → actions/action_cache.rs
nt_act_ai_assistant.rs     → actions/ai_assistant.rs
nt_act_workflow.rs         → actions/workflow.rs
checkpoint_persistence.rs  → actions/checkpoint_persistence.rs
operator_runbook.rs        → actions/operator_runbook.rs
publish_gateway.rs         → actions/publish_gateway.rs
provider_migration_router.rs → actions/provider_migration_router.rs
recipe_refactor.rs         → actions/recipe_refactor.rs
yagni_ladder.rs            → actions/yagni_ladder.rs
production_pipeline.rs     → actions/production_pipeline.rs
```

**Batch 3g — Trade cluster (all isolated):**
```
nt_trade_finance_compliance.rs → actions/trade/finance_compliance.rs
nt_trade_full_cycle.rs         → actions/trade/full_cycle.rs
nt_trade_mock_adapters.rs      → actions/trade/mock_adapters.rs
nt_trade_orchestrator.rs       → actions/trade/orchestrator.rs
nt_trade_production_logistics.rs → actions/trade/production_logistics.rs
nt_trade_quote_negotiation.rs  → actions/trade/quote_negotiation.rs
```

**Batch 3h — Backward-compat aliases (MOVE LAST):**
```
resource_budget.rs       → actions/resource_budget.rs
temporal_continuity.rs   → actions/temporal_continuity.rs
parallel_task.rs         → actions/parallel_task.rs
production_orchestrator.rs → actions/production_orchestrator.rs
```

**Update mod.rs re-exports:**
```rust
pub use actions::resource_budget::CostManager;
pub use actions::temporal_continuity::ShotContinuityChecker;
pub use actions::parallel_task::TaskScheduler;
pub use actions::production_orchestrator::BatchProductionManager;
```

### Phase 4: Move Clustered Files (Coordinated Groups)

**DO NOT attempt until Phases 1-3 are stable.**

**Cluster moves (all-or-nothing):**

| Cluster | Files | Target |
|---------|-------|--------|
| Crypto | chain, evm, token, wallet, cipher, wallet_store, tx, gas, dex, bridge, yields, portfolio, monitor, airdrop, security, self_evolve, collector, opportunity | Already in `nt_act_crypto/` — NO MOVE NEEDED |
| Orchestrator | planner, types, worker, state_graph, task_state_dag, pm_workflow, critic, harness_scaffold | Already in `nt_act_orchestrator/` — NO MOVE NEEDED |
| Code | edit_history, semantic_entropy, safe_applier, code_writer, template_registry, pattern_extractor, ast_searcher, pipeline_autofixer, recipe_refactor | Already in `nt_act_code/` — NO MOVE NEEDED |
| Goal | goal_generator, behavioral_verifier, conflict_resolver, coverage_analyzer, rl_feedback, test_writer, neotrix_bench | Already in `nt_act_goal/` — NO MOVE NEEDED |
| Voice | command, transcribe, trigger | Already in `nt_act_voice/` — NO MOVE NEEDED |

**Key insight:** The clustered files are ALREADY in subdirectories. The flat files at the nt_act root level that reference these clusters do so via `super::` (which works because they're siblings) or `crate::` (which works regardless of location).

### Phase 5: Update neotrix/mod.rs Re-exports

If modules move into subdirectories, update the re-export paths:

```rust
// Before:
pub use crate::l5_cognition::nt_mind::{
    nt_mind_autofixer, nt_mind_background_loop, ...
};

// After (if nt_mind_autofixer moves to mind_modules/):
pub use crate::l5_cognition::nt_mind::mind_modules::{
    nt_mind_autofixer, ...
};
```

**Important:** Only update re-exports for modules that actually moved.

---

## Part 4: Risk Assessment

### Risk Matrix

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| `pub use` chain breaks | HIGH — downstream code won't compile | MEDIUM | Verify neotrix/mod.rs consumers after each move |
| `super::` import breaks | HIGH — file won't compile | LOW (only for clustered files) | Never move clustered files individually |
| `crate::` path breaks | LOW — absolute paths, less common | LOW | grep for `crate::l5_cognition::nt_mind::nt_mind_*` patterns |
| Test-only `super::*` breaks | NONE — tests don't affect lib | N/A | Ignore test compilation until final verification |
| Re-export ambiguity | MEDIUM — two paths to same type | LOW | Prefer `crate::` paths over re-exports |

### Files Explicitly DO NOT MOVE

| File | Reason |
|------|--------|
| `nt_mind_benchmark.rs` | `pub use super::nt_mind_benchmark::*` in mod.rs |
| `nt_mind_autofixer.rs` | Hub of Evolution Loop cluster |
| `nt_mind_evolution_loop.rs` | Hub of Evolution Loop cluster |
| `nt_mind_cleanup.rs` | Imported by background_loop |
| `nt_mind_hook.rs` | Imported by skill_engine + background_loop |
| `nt_mind_skill_engine.rs` | Imported by background_loop |
| `nt_mind_knowledge_pipeline.rs` | Imported by background_loop |
| `nt_mind_background_config.rs` | Imported by background_loop |
| `nt_mind_evolution_daemon.rs` | Part of evolution cluster |
| `nt_mind_self_diagnose.rs` | Part of evolution cluster |
| All nt_act_crypto/* files | Already clustered |
| All nt_act_orchestrator/* files | Already clustered |
| All nt_act_code/* files | Already clustered |
| All nt_act_goal/* files | Already clustered |
| All nt_act_voice/* files | Already clustered |

---

## Part 5: Execution Checklist

### Pre-flight
- [ ] `cargo clean && cargo check -p neotrix --lib 2>&1 | tee /tmp/baseline-errors.txt`
- [ ] Record baseline error count: ___
- [ ] `git stash` any uncommitted changes

### Phase 1: Orphans
- [ ] Delete `nt_mind_new_cleanup.rs`
- [ ] `cargo check -p neotrix --lib` — errors: ___

### Phase 2: nt_mind safe moves (batches of 5-8)
- [ ] Batch 2a: Create `mind_modules/`, move 5 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 2b: Move 7 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 2c: Move 6 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___

### Phase 3: nt_act safe moves (batches of 5-8)
- [ ] Batch 3a: Create `actions/`, move 4 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3b: Move 5 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3c: Move 2 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3d: Move 6 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3e: Move 5 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3f: Move 9 files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3g: Move 6 trade files, update mod.rs
- [ ] `cargo check -p neotrix --lib` — errors: ___
- [ ] Batch 3h: Move 4 backward-compat files, update re-exports
- [ ] `cargo check -p neotrix --lib` — errors: ___

### Phase 5: Final verification
- [ ] `cargo test -p neotrix --lib` — tests pass
- [ ] `cargo check --all-targets -p neotrix` — full check
- [ ] `git diff --stat` — review all changes
- [ ] Commit with descriptive message

---

## Appendix A: File Counts

| Category | nt_mind | nt_act |
|----------|---------|--------|
| Total flat files on disk | 36 | ~100 |
| Declared in mod.rs | 29 + 3 re-export aliases | ~40 + 4 re-exports |
| Orphan files (dead) | 1 (nt_mind_new_cleanup) | ~60 (in subdirectories) |
| Safe to move (isolated) | 19 | ~40 |
| Cannot move (clustered) | 10 (evolution + background) | 0 (clusters already in subdirs) |
| Existing subdirectories | 8 | 8 |

## Appendix B: Public API Surface

External consumers access these modules through:
1. `crate::neotrix::*` — via neotrix/mod.rs re-exports
2. `crate::l5_cognition::nt_mind::*` — direct
3. `crate::l1_action::nt_act::*` — direct
4. `crate::l5_cognition::nt_mind::reason::*` — via mod.rs pub use re-export
5. `crate::l5_cognition::nt_mind::infrastructure::*` — via mod.rs pub use re-export

Moving files changes paths (2), (3), (4), (5). Path (1) must be updated if re-exports change.
