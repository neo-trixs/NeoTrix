# Phase 4: Elite Archive & RVO Collision Avoidance

## Summary

Added two new modules to neotrix-sim:
- **Elite Archive** (`evolution/archive.rs`): Maintains a sorted collection of top-performing agents for evolutionary selection
- **RVO Collision Avoidance** (`navigation/rvo.rs`): Reciprocal Velocity Obstacles for agent collision avoidance

## Files Created

### `neotrix-sim/src/evolution/archive.rs`

```rust
// Key types:
EliteEntry    - Archived agent with fitness, generation, novelty
EliteArchive  - Sorted archive with max_size constraint

// Key methods:
insert()          - Add agent to archive (maintains sorted order)
get_best()        - Get highest-fitness entry
get_diverse_sample(n) - Get n spread across fitness spectrum
average_fitness() - Average fitness of all entries
generation_span() - Range of generations in archive
```

### `neotrix-sim/src/navigation/rvo.rs`

```rust
// Key types:
RVOAgent      - Agent with position, velocity, preferred velocity
RVOSimulator  - Manages multi-agent collision avoidance

// Key methods:
add_agent()              - Register agent in simulation
set_preferred_velocity() - Set desired movement direction
compute_new_velocity()   - Calculate collision-free velocity
step()                   - Advance simulation one tick
get_neighbors()          - Find nearby agents within neighbor_dist
```

## Integration Points

### Evolution Pipeline
- `EliteArchive` can replace or augment existing selection mechanisms
- `get_diverse_sample()` enables niched evolution strategies
- `generation_span()` tracks evolutionary progress over time

### Navigation System
- `RVOSimulator::from_agent()` bridges SimAgent → RVOAgent
- Collision avoidance integrates with existing A* pathfinding
- Time horizon parameter allows lookahead for proactive avoidance

## Test Results

```
cargo test -p neotrix-sim --lib -- archive
running 3 tests
test evolution::archive::tests::test_archive_max_size ... ok
test evolution::archive::tests::test_archive_insert ... ok
test evolution::archive::tests::test_archive_best ... ok
test result: ok. 3 passed; 0 failed; 0 ignored

cargo test -p neotrix-sim --lib -- rvo
running 3 tests
test navigation::rvo::tests::test_rvo_agent ... ok
test navigation::rvo::tests::test_rvo_simulator ... ok
test navigation::rvo::tests::test_rvo_velocity ... ok
test result: ok. 3 passed; 0 failed; 0 ignored
```

## Usage Examples

### Elite Archive
```rust
use crate::evolution::EliteArchive;
use crate::agents::SimAgent;

let mut archive = EliteArchive::new(100);

// Add agents during evolution
for agent in &population {
    archive.insert(agent, generation, novelty_score);
}

// Select parents for next generation
let diverse_parents = archive.get_diverse_sample(10);
let best = archive.get_best();
```

### RVO Collision Avoidance
```rust
use crate::navigation::{RVOSimulator, RVOAgent};

let mut sim = RVOSimulator::new();

// Register agents
let idx0 = sim.add_agent(RVOAgent::new(0.0, 0.0));
let idx1 = sim.add_agent(RVOAgent::new(1.0, 0.0));

// Set desired velocities
sim.set_preferred_velocity(idx0, 1.0, 0.0);
sim.set_preferred_velocity(idx1, -1.0, 0.0);

// Advance simulation (velocities adjusted to avoid collision)
sim.step();
```

## Design Decisions

1. **Vec2 Consistency**: RVO module uses `Vec2` from `math_bridge` instead of raw `[f32; 2]` arrays
2. **Sorted Archive**: Entries always sorted by fitness for O(1) best access
3. **Min-Fitness Cache**: Avoids scanning entire archive for rejection checks
4. **Penalty-Based RVO**: Simplified velocity obstacle resolution using penetration depth
