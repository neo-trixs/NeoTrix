# RESEARCH_V13 — Game AI Literature Survey

> 15-source parallel search. Each entry: URL, key insight, NeoTrix mapping.

---

## 1. Bevy ECS Game 2025

**URLs:**
- https://github.com/linzell/space-looter — 3D isometric RPG with DDD + Bevy ECS + WASM
- https://github.com/xiongchenyu6/bevy-open-siege — Lane-defense strategy, 10 plant types, 10 undead types
- https://github.com/nixonyh/bunguette — Couch co-op, Bevy + Avian physics + Leafwing input
- https://github.com/foxzool/last-stop — Bus route puzzle, A* pathfinding, ECS architecture

**Key Insight:** Bevy ECS in 2025 is production-viable for complex games. The `space-looter` project proves Domain-Driven Design maps cleanly onto ECS: `domain/` → components, `services/` → systems, `infrastructure/bevy/` → Bevy integration layer. This is the exact pattern NeoTrix needs — domain logic decoupled from engine specifics.

**NeoTrix Mapping:** Adopt DDD-over-ECS pattern from `space-looter`. Our `neotrix-sim/src/` already separates `components/` (data) from `systems/` (logic). Extend this with an `infrastructure/` layer for Bevy-specific wiring. The `bevy-open-siege` data-driven level system (`assets/data/levels.ron`) validates our RON-based configuration approach.

---

## 2. MOBA AI Bot

**URLs:**
- https://github.com/adrian27513/MOBA-AI-Gamer — LoL bot: YoloV5 object detection + OCR + Deep Q-Learning
- https://github.com/kgemas/League-AI — Auto-play LoL bot, keyboard/mouse emulation
- https://github.com/forest0xia/dota2bot-OpenHyperAI — Full Dota 2 bot architecture, 14K+ line ability system
- https://gamebot.ai/product/game-bot — Commercial MOBA bots: auto-fill, confidence-building, 9% DAU increase

**Key Insight:** The Dota 2 bot (`OpenHyperAI`) reveals the real complexity: `ability_item_usage_generic.lua` alone is ~8000 lines. Each hero has a `BotLib/hero_[name].lua` with role-specific logic. The `FunLib/` contains 15+ utility libraries (items, skills, roles, positions, spells). Key lesson: MOBA AI is 20% decision-making, 80% data (item builds, skill orders, matchup tables).

**NeoTrix Mapping:** Our `nt_world::knowledge_base` should store hero/item/matchup data as KB nodes, not hardcoded Lua. The `FretBots` difficulty scaling system (bonus gold/XP multipliers) maps to our `nt_core_self::DifficultyController`. The behavior mode system (`mode_*_generic.lua` × 12 modes) maps to our `nt_act::GoalPlanner` with mode-specific sub-goals.

---

## 3. Game Behavior Tree

**URLs:**
- https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control) — Formal model: Sequence (AND), Selector (OR), Decorator, Leaf
- https://www.gamedeveloper.com/programming/behavior-trees-for-ai-how-they-work — Practical guide: tick-based, 3 statuses (Running/Success/Failure), blackboard integration
- https://dev.epicgames.com/community/learning/tutorials/qzZ2/unreal-engine-behavior-tree-theory — Event-driven BTs with abort mechanisms
- https://opsive.com/support/documentation/behavior-designer-pro/concepts/what-is-a-behavior-tree/ — Unity Behavior Designer Pro: DOTS backend, visual debugger

**Key Insight:** Modern BTs are **event-driven** with abort mechanisms, not just tick-based. The Halo franchise pioneered this: trees can interrupt running nodes when high-priority events occur. The blackboard is the critical missing piece in most BT implementations — it's the shared working memory that decouples data producers from consumers. A typical shipped NPC has 20-50 blackboard keys; 100+ means the blackboard is a junk drawer.

**NeoTrix Mapping:** Our `nt_act::BehaviorTree` should be event-driven with abort, not pure tick-based. The blackboard pattern maps directly to our `nt_memory::Blackboard` — typed key-value store with 3 scopes: NPC-local (no sync), squad-shared (timestamps + single-writer-per-key), world-global (many readers, one writer per key). The condition-publishes-to-siblings pattern is our `PerceptionBridge`'s data channel.

---

## 4. Game Pathfinding

**URLs:**
- https://www.redblobgames.com/pathfinding/ — Amit Patel's definitive pathfinding guide: A*, flow fields, visibility graphs
- https://en.wikipedia.org/wiki/Pathfinding — A* = Dijkstra + heuristic; HPA* for hierarchical; multi-agent pathfinding (MAPF)
- https://iopscience.iop.org/article/10.1088/1757-899X/769/1/012021/pdf — Metaheuristics (GA, ACO) outperform A* in time+memory for dynamic environments

**Key Insight:** The key tradeoff is **planning vs reacting**. A* plans ahead (better paths, slower), steering behaviors react (faster, gets stuck). Production games use BOTH: A* for long-range, slow-changing paths; local steering for short-range, fast-changing obstacles. Hierarchical pathfinding (HPA*) reduces 6M-tile maps to 100-cluster abstract graphs, then refines within clusters.

**NeoTrix Mapping:** Our `nt_act::Pathfinding` should implement layered pathfinding: HPA* for global routes (map-level clusters), local A* for cluster-internal paths, and steering behaviors for micro-navigation. The flow field technique (gradient of distance field) is perfect for tower defense / MOBA lane routing — precompute once, reuse for all units. Store flow fields in `nt_memory::SpatialIndex`.

---

## 5. Game Memory

**URLs:**
- https://www.socratopia.app/library/game-ai-patterns-en/chapter-9 — Blackboard Pattern: typed key-value store, 3 scopes, condition-publishes-to-siblings
- https://tonogameconsultants.com/ai-blackboard/ — Tactical blackboard: squad coordination without central commander
- https://www.qrg.northwestern.edu/Resources/aigames.org/2001/mcavazza01.pdf — Multi-agent blackboard for RTS: knowledge sources + time-sliced working space
- https://doc.photonengine.com/quantum/v3/addons/bot-sdk/blackboard — Quantum 3 Bot SDK: union-typed blackboard, 8-byte slots

**Key Insight:** The blackboard solves the **BT wiring problem**: BTs compose by control flow, not data flow. Without blackboards, nodes re-derive context every tick. Three scope levels exist: NPC-local (one per entity, no sync), squad-shared (multiple writers, needs timestamps/single-writer), world-global (many readers, singleton writers). The original blackboard concept (1980s expert systems) used knowledge sources that fire when pattern-matched — modern game BTs simplify this to direct read/write.

**NeoTrix Mapping:** Our `nt_memory::Blackboard` becomes the central integration point. Implement 3 scopes matching the pattern. The `PerceptionBridge` writes perception data; `GoalPlanner` reads and writes intent; `BehaviorTree` leaf nodes read conditions and write results. The time-sliced working space from the Northwestern paper maps to our `nt_world::EventBus` — events propagate at different rates (visual instant, audio delayed by distance).

---

## 6. Game Evolution

**URLs:**
- https://keiwando.com/evolution/ — Sandbox: neural network + genetic algorithm, creatures evolve to run/climb/fly
- https://www.cse.unr.edu/~bdbryant/papers/stanley-2006-aaai.pdf — rtNEAT: real-time neuroevolution, complexify networks during gameplay (NERO game)
- https://github.com/lancejepsen/Snake-AI-Neuroevolution-Python — Live-learning snake: ray sensors + memory + anti-circle fitness

**Key Insight:** rtNEAT's key innovation is **complexification during gameplay**: start with minimal neural networks (no hidden nodes), add nodes/connections as behavior becomes more complex. This avoids the cold-start problem of large networks. The NERO game proved players can train agents in real-time without understanding ML — the player designs training exercises, evolution does the learning. The snake AI shows that ray-based perception + short-term memory + curriculum learning (gradually increasing complexity) is sufficient for emergent navigation.

**NeoTrix Mapping:** Our `nt_mind::EvolutionEngine` should use rtNEAT-style complexification: start agents with minimal policy networks, grow complexity as needed. The `nt_core_self::DifficultyController` adapts training curriculum. The ray-perception pattern maps to our `nt_sense::RayCaster` component. The snake's anti-circle fitness shaping maps to our `nt_mind::FitnessShaper` — prevent degenerate strategies via penalty terms.

---

## 7. Game Self-Play

**URLs:**
- https://arxiv.org/abs/2408.01072 — Comprehensive survey: 4 families (traditional, PSRO, ongoing-training, regret-minimization)
- https://arxiv.org/html/2601.03306 — QZero: model-free off-policy RL, self-play Go without human data, 7 GPUs ≈ AlphaGo
- https://discovery.ucl.ac.uk/id/eprint/10045895/1/agz_unformatted_nature.pdf — AlphaGo Zero: pure self-play, no human knowledge, 100-0 against AlphaGo Lee

**Key Insight:** Self-play converges to Nash equilibrium when the population is larger than the largest strategy层级. The key challenge is **catastrophic forgetting** — agents forget strategies that beat previous opponents. Solutions: PSRO (Policy-Space Response Oracle) maintains a population of diverse strategies; ongoing-training uses Polyak averaging for stability. AlphaGo Zero's "ignition mechanism" is critical: warm up Q-learning with episode returns before standard bootstrapping, or training collapses.

**NeoTrix Mapping:** Our `nt_mind::SelfPlayManager` should maintain a population of strategy checkpoints (not just the latest). Use PSRO-style meta-game solving: maintain payoff matrix between all historical strategies, solve for Nash distribution, sample opponents from that distribution. The ignition mechanism maps to our `nt_core::WarmupPhase` — agents start with supervised human data before pure self-play.

---

## 8. Game Team Strategy

**URLs:**
- https://arxiv.org/html/2506.10326v3 — VGC-Bench: Pokémon VGC, 10^139 team configs, self-play + fictitious play + double oracle
- https://arxiv.org/html/2410.13769v2 — BERTeam: transformer for team selection, coevolutionary deep RL
- https://github.com/deepeshahlawat/Cloud9_Hack — DraftGap AI: multi-agent (Strategist + LSTM + Reasoning + Critic) for LoL draft

**Key Insight:** Team strategy has two coupled challenges: **team building** (combinatorial) and **team usage** (sequential decision). VGC-Bench shows that as team diversity increases, single-team-trained agents degrade severely — the generalization frontier is the real challenge. BERTeam treats team selection as sequence generation (masked → predict next agent), learning team composition distributions. DraftGap's 4-agent pipeline (Strategist→LSTM→Reasoning→Critic) proves that multi-agent committee reasoning outperforms single-model approaches for strategic decisions.

**NeoTrix Mapping:** Our `nt_core::TeamStrategy` should separate team building (combinatorial optimization via PSRO) from team usage (sequential policy via self-play). The BERTeam transformer maps to our `nt_mind::TeamSelector` — predict optimal team composition given opponent profile. The DraftGap 4-agent committee maps to our `nt_meta::MultiAgentConsensus` — Strategist (high-level), LSTM (statistical baseline), Reasoner (symbolic KB integration), Critic (constraint validation).

---

## 9. Game Emergence

**URLs:**
- https://arxiv.org/html/2404.17027v1 — LLM-driven emergence: players discover narrative paths not in designer's graph
- https://www.gamedeveloper.com/programming/ai-design-for-emergent-behaviour — Essential properties system: Financial/Safety/Food/Sex/Excitement/Sleep/Respect decay + event reactions
- https://psichix.github.io/emergent/ — "Emergent AI" book: behavior-driven approach with stimulus reactions
- https://dfairesearch.com/rrd-genesis/ — RRD-PHS: 52-neuron brain discovers fire, deception, betrayal, grief through Hebbian plasticity
- https://en.wikipedia.org/wiki/Emergent_gameplay — Emergent gameplay: complex behavior from simple rules (Deus Ex, Dwarf Fortress, Left 4 Dead)

**Key Insight:** The most profound finding is from RRD-PHS: a 52-neuron network with Oja plasticity independently discovered fire, social deception, and grief — without any behavioral code. The key is **essential properties** that decay over time (hunger, sleep, bravery) and interact chaotically (too much excitement → reduced safety). Events propagate at different speeds (visual instant, audio delayed by distance/surfaces). The Left 4 Dead "Director" proves that adaptive difficulty creates emergent narrative — the AI analyzes player performance and adjusts pacing dynamically.

**NeoTrix Mapping:** Our `nt_feel::EmotionEngine` implements the essential properties pattern: decay rates, cross-property interactions, event-driven updates. The `nt_world::EventBus` implements speed-of-sound propagation for audio events. The "Director" pattern maps to our `nt_meta::PacingController` — monitors player state, adjusts encounter frequency/difficulty. The RRD-PHS Hebbian learning maps to our `nt_mind::SynapticPlasticity` — agents learn associations from experience, not from rules.

---

## 10. Game Consciousness

**URLs:**
- https://github.com/ResInferrer/Game-Agnostic-Cognitive-Player — GACP: LLM agent that learns games from scratch via observation → world model → experimentation
- https://doi.org/10.1109/cig.2009.5286473 — Consciousness-based cognitive architecture for FPS game characters
- https://arxiv.org/abs/2608.21439 — WorldMind: decoupled game world model for state-aware NPC behavior
- https://theconsciousness.ai/posts/ai-video-games-2026-agent-driven-npc-consciousness/ — 2026 industry: "memory-first AI" — persistence + adaptation, not consciousness
- https://github.com/darianrosebrook/conscious-bot-experiment — Minecraft bot: embodied sensory feedback + hierarchical planning + long-term memory + LLM "inner voice"

**Key Insight:** The 2026 industry consensus is **functional realism** — not claiming consciousness, but achieving behavioral indistinguishability. NPCs remember player actions across sessions, adapt dialogue, pursue independent goals. The GACP architecture is the gold standard: Perception → Memory → World Model → Decision → Action → Introspection, all on abstract JSON. The Minecraft bot's dual-system (Sterling symbolic planner + LLM "inner voice") proves that integrative architecture (not scale) yields situated intelligence. Consciousness research says: current game AI lacks global workspace (GWT), integrated information (IIT), and higher-order self-representation.

**NeoTrix Mapping:** Our architecture IS the GACP pattern: `nt_sense` (Perception) → `nt_memory` (Memory) → `nt_core` (World Model) → `nt_act` (Decision) → `nt_io` (Action). The GWT attention routing IS the global workspace. The `nt_feel::EmotionEngine` IS the affective state that modulates cognition. The "inner voice" maps to our `nt_meta::SelfReflection` — LLM-powered introspection on memory streams. We are building what the consciousness paper says is missing: genuine integration architecture, not just modular databases sharing data.

---

## 11. Open Source MOBA

**URLs:**
- https://github.com/xgend/MOBA_CSharp_Unity — Server-side MOBA framework, 3000 lines, navmesh pathfinding, shared sight, bush mechanics
- https://github.com/tammukul/UNION-OpenSource-MOBA — Unity MOBA with PlayFab + Photon, real-time sync + RPC
- https://github.com/The-JDdev/Heroes-Arena — 5v5 MOBA in Kotlin, 7 heroes, 6 roles, MVVM architecture
- https://github.com/DaughterOfZaun/League-of-Jinx — LoL clone in Godot, WIP, 41 stars
- https://github.com/OpenChamp/_original_client — Open-source LoL competitor post-Vanguard, Godot 4.3

**Key Insight:** The `MOBA_CSharp_Unity` project reveals the minimal viable MOBA: 3000 lines covers server, pathfinding, sight sharing, bush mechanics, and scriptable heroes/skills/buffs/items. Key architectural decisions: server is a standalone console app (runs on Linux via Mono), client is Unity. Communication via ENet-CSharp (UDP), serialization via MessagePack. The Heroes-Arena project proves that a complete MOBA can be built with MVVM + Canvas rendering — no need for heavy 3D engine.

**NeoTrix Mapping:** Our `neotrix-sim` should follow the `MOBA_CSharp_Unity` minimal architecture: headless Bevy server (no rendering), lightweight client. Use RON for hero/skill/buff/item data (replacing CSV/JSON). The sight-sharing system maps to our `nt_world::FogOfWar` component. Bush mechanics map to `nt_shield::StealthZone`. The 3000-line benchmark proves our architecture can be production-viable without bloat.

---

## 12. Bevy 2D

**URLs:**
- https://bevy.org/ — Official: ECS is core, 2D renderer with sprite sheets, dynamic texture atlases, cameras
- https://dev.to/trish_07/create-a-2d-pong-game-with-rust-and-bevy-a-step-by-step-guide-1dbk — Pong tutorial: Bevy + Bevy Rapier for physics
- https://github.com/LiTianchu/bevy-infinite-world-2d-starter — Infinite 2D grid world, procedural generation
- https://github.com/TheBevyFlock/bevy-template — Official 2D template: asset tracking, audio, menus, screens, theme
- https://dev.to/sbelzile/rust-platformer-part-1-bevy-and-ecs-2pci — Platformer: ECS = Entities (IDs) + Components (data) + Systems (logic)

**Key Insight:** Bevy's ECS is the cleanest Rust game ECS: Components are plain Rust structs, Systems are plain Rust functions. No macros needed for basic usage. The `bevy-rapier2d` integration proves physics works seamlessly. The official 2D template reveals production patterns: `asset_tracking.rs` for batch asset loading, `theme/` for reusable UI widgets, `menus/` for state-specific UI. Hot-patching (experimental) allows editing code while running — revolutionary for iteration speed.

**NeoTrix Mapping:** Our `neotrix-sim/src/` follows Bevy ECS exactly: `components/` = plain structs, `systems/` = plain functions. Adopt the template's `asset_tracking.rs` pattern for loading hero sprites, skill effects, map tiles. The `theme/` pattern maps to our `nt_io::UIWidgets`. Hot-patching is critical for our evolution loop — agents can modify system logic without recompilation.

---

## 13. Game Utility AI

**URLs:**
- https://www.socratopia.app/library/game-ai-patterns-en/chapter-9 — (Same as #5, but utility-specific: utility scores + blackboard = emergent squad coordination)
- https://tonogameconsultants.com/ai-blackboard/ — Utility systems score tasks by suitability; blackboard blocks claimed tasks

**Key Insight:** Utility AI + Blackboard = emergent squad coordination without central commander. Each agent scores all possible tasks (breach, cover, flank, secure) based on urgency + personal capability. Once an agent claims a task, it's posted to the blackboard as blocked. Other agents see it's taken and select alternatives. This produces believable coordination that scales from 5 to 50 agents without a commander AI. The key advantage over BTs: utility systems handle continuous decision-making with multiple competing goals naturally.

**NeoTrix Mapping:** Our `nt_act::UtilityAI` evaluates actions by: distance to target, health status, ammo, role capability, squad needs. The `nt_memory::Blackboard` stores claimed tasks with timestamps. The agent's utility function = Σ (weight_i × value_i) where weights are persona-derived (bravery, aggression, caution). This replaces the rigid BT priority system with fluid, context-dependent action selection.

---

## 14. Game GOAP

**URLs:**
- https://github.com/luxkun/ReGoap — C# GOAP library: world state facts, actions with preconditions/effects/cost, A* planner
- https://github.com/crashkonijn/GOAP — Multi-threaded Unity GOAP, ScriptableObject config, visual debugger
- https://github.com/imaklee/GdPlanningAI — Godot GOAP: backward-chaining A*, SpatialAction (goto+interact fused), 4 planning modes
- https://github.com/pixelrogueart/goap-godot-4 — Pure GDScript GOAP: backward search, editor debugger, planner explorer

**Key Insight:** GOAP's killer feature is **runtime replanning**: if world state changes mid-plan, agent re-evaluates and generates a new plan. The `GdPlanningAI` project reveals a critical optimization: `SpatialAction` fuses `goto` + `object interaction` into one action. Without this, the planner explodes because it must simulate movement to every possible location during planning. The 4 planning modes (CONTINUOUS, ON_INTERVAL, ON_DEMAND, ON_INTERVAL_FORCED) let developers balance responsiveness vs performance. The `ReGoap` library's weighted-random goal selection adds naturalism — agents don't always pick the optimal goal.

**NeoTrix Mapping:** Our `nt_core::GoalPlanner` uses GOAP-style backward-chaining A* from desired state → action chain. The `SpatialAction` optimization maps to our `nt_act::MoveAndInteract` — fuse navigation + action into one planning step. The 4 planning modes map to our `PlanningMode` enum. The weighted-random goal selection adds behavioral variety — agents occasionally pursue suboptimal goals for naturalness.

---

## 15. Game Faction

**URLs:**
- https://gamesbyhyper.com/product/reputation-manager/ — UE5: faction + individual NPC reputation, tiers, decay, rewards
- https://adrenalinegames.pl/reputationfactionsystem — UE5: 7 tiers (-100 to +100), 6 relationship types, wars/alliances
- https://docs.rs/mercs2_faction/latest/mercs2_faction/ — Rust crate: faction reputation + pursuit ("heat") + mood accumulator
- https://gamesbyhyper.com/docs/progression-and-leveling/reputation-system/ — Data-table driven: actions → reputation changes → stage transitions → rewards
- https://obipandawan.itch.io/rmmz-reputation-plugin — RMMZ: 6 tiers, rival/alliance ripple, kill-reputation auto-tagging

**Key Insight:** The `mercs2_faction` Rust crate is the most relevant: it implements faction reputation as a **mood accumulator** (7 infraction keys weighted into a single mood), which feeds into relation [-100,100] → attitude level → price multiplier + HUD color + pursuit escalation. The key innovation: infractions never passively decay — they're consumed by a `report` tick that folds accumulated infractions into the relation. The carve rule (leaf crates never depend on each other) is exactly our domain isolation principle. The 7-tier system (Hated→Hostile→Unfriendly→Neutral→Friendly→Honored→Exalted) is the industry standard.

**NeoTrix Mapping:** Our `nt_memory::FactionReputation` implements the `mercs2_faction` pattern: per-faction infraction accumulators, report tick that folds into relation, attitude-level crossing events. The carve rule maps to our domain isolation — `nt_memory` emits `RelationChange` intents, `nt_act` reads them. The 7-tier system maps to our `ReputationTier` enum. Rival/alliance ripple maps to our `nt_memory::FactionRelation` graph — changing one faction's reputation propagates to linked factions.

---

## Cross-Cutting Patterns

| Pattern | Sources | NeoTrix Component |
|---------|---------|-------------------|
| **Blackboard = shared working memory** | BT (#3), Memory (#5), Utility (#13) | `nt_memory::Blackboard` (3 scopes) |
| **Event-driven > tick-based** | BT (#3), Consciousness (#10) | `nt_world::EventBus` (speed-of-sound propagation) |
| **Dual planning + reacting** | Pathfinding (#4), GOAP (#14) | `nt_core::GoalPlanner` (GOAP) + `nt_act::Steering` (reactive) |
| **Essential properties + decay** | Emergence (#9), Faction (#15) | `nt_feel::EmotionEngine` (decay rates + cross-interactions) |
| **Self-play with population** | Self-Play (#7), Team (#8) | `nt_mind::SelfPlayManager` (PSRO-style population) |
| **Functional realism > consciousness** | Consciousness (#10) | Architecture IS the integration (GWT + emotion + memory) |
| **Data-driven > hardcoded** | MOBA (#2), Faction (#15) | RON configs for heroes/items/factions, not Lua/scripts |
| **Hierarchical pathfinding** | Pathfinding (#4), Bevy 2D (#12) | HPA* global + local A* + steering |
| **SpatialAction fusion** | GOAP (#14) | `nt_act::MoveAndInteract` (goto + interact = 1 action) |
| **Mood accumulator** | Faction (#15), Emergence (#9) | `nt_memory::FactionReputation` (infraction → report → relation) |
