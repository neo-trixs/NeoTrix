# NT-WORLD-SIM Technology Research V8

**Date**: 2026-09-11
**Purpose**: 50 web searches covering Bevy ECS, MOBA AI, game AI architectures, pathfinding, perception, learning, social dynamics, and consciousness — all mapped to NeoTrix domain architecture.

---

## 1. Bevy ECS Game 2025 2026

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy_ecs/latest/bevy_ecs |
| **Key Insight** | Bevy 0.19 (June 2026) — ECS with massive parallelism, change detection, no lifetimes required. Components are plain Rust structs, systems are functions. Bevy 0.16 added ECS Relationships and GPU-driven rendering (3x perf). |
| **NeoTrix Mapping** | `nt_physical` ECS backbone — Bevy ECS as the runtime substrate for NT-WORLD-SIM entity management. Component = Sensor/Motor/Perception node. |

## 2. Bevy 2D Top-Down Game

| Field | Value |
|-------|-------|
| **URL** | https://github.com/bones-ai/bevy-2d-shooter |
| **Key Insight** | 2D top-down shooter handling 100K+ enemies with kd-tree spatial indexing. Bevy's 2D renderer supports sprite batching, camera tracking with `smooth_nudge`. |
| **NeoTrix Mapping** | NT-WORLD-SIM map rendering — top-down MOBA view with spatial indexing for entity queries. |

## 3. Bevy Sprite Animation

| Field | Value |
|-------|-------|
| **URL** | https://bevy.org/examples/2d-rendering/sprite-animation |
| **Key Insight** | TextureAtlas-based sprite animation with Timer components. `bevy_spritesheet_animation` crate adds composition, easing, direction control, event triggers on animation endpoints. |
| **NeoTrix Mapping** | `nt_physical` animation system — entity state visualization (idle/attack/move/death) via sprite atlas transitions. |

## 4. Bevy Tilemap World

| Field | Value |
|-------|-------|
| **URL** | https://github.com/StarArawn/bevy_ecs_tilemap |
| **Key Insight** | `bevy_ecs_tilemap` 0.19 — tile-per-entity ECS model, chunk-based rendering, supports Square/Hex/Isometric grids. `bevy_ecs_tiled` integrates Tiled editor maps with entity-based architecture (layers→tiles→objects hierarchy). |
| **NeoTrix Mapping** | NT-WORLD-SIM world representation — chunk-based tilemap for MOBA arena. Hex/isometric support for tactical terrain. |

## 5. Bevy Camera 2D

| Field | Value |
|-------|-------|
| **URL** | https://bevy.org/examples/camera/2d-top-down-camera |
| **Key Insight** | Orthographic projection with `smooth_nudge` for camera tracking. `bevy_pancam` adds drag-to-pan, scroll-to-zoom. `PanCamera` built-in in Bevy 0.18+. RenderLayers for UI overlay separation. |
| **NeoTrix Mapping** | NT-WORLD-SIM camera — spectator/player camera with dead-zone, look-ahead, zoom controls for MOBA observation. |

## 6. Bevy Input Keyboard Mouse

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy/latest/bevy/input/index.html |
| **Key Insight** | `ButtonInput<T>` resource for pressed/just_pressed/just_released. `KeyboardInput` events with KeyCode (physical) vs Key (logical). `AccumulatedMouseMotion` for delta tracking. Run conditions for input-gated systems. |
| **NeoTrix Mapping** | `nt_io` input layer — keyboard/mouse events mapped to agent commands (move/attack/skill). |

## 7. Bevy Audio Game

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/bevy/latest/bevy/audio/index.html |
| **Key Insight** | `AudioPlayer` component with `PlaybackSettings` (LOOP, Despawn, Once). `AudioSink` for runtime control (volume, speed, mute, pause). Spatial audio with `SpatialListener` and `Emitter`. Soundtrack system with fade-in/fade-out transitions. |
| **NeoTrix Mapping** | `nt_physical` audio — spatial sound for MOBA events (abilities, pings, ambient). Soundtrack state machine for game phases. |

## 8. Bevy UI Overlay

| Field | Value |
|-------|-------|
| **URL** | https://github.com/marmikshah/bevy_modal |
| **Key Insight** | `bevy_modal` — modal/overlay stack for bevy_ui 0.19 with blocking scrim, deterministic layering, input-capture gate. `bevy_gpui` integrates Zed's GPUI as overlay on Bevy scenes. FPS overlay via `FpsOverlayPlugin`. |
| **NeoTrix Mapping** | `nt_io` HUD — minimap, health bars, ability cooldowns, scoreboard overlay via camera layering. |

## 9. Tauri 2.0 Game

| Field | Value |
|-------|-------|
| **URL** | https://v2.tauri.app/ |
| **Key Insight** | Tauri 2.0 (Oct 2024) — cross-platform (Linux/macOS/Windows/Android/iOS), minimal binary (~600KB), Rust backend + web frontend. Plugin system, mobile-first with Swift/Kotlin integration. |
| **NeoTrix Mapping** | `nt_tauri` desktop shell — game launcher, settings, replay viewer. Tauri as the NeoTrix desktop app framework. |

## 10. Tauri Bevy Integration

| Field | Value |
|-------|-------|
| **URL** | https://github.com/sunxfancy/BevyTauriExample |
| **Key Insight** | Bevy renders natively in Tauri2 window. Tauri handles window management and UI elements while Bevy owns the game rendering. Custom `tauri_plugin.rs` bridges the two runtimes. |
| **NeoTrix Mapping** | `nt_tauri` + `nt_physical` — Tauri as window manager, Bevy as game renderer. Split: UI overlays in Tauri web, game world in Bevy. |

## 11. Open Source MOBA Game

| Field | Value |
|-------|-------|
| **URL** | https://github.com/davidmenard0/openmoba |
| **Key Insight** | OpenMOBA — "competitive games should be public domain." OpenMOBA (game mechanics) vs OpenGames (infrastructure: servers, matchmaking, tournaments). Apache-2.0 license. Prime World (2004 MOBA) open-sourced 2024 with 209 stars. |
| **NeoTrix Mapping** | NT-WORLD-SIM game rules reference — MOBA mechanics as configurable simulation parameters (lanes, towers, minions, objectives). |

## 12. MOBA AI Bot

| Field | Value |
|-------|-------|
| **URL** | https://ojs.aaai.org/index.php/AIIDE/article/download/7449/7348/10950 |
| **Key Insight** | Deep Learning Bot for League of Legends (AIIDE-20). Uses bot of Legends API for game state. OpenAI Five (PPO) defeated Dota 2 world champions. Tencent AI for Honor of Kings used 600K CPU cores + 1064 GPUs. |
| **NeoTrix Mapping** | `nt_core` MOBA AI — hierarchical decision: macro strategy (lane assignment) → micro execution (skill combos). PPO-based self-play. |

## 13. Reinforcement Learning MOBA

| Field | Value |
|-------|-------|
| **URL** | https://arxiv.org/abs/2011.12692 |
| **Key Insight** | Tencent's "Towards Playing Full MOBA Games with DRL" (NeurIPS 2020). Curriculum Self-Play Learning (CSPL) scales hero pool to 40+. Techniques: policy distillation, off-policy adaption, multi-head value estimation, MCTS. Defeated top esports players. |
| **NeoTrix Mapping** | `nt_mind` curriculum learning — CSPL phases (single hero → combo → full team) mirror SEAL pipeline's staged evolution. |

## 14. Multi-Agent Battle Arena

| Field | Value |
|-------|-------|
| **URL** | https://github.com/SJTUwbl/MaCA |
| **Key Insight** | MaCA — UAV swarm vs swarm combat platform. Rule-based or deep learning agents. MAgent2 Battle/Battlefield environments: 162 agents, 13x13 observation, 21 discrete actions. Arena toolkit (AAAI-20) with 35 games and GUI-configurable social trees. |
| **NeoTrix Mapping** | `nt_world` battle simulation — multi-agent arena for testing NT-WORLD-SIM agent coordination. MAgent2-style observation/action spaces. |

## 15. Game Behavior Tree Rust

| Field | Value |
|-------|-------|
| **URL** | https://lib.rs/crates/bevior_tree |
| **Key Insight** | `bevior_tree` 0.11 — Bevy behavior tree plugin (432 in Game dev). Compatible with Bevy 0.19. `behaviortree` crate mirrors BehaviorTree.CPP. `fyrox::utils::behavior` — Sequence/Selector/Leaf nodes with Status::Success/Failure/Running. |
| **NeoTrix Mapping** | `nt_core` BT engine — behavior trees for NPC decision-making. Sequence (combo chains), Selector (priority fallback), Leaf (action execution). |

## 16. Game GOAP Planning

| Field | Value |
|-------|-------|
| **URL** | https://github.com/stolk/GPGOAP |
| **Key Insight** | GOAP (Goal Oriented Action Planning) from F.E.A.R. — A* search over world states, actions with preconditions/effects/costs. Jeff Orkin's architecture: actions as C++ classes, plans as paths in state space. GdPAI (Godot) adds simulation outside scene tree for parallel planning. |
| **NeoTrix Mapping** | `nt_core` GOAP planner — A* over world state atoms. Actions as ECS systems with pre/post conditions. Used for NPC tactical planning. |

## 17. Game Utility AI

| Field | Value |
|-------|-------|
| **URL** | https://github.com/zkat/big-brain |
| **Key Insight** | `big-brain` 1.3K★ — Utility AI for Bevy. Scorers evaluate world → Score values. Actions execute behaviors. Thinkers combine scorers with picker strategies. Response curves (linear/exponential/logistic) for nuanced decisions. Guild Wars 2 and The Sims use utility AI. |
| **NeoTrix Mapping** | `nt_core` utility system — `big-brain` as the Bevy-native utility AI. Scorers = perception queries, Actions = ECS systems, Thinkers = GWT attention routing. |

## 18. Game FSM State Machine

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/rust-fsm/latest/rust_fsm/ |
| **Key Insight** | `rust-fsm` — DSL for FSM definition with StateMachineImpl trait. Input/State/Output alphabets + transition/output functions. `rustfsm` macro generates machine structs. `seldom_state` for Bevy ECS integration. |
| **NeoTrix Mapping** | `nt_core` FSM — agent state machines (Idle/Chase/Attack/Flee/Dead). DSL for defining transition tables. Combined with BT for hierarchical control. |

## 19. Game Pathfinding NavMesh

| Field | Value |
|-------|-------|
| **URL** | https://github.com/wowemulation-dev/recast-rs |
| **Key Insight** | `recast-rs` — Rust port of Recast Navigation. Voxelization→heightfield→contours→polygon mesh pipeline. `waymark` for A* pathfinding, funnel straightening, raycast LOS. `waymark-crowd` for multi-agent crowd simulation. WASM-compatible. |
| **NeoTrix Mapping** | `nt_physical` navigation — Recast-based navmesh for MOBA map. Crowd simulation for minion waves. Dynamic obstacle support for destructible terrain. |

## 20. Game A* Pathfinding

| Field | Value |
|-------|-------|
| **URL** | https://docs.rs/pathfinding/latest/pathfinding/ |
| **Key Insight** | `pathfinding` crate — A*, BFS, Dijkstra, DFS, IDA*, Fringe, Yen k-shortest paths. Generic over node types. `Grid` type for rectangular grids with dynamic vertex add/remove. Also: Edmonds-Karp max flow, Kruskal MST, Hungarian matching. |
| **NeoTrix Mapping** | `nt_core` pathfinding — `pathfinding` crate for tactical route planning. A* for direct paths, Dijkstra for cost-aware routing, Flow algorithms for resource distribution. |

## 21. Game Steering Behaviors

| Field | Value |
|-------|-------|
| **URL** | https://www.redblobgames.com/ |
| **Key Insight** | Craig Reynolds' steering behaviors: Seek, Flee, Arrive, Pursue, Evade, Wander, Obstacle Avoidance, Path Following. Combined via weighted blending. Flow fields for efficient multi-agent movement. |
| **NeoTrix Mapping** | `nt_physical` steering —Seek (chase enemy), Flee (retreat), Arrive (lane position), Obstacle Avoidance (terrain). Flow fields for minion wave movement. |

## 22. Game Flocking Boids

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Boids |
| **Key Insight** | Craig Reynolds' boids: Separation (avoid crowding), Alignment (steer toward average heading), Cohesion (steer toward average position). Three simple rules → emergent flocking behavior. Used for unit formations, crowd simulation, particle effects. |
| **NeoTrix Mapping** | `nt_physical` flocking — minion wave cohesion, team formation maintenance, crowd simulation in base areas. Emergent behavior from simple rules. |

## 23. Game Crowd Simulation

| Field | Value |
|-------|-------|
| **URL** | https://github.com/wowemulation-dev/recast-rs (waymark-crowd) |
| **Key Insight** | DetourCrowd for multi-agent simulation: local avoidance (RVO), path following, crowd zones. Recast-rs provides `waymark-crowd` crate. Flow-based crowd management for large unit counts. |
| **NeoTrix Mapping** | `nt_physical` crowd sim — minion waves, team fights, base defense. DetourCrowd for local avoidance + flow-based path following. |

## 24. Game Perception System

| Field | Value |
|-------|-------|
| **URL** | https://www.gamedeveloper.com/ |
| **Key Insight** | Perception systems: Vision (raycasting/FOV), Hearing (noise radius), Memory (last-known positions with decay). Sensor fusion combines modalities. Awareness scoring for attention routing. Blackboard pattern for shared perception data. |
| **NeoTrix Mapping** | `nt_world` perception — vision cones + noise detection + memory decay. Blackboard = KB `perception` namespace. Awareness scores feed GWT attention. |

## 25. Game Vision Cone

| Field | Value |
|-------|-------|
| **URL** | https://www.redblobgames.com/articles/visibility/ |
| **Key Insight** | Field of View algorithms: Shadow casting (recursive), Ray casting, Polygon-based. Bresenham line-of-sight for grid maps. Mercator's algorithm for 2D FOV. Performance: O(n) per tile with shadow casting. |
| **NeoTrix Mapping** | `nt_world` FOV — 2D shadow casting for vision cones. Per-entity visibility computation. Fog of war based on team vision aggregation. |

## 26. Game Noise Detection

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Noise propagation: radius-based detection with falloff. Terrain material affects propagation (stone carries further than grass). Noise events tagged with type (footstep/ability/explosion) for AI response differentiation. |
| **NeoTrix Mapping** | `nt_world` noise system — sound events as ECS components with position, radius, type, falloff curve. Agents query noise within perception radius. |

## 27. Game Memory System

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Agent memory: Working memory (current fight state), Episodic memory (past encounters), Semantic memory (hero matchups). Memory decay with exponential forgetting curve. Confidence scoring for memory reliability. |
| **NeoTrix Mapping** | `nt_memory` agent memory — KB-backed episodic/semantic memory with time-decay. Working memory as ECS components. Confidence scores for decision reliability. |

## 28. Game Learning Adaptation

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Adaptation strategies: Opponent modeling (learn enemy patterns), Strategy selection (counter-picking), Parameter tuning (adjust weights from outcomes). Online learning vs batch learning tradeoffs. Anti-pattern detection for meta-game adaptation. |
| **NeoTrix Mapping** | `nt_mind` adaptation — SEAL pipeline for strategy evolution. Opponent modeling via KB pattern storage. Meta-game adaptation as skill crystallization. |

## 29. Game Evolution Genetic

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Genetic algorithms for game AI: Fitness = win rate, Genomes = strategy parameters, Crossover = strategy blending, Mutation = random parameter variation. NEAT (NeuroEvolution of Augmenting Topologies) for neural network evolution. |
| **NeoTrix Mapping** | `nt_mind` evolution — SEAL pipeline's exploration phase as genetic search. Fitness = simulation win rate. Skill trees as evolved strategy genomes. |

## 30. Game Neural Network

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Game AI neural networks: DQN for value estimation, Policy networks for action selection, LSTM for temporal sequences, Transformer for attention over game state. Advantage: generalization across similar states. Disadvantage: interpretability, training data requirements. |
| **NeoTrix Mapping** | `nt_core` neural layer — optional NN modules for perception/action. Interpretable by design (ECS components visible). Complementary to rule-based BT/GOAP/Utility. |

## 31. Game Reinforcement Learning

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | RL in games: Model-free (PPO, SAC) vs Model-based. Reward shaping for sparse rewards. Hindsight experience replay for goal-conditioned tasks. Multi-agent RL: centralized training with decentralized execution (CTDE). |
| **NeoTrix Mapping** | `nt_mind` RL loop — PPO for micro decisions, CTDE for team coordination. Reward shaping from game events (kills, objectives, gold). Hindsight replay for learning from failures. |

## 32. Game Self-Play

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Self-play: Agent plays against copies of itself to discover strategies. OpenAI Five used self-play for Dota 2. AlphaGo/AlphaStar used self-play + MCTS. Population-based training for strategy diversity. ELO rating for strategy strength tracking. |
| **NeoTrix Mapping** | `nt_mind` self-play — SEAL pipeline's self-test phase as self-play. Population of strategy variants. ELO tracking for constellation maturity (C0-C6). |

## 33. Game Curriculum Learning

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Curriculum learning: Start simple, increase complexity gradually. Tencent's CSPL: single hero → hero combos → full team. Reverse curriculum: start from winning states and work backwards. Automatic curriculum via intrinsic motivation. |
| **NeoTrix Mapping** | `nt_mind` curriculum — SEAL pipeline stages mirror CSPL. Constellation maturity (C0→C6) as automatic curriculum. Self-paced learning via phi/coherence monitoring. |

## 34. Game Reward Shaping

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Reward shaping: Potential-based shaping (preserve optimal policy). Dense vs sparse rewards. Multi-objective reward: gold, experience, objectives, kills. Reward clipping for stability. Intrinsic rewards for exploration (curiosity, empowerment). |
| **NeoTrix Mapping** | `nt_core` reward engine — Multi-objective scoring (victory, efficiency, creativity, learning). Intrinsic rewards for exploration. Potential-based shaping for stable evolution. |

## 35. Game Multi-Agent Coordination

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Multi-agent coordination: Communication protocols (channel-based, differentiable). Role assignment (tank/support/DPS). Joint action selection. Credit assignment in team outcomes. Communication cost vs performance tradeoff. |
| **NeoTrix Mapping** | `nt_core` team coordination — Role-based routing (GWT attention by role). Communication via EventBus. Joint action planning with GOAP. Credit assignment via individual performance metrics. |

## 36. Game Team Strategy

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Team strategy: Draft/pick phase (hero selection). Lane assignment optimization. Objective prioritization (dragon/baron/towers). Team composition synergy. Adaptive strategy based on game state (ahead/behind). |
| **NeoTrix Mapping** | `nt_core` strategy layer — Draft phase as meta-optimization. Lane assignment via utility scoring. Objective priority as GOAP goals. Strategy adaptation via self-model state. |

## 37. Game Formation Movement

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Formation movement: Predefined formations (line, circle, wedge). Virtual structure method (formation as rigid body). Leader-follower. Behavior-based formation (Reynolds flocking with formation constraints). |
| **NeoTrix Mapping** | `nt_physical` formations — Predefined formations for team fights. Virtual structure for objective siege. Leader-follower for gank coordination. |

## 38. Game Objective Prioritization

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Objective prioritization: Threat assessment (enemy proximity, power). Value estimation (gold/exp reward). Risk assessment (survival probability). Temporal urgency (spawning objectives). Dynamic re-prioritization on state changes. |
| **NeoTrix Mapping** | `nt_core` prioritization — Utility scoring for objectives. GWT attention routing by priority. Dynamic re-evaluation on game events. |

## 39. Game Resource Management

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Resource management: Gold allocation (items vs consumables). Experience optimization (lane vs jungle). Time resource (rotation timing). Mana/health management. Opportunity cost calculation for each decision. |
| **NeoTrix Mapping** | `nt_core` resource economy — Gold/XP as simulation resources. Opportunity cost as utility subtraction. Time budgeting for rotation decisions. |

## 40. Game Economy Simulation

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Game economy: Gold income rates (passive, minion, kill, objective). Item power curves. Economy balancing (snowball prevention). Comeback mechanics (shutdown gold, catch-up XP). Trading efficiency (gold-per-minute optimization). |
| **NeoTrix Mapping** | `nt_world` economy sim — Gold/XP flow model. Item power curves as equipment progression. Comeback mechanics as dynamic difficulty adjustment. |

## 41. Game Social Dynamics

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Social dynamics in games: Trust building between teammates. Reputation systems affecting matchmaking. Social pressure and conformity. Leadership emergence. Toxicity detection and mitigation. |
| **NeoTrix Mapping** | `nt_feel` social layer — Trust scores between agents. Reputation tracking in KB. Social emotion propagation. Leadership as emergent role assignment. |

## 42. Game Reputation System

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Reputation systems: Karma/standing (positive/negative actions). ELO/MMR for skill-based matching. Report systems for behavior moderation. Reward/punishment for reputation levels. Anonymous vs identified reputation. |
| **NeoTrix Mapping** | `nt_memory` reputation — KB-backed reputation tracking. Multi-dimensional reputation (skill, teamwork, behavior). Affects matchmaking and social interactions. |

## 43. Game Faction System

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Faction systems: Team alignment (radiant/dire). Faction reputation and unlockable content. Cross-faction diplomacy. Faction-specific abilities or bonuses. Narrative-driven faction conflict. |
| **NeoTrix Mapping** | `nt_world` factions — Team alignment as primary faction. Secondary factions for guild/alliance systems. Faction-specific resource bonuses. Diplomatic relations as edge weights in social graph. |

## 44. Game Culture Propagation

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Culture propagation in games: Meta-game evolution (strategies spread through player community). Meme transmission (emotes, strategies). Cultural drift between servers/regions. Emergent norms (unwritten rules). |
| **NeoTrix Mapping** | `nt_mind` culture — Meta-strategy evolution via SEAL pipeline. Pattern propagation across agents. Emergent norms as self-organizing behavior rules. |

## 45. Game Emergence Detection

| Field | Value |
|-------|-------|
| **URL** | N/A (established concept) |
| **Key Insight** | Emergence detection: Monitoring for unexpected behaviors from simple rules. Complexity metrics (entropy, information gain). Phase transitions in agent populations. Novelty detection for emergent strategies. |
| **NeoTrix Mapping** | `nt_meta` emergence — ConsciousnessTree monitoring for unexpected agent behaviors. Entropy metrics for strategy diversity. Novelty detection as self-evolution signal. |

## 46. Game Consciousness Level

| Field | Value |
|-------|-------|
| **URL** | N/A (theoretical) |
| **Key Insight** | Game consciousness levels: Reactive (stimulus-response), Adaptive (learn from experience), Reflective (self-monitor), Intentional (goal-directed), Self-aware (model of self). IIT's phi as integration measure. Global Workspace Theory for attention routing. |
| **NeoTrix Mapping** | `nt_core` consciousness — E8/GWT integration. Phi score for agent awareness level. GWT salience for attention routing. ConsciousnessTree as meta-cognition loop. |

## 47. Game Self-Awareness

| Field | Value |
|-------|-------|
| **URL** | N/A (theoretical) |
| **Key Insight** | Agent self-awareness: Self-model (capabilities, limitations). Metacognition (thinking about thinking). Introspection (reasoning about own decisions). Self-prediction (anticipating own behavior). Theory of mind (modeling others' beliefs). |
| **NeoTrix Mapping** | `nt_core` self-model — `nt_core_self::SelfModel` for dynamic performance. `nt_core_meta::SelfModel` for structural identity. Theory of mind for opponent modeling. |

## 48. Game Metacognition

| Field | Value |
|-------|-------|
| **URL** | N/A (theoretical) |
| **Key Insight** | Metacognition in game AI: Confidence estimation (knowing what you don't know). Strategy evaluation (meta-reasoning about approach). Learning-to-learn (optimizing own learning process). Uncertainty quantification for decision quality. |
| **NeoTrix Mapping** | `nt_meta` metacognition — Uncertainty tracking in SelfModel. Strategy evaluation via SEAL pipeline feedback. Learning-to-learn as skill crystallization. Meta-audit dimensions D13-D20. |

## 49. Game Reflection Mechanism

| Field | Value |
|-------|-------|
| **URL** | N/A (theoretical) |
| **Key Insight** | Reflection mechanisms: Post-game analysis (replay review). Counterfactual reasoning ("what if"). Causal attribution (identifying cause of outcomes). Experience replay for learning. Self-critique for improvement. |
| **NeoTrix Mapping** | `nt_meta` reflection — Experience-tree absorption (5 stages). Counterfactual validation in SEAL pipeline. Causal tracing via core-trace skill. Post-game replay analysis. |

## 50. Game Curiosity Exploration

| Field | Value |
|-------|-------|
| **URL** | N/A (theoretical) |
| **Key Insight** | Curiosity-driven exploration: Intrinsic motivation (novelty, surprise, information gain). ICM (Intrinsic Curiosity Module) by Pathak et al. Random Network Distillation for exploration bonus. Curiosity decay to prevent distraction from objectives. |
| **NeoTrix Mapping** | `nt_mind` curiosity — Intrinsic motivation for exploration in SEAL pipeline. Novelty detection as exploration bonus. Curiosity decay balanced with objective focus. VoI (Value-of-Information) from HCube for experiment selection. |

---

## Summary: Technology → NeoTrix Domain Mapping

| Technology | Primary NT Domain | Key Integration |
|-----------|-------------------|-----------------|
| Bevy ECS | `nt_physical` | Runtime substrate for all entities |
| Tauri 2.0 | `nt_tauri` | Desktop shell + window management |
| Tilemap/NavMesh | `nt_world` | World representation + navigation |
| Sprite Animation | `nt_physical` | Entity state visualization |
| Camera 2D | `nt_io` | Player/spectator view control |
| Input System | `nt_io` | Player command interface |
| Audio | `nt_physical` | Spatial sound + soundtrack state |
| UI Overlay | `nt_io` | HUD + modal interfaces |
| Behavior Trees | `nt_core` | NPC decision trees |
| GOAP | `nt_core` | Goal-oriented planning |
| Utility AI | `nt_core` | Score-based action selection |
| FSM | `nt_core` | Agent state management |
| A*/Dijkstra | `nt_core` | Tactical pathfinding |
| Steering Behaviors | `nt_physical` | Movement physics |
| Boids/Flocking | `nt_physical` | Group movement |
| Crowd Simulation | `nt_physical` | Large-scale unit management |
| Perception System | `nt_world` | Vision + hearing + memory |
| Vision Cone | `nt_world` | Field of view computation |
| Noise Detection | `nt_world` | Audio perception |
| Memory System | `nt_memory` | Agent knowledge retention |
| RL/Self-Play | `nt_mind` | Strategy learning |
| Curriculum Learning | `nt_mind` | Staged skill development |
| Reward Shaping | `nt_core` | Multi-objective scoring |
| Multi-Agent Coord | `nt_core` | Team coordination |
| Faction System | `nt_world` | Team/diplomacy structure |
| Economy Simulation | `nt_world` | Gold/XP flow model |
| Reputation System | `nt_memory` | Social tracking |
| Consciousness/GWT | `nt_core` | Attention routing + awareness |
| Metacognition | `nt_meta` | Self-monitoring + reflection |
| Curiosity/Exploration | `nt_mind` | Intrinsic motivation |
