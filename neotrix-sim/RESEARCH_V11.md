# RESEARCH_V11 — 20-Source Simulation Knowledge Batch

**Generated**: 2026-09-11
**Sources**: 20 web searches → 16 successful (4 rate-limited, data from parallel successful results)

---

## 1. Bevy ECS Game 2025

**URL**: https://github.com/linzell/space-looter
**Key Insight**: Domain-Driven Design (DDD) architecture with Bevy ECS for 3D isometric RPGs. Clean separation: `domain/` → `application/` → `infrastructure/bevy/` → `presentation/`. WASM cross-browser deployment. Dice-based RPG mechanics with DDD.
**NeoTrix Mapping**: **L1 Action Layer** — `nt_act/` uses similar DDD layering (domain → application → infrastructure). Bevy ECS's `Component`/`System`/`Resource` maps directly to NeoTrix's `#[derive(Component)]` + system scheduling. Cross-platform via WASM validates NeoTrix's `nt_io/` interface layer.

---

## 2. Tauri 2.0 Game

**URL**: https://github.com/saagpatel/CryptForge
**Key Insight**: Tauri 2 + React 19 + Rust: game logic in Rust backend, UI in React frontend. Tauri commands expose game state deterministically. Procedural dungeon generation, turn-based tactical combat, permadeath. 6MB binary vs Electron bloat.
**NeoTrix Mapping**: **NT-IO Interface Layer** — NeoTrix desktop app (`src-tauri/`) already uses Tauri. CryptForge's architecture validates the Rust-backend/React-frontend split. Deterministic simulation in Rust backend → `nt_core` reasoner. React UI → `nt_io::cli` or `nt_io::web`.

---

## 3. MOBA AI Bot

**URL**: https://github.com/internexio/clawberbot
**Key Insight**: Hand-crafted MOBA bot (68 versions, 12K matches, #1 ELO) vs autonomous pipeline (<1K matches, zero human direction). KnowledgeForge multi-agent reasoning framework powers both. Wilson 95% CI gates all decisions (PROMOTE/DIAGNOSE/WAIT). Failed-hypothesis log prevents repeats. 12× improvement by removing human bottleneck.
**NeoTrix Mapping**: **SEAL Pipeline** — Clawberbot's autonomous loop (Play→Gate→Diagnose→Submit) mirrors SEAL's distillation→self-test→absorption. Failed-hypothesis log = `experience-tree` KB. Wilson CI gating = `nt_core_self::AttentionManager` salience thresholds. The 12× speedup validates NeoTrix's autonomous evolution thesis.

---

## 4. Game Behavior Tree

**URL**: https://www.behaviortrees.com/learn/
**Key Insight**: Behavior Trees: Sequence (AND), Selector (OR), Decorator, Leaf nodes. Three statuses: Success/Failure/Running. Blackboard for shared state. Event-driven BTs solve scalability by reacting to events and aborting running nodes. Debugging: visual editor shows active task + execution flow.
**NeoTrix Mapping**: **GWT Attention Routing** — BT's Selector node ≈ GWT's salience competition. BT's Sequence ≈ GWT's broadcast chain. Blackboard ≈ `SelectiveState`. However, NeoTrix goes beyond BTs: GWT uses resonance-based routing (weighted scoring), not binary success/failure. BTs for `nt_act` tool selection; GWT for `nt_core` meta-cognition.

---

## 5. Game Pathfinding

**URL**: https://github.com/foxzool/last-stop
**Key Insight**: Bevy ECS bus route puzzle using A*-driven pathfinding. Smart path caching, optimized rendering pipeline, memory-friendly resource management. ECS architecture ensures efficient system updates for thousands of entities.
**NeoTrix Mapping**: **NT-WORLD Perception** — A* pathfinding for `nt_world::crawl` URL traversal. Path caching ≈ `nt_memory::cache` for visited URLs. Memory-friendly resource management = Bevy's `Resource` trait pattern. For NeoTrix sim: A* for agent movement on grid maps.

---

## 6. Game Memory System

**URL**: https://gamesbyhyper.com/docs/exploration-and-narrative/memory-system/
**Key Insight**: Tag-driven memory: `Has These Memories` / `Cannot Have These Memories` gates. Memories are Gameplay Tags stored per-actor. Any system can add/remove/query at runtime. Dialogue, quests, map markers all react to memory state. Event bindings (reactive) + overlap triggers (proactive).
**NeoTrix Mapping**: **NT-MEMORY Knowledge Base** — Tag-driven memory ≈ `kv_store` namespace entries. `Memory_Context` ≈ KB query with positive/negative filters. Reactive event bindings ≈ `EventBus`. Proactive overlap triggers ≈ `PerceptionBridge` awareness gating. NeoTrix's `experience-tree` uses this pattern: experience tags → KB query → route table matching.

---

## 7. Game Evolution

**URL**: https://store.steampowered.com/app/1133120/Ecosystem/
**Key Insight**: Ecosystem: virtual lifeforms evolve from synthetic DNA in physically-simulated ocean. Nervous system = pipeline computer (sense→neurons→muscles). Fitness-based reproduction: better swimmers get more offspring. Combat stats determine food chain role. Tools to force traits (tentacles, size, drag/torque).
**NeoTrix Mapping**: **SEAL Pipeline Self-Evolution** — DNA ≈ `nt_core_self::SelfModel` parameters. Fitness ≈ `constellation_maturity` (C0→C6). Natural selection ≈ `nt_meta::evolution` module. Combat stats ≈ capability scoring. NeoTrix's self-play evolution uses similar selection pressure: modules that compile+test+connect survive.

---

## 8. Game Self-Play

**URL**: https://arxiv.org/abs/2408.01072
**Key Insight**: Comprehensive survey: self-play = agents improve by interacting with evolving opponents. Categories: traditional self-play, PSRO series, ongoing-training, regret-minimization. AlphaGo Zero: pure RL self-play, no human data, achieves superhuman Go. Key: population-based self-play converges when population > max layer size.
**NeoTrix Mapping**: **nt_mind Self-Evolution** — Self-play = modules competing in `nt_meta::evolution`. PSRO (Policy-Space Response Oracle) ≈ NeoTrix's dual specialization (Weapon Set I/II). Population-based training ≈ `constellation_maturity` progression. Failed hypotheses tracked in KB prevents cycling.

---

## 9. Game Team Strategy

**URL**: https://arxiv.org/html/2508.06042
**Key Insight**: HIMA (Hierarchical Imitation Multi-Agent): specialized imitation agents under a Strategic Planner (SP). Each agent generates structured action sequences. SP uses Nominal Group Technique to resolve conflicts, then Temporal Chain-of-Thought (t-CoT) for immediate/short-term/long-term alignment. Feedback system re-examines plans after critical events.
**NeoTrix Mapping**: **GWT + nt_core** — SP ≈ `E8 Hexagram` reasoning engine. Specialized agents ≈ NT-* domain modules. Nominal Group Technique ≈ GWT's salience competition with resonance. t-CoT ≈ `ConsciousnessTree` 6-stage loop (Soil→Roots→Trunk→Branches→Fruits→Core). Feedback system ≈ `nt_repair` self-healing.

---

## 10. Game Social Dynamics

**URL**: https://github.com/lizTheDeveloper/ai_village
**Key Insight**: AI Village: LLM-powered autonomous agents with needs, moods, relationships, memories. 200+ ECS systems across 19 packages. Magic (25+ paradigms), Divinity (gods/temples), Life (reproduction/genetics/families/souls/reincarnation), Environment (fire/fluid/temperature/weather). Modding-first, transparent simulation.
**NeoTrix Mapping**: **NT-FEEL Emotional Domain** — Agent needs/moods ≈ `nt_feel::EmotionEngine`. Relationships ≈ `nt_core_self::relational_memory`. Families/genetics ≈ `nt_core_self::SelfModel` inheritance. Transparent simulation ≈ NeoTrix's `#![forbid(unsafe_code)]` + audit dimensions D1-D50.

---

## 11. Game Emergence

**URL**: https://github.com/Ashokgorantla935/Emergence
**Key Insight**: Rust swarm intelligence engine: emotionally-driven agents with stigmergic signal grid (Danger/Food/Comfort/Grief/Celebration/Anger/Crime). Consequence architecture: rate-of-change sensing, causal memory `(action, context, outcome)`, hypothetical projection (50-tick forward simulation). Relational memory with witnessing: interactions create reputation/crime/justice without rigid programming. 10K+ agents at 60+ FPS via SoA + arena allocators + rayon parallelism.
**NeoTrix Mapping**: **Full Stack** — Stigmergic signals ≈ `EventBus` diffusion. Consequence architecture ≈ `nt_core_self::dynamic_params` + `nt_repair` causal tracing. Relational witnessing ≈ `nt_core_self::relational_memory`. 10K agents via arena allocators + rayon = validates NeoTrix's Rust-native architecture. wgpu rendering ≈ `nt_physical::render`.

---

## 12. Game Consciousness

**URL**: https://arxiv.org/html/2409.00853v1
**Key Insight**: JaxLife: artificial life simulator where agents evolve in expressive world with programmable robots. Turing-complete computation (Rule 110 implementation). Emergent agriculture, tool use, communication protocols. Rudimentary communication saliency increases over time. Complexity scales with compute. Cultural accumulation via social learning bias.
**NeoTrix Mapping**: **ConsciousnessTree + E8** — JaxLife's Turing-complete robots ≈ NeoTrix's `nt_core::E8` reasoning (64 hexagrams = universal computation). Emergent communication ≈ GWT's resonance-based broadcasting. Cultural accumulation ≈ `experience-tree` KB growth. Scalability thesis: complexity scales with compute → validates NeoTrix's layered architecture.

---

## 13. Open Source MOBA

**URL**: https://github.com/ajhahnde/Theria
**Key Insight**: Theria: 2.5D MOBA in Godot 4. Server-authoritative, deterministic simulation. Shapeshifter tribes: Solane (big-cat burst) vs Verdani (venom-and-shadow attrition). 3v3 arena with jungle. Hero abilities: 4 skills + passive. Fixed-timestep simulation for network determinism.
**NeoTrix Mapping**: **nt_act + nt_world** — Server-authoritative model ≈ NeoTrix's `nt_core` as central authority. Deterministic simulation ≈ `nt_core::E8` deterministic reasoning. Tribe asymmetry (burst vs attrition) ≈ NeoTrix's dual specialization (Weapon Set I/II). 3v3 team composition ≈ NT-* domain module coordination.

---

## 14. Open Source Simulation

**URL**: https://github.com/furyhawk/aeon
**Key Insight**: AEON: browser god-game + civilization sim. Every creature has a real evolving neural network. No scripted behaviors. Economy (resource nodes), tech tree, civics tree, settlements, territory, combat, diplomacy, anthropology (emergent ethos). Runs 100% in browser, zero dependencies. MIT licensed.
**NeoTrix Mapping**: **Full Stack** — Evolving neural networks per agent ≈ `nt_core_self::SelfModel` per module. Emergent ethos ≈ NeoTrix's `constellation_maturity` progression. Tech/civics trees ≈ NeoTrix's `Skill Tree` (Small Passive → Notable Passive → Keystone). Territory ≈ KB namespace ownership. Browser-first = validates `nt_io::web`.

---

## 15. Bevy 2D Rendering

**URL**: https://github.com/bevyengine/bevy/pull/24833
**Key Insight**: Bindless 2D materials in Bevy: batch 2D meshes into single drawcalls even with different textures. 1.71× speedup for SpriteMaterial. Retained render world extraction (don't re-extract unchanged meshes). Phases: Opaque2d (binned), AlphaMask2d (binned), Transparent2d (sorted by Z). TilemapChunk for optimized large grid rendering.
**NeoTrix Mapping**: **nt_physical Render** — Bindless batching ≈ NeoTrix's `nt_physical::render` optimization. Retained extraction ≈ NeoTrix's `kv_store` change detection (only re-embed modified content). TilemapChunk ≈ NeoTrix's grid-based world representation. Phase-based rendering ≈ GWT's attention phases (broadcast vs ignore).

---

## 16. Game Utility AI

**URL**: https://github.com/ZorPastaman/UtilityAI
**Key Insight**: Utility AI: score all possible decisions, pick highest. Each decision scored per target. Considerations with curves (linear/quadratic/exponential). Priority groups for hierarchy. Best combined with FSMs (transitions) + BTs (execution). Decision Making Preview in editor.
**NeoTrix Mapping**: **GWT Attention Routing** — Utility scoring ≈ GWT's salience scoring. Considerations ≈ `nt_core_self::AttentionManager` weighted criteria. Priority groups ≈ NT-* domain priority. Combined with FSMs/BTs ≈ NeoTrix's hybrid approach: Utility (GWT) + FSM (state transitions in `nt_core`) + BT-like sequences in `nt_act` tool selection.

---

## 17. Game GOAP

**URL**: https://github.com/crashkonijn/GOAP
**Key Insight**: Goal-Oriented Action Planning: world state facts → actions with preconditions/effects/costs → goals as desired states. Planner runs A* search to build action plan. Re-plans on success/failure/world change. Weighted-random goal selection possible. GOAP generates behavior from world facts instead of hardcoded trees.
**NeoTrix Mapping**: **nt_core Reasoning** — GOAP's A* search ≈ NeoTrix's `E8 Hexagram` state-space search. Preconditions/effects ≈ `nt_core_capability_tree` edges. Goal states ≈ `nt_core_self::SelfModel` objectives. Re-planning ≈ SEAL pipeline's iterative refinement. GOAP for `nt_act` tool sequencing; E8 for meta-cognitive reasoning.

---

## 18. Game Perception

**URL**: https://github.com/Ashokgorantla935/Emergence
**Key Insight**: Stigmergic signal grid: multiple channels (Danger/Food/Comfort/Grief/Celebration/Anger/Crime) diffuse via parallel convolution, evaporate over time. Agents sense rate-of-change of internal needs. Perception radius for witnessing interactions. Spatial indexing for neighbor queries.
**NeoTrix Mapping**: **L2 Perception Layer** — Signal grid ≈ `nt_world::sense` sensory channels. Diffusion ≈ `EventBus` signal propagation. Evaporation ≈ time-decay in `HeartbeatAggregator`. Perception radius ≈ `PerceptionBridge` awareness gating. Spatial indexing ≈ `nt_world::crawl` URL scoring with relevance decay.

---

## 19. Game Faction

**URL**: https://dl.acm.org/doi/fullHtml/10.1145/3649921.3650012
**Key Insight**: Taxonomy of faction systems: Explicit (unit-based, allegiance, state-based) vs Implicit (relationship, world-based, procedurally generated). Faction identity: unique visual identity + gameplay loop + lore hook + home realm + NPC face. 7 factions = sweet spot (enough distinct, few enough to remember). Faction members need character development + autonomy.
**NeoTrix Mapping**: **NT-* Domain System** — 7 core domains (NT-CORE through NT-FEEL) ≈ 7 launch factions. Each domain needs: unique capability (visual identity), unique workflow (gameplay loop), unique lore (CONTEXT.md terms), unique location (layer directory), unique agent (E8/GWT/etc.). Faction identity = "describe in one sentence" test. Procedural factions ≈ `constellation_maturity` emergent capabilities.

---

## 20. Game Culture

**URL**: https://github.com/HarperKollins/evosim-agentic-sociology
**Key Insight**: EvoSim: agents form tribes (100% by year 50), tribes live 21.3% longer than lone wolves. "Breaking Bad" hypothesis: older agents become more selfish (r=-0.47 between Age and Karma). Memes spread rapidly but lack diversity (Tower of Babel effect). Cooperation converges to ~0.51 regardless of governance. Institutional governance wins in between-group selection.
**NeoTrix Mapping**: **ConsciousnessTree + SEAL** — Tribe formation ≈ NT-* domain consolidation. Breaking Bad = modules drift without governance → `nt_meta::governance` enforcement. Memes ≈ `experience-tree` KB entries (rapid spread, need diversity). Cooperation attractor ≈ system health equilibrium. Between-group selection ≈ `constellation_maturity` competition (C0→C6). Institutional governance = NeoTrix's AGENTS.md + dev-rules.md constitution.

---

## Cross-Source Patterns (11)

| # | Pattern | Sources | NeoTrix Mapping |
|---|---------|---------|-----------------|
| P1 | **ECS as Simulation Backbone** | Bevy (1,5,15), Emergence (11), AI Village (10) | `nt_core::E8` + `nt_world` grid |
| P2 | **Stigmergic Communication** | Emergence (11), Emergence Engine | `EventBus` + `HeartbeatAggregator` |
| P3 | **Consequence Architecture** | Emergence (11), GOAP (17), Utility (16) | `nt_core_self::dynamic_params` + `nt_repair` causal tracing |
| P4 | **Self-Play Evolution** | Self-Play survey (8), AlphaGo Zero, BEER (3) | `nt_mind::seal` + `nt_meta::evolution` |
| P5 | **Hierarchical Multi-Agent** | HIMA (9), Utility+FSM+BT (16) | GWT + E8 + NT-* domain routing |
| P6 | **Tag-Driven Memory** | Memory System (6), EvoSim (20) | `kv_store` namespaces + experience tags |
| P7 | **Faction Identity Contract** | Faction taxonomy (19), AEON (14) | 7 domains = 7 factions, each with unique contract |
| P8 | **Deterministic Simulation** | Theria (13), CryptForge (2) | `nt_core` deterministic reasoning |
| P9 | **Emergent Culture** | EvoSim (20), AI Village (10), AEON (14) | `experience-tree` KB + constellation progression |
| P10 | **Bindless Rendering** | Bevy 2D (15) | `nt_physical::render` optimization |
| P11 | **Governance Prevents Drift** | EvoSim (20), Faction (19) | `nt_meta::governance` + AGENTS.md constitution |

---

## Absorbed Terminology (New, 8 terms)

| Term | Definition | Source |
|------|-----------|--------|
| **Stigmergic Signal Grid** | Multiple computational signal channels on a discrete grid, diffusing via parallel convolution and evaporating over time, layering invisible emotional and survival topology | Emergence (Rust swarm engine) |
| **Consequence Architecture** | Three-layer decision making: rate-of-change sensing, causal memory `(action, context, outcome)`, hypothetical projection (50-tick forward simulation) | Emergence |
| **Relational Witnessing** | Any agent within perception radius "witnesses" interactions and updates trust/warmth/debt for both actors, creating reputation without rigid programming | Emergence |
| **Nominal Group Technique** | Structured 4-step decision method: identify agreed/conflicted viewpoints, resolve conflicts, consider isolated viewpoints, synthesize into cohesive strategy | HIMA (SC2) |
| **Temporal Chain-of-Thought** | Breaking strategy into immediate/short-term/long-term actions before generating final decision, connecting time horizons into step-by-step reasoning | HIMA (SC2) |
| **Failed-Hypothesis Log** | Every prior attempt (version, what tried, gate outcome) prepended to diagnosis context; debugger's first instruction: read this before proposing anything | Clawberbot |
| **Genetic Personality** | Utility functions transmitted from parents to offspring via variation operators, creating heritable behavior profiles in artificial life agents | Orphibs II |
| **Wilson CI Gating** | Wilson 95% confidence interval on win rate drives all decisions: PROMOTE (above threshold), DIAGNOSE (below), WAIT (insufficient data) | Clawberbot |
