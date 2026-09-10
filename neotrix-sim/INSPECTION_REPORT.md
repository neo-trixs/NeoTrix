# neotrix-sim Phase 1-2 Inspection Report

**Date**: 2026-09-10
**Inspector**: Auto-inspection agent
**Scope**: Full codebase scan after Phase 1-2 refactoring
**Crate**: `neotrix-sim` (lib + lib test)
**Test Result**: 248 passed, 0 failed

---

## 1. Compilation Warnings (24 total)

### 1.1 Unnecessary Parentheses / Irrefutable Pattern

| # | File:Line | Warning | Fix |
|---|-----------|---------|-----|
| W1 | `agents/graph_memory.rs:127` | `while let ((id, depth)) = queue.pop_front()?` — unnecessary parentheses around pattern | Remove outer parens: `while let (id, depth) = queue.pop_front()?` |
| W2 | `agents/graph_memory.rs:127` | Irrefutable `while let` pattern — loop will never exit naturally | Replace with `loop { let (id, depth) = queue.pop_front()?; ... }` and add explicit break/return |

### 1.2 Unused Imports (10 locations, 14 items)

| # | File:Line | Unused Items | Fix |
|---|-----------|--------------|-----|
| W3 | `environment/gpu_grid.rs:1` | `std::collections::HashMap` | Remove import |
| W4 | `consciousness/llm_hooks.rs:1` | `serde::{Serialize, Deserialize}` | Remove import (types not derived) |
| W5 | `evolution/fitness_landscape.rs:7` | `std::collections::HashMap` | Remove import |
| W6 | `world_sim.rs:4` | `SimTime`, `Season` | Remove from use statement |
| W7 | `world_sim.rs:20` | `ConsciousnessState` | Remove from use statement |
| W8 | `world_sim.rs:21` | `GlobalCoherence` | Remove from use statement |
| W9 | `world_sim.rs:22` | `ConvergenceState` | Remove from use statement |
| W10 | `world_sim.rs:25` | `SelectionResult` | Remove from use statement |
| W11 | `world_sim.rs:30` | `Inventory`, `ResourceType`, `TradeOffer` | Remove from use statement |
| W12 | `bridge/host_bridge.rs:4` | `crate::world_sim::EvolutionRecord` | Remove import (only used in tests, which import it directly) |

### 1.3 Unused Variables (8 locations)

| # | File:Line | Variable | Suggested Fix |
|---|-----------|----------|---------------|
| W13 | `agents/sim_agent.rs:231` | `dt` in `update_needs()` | Prefix with `_dt` or remove parameter |
| W14 | `consciousness/emergence_detector.rs:115` | `relationships` parameter | Prefix with `_relationships` |
| W15 | `consciousness/phi_bridge.rs:122` | `agent_id` parameter | Prefix with `_agent_id` |
| W16 | `evolution/speciation.rs:89` | `n` (computed but unused) | Prefix with `_n` or remove |
| W17 | `world_sim.rs:139` | `x` in biome generation closure | Prefix with `_x` |
| W18 | `world_sim.rs:694` | `message` in Talk action match | Use `message: _` or prefix |
| W19 | `world_sim.rs:524` | `has_rels` computed but unused in decide_action | Prefix with `_has_rels` or remove |
| W20 | `agents/action_awareness.rs:363` (test) | `mut` not needed on `agent` | Remove `mut` |
| W21 | `agents/reflection.rs:210` (test) | `mut` not needed on `engine` | Remove `mut` |

### 1.4 Dead Code (3 findings)

| # | File:Line | Item | Verdict |
|---|-----------|------|---------|
| D1 | `agents/personality_drift.rs:164` | `fn signal_achievement(v: f32)` | Test-only helper. **Truly dead in production** — remove from non-test code or move to `#[cfg(test)]` block. |
| D2 | `environment/terrain/heightmap.rs:11` | `ValueNoise::seed` field | Set in constructor but never read by `sample()` or `fbm()`. Used for LCG seeding only. **Truly dead** — remove field or prefix with `_` |
| D3 | `consciousness/dual_representation.rs:28` | `DualRepresentation::embedding_size` | Set in constructor but never read. The `generate_embedding()` method hardcodes `[f32; 16]`. **Should be wired in** — use `embedding_size` in `generate_embedding()` to produce variable-length embeddings, or remove the field |

---

## 2. Module Connectivity Check

### 2.1 Registration Status

| Module | Declared in `mod.rs` | Imported in `world_sim.rs` | Actually Used in `world_sim.rs` |
|--------|:-------------------:|:-------------------------:|:------------------------------:|
| `action_awareness` | YES | YES | YES |
| `action_costs` | YES | YES | YES |
| `event_reactive` | YES | **NO** | **NO** |
| `graph_memory` | YES | YES | YES |
| `memory_stream` | YES | YES | YES |
| `personality_drift` | YES | YES | YES |
| `planning` | YES | YES | YES |
| `reflection` | YES | YES | YES |
| `sim_agent` | YES | YES | YES |
| `spatial_memory` | YES | YES | YES |

### 2.2 Critical Finding: `event_reactive` is NOT wired into WorldSim

**Status**: `event_reactive` has 11 passing tests and is declared in `mod.rs`, but:
- Not imported in `world_sim.rs`
- Not instantiated in `WorldSim::new()`
- Not called in `tick()`, `decide_action()`, or `execute_action()`
- The `SimulationBus` emits events but `EventReactiveSystem` is never subscribed to them

**Impact**: Event-driven reactive behavior (flee on agent death, react to resource depletion, alert nearby agents) is completely non-functional. The `SimEvent::AgentDied` and `SimEvent::ResourceDepleted` events are emitted but无人监听.

**Recommendation**: Wire `EventReactiveSystem` into `WorldSim`:
```rust
// Add to WorldSim struct
pub event_reactive: EventReactiveSystem,

// In new(): register default subscriptions for each agent
// In tick() Slow tier: subscribe new agents, process bus events
// In decide_action(): check for pending reactive responses
```

---

## 3. WorldSim Integration Depth

### 3.1 Field Usage Matrix

| Field | `new()` | `tick()` | `decide_action()` | `execute_action()` | `evolution_cycle()` | Status |
|-------|:-------:|:--------:|:------------------:|:-------------------:|:-------------------:|--------|
| `config` | YES | YES | YES | YES | YES | **Healthy** |
| `bus` | YES | YES | - | - | - | Partial (only emit) |
| `clock` | YES | YES | - | - | YES | **Healthy** |
| `schedule` | YES | YES | - | - | - | **Healthy** |
| `heightmap` | YES | **NO** | **NO** | **NO** | **NO** | **DEAD after init** |
| `biome_map` | YES | **NO** | **NO** | **NO** | **NO** | **DEAD after init** |
| `resources` | YES | YES | - | YES | - | **Healthy** |
| `agents` | YES | YES | YES | YES | YES | **Healthy** |
| `spatial_grid` | YES | YES | - | - | - | **Healthy** |
| `phi_bridge` | YES | YES | - | YES | YES | **Healthy** |
| `coherence_tracker` | YES | YES | - | - | YES | **Healthy** |
| `landscape` | YES | - | - | - | YES | Partial (evolution only) |
| `selection` | YES | - | - | - | YES | Partial (evolution only) |
| `mutator` | YES | - | - | - | YES | Partial (evolution only) |
| `speciation` | YES | - | - | - | YES | Partial (evolution only) |
| `relationships` | YES | - | YES | YES | YES | **Healthy** |
| `economy` | YES | **NO** | **NO** | **NO** | **NO** | **DEAD after init** (only `total_trades` read in `snapshot()`) |
| `culture` | YES | **NO** | **NO** | **NO** | **NO** | **DEAD after init** |
| `rng` | YES | - | YES | - | YES | **Healthy** |
| `emotion` | YES | YES | YES | - | - | **Healthy** |
| `evolution_history` | YES | - | - | - | YES | Partial (evolution only) |
| `action_awareness` | YES | YES | YES | - | - | **Healthy** |
| `structures` | YES | - | - | YES | - | Partial (Build only) |
| `tick` | YES | YES | - | YES | YES | **Healthy** |
| `memory_streams` | YES | YES | - | - | - | **Healthy** |
| `graph_memories` | YES | YES | - | - | - | **Healthy** |
| `spatial_memories` | YES | YES | - | - | - | **Healthy** |
| `planning` | YES | YES | YES | - | - | **Healthy** |
| `reflections` | YES | YES | - | - | - | **Healthy** |
| `action_costs` | YES | YES | - | - | - | **Healthy** |
| `action_budgets` | YES | YES | - | - | - | **Healthy** |
| `personality_drift` | YES | YES | - | - | - | **Healthy** |
| `theory_of_mind` | YES | YES | YES | - | - | **Healthy** |
| `constitutional` | YES | YES | - | - | - | **Healthy** |
| `convergence` | YES | YES | - | - | YES | **Healthy** |
| `dual_repr` | YES | **NO** | **NO** | **NO** | **NO** | **DEAD after init** |

### 3.2 Fields Never Used After Initialization (5 fields)

| Field | Issue | Recommendation |
|-------|-------|----------------|
| `heightmap` | Generated but never queried for terrain height in agent decisions | Wire into `build_observation()` to provide actual terrain data instead of hardcoded `"plain"` |
| `biome_map` | Generated but never queried for biome classification | Wire into `build_observation()` to provide actual biome data |
| `economy` | Created but trade actions don't update it; only `total_trades` read in snapshot | Wire `AgentAction::Trade` in `execute_action()` to update economy state |
| `culture` | Created but never read or written | Wire into social decisions or remove if not needed yet |
| `dual_repr` | Created with size 16 but never populated or queried | Wire into reflection/graph memory to enable dual symbolic+numeric representation |

---

## 4. Test Coverage

### 4.1 Per-Module Test Counts

| Module | Tests | Status |
|--------|:-----:|--------|
| `action_awareness` | 14 | OK |
| `action_costs` | 10 | OK |
| `event_reactive` | 11 | OK (but module not wired in) |
| `graph_memory` | 7 | OK |
| `memory_stream` | 14 | OK |
| `personality_drift` | 8 | OK |
| `planning` | 15 | OK |
| `reflection` | 8 | OK |
| `sim_agent` | 7 | OK |
| `spatial_memory` | 7 | OK |
| **Total agents** | **101** | |
| Other modules (consciousness, evolution, foundation, etc.) | 147 | OK |
| **Total** | **248** | **All passing** |

### 4.2 Coverage Gaps

| Gap | Description |
|-----|-------------|
| No WorldSim integration tests | `WorldSim::tick()` is async and has complex borrow patterns — no integration test exercises the full tick loop |
| No `HostBridge` integration | `HostBridge` is tested in isolation but never called from WorldSim |
| No economy tests in WorldSim context | Economy is initialized but trade execution doesn't update it |
| No culture tests in WorldSim context | Culture is initialized but never exercised |

---

## 5. Cross-Module Dependency Analysis

### 5.1 Dependency Graph (Agent Modules)

```
sim_agent ← (all other agent modules depend on it)
    ↑
    ├── action_awareness (uses AgentAction, SimAgent)
    ├── action_costs (uses AgentAction)
    ├── event_reactive (uses AgentAction, SimEvent)
    ├── personality_drift (uses Personality)
    ├── planning (uses AgentAction)
    └── memory_stream ← reflection (uses MemoryNode, MemoryKind, MemoryStream)

graph_memory: standalone (no agent module deps)
spatial_memory: standalone (no agent module deps)
```

### 5.2 Circular Dependencies

**None detected.** All agent module dependencies are acyclic.

### 5.3 One-Way Dependencies That Could Be Bidirectional

| From | To | Current | Suggested |
|------|----|---------|-----------|
| `reflection` | `memory_stream` | reflection reads from MemoryStream | Correct — reflection produces insights that get added back to MemoryStream. This is already wired in `WorldSim::tick()` |

### 5.4 Missing Dependencies (Modules That Should Interact But Don't)

| Module A | Module B | Rationale | Priority |
|----------|----------|-----------|----------|
| `event_reactive` | `world_sim` | EventReactiveSystem should process bus events and influence agent decisions | P0 |
| `graph_memory` | `reflection` | Reflection insights could be stored as graph nodes with citation edges | P1 |
| `spatial_memory` | `planning` | Planning should use spatial memory to generate informed exploration goals | P1 |
| `memory_stream` | `graph_memory` | MemoryStream nodes could be cross-referenced with graph nodes | P2 |
| `theory_of_mind` | `reflection` | Theory of Mind observations could feed into reflective insights | P2 |
| `economy` | `action_costs` | Trade costs should be reflected in the economy state | P1 |

---

## 6. Summary of Critical Issues

### P0 — Must Fix (blocks functionality)

1. **`event_reactive` not wired into WorldSim** — Event-driven behavior is completely non-functional
2. **14 unused imports** — Minor but noisy; `cargo fix` can auto-resolve 18 of 24 warnings
3. **`heightmap`/`biome_map` dead after init** — Terrain generated but never used for agent perception

### P1 — Should Fix (reduces technical debt)

4. **`economy` dead after init** — Trade actions don't update economy state
5. **`culture` dead after init** — Never read or written
6. **`dual_repr` dead after init** — Created but never populated
7. **`signal_achievement` dead code** — Test helper leaked into production code
8. **`graph_memory.rs:127` irrefutable while-let** — Potential infinite loop (queue never empties because pattern always matches)

### P2 — Nice to Fix (code quality)

9. **8 unused variables** — Should prefix with `_` or remove
10. **`ValueNoise::seed` field never read** — Remove or use
11. **`DualRepresentation::embedding_size` never read** — Wire into `generate_embedding()` or remove
12. **No WorldSim integration tests** — Full tick loop untested
13. **`HostBridge` not connected to WorldSim** — Bridge infrastructure exists but is unused

---

## 7. Recommended Fix Order

```
Phase 3-A (Quick wins, ~30 min):
  1. cargo fix -p neotrix-sim --lib (auto-fix 18 warnings)
  2. Manually fix graph_memory.rs:127 irrefutable while-let
  3. Prefix unused variables with _
  4. Remove signal_achievement from non-test code
  5. Remove unused fields (seed, embedding_size) or wire them in

Phase 3-B (Integration wiring, ~2 hours):
  6. Wire EventReactiveSystem into WorldSim
  7. Wire heightmap/biome_map into build_observation()
  8. Wire economy into execute_action() Trade branch
  9. Connect HostBridge to WorldSim tick loop

Phase 3-C (Missing interactions, ~1 hour):
  10. Wire spatial_memory into planning exploration goals
  11. Wire graph_memory into reflection insights
  12. Add WorldSim integration test (50-tick tick loop)
```

---

*Report generated by auto-inspection agent. All findings are evidence-based (cargo output + source code analysis).*
