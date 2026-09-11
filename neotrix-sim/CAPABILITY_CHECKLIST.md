# NT-WORLD-SIM Capability Checklist

> Generated: 2026-09-11 | Based on: `mod.rs` implementation + RESEARCH_V15 (20-source survey)
> Legend: ✅ Implemented | 🔧 Partial | ❌ Missing

---

## Core Simulation

- [x] **Agent spawn/death lifecycle** — `SimAgent::new()`, `agents.retain()`, `alive` flag, death via metabolize
- [x] **Resource gathering/consumption** — `ResourceDistribution::generate()`, `regenerate_all(season_mod)`, season modifiers
- [x] **Trade execution** — `AgentAction::Trade { target_id, items }`, `Economy::total_trades`
- [x] **Environmental hazards** — `Heightmap`, `BiomeMap`, terrain-based danger scoring in `SpatialMemory`
- [x] **Knowledge propagation** — `MemoryStream`, `GraphMemory`, `SpatialMemory` per-agent memory subsystems
- [ ] **Agent spawning with configurable templates** — ❌ Currently only random spawn; need spawn profiles (tribe, class, role) + initial resource allocation. Module: `world_sim/spawn.rs` | P1 | ~120 LOC
- [ ] **Dynamic world events** — ❌ No weather system, natural disasters, resource booms/busts driven by world state. Module: `environment/events.rs` | P1 | ~200 LOC
- [ ] **Day/night cycle affecting behavior** — 🔧 Clock exists but agents don't change behavior based on time-of-day. Module: `world_sim/daynight.rs` | P1 | ~80 LOC
- [ ] **World serialization (save/load)** — 🔧 `WorldSnapshot` exists but no deserialization or save-to-disk. Module: `world_sim/persistence.rs` | P0 | ~250 LOC
- [ ] **Deterministic replay** — ❌ No seed-locked replay for debugging. Module: `world_sim/replay.rs` | P1 | ~150 LOC

---

## Agent Intelligence

- [x] **Behavior tree** — Implicit in `decide_action_by_id()` decision pipeline (Selector/Sequence pattern)
- [x] **Utility AI** — `build_observation()` → scoring → `decide_action_by_id()` (SUSS-style consideration curves)
- [x] **FSM state machine** — `SimAgent` alive/resting/moving states, `AgentAction` enum
- [x] **Perception system** — `SpatialGrid::query_radius()`, `build_observation()` returns `AgentObservation`
- [ ] **GOAP planning** — 🔧 `PlanningStack` exists but only generates goals, no A*-based action chain discovery. Module: `agents/planning/goap.rs` | P1 | ~300 LOC
- [ ] **Explicit behavior tree nodes** — ❌ No composable BT nodes (Sequence/Selector/Decorator/Leaf). Module: `agents/behavior_tree/` | P1 | ~250 LOC
- [ ] **Stimulus-response layer** — ❌ No reflexive reaction layer below deliberation (fight/flight/freeze). Module: `agents/stimulus.rs` | P1 | ~100 LOC
- [ ] **Multi-objective optimization** — ❌ Single fitness scalar; need Pareto-optimal tradeoffs (survival vs social vs exploration). Module: `agents/moo.rs` | P2 | ~180 LOC
- [ ] **Curiosity/intrinsic motivation** — ❌ No ICM (Intrinsic Curiosity Module) or novelty bonus in action selection. Module: `agents/curiosity.rs` | P2 | ~120 LOC
- [ ] **Emotional action biasing** — 🔧 `EmotionEngine` exists but doesn't modulate action selection (anger→aggression). Module: `agents/emotional_bias.rs` | P1 | ~100 LOC

---

## Memory Systems

- [x] **Episodic memory** — `MemoryStream` with `MemoryKind::Observation`, importance decay, access tracking
- [x] **Working memory** — `MemoryStream` (capacity 200), `recent_importance_sum()` for reflection triggers
- [ ] **Semantic memory** — ❌ No structured knowledge store (facts, categories, relations). `GraphMemory` is event-focused, not concept-focused. Module: `agents/semantic_memory.rs` | P1 | ~200 LOC
- [ ] **Memory consolidation** — ❌ No sleep-like consolidation (short-term → long-term transfer, forgetting). Module: `agents/memory_consolidation.rs` | P1 | ~150 LOC
- [ ] **Spatial memory persistence** — 🔧 `SpatialMemory` exists but no persistence across death/rebirth. Module: `agents/spatial_memory.rs` (extend) | P2 | ~60 LOC
- [ ] **Memory retrieval (cue-based)** — ❌ No similarity-based recall; memories stored but not searched by relevance. Module: `agents/memory_retrieval.rs` | P1 | ~120 LOC
- [ ] **Counterfactual memory** — ❌ No "what if" simulation of alternative action outcomes. Module: `agents/counterfactual.rs` | P2 | ~150 LOC
- [ ] **Memory decay/expiry** — ❌ Memories never fully deleted; `last_accessed_tick` exists but no TTL. Module: `agents/memory_decay.rs` | P1 | ~60 LOC

---

## Social Dynamics

- [x] **Relationship graph** — `RelationshipGraph` with weighted edges, `update_interaction()`, `neighbors()`
- [x] **Reputation system** — `TheoryOfMind` tracks per-agent observations, compliance scores
- [x] **Culture propagation** — `Culture` module exists with culture state
- [x] **Cooperation/emergence** — `ConstitutionalFeedback` enforces norms, `SocialLearning` propagates behavior
- [ ] **Faction system** — ❌ No faction identity, territory, or inter-faction relations. Module: `society/faction.rs` | P1 | ~250 LOC
- [ ] **Gossip/diffusion** — ❌ `SocialLearning` exists but no gossip protocol for rapid information spread. Module: `society/gossip.rs` | P1 | ~150 LOC
- [ ] **Alliance formation** — ❌ No dynamic alliance creation/dissolution based on shared interests. Module: `society/alliance.rs` | P2 | ~180 LOC
- [ ] **Conflict resolution** — ❌ No negotiation, mediation, or warfare mechanics between agents. Module: `society/conflict.rs` | P2 | ~200 LOC
- [ ] **Leadership/hierarchy** — ❌ No emergent leadership or dominance hierarchy. Module: `society/hierarchy.rs` | P2 | ~150 LOC
- [ ] **Trade negotiation** — 🔧 `AgentAction::Trade` exists but no negotiation (take-it-or-leave-it). Module: `society/negotiation.rs` | P1 | ~120 LOC
- [ ] **Social norms evolution** — 🔧 `ConstitutionalFeedback` decays but doesn't evolve norms from behavior. Module: `society/norm_evolution.rs` | P2 | ~100 LOC
- [ ] **Communication channel** — ❌ No structured message passing; `AgentAction::Talk` is primitive. Module: `society/communication.rs` | P1 | ~180 LOC

---

## Evolution

- [x] **Genetic algorithm** — `FitnessLandscape`, `SelectionPressure`, `MutationOps`, `Speciation`, `evolution_cycle()`
- [ ] **Self-play** — ❌ No mechanism for agents to compete against historical versions of themselves. Module: `evolution/self_play.rs` | P1 | ~200 LOC
- [ ] **Curriculum learning** — ❌ No progressive difficulty scaling; all agents face same world from start. Module: `evolution/curriculum.rs` | P2 | ~150 LOC
- [ ] **Novelty search** — ❌ Only fitness-based selection; no novelty/behavior-diversity metric. Module: `evolution/novelty.rs` | P1 | ~180 LOC
- [ ] **Neuroevolution (NEAT/rtNEAT)** — ❌ No neural network evolution; only behavioral parameter mutation. Module: `evolution/neuroevolution.rs` | P2 | ~300 LOC
- [ ] **Coevolution (arms race)** — ❌ Species evolve independently; no predator-prey or host-parasite dynamics. Module: `evolution/coevolution.rs` | P2 | ~200 LOC
- [ ] **Archive of elites** — ❌ No preservation of best-performing agents across generations. Module: `evolution/archive.rs` | P1 | ~100 LOC
- [ ] **Fitness sharing/niching** — ❌ Speciation exists but no fitness sharing to prevent domination. Module: `evolution/sharing.rs` | P2 | ~80 LOC
- [ ] **Transfer learning** — ❌ No skill transfer between agents or generations. Module: `evolution/transfer.rs` | P2 | ~150 LOC

---

## UI/Game Layer

- [ ] **World rendering** — ❌ No visual output; simulation runs headless. Needs Bevy 2D renderer. Module: `ui/renderer.rs` | P0 | ~400 LOC
- [ ] **Agent selection** — ❌ No click-to-select agents for inspection. Module: `ui/agent_inspector.rs` | P1 | ~150 LOC
- [ ] **Camera controls** — ❌ No pan/zoom/scroll for world navigation. Module: `ui/camera.rs` | P1 | ~100 LOC
- [ ] **UI panels** — ❌ No HUD, minimap, agent stats, or simulation controls. Module: `ui/panels/` | P1 | ~300 LOC
- [ ] **Save/load UI** — ❌ No GUI for save state management. Module: `ui/save_load.rs` | P2 | ~120 LOC
- [ ] **Time controls** — ❌ No pause/play/speed slider. Module: `ui/time_control.rs` | P1 | ~80 LOC
- [ ] **Graph visualizer** — ❌ No visualization of relationship graphs or memory networks. Module: `ui/graph_viz.rs` | P2 | ~200 LOC
- [ ] **Statistics dashboard** — ❌ No population charts, fitness graphs, economy plots. Module: `ui/stats.rs` | P2 | ~200 LOC
- [ ] **Agent tooltip** — ❌ No hover/click tooltip showing agent state. Module: `ui/tooltip.rs` | P2 | ~100 LOC
- [ ] **Event log panel** — ❌ No scrolling event feed. Module: `ui/event_log.rs` | P2 | ~80 LOC

---

## Pathfinding & Movement

- [x] **Spatial grid** — `SpatialGrid::query_radius()` for neighbor queries
- [x] **Basic movement** — `AgentAction::Explore { direction }` with speed
- [ ] **A* pathfinding** — ❌ No global pathfinding; agents move in straight lines. Module: `navigation/astar.rs` | P0 | ~200 LOC
- [ ] **Local avoidance (RVO)** — ❌ No collision avoidance between agents. Module: `navigation/rvo.rs` | P1 | ~250 LOC
- [ ] **Hierarchical pathfinding (HPA*)** — ❌ No cluster-based large-world pathfinding. Module: `navigation/hpa.rs` | P2 | ~300 LOC
- [ ] **Flow fields** — ❌ No gradient-based multi-agent path following. Module: `navigation/flow_field.rs` | P2 | ~150 LOC
- [ ] **NavMesh** — ❌ No walkable area definition. Module: `navigation/navmesh.rs` | P2 | ~200 LOC

---

## Economy & Resources

- [x] **Resource distribution** — `ResourceDistribution::generate()` with biome-based placement
- [x] **Resource regeneration** — `regenerate_all(season_mod)` with seasonal modifiers
- [x] **Trade execution** — `AgentAction::Trade` + `Economy` tracker
- [ ] **Supply/demand pricing** — ❌ No dynamic pricing based on scarcity. Module: `economy/pricing.rs` | P1 | ~120 LOC
- [ ] **Currency system** — ❌ No medium of exchange; trade is barter-only. Module: `economy/currency.rs` | P2 | ~150 LOC
- [ ] **Marketplace** — ❌ No centralized or distributed exchange. Module: `economy/marketplace.rs` | P2 | ~200 LOC
- [ ] **Crafting/production** — ❌ No resource combination into higher-value items. Module: `economy/crafting.rs` | P2 | ~250 LOC
- [ ] **Property/ownership** — ❌ No territory claims or resource ownership. Module: `economy/property.rs` | P2 | ~120 LOC
- [ ] **Economic indicators** — ❌ No Gini coefficient, trade volume, or wealth distribution tracking. Module: `economy/indicators.rs` | P2 | ~100 LOC

---

## Safety & Monitoring

- [x] **Safety monitor** — `SafetyMonitor`, `CapabilityTracker`, `EvolutionConstraints`
- [x] **Audit trail** — `AuditTrail` with 10K capacity, `SafetyViolation` events
- [x] **Personality drift detection** — `check_personality_drift()` with threshold alerts
- [ ] **Agent containment** — ❌ No forced shutdown when agent exceeds behavioral bounds. Module: `safety/containment.rs` | P1 | ~120 LOC
- [ ] **Evolution sandbox** — ❌ No isolation for experimental mutations before production. Module: `safety/sandbox.rs` | P2 | ~150 LOC
- [ ] **Resource budget enforcement** — ❌ `ActionBudget` exists but no hard cap enforcement. Module: `safety/budget_enforcement.rs` | P1 | ~80 LOC
- [ ] **Behavioral anomaly detection** — ❌ No statistical outlier detection for agent behavior. Module: `safety/anomaly.rs` | P2 | ~150 LOC

---

## Consciousness Integration

- [x] **Phi bridge** — `PhiBridge` registers agents, tracks IIT integration
- [x] **Coherence tracker** — `CoherenceTracker` with mean_phi/mean_coherence
- [x] **Convergence detector** — `ConvergenceDetector` records fitness mean
- [x] **Dual representation** — `DualRepresentation` with 16 slots
- [ ] **GWT attention routing** — ❌ No salience-based broadcast across agents. Module: `consciousness/gwt.rs` | P1 | ~200 LOC
- [ ] **Consciousness threshold gating** — ❌ Agents always fully conscious; no variable awareness levels. Module: `consciousness/threshold.rs` | P2 | ~100 LOC
- [ ] **Phi-driven behavior modulation** — ❌ Phi computed but not used in decisions. Module: `consciousness/phi_modulation.rs` | P2 | ~80 LOC

---

## Summary

| Category | Implemented | Missing | Partial |
|----------|:-----------:|:-------:|:-------:|
| Core Simulation | 5 | 5 | 2 |
| Agent Intelligence | 4 | 6 | 2 |
| Memory Systems | 2 | 6 | 2 |
| Social Dynamics | 4 | 8 | 3 |
| Evolution | 1 | 8 | 0 |
| UI/Game Layer | 0 | 10 | 0 |
| Pathfinding & Movement | 2 | 5 | 0 |
| Economy & Resources | 3 | 6 | 0 |
| Safety & Monitoring | 3 | 4 | 0 |
| Consciousness Integration | 4 | 3 | 0 |
| **TOTAL** | **28** | **61** | **9** |

### Priority Distribution

| Priority | Count | Description |
|----------|:-----:|-------------|
| **P0** | 3 | Save/load, world rendering, A* pathfinding |
| **P1** | 28 | Spawn templates, GOAP, BT nodes, faction, self-play, novelty, camera, time controls, etc. |
| **P2** | 30 | MOO, curiosity, neuroevolution, currency, crafting, NavMesh, etc. |

### Estimated Total Missing LOC

| Category | LOC |
|----------|----:|
| Core Simulation | ~880 |
| Agent Intelligence | ~1,050 |
| Memory Systems | ~860 |
| Social Dynamics | ~1,330 |
| Evolution | ~1,380 |
| UI/Game Layer | ~1,830 |
| Pathfinding & Movement | ~1,100 |
| Economy & Resources | ~940 |
| Safety & Monitoring | ~500 |
| Consciousness Integration | ~380 |
| **TOTAL** | **~10,250** |

### Recommended Build Order

1. **P0 Foundation** — Save/load, A* pathfinding, world renderer (makes sim observable + debuggable)
2. **P1 Intelligence** — GOAP, BT nodes, emotional bias, memory consolidation (deeper agent behavior)
3. **P1 Social** — Factions, gossip, negotiation, communication (emergent social dynamics)
4. **P1 Evolution** — Self-play, novelty search, archive (accelerates evolution)
5. **P2 Advanced** — Neuroevolution, currency, NavMesh, curriculum, anomaly detection
