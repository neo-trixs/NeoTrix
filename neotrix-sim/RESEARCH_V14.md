# RESEARCH_V14 — NeoTrix MOBA Sim Research

25 web searches compiled on 2026-09-11. Each entry: URL, key insight, NeoTrix mapping.

---

## 1. Bevy ECS game 2025

- **URL**: https://github.com/linzell/space-looter
- **Key Insight**: Bevy ECS + DDD architecture for a 3D isometric RPG. Domain-Driven Design with `domain/` → `services/` → `infrastructure/bevy/` layering. Components are plain Rust structs, systems are functions. Demonstrates Bevy's archetype SoA storage achieving high cache coherence for 320+ simultaneous entities.
- **NeoTrix Mapping**: Bevy's DDD layering maps directly to NeoTrix's 6-layer architecture (L1-L6). The `domain/ → services/ → infrastructure/bevy/` split mirrors NeoTrix's `l1_action/ → l2_perception/ → ...` pattern. Bevy's archetype ECS validates NeoTrix's own `nt_core_*` component-based design.

## 2. MOBA AI bot

- **URL**: https://doi.org/10.4324/9781315151700-32 (Paragon Bots)
- **Key Insight**: All Paragon MOBA bots use a SINGLE behavior tree with extreme parameterization via Blackboard key-value pairs. A "One-Step Influence Map" provides spatial tactical analysis — units exert influence proportional to strength that decays with distance. "Lane Space" normalizes position as 0.0-1.0 along each lane. A Front-Line Manager coordinates team positioning.
- **NeoTrix Mapping**: The single-BT-with-parameterization pattern maps to NeoTrix's `AttentionManager` routing — one cognitive pipeline, many task-specific parameterizations. Influence maps map directly to `VSA HyperCube` spatial reasoning. Lane Space maps to NeoTrix's `GWT salience` scoring (normalized importance across dimensions).

## 3. Game behavior tree

- **URL**: https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control)
- **Key Insight**: Behavior trees use Selector (OR gate — try children until one succeeds) and Sequence (AND gate — all children must succeed) composites. Three return states: Success, Failure, Running. Blackboards share data across trees. Event-driven BTs (from Halo 2) abort running branches when conditions change, solving scalability issues.
- **NeoTrix Mapping**: Selector/Sequence composites map to NeoTrix's `GWT` broadcast-and-select pattern. The Blackboard pattern maps to NeoTrix's `KB` shared state. Event-driven abort maps to NeoTrix's `ConsciousnessTree` re-evaluation cycle (Soil→Roots→Trunk→Branches→Fruits→Core).

## 4. Game pathfinding

- **URL**: https://www.redblobgames.com/pathfinding/ + https://theory.stanford.edu/~amitp/GameProgramming/AStarComparison.html
- **Key Insight**: A* combines Dijkstra's completeness with Greedy Best-First's speed via f(n) = g(n) + h(n). Hierarchical Pathfinding A* (HPA*) partitions maps into clusters for multi-level search. Flow fields (gradient operator on distance fields) are optimal for tower defense. Multi-agent pathfinding needs coordination to avoid collisions.
- **NeoTrix Mapping**: A* maps to NeoTrix's `CapabilityTree` path planning. HPA*'s cluster hierarchy maps to NeoTrix's 3-layer architecture (L5 Consciousness → L3 Embodiment → L1 Capability Network). Flow fields map to NeoTrix's `HeartbeatAggregator` — a field of health signals across all modules.

## 5. Game memory

- **URL**: https://arxiv.org/html/2607.01224 (AutoMem) + https://github.com/AIRI-Institute/AriGraph
- **Key Insight**: AutoMem: memory management is an independently learnable skill for LLM agents. Optimizing memory alone (without changing task-action weights) yields 2-4× performance gains. File-system operations as first-class memory actions. AriGraph: knowledge graph with episodic vertices/edges beats full-history and summary baselines for text games.
- **NeoTrix Mapping**: AutoMem's "memory as trainable skill" maps directly to NeoTrix's `nt_memory` domain — KB is not just storage but an active cognitive skill. AriGraph's knowledge graph maps to NeoTrix's KB `nodes + edges + embeddings`. The "consult-before-write" discipline from AutoMem maps to NeoTrix's `experience-tree` protocol.

## 6. Game evolution

- **URL**: https://keiwando.com/evolution/ + https://www.cse.unr.edu/~bdbryant/papers/stanley-2006-aaai.pdf (NERO/rtNEAT)
- **Key Insight**: rtNEAT (real-time NeuroEvolution of Augmenting Topologies) complexifies neural networks AS the game is played — starting simple, adding nodes/connections as behavior demands. NERO game: players train agents via customized training exercises; agents learn to attack, dodge, navigate mazes. Evolution produces diverse populations (not converging to single strategy).
- **NeoTrix Mapping**: rtNEAT's complexification maps to NeoTrix's `SEAL pipeline` — starting with simple skill nodes, adding complexity as the system evolves. NERO's player-as-trainer model maps to NeoTrix's `self-play evolution` where the system trains itself. Population diversity maps to NeoTrix's `Dual Specialization` (Weapon Set I/II).

## 7. Game self-play

- **URL**: https://www.science.org/doi/10.1126/science.aar6404 (AlphaZero)
- **Key Insight**: AlphaZero achieves superhuman play in chess, shogi, and Go from tabula rasa self-play with NO domain knowledge beyond rules. Uses deep neural network (p,v) = f_θ(s) for move probabilities + value estimation, guided by Monte Carlo Tree Search. Same algorithm, same hyperparameters across all three games. Outperforms Stockfish after 4 hours (300k steps) searching 1000× fewer positions.
- **NeoTrix Mapping**: AlphaZero's generalizability maps to NeoTrix's `ConsciousnessTree` — one meta-cognition loop serving all domains. MCTS tree search maps to NeoTrix's `CapabilityTree` exploration. The (policy, value) network maps to NeoTrix's `SelfModel` (nt_core_self::SelfModel) which estimates both action quality and uncertainty.

## 8. Game team strategy

- **URL**: http://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter24_Being_Where_It_Counts_Telling_Paragon_Bots_Where_to_Go.pdf
- **Key Insight**: MOBA strategic layer uses an "Objective Graph" — nodes are important locations (towers, jungle camps), edges are navigable paths with costs. A* search on this graph finds smart paths avoiding enemies. Influence maps propagate hero strength across graph nodes. "Opportunistic objectives" let bots pick up tasks along their path.
- **NeoTrix Mapping**: Objective Graph maps to NeoTrix's `CapabilityTree` (evolution view) connected to `CapabilityRegistry` (runtime view) via `CapabilityBridge`. Influence propagation maps to NeoTrix's `GWT salience` broadcasting. Opportunistic objectives map to NeoTrix's `SEAL pipeline` opportunistic learning.

## 9. Game emergence

- **URL**: https://arxiv.org/html/2404.17027v1 + https://psichix.github.io/emergent/ + https://dfairesearch.com/rrd-genesis/
- **Key Insight**: RRD-GENESIS: 52-neuron entities independently discover fire, deception, and social manipulation through Hebbian plasticity — no scripted behavior. "Absolute Emergence" from raw neuronal activations. Players interacting with LLM-driven NPCs create "desire paths" — unanticipated but productive narrative branches. Emergent gameplay arises from simple rules + complex interactions.
- **NeoTrix Mapping**: RRD-GENESIS's emergent behavior from simple neurons maps to NeoTrix's `E8 Hexagram` — 64 simple hexagrams producing complex reasoning. Hebbian plasticity maps to NeoTrix's `experience-tree` KB learning. "Desire paths" map to NeoTrix's `SEAL pipeline` spontaneous capability discovery.

## 10. Game consciousness

- **URL**: https://theconsciousness.ai/posts/ai-video-games-2026-agent-driven-npc-consciousness/ + https://doi.org/10.1109/cig.2009.5286473
- **Key Insight**: 2026 game industry uses "memory-first AI" — persistence across sessions, adaptive behavior. NO game claims consciousness. Architecture required for consciousness (GWT global workspace, IIT integrated information) is absent — game AI is modular/decomposable by design for maintainability. Functional realism beats consciousness claims.
- **NeoTrix Mapping**: NeoTrix's `ConsciousnessTree` is NOT claimed as consciousness but as meta-cognition. The GWT attention routing in NeoTrix IS a global workspace architecture — making NeoTrix architecturally closer to consciousness theories than game AI. NeoTrix's `Phi` integration score measures IIT-like integration. This is intentional design, not accidental modularity.

## 11. Open source MOBA

- **URL**: https://github.com/The-JDdev/Heroes-Arena + https://github.com/Nival-Pub/Prime-World + https://github.com/ajhahnde/Theria
- **Key Insight**: Heroes Arena: 5v5 MOBA in Kotlin with 7 heroes, 6 classes, minion waves, tower defense, jungle, item shop, ELO ranking. Prime World: MOBA open-sourced 2024, C++ in-house engine. Theria: 2.5D MOBA in Godot 4 with server-authoritative deterministic simulation, shapeshifter tribes.
- **NeoTrix Mapping**: Heroes Arena's 6 hero classes map to NeoTrix's 7 domains. Prime World's Talent system maps to NeoTrix's `Skill Tree` (3 node tiers). Theria's deterministic simulation maps to NeoTrix's `SEAL pipeline` deterministic execution. The faction/tribe system maps to NeoTrix's `domain_nt_*` namespace system.

## 12. Bevy 2D

- **URL**: https://bevy.org/ + https://bevy.org/learn/quick-start/getting-started/ecs/ + https://github.com/LiTianchu/bevy-infinite-world-2d-starter
- **Key Insight**: Bevy ECS uses normal Rust datatypes (no macros/traits for basics). Systems are plain functions with dependency injection via parameters. Queries filter entities by component sets. Bevy 0.19+ supports 2D sprites, dynamic texture atlases, cameras. Infinite world generation demonstrated with procedural terrain.
- **NeoTrix Mapping**: Bevy's "systems are plain functions" maps to NeoTrix's `nt_*` module pattern where each domain has standalone system functions. Bevy's Query pattern maps to NeoTrix's `KB query` (filtering nodes by type/namespace). Bevy's dependency injection maps to NeoTrix's `GWT` resource injection.

## 13. Game utility AI

- **URL**: https://www.socratopia.app/library/game-ai-patterns-en/chapter-12 + https://en.wikipedia.org/wiki/Utility_system
- **Key Insight**: Utility AI scores each candidate action with: need × object_affordance × modifier ^ urgency. Curves (sigmoid, piecewise, linear) map raw values to 0-1 scores. The Sims pioneered this: NPC needs + object satisfaction values. Infinite Axis Utility System (IAUS) makes it data-driven. Utility can be hybridized with BTs.
- **NeoTrix Mapping**: Utility AI's scoring maps directly to NeoTrix's `GWT salience` — each module broadcasts a utility score, GWT selects the highest. The curve-based scoring maps to NeoTrix's `HeartbeatAggregator` time-decay scoring. IAUS's data-driven approach maps to NeoTrix's `Rune Socketing` (5 configuration slots per module).

## 14. Game GOAP

- **URL**: https://github.com/crashkonijn/goap + https://github.com/luxkun/ReGoap
- **Key Insight**: GOAP (Goal Oriented Action Planning) uses A* to find action chains: preconditions → effects → cost. Agents have multiple weighted goals; planner finds highest-utility achievable goal. ReGoap: engine-agnostic C# library with weighted-random goal selection. F.E.A.R. pioneered GOAP — agents plan action sequences from world state, not scripted trees.
- **NeoTrix Mapping**: GOAP's precondition/effect/cost model maps to NeoTrix's `CapabilityTree` edges (preconditions = input components, effects = output components, cost = execution weight). Weighted goal selection maps to NeoTrix's `AttentionManager` dual specialization. GOAP's replanning on world change maps to NeoTrix's `ConsciousnessTree` cycle re-evaluation.

## 15. Game faction

- **URL**: https://github.com/The-JDdev/Heroes-Arena + https://github.com/ajhahnde/Theria
- **Key Insight**: MOBA factions define hero pools, lane assignments, and win conditions. Theria's Solane (big-cat burst) vs Verdani (venom-and-shadow attrition) — asymmetric faction design where each faction's mechanics create natural counterplay. Heroes Arena: 6 classes (Tank/Fighter/Assassin/Mage/Marksman/Support) as faction roles.
- **NeoTrix Mapping**: Asymmetric faction design maps to NeoTrix's 7 domains (NT-CORE through NT-FEEL) — each domain has unique "mechanics" (E8 reasoning, KB storage, GWT attention, etc.). Class roles map to NeoTrix's `Ascendancy dual specialization` — agents specialize in two domains per session. Counterplay maps to NeoTrix's `Dark Forest` axiom (modules must compile+test+connect or be deleted).

## 16. Unity custom game framework

- **URL**: https://github.com/AnisKaram/Unity-Modular-Game-Architecture + https://github.com/invertibleMatrix/unity-game-framework + https://github.com/heathen-engineering/Unity-Game-Framework
- **Key Insight**: UGFW: all-in-one framework with assembly definitions, DI via Reflex, no singletons. AppStateMachine for app flow, MetaData pattern for game content. Heathen: Unreal-inspired World/GameMode/GameState/PlayerState structure with subsystem lifecycle. RC-Framework: MVC with scoped EventBus and automatic cleanup.
- **NeoTrix Mapping**: UGFW's "no singletons, DI everywhere" maps to NeoTrix's `dependency injection` via GWT. Heathen's World/GameMode/GameState maps to NeoTrix's `World (nt_world) / Core (nt_core) / Memory (nt_memory)` triple. The assembly definition pattern maps to NeoTrix's `domain_nt_*` namespace isolation.

## 17. Unity ECS DOTS game

- **URL**: https://unity.com/dots + https://unity.com/ecs + https://docs.unity3d.com/Packages/com.unity.entities%400.17/manual/gp_overview.html
- **Key Insight**: Unity ECS uses archetype SoA storage — entities with identical component sets grouped in columnar tables. Burst Compiler + Job System for parallel execution. SubScene converts GameObjects to entities at build time. Entities are simple integer IDs, components are unmanaged structs, systems are ISystem structs.
- **NeoTrix Mapping**: Unity ECS's archetype SoA maps to NeoTrix's KB storage (entities = nodes, components = node properties, systems = domain functions). Burst+Job parallelism maps to NeoTrix's `lock-free parallel scheduler`. SubScene conversion maps to NeoTrix's `experience-tree` absorption pipeline (raw data → distilled KB entries).

## 18. Unity ML-Agents training

- **URL**: https://unity-technologies.github.io/ml-agents/Training-ML-Agents/ + https://unity-technologies.github.io/ml-agents/Training-Configuration-File/
- **Key Insight**: ML-Agents supports PPO, SAC, and POCA trainers. Self-play: snapshots of past policies as opponents, with team_change and players_to_eat counters for opponent diversity. Curriculum learning with lesson-based parameter progression. Environment parameter randomization for domain randomization. GAIL for imitation learning from demonstrations.
- **NeoTrix Mapping**: ML-Agents' self-play snapshot system maps to NeoTrix's `experience-tree` — snapshots of past skill states as "opponents" for self-improvement. Curriculum learning maps to NeoTrix's `SEAL pipeline` staged evolution (C0→C6 constellation maturity). GAIL imitation learning maps to NeoTrix's `external-absorption` skill.

## 19. Unity behavior tree

- **URL**: https://opsive.com/support/documentation/behavior-designer-pro/concepts/what-is-a-behavior-tree/
- **Key Insight**: Behavior Designer Pro uses DOTS as backend for performance. Visual debugging shows active task + execution flow in real-time. Event-driven BTs from Halo 2: abort running branches on condition change. Blackboards share data across multiple BTs — different AI characters share information like player position.
- **NeoTrix Mapping**: Behavior Designer Pro's DOTS backend maps to NeoTrix's `data-oriented design` across all layers. The blackboard-sharing pattern maps to NeoTrix's `KB` — all domains read/write shared state. Event-driven abort maps to NeoTrix's `ConsciousnessTree` re-evaluation on system health changes.

## 20. Unity MOBA template

- **URL**: https://github.com/nidaynere/easymoba + https://github.com/NicoRuedaA/Mobalike
- **Key Insight**: EasyMOBA: client-server split, server runs all game sessions with custom collision detection and pathfinding. Data-driven: heroes, creatures, maps, game modes all defined in folders. Mobalike: "Brain and Body" architecture separating logic from representation. Data-driven abilities with 5 behavior types (projectile, AoE, trail, buff, smash).
- **NeoTrix Mapping**: EasyMOBA's data-driven folder system maps to NeoTrix's `KB node types` (entities, relations, embeddings). Mobalike's "Brain and Body" maps to NeoTrix's `nt_core (cognition) / nt_physical (embodiment)` split. The 5 ability behavior types map to NeoTrix's `Rune Socketing` 5 colors (Crimson/Indigo/Obsidian/Golden/Alabaster).

## 21. Unity multiplayer game

- **URL**: https://docs.unity.com/en-us/multiplayer + https://docs.unity3d.com/Packages/com.unity.netcode.gameobjects%402.4/manual/tutorials/get-started-with-ngo.html
- **Key Insight**: Unity offers two netcode frameworks: Netcode for Entities (server-authoritative, ECS-based, deterministic) and Netcode for GameObjects (high-level, GameObject-based). MPS SDK unifies Lobby, Matchmaker, and Relay. Sessions abstraction manages player connections. Server-authoritative model: server owns game state, clients request changes.
- **NeoTrix Mapping**: Netcode for Entities' server-authoritative model maps to NeoTrix's `KB as single source of truth`. The Sessions abstraction maps to NeoTrix's `nt_nexus` (cross-session memory). The Lobby/Matchmaker pattern maps to NeoTrix's `GWT` — routing tasks to the most capable available module.

## 22. Unity AI navigation

- **URL**: https://docs.unity3d.com/ScriptReference/AI.NavMesh.html + https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html
- **Key Insight**: NavMesh uses convex polygons for walkable areas. A* finds corridor of polygons, then steering follows next visible corner. RVO (Reciprocal Velocity Obstacles) for local collision avoidance. Global navigation (pathfinding) vs local navigation (avoidance) are distinct layers. NavMesh carving for dynamic obstacles.
- **NeoTrix Mapping**: NavMesh's two-layer model (global pathfinding + local avoidance) maps to NeoTrix's `CapabilityTree (strategic) + runtime systems (tactical)`. RVO collision avoidance maps to NeoTrix's `NT-SHIELD` safety kernel. NavMesh carving maps to NeoTrix's `nt_repair` dynamic system modification.

## 23. Unity simulation game

- **URL**: https://docs.unity3d.com/Packages/com.unity.simulation.games@0.4/manual/index.html + https://docs.unity3d.com/Simulation/manual/
- **Key Insight**: Unity Game Simulation: grid search across parameter combinations for balancing. Counter-based metrics tracking with snapshots. headless Linux builds for CI integration. Unity Simulation (robotics): PhysX 4.1 physics, Articulation Body for articulated systems, multi-GPU scaling.
- **NeoTrix Mapping**: Game Simulation's grid search maps to NeoTrix's `SEAL pipeline` parameter optimization. Counter-based metrics maps to NeoTrix's `HeartbeatAggregator` health tracking. The headless build pattern maps to NeoTrix's `nt_io` headless CLI mode. Multi-GPU scaling maps to NeoTrix's `parallel_task` resource management.

## 24. Unity entity component system

- **URL**: https://github.com/SanderMertens/flecs + https://github.com/oasys-works/oecs + https://boyang.cs.uwm.edu/publication/sac2026_ECS.pdf
- **Key Insight**: Flecs: archetype SoA with entity relationships, hierarchies, prefabs. oecs: two-layer split — archetype ECS (logic/identity) and backing-neutral column store (bytes). ECS research: archetype SoA nearly doubles iteration throughput vs sparse-set at high entity counts. Component addition/removal deferred via command queues for thread safety.
- **NeoTrix Mapping**: Flecs's entity relationships map to NeoTrix's KB `edges` (relations between nodes). oecs's two-layer split maps to NeoTrix's `KB storage layer (SQLite) + domain logic layer (nt_*)`. Deferred command buffer pattern maps to NeoTrix's `EventBus` event queue. The archetype concept maps to NeoTrix's `Constellation maturity` (entities grouped by component signature).

## 25. Unity game architecture patterns

- **URL**: https://unity.com/blog/game-programming-patterns-update-ebook + https://unity.com/how-to/how-architect-code-your-project-scales + https://unityqueen.com/2026/06/21/unity-game-architecture-guide-best-patterns-for-clean-code/
- **Key Insight**: 11 patterns: Factory, Object Pool, Singleton, Command, State, Observer, MVP, MVVM, Strategy, Flyweight, Dirty Flag. SOLID principles essential for scaling. Separate data (ScriptableObjects) from behavior (MonoBehaviours). Event-driven decoupling via message bus. Composition over inheritance. Avoid god objects — break into smaller systems.
- **NeoTrix Mapping**: All 11 patterns map to NeoTrix's domain architecture. Factory → `nt_act` tool creation. Object Pool → `nt_memory` KB caching. Command → `experience-tree` absorption steps. State → `ConsciousnessTree` 6-stage cycle. Observer → `EventBus` pub/sub. MVP → `nt_core (model) / nt_io (view) / nt_mind (presenter)`. Strategy → `AttentionManager` dual specialization. Dirty Flag → `HeartbeatAggregator` health change detection.

---

## Cross-Cutting Themes

| Theme | Sources | NeoTrix Integration |
|-------|---------|---------------------|
| **ECS as universal architecture** | Bevy, Unity DOTS, Flecs, oecs | `nt_*` modules as systems, KB nodes as entities, domain traits as components |
| **Self-play for evolution** | AlphaZero, NERO, ML-Agents | `SEAL pipeline` self-play, `experience-tree` opponent snapshots |
| **Utility scoring for attention** | Utility AI, Sims, IAUS | `GWT salience` = utility scoring across all modules |
| **GOAP-style planning** | F.E.A.R., ReGoap, crashkonijn/GOAP | `CapabilityTree` as GOAP planner, KB state as world model |
| **Memory as trainable skill** | AutoMem, AriGraph | `nt_memory` as active cognitive skill, not passive storage |
| **Emergence from simple rules** | RRD-GENESIS, LLM emergent narrative | `E8 Hexagram` simple rules → complex reasoning |
| **Faction/role asymmetry** | Theria, Heroes Arena | 7 domains with unique mechanics, dual specialization per session |
