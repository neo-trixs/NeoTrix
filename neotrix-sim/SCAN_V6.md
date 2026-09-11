# neotrix-sim SCAN V6

Date: 2026-09-11

## Build Status

- **cargo check**: Clean — zero errors, zero warnings
- **cargo test --lib**: **322 passed; 0 failed** (0.02s)

## Source Metrics

| Metric | Value |
|--------|-------|
| `world_sim.rs` lines | 1550 |
| Agent modules (`src/agents/*.rs`) | 16 files |

## Agent Modules

```
action_awareness.rs      action_costs.rs         event_reactive.rs
goal_outcome_feedback.rs graph_memory.rs         intention_commitment.rs
memory_stream.rs         mod.rs                  personality_drift.rs
pheromone.rs             planning.rs             reflection.rs
sim_agent.rs             social_learning.rs      spatial_memory.rs
thought_generation.rs
```

## Summary

Full green. 322/322 tests pass, clean build with no warnings. The simulation crate is healthy at V6 scan.
