# NeoTrix Media Source — Refactoring Guide

## Overview

This guide defines a 5-phase refactoring plan for the `nt_world_media_source` module. Each phase is independently shippable and produces a compiling, tested codebase.

**Prerequisites**:
- Run `cargo clean` before Phase 1 (build cache distrust, R-P9)
- Run `cargo check --all-targets -p neotrix` after each phase
- Re-read every edited file after write (R-P16)
- No `#![forbid(unsafe_code)]` violations (R-P1)

---

## Phase 1: Consolidate Redundant Modules

**Goal**: Merge 5 circuit breaker implementations, 3+ retry implementations, and 5 caching modules into canonical locations.

**Duration**: 1-2 sessions
**Risk**: Medium (interface changes)
**Constellation**: C1 → C2

### 1.1 Circuit Breaker Consolidation

**Current state** (5 files):
```
l1_action/nt_act/nt_act_circuit_breaker.rs
l1_action/nt_infra_breaker.rs
l1_action/nt_io/nt_io_provider/circuit_breaker.rs
unified/layers/meta/nt_repair/circuit_breaker.rs
unified/layers/perception/nt_world/nt_world_media_source/reliability/circuit_breaker.rs
```

**Target** (1 file):
```
l3_embodiment/nt_shield/reliability/circuit_breaker.rs
```

**Steps**:
1. Read all 5 files, identify the canonical implementation (the one with the most features/test coverage)
2. Move canonical implementation to `l3_embodiment/nt_shield/reliability/circuit_breaker.rs`
3. Create thin re-export modules at original locations:
   ```rust
   // l1_action/nt_act/nt_act_circuit_breaker.rs
   pub use crate::l3_embodiment::nt_shield::reliability::circuit_breaker::*;
   ```
4. Update all import paths to point to canonical location
5. Delete the re-export shims after all consumers are migrated
6. Run `cargo check --all-targets` — fix any orphan rule violations

**Verification**:
```sh
cargo test -p neotrix --lib -- circuit_breaker
cargo check --all-targets -p neotrix
```

### 1.2 Retry Consolidation

**Current state** (3+ implementations):
```
media_source/reliability/retry.rs
nt_world_crawl/resilient.rs (contains retry logic)
nt_io_provider retry wrappers
```

**Target** (1 file):
```
l3_embodiment/nt_shield/reliability/retry.rs
```

**Steps**:
1. Read `media_source/reliability/retry.rs` — identify core retry types (`RetryPolicy`, `RetryExecutor`)
2. Read `nt_world_crawl/resilient.rs` — extract pure retry logic, leave crawl-specific logic in place
3. Merge into canonical `retry.rs`
4. Update `nt_world_crawl/resilient.rs` to import from canonical
5. Update provider retry wrappers to use canonical types

### 1.3 Caching Consolidation

**Current state** (5 files):
```
media_source/media_cache/
media_source/engine/cache.rs
media_source/cost/cache_optimization.rs
core/nt_core_cache.rs
core/nt_core_deploy_cache.rs
```

**Target** (1 module):
```
l1_action/nt_memory/cache/
├── mod.rs              # Cache trait + registry
├── multi_level.rs      # L1/L2/L3 hierarchy (from media_cache/)
├── consistency.rs      # Invalidation (from media_cache/)
├── warmup.rs           # Warming (from media_cache/)
└── deploy_cache.rs     # Deploy-specific (from nt_core_deploy_cache)
```

**Steps**:
1. Create `l1_action/nt_memory/cache/mod.rs` with `CacheBackend` trait
2. Move `media_cache/multi_level.rs`, `consistency.rs`, `warmup.rs` into new location
3. Move `nt_core_deploy_cache.rs` content into `deploy_cache.rs`
4. Delete `engine/cache.rs`, `cost/cache_optimization.rs`, `nt_core_cache.rs`
5. Update all imports

### 1.4 Rate Limiter Consolidation

**Current state** (3 files):
```
media_source/security/rate_limit.rs
media_source/engine/ratelimit.rs
l1_action/nt_act/nt_act_rate_limiter.rs
```

**Target** (1 file):
```
l3_embodiment/nt_shield/reliability/rate_limiter.rs
```

### Phase 1 Exit Criteria

```sh
cargo clean && cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
```

Files deleted: ~12
Files created: 1 (canonical module)
Import rewrites: ~50-80 call sites

---

## Phase 2: Flatten Hierarchy

**Goal**: Reduce `core/` from 122 top-level entries to ~30 by grouping related modules.

**Duration**: 1-2 sessions
**Risk**: Low (directory moves only, no logic changes)
**Constellation**: C0 → C1

### 2.1 Group `core/nt_core_*` into Subdirectories

**Current** (scattered):
```
core/
├── nt_core_absorb/
├── nt_core_accessor.rs
├── nt_core_agent_patterns.rs
├── nt_core_answer_engine.rs
├── nt_core_aura/
├── nt_core_aware/
├── nt_core_axiom_tree.rs
├── nt_core_bank/
├── ... (90+ items)
```

**Target** (grouped):
```
core/
├── consciousness/           # Consciousness-related
│   ├── nt_core_consciousness_core.rs
│   ├── nt_core_consciousness_tree/
│   ├── nt_core_consciousness/
│   ├── nt_core_cad_consciousness.rs
│   ├── nt_core_iit_phi.rs
│   └── mod.rs
├── knowledge/               # Knowledge representation
│   ├── nt_core_knowledge/
│   ├── nt_core_kb_primitives.rs
│   ├── nt_core_kb_types.rs
│   ├── nt_core_vector_store/
│   ├── nt_core_embed.rs
│   └── mod.rs
├── reasoning/               # Reasoning engine
│   ├── nt_core_e8/
│   ├── nt_core_hcube/
│   ├── nt_core_hex.rs
│   ├── nt_core_reasoning.rs
│   ├── nt_core_cot_generator.rs
│   └── mod.rs
├── self/                    # Self-model & evolution
│   ├── nt_core_self/
│   ├── nt_core_self_model.rs
│   ├── nt_core_self_review/
│   ├── nt_core_self_test.rs
│   ├── nt_core_self_constitution.rs
│   └── mod.rs
├── infrastructure/          # Infrastructure primitives
│   ├── nt_core_cache.rs
│   ├── nt_core_event.rs
│   ├── nt_core_error.rs
│   ├── nt_core_state.rs
│   ├── nt_core_router.rs
│   ├── nt_core_scheduler/
│   └── mod.rs
├── orchestration/           # Orchestration & dispatch
│   ├── nt_core_task_dispatcher.rs
│   ├── nt_core_subagent.rs
│   ├── nt_core_dispatch.rs
│   ├── nt_core_orchestration_failure_taxonomy.rs
│   └── mod.rs
├── integration/             # External integrations
│   ├── nt_core_llm.rs
│   ├── nt_core_mcp.rs
│   ├── nt_core_deploy.rs
│   ├── nt_core_ws.rs
│   └── mod.rs
├── l0_substrate/            # Layer 0 (keep as-is)
├── l1_body/                 # Layer 1 (keep as-is)
├── ... (l* directories)
├── energy_core/             # Energy (keep as-is)
└── nt_game/                 # Game (keep as-is)
```

### 2.2 Module Move Commands

For each group, use `git mv` to preserve history:

```sh
# Consciousness group
mkdir -p core/consciousness
git mv core/nt_core_consciousness_core.rs core/consciousness/
git mv core/nt_core_consciousness_tree core/consciousness/
git mv core/nt_core_consciousness core/consciousness/
git mv core/nt_core_cad_consciousness.rs core/consciousness/
git mv core/nt_core_iit_phi.rs core/consciousness/

# Create mod.rs for each group
# Update parent mod.rs to use `pub mod consciousness;`
```

### 2.3 Flatten `l1_action/nt_act/`

The `nt_act` directory has 100+ files at the top level. Group into:

```
nt_act/
├── autonomy/        # Autonomous agents (existing nt_act_autonomy/)
├── code/            # Code operations (existing nt_act_code/)
├── crypto/          # Crypto operations (existing nt_act_crypto/)
├── goal/            # Goal system (existing nt_act_goal/)
├── orchestrator/    # Orchestration (existing nt_act_orchestrator/)
├── voice/           # Voice operations (existing nt_act_voice/)
├── media/           # Media operations (new grouping)
│   ├── nt_act_media.rs
│   ├── video_job_pipeline.rs
│   ├── video_stitcher.rs
│   ├── video_spec.rs
│   └── audio_orchestrator.rs
├── workflow/        # Workflow engine
│   ├── nt_act_workflow.rs
│   └── pipeline_checkpointing.rs
├── monitoring/      # Monitoring & observability
│   ├── observability_stack.rs
│   ├── monitor.rs
│   └── operator_runbook.rs
├── security/        # Security actions
│   ├── nt_act_security.rs
│   ├── nt_act_sandbox.rs
│   └── nt_act_disk_guard.rs
└── primitives/      # Small utility modules
    ├── nt_act_types.rs
    ├── nt_act_eventbus.rs
    └── error_classifier.rs
```

### Phase 2 Exit Criteria

```sh
cargo clean && cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
```

Files moved: ~100+
Files deleted: 0
`mod.rs` updates: ~20

---

## Phase 3: Fix Cross-Domain Alignment

**Goal**: Move security, governance, and compliance modules to their correct domain owners.

**Duration**: 1 session
**Risk**: Medium (domain boundary changes)
**Constellation**: C1 → C2

### 3.1 Security Consolidation

**Move**:
```
media_source/security/         → l3_embodiment/nt_shield/security/
media_source/integration/security_audit.rs → l3_embodiment/nt_shield/audit.rs
l1_action/nt_act/nt_act_security.rs → l3_embodiment/nt_shield/security/act_security.rs
l1_action/nt_act/nt_act_sandbox.rs → l3_embodiment/nt_shield/sandbox.rs
l1_action/nt_act/nt_act_disk_guard.rs → l3_embodiment/nt_shield/disk_guard.rs
```

**Keep in nt_act** (action-specific security):
- `nt_act_crypto/security.rs` — crypto-specific security

### 3.2 Governance Consolidation

**Move**:
```
media_source/governance/       → l6_meta/nt_governance/
media_source/compliance/       → l6_meta/nt_governance/compliance/
```

**Update** `l6_meta/nt_governance/mod.rs` to re-export:
```rust
pub mod audit_trail;
pub mod classification;
pub mod compliance;  // GDPR, HIPAA, SOC2
pub mod retention;
```

### 3.3 Observability Consolidation

**Move**:
```
media_source/observability/    → l1_action/nt_act/observability/
l1_action/nt_infra_tracing.rs  → l1_action/nt_act/observability/tracing.rs
l1_action/nt_act/observability_stack.rs → l1_action/nt_act/observability/stack.rs
```

### 3.4 Evolution Module Relocation

The `media_source/evolution/` directory contains SEAL-related functionality that belongs in NT-MIND:

**Move**:
```
media_source/evolution/        → l5_cognition/nt_mind/media_evolution/
media_source/ecosystem/        → l5_cognition/nt_mind/media_ecosystem/
```

### Phase 3 Exit Criteria

```sh
cargo clean && cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
```

Verify: No circular dependencies
```sh
cargo udeps -p neotrix  # if available, or manual import graph check
```

---

## Phase 4: Extract Cross-Cutting Concerns

**Goal**: Create shared infrastructure modules for logging, metrics, tracing, and config.

**Duration**: 1 session
**Risk**: Low (new modules, no deletions)
**Constellation**: C1 → C2

### 4.1 Create Observability Module

**New location**:
```
l1_action/nt_act/observability/
├── mod.rs              # Re-exports
├── logging.rs          # Structured logging facade
├── metrics.rs          # Metrics collection (Prometheus/OpenTelemetry)
├── tracing.rs          # Distributed tracing (from nt_infra_tracing)
├── alerting.rs         # Alert rules (from media_source/observability)
└── log_aggregation.rs  # Log aggregation (from media_source/observability)
```

### 4.2 Create Config Module

**New location**:
```
crates/neotrix-types/src/config/
├── mod.rs              # Config trait
├── media_config.rs     # Media source config
├── engine_config.rs    # Engine config
├── deploy_config.rs    # Deployment config
└── validation.rs       # Config validation
```

### 4.3 Create Metrics Module

**Move**:
```
media_source/observability/metrics.rs → l1_action/nt_act/observability/metrics.rs
media_source/observability/custom_metrics.rs → l1_action/nt_act/observability/custom_metrics.rs
media_source/engine/health.rs → l3_embodiment/nt_shield/health.rs
core/nt_core_heartbeat.rs → l3_embodiment/nt_shield/heartbeat.rs
core/nt_core_telemetry.rs → l1_action/nt_act/observability/telemetry.rs
```

### Phase 4 Exit Criteria

```sh
cargo clean && cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
```

New modules: 3 (observability, config, metrics)
Re-exports: Preserve backward compatibility via `pub use` in old locations

---

## Phase 5: Create Domain Boundaries

**Goal**: Enforce module ownership, eliminate circular dependencies, define clear trait interfaces.

**Duration**: 1-2 sessions
**Risk**: High (interface changes)
**Constellation**: C2 → C3

### 5.1 Define Layer Traits

Each layer exposes only trait interfaces:

```rust
// l2_perception/nt_world/traits.rs
pub trait MediaProvider: Send + Sync {
    async fn fetch(&self, query: &MediaQuery) -> Result<MediaResult>;
    fn supports(&self, media_type: MediaType) -> bool;
    fn name(&self) -> &str;
}

// l3_embodiment/nt_shield/traits.rs
pub trait ReliabilityPolicy: Send + Sync {
    async fn execute_with_retry<F, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Future<Output = Result<T>> + Send;
    fn circuit_state(&self) -> CircuitState;
}
```

### 5.2 Eliminate Circular Dependencies

**Problem**: `media_source/ecosystem/shield_integration.rs` → `nt_shield` → `nt_world` → `media_source`

**Solution**: Introduce event-based decoupling:

```
nt_world_media_source
    ↓ (publishes MediaIngested event)
EventBus
    ↓ (subscribes)
nt_shield (consumes event, no direct import from nt_world)
```

### 5.3 Module Boundary Enforcement

Add to `Cargo.toml` (when workspace-level enforcement is available):

```toml
[dependencies]
# L2 modules may import L1
nt_world = { path = "../l2_perception/nt_world" }

# L3 modules may import L1, L2
nt_shield = { path = "../l3_embodiment/nt_shield" }

# L5 modules may import L1, L2 (via traits)
nt_mind = { path = "../l5_cognition/nt_mind" }

# L6 modules may import all (via traits)
nt_meta = { path = "../l6_meta/nt_meta" }
```

### 5.4 Delete `unified/` Directory

After all migrations are complete:

```sh
# Verify unified/ is not imported by anything
grep -r "unified" neotrix-core/src/ --include="*.rs" | grep -v "// "
# If clean:
rm -rf neotrix-core/src/unified/
```

**Pre-requisite**: All 1,603 files in `unified/` must be verified as either:
1. Already present in canonical layer directories, or
2. Contain unique logic that has been migrated

### Phase 5 Exit Criteria

```sh
cargo clean && cargo check --all-targets -p neotrix
cargo test -p neotrix --lib
```

Verify no cycles:
```sh
# Manual: check that no L2 module imports L3, no L1 imports L2, etc.
grep -rn "use crate::l[2-6]" neotrix-core/src/l1_action/ --include="*.rs"
# Should return empty
```

---

## Summary: Phase Dependencies

```
Phase 1 (Consolidate) ──→ Phase 2 (Flatten) ──→ Phase 3 (Cross-Domain)
                                                        │
                                                        ▼
                                                 Phase 4 (Cross-Cutting)
                                                        │
                                                        ▼
                                                 Phase 5 (Boundaries)
```

**Parallelizable**: Phase 2 and Phase 4 can run in parallel after Phase 1 completes.

**Rollback**: Each phase is independently revertable via `git revert`.

**Total estimated effort**: 5-8 sessions.

---

## Appendix: File Count Tracker

| Phase | Files Created | Files Deleted | Files Moved | Net Delta |
|-------|--------------|---------------|-------------|-----------|
| Phase 1 | 1 | ~12 | ~80 | -11 |
| Phase 2 | ~10 (mod.rs) | 0 | ~100 | +10 |
| Phase 3 | 0 | ~8 | ~30 | -8 |
| Phase 4 | 3 | ~5 | ~10 | -2 |
| Phase 5 | ~5 (traits) | 1,603 | 0 | -1,598 |
| **Total** | ~19 | ~1,628 | ~220 | **-1,609** |
