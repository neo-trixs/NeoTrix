# ECS Integration Plan — Bevy ECS for neotrix-sim

**Date**: 2026-09-11
**Status**: Plan (not yet implemented)
**Target**: Migrate `world_sim.rs` (1286 lines) to Bevy ECS substrate

---

## Executive Summary

The current `WorldSim` is a 1286-line monolith with 28 fields on a single struct, 11+ per-agent `HashMap<String, T>` lookups, and 7 methods that all take `&mut self`. Every tick clones agent IDs to work around borrow conflicts. ECS eliminates this by making each subsystem a standalone function with explicit component queries.

**Key benefits**:
- Parallel agent processing (currently sequential)
- Change detection (perception only processes changed state)
- Plugin architecture (NT-* domains become Bevy plugins)
- Eliminates all `HashMap<String, T>` per-agent lookups

**Estimated effort**: ~2500 lines of new code, ~800 lines deleted, net +1700 lines over 5 phases.

---

## 1. Current Architecture Assessment

### 1.1 Entity Inventory

| Entity Type | Count | Storage | Notes |
|-------------|-------|---------|-------|
| Agents | 10-50 | `Vec<SimAgent>` | Spawned at init, culled on death |
| Resource Nodes | ~200 | `ResourceDistribution.nodes` | Per-biome distributed |
| Structures | 0-N | `StructureManager` | Built by agents at runtime |

### 1.2 Per-Agent State (11 HashMap lookups per tick)

| Field | Type | Accessed In |
|-------|------|-------------|
| `action_awareness` | `HashMap<String, ActionAwareness>` | decide, verify |
| `memory_streams` | `HashMap<String, MemoryStream>` | record, reflect |
| `graph_memories` | `HashMap<String, GraphMemory>` | record actions |
| `spatial_memories` | `HashMap<String, SpatialMemory>` | record visits |
| `planning` | `HashMap<String, PlanningStack>` | goal generation, next_action |
| `reflections` | `HashMap<String, ReflectionEngine>` | importance-triggered reflect |
| `action_budgets` | `HashMap<String, ActionBudget>` | cost tracking |
| `personality_drift` | `HashMap<String, PersonalityDrift>` | record, drift |
| `theory_of_mind` | `HashMap<String, TheoryOfMind>` | observe, threat_of |
| `action_costs` | `ActionCostTable` | can_afford (shared) |
| `dual_repr` | `DualRepresentation` | cycle-breaking (shared) |

### 1.3 Global Shared State (14 fields)

| Field | Type | Read By | Write By |
|-------|------|---------|----------|
| `heightmap` | `Heightmap` | build_observation, layer_survival | never |
| `biome_map` | `BiomeMap` | build_observation | never |
| `resources` | `ResourceDistribution` | build_observation, execute_action | execute_action (harvest) |
| `spatial_grid` | `SpatialGrid` | build_observation (query_radius) | tick (rebuild) |
| `phi_bridge` | `PhiBridge` | compute_consciousness_metrics | tick, evolution |
| `coherence_tracker` | `CoherenceTracker` | compute_consciousness_metrics | tick |
| `landscape` | `FitnessLandscape` | evolution_cycle | never |
| `selection` | `SelectionPressure` | evolution_cycle | never |
| `mutator` | `MutationOps` | evolution_cycle | evolution_cycle |
| `speciation` | `Speciation` | evolution_cycle | evolution_cycle |
| `relationships` | `RelationshipGraph` | build_observation, execute_action | execute_action |
| `economy` | `Economy` | layer_survival, execute_action | execute_action |
| `culture` | `Culture` | layer_social, execute_action | execute_action |
| `emotion` | `EmotionEngine` | layer_personality, collect_world_events | tick (process_events) |
| `structures` | `StructureManager` | execute_action | execute_action |
| `constitutional` | `ConstitutionalFeedback` | tick (compliance check) | tick (decay) |
| `convergence` | `ConvergenceDetector` | evolution_cycle | tick |
| `event_reactive` | `EventReactiveSystem` | tick (reactive events) | never |
| `bus` | `SimulationBus` | tick (emit events) | tick (emit events) |
| `clock` | `SimClock` | tick (time) | tick (advance) |
| `schedule` | `TickSchedule` | tick (tier checks) | tick (advance) |

### 1.4 System Execution Tiers

| Tier | Frequency | Systems |
|------|-----------|---------|
| Reflex | Every tick | Time advance, resource regen, spatial grid rebuild, metabolism |
| Fast | Every 5 ticks | Agent perception → decision → action (5 layers) → execute → memory record → ToM observe |
| Medium | Every 20 ticks | Consciousness metrics (phi, coherence) |
| Slow | Every 100 ticks | Emotion update, planning goals, reflection, personality drift, constitutional decay, convergence |
| Background | Every 500 ticks | Evolution cycle (fitness, selection, mutation, speciation) |

### 1.5 Borrow Checker Pain Points

1. **Clone-for-ID pattern** (lines 261-285): Agent IDs cloned into `Vec<String>` before iteration because `build_observation` borrows `&self` while `decide_action` needs `&mut self`.
2. **Index juggling** (lines 277-284): `self.agents.iter().position()` + `self.agents[idx]` to avoid double-borrow.
3. **Temporary clone** (line 308): Agent cloned just to pass to `action_awareness.predict()` while `self` is mutably borrowed.
4. **Double iteration** (lines 470-474): Agents iterated once for planning, again for reflection — each requiring `self.agents.iter().find()` + `self.planning.get_mut()`.

---

## 2. Component Mapping

### 2.1 Agent Components (per-entity bundle)

| Current Struct | ECS Component | Notes |
|----------------|---------------|-------|
| `SimAgent.core` | `AgentCore` | position, energy, health, hunger, alive, age |
| `SimAgent.personality` | `AgentPersonality` | sociability, curiosity, cooperativeness, aggression |
| `SimAgent.recent_actions` | `ActionHistory` | VecDeque of recent actions |
| `action_awareness[id]` | `ActionAwareness` | predict/verify/should_explore |
| `memory_streams[id]` | `MemoryStream` | 200-node working memory |
| `graph_memories[id]` | `GraphMemory` | 500-node episodic graph |
| `spatial_memories[id]` | `SpatialMemory` | visit history at positions |
| `planning[id]` | `PlanningStack` | survival/social/exploration goals |
| `reflections[id]` | `ReflectionEngine` | importance-triggered reflection |
| `action_budgets[id]` | `ActionBudget` | cost tracking per action type |
| `personality_drift[id]` | `PersonalityDrift` | accumulated drift signals |
| `theory_of_mind[id]` | `TheoryOfMind` | threat/cooperativeness of others |

**Agent Entity Bundle**:
```rust
#[derive(Bundle)]
struct AgentBundle {
    core: AgentCore,
    personality: AgentPersonality,
    action_history: ActionHistory,
    awareness: ActionAwareness,
    memory: MemoryStream,
    graph_memory: GraphMemory,
    spatial_memory: SpatialMemory,
    planning: PlanningStack,
    reflection: ReflectionEngine,
    budget: ActionBudget,
    drift: PersonalityDrift,
    tom: TheoryOfMind,
}
```

### 2.2 World Components (singleton entities or Resources)

| Current Struct | ECS Type | Notes |
|----------------|----------|-------|
| `heightmap` | `Resource(Heightmap)` | Read-only after init |
| `biome_map` | `Resource(BiomeMap)` | Read-only after init |
| `resources` | `Resource(ResourceNodes)` | Mutable (harvest/regen) |
| `spatial_grid` | `Resource(SpatialGrid)` | Rebuilt every tick |
| `structures` | `Resource(StructureManager)` | Mutable (build) |
| `relationships` | `Resource(RelationshipGraph)` | Mutable (interactions) |
| `economy` | `Resource(Economy)` | Mutable (trades) |
| `culture` | `Resource(Culture)` | Mutable (meme spread) |
| `emotion` | `Resource(EmotionEngine)` | Mutable (process events) |
| `phi_bridge` | `Resource(PhiBridge)` | Mutable (update phi) |
| `coherence_tracker` | `Resource(CoherenceTracker)` | Mutable (compute) |
| `landscape` | `Resource(FitnessLandscape)` | Read-only during eval |
| `selection` | `Resource(SelectionPressure)` | Read-only during select |
| `mutator` | `Resource(MutationOps)` | Mutable (breed) |
| `speciation` | `Resource(Speciation)` | Mutable (speciate) |
| `constitutional` | `Resource(ConstitutionalFeedback)` | Mutable (decay) |
| `convergence` | `Resource(ConvergenceDetector)` | Mutable (record) |
| `dual_repr` | `Resource(DualRepresentation)` | Mutable (encode) |
| `event_reactive` | `Resource(EventReactiveSystem)` | Read-only per tick |
| `action_costs` | `Resource(ActionCostTable)` | Read-only per tick |
| `bus` | `Resource(SimulationBus)` | Mutable (emit) |
| `clock` | `Resource(SimClock)` | Mutable (advance) |
| `schedule` | `Resource(TickSchedule)` | Mutable (advance) |
| `config` | `Resource(WorldSimConfig)` | Read-only |
| `rng` | `Resource(SimulationRng)` | Mutable (random) |

### 2.3 Resource Node Components

| Current Struct | ECS Component | Notes |
|----------------|---------------|-------|
| ResourceNode fields | `ResourceNode` component on resource entity | position, type, amount, depleted |

### 2.4 Structure Components

| Current Struct | ECS Component | Notes |
|----------------|---------------|-------|
| Structure fields | `Structure` component on structure entity | position, type, owner, health |

---

## 3. System Mapping

### 3.1 Reflex Systems (every tick)

| Current Code | ECS System | Inputs | Outputs |
|--------------|-----------|--------|---------|
| `clock.tick()` | `advance_time` | `ResMut<SimClock>`, `ResMut<SimulationBus>` | Time events |
| `resources.regenerate_all()` | `regenerate_resources` | `ResMut<ResourceNodes>`, `Res<SimClock>` | — |
| `spatial_grid` rebuild | `rebuild_spatial_grid` | `Query<(&AgentCore,)>, ResMut<SpatialGrid>` | — |
| `agent.core.metabolize()` | `agent_metabolism` | `Query<(&mut AgentCore,)>, Res<TimeModifiers>` | — |

### 3.2 Fast Systems (every 5 ticks)

| Current Code | ECS System | Notes |
|--------------|-----------|-------|
| `build_observation` | `build_observations` | Queries: `AgentCore`, `SpatialGrid`, `ResourceNodes`, `BiomeMap`, `Heightmap`, `RelationshipGraph` |
| `decide_action` (5 layers) | `decide_actions` | Runs 5 sub-systems in order. Each returns `Option<AgentAction>` |
| `execute_action` | `execute_actions` | Writes to `AgentCore`, `ResourceNodes`, `RelationshipGraph`, `Culture`, `Economy`, etc. |
| Memory record (6 subsystems) | `record_agent_memories` | Writes to per-agent `MemoryStream`, `GraphMemory`, `SpatialMemory`, `ActionHistory` |
| ToM observe | `tom_observe` | Writes to `TheoryOfMind` |
| Personality drift record | `drift_record` | Writes to `PersonalityDrift` |

### 3.3 Medium Systems (every 20 ticks)

| Current Code | ECS System |
|--------------|-----------|
| `compute_consciousness_metrics` | `compute_phi_coherence` |

### 3.4 Slow Systems (every 100 ticks)

| Current Code | ECS System |
|--------------|-----------|
| Emotion process events | `process_emotion` |
| Planning generate goals | `generate_planning_goals` |
| Reflection trigger | `trigger_reflections` |
| Personality drift apply | `apply_personality_drift` |
| Constitutional decay | `decay_constitutional` |
| Convergence detection | `detect_convergence` |
| ActionAwareness learn | `learn_action_awareness` |

### 3.5 Background Systems (every 500 ticks)

| Current Code | ECS System |
|--------------|-----------|
| `evolution_cycle` | `run_evolution` |

### 3.6 Decision Layer Systems (sub-systems of decide_actions)

| Layer | ECS System | Priority |
|-------|-----------|----------|
| `layer_survival` | `decision_survival` | 1 (highest) |
| `layer_goals` | `decision_goals` | 2 |
| `layer_social` | `decision_social` | 3 |
| `layer_personality` | `decision_personality` | 4 |
| `layer_default` | `decision_default` | 5 (lowest) |

Each returns `Option<AgentAction>`. First `Some` wins. This replaces the current 5-layer cascade.

### 3.7 Plugin Architecture

```rust
// Main app setup
fn main() {
    App::new()
        .add_plugins(NeotrixCorePlugin)     // E8, HyperCube, GWT
        .add_plugins(NeotrixWorldPlugin)    // terrain, resources, structures
        .add_plugins(NeotrixAgentPlugin)    // agent components + systems
        .add_plugins(NeotrixFEELPlugin)     // emotion engine
        .add_plugins(NeotrixEvolutionPlugin)// fitness, selection, mutation
        .add_plugins(NeotrixConsciousnessPlugin) // phi, coherence
        .add_plugins(NeotrixSocietyPlugin)  // relationships, economy, culture
        .add_plugins(NeotrixShieldPlugin)   // constitutional, convergence
        .run();
}
```

Each plugin registers its own components, resources, and systems with appropriate `SystemSet` scheduling.

---

## 4. Migration Strategy

### Phase 1: Dependency + Component Types (1-2 days, ~400 LOC)

**Goal**: Add `bevy_ecs` as dependency, define all component/resource types, keep existing code working.

**Steps**:
1. Add `bevy_ecs` to `Cargo.toml` (standalone, no rendering)
2. Create `src/ecs/components.rs` — define `AgentBundle`, all per-agent components
3. Create `src/ecs/resources.rs` — define resource wrapper types for all global state
4. Create `src/ecs/mod.rs` — module declarations
5. Write unit tests for component creation/despawn
6. **Existing code unchanged** — new ECS types coexist with old `WorldSim`

**Verification**: `cargo test -p neotrix-sim --lib` passes, no existing tests break.

### Phase 2: ECS World Setup + Agent Migration (2-3 days, ~600 LOC)

**Goal**: Create Bevy `World`, spawn agent entities, verify query mechanics.

**Steps**:
1. Create `src/ecs/setup.rs` — `WorldSimEcs::new()` that creates Bevy World + spawns initial agents
2. Migrate `SimAgent` fields into `AgentBundle` components
3. Create `src/ecs/bridge.rs` — adapter that reads from Bevy World into old `WorldSim` format (temporary)
4. Write tests: spawn agents, query components, verify data round-trips
5. Keep old `WorldSim::tick()` working via bridge

**Verification**: Can create Bevy World, spawn 10 agents, query all components.

### Phase 3: System Migration (5-7 days, ~1000 LOC)

**Goal**: Migrate tick() logic into ECS systems, one tier at a time.

**Order** (bottom-up, least coupled first):

1. **Reflex systems** (1 day): `advance_time`, `regenerate_resources`, `rebuild_spatial_grid`, `agent_metabolism`
   - Simplest: no inter-agent dependencies
   - Test: reflex systems produce same output as old tick()

2. **Medium/Slow systems** (2 days): consciousness metrics, emotion, planning, reflection, convergence
   - Mostly per-agent, some shared resources
   - Test: metrics match old values within epsilon

3. **Fast systems** (3-4 days): observation, decision (5 layers), execution, memory recording
   - Most complex: reads from many resources, writes to agent components
   - Decision layers must preserve priority order
   - Test: agent actions match old system for deterministic seeds

4. **Background systems** (1 day): evolution cycle
   - Batch operation on all agents
   - Test: evolution records match

### Phase 4: Parallelization + Performance (2-3 days, ~300 LOC)

**Goal**: Leverage Bevy's parallel scheduler.

**Steps**:
1. Add `SystemSet` constraints to prevent ordering violations
2. Enable Bevy's parallel scheduling for independent agent processing
3. Benchmark: compare tick throughput (old vs new) at 10, 50, 100 agents
4. Profile: identify bottlenecks (likely `SpatialGrid` rebuild, `build_observation`)
5. Optimize: use `ParamSet` for conflicting access, `Without<>` filters for exclusive queries

**Expected gains**:
- Agent perception/decision: ~4-8x on 8-core machine (parallel per-agent)
- Spatial grid: ~2x (parallel insert)
- Evolution: ~2x (parallel fitness eval)

### Phase 5: Cleanup (1-2 days, ~200 LOC deletion)

**Goal**: Remove old `WorldSim` code, finalize.

**Steps**:
1. Remove `WorldSim` struct and all methods
2. Remove bridge adapter (Phase 2)
3. Rename `WorldSimEcs` → `WorldSim`
4. Update CLI/UI to use new API
5. Remove unused imports
6. Final `cargo clippy` + `cargo test`

**Verification**: All tests pass, no dead code warnings.

---

## 5. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Performance regression during migration** | Medium | Bridge adapter lets old/new coexist. Benchmark before switching. |
| **Breaking existing tests** | High | Phase 1-2 keep old code working. New tests written per phase. |
| **Increased complexity** | Medium | ECS is more explicit but less ad-hoc. Good documentation offsets. |
| **Bevy version dependency** | Low | `bevy_ecs` is stable, semver-guaranteed. Pin to specific version. |
| **Borrow checker surprises in systems** | Medium | Use `ParamSet` for conflicting access. Test each system in isolation. |
| **Decision layer ordering** | High | Must preserve 5-layer priority. Test with deterministic seeds. |
| **Shared resource contention** | Medium | Use `Res` vs `ResMut` explicitly. `ParamSet` for conflicting writes. |
| **Async compatibility** | Low | Current `tick()` is async. ECS systems are sync. Use `async` system wrapper or restructure. |

### Test Strategy

| Phase | Test Approach |
|-------|--------------|
| 1 | Unit tests for component/resource creation |
| 2 | Integration tests: spawn → query → round-trip |
| 3 | Golden tests: deterministic seed → compare old/new output |
| 4 | Benchmarks: throughput at various agent counts |
| 5 | Full test suite regression |

### Decision Layer Integrity Test

```rust
#[test]
fn decision_layers_preserve_priority() {
    let mut world = setup_ecs_world();
    // Spawn agent with high hunger, low energy, nearby threat
    // Old system: returns Eat (survival layer wins)
    // New system: must also return Eat
    let action = run_decision_systems(&mut world, agent_id);
    assert!(matches!(action, AgentAction::Eat { .. }));
}
```

---

## 6. Estimated Effort

| Phase | New LOC | Deleted LOC | Net LOC | Days |
|-------|---------|-------------|---------|------|
| Phase 1: Components | 400 | 0 | +400 | 1-2 |
| Phase 2: Setup + Bridge | 600 | 0 | +600 | 2-3 |
| Phase 3: Systems | 1000 | 400 | +600 | 5-7 |
| Phase 4: Parallelization | 300 | 100 | +200 | 2-3 |
| Phase 5: Cleanup | 200 | 800 | -600 | 1-2 |
| **Total** | **2500** | **1300** | **+1200** | **11-17** |

### File Structure After Migration

```
src/
├── world_sim.rs          # Thin wrapper: creates Bevy App, runs tick
├── ecs/
│   ├── mod.rs            # Module declarations
│   ├── components.rs     # AgentBundle, per-agent components
│   ├── resources.rs      # Resource wrapper types
│   ├── setup.rs          # WorldSimEcs::new(), entity spawning
│   ├── bridge.rs         # Adapter (temporary, deleted in Phase 5)
│   ├── reflex.rs         # Reflex-tier systems
│   ├── fast.rs           # Fast-tier systems (perception, decision, action)
│   ├── medium.rs         # Medium-tier systems (consciousness)
│   ├── slow.rs           # Slow-tier systems (emotion, planning, reflection)
│   ├── background.rs     # Background-tier systems (evolution)
│   └── decisions.rs      # 5-layer decision sub-systems
├── agents/               # Unchanged (components reference these structs)
├── environment/          # Unchanged
├── consciousness/        # Unchanged
├── evolution/            # Unchanged
├── feel/                 # Unchanged
└── society/              # Unchanged
```

---

## 7. Dependency Decision

**Use `bevy_ecs` standalone** (not full `bevy`):

```toml
[dependencies]
bevy_ecs = "0.15"  # standalone ECS, no rendering
```

Rationale from RESEARCH_V3:
- "Bevy ECS usable as standalone crate" (§8)
- All large-scale sims (AgentSociety 10K+, GenSim 100K+, Project Sid 1000+) use ECS-like archetypes
- No rendering needed — this is pure simulation
- Change detection + parallel scheduler are the key wins

---

## 8. Open Questions

1. **Agent entity lifetime**: Should dead agents be despawned (clean) or have `Alive(false)` component (reuse)?
   - **Recommendation**: Keep alive flag, despawn only on evolution elimination. Matches current behavior.

2. **SpatialGrid as Resource vs System**: Rebuild grid in a system or as explicit resource mutation?
   - **Recommendation**: Resource + dedicated system. Clear data ownership.

3. **EventBus integration**: Keep `SimulationBus` as Resource or replace with Bevy Events?
   - **Recommendation**: Keep `SimulationBus` as Resource for now. Bevy Events for future.

4. **Observation pre-computation**: Build all observations in parallel, then decision reads pre-computed?
   - **Recommendation**: Yes. Two-pass: `build_observations` → `decide_actions`. Avoids redundant spatial queries.
