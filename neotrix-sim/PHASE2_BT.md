# PHASE 2: Explicit Behavior Tree Nodes

## Summary

Replaced monolithic `behavior_tree.rs` with `behavior_tree/mod.rs` containing explicit, composable BT node types backed by a generic `Any`-typed blackboard.

## Changes

| Before | After |
|--------|-------|
| `BtStatus` | `BTStatus` (explicit naming) |
| Typed `Blackboard` (floats/ints/bools/strings/vectors maps) | Generic `Blackboard` (`HashMap<String, Box<dyn Any + Send + Sync>>`) |
| `Inverter`, `Repeater` | `Decorator` (generic status modifier) |
| `Selector::new(name, children)` | `Selector::new(children)` |
| `Sequence::new(name, children)` | `Sequence::new(children)` |
| `BehaviorTree` owned `blackboard` | `BehaviorTree` takes `Blackboard` via `tick(&mut self, &mut Blackboard)` |
| `bt_action_to_agent_action` bridge | Removed (no external consumers) |

## Node Types

| Node | Purpose |
|------|---------|
| `Selector` | Tries children until one succeeds |
| `Sequence` | Runs children until one fails |
| `Condition` | Predicate check on blackboard |
| `Action` | Side-effect execution |
| `Decorator` | Wraps child, modifies status output |
| `BehaviorTree` | Root wrapper, ticks blackboard through |

## Verification

```
cargo check -p neotrix-sim     → OK (1 pre-existing warning)
cargo test -p neotrix-sim --lib -- behavior_tree → 4 passed; 0 failed
```

- `test_blackboard` — generic get/set roundtrip
- `test_condition` — predicate reads blackboard
- `test_selector` — picks first success
- `test_sequence` — runs all children

## Integration

No external code depends on `bt_action_to_agent_action` or the old `BtStatus`/`Blackboard` APIs. The `mod.rs` path is resolved automatically by `pub mod behavior_tree;` in `agents/mod.rs`.
