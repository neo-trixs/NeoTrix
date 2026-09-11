# SCAN V4 — neotrix-sim Code Scan Report

**Date**: 2026-09-11  
**Crate**: `neotrix-sim` v0.1.0  
**Workspace**: `/Users/neo/Downloads/neotrix`

## 1. Compilation Status

| Check | Result |
|-------|--------|
| `cargo check -p neotrix-sim` | **PASS** — 0 errors, 0 warnings |
| `cargo test -p neotrix-sim --lib` | **325/325 passed**, 0 failed |
| Integration tests | 12/12 passed |

## 2. Issues Found & Fixed

### FIX-1: Redundant `as f32` cast (cosmetic)
- **File**: `src/physics/mod.rs:174`
- **Before**: `let tilt = (self.roll.abs() + self.pitch.abs()) as f32;`
- **After**: `let tilt = self.roll.abs() + self.pitch.abs();`
- **Reason**: `self.roll` and `self.pitch` are already `f32`. The `as f32` cast is a no-op that triggers `clippy::unnecessary_cast`.

### FIX-2: Unused dependency `rand = "0.8"` (dead weight)
- **File**: `Cargo.toml`
- **Before**: `rand = "0.8"` listed under `[dependencies]`
- **After**: Removed
- **Reason**: No source file in `neotrix-sim/src/` imports or uses `rand`. The crate uses its own `SimulationRng` in `foundation/math_bridge.rs` instead. This adds unnecessary compile time and binary size.

## 3. Codebase Summary

| Metric | Value |
|--------|-------|
| Source files (`.rs`) | 65 |
| Total lines of code | 16,814 |
| Largest file | `world_sim.rs` (1,550 lines) |
| Module count | 16 modules |
| Test count | 325 unit + 12 integration |
| Test coverage | All modules have `#[cfg(test)] mod tests` |

### Module Breakdown

| Module | Lines | Purpose |
|--------|-------|---------|
| `world_sim.rs` | 1,550 | Main simulation loop (WorldSim) |
| `agents/graph_memory.rs` | 1,207 | Graph-based memory with spreading activation |
| `consciousness/emergence_detector.rs` | 655 | Collective behavior pattern detection |
| `feel/mod.rs` | 552 | 15-emotion engine with PAD model |
| `agents/memory_stream.rs` | 515 | Short-term memory with keyword/cosine retrieval |
| `agents/planning.rs` | 483 | Multi-tier goal planning (survival/social/exploration) |
| `safety/safety_monitor.rs` | 469 | Behavioral loop + personality drift detection |
| `agents/action_awareness.rs` | 462 | Predict-verify cycle for action calibration |
| `agents/pheromone.rs` | 447 | Stigmergy: shared pheromone field for indirect coordination |
| `agents/event_reactive.rs` | 442 | Event-driven reactive behavior system |
| `foundation/math_bridge.rs` | 413 | Vec2/Vec3, SpatialGrid, SimulationRng, cosine_sim |
| `safety/audit_trail.rs` | 366 | Immutable event log with type-safe audit events |
| `agents/social_learning.rs` | 360 | Social action pattern mimicry + meme sharing |
| `agents/goal_outcome_feedback.rs` | 354 | Goal success/failure feedback for planning |
| `agents/sim_agent.rs` | 342 | Core agent with metabolism, personality, fitness |

### Architecture: 6-Layer Simulation

```
L5 Cognition     → consciousness/ (phi, coherence, convergence, dual_repr, behavior_vm)
L4 Emotion       → feel/ (15-emotion engine, PAD, conflict resolution)
L3 Embodiment    → physics/ + sensor/ + actuator/ (robot body model)
L2 Perception    → environment/ (terrain, biomes, resources, structures, GPU grid)
L1 Action        → agents/ (planning, action costs, memory, social learning, stigmergy)
L0 Foundation    → foundation/ (math, simulation bus, sim time, tick schedule)
```

Cross-cutting:
- `bridge/` — Bidirectional bridge between sim and NeoTrix core (directives, reports)
- `core_bridge/` — Emotion → reasoning modulation (GWT, E8, CT)
- `safety/` — Guardrails: capability tracking, evolution constraints, audit trail
- `society/` — Social: relationships, economy, culture, theory of mind, constitutional
- `evolution/` — Fitness landscape, selection, mutation, speciation
- `world/` — World state, obstacles, BLE fleet simulation

## 4. Observations (No Action Required)

### 4.1 Duplicate Vec3 Types
- `physics::Vec3` — local to physics simulation
- `foundation::math_bridge::Vec3` — shared foundation type
- Both implement similar methods (new, normalize, dot, cross, etc.)
- **Status**: Intentional separation (physics-specific vs general-purpose). Would benefit from unifying in a future refactor.

### 4.2 `_old_hour` Unused Variable
- **File**: `foundation/sim_time.rs:62` — `let _old_hour = self.current.hour;`
- **Status**: Unused variable with `_` prefix (conventional). No action needed.

### 4.3 `world_sim.rs` Size (1,550 lines)
- The main simulation loop is the largest file. It contains WorldSim struct, tick loop, decision layers, action execution, evolution cycle, and world event collection.
- **Status**: Acceptable for a simulation orchestrator. Could be split into smaller modules in future refactoring.

### 4.4 `nt-world-sim` (Tauri Wrapper)
- Separate crate that wraps `neotrix-sim` for desktop GUI
- Has its own `SimState` struct (Tauri managed state) that wraps `WorldSim`
- **Status**: Clean separation. No issues found.

## 5. File Structure

```
neotrix-sim/
├── Cargo.toml          # Dependencies: serde, serde_json, tokio (rand removed)
├── src/
│   ├── lib.rs          # Crate root: SimState, tick loop, 61-dim observation
│   ├── main.rs         # CLI binary: nt-sim simulation runner
│   ├── actuator/       # ServoBus: 14-DOF joint control
│   ├── agents/         # 16 agent subsystems (planning, memory, social, etc.)
│   ├── bridge/         # HostBridge + trait_defs (Core↔Sim interface)
│   ├── consciousness/  # Phi, coherence, convergence, dual repr, behavior VM
│   ├── core_bridge/    # Emotion→reasoning modulation
│   ├── environment/    # Terrain, structures, GPU grid
│   ├── evolution/      # Fitness, selection, mutation, speciation
│   ├── feel/           # 15-emotion engine with PAD model
│   ├── foundation/     # Math, event bus, sim time, tick schedule
│   ├── physics/        # Robot body physics simulation
│   ├── safety/         # Capability tracker, safety monitor, evolution constraints, audit
│   ├── sensor/         # Virtual sensor array (ToF, IMU, battery)
│   ├── society/        # Relationships, economy, culture, ToM, constitutional
│   ├── world/          # World state, obstacles, BLE fleet
│   └── world_sim.rs    # Main simulation loop (1,550 lines)
├── tests/
│   └── integration.rs  # 12 integration tests
└── SCAN_V4.md          # This report
```

## 6. nt-world-sim (Tauri Desktop)

```
nt-world-sim/
├── Cargo.toml          # Depends on neotrix-sim + tauri
├── src/main.rs         # Tauri commands (sim_get_state, sim_tick, sim_get_agents, etc.)
├── build.rs            # Tauri build script
├── tauri.conf.json     # Tauri config
├── dist/               # Frontend: index.html, main.js
├── gen/                # Generated Tauri files
└── icons/              # App icons
```

## 7. Conclusion

**neotrix-sim is in good health.** All 337 tests pass (325 unit + 12 integration). Two minor issues were fixed:

1. Redundant `as f32` cast removed (cosmetic)
2. Unused `rand` dependency removed (dead weight)

No errors, no warnings, no unsafe code. The codebase is well-structured with clear module boundaries and comprehensive test coverage across all 16 modules.
