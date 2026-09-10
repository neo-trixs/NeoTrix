# NT-WORLD-SIM Complete Evolution Task List

## Executive Summary

NT-WORLD-SIM is a 248-test, 0-warning simulation engine implementing 7 agent subsystems (memory, planning, personality, theory-of-mind, reflection, action-awareness, emotion) plus world infrastructure (terrain, evolution, relationships, economy, culture). Phase 1-2 completed core wiring (R1+M1-M10). This document is the master plan for the remaining evolution: wiring 5 dead fields into the tick loop, implementing agent lifecycle mechanics, building cross-module interactions, scaling to 1000+ agents, and establishing the testing/quality baseline for production readiness.

## Current State

| Metric | Value |
|--------|-------|
| Tests | 248 pass, 0 fail |
| Warnings | 0 (after Phase 1-2 cleanup) |
| `world_sim.rs` lines | 986 |
| Dead fields (initialized, never used in tick) | 5: `heightmap`, `biome_map`, `economy`, `culture`, `dual_repr` |
| Dead modules (declared but not wired) | 0 (event_reactive now wired in Reflex tier) |
| `event_reactive` | Wired: processes bus events in Reflex tier (lines 403-431) |
| GraphMemory write+query | Wired: M5 nodes+edges in Fast tier (lines 332-358) |
| SpatialMemory write | Wired: M6 visit in Fast tier (lines 362-374) |
| ActionAwareness.should_explore() | Wired: M8 in decide_action (lines 677-683) |
| ConvergenceDetector | Wired: M9 mutation rate modulation in evolution_cycle (lines 819-836) |
| ConstitutionalFeedback | Wired: M10 type-safe evaluate_action (line 276) |
| PlanningStack | Wired: M1 goal-driven decisions (lines 554-580) |
| PersonalityDrift | Wired: M2 bias in decide_action (lines 636-654) |
| TheoryOfMind | Wired: M3 social decisions (lines 601-633) |
| ReflectionEngine | Wired: M4 triggers in Slow tier (lines 467-483) |
| Emotion modulation | Wired: M7 dominant emotion in decide_action (lines 657-675) |

---

## Task Queue (Priority-Ordered)

### P0: Core Integration — Wire Dead Fields + Fix Architecture Gaps

These tasks must complete before any capability expansion. They resolve the 5 dead fields and close architecture gaps identified in the inspection report.

| ID | Task | Module | Dependencies | Est. Lines | Status |
|----|------|--------|--------------|------------|--------|
| P0-1 | **Wire `heightmap` into `build_observation()`** — Query heightmap at agent position to replace hardcoded `"plain"` terrain_type. Provide actual elevation, slope, and terrain features to agent perception. | environment/terrain + world_sim | None | +30 | TODO |
| P0-2 | **Wire `biome_map` into `build_observation()`** — Query biome classification at agent position. Expose biome type (forest, desert, ocean, mountain, etc.) and associated properties (resource density, danger level) to agent observation. | environment/terrain + world_sim | P0-1 | +25 | TODO |
| P0-3 | **Wire `economy` into `execute_action()` Trade branch** — When `AgentAction::Trade` executes, update economy market prices based on supply/demand. Track trade volume, record trades in economy log. Currently Trade action is not in the match (falls to `_ => {}`). | society/economy + world_sim | None | +40 | TODO |
| P0-4 | **Wire `culture` into social decisions** — In `decide_action()` when agents Talk, propagate memes via `culture.spread_meme()`. Create memes from novel agent behaviors. Culture compliance modulates social action selection. | society/culture + world_sim | None | +35 | TODO |
| P0-5 | **Wire `dual_repr` into tick loop** — After each action, call `dual_repr.add(action_label, "action", attrs, tick)` to build dual symbolic+numeric representation. In `decide_action()`, use `dual_repr.similar_to()` to find historically similar situations. | consciousness/dual_representation + world_sim | None | +30 | TODO |
| P0-6 | **Wire `spatial_memory` into planning exploration goals** — `planning.generate_exploration_goals()` should use `spatial_memory` to prefer unvisited or high-value locations. Currently generates random exploration goals. | agents/planning + agents/spatial_memory | P0-5 | +25 | TODO |
| P0-7 | **Wire `graph_memory` into reflection insights** — When `reflection.reflect()` produces insights, store them as `NodeKind::Insight` in graph_memory with citation edges to the memories that produced them. | agents/reflection + agents/graph_memory | None | +20 | TODO |
| P0-8 | **Wire `spatial_memory` into `decide_action()` exploration** — Use `spatial_memory.safe_locations()` to steer agents away from known-dangerous areas. Currently exploration is random direction. | agents/spatial_memory + world_sim | P0-6 | +20 | TODO |
| P0-9 | **Add AgentAction::Trade to execute_action() match** — The Trade variant exists in the enum but `execute_action()` has no handler for it (falls to `_ => {}`). Implement trade execution between two agents. | world_sim + agents/sim_agent | P0-3 | +50 | TODO |
| P0-10 | **Add AgentAction::Attack handler to execute_action()** — Attack variant exists but execute_action() has no handler. Implement damage calculation, health reduction, and event emission (AgentNearDeath/AgentDied). | world_sim + agents/sim_agent | None | +40 | TODO |
| P0-11 | **Add `AgentAction::Gather` action variant** — Currently Eat is the only resource interaction. Add explicit Gather for collecting resources into agent inventory (needed for trade mechanics). | agents/sim_agent | P0-3 | +30 | TODO |
| P0-12 | **Unify `decide_action()` into pipeline architecture** — Replace the 130-line if-else chain with the pipeline from REFACTOR_PLAN 5.1: personality_bias → emotion_bias → social_bias → planned → explore_signal → candidates → filter → score → select. | world_sim | P0-1 through P0-8 | +80 | TODO |

**P0 Subtotal: ~425 lines, ~12 tasks**

---

### P1: Capability Expansion — Agent Lifecycle, Resources, Social Mechanics

These tasks add the missing simulation capabilities that make the world feel alive.

| ID | Task | Module | Dependencies | Est. Lines | Status |
|----|------|--------|--------------|------------|--------|
| P1-1 | **Agent spawn lifecycle** — When evolution creates offspring, initialize ALL per-agent subsystems (memory_streams, graph_memories, spatial_memories, planning, reflections, action_budgets, personality_drift, theory_of_mind, action_awareness, event_reactive subscriptions). Currently only phi_bridge is registered. | world_sim + agents/* | None | +40 | TODO |
| P1-2 | **Agent death cleanup** — When agents die (evolution elimination or health=0), clean up all per-agent HashMap entries to prevent memory leaks. Unsubscribe from event_reactive. Emit SimEvent::AgentDied. | world_sim | P1-1 | +25 | TODO |
| P1-3 | **Agent inventory system** — Add `inventory: Inventory` field to SimAgent. Track gathered resources. Inventory affects trade capability, survival, and tool crafting prerequisites. | agents/sim_agent + society/economy | P0-11 | +50 | TODO |
| P1-4 | **Resource gathering mechanics** — Gather action harvests resources from the world into agent inventory. Resource type depends on biome. Amount depends on agent skills and tool quality. | world_sim + environment | P0-11, P1-3 | +35 | TODO |
| P1-5 | **Resource consumption mechanics** — Agents consume food/water periodically (metabolism). Depletion causes health reduction. Starvation → death. Currently metabolism only reduces energy. | agents/sim_agent + world_sim | P1-3 | +30 | TODO |
| P1-6 | **Inter-agent trade execution** — Full trade flow: Agent A proposes Trade to Agent B. B evaluates based on economy prices, inventory, and relationship. Accept/reject. Economy updates. Relationship improves on successful trade. | world_sim + society/* | P0-9, P1-3 | +60 | TODO |
| P1-7 | **Knowledge propagation (social learning)** — When agents Talk, they share memory summaries. Receiving agent adds shared memories to their MemoryStream. Knowledge spreads through social network. | world_sim + agents/memory_stream + society/relationship_graph | P0-7 | +45 | TODO |
| P1-8 | **Environmental hazards** — Biome-dependent hazards: forest fires, floods, droughts. Hazard events emitted on SimulationBus. EventReactiveSystem triggers agent responses. Damage applied to agents in hazard zone. | environment + foundation/simulation_bus + agents/event_reactive | P0-2 | +60 | TODO |
| P1-9 | **Tool crafting** — Agents in Workshop structures can combine resources to create tools. Tools modify action costs (reduced gather cost, increased attack damage). Tools stored in inventory. | world_sim + environment/structures + agents/sim_agent | P1-3, P1-4 | +55 | TODO |
| P1-10 | **Territory claiming** — Agents build structures in a region → claim territory. Territory gives resource bonuses. Other agents entering territory trigger social negotiation or conflict. | world_sim + environment/structures + society/* | P1-6 | +50 | TODO |
| P1-11 | **Emotion→behavior feedback loop** — Currently emotion modulates decide_action() but actions don't feed back into emotion. Add: successful trade → Joy, attack received → Anger/Fear, exploration → Curiosity. | world_sim + feel + agents/sim_agent | None | +25 | TODO |
| P1-12 | **TheoryOfMind prediction integration** — Use `tom.predict_action(target)` in decide_action() to anticipate nearby agents' next moves and plan counter-strategies or cooperative actions. | world_sim + society/theory_of_mind | P0-8 | +30 | TODO |
| P1-13 | **Memory consolidation scheduling** — Periodic (Background tier) consolidation: merge low-importance MemoryStream nodes, prune old GraphMemory nodes, compress SpatialMemory history. Prevents unbounded memory growth. | agents/memory_stream + agents/graph_memory + agents/spatial_memory | None | +40 | TODO |
| P1-14 | **Action budget risk modulation** — In decide_action(), when agent health is low, use `action_budget.risk_per_action()` to avoid high-risk actions. Currently risk_per_action() is computed but never read (D12). | world_sim + agents/action_costs | None | +15 | TODO |
| P1-15 | **Personality skills filling** — In evolution_cycle(), after offspring spawn, populate `SimAgent.skills` based on action history of parent agents. Currently `skills` is never filled (D15). | world_sim + evolution | None | +20 | TODO |

**P1 Subtotal: ~580 lines, ~15 tasks**

---

### P2: Optimization — Performance, Scale, Architecture

These tasks improve performance and enable scaling to 1000+ agents.

| ID | Task | Module | Dependencies | Est. Lines | Status |
|----|------|--------|--------------|------------|--------|
| P2-1 | **Batch agent processing** — Replace sequential agent loop with batch processing. Collect all observations first (immutable), then all decisions (mutable in isolated scope), then all executions. Reduces borrow conflicts and enables parallelism. | world_sim | P0-12 | +60 | TODO |
| P2-2 | **Spatial partitioning optimization** — SpatialGrid uses brute-force radius query O(n). Upgrade to uniform grid or quadtree for O(1) neighbor lookup. Critical for 1000+ agents. | foundation/math_bridge | None | +80 | TODO |
| P2-3 | **Extract `cosine_sim` to `foundation/math_bridge`** — R2: cosine_sim is duplicated in memory_stream.rs and dual_representation.rs. Extract to shared utility. | foundation/math_bridge + agents/memory_stream + consciousness/dual_representation | None | +15 | TODO |
| P2-4 | **HashMap→Vec optimization for per-agent data** — Replace `HashMap<String, X>` for per-agent subsystems with `Vec<(String, X)>` or indexed arena. Reduces hash overhead for tight loops. | world_sim | P2-1 | +40 | TODO |
| P2-5 | **Lazy subsystem initialization** — Per-agent subsystems are initialized in Fast tier on first encounter. Move initialization to spawn time and use `Option<X>` with `get_or_insert` only for late-joining agents. | world_sim | P1-1 | +20 | TODO |
| P2-6 | **GraphMemory spread_activation caching** — Cache spread_activation results per tick to avoid redundant graph traversals when multiple decision points query the same agent. | agents/graph_memory | None | +25 | TODO |
| P2-7 | **EmotionEngine batch processing** — Process all agent emotions in a single pass instead of per-agent. Aggregate system events once, apply to all agents simultaneously. | feel + world_sim | None | +20 | TODO |
| P2-8 | **Evolution cycle streaming** — Stream genome evaluation instead of collecting all genomes into a Vec. Reduces peak memory for large populations. | evolution + world_sim | None | +30 | TODO |
| P2-9 | **GPU acceleration hooks** — Add optional GPU compute paths for: fitness evaluation, spatial queries, emotion processing. Use feature flags for CPU fallback. | foundation + evolution + feel | None | +50 | TODO |
| P2-10 | **Event batch emission** — Batch SimulationBus emissions per tick instead of per-event. Reduces lock contention on event history. | foundation/simulation_bus + world_sim | None | +25 | TODO |
| P2-11 | **Tick schedule adaptive** — Dynamic tier intervals based on population pressure. More agents → faster resource regen checks, slower evolution. | foundation/tick_schedule | None | +20 | TODO |
| P2-12 | **Memory pool allocator** — Pre-allocate agent subsystem objects in a pool to avoid per-agent heap allocation overhead during spawn. | world_sim + agents/* | P1-1 | +35 | TODO |

**P2 Subtotal: ~420 lines, ~12 tasks**

---

### P3: Testing & Quality — Integration, Stress, Property-based, Benchmarks

| ID | Task | Module | Dependencies | Est. Lines | Status |
|----|------|--------|--------------|------------|--------|
| P3-1 | **WorldSim integration test: 50-tick loop** — Create `tests/world_sim_integration.rs` that runs `WorldSim::tick()` 50 times with 10 agents, asserting: agents move, resources deplete, memories grow, evolution runs, no panics. | tests/ | P0-1 through P0-12 | +120 | TODO |
| P3-2 | **WorldSim integration test: 200-tick evolution** — Run 200 ticks to trigger multiple evolution cycles. Assert: population fluctuates, species form, fitness trends upward, phi values stabilize. | tests/ | P3-1 | +100 | TODO |
| P3-3 | **Trade execution integration test** — Set up 2 agents with complementary inventories. Execute trade. Assert: inventories swap, economy prices update, relationship improves. | tests/ | P0-9, P1-3, P1-6 | +80 | TODO |
| P3-4 | **EventReactive full loop test** — Emit AgentDied event → verify nearby agents receive reactive responses → verify agents actually move/rest/attack. | tests/ | P0-1 through P0-3 | +60 | TODO |
| P3-5 | **Knowledge propagation test** — Two agents Talk → verify shared memories appear in both MemoryStreams. Three-agent chain → verify knowledge propagates through network. | tests/ | P1-7 | +60 | TODO |
| P3-6 | **1000-agent stress test** — Spawn 1000 agents, run 100 ticks. Measure: tick duration < 100ms, no memory leaks, no panics, population remains bounded. | tests/ | P2-1, P2-2 | +80 | TODO |
| P3-7 | **Property-based: evolution monotonicity** — `proptest` test that fitness mean is non-decreasing over 50 generations (with probabilistic tolerance). | tests/ | None | +40 | TODO |
| P3-8 | **Property-based: action cost conservation** — `proptest` test that total energy consumed across all agents equals sum of individual action costs (within floating-point tolerance). | tests/ | None | +40 | TODO |
| P3-9 | **Property-based: memory bounded** — `proptest` test that MemoryStream size never exceeds capacity after N ticks of random actions. | tests/ | None | +30 | TODO |
| P3-10 | **Benchmark: tick throughput** — `criterion` benchmark: measure ticks/sec for 10, 100, 1000 agents. Establish baseline for P2 optimization comparison. | benches/ | P2-1 | +50 | TODO |
| P3-11 | **Benchmark: spatial query** — `criterion` benchmark: compare brute-force vs quadtree spatial queries at various agent densities. | benches/ | P2-2 | +40 | TODO |
| P3-12 | **Benchmark: evolution cycle** — `criterion` benchmark: measure evolution_cycle() duration for 50, 200, 500 agents. | benches/ | None | +30 | TODO |
| P3-13 | **Dead code cleanup: signal_achievement** — Move `signal_achievement()` to `#[cfg(test)]` block or remove from production code (D1). | agents/personality_drift | None | +5 | TODO |
| P3-14 | **Dead code cleanup: ValueNoise::seed** — Remove unused `seed` field or prefix with `_` (D2). | environment/terrain/heightmap | None | +3 | TODO |
| P3-15 | **Dead code cleanup: DualRepresentation::embedding_size** — Wire into `generate_embedding()` to produce variable-length embeddings, or remove field (D3). | consciousness/dual_representation | P0-5 | +10 | TODO |
| P3-16 | **graph_memory.rs:127 irrefutable while-let fix** — Replace `while let ((id, depth)) = queue.pop_front()?` with explicit `loop { match queue.pop_front() { ... } }` (W2). | agents/graph_memory | None | +8 | TODO |
| P3-17 | **HostBridge → WorldSim connection** — Wire HostBridge into tick loop for external observation/control. Currently exists but unused. | bridge/host_bridge + world_sim | None | +30 | TODO |
| P3-18 | **SimEvent::AgentDied emission in evolution** — When agents are eliminated in evolution_cycle(), emit AgentDied event on the bus so event_reactive can trigger responses. | world_sim | P1-2 | +10 | TODO |

**P3 Subtotal: ~806 lines, ~18 tasks**

---

### P4: Documentation & Architecture

| ID | Task | Module | Dependencies | Est. Lines | Status |
|----|------|--------|--------------|------------|--------|
| P4-1 | **ADR: Decision Pipeline Architecture** — Document the unified decide_action() pipeline (personality → emotion → ToM → planning → exploration → scoring → filtering). Record rationale for ordering. | docs/ | P0-12 | +100 | TODO |
| P4-2 | **ADR: Memory Architecture** — Document three-memory system (MemoryStream=episodic, GraphMemory=semantic, SpatialMemory=environmental) and their interactions. | docs/ | P0-7 | +80 | TODO |
| P4-3 | **ADR: Evolution-MetaCognition Coupling** — Document how ConvergenceDetector modulates mutation rates and how Phi/Coherence influence selection pressure. | docs/ | P0-12 | +60 | TODO |
| P4-4 | **Module interaction diagram** — Create ASCII/Mermaid diagram showing all module interactions in the tick loop: Reflex → Fast → Medium → Slow → Background tiers with all subsystem calls. | docs/ | P0-12 | +50 | TODO |
| P4-5 | **API documentation for WorldSim** — Document all public methods, fields, and their roles. Include safety invariants and usage examples. | world_sim | None | +80 | TODO |
| P4-6 | **Agent subsystem API docs** — Document each agent module's public API: MemoryStream, GraphMemory, SpatialMemory, PlanningStack, ReflectionEngine, ActionAwareness, PersonalityDrift, TheoryOfMind. | agents/* | None | +120 | TODO |
| P4-7 | **Simulation configuration guide** — Document WorldSimConfig fields, tuning guidelines, and expected behaviors for different parameter ranges. | docs/ | None | +40 | TODO |
| P4-8 | **Evolution task status tracker update** — After each task completion, update this document's Status column. | docs/ | All | +5 | TODO |

**P4 Subtotal: ~535 lines, ~8 tasks**

---

## Dependency Graph

```
P0-1 (heightmap) ──────────────────────────────────────────────────┐
P0-2 (biome_map) ──depends──► P0-1                                │
P0-3 (economy) ────────────────────────────────────────────────────┤
P0-4 (culture) ────────────────────────────────────────────────────┤
P0-5 (dual_repr) ──────────────────────────────────────────────────┤
P0-6 (spatial→planning) ──depends──► P0-5                         ├──► P0-12 (unify pipeline)
P0-7 (graph→reflection) ──────────────────────────────────────────┤
P0-8 (spatial→decide) ──depends──► P0-6                           │
P0-9 (Trade execute) ──depends──► P0-3                            │
P0-10 (Attack execute) ───────────────────────────────────────────┤
P0-11 (Gather action) ──depends──► P0-3                           │
                                                                      │
P0-12 (unify pipeline) ──depends──► ALL P0-* ─────────────────────┘
                                                                      │
P1-1 (spawn lifecycle) ────────────────────────────────────────────┤
P1-2 (death cleanup) ──depends──► P1-1                            ├──► P2-1 (batch)
P1-3 (inventory) ──depends──► P0-11                               │
P1-4 (gathering) ──depends──► P0-11, P1-3                         │
P1-5 (consumption) ──depends──► P1-3                              │
P1-6 (trade execution) ──depends──► P0-9, P1-3                   │
P1-7 (knowledge propagation) ──depends──► P0-7                    │
P1-8 (hazards) ──depends──► P0-2                                  │
P1-9 (tool crafting) ──depends──► P1-3, P1-4                      │
P1-10 (territory) ──depends──► P1-6                               │
P1-11 (emotion feedback) ─────────────────────────────────────────┤
P1-12 (ToM prediction) ──depends──► P0-8                          │
P1-13 (memory consolidation) ─────────────────────────────────────┘
P1-14 (risk modulation) ──────────────────────────────────────────┤
P1-15 (skills filling) ───────────────────────────────────────────┘
                                                                      │
P2-1 (batch) ──depends──► P0-12, P1-2                             │
P2-2 (spatial partition) ─────────────────────────────────────────┤
P2-3 (cosine_sim) ────────────────────────────────────────────────┤
P2-4 (HashMap→Vec) ──depends──► P2-1                              ├──► P3-6 (1000-agent)
P2-5 (lazy init) ──depends──► P1-1                                │
P2-6 (activation cache) ──────────────────────────────────────────┤
P2-7 (emotion batch) ─────────────────────────────────────────────┤
P2-8 (evolution streaming) ───────────────────────────────────────┤
P2-9 (GPU hooks) ─────────────────────────────────────────────────┘
P2-10 (event batch) ──────────────────────────────────────────────┤
P2-11 (adaptive schedule) ────────────────────────────────────────┤
P2-12 (memory pool) ──depends──► P1-1                             ┘

P3-1 (integration 50-tick) ──depends──► ALL P0-*                  ──► P3-2 (200-tick)
P3-3 (trade test) ──depends──► P0-9, P1-3, P1-6                   ──► P3-6
P3-4 (event reactive test) ──depends──► P0-1~P0-3                 ──► P3-6
P3-5 (knowledge test) ──depends──► P1-7                            ──► P3-6
P3-6 (1000-agent) ──depends──► P2-1, P2-2, P3-1
P3-7 (proptest fitness) ──────────────────────────────────────────
P3-8 (proptest cost) ─────────────────────────────────────────────
P3-9 (proptest memory) ───────────────────────────────────────────
P3-10 (bench tick) ──depends──► P2-1                               ──► P3-11, P3-12
P3-11 (bench spatial) ──depends──► P2-2
P3-12 (bench evolution) ──────────────────────────────────────────
```

---

## Execution Timeline

### Week 1: Core Integration (P0)

| Day | Tasks | Deliverable |
|-----|-------|-------------|
| Day 1 | P0-1, P0-2, P0-3, P0-4 | heightmap/biome_map/economy/culture wired into tick |
| Day 2 | P0-5, P0-6, P0-7, P0-8 | dual_repr/spatial→planning/graph→reflection wired |
| Day 3 | P0-9, P0-10, P0-11 | Trade/Attack/Gather action handlers implemented |
| Day 4 | P0-12 | Unified decide_action() pipeline — all if-else replaced |
| Day 5 | P3-13, P3-14, P3-15, P3-16, P3-17, P3-18 | Dead code cleanup + warnings fix |

**Week 1 Exit Criteria**: All P0 tasks complete. 248+ tests pass. `cargo clippy` clean. `decide_action()` is a clean pipeline.

### Week 2: Capability Expansion (P1)

| Day | Tasks | Deliverable |
|-----|-------|-------------|
| Day 1 | P1-1, P1-2 | Agent spawn/death lifecycle fully managed |
| Day 2 | P1-3, P1-4, P1-5 | Inventory + gathering + consumption mechanics |
| Day 3 | P1-6, P1-10 | Trade execution + territory claiming |
| Day 4 | P1-7, P1-8, P1-11 | Knowledge propagation + hazards + emotion feedback |
| Day 5 | P1-9, P1-12, P1-13, P1-14, P1-15 | Tool crafting + ToM prediction + consolidation + risk + skills |

**Week 2 Exit Criteria**: All P1 tasks complete. Agents can spawn, die, gather, trade, share knowledge, craft tools, claim territory. 280+ tests.

### Week 3: Optimization (P2)

| Day | Tasks | Deliverable |
|-----|-------|-------------|
| Day 1 | P2-1, P2-5, P2-12 | Batch processing + lazy init + memory pool |
| Day 2 | P2-2, P2-3, P2-6 | Spatial partition + cosine_sim dedup + activation cache |
| Day 3 | P2-4, P2-7, P2-8 | HashMap→Vec + emotion batch + evolution streaming |
| Day 4 | P2-9, P2-10, P2-11 | GPU hooks + event batch + adaptive schedule |
| Day 5 | Benchmark runs, profiling | Baseline performance numbers |

**Week 3 Exit Criteria**: 1000-agent tick < 100ms. All P2 tasks complete.

### Week 4: Testing, Docs, Ship (P3+P4)

| Day | Tasks | Deliverable |
|-----|-------|-------------|
| Day 1 | P3-1, P3-2 | Integration tests (50-tick + 200-tick) |
| Day 2 | P3-3, P3-4, P3-5 | Trade/Event/Knowledge integration tests |
| Day 3 | P3-6 | 1000-agent stress test passes |
| Day 4 | P3-7, P3-8, P3-9, P3-10, P3-11, P3-12 | Property-based + benchmark suite |
| Day 5 | P4-1 through P4-8 | ADRs + API docs + diagrams + status tracker |

**Week 4 Exit Criteria**: All tests pass. 330+ total tests. All docs complete. Release-ready.

---

## Summary Statistics

| Priority | Tasks | Est. Lines | Est. Effort |
|----------|-------|-----------|-------------|
| P0: Core Integration | 12 | ~425 | 3 days |
| P1: Capability Expansion | 15 | ~580 | 4 days |
| P2: Optimization | 12 | ~420 | 3 days |
| P3: Testing & Quality | 18 | ~806 | 3 days |
| P4: Documentation | 8 | ~535 | 2 days |
| **Total** | **65** | **~2,766** | **~15 days** |

---

*Generated by NT-WORLD-SIM task planning agent. All tasks are evidence-based (source code analysis + inspection report + refactor plan). Status fields to be updated as tasks complete.*
