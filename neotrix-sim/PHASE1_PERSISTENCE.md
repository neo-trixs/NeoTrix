# Phase 1: Persistence System

## What was done

Created `neotrix-sim/src/world_sim/persistence.rs` — full save/load system for `WorldSim` state.

## Files changed

| File | Change |
|------|--------|
| `neotrix-sim/src/world_sim/persistence.rs` | New file — `FullWorldSnapshot`, `AgentSnapshot`, `ResourceSnapshot`, `PersistenceManager` |
| `neotrix-sim/src/world_sim/mod.rs` | Added `pub mod persistence;` |

## Types

### `FullWorldSnapshot`
Full serializable snapshot (distinct from the summary `WorldSnapshot` in `config.rs`). Contains:
- `tick: u64` — simulation tick
- `config: WorldSimConfig` — full config for re-creating the world
- `agents: Vec<AgentSnapshot>` — all agent states
- `resources: Vec<ResourceSnapshot>` — all resource node states
- `terrain_seed: u64` — seed used for terrain generation

### `AgentSnapshot`
Captures: `id, x, y, alive, energy, health, hunger, age, phi, personality, needs, skills`

### `ResourceSnapshot`
Captures: `id, resource_type, position, amount, max_amount, regeneration_rate, depleted`

### `PersistenceManager`
- `new(save_dir)` — set save directory
- `save(sim)` → `Result<String>` — save to `save_{tick}.json`, auto-creates dir
- `load(filename)` → `Result<FullWorldSnapshot>` — load from file
- `load_latest()` → `Result<FullWorldSnapshot>` — find and load most recent save
- `restore_from_snapshot(snapshot)` → `Result<WorldSim>` — rebuild full sim from snapshot (terrain from seed + agent/resource state overlay)

## Design decisions

1. **Name `FullWorldSnapshot`** — avoids collision with the summary `WorldSnapshot` already in `config.rs`
2. **Resources saved in full** — regenerated from seed but amounts/ depletion state overlaid from snapshot
3. **Terrain regenerated from seed** — deterministic heightmap/biome, no need to serialize the grid
4. **Agent state fully captured** — personality, needs, phi, skills preserved across save/load

## Verification

```
cargo check -p neotrix-sim          ✅ (0 errors, 1 pre-existing warning)
cargo test -p neotrix-sim --lib -- persistence
  test_snapshot_creation            ✅
  test_save_load_cycle              ✅
  test_restore_roundtrip            ✅
  test_resource_snapshot_roundtrip  ✅
```

4/4 tests passed. 0 failures.
