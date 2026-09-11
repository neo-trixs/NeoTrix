# P0 Dead-Code Activation Log — 2026-09-11

## Summary

Activated 3 dead-code handlers (Attack, Gather, pheromone integration) and fixed 1 signature mismatch.

## Tasks Completed

### P0-7: Attack Handler (`actions.rs:188-234`)
- **Status**: DONE
- **Implementation**: Damage = (5 + aggression*10) - (defender.cooperativeness*5), min 1.0
- Calls `target.core.take_damage(damage)`, death check handled by `AgentCore`
- Emits `AgentActed` event (attack result) + death event if killed
- Updates relationship graph (-0.3 delta)
- Deposits `Danger` pheromone at attack location

### P0-8: Gather Handler (`actions.rs:236-266`)
- **Status**: DONE
- **New variant**: Added `AgentAction::Gather { resource_id }` to enum (`sim_agent.rs:29`)
- Harvests 15.0 units from nearest `ResourceNode`
- Feeds agent (gathered*0.5) + energy (gathered*0.3)
- Emits `AgentActed` + `ResourceDepleted` events
- Deposits `Food` pheromone at resource location

### P0-9: Pheromone Integration Fix (`decision.rs:26`)
- **Status**: DONE
- **Bug fixed**: `layer_personality()` call was missing `pheromone: &PheromoneSignal` argument
- Now passes `&pheromone_signal` (already computed at line 12) to the personality layer
- Pheromone net_valence now correctly modulates exploration/exploitation tradeoff

### P0-10: Compilation
- **Status**: PASS
- `cargo check -p neotrix-sim` — 0 errors (3 pre-existing warnings only)

### P0-11: Tests
- **Status**: PASS
- `cargo test -p neotrix-sim --lib` — 395 passed, 0 failed

## Files Modified

| File | Change |
|------|--------|
| `src/agents/sim_agent.rs` | Added `Gather { resource_id: String }` variant to `AgentAction` |
| `src/world_sim/actions.rs` | Implemented Attack + Gather handlers; added Gather pheromone deposit |
| `src/world_sim/decision.rs` | Fixed `layer_personality` call signature (added `pheromone` arg) |
| `src/agents/action_awareness.rs` | Added `Gather` match arm in `predict()` |
| `src/agents/action_costs.rs` | Added `Gather` match arm + cost entry (energy:2, time:2, risk:0.05) |
| `src/agents/social_learning.rs` | Added `Gather` match arm in `action_pattern_key()` |

## Derived Stats (Attack)

No explicit `attack`/`defense` fields on `AgentCore`. Used personality proxies:
- **Attack power**: `5.0 + aggression * 10.0` (range: 5.0-15.0)
- **Defense**: `cooperativeness * 5.0` (range: 0.0-5.0)
- **Min damage**: 1.0 (always at least 1 HP)

## Notes

- `AgentCore::take_damage()` already handles HP decrement + death flag
- `WorldSim::tick()` already retains only alive agents (line 241)
- `deposit_pheromones_for_action` was already wired for Attack/Danger (line 196-198), just needed Gather/Food
