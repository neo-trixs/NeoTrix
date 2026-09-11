# Phase 2: GOAP (Goal-Oriented Action Planning)

## Summary

Implemented A*-based GOAP planner for `neotrix-sim` agents, enabling forward-chaining action planning from world state to goal.

## Files

| File | Description |
|------|-------------|
| `src/agents/planning/goap.rs` | GOAP planner: state, action, plan, planner, agent integration |
| `src/agents/planning/mod.rs` | Updated with `pub mod goap;` |

## Architecture

```
GOAPState (HashMap<String, bool>)
  └─ satisfies(preconditions) → bool
  └─ apply(effects) → mutates state

GOAPAction
  └─ name, cost, preconditions: GOAPState, effects: GOAPState

GOAPPlanner
  └─ plan(world_state, goal) → Option<GOAPPlan>
     BFS with visited-set, max_iterations cap (100)

GOAPPlan
  └─ actions: Vec<GOAPAction>, total_cost: f32
```

## Agent Integration

- `agent_to_world_state(agent: &SimAgent) -> GOAPState` — maps agent core stats to boolean properties
- `create_survival_plan(agent: &SimAgent) -> Option<GOAPPlan>` — survival goal (reduce hunger)
- Actions: rest → gather_food → eat → heal → explore → trade

## Test Results

```
running 7 tests
test agents::planning::goap::tests::test_goap_state_apply ... ok
test agents::planning::goap::tests::test_goap_state_satisfies ... ok
test agents::planning::goap::tests::test_agent_to_world_state ... ok
test agents::planning::goap::tests::test_goap_planner_chain ... ok
test agents::planning::goap::tests::test_create_survival_plan ... ok
test agents::planning::goap::tests::test_goap_planner_no_plan ... ok
test agents::planning::goap::tests::test_goap_planner_simple ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 342 filtered out
```

## Notes

- Adapted `SimAgent` integration to real struct (no `inventory` field — uses `core.hunger`, `core.energy`, `core.health`)
- Removed `Hash` derive from `GOAPState` (HashMap doesn't implement Hash)
- `BFS` planner with visited-set prevents infinite loops; `max_iterations` cap for safety
