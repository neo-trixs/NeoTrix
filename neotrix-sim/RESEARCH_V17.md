# RESEARCH_V17.md — Game AI & Simulation Architecture Research

**Date**: 2026-09-10
**Purpose**: 20 web searches covering Bevy ECS, MOBA AI, behavior trees, pathfinding, memory, evolution, self-play, team strategy, emergence, consciousness, open-source MOBAs, utility AI, GOAP, faction systems, Unity DOTS/ECS, Unity ML-Agents, Unity MOBA, Unity architecture, and simulation architecture.

---

## 1. "Bevy ECS game 2025"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/bevyengine/bevy |
| **Key Insight** | Bevy is a data-driven Rust game engine using ECS. Key features: modular architecture, WASM support, 2D/3D rendering, parallel system execution. Ships include `space-looter` (DDD RPG), `bevy-open-siege` (lane defense), `last-stop` (A* pathfinding puzzle). |
| **NeoTrix Mapping** | Bevy ECS maps directly to NeoTrix's `nt_*` module architecture — entities = domain modules, components = state, systems = domain logic. WASM target enables browser-deployable simulations. The `Juice: Zero Bugs Given` game demonstrates boss AI via state components rather than inheritance. |

## 2. "MOBA AI Bot"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/internexio/clawberbot |
| **Key Insight** | Clawber.ai 5v5 arena: hand-crafted SEMalytics bot reached #1 ELO in 12K matches/68 versions. Autonomous version matched that ELO in <1K matches using KnowledgeForge reasoning + Wilson CI statistical gating + failed-hypothesis log. 12x improvement by removing human bottleneck. |
| **NeoTrix Mapping** | The autonomous bot's pipeline (play → gate → diagnose → submit → repeat) is structurally identical to SEAL pipeline phases. Wilson CI gating maps to `QualityGate`. Failed-hypothesis log is a form of `experience-tree` absorption. MOBA-AI-Gamer uses YoloV5 + OCR + Deep Q-Learning — screen-based perception pipeline. |

## 3. "Game Behavior Tree"

| Field | Detail |
|-------|--------|
| **URL** | https://www.behaviortrees.com/learn/ |
| **Key Insight** | BTs: hierarchical nodes (Sequence=AND, Selector=OR), 3 statuses (Running/Success/Failure), blackboard for shared data. Event-driven BTs solve scalability. Used in Halo 2, Bioshock, Spore. Behavior Designer Pro (Unity DOTS backend) offers 15+ sample scenes. |
| **NeoTrix Mapping** | BT structure maps to GWT attention routing: Selector = salience-based selection, Sequence = task chain execution, Decorator = attention filtering. Blackboard = KB shared state. NeoTrix can implement BT runtime as an `nt_core` system where nodes are ECS entities with `BTNode` components. |

## 4. "Game Pathfinding"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/RecastNavigation/recastnavigation |
| **Key Insight** | Recast Navigation: industry-standard navmesh (Unity/Unreal/Godot/O3DE). Pipeline: rasterize triangles → voxelize → filter walkable areas → polygonal regions → triangulate. DetourCrowd handles agent movement + collision avoidance. `constructive` (Rust) uses BSP trees for navmesh generation. AeonixNavigation: 3D SVO navmesh for UE5. |
| **NeoTrix Mapping** | Recast maps to NT-WORLD spatial perception. DetourCrowd agent simulation = `nt_physical` motor control. For NeoTrix sim: navmesh components on entities, pathfinding system queries walkable polygons, crowd avoidance via Detour-style steering. `constructive` BSP approach is more Rust-native. |

## 5. "Game Memory"

| Field | Detail |
|-------|--------|
| **URL** | https://gamesbyhyper.com/docs/exploration-and-narrative/memory-system/ |
| **Key Insight** | Tag-driven memory system: each actor has a Memory Component with gameplay tags. Memory Context = required tags + blocked tags. Supports dialogue awareness, quest tracking, map markers, faction relations. Asymmetric knowledge (NPC remembers player lied, player remembers spared bandits). |
| **NeoTrix Mapping** | Maps directly to NT-MEMORY KB. Memory tags = KB nodes with namespace `memory_*`. Memory Context = query with required/blocked filters. Asymmetric knowledge = separate KB namespaces per entity. The tag-based system is simpler than full graph traversal but misses cross-session persistence (which NT-MEMORY provides). |

## 6. "Game Evolution"

| Field | Detail |
|-------|--------|
| **URL** | https://www.raillab.org/publication/beukman-2022-procedural/beukman-2022-procedural.pdf |
| **Key Insight** | PCGNN: NEAT + novelty search for level generation. Evolves neural network generators, no training data needed. Generates diverse levels 10x faster than direct search or RL baselines. Hybrid approach (grammar + cellular automata) for dungeon generation achieves high completability. |
| **NeoTrix Mapping** | Maps to NT-MIND SEAL pipeline. Evolutionary search = SEAL exploration phase. Novelty search = preventing convergence to local optima. The "evolve generator, not level" principle = meta-learning, which aligns with NT-MIND's skill crystallization. NeoTrix can use PCGNN for procedural test environment generation. |

## 7. "Game Self-Play"

| Field | Detail |
|-------|--------|
| **URL** | https://arxiv.org/abs/2408.01072 |
| **Key Insight** | Self-play survey: 4 categories (traditional, PSRO, ongoing-training, regret-minimization). AlphaGo Zero: pure self-play RL, tabula rasa, single network, MCTS inside training loop. QZero: model-free off-policy RL achieves AlphaGo-level with 7 GPUs. Key: "Ignition Mechanism" to bootstrap Q-learning, Polyak averaging for stability. |
| **NeoTrix Mapping** | Self-play = NT-MIND evolution cycle. Population-based self-play = maintaining skill variants. The "ignition mechanism" maps to bootstrapping new skill domains. Wilson CI gating (from Clawber.ai) provides the statistical promotion/demotion. This is the core of NeoTrix's self-evolving reasoning. |

## 8. "Game Team Strategy"

| Field | Detail |
|-------|--------|
| **URL** | https://arxiv.org/abs/2304.11632 |
| **Key Insight** | MCC framework (Tencent): Meta-Command Communication for human-AI collaboration in MOBA. Protocol: <Location, Event, TimeLimit> meta-commands. Meta-Command Selector evaluates value of commands. Hierarchical model: macro-strategy layer → micro-action layer. Tested on Honor of Kings 5v5. |
| **NeoTrix Mapping** | Meta-Command = GWT broadcast signal with structured intent. Meta-Command Selector = salience evaluation with cost weight (Axiom A1). The macro/micro hierarchy maps to L5 cognition (strategy) → L1 action (execution). Human-AI collaboration aligns with NT-IO interface layer. |

## 9. "Game Emergence"

| Field | Detail |
|-------|--------|
| **URL** | https://oldlight.io/blog/emergent-gameplay-simple-rules/ |
| **Key Insight** | Old Light (RTS): 5 rules produce emergence — border tiebreak → land rush + readable history; energy deficit → overextension trap + comeback; credit drain → economic warfare; targeting priority → probe screen exploit; local AI pacing → self-balancing difficulty. Key principle: simple rules, no special cases. |
| **NeoTrix Mapping** | Emergence = GWT attention routing producing unexpected cross-domain effects. Simple rules = domain axioms (R-P1, R-P79, etc.). The "borders as readable history" = KB experience log as emergent intelligence. Self-balancing difficulty = HeartbeatAggregator modulating GWT salience. |

## 10. "Game Consciousness"

| Field | Detail |
|-------|--------|
| **URL** | https://arxiv.org/abs/2608.21439 |
| **Key Insight** | WorldMind: decoupled game world model for state-aware NPC behavior. 4 layers: Understanding (compact state from frames) → Decision (reason over state) → Control (temporal alignment) → Generation (visual output). BOSS-140K dataset of gameplay + internal game states. Preferred over baselines in 70% of comparisons. |
| **NeoTrix Mapping** | WorldMind's 4 layers map to NeoTrix's 6-layer architecture: Understanding = L2 Perception, Decision = L5 Cognition, Control = L3 Embodiment, Generation = L1 Action. The "compact state" = VSA HyperCube embedding. State-aware NPC = SelfModel-informed agent. Consciousness-as-decoupled-world-model validates NeoTrix's architecture. |

## 11. "Open Source MOBA"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/Nival-Pub/Prime-World |
| **Key Insight** | Prime World: MOBA open-sourced 2024 (C++, in-house engine). Heroes-Arena: Kotlin 5v5 with 7 heroes, 6 classes, item shop, ELO ranking. Theria: Godot 4 shapeshifter MOBA, server-authoritative deterministic simulation. UNION-OpenSource-MOBA: Unity + PlayFab/Photon. |
| **NeoTrix Mapping** | Prime World C++ = reference for MOBA state architecture. Theria's deterministic simulation = ideal for self-play training (reproducible outcomes). The Kotlin Heroes-Arena shows 5v5 lane/jungle/tower mechanics as reusable patterns. NeoTrix sim can use Theria-style deterministic tick for reproducible AI training. |

## 12. "Bevy 2D"

| Field | Detail |
|-------|--------|
| **URL** | https://bevy.org/ |
| **Key Insight** | Bevy 0.18+: 2D feature profile with sprite sheets, dynamic texture atlases, cameras, textures, materials. Custom shaders/materials/render pipelines. Feature-gated compilation (2d, 3d, ui, audio profiles). WASM compilation. |
| **NeoTrix Mapping** | Bevy 2D = rendering backend for NeoTrix sim visualization. Feature-gated compilation enables headless simulation (no render) or full visual mode. Bevy's `2d_bevy_render` profile includes gizmos for debug visualization. The `bevy_sprite` + `bevy_core_pipeline` stack handles MOBA map rendering. |

## 13. "Game Utility AI"

| Field | Detail |
|-------|--------|
| **URL** | https://uintel-ecs.utilityworlds.com/Documentation/Overview/ |
| **Key Insight** | Utility Intelligence (ECS): evaluates ALL decisions, scores per target, selects highest. Combines: Utility AI (decision-making) + FSM (transitions) + BT (execution). IAUS (Infinite Axis Utility System) pattern. DOTS backend for high agent counts. Decision Making Preview in editor. |
| **NeoTrix Mapping** | Utility AI = GWT salience scoring. IAUS = multi-dimensional scoring with response curves, mapping to EmotionLabel influence on decision weights. The UTILITY+FSM+BT hybrid is exactly NeoTrix's pattern: Utility for choice (GWT), FSM for state transitions (ConsciousnessTree), BT for execution (action sequences). |

## 14. "Game GOAP"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/jeanfbrito/world-simulator |
| **Key Insight** | GOAP + Utility AI hybrid: GOAP for strategic planning (A* over action space), Utility for reactive behaviors. Medieval economy sim: peasants autonomously manage hunger/energy, plan paths, gather resources, build houses. 100+ agents at 60 FPS on Bevy ECS. Lua scripting for behaviors. |
| **NeoTrix Mapping** | GOAP maps to NT-ACT goal-directed action planning. The GOAP+Utility hybrid = SEAL pipeline (strategic) + GWT attention (reactive). Bevy ECS + spatial indexing for performance. Lua scripting = skill node customization. This is a production-proven pattern for NeoTrix sim. |

## 15. "Game Faction"

| Field | Detail |
|-------|--------|
| **URL** | https://dl.acm.org/doi/fullHtml/10.1145/3649921.3650012 |
| **Key Insight** | Faction taxonomy: Absolute (role-based, unit-based), Allegiance/State-based (player choice), Implicit (world-based, procedurally generated). Key dimensions: moral spectrum, player choices, hierarchy, NPC roles. Seven is optimal faction count for memory (Farouk Fusion). Sub-factions enable mirror-match differentiation. |
| **NeoTrix Mapping** | Faction system = NT-* domain faction mapping. Seven optimal factions = 7 NeoTrix domains. Sub-factions = skill nodes within domains. Procedural factions = SEAL-generated domain variants. Reputation systems = KB namespace `faction_*` with weighted edges. The taxonomy validates NeoTrix's domain design. |

## 16. "Unity DOTS ECS"

| Field | Detail |
|-------|--------|
| **URL** | https://unity.com/dots |
| **Key Insight** | Unity DOTS: ECS + Burst Compiler + Job System. Archetype-based storage (SoA), cache-friendly iteration. "ECS for all" initiative: every GameObject backed by Entity. System scheduling with dependency tracking. 128+ player demos. Used in V Rising (open-world survival), IXION (city builder). |
| **NeoTrix Mapping** | Unity DOTS architecture = reference implementation for NeoTrix ECS. Archetype storage = KB entity clustering. Burst compilation = native Rust performance equivalent. Job system parallelism = `nt_core` parallel system execution. The "ECS for all" direction validates NeoTrix's ECS-first design. |

## 17. "Unity ML-Agents"

| Field | Detail |
|-------|--------|
| **URL** | https://docs.unity3d.com/Packages/com.unity.ml-agents@4.1/manual/Training-ML-Agents.html |
| **Key Insight** | ML-Agents: PPO/SAC/POCA trainers. Self-play with population-based training. Curriculum learning with lesson progression. Environment parameter randomization. Behavioral cloning from demonstrations. GAIL/RND intrinsic rewards. ELO scoring for adversarial evaluation. |
| **NeoTrix Mapping** | ML-Agents = training backend for NeoTrix sim agents. PPO/SAC = reinforcement learning for skill acquisition. Self-play = SEAL evolution cycle. Curriculum learning = Constellation maturity ladder (C0→C6). ELO scoring = performance metrics for skill promotion/demotion. Behavioral cloning = experience-tree absorption from human play. |

## 18. "Unity MOBA"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/olivercrush/UnityMOBA |
| **Key Insight** | UnityMOBA: CQ/Distance minions, barracks spawning, towers attacking enemies. EasyMOBA: client-server split, server-authoritative mechanics, custom collision + pathfinding. MOBA_CSharp_Unity: 3000 lines, navmesh pathfinding, bush/vision, scriptable heroes/skills/buffs/items. |
| **NeoTrix Mapping** | UnityMOBA architecture = simulation reference. Client-server split = NT-IO (client) + NT-ACT (server). Scriptable heroes = ECS archetype templates. Bush/vision = NT-WORLD perception system with fog-of-war. The 3000-line framework shows minimal viable MOBA mechanics for NeoTrix sim. |

## 19. "Unity Game Architecture"

| Field | Detail |
|-------|--------|
| **URL** | https://discussions.unity.com/t/the-various-ways-to-use-unity-ecs-a-starter-guide/1557214 |
| **Key Insight** | 4 ECS styles: Hybrid (MonoBehaviour + Entity), Simple ECS (ISystem, no jobs), Job-heavy ECS (IJobEntity chains), Data-Driven ECS (everything is data). Simulation games = Job-heavy with one big job chain. Data-driven = bakers + authoring tools express complexity. |
| **NeoTrix Mapping** | Data-Driven ECS = NeoTrix's architecture where everything is data (KB nodes/edges). Job-heavy ECS = `nt_core` parallel system execution. The "simulation games" pattern (factory/base-building) directly applies to NeoTrix's domain simulation. Multiple Worlds = isolated simulation contexts. |

## 20. "Simulation Game Architecture"

| Field | Detail |
|-------|--------|
| **URL** | https://github.com/lizTheDeveloper/ai_village |
| **Key Insight** | AI Village: ECS with 200+ systems across 19 packages. AI agents with LLM brains. Features: magic systems, divinity, reproduction/genetics, fire propagation, fluid dynamics, power grids, conveyor belts, multiverse forks. Dwarf Fortress-inspired architecture: component-based items, material templates, data/behavior separation. |
| **NeoTrix Mapping** | AI Village = largest open-source ECS sim reference. 200+ systems = scale target for NeoTrix. LLM brains = NT-IO provider integration. Multiverse forks = SEAL parallel exploration. Dwarf Fortress patterns = data-driven design philosophy. The 19-package structure maps to NeoTrix's domain modules. The ECS→ABM research paper confirms ECS provides better parallel efficiency than OOP for agent-based simulations. |

---

## Cross-Cutting Patterns

### Pattern A: ECS as Universal Simulation Architecture
- Bevy ECS, Unity DOTS, AI Village, world-simulator all converge on ECS
- Components = pure data, Systems = logic, Entities = identity
- Archetype-based storage for cache efficiency
- NeoTrix already follows this with `nt_*` module architecture

### Pattern B: Hybrid AI Decision Stack
- Utility AI (scoring) + GOAP (planning) + BT (execution) = production standard
- WorldMind adds "Understanding Layer" as pre-decision state compression
- NeoTrix: GWT (Utility) + SEAL (GOAP) + action sequences (BT)

### Pattern C: Self-Play as Evolution Engine
- AlphaGo Zero, QZero, Clawber.ai all use self-play for improvement
- Wilson CI gating for promotion/demotion
- Population-based training for diversity
- NeoTrix: SEAL pipeline + experience-tree absorption

### Pattern D: Emergence from Simple Rules
- Old Light: 5 rules → land rush, economic warfare, self-balancing difficulty
- Minecraft: simple blocks → complex machines
- NeoTrix: domain axioms → cross-domain intelligence

### Pattern E: Faction/Domain Design
- Seven optimal factions for memory capacity
- Each faction needs: visual identity, gameplay loop, lore hook, territory, representative
- NeoTrix: 7 domains, each with unique capabilities, all sharing KB

### Pattern F: Communication Protocol for Human-AI Collaboration
- MCC Meta-Commands: <Location, Event, TimeLimit>
- Interpretable, bidirectional, value-aligned
- NeoTrix: GWT broadcast signals with structured intent

---

## Priority Recommendations

| Priority | Area | Source | Action |
|----------|------|--------|--------|
| P0 | ECS Architecture | Bevy/Unity DOTS | Adopt archetype-based ECS for sim core |
| P0 | GOAP + Utility Hybrid | world-simulator | Implement dual AI decision stack |
| P1 | Self-Play Training | ML-Agents/Clawber | Wilson CI gating for skill promotion |
| P1 | NavMesh Pathfinding | Recast Navigation | Integrate recastnavigation for spatial reasoning |
| P2 | Memory System | Hyper docs | Tag-driven memory with asymmetric knowledge |
| P2 | Faction Taxonomy | ACM paper | 7-domain design with sub-faction skills |
| P3 | Emergence Rules | Old Light | Simple axioms → cross-domain effects |
| P3 | Meta-Command Protocol | MCC framework | Interpretable human-AI communication |
