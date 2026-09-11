# P0 Dead-Code Activation Log

**Date**: 2026-09-11
**Agent**: NT-WORLD-SIM Fix Agent

## Task Results

| Task | Status | Details |
|------|--------|---------|
| P0-1: A* Pathfinding | **SUCCESS** | `AStar::find_path()` integrated into `AgentAction::Move` branch |
| P0-2: RVO Collision Avoidance | **SUCCESS** | `RVOSimulator::compute_new_velocity()` applied before position update |
| P0-3: GOAP Planner Integration | **SUCCESS** | `create_survival_plan()` wired as fallback in `layer_goals()` |
| P0-4: Compile Check | **SUCCESS** | `cargo check -p neotrix-sim` passes (2 pre-existing warnings only) |
| P0-5: Test Verification | **SUCCESS** | 395/395 tests pass |

## Changes Made

### 1. `src/world_sim/actions.rs` — A* + RVO Integration

**Before**: `AgentAction::Move` did straight-line movement:
```rust
let dir = (*target - agent.core.position).normalize();
agent.core.position = agent.core.position + dir * speed;
```

**After**: A* pathfinding with RVO collision avoidance:
- Grid conversion: agent.position (Vec2) → GridPos (i32) with 10.0 scale
- A* obstacles: mountain cells marked as blocked via `heightmap.is_mountain()`
- Path following: moves toward next waypoint on A* path
- RVO: builds temporary `RVOSimulator` with nearby agents (30.0 radius), computes collision-free velocity
- Fallback: if A* fails, falls back to direct movement

**Imports added**:
```rust
use crate::navigation::astar::{AStar, GridPos};
use crate::navigation::rvo::{RVOSimulator, RVOAgent};
```

### 2. `src/world_sim/decision.rs` — GOAP Fallback

**Before**: `layer_goals()` only used `PlanningStack::next_action()`, returned `None` if no active goal.

**After**: Falls back to `create_survival_plan(agent)` when PlanningStack has no action:
- GOAP actions mapped to `AgentAction` via `goap_action_to_agent_action()`
- Mapping: "rest"→Rest, "gather_food"/"eat"→Eat (with nearby resource check), "heal"→Rest, "explore"→Explore, "trade"→Trade
- Backward compatible: PlanningStack still takes priority when it has a valid action

**Import added**:
```rust
use crate::agents::planning::goap::create_survival_plan;
```

**New function added**:
```rust
fn goap_action_to_agent_action(name: &str, obs: &AgentObservation) -> Option<AgentAction>
```

## Pre-existing Warnings (not introduced by this change)

| File | Warning |
|------|---------|
| `navigation/rvo.rs:79` | Unused variable `rel_vel` |
| `navigation/astar.rs:31` | Field `h` is never read in `Node` |

## Interface Match Assessment

| Module | Interface | Match? |
|--------|-----------|--------|
| AStar::find_path(GridPos, GridPos) → Option<Vec<GridPos>> | Direct | Yes |
| RVOSimulator::compute_new_velocity(idx) → Vec2 | Built on-the-fly per Move | Yes |
| GOAPPlanner::plan(state, goal) → Option<GOAPPlan> | Wrapped via create_survival_plan() | Yes |
| MemoryStream::retrieve() | Not wired (future P1) | N/A |

## Notes

- A* grid rebuilds every Move tick (O(grid_size) = O(128²) worst-case). Could be cached for performance in P1.
- RVO builds temporary simulator per Move tick. Could be maintained incrementally in P1.
- GOAP fallback only uses `create_survival_plan()` which has hardcoded actions. Full GOAP with dynamic world-state grounding is a P1 task.
- `MemoryStream::retrieve()` was not integrated — it requires embedding computation that doesn't exist in the current pipeline. Flagged for P1.
