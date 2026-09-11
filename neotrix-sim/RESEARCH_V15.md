# RESEARCH_V15 — 20-Source Game AI & Engine Survey

> Generated: 2026-09-11 | Sources: 20 web searches | Focus: MOBA AI, ECS, Game Architecture, Self-Play, Emergence

---

## 1. Bevy ECS Game (2025)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/linzell/space-looter |
| **Key Insight** | Space Looter: 3D isometric RPG built with Bevy ECS + Domain-Driven Design. Key pattern: `infrastructure/bevy/` as a thin adapter layer over pure domain logic. WASM-compiled for cross-browser deployment. Bevy ECS used for modular game state management with `game_state.rs`, `input.rs`, `rendering.rs`. |
| **NeoTrix Mapping** | Validates Bevy as a viable ECS runtime for nt-sim. The DDD layering (`domain/ → infrastructure/bevy/ → presentation/`) mirrors NeoTrix's 6-layer architecture. WASM deployment path enables browser-based sim instances. Pattern: `bevy/` as thin adapter, not architectural anchor. |

---

## 2. MOBA AI Bot

| Field | Value |
|-------|-------|
| **URL** | https://github.com/The-JDdev/Heroes-Arena |
| **Key Insight** | Heroes Arena: Full open-source 5v5 MOBA (Mobile Legends-style) in Kotlin. Features: 7 heroes × 4 skills + passive, 6 classes (Tank/Fighter/Assassin/Mage/Marksman/Support), minion waves, tower defense, jungle monsters, 25+ item shop, ELO ranking. Client-server architecture with real-time combat. |
| **NeoTrix Mapping** | Hero class taxonomy maps to NT-* faction specialization. Item shop + build paths = Constellation rune-socketing system (5 rune colors). Minion wave AI = emergent unit behavior from simple spawning rules. Class role system (Tank/Assassin/etc.) = Ascendancy dual-specialization (Weapon Set I/II). |

---

## 3. Game Behavior Tree

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control) |
| **Key Insight** | BTs are the dominant AI technique in AAA games (Halo, BioShock, Spore). Key types: Sequence (AND), Selector (OR), Decorator (transform). Event-driven BTs solve scalability by aborting running nodes on world changes. Blackboard = shared memory across BT instances. Three statuses: Running/Success/Failure. |
| **NeoTrix Mapping** | BT Selector ≈ GWT attention routing (try best option, fall back). BT Sequence ≈ SEAL pipeline stages (ordered, all-must-succeed). Blackboard ≈ KB namespace shared state. Event-driven BTs map to ConsciousnessTree's reactive consciousness loop. Decorator nodes = meta-cognitive modifiers (self-test, repair hooks). |

---

## 4. Game Pathfinding

| Field | Value |
|-------|-------|
| **URL** | http://theory.stanford.edu/~amitp/GameProgramming/AStarComparison.html |
| **Key Insight** | A* combines Dijkstra (guaranteed shortest) + Greedy Best-First (fast heuristic). f(n) = g(n) + h(n). Heuristic design critical: Manhattan for grids, Euclidean for free movement. Hierarchical Pathfinding A* (HPA*) partitions maps into clusters for large-world performance. Metaheuristic techniques (GA, ACO) outperform A* in time/memory for dynamic environments. |
| **NeoTrix Mapping** | A* heuristic ≈ GWT salience scoring (cost-to-go + urgency). HPA* cluster hierarchy ≈ NT-WORLD's PerceptionBridge abstraction layers. Flow fields (gradient of distance field) ≈ VSA HyperCube vector space traversal. Dynamic replanning ≈ ConsciousnessTree cycle re-evaluation. Multi-agent pathfinding = team coordination challenge. |

---

## 5. Game Memory

| Field | Value |
|-------|-------|
| **URL** | https://game-developers.org/memory-management-in-games-explained |
| **Key Insight** | Memory management is the invisible architecture of game performance. Key patterns: Arena allocators (50-100x speedup over malloc), pool allocators for fixed-size objects, budgeted caches with eviction. Fragmentation is the silent killer — 500MB free but scattered blocks fails large allocation. Four critical metrics: heap usage, allocation rate, fragmentation, GPU residency. |
| **NeoTrix Mapping** | Arena allocator pattern ≈ NT-MEMORY's KB namespace isolation (experience/knowledge separate from runtime). Pool allocators ≈ Constellation node allocation (fixed tiers: Small Passive/Notable Passive/Keystone). Memory budgets ≈ Rune Socketing capacity limits (5 slots). Fragmentation prevention ≈ Dark Forest axiom (compile + test + connect or delete). |

---

## 6. Game Evolution

| Field | Value |
|-------|-------|
| **URL** | https://www.cse.unr.edu/~bdbryant/papers/stanley-2006-aaai.pdf |
| **Key Insight** | rtNEAT (real-time NeuroEvolution of Augmenting Topologies): evolve neural networks as game is played. NERO game: players train robot teams via custom training exercises. Agents learn attacking, dodging, navigation from scratch. Key: evolution speed enables real-time player interaction — no offline training required. Complexification: start simple, add nodes/connections as behavior improves. |
| **NeoTrix Mapping** | rtNEAT complexification ≈ Constellation maturity ladder (C0→C5). Player-as-trainer ≈ NeoTrix SEAL pipeline where agent trains itself. NERO's training curriculum ≈ experience-tree five-stage absorption (snapshot→distill→classify→persist→feedback). Real-time evolution ≈ ConsciousnessTree's live growth cycles. Emergent tactics from simple fitness = GWT attention-driven behavior emergence. |

---

## 7. Game Self-Play

| Field | Value |
|-------|-------|
| **URL** | https://arxiv.org/abs/2408.01072 |
| **Key Insight** | Comprehensive survey of self-play methods in MARL. Four categories: Traditional self-play, PSRO series, Ongoing-training-based, Regret-minimization-based. AlphaGo Zero: pure self-play from tabula rasa, no human data. Key challenge: convergence to suboptimal strategies + computational requirements. Self-play generates unlimited training data through game dynamics alone. |
| **NeoTrix Mapping** | Self-play ≈ NT-MIND's self-evolution loop (agent improves by competing against historical versions). Policy population ≈ skill tree node variants (multiple strategies per domain). PSRO meta-strategy ≈ GWT attention routing across competing specialist modules. AlphaGo Zero's "ignition mechanism" ≈ experience-tree cold-start bootstrap. Multi-game transfer (SPIRAL) ≈ cross-domain pattern propagation in ConsciousnessTree. |

---

## 8. Game Team Strategy

| Field | Value |
|-------|-------|
| **URL** | https://arxiv.org/html/2508.06042 |
| **Key Insight** | HIMA: Hierarchical Imitation Multi-Agent framework for StarCraft II. Society of Mind principle — specialized agents under a Strategic Planner meta-controller. Temporal Chain-of-Thought (t-CoT): immediate/short-term/long-term action alignment. Structured decision via Nominal Group Technique. Agents specialize by unit composition (ground-heavy vs air-heavy). |
| **NeoTrix Mapping** | HIMA hierarchy ≈ NeoTrix 6-layer architecture (L6 meta → L5 cognition → L4 emotion → ...). Strategic Planner ≈ NT-CORE's E8 Hexagram reasoning engine. Temporal CoT ≈ ConsciousnessTree's 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). Specialized agents ≈ NT-* domain factions (each with unique capabilities). NGT conflict resolution ≈ NT-GOVERNANCE arbitration. |

---

## 9. Game Emergence

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Emergent_gameplay |
| **Key Insight** | Emergent gameplay = complex situations from simple mechanics interaction. Intentional emergence (designer-encouraged) vs unintentional (glitches/physics quirks). Left 4 Dead's "Director" = procedural narrative that adapts to player performance. Key design principle: few core mechanics + combinatorial interactions > many isolated systems. Prologue (PLAYERUNKNOWN) uses ML-generated terrain for emergence. |
| **NeoTrix Mapping** | Emergence ≈ SEAL pipeline's self-evolution (simple rules → complex behavior). Prologue's ML terrain generation ≈ NT-WORLD's crawl pipeline generating novel knowledge spaces. Left 4 Dead Director ≈ GWT attention modulation based on system health. "Few core mechanics" principle ≈ NeoTrix axiom: KISS across 7 domains, composition over complexity. Unintentional emergence = the goal of self-play and Constellation cross-domain interactions. |

---

## 10. Game Consciousness

| Field | Value |
|-------|-------|
| **URL** | https://arxiv.org/abs/2608.21439 |
| **Key Insight** | WorldMind: first decoupled framework for state-aware NPC behavior. Four layers: Understanding (compact state from frames) → Decision (reason over state) → Control (translate actions) → Generation (visual output). BOSS-140K dataset: gameplay videos paired with rich internal game states. 70% preference over baselines for tactically appropriate NPC behavior. |
| **NeoTrix Mapping** | WorldMind's 4-layer decoupling ≈ NeoTrix PerceptionBridge (L2 perception → L5 consciousness flow). Understanding Layer ≈ NT-SENSE sensory integration. Decision Layer ≈ NT-CORE E8 reasoning. Control Layer ≈ NT-ACT action execution. Generation Layer ≈ NT-IO output. The "compact state" concept validates VSA HyperCube as state compression mechanism. |

---

## 11. Open Source MOBA

| Field | Value |
|-------|-------|
| **URL** | https://github.com/ajhahnde/Theria |
| **Key Insight** | Theria: 2.5D MOBA in Godot 4. Server-authoritative, deterministic simulation. Two tribes: Solane (burst damage, big-cat shifters) vs Verdani (attrition, venom/spider shifters). Fixed-timestep simulation for network determinism. Click-to-move MOBA controls. Practice/Host/Join networking model. |
| **NeoTrix Mapping** | Deterministic simulation ≈ NeoTrix requirement for reproducible agent behavior. Tribe asymmetry (burst vs attrition) ≈ Ascendancy dual-specialization (Weapon Set I: acquisition vs Weapon Set II: evolution). Server-authoritative ≈ NT-SHIELD security model (centralized trust). Two-tribe design ≈ minimum viable faction system for emergent team dynamics. |

---

## 12. Bevy 2D

| Field | Value |
|-------|-------|
| **URL** | https://bevy.org/ |
| **Key Insight** | Bevy: data-driven game engine in Rust. Modular: use only what you need. 2D features: sprite sheets, dynamic texture atlases, cameras, textures, materials. Custom shaders/materials/render pipelines. WASM support via WebGL2/WebGPU. Feature flags: `2d`, `3d`, `ui`, `audio` — compile only what you need. |
| **NeoTrix Mapping** | Bevy's modularity ≈ NeoTrix's "replace what you don't like" philosophy. Feature-flag compilation ≈ Constellation maturity (C0=compiles → C5=self-healing). WASM deployment ≈ browser-accessible sim instances. Bevy ECS as Rust-native ECS validates architectural choice for NeoTrix (no FFI overhead vs Unity C# interop). Data-oriented design aligns with NeoTrix's data-flow-first approach. |

---

## 13. Game Utility AI

| Field | Value |
|-------|-------|
| **URL** | https://github.com/sinbad/SUSS |
| **Key Insight** | SUSS (Steve's Utility AI SubSystem): Actions scored by Considerations (curves × inputs), weighted random selection with cut-off. Actions grouped by Priority Groups. Curves: linear/quadratic/exponential/custom. Contexts generated by Queries (perceived targets, available abilities). Unlike BTs/FSMs/GOAP — evaluates ALL decisions, picks highest score. |
| **NeoTrix Mapping** | Utility AI scoring ≈ GWT salience function (attention weight = f(urgency, relevance, cost)). Consideration curves ≈ VSA HyperCube similarity scoring (cosine distance in vector space). Priority Groups ≈ NT-* domain routing (which faction handles this task?). Weighted random selection ≈ GWT attention broadcast with stochastic exploration. Utility AI excels at decision-making but not execution — pair with BT for action sequences. |

---

## 14. Game GOAP

| Field | Value |
|-------|-------|
| **URL** | https://github.com/crashkonijn/GOAP |
| **Key Insight** | GOAP: agents form action chains at runtime from world facts. Planner uses A* search over preconditions/effects/costs. Multi-threaded via Unity Job System. Key advantage over BTs: no predefined sequences — agent discovers solutions. Drawbacks: higher complexity, harder to debug, less control over "fun". ReGoap adds weighted-random goal selection for more natural behavior. |
| **NeoTrix Mapping** | GOAP planning ≈ SEAL pipeline's distillation phase (world state → goal → action chain). Preconditions/effects ≈ KB edge semantics (domain relationships). A* over action space ≈ E8 Hexagram pathfinding through reasoning states. GOAP's emergent sequences ≈ Constellation cross-module interactions producing unanticipated behaviors. Weighted-random goals ≈ GWT attention with exploration bonus. |

---

## 15. Game Faction

| Field | Value |
|-------|-------|
| **URL** | https://dl.acm.org/doi/fullHtml/10.1145/3649921.3650012 |
| **Key Insight** | Taxonomy of game faction systems: Absolute (role-based), Allegiance (player choice), World-based (exploration-revealed), Procedurally-generated. Key finding: NPCs lack character development and autonomy — they just react to players. Reputation systems: linear vs multidimensional, global vs local. Faction design must balance visual identity + gameplay loop + lore hook. |
| **NeoTrix Mapping** | Faction taxonomy ≈ NT-* domain system (7 factions with distinct identities). Reputation system ≈ KB edge weights (trust/distrust between domains). Player-choice allegiance ≈ GWT attention routing (which domain gets this task?). Procedural factions ≈ NT-WORLD's crawl-generated knowledge entities. Key gap: NeoTrix domains must evolve autonomously (ConsciousnessTree growth cycles) unlike static game factions. |

---

## 16. Unity DOTS ECS

| Field | Value |
|-------|-------|
| **URL** | https://unity.com/dots |
| **Key Insight** | Unity DOTS: Data-Oriented Technology Stack. ECS scales to 128+ players. Deterministic simulation enables rollback networking. Used in production: V Rising (open-world survival), IXION (city builder NPC simulation), Hardspace: Shipbreaker (process optimization 1hr → 100ms). Havok Physics integration for complex simulations. Hybrid renderer bridges GameObjects ↔ Entities. |
| **NeoTrix Mapping** | DOTS ECS determinism ≈ NeoTrix requirement for reproducible agent behavior across sessions. Havok Physics ≈ NT-PHYSICAL's physics simulation needs. 128+ player scaling ≈ multi-agent sim with many concurrent NT-* domain agents. Hybrid renderer pattern ≈ NeoTrix's L3 Embodiment layer bridging abstract cognition with physical representation. DOTS performance validates ECS as architecture for large-scale simulations. |

---

## 17. Unity ML-Agents Training

| Field | Value |
|-------|-------|
| **URL** | https://unity-technologies.github.io/ml-agents/Training-ML-Agents/ |
| **Key Insight** | ML-Agents: PPO/SAC trainers with self-play, curriculum learning, behavioral cloning, curiosity (ICM), GAIL. YAML-config-driven. Self-play: `save_steps` controls opponent diversity; `team_change` for alternating opponents. Reward signals: extrinsic + intrinsic. Memory via LSTM. Environment parameter randomization for robustness. Concurrent Unity instances for parallel training. |
| **NeoTrix Mapping** | PPO/SAC training ≈ NT-MIND's SEAL pipeline optimization. Self-play config (save_steps/team_change) ≈ experience-tree absorption frequency. Curriculum learning ≈ Constellation maturity progression (C0→C5). Curiosity-driven exploration ≈ GWT attention with intrinsic motivation bonus. Environment randomization ≈ NeoTrix's multi-domain stress testing (each NT-* domain as different "environment"). |

---

## 18. Unity MOBA Template

| Field | Value |
|-------|-------|
| **URL** | https://github.com/NicoRuedaA/Mobalike |
| **Key Insight** | Mobalike: Unity 6 MOBA prototype. "Brain and Body" architecture — logic (Brain) separate from representation (Body). Data-driven abilities via ScriptableObjects. 5 behavior types: projectile, AoE, trail, buff, smash. Entity framework: BaseEntity → HeroEntity → EnemyEntity. Complete systems: movement, combat, targeting, inventory (20 slots), equipment (6 slots with STR/AGI/INT). |
| **NeoTrix Mapping** | "Brain and Body" separation ≈ NeoTrix L5 Cognition (Brain) ↔ L3 Embodiment (Body) layer split. Data-driven abilities ≈ SKILL-SPEC.md contract (skill as template, not prompt). Entity hierarchy ≈ NT-* domain entity types (nodes, edges, embeddings). Ability behavior types ≈ Rune Socketing effects (5 colors → emergent Runewords). Equipment stats (STR/AGI/INT) ≈ Ascendancy specialization axes. |

---

## 19. Unity Game Architecture

| Field | Value |
|-------|-------|
| **URL** | https://unity.com/how-to/how-architect-code-your-project-scales |
| **Key Insight** | Unity architecture principles: Single Responsibility Principle, separate logic/presentation, use message bus for decoupling. ScriptableObjects for shared settings, Prefabs for instances. Key insight: "run code in two modes — logic only and logic plus presentation." Events/queues between systems. Regular C# classes for logic (better testability than MonoBehaviours). |
| **NeoTrix Mapping** | Logic/presentation separation ≈ NeoTrix's "app logic should run quickly, and when possible, in parallel" (Bevy principle). Message bus ≈ EventBus in NeoTrix (inter-domain communication). ScriptableObjects ≈ KB node templates (reusable configurations). Two-mode execution ≈ NT-IO (interface layer) being swappable without touching core logic. Testable pure C# ≈ NeoTrix's `#![forbid(unsafe_code)]` safety guarantee. |

---

## 20. Unity AI Navigation

| Field | Value |
|-------|-------|
| **URL** | https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html |
| **Key Insight** | NavMesh: convex polygon walkable surface baked from scene geometry. A* for global pathfinding, RVO (Reciprocal Velocity Obstacles) for local collision avoidance. Key distinction: global navigation (corridor finding, expensive) vs local navigation (steering, cheap). Dynamic obstacles: moving = local avoidance, stationary = NavMesh carving. NavMesh Links for jump/door actions. |
| **NeoTrix Mapping** | NavMesh polygon hierarchy ≈ VSA HyperCube vector space (abstract walkable regions). Global vs local navigation ≈ GWT global broadcast vs NT-ACT local action execution. RVO collision avoidance ≈ NT-SHIELD's interference detection between domain agents. NavMesh Links ≈ ConsciousnessTree cross-domain bridges (special actions connecting unrelated domains). Dynamic obstacle handling ≈ real-time adaptation when KB entities change. |

---

## Cross-Cutting Patterns

| Pattern | Sources | NeoTrix Integration |
|---------|---------|---------------------|
| **Hierarchical Decision** | BT, GOAP, HIMA, Utility AI | L6→L5→L4→...→L1 layered reasoning |
| **Shared Blackboard/Memory** | BT Blackboard, GOAP World State, KB | NT-MEMORY as single fact source |
| **Emergent from Simple Rules** | Emergence, Self-Play, rtNEAT | SEAL pipeline + Constellation cross-links |
| **Data-Driven Configuration** | Bevy, Unity SO, ML-Agents YAML | SKILL-SPEC.md + Rune Socketing |
| **Deterministic Simulation** | Theria, Unity DOTS, GOAP | Reproducible agent behavior across sessions |
| **Layered Architecture** | HIMA, WorldMind, Unity Architect | 6-Layer Architecture (L1-L6) |
| **Real-Time Adaptation** | Event-driven BT, Utility AI, Self-Play | ConsciousnessTree live growth cycles |

---

## Key Takeaways for nt-sim

1. **Bevy ECS is production-viable** for MOBA-scale simulations (Rust-native, WASM deployable, modular)
2. **Utility AI + BT hybrid** is the industry standard: Utility for decision selection, BT for action execution
3. **GOAP adds emergence** but trades debuggability — use for complex NPC behavior, BT for predictable units
4. **Self-play generates unlimited training data** — core mechanism for NT-MIND's self-evolution
5. **Hierarchical team strategy** (Society of Mind) maps directly to NeoTrix's multi-domain architecture
6. **Faction identity requires 5 pillars**: visual identity, gameplay loop, lore hook, territory, face (NPC)
7. **Memory management = performance** — arena allocators, pool allocators, fragmentation prevention
8. **Pathfinding hierarchy** (global A* + local RVO) is essential for multi-agent coordination
9. **Emergence comes from combinatorial simplicity**, not complex individual systems
10. **Consciousness in games** = decoupled state-aware decision-making layers (WorldMind pattern)
