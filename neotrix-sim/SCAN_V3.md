# neotrix-sim Codebase Scan — V3

**Date**: 2026-09-11
**Crate**: `neotrix-sim` v0.1.0
**Binary**: `nt-sim`
**Description**: NeoTrix consciousness embodiment simulator — virtual world for NT-PHYSICAL, NT-FEEL, NT-CORE co-evolution

---

## 1. Build Status

| Check | Result |
|-------|--------|
| `cargo check -p neotrix-sim` | **PASS** — 0 errors, 0 warnings |
| `cargo test -p neotrix-sim --lib` | **Timeout** — build lock contention (cargo check held lock); 325 `#[test]` functions found across 42 files |

---

## 2. Size Metrics

| Metric | Value |
|--------|-------|
| Total `.rs` files | 65 |
| Total lines of code | 16,814 |
| `pub struct` declarations | 156 |
| `pub enum` declarations | 36 |
| `#[test]` functions | 325 |
| Files with tests | 42 / 65 (64.6%) |
| TODO/FIXME/HACK markers | 0 |

### Lines by Module

| Module | Lines | Purpose |
|--------|------:|---------|
| `agents/` | 6,224 | Agent cognition: planning, memory, social, pheromone, reflection |
| `consciousness/` | 1,641 | Emergence detection, behavior VM, dual representation, phi bridge |
| `safety/` | 1,397 | Safety monitor, audit trail, capability tracker, evolution constraints |
| `foundation/` | 1,084 | Math bridge, tick schedule, simulation bus |
| `environment/` | 1,079 | Terrain (heightmap/biome/resources), GPU grid, structures |
| `society/` | 683 | Theory of mind, constitutional rules, economy |
| `evolution/` | 600 | Evolution simulation |
| `feel/` | 552 | Emotion engine (PAD model, SystemEvent, emotion dynamics) |
| `world/` | 408 | World state, fleet, obstacles |
| `bridge/` | 375 | Host bridge (sim ↔ neotrix-core) |
| `physics/` | 258 | Physics state, Vec3, rigid body dynamics |
| `core_bridge/` | 201 | Core bridge (emotion → attention weights) |
| `actuator/` | 194 | Servo bus, joint limits, 14-DOF actuator model |
| `sensor/` | 171 | Sensor array, battery, IMU, observation vector |
| `world_sim.rs` | 1,550 | Standalone world simulation (separate from `world/` module) |
| `lib.rs` | 236 | SimState: 50Hz tick loop, 61-dim observation, reactive policy, safety clamp |
| `main.rs` | 161 | CLI demo: 5-phase simulation (walk → push → novelty → fatigue → multi-emotion) |

---

## 3. Agent Subsystem (16 files, 6,224 lines)

| File | Lines | Tests | Public Items | Purpose |
|------|------:|------:|-------------:|---------|
| `graph_memory.rs` | 1,207 | 17 | 54 | Graph-based episodic/semantic memory |
| `memory_stream.rs` | 515 | 18 | 15 | Streaming memory buffer |
| `planning.rs` | 483 | 18 | 17 | Goal planning & decomposition |
| `action_awareness.rs` | 462 | 14 | — | Action awareness & self-monitoring |
| `pheromone.rs` | 447 | 10 | 20 | Pheromone trail stigmergy |
| `event_reactive.rs` | 442 | 11 | — | Reactive event processing |
| `social_learning.rs` | 360 | 11 | — | Social learning & imitation |
| `goal_outcome_feedback.rs` | 354 | 6 | — | Goal outcome feedback loop |
| `sim_agent.rs` | 342 | 8 | 21 | Core SimAgent struct & lifecycle |
| `personality_drift.rs` | 305 | 9 | — | Personality trait drift over time |
| `intention_commitment.rs` | 278 | 11 | — | Intention & commitment tracking |
| `action_costs.rs` | 275 | 11 | 13 | Action cost computation |
| `spatial_memory.rs` | 248 | 7 | 15 | Spatial/positional memory |
| `reflection.rs` | 242 | 8 | — | Self-reflection & introspection |
| `thought_generation.rs` | 239 | 5 | — | Thought generation pipeline |
| `mod.rs` | — | — | — | Module re-exports |

---

## 4. Consciousness Subsystem (6 files, 1,641 lines)

| File | Lines | Tests | Purpose |
|------|------:|------:|---------|
| `emergence_detector.rs` | 655 | 10 | Detects emergent patterns in agent behavior |
| `behavior_vm.rs` | — | 5 | Behavior state machine / virtual machine |
| `dual_representation.rs` | — | 7 | Dual symbolic/subsymbolic representation |
| `convergence.rs` | — | 7 | Convergence detection for evolution cycles |
| `llm_hooks.rs` | — | 5 | LLM integration hooks |
| `phi_bridge.rs` | — | — | Phi (IIT integration) bridge |

---

## 5. Safety Subsystem (4 files, 1,397 lines)

| File | Lines | Tests | Purpose |
|------|------:|------:|---------|
| `safety_monitor.rs` | 469 | 5 | Runtime safety monitoring & clamping |
| `audit_trail.rs` | 366 | 5 | Action audit trail logging |
| `capability_tracker.rs` | 285 | 5 | Capability level tracking |
| `evolution_constraints.rs` | 265 | 7 | Evolution safety constraints (R-P1 compliance) |

---

## 6. Environment Subsystem (5 files, 1,079 lines)

| File | Lines | Tests | Purpose |
|------|------:|------:|---------|
| `terrain/resources.rs` | 244 | 4 | Resource nodes (food, energy, materials) |
| `gpu_grid.rs` | — | 8 | GPU-accelerated spatial grid |
| `structures.rs` | — | 7 | Environmental structures |
| `terrain/heightmap.rs` | — | 4 | Heightmap terrain generation |
| `terrain/biome.rs` | — | 6 | Biome classification |

---

## 7. Other Subsystems

| Module | Files | Lines | Key Components |
|--------|------:|------:|----------------|
| `society/` | 3 | 683 | Theory of mind, constitutional rules, economy |
| `evolution/` | — | 600 | Evolution simulation |
| `feel/` | 1 | 552 | EmotionEngine (PAD model), 9 emotion types, SystemEvent |
| `world/` | 2 | 408 | World state, fleet management, obstacles |
| `bridge/` | 1 | 375 | Host bridge (sim ↔ core communication) |
| `physics/` | 1 | 258 | Vec3, PhysicsState, 14-DOF dynamics |
| `core_bridge/` | 1 | 201 | Emotion → attention weight mapping |
| `actuator/` | 1 | 194 | ServoBus, joint limits |
| `sensor/` | 1 | 171 | SensorArray, battery, IMU |

---

## 8. Architecture

```
SimState (lib.rs)
├── physics::PhysicsState      — rigid body dynamics
├── sensor::SensorArray        — 61-dim observation vector
├── actuator::ServoBus         — 14-DOF actuator model
├── feel::EmotionEngine        — PAD emotion model
├── core_bridge::CoreBridge    — emotion → attention weights
├── world::World               — obstacles, commands
└── tick() loop:
    1. physics.step()
    2. sensors.update()
    3. observe() → [f32; 61]
    4. policy() → [f32; 14]
    5. safety() → clamp + NaN rejection
    6. actuators.write_goals()
    7. collect_events() → feel.process_events()
    8. core.update_from_feel()
```

**Tick rate**: 50 Hz (20ms per tick)
**Observation space**: 61 dimensions (gyro 3 + gravity 3 + joint_pos 14 + joint_vel 14 + last_action 14 + command 13)
**Action space**: 14 DOF (5 left leg + 5 right leg + 4 head)

---

## 9. Test Coverage Summary

- **42 of 65 files** contain `#[test]` blocks (64.6% file coverage)
- **325 total test functions**
- **Top-tested modules**: `agents/planning.rs` (18), `agents/memory_stream.rs` (18), `agents/graph_memory.rs` (17), `agents/action_awareness.rs` (14)
- **Untested modules**: `evolution/`, `main.rs`, several `mod.rs` files

---

## 10. Quality Indicators

| Indicator | Status |
|-----------|--------|
| Compiles cleanly | Yes (0 errors, 0 warnings) |
| TODO/FIXME markers | None found |
| Test coverage | 64.6% file coverage, 325 tests |
| Code duplication | Low — modules are well-separated |
| Documentation | Sparse — no doc comments on most public items |
| Dependencies | Minimal: serde, serde_json, tokio, rand |

---

## 11. Observations

1. **`world_sim.rs` (1,550 lines)** is the largest single file and is declared as a standalone module — likely contains an alternative or extended world simulation separate from `lib.rs`.
2. **`agents/`** is the dominant subsystem (37% of all code) with 16 files covering planning, memory, social learning, pheromone trails, and reflection.
3. **Zero TODO/FIXME markers** — codebase is clean of debt indicators.
4. **Cargo test was blocked** by the prior `cargo check` holding the artifact lock — tests could not be run in this session. Recommend re-running `cargo test -p neotrix-sim --lib` independently.
5. **Minimal dependencies** — only 4 direct deps (serde, serde_json, tokio, rand) keep the build fast and the supply chain small.
