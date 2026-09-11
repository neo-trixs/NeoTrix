# Phase 3: Faction System

## Summary

Implemented faction system for neotrix-sim — enabling multi-faction social dynamics, territory control, reputation tracking, and agent-faction membership.

## Files Created/Modified

| File | Action |
|------|--------|
| `neotrix-sim/src/society/faction.rs` | **Created** — Faction + FactionManager |
| `neotrix-sim/src/society/mod.rs` | **Updated** — added `pub mod faction;` + re-export |

## Components

### `FactionId`
Newtype wrapper `FactionId(pub u32)` — unique faction identifier, implements `Hash`, `Eq`, `Serialize/Deserialize`.

### `Faction`
Core faction struct with:
- **Identity**: `id`, `name`, `ideology`
- **Territory**: `territory_center: (f32, f32)`, `territory_radius: f32` — circular territory with `is_in_territory()` check
- **Membership**: `member_ids: Vec<u32>` — `add_member()`, `remove_member()`
- **Diplomacy**: `reputation: HashMap<FactionId, f32>` — `get_reputation()`, `modify_reputation()` (clamped to [-1.0, 1.0])
- **Resources**: `resources: f32` — grows by `members × 0.1` per tick
- **Personality**: `aggression`, `cooperation` (0.0–1.0 range)
- **Lifecycle**: `age: u64` — incremented each tick

### `FactionManager`
Manages all factions and agent-faction assignments:
- `create_faction()` — auto-increments ID
- `assign_agent()` — removes from old faction, adds to new
- `get_faction_for_agent()` — lookup by agent ID
- `get_closest_faction(x, y)` — nearest faction to coordinates
- `tick()` — advances all factions

## Verification

```
cargo check -p neotrix-sim --tests    ✅ passed (0 errors)
cargo test -p neotrix-sim --lib -- faction
  test_faction_creation    ... ok
  test_faction_territory   ... ok
  test_faction_manager     ... ok
  test_faction_reputation  ... ok
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 359 filtered out
```

## Default Values

| Field | Default | Purpose |
|-------|---------|---------|
| `territory_radius` | 50.0 | Circular territory extent |
| `resources` | 100.0 | Initial resource stock |
| `aggression` | 0.5 | Combat倾向 |
| `cooperation` | 0.5 | Trade倾向 |

## Integration Points

- Agent structs can hold `FactionId` to track membership
- `FactionManager` can be added to `SimState` for world-level faction tracking
- `reputation` map enables inter-faction diplomacy mechanics
- `territory_radius` + `is_in_territory()` enables spatial control logic
