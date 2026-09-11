# Phase 1: A* Pathfinding — neotrix-sim

## Summary

Implemented A* grid pathfinding for `neotrix-sim` navigation subsystem.

## Files Created

| File | Purpose |
|------|---------|
| `neotrix-sim/src/navigation/mod.rs` | Module root |
| `neotrix-sim/src/navigation/astar.rs` | A* implementation + 4 unit tests |

## Files Modified

| File | Change |
|------|--------|
| `neotrix-sim/src/lib.rs:18` | Added `pub mod navigation;` |

## API

```rust
pub struct GridPos { pub x: i32, pub y: i32 }
pub struct AStar { width, height, obstacles, movement_cost }

AStar::new(width, height) -> Self
AStar::set_obstacle(x, y, blocked)
AStar::set_movement_cost(x, y, cost)
AStar::is_walkable(pos) -> bool
AStar::find_path(start, goal) -> Option<Vec<GridPos>>
AStar::find_path_with_limit(start, goal, max_steps) -> Option<Vec<GridPos>>
```

## Features

- 8-directional movement (cardinal + diagonal)
- Diagonal cost: `1.414 × cell_cost` (sqrt(2) approximation)
- Per-cell movement cost via `set_movement_cost`
- Obstacle map via `set_obstacle`
- Step-limited pathfinding (`find_path_with_limit`)
- Manhattan distance heuristic (admissible for 8-dir grid)

## Test Results

```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
  test_astar_straight_line .... ok
  test_astar_avoids_obstacle .. ok
  test_astar_no_path ......... ok
  test_astar_with_step_limit .. ok
```

## Build

```
cargo check -p neotrix-sim  → 1 warning (unused `h` field in Node struct)
cargo test -p neotrix-sim --lib -- navigation  → 4/4 pass
```

## Known Warning

`h` field in internal `Node` struct is never read — kept for struct clarity and debuggability.
