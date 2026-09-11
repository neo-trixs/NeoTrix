# STIGMERGY_IMPLEMENTATION.md — Stigmergic Coordination via Shared KB

**Date**: 2026-09-11
**Status**: Implemented and verified (325/325 tests pass, 0 errors)

---

## Overview

Implemented a stigmergic coordination system for neotrix-sim. Agents coordinate indirectly by depositing and sensing virtual pheromones in a shared `PheromoneField` — no direct agent-to-agent communication needed.

**Research grounding**: Stigmergic Multi-Agent DRL (Springer 2025) — virtual pheromones enable decentralized coordination, scaling to 5-8 agents (vs MADDPG fails at 2+).

## Architecture

```
Agent A executes action
    → deposit_pheromones_for_action()
        → PheromoneField.deposit(type, position, agent_id, tick)
            → merged or new pheromone in shared field

Agent B runs decide_action()
    → layer_stigmergy()
        → PheromoneField.sense(position, radius, tick) → PheromoneSignal
            → adjusts action: flee danger, seek food, rest, socialize
```

## New Module: `agents/pheromone.rs`

### Types

| Type | Purpose | Decay Rate | Strength | Influence |
|------|---------|-----------|----------|-----------|
| `Food` | Food discovered | 0.003/tick | 1.0 | +0.8 (attract) |
| `Danger` | Threat detected | 0.005/tick | 1.5 | -1.0 (repel) |
| `Rest` | Good rest spot | 0.002/tick | 0.6 | +0.5 (attract) |
| `Social` | Interaction occurred | 0.008/tick | 0.4 | +0.6 (attract) |
| `Explore` | Exploration trail | 0.01/tick | 0.3 | +0.2 (mild attract) |
| `Territory` | Build/marking | 0.001/tick | 0.8 | +0.3 (mild attract) |

### Key Structs

- **`Pheromone`**: Single marker — type, position, strength, decay_rate, deposited_by, deposited_tick
- **`PheromoneField`**: Shared coordination substrate — Vec of pheromones + spatial grid index
- **`PheromoneSignal`**: Aggregated signal at a point — food/danger/rest/social/explore/territory attractors
- **`PheromoneType`**: Enum with base_decay_rate(), base_strength(), influence() methods

### Spatial Hashing

PheromoneField uses spatial grid hashing (cell_size=50.0) for O(1) neighbor lookups. Sensing queries only nearby cells, not all pheromones.

### Merge-on-Deposit

Same-type pheromones within merge_radius (cell_size * 0.5) stack strength instead of duplicating. This creates pheromone "hotspots" — locations where multiple agents confirmed a signal.

## Integration Points

### 1. WorldSim struct (`world_sim.rs`)

```rust
pub pheromone_field: PheromoneField,  // initialized with cell_size=50.0, max=5000
```

### 2. Decision Pipeline — `layer_stigmergy` (new layer 3.5)

Inserted between `layer_social` and `layer_personality` in the 6-layer decision pipeline:

- **Danger > 0.5**: Flee away from strongest danger pheromone
- **Food > 0.3 + hungry**: Navigate toward food pheromone
- **Rest > 0.4 + tired**: Rest at location
- **Social > 0.6 + nearby agent**: Talk to nearby agent

### 3. Action Execution — `deposit_pheromones_for_action`

After every action, agents deposit appropriate pheromones:

| Action | Pheromone | Location |
|--------|-----------|----------|
| Eat/Harvest | Food | Resource position |
| Rest | Rest | Agent position |
| Talk | Social | Agent position |
| Attack | Danger | Agent position |
| Explore | Explore | Agent position |
| Build | Territory | Build position |
| Move/Trade/Think | None | — |

### 4. Tick Loop — Decay (Slow tier, every 100 ticks)

```rust
self.pheromone_field.decay(tick);    // Remove dead pheromones
self.pheromone_field.prune(tick);    // Enforce max limit
// Emit PheromoneDeposited events for EventBus
```

### 5. EventBus Events

```rust
SimEvent::PheromoneDeposited { agent_id, ptype, position, strength }
SimEvent::PheromoneSensed { agent_id, dominant_type, signal_strength }
```

## Test Results

All 10 pheromone-specific tests pass:

```
pheromone_creation              ✓
pheromone_decay                 ✓
field_deposit_and_sense         ✓
field_merge_stacks_strength     ✓
field_different_types_dont_merge ✓
field_prune_respects_limit      ✓
strongest_direction             ✓
decay_removes_old_pheromones    ✓
signal_net_valence              ✓
signal_dominant_type            ✓
```

Full crate: **325/325 tests pass**, 0 errors.

## Files Modified

| File | Change |
|------|--------|
| `src/agents/pheromone.rs` | **NEW** — PheromoneType, Pheromone, PheromoneField, PheromoneSignal |
| `src/agents/mod.rs` | Added `pub mod pheromone` + `pub use pheromone::*` |
| `src/foundation/simulation_bus.rs` | Added PheromoneDeposited/PheromoneSensed events |
| `src/world_sim.rs` | Added PheromoneField field, layer_stigmergy, deposit_pheromones_for_action, decay in Slow tier |

## Stigmergic Coordination Flow

```
1. Agent A finds food → deposits Food pheromone at resource location
2. Pherevent: Agent A also deposits Food pheromone at resource location
2. PheromoneField stores the marker with strength=1.0, decay=0.003/tick
3. Agent B (hungry, nearby) senses Food pheromone during decide_action
4. layer_stigmergy: Food signal > 0.3 → navigate toward strongest Food pheromone
5. Agent B moves toward food → no direct A→B communication needed
6. Pheromone decays over time → signal fades if not reinforced
7. Other agents can reinforce the pheromone if they also find food
```

## Design Decisions

1. **Shared PheromoneField** (not per-agent): All agents read/write the same field — this IS the "shared KB" for stigmergic coordination.

2. **Spatial hashing**: O(1) neighbor lookups enable real-time sensing even with 5000+ pheromones.

3. **Merge-on-deposit**: Prevents pheromone explosion from repeated deposits at same location. Strength stacks up to cap of 3.0.

4. **Layer priority**: Stigmergy (layer 3.5) overrides personality but not survival/goals/social — pheromones inform but don't dominate.

5. **Decay rates tuned per type**: Danger decays faster (0.005) than Food (0.003) — stale danger warnings fade quicker than food trails. Social decays fastest (0.008) — ephemeral signals.
