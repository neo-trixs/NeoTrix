# NT-WORLD-SIM Architecture Fusion Analysis

> **Date**: 2026-09-11
> **Scope**: External technology mapping → redundancy detection → defect identification → cross-domain misalignment → refactoring recommendations

---

## 1. External Technology Fusion Inventory

### 1.1 Bevy ECS (Data-Oriented Architecture)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Archetype-based storage: entities with identical component sets share contiguous columnar tables (SoA). Systems declare `Query<D, F>` for data access; Bevy auto-parallelizes non-conflicting systems. Components are plain data; systems are free functions. |
| **NeoTrix 映射** | `WorldSim` has 28 fields on a single struct, 11 `HashMap<String, T>` per-agent lookups, sequential `tick()`. `Vec<SimAgent>` with `clone-for-ID` borrow hack. ECS_PLAN.md already proposes Bevy migration. |
| **融合方案** | **增强现有节点**: `bevy_ecs` standalone crate (no rendering). Agent → `AgentBundle` (12 components). Global state → Resources. Decision layers → systems with `SystemSet` scheduling. |
| **冗余检测** | SpatialGrid (custom HashMap) vs Bevy's archetype-based storage. SpatialGrid should become a Resource, not replace archetype iteration. |
| **扁平化缺陷** | Current: flat `Vec<SimAgent>` + flat HashMaps. Missing: archetype-based storage (auto-groups by component set), change detection (`Added<T>`/`Changed<T>`), deferred structural mutations (`Commands`), parallel system execution. |
| **跨域错位** | ECS is an **infrastructure** pattern, not an AI pattern. Using Bevy ECS as pure substrate while keeping OOP-style decision logic would miss the point. Each decision layer should be an independent ECS system with explicit query parameters. |

### 1.2 GOAP (Goal-Oriented Action Planning)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | World state = `HashMap<String, bool>`. Actions = `{preconditions, effects, cost}`. Goals = desired world states with priority. A* search finds minimum-cost action chain from current state to goal state. F.E.A.R. AI paper (Orkin 2006). |
| **NeoTrix 映射** | `agents/planning/goap.rs` implements full GOAP: `GOAPState`, `GOAPAction`, `GOAPPlanner` with A*. **Already exists (303 lines)**. `PlanningStack` uses it. But `decide_action()` in `decision.rs` bypasses it — survival layer and goal layer use hardcoded if-else. |
| **融合方案** | **接入现有节点**: Wire `GOAPPlanner::find_plan()` into `layer_goals()` in `decision.rs`. Currently `layer_goals()` at decision.rs:553 is 140-line if-else. Replace with: assess world state → select highest-priority goal with viable plan → execute first action. |
| **冗余检测** | `PlanningStack::generate_survival_goals()` + `generate_social_goals()` + `generate_exploration_goals()` produce goals, but `decide_action()` never calls `planning.next_action()`. GOAP planner exists but is dead code in the decision pipeline. |
| **扁平化缺陷** | GOAP exists but lacks: (1) hierarchical goal decomposition (Goal→SubGoal→Action), (2) reactive replanning on world change, (3) action layer filtering (avoid checking "sleep" when running from danger), (4) utility-weighted goal selection. |
| **跨域错位** | GOAP is **decision infrastructure** (NT-CORE/NT-ACT), not memory or social. Current mapping puts it in `agents/planning/` which is correct, but the planner should be an **ECS system** that runs on the Fast tier, not a method on `WorldSim`. |

### 1.3 Utility AI (Consideration Scoring)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Each action scored by `response_curve(consideration)`. Considerations: distance, hunger, social need, danger. Final score = product (or sum) of considerations. Highest-scored action selected. "Big Brain" crate for Bevy. |
| **NeoTrix 映射** | `decision.rs` `decide_action()` is a 6-layer priority cascade (survival → goals → social → stigmergy → personality → default). Each layer returns `Option<AgentAction>`. This is a **hybrid**: priority-layered with utility-like scoring within layers (e.g., `layer_social` scores agents by distance+relationship). |
| **融合方案** | **增强现有节点**: Refactor each layer to return `(AgentAction, f32)` score. Top-level selector picks highest score across layers. This converts the priority cascade into a proper utility system with layer-weighted scoring. |
| **冗余检测** | `layer_personality` and `layer_stigmergy` both influence exploration direction. `layer_social` and `layer_goals` both consider nearby agents. No explicit scoring function — just early returns. |
| **扁平化缺陷** | Missing: (1) configurable response curves (linear, exponential, logistic), (2) consideration weights learnable from experience, (3) emotional modulation of consideration weights (anger → aggression consideration boost), (4) temporal discounting (urgent needs scored higher). |
| **跨域错位** | Utility scoring is **NT-CORE** (reasoning), but emotion modulation is **NT-FEEL**. Cross-domain coupling: `EmotionEngine` should feed into utility scores via `GwtModulation` (which exists but is unused in decision pipeline). |

### 1.4 Behavior Tree (BT)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Tree of composable nodes: `Selector` (try children until success), `Sequence` (run children until failure), `Decorator` (modify child result), `Leaf` (action/condition). State machine represented as tree traversal. Interruptible via `Running` state. |
| **NeoTrix 映射** | `agents/behavior_tree/mod.rs` implements `BehaviorNode` trait, `Selector`, `Sequence`, `Blackboard`. **Already exists (244 lines)**. But **never used** — `decide_action()` doesn't reference it. Decision pipeline uses if-else cascade, not BT traversal. |
| **融合方案** | **增强现有节点**: Replace `decide_action()` cascade with BT root: `Selector { survival_sequence, goal_sequence, social_sequence, exploration_sequence }`. Each sequence wraps the corresponding layer's logic as BT nodes. Blackboard stores shared agent state. |
| **冗余检测** | BT exists as dead code. `decide_action()` reimplements what BT should do (priority-ordered fallback). `EventReactiveSystem` is a manual event→response mapping that could be BT nodes. |
| **扁平化缺陷** | Missing: (1) `Decorator` nodes (inverter, repeater, cooldown), (2) `Parallel` node (run multiple children simultaneously), (3) `Condition` nodes (boolean checks as first-class), (4) visual/debug BT state tracking. |
| **跨域错位** | BT is **NT-ACT** (action execution), but conditions need **NT-CORE** (perception) and **NT-FEEL** (emotion). Cross-domain: BT conditions should query agent's `EmotionEngine` state, not just raw observations. |

### 1.5 Spatial Indexing (Quadtree / Spatial Hash Grid)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Uniform grid: O(1) cell lookup, O(k) within-cell search. Quadtree: adaptive subdivision, O(log n) for sparse regions. Both reduce O(n²) neighbor queries to O(n + k). Game engines use both. |
| **NeoTrix 映射** | `foundation/math_bridge.rs:229` implements `SpatialGrid` with `HashMap<(i32,i32), Vec<(String, Vec2)>>`. Cell size = 50.0. `query_radius()` is O(cells_in_radius × cell_size). Already functional. |
| **融合方案** | **增强现有节点**: Upgrade `SpatialGrid` to support concurrent reads via `DashMap` (P1-7 in EVOLUTION_TASKS). Add quadtree variant for adaptive resolution. Keep as Resource, not component. |
| **冗余检测** | `SpatialGrid` (foundation) vs `SpatialMemory` (per-agent visited locations). Different purposes: SpatialGrid = global spatial lookup, SpatialMemory = per-agent memory of locations. No redundancy — both needed. |
| **扁平化缺陷** | Missing: (1) concurrent read/write (DashMap), (2) batch rebuild (clear+insert all in one pass), (3) AABB query (not just radius), (4) query with filters (type, owner, state). |
| **跨域错位** | SpatialGrid is **foundation** (infrastructure), but queries are used by **NT-WORLD** (observation), **NT-ACT** (pathfinding), **NT-SHIELD** (threat detection). Single grid for all domains is correct but needs thread-safe access for parallel agent processing. |

### 1.6 Event-Driven Architecture

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Decouple producers from consumers via event bus. Producers emit typed events; consumers subscribe to event types. Three layers: channels (system-level), entity events, property change notifications. Enables reactive behavior without polling. |
| **NeoTrix 映射** | `foundation/simulation_bus.rs` implements `SimulationBus` with `SimEvent` enum (20+ variants), priority routing, event history. **Wired**: `tick_reactive_events()` processes bus history in Reflex tier. `EventReactiveSystem` processes events into agent responses. |
| **融合方案** | **增强现有节点**: SimulationBus is functional but underutilized. Wire more events: `ResourceDepleted` → agents relocate, `AgentNearDeath` → nearby agents respond, `FactionConflict` → agents choose sides. Add per-agent event subscriptions (currently global). |
| **冗余检测** | `EventReactiveSystem` (agents) vs `SimulationBus` (foundation) vs `PheromoneField` (environment). All three are event/communication mechanisms. `EventReactiveSystem` should consume from `SimulationBus`, not duplicate event routing. `PheromoneField` is physical-world communication (stigmergy), different from digital events. |
| **扁平化缺陷** | Missing: (1) per-agent event subscriptions (agent only receives relevant events), (2) event-driven state changes (property observation pattern), (3) event replay for deterministic debugging, (4) event filtering by domain. |
| **跨域错位** | Events span **all domains**: `SimEvent::AgentDied` (NT-ACT) → `EventReactiveSystem` (NT-ACT) → `EmotionEngine` (NT-FEEL) → `SafetyMonitor` (NT-SHIELD). Current bus is flat; needs domain-aware routing. |

### 1.7 A* Pathfinding

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Graph search with heuristic h(n). f(n) = g(n) + h(n). Expands lowest-f admissibly. 8-directional grid movement. Returns path as `Vec<GridPos>`. |
| **NeoTrix 映射** | `navigation/astar.rs` implements full A* (288 lines): `GridPos`, `AStar` with obstacles/movement_cost, `find_path()`. **Exists but not wired** — `execute_action()` moves agents in straight lines toward target, never calls A*. |
| **融合方案** | **接入现有节点**: In `execute_action()`, when `AgentAction::Move { target }`, call `AStar::find_path(agent.position, target)` and follow path waypoints. Need to convert world coordinates to grid coordinates. |
| **冗余检测** | A* exists but `Move` action just does `position + direction * speed`. No pathfinding in the actual movement. A* is dead code. |
| **扁平化缺陷** | Missing: (1) hierarchical pathfinding (HPA*) for large worlds, (2) flow fields for multi-agent flocking, (3) NavMesh for non-grid terrain, (4) path caching (don't recompute every tick). |
| **跨域错位** | Pathfinding is **NT-ACT** (movement), but terrain data is **NT-WORLD** (heightmap/biome). A* needs heightmap for walkability. Cross-domain: A* should query `Heightmap` and `BiomeMap` for terrain cost, not just obstacles array. |

### 1.8 RVO (Reciprocal Velocity Obstacles)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Local collision avoidance. Each agent considers velocity obstacles of neighbors. Computes optimal velocity that avoids collisions while moving toward target. Reciprocal: agent assumes others also avoid. |
| **NeoTrix 映射** | `navigation/rvo.rs` implements `RVOSimulator` (158 lines): `RVOAgent`, `add_agent()`, `compute_new_velocity()`. **Exists but not wired** — agents don't avoid each other. |
| **融合方案** | **接入现有节点**: In `execute_action()` for `Move`/`Explore`, run RVO with nearby agents from SpatialGrid before applying velocity. |
| **冗余检测** | RVO exists as dead code. SpatialGrid already provides neighbor queries needed for RVO. No redundancy — complementary systems. |
| **扁平化缺陷** | Missing: (1) ORCA (Optimal Reciprocal Collision Avoidance) for guaranteed collision-free, (2) dynamic obstacle support, (3) formation movement, (4) velocity profiling for different agent types. |
| **跨域错位** | RVO is **NT-ACT** (movement) but needs **NT-WORLD** (obstacle data). Cross-domain: RVO should consider structures and terrain as obstacles. |

### 1.9 Fog of War (Visibility System)

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Per-agent visibility radius. Three states: Unexplored (black), Explored (grey), Visible (lit). Bresenham/shadowcasting for line-of-sight. GPU rendering for visual fog. |
| **NeoTrix 映射** | **Not implemented**. Agents have full world knowledge — `build_observation()` queries all resources within 100.0 radius regardless of visibility. No fog of war mechanic. |
| **融合方案** | **新增节点**: Add `FogOfWar` component to agent entities: tracks `visible_cells: HashSet<(i32,i32)>`, `explored_cells: HashSet<(i32,i32)>`. `build_observation()` filters through fog. Shadowcasting for LOS computation. |
| **冗余检测** | `SpatialMemory` tracks per-agent visited locations — this is **explored** state. `FogOfWar` would add **visible** state (current line of sight). Complementary, not redundant. |
| **扁平化缺陷** | Missing entirely: (1) per-agent visibility state, (2) LOS computation (shadowcasting), (3) explored vs visible distinction, (4) knowledge asymmetry (agent A sees resources that agent B doesn't). |
| **跨域错位** | FoW is **NT-WORLD** (perception), but affects **NT-ACT** (decision-making based on partial info) and **NT-SHIELD** (threat detection limited to visible area). Cross-domain: FoW visibility state feeds into `build_observation()` which feeds into `decide_action()`. |

### 1.10 Influence Map

| Dimension | Detail |
|-----------|--------|
| **底层模型原理** | Grid-based spatial influence propagation. Agents deposit influence (danger, safety, territory, resource). Influence decays over distance/time. Used for strategic decision-making (avoid dangerous areas, seek safe zones). |
| **NeoTrix 映射** | `agents/pheromone.rs` implements `PheromoneField` (447 lines): 6 pheromone types (Food/Danger/Rest/Social/Explore/Territory), deposit/decay/diffusion. **Wired** in `tick_slow_systems()`. `PheromoneType::Territory` is an influence map. |
| **融合方案** | **增强现有节点**: `PheromoneField` IS an influence map. Enhance: (1) agent reads pheromone gradients for movement direction, (2) add faction-specific pheromone channels, (3) integrate with `SpatialGrid` for O(1) pheromone queries. |
| **冗余检测** | `PheromoneField` vs potential `InfluenceMap`: same concept, different naming. No redundancy. `PheromoneField` already handles deposit/decay/diffusion. |
| **扁平化缺陷** | Missing: (1) pheromone gradient-following in movement decisions, (2) multi-layer influence (danger + resource + territory), (3) agent-specific pheromone visibility, (4) pheromone interference patterns (cross-type amplification/inhibition). |
| **跨域错位** | Pheromones are **NT-ACT** (stigmergic communication) but influence **NT-CORE** (decision-making) and **NT-WORLD** (environment modification). Cross-domain: pheromone gradients should be a consideration in utility scoring. |

---

## 2. Redundancy Inventory

### 2.1 Duplicate Implementations

| # | Duplicate A | Duplicate B | Location A | Location B | Resolution |
|---|-------------|-------------|------------|------------|------------|
| R1 | `decision.rs` if-else cascade | `behavior_tree/mod.rs` Selector/Sequence | `world_sim/decision.rs` | `agents/behavior_tree/mod.rs` | **Unify**: Replace if-else cascade with BT root. BT is the formalization of what decision.rs already does. |
| R2 | `decision.rs` layer_social scoring | `theory_of_mind.rs` agent evaluation | `world_sim/decision.rs:100-140` | `society/theory_of_mind.rs` | **Unify**: `decide_action()` should call `TheoryOfMind::predict_action()` instead of reimplementing social scoring. |
| R3 | `SpatialGrid` neighbor query | `SpatialMemory` location recall | `foundation/math_bridge.rs:256` | `agents/spatial_memory.rs` | **No action**: Different scopes. SpatialGrid = global spatial index. SpatialMemory = per-agent memory of visited locations. |
| R4 | `cosine_sim` in math_bridge | Potentially duplicated in dual_representation | `foundation/math_bridge.rs:318` | `consciousness/dual_representation.rs` | **Verify**: Check if dual_representation has its own cosine_sim. If so, deduplicate to math_bridge. |
| R5 | `EventReactiveSystem` event routing | `SimulationBus` event routing | `agents/event_reactive.rs` | `foundation/simulation_bus.rs` | **Unify**: `EventReactiveSystem` should subscribe to `SimulationBus` events, not maintain separate event routing. |
| R6 | `MemoryStream` scoring | `GraphMemory` scoring | `agents/memory_stream.rs:101` | `agents/graph_memory.rs` | **Partial**: Different scoring targets (episodic vs graph). Keep separate but share scoring utilities. |
| R7 | `ActionCostTable.can_afford()` | `ActionBudget.risk_per_action()` | `agents/action_costs.rs` | `agents/action_costs.rs` | **Merge**: Both on same file, both cost-related. `can_afford()` should incorporate `risk_per_action()`. |

### 2.2 Dead Code (Modules That Exist But Aren't Wired)

| # | Module | Lines | Location | Status | Resolution |
|---|--------|-------|----------|--------|------------|
| D1 | `behavior_tree/` | 244 | `agents/behavior_tree/mod.rs` | Selector/Sequence/Decorator/Leaf defined, never instantiated | Wire as decision pipeline root |
| D2 | `navigation/astar.rs` | 288 | `navigation/astar.rs` | Full A* implemented, never called from `execute_action()` | Wire into Move/Explore actions |
| D3 | `navigation/rvo.rs` | 158 | `navigation/rvo.rs` | Full RVO implemented, never called from tick loop | Wire into movement actions |
| D4 | `planning/goap.rs` | 303 | `agents/planning/goap.rs` | Full GOAP planner implemented, bypassed by if-else | Wire into `layer_goals()` |
| D5 | `stimulus.rs` | unknown | `agents/stimulus.rs` | Exists in module list, unknown if wired | Check and wire into Reflex tier |
| D6 | `emotional_bias.rs` | unknown | `agents/emotional_bias.rs` | Exists in module list, unknown if wired | Wire into `decide_action()` personality layer |
| D7 | `intention_commitment.rs` | unknown | `agents/intention_commitment.rs` | Exists in module list, unknown if wired | Wire into planning/action execution |
| D8 | `thought_generation.rs` | unknown | `agents/thought_generation.rs` | Exists in module list, unknown if wired | Wire into Slow tier reflection |
| D9 | `consciousness/behavior_vm.rs` | ~200 | `consciousness/behavior_vm.rs` | Behavior VM defined, never used | Convert to AgentAction orchestration or remove |
| D10 | `consciousness/emergence_detector.rs` | 655 | `consciousness/emergence_detector.rs` | EmergenceDetector defined, never instantiated | Wire into Slow tier |
| D11 | `consciousness/dual_representation.rs` | ~150 | `consciousness/dual_representation.rs` | DualRepresentation defined, never used in tick | Wire into tick loop |
| D12 | `consciousness/llm_hooks.rs` | ~100 | `consciousness/llm_hooks.rs` | LLM hooks defined, never used | Keep as trait interface |
| D13 | `society/faction.rs` | 166 | `society/faction.rs` | Faction defined, never instantiated in WorldSim | Wire into society simulation |
| D14 | `society/gossip.rs` | 184 | `society/gossip.rs` | GossipProtocol defined, never instantiated | Wire into social learning |
| D15 | `society/communication.rs` | exists | `society/communication.rs` | Exists in module list, unknown if wired | Check and wire |
| D16 | `society/negotiation.rs` | exists | `society/negotiation.rs` | Exists in module list, unknown if wired | Check and wire into Trade actions |
| D17 | `economy/pricing.rs` | exists | `economy/pricing.rs` | Exists in module list, unknown if wired | Check and wire into Economy |
| D18 | `world_sim/actions.rs` Trade handler | partial | `world_sim/actions.rs:53-80` | Trade action exists but hardcodes Food→Wood swap | Make generic with economy pricing |
| D19 | `world_sim/actions.rs` Attack handler | missing | `world_sim/actions.rs` | Attack variant exists but no handler in match | Implement damage calculation |
| D20 | `world_sim/actions.rs` Gather handler | missing | `world_sim/actions.rs` | Gather variant exists but no handler in match | Implement resource collection |

---

## 3. Defect Inventory (Missing Capabilities)

### 3.1 Architecture Defects

| # | Defect | Impact | Source | Priority | LOC Estimate |
|---|--------|--------|--------|----------|-------------|
| F1 | **No GOAP integration in decision pipeline** | Agents don't plan; they react with if-else. No goal decomposition, no action sequencing. | ARCHITECTURE_ANALYSIS T3, CAPABILITY_CHECKLIST | **P0** | +200 |
| F2 | **No BT-based decision tree** | Decision pipeline is 140-line if-else, not composable or debuggable. | FUSION_PLAN A2 | **P0** | +150 (wire existing) |
| F3 | **Memory retrieval never called** | `MemoryStream::retrieve()` exists but `decide_action()` never queries it. Agents learn nothing from past experience. | FUSION_PLAN A1, D8 | **P0** | +40 (wire) |
| F4 | **No emotional modulation of decisions** | `EmotionEngine` exists and is wired into WorldSim, but `decide_action()` ignores emotion state. | FUSION_PLAN T5, EVOLUTION_TASKS P1-11 | **P0** | +60 |
| F5 | **No fog of war** | Agents have full world knowledge. No knowledge asymmetry, no exploration pressure. | CAPABILITY_CHECKLIST | **P1** | +300 |
| F6 | **No hierarchical memory consolidation** | Memories never promote from working→episodic→semantic. No forgetting. Unbounded growth. | ARCHITECTURE_ANALYSIS M2, CAPABILITY_CHECKLIST | **P1** | +200 |
| F7 | **No pathfinding integration** | A* exists but agents move in straight lines. No terrain-aware movement. | CAPABILITY_CHECKLIST | **P0** | +50 (wire) |
| F8 | **No collision avoidance** | RVO exists but agents overlap. No crowd simulation. | CAPABILITY_CHECKLIST | **P1** | +40 (wire) |
| F9 | **No per-agent event subscriptions** | `EventReactiveSystem` processes all events for all agents. No selective attention. | FUSION_PLAN R4 | **P1** | +80 |
| F10 | **No concurrent agent processing** | Sequential `tick()` loop. No parallelism for 1000+ agents. | ECS_PLAN, FUSION_PLAN R3 | **P1** | +200 (DashMap) |
| F11 | **No A*-based GOAP replanning** | `GOAPPlanner` has A* but no reactive replanning on world change. Plans are computed once and executed linearly. | CAPABILITY_CHECKLIST | **P1** | +100 |
| F12 | **No influence map gradient following** | PheromoneField exists but agents don't follow pheromone gradients for movement. | EVOLUTION_TASKS P0-8 | **P1** | +40 |
| F13 | **No faction/territory integration** | `Faction` struct defined but never used in WorldSim. No territory mechanics. | CAPABILITY_CHECKLIST | **P1** | +120 |
| F14 | **No gossip protocol integration** | `GossipProtocol` defined but never instantiated. No rapid information spread. | CAPABILITY_CHECKLIST | **P1** | +80 |

### 3.2 Integration Defects (Existing Code Not Connected)

| # | Module A | Module B | Missing Connection | Resolution |
|---|----------|----------|-------------------|------------|
| I1 | `PlannerStack` | `decide_action()` | Planner generates goals but decision uses if-else | Wire planner into `layer_goals()` |
| I2 | `MemoryStream` | `decide_action()` | Memory has retrieval but decision doesn't query | Call `retrieve()` in decision pipeline |
| I3 | `EmotionEngine` | `decide_action()` | Emotion modulates GWT but decision ignores it | Read emotion state in `decide_action()` |
| I4 | `AStar` | `execute_action()` | A* has pathfinding but movement uses straight lines | Call A* in `Move`/`Explore` actions |
| I5 | `RVO` | `execute_action()` | RVO has collision avoidance but agents overlap | Call RVO before applying velocity |
| I6 | `BehaviorTree` | `decide_action()` | BT has composable nodes but decision uses if-else | Replace if-else with BT root |
| I7 | `GOAPPlanner` | `decide_action()` | GOAP has planning but decision doesn't plan | Call planner in `layer_goals()` |
| I8 | `PheromoneField` | `decide_action()` | Pheromones exist but agents don't follow gradients | Read pheromone values as utility consideration |
| I9 | `Faction` | `WorldSim` | Faction defined but not in WorldSim struct | Add faction tracking to WorldSim |
| I10 | `GossipProtocol` | `SocialLearning` | Both exist for information spread but disconnected | GossipProtocol feeds into SocialLearning |
| I11 | `heightmap` | `build_observation()` | Heightmap queries exist but terrain_type was hardcoded | Already wired (P0-1 in EVOLUTION_TASKS) |
| I12 | `biome_map` | `build_observation()` | Biome queries exist but weren't used | Already wired (P0-2 in EVOLUTION_TASKS) |
| I13 | `economy` | `execute_action()` Trade | Trade handler exists but hardcodes Food→Wood | Make generic with pricing |
| I14 | `culture` | Social decisions | Culture exists but meme propagation limited | Extend meme spread in Talk actions |

---

## 4. Cross-Domain Misalignment Inventory

### 4.1 Domain Boundary Violations

| # | Violation | Domains | Description | Resolution |
|---|-----------|---------|-------------|------------|
| X1 | **Decision logic in WorldSim** | NT-CORE ↔ NT-WORLD | `decide_action()` is a method on `WorldSim` (NT-WORLD domain) but implements reasoning (NT-CORE). Decision logic should be in a separate NT-CORE system. | Extract `DecisionEngine` as standalone system that takes `&WorldState` as input |
| X2 | **Emotion engine in WorldSim** | NT-FEEL ↔ NT-WORLD | `EmotionEngine` is field on `WorldSim` but emotion is NT-FEEL domain. | Keep EmotionEngine as Resource, but move processing to NT-FEEL system |
| X3 | **Safety monitor in WorldSim** | NT-SHIELD ↔ NT-WORLD | `SafetyMonitor` is field on `WorldSim` but safety is NT-SHIELD domain. | Extract SafetyMonitor as NT-SHIELD system |
| X4 | **Evolution in WorldSim** | NT-MIND ↔ NT-WORLD | `evolution_cycle()` is method on `WorldSim` but evolution is NT-MIND domain. | Extract as NT-MIND system |
| X5 | **Consciousness metrics in WorldSim** | NT-CORE ↔ NT-WORLD | `compute_consciousness_metrics()` is on `WorldSim` but consciousness is NT-CORE. | Extract as NT-CORE system |
| X6 | **Social systems mixed** | NT-ACT ↔ NT-SHIELD ↔ NT-CORE | `relationships`, `economy`, `culture`, `theory_of_mind` all on WorldSim but span NT-ACT, NT-SHIELD, NT-CORE. | Each social subsystem becomes its own ECS system with Resource access |

### 4.2 Incorrect Domain Assignments

| # | Module | Current Domain | Correct Domain | Rationale |
|---|--------|---------------|----------------|-----------|
| Y1 | `behavior_tree/` | NT-ACT (agents) | NT-CORE (reasoning) | BT is decision-making infrastructure, not action execution |
| Y2 | `planning/goap.rs` | NT-ACT (agents) | NT-CORE (reasoning) | GOAP is planning/reasoning, not action |
| Y3 | `pheromone.rs` | NT-ACT (agents) | NT-WORLD (environment) | Pheromones are world-level stigmergic signals, not agent-level |
| Y4 | `event_reactive.rs` | NT-ACT (agents) | NT-WORLD (perception) | Event reaction is perception-mediated, not pure action |
| Y5 | `reflection.rs` | NT-ACT (agents) | NT-MIND (evolution) | Reflection is meta-cognition, not action |
| Y6 | `personality_drift.rs` | NT-ACT (agents) | NT-FEEL (emotion) | Personality evolution is emotional, not action |
| Y7 | `consciousness/behavior_vm.rs` | NT-CORE | NT-ACT | Behavior VM is action orchestration |

---

## 5. Recommended Refactoring Plan

### Phase 1: Wire Dead Code (Week 1, ~600 LOC, P0)

**Goal**: Connect existing modules that are implemented but never called.

| # | Task | Source | Target | LOC | Dependencies |
|---|------|--------|--------|-----|-------------|
| R1 | **Wire GOAP into `layer_goals()`** | `planning/goap.rs` | `decision.rs` | +80 | None |
| R2 | **Wire BT as decision root** | `behavior_tree/mod.rs` | `decision.rs` | +60 | R1 |
| R3 | **Wire MemoryStream into decision** | `memory_stream.rs` | `decision.rs` | +40 | None |
| R4 | **Wire EmotionEngine into decision** | `feel/mod.rs` | `decision.rs` | +30 | None |
| R5 | **Wire AStar into movement** | `navigation/astar.rs` | `actions.rs` | +40 | None |
| R6 | **Wire RVO into movement** | `navigation/rvo.rs` | `actions.rs` | +30 | None |
| R7 | **Wire Pheromone gradients into decision** | `pheromone.rs` | `decision.rs` | +25 | None |
| R8 | **Wire Faction into WorldSim** | `society/faction.rs` | `world_sim/mod.rs` | +50 | None |
| R9 | **Wire Gossip into SocialLearning** | `society/gossip.rs` | `agents/social_learning.rs` | +40 | None |
| R10 | **Wire EmergenceDetector** | `consciousness/emergence_detector.rs` | `world_sim/mod.rs` | +30 | None |
| R11 | **Wire DualRepresentation** | `consciousness/dual_representation.rs` | `world_sim/mod.rs` | +25 | None |
| R12 | **Implement Attack/Gather handlers** | `world_sim/actions.rs` | `world_sim/actions.rs` | +80 | None |

### Phase 2: Architecture Cleanup (Week 2, ~400 LOC, P1)

**Goal**: Resolve domain misalignments and remove redundancy.

| # | Task | Description | LOC |
|---|------|-------------|-----|
| R13 | **Extract DecisionEngine** | Move `decide_action()` out of WorldSim into standalone `DecisionEngine` system | +100 |
| R14 | **Extract SocialEngine** | Move `relationships`, `economy`, `culture`, `theory_of_mind` into standalone `SocialEngine` | +80 |
| R15 | **Extract EvolutionEngine** | Move `evolution_cycle()`, `fitness_landscape`, `selection`, `mutation`, `speciation` into standalone `EvolutionEngine` | +60 |
| R16 | **Unify EventReactiveSystem with SimulationBus** | Remove duplicate event routing, make EventReactiveSystem subscribe to SimulationBus | +40 |
| R17 | **Merge ActionCostTable.can_afford() + ActionBudget.risk_per_action()** | Single cost evaluation function | +20 |
| R18 | **Remove BehaviorVM dead code** | Delete `consciousness/behavior_vm.rs` or convert to orchestration layer | -200 |

### Phase 3: New Capabilities (Week 3-4, ~800 LOC, P1)

**Goal**: Add missing capabilities that external research identifies as critical.

| # | Task | Source | LOC | Dependencies |
|---|------|--------|-----|-------------|
| R19 | **Fog of War** | Per-agent visibility, shadowcasting, explored/visible states | +300 | R5 (AStar for LOS) |
| R20 | **Hierarchical memory consolidation** | Working→Episodic→Semantic promotion, forgetting | +200 | R3 (memory wiring) |
| R21 | **GOAP replanning** | Reactive replanning on world change | +100 | R1 (GOAP wiring) |
| R22 | **Per-agent event subscriptions** | Agent subscribes to relevant event types | +80 | R16 (bus integration) |
| R23 | **Utility scoring with response curves** | Configurable response curves for each consideration | +120 | R4 (emotion wiring) |

### Phase 4: ECS Migration (Week 5-6, ~1700 LOC net, P2)

**Goal**: Migrate to Bevy ECS substrate for parallelism.

| # | Task | Description | LOC |
|---|------|-------------|-----|
| R24 | **Add `bevy_ecs` dependency** | Standalone crate, no rendering | +5 |
| R25 | **Define component types** | AgentBundle, all per-agent components | +400 |
| R26 | **Define resource types** | All global state as Resources | +200 |
| R27 | **Create ECS world + spawn agents** | WorldSimEcs::new() | +300 |
| R28 | **Migrate reflex systems** | Time advance, resource regen, spatial grid, metabolism | +200 |
| R29 | **Migrate fast systems** | Observation, decision (5 layers), execution, memory recording | +400 |
| R30 | **Migrate slow/medium/background systems** | Emotion, planning, reflection, evolution | +200 |
| R31 | **Enable parallel agent processing** | rayon par_iter for perception+decision | +100 |
| R32 | **Remove old WorldSim code** | Delete monolithic struct and all methods | -800 |

---

## 6. Priority Matrix

| Priority | Items | Effort | Impact |
|----------|-------|--------|--------|
| **P0 — Wire Dead Code** | R1-R12 | 1 week | Agents gain planning, pathfinding, collision avoidance, emotion modulation, faction support |
| **P1 — Architecture Cleanup** | R13-R18 | 1 week | Clean domain boundaries, remove duplication, prepare for ECS |
| **P1 — New Capabilities** | R19-R23 | 2 weeks | Fog of war, memory consolidation, GOAP replanning, utility scoring |
| **P2 — ECS Migration** | R24-R32 | 2 weeks | Parallel execution, 1000+ agent support, plugin architecture |

---

## 7. Key Metrics

| Metric | Current | After P0 | After P1 | After P2 |
|--------|---------|----------|----------|----------|
| Dead modules wired | 0/20 | 12/20 | 16/20 | 20/20 |
| Decision pipeline quality | if-else cascade | GOAP + BT + utility | + emotion modulation + memory retrieval | + parallel execution |
| Agent throughput | ~50 agents/tick | ~50 (no change) | ~50 (no change) | 1,000+ agents/tick |
| Memory retrieval usage | 0 calls/tick | 1 call/agent/tick | 1 call/agent/tick | 1 call/agent/tick |
| A* pathfinding usage | 0 calls/tick | 1 call/move | 1 call/move | 1 call/move (parallel) |
| RVO collision avoidance | 0 calls/tick | 1 call/move | 1 call/move | 1 call/move (parallel) |
| Domain misalignments | 7 | 4 (after R13-R15) | 2 (after full cleanup) | 0 (ECS enforces) |
| Lines of dead code | ~2,500 | ~1,900 | ~1,500 | ~700 (net after cleanup) |
