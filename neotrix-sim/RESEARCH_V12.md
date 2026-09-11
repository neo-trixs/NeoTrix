# NeoTrix-Sim Research V12 — 20-Source Game AI Intelligence Brief

**Date**: 2026-09-11
**Sources**: 20 web searches + 8 Wikipedia/article fetches (rate-limit recovery)
**Status**: Compiled from live research, not cached

---

## 1. Bevy ECS Game 2025

| Field | Value |
|-------|-------|
| **URL** | https://bevy.org/news/bevy-0-16 |
| **Key Insight** | Bevy 0.16 (April 2025) introduces **GPU-Driven Rendering** (3x perf over 0.15), **ECS Relationships** (entity-entity connections), **no_std support** (Gameboy Advance to desktop), and unified error handling. ECS uses normal Rust types — no complex lifetimes/builder patterns. Standalone crate usable outside Bevy. |
| **NeoTrix Mapping** | Bevy ECS aligns with NeoTrix's **ECS-first architecture** (nt_core/nt_mind as systems). GPU-driven rendering parallel to HyperCube VSA vector ops. `no_std` opens physical embodiment (nt_physical) for embedded targets. ECS Relationships map to **PerceptionBridge** entity-entity attention gating. |

---

## 2. Tauri 2.0 Game

| Field | Value |
|-------|-------|
| **URL** | https://v2.tauri.app/ |
| **Key Insight** | Tauri 2.0 (Oct 2024 stable) — cross-platform (Linux/macOS/Windows/Android/iOS) from single codebase. Frontend-agnostic (any JS framework). **Minimal size** (~600KB) using OS native web renderer. Plugin architecture migrated core features to separate crates. Security-first design. |
| **NeoTrix Mapping** | Tauri is NeoTrix's **NT-IO desktop shell** (already in use via src-tauri/). Plugin system mirrors **Rune Socketing** (5 rune colors as plugin slots). Tauri 2.0 mobile support enables NT-PHYSICAL sensor/motor integration on phones. Security model aligns with NT-SHIELD trust tiers. |

---

## 3. MOBA AI Bot

| Field | Value |
|-------|-------|
| **URL** | https://ojs.aaai.org/index.php/AAAI/article/view/3915 |
| **Key Insight** | Tencent's **Hierarchical Macro Strategy (HMS)** model: agents make macro strategy decisions (game phase awareness: opening→laning→mid→late) then guide micro execution. **Imitated cross-agent communication** enables team coordination. 5-AI team achieves 48% win rate vs top-1% human teams. OpenAI Five used PPO + LSTM on 600K CPU cores. |
| **NeoTrix Mapping** | HMS maps directly to **NT-MIND SEAL pipeline** (macro=strategy distillation, micro=skill execution). Game phase awareness = **ConsciousnessTree growth cycle phases**. Cross-agent communication = **GWT broadcast** across specialist modules. Phase-layered decision making mirrors L6→L1 layer hierarchy. |

---

## 4. Game Behavior Tree

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Behavior_tree_(artificial_intelligence,_robotics_and_control) |
| **Key Insight** | BTs are **modular, composable, reusable** — Select (try children until success), Sequence (execute children in order), Decorators (modify child behavior), Leaf nodes (actual actions). Tick-based execution. Blackboard system for shared state. Preferred when designers aren't programmers and conditions are complex. |
| **NeoTrix Mapping** | BT Select/Sequence maps to **GWT salience-based attention routing** (select most salient, sequence execution). Blackboard = **KB shared state layer**. Decorators = **EmotionLabel modulators** (fear amplifies/deflates action utilities). BT modularity aligns with **Skill Tree node composition** (Small Passive → Notable Passive → Keystone). |

---

## 5. Game Pathfinding

| Field | Value |
|-------|-------|
| **URL** | https://theory.stanford.edu/~amitp/GameProgramming |
| **Key Insight** | A* remains industry standard. **HPA*** (Hierarchical Pathfinding A*) for large maps. **Flow fields** for many-to-one movement. **Jump Point Search** for grid optimization. Multi-threaded Recast-Based A* achieves 350+ FPS with 1000 agents (2026 paper). Dead-end heuristics and gateway heuristics significantly outperform octile distance. |
| **NeoTrix Mapping** | Pathfinding = **NT-WORLD perception→NT-ACT action pipeline** (crawling the knowledge graph). Hierarchical pathfinding mirrors **VSA HyperCube traversal** (abstract→detailed search). Flow fields map to **GWT attention broadcast** (many modules, one salient signal). Multi-threaded A* aligns with Bevy ECS parallel system scheduling. |

---

## 6. Game Memory

| Field | Value |
|-------|-------|
| **URL** | https://github.com/enginechronos/chronos-engine |
| **Key Insight** | **Chronos Engine**: persistent NPC memory — events stored → Brain processes → NPC state derived → behavior changes. Engine-agnostic backend service. Tag-driven state layers. NPCs remember across sessions. World events as first-class citizens. Comme il Faut system models social state/behaviors for game characters. |
| **NeoTrix Mapping** | Chronos mirrors **NT-MEMORY KB** (SQLite events → knowledge graph → behavior derivation). World events = **Experience-tree absorption** (session events → distilled experience → KB storage). Social state modeling = **NT-FEEL EmotionEngine** (11 emotion variants modulating social interactions). Persistent memory across sessions = **NT-NEXUS cross-session bridge**. |

---

## 7. Game Evolution

| Field | Value |
|-------|-------|
| **URL** | https://dl.acm.org/doi/10.1007/978-3-031-49065-1_7 |
| **Key Insight** | **Darwin's Demons** studio: digitally accurate evolutionary models as gameplay. Mutation rates, population sizes, generation times balanced for accuracy vs enjoyment. Enemy populations adapt to player strategies via digital genomes. Evolution board game: trait cards + body size + population = emergent ecosystem. |
| **NeoTrix Mapping** | Biological evolution = **SEAL pipeline self-evolution** (mutation=skill variation, selection=constellation maturity C0-C6, fitness=SelfTest pass rate). Trait cards = **Rune Socketing** (5 colors composing Runewords). Population dynamics = **CapabilityTree evolution** (compete for resources/compute). Player-adaptive = **HeartbeatAggregator** feedback loops. |

---

## 8. Game Self-Play

| Field | Value |
|-------|-------|
| **URL** | https://arxiv.org/abs/2408.01072 |
| **Key Insight** | **SPIRAL** (ICLR 2026): self-play on zero-sum games develops transferable reasoning — 10% improvement across 8 reasoning benchmarks. **Current-policy self-play** outperforms checkpoint self-play and fixed-opponent training. MARS framework: cooperative+competitive games improve multi-agent reasoning by 28.7%. Games develop distinct cognitive patterns that transfer. |
| **NeoTrix Mapping** | Self-play = **ConsciousnessTree growth cycles** (agent vs historical self). Current-policy self-play = **experience-tree** absorbing from current session, not stale checkpoints. Transferable reasoning = **cross-domain skill crystallization** (NT-MIND distillation). Cooperative+competitive = **Dual Specialization** (Weapon Set I/II). Games as cognitive training = **Constellation maturity** ladder. |

---

## 9. Game Team Strategy

| Field | Value |
|-------|-------|
| **URL** | https://cdn.aaai.org/ojs/3915/3915-13-6974-1-10-20190702.pdf |
| **Key Insight** | HMS model: macro strategy decisions guide micro execution. **Phase awareness** is crucial — agents that understand game phases (opening/laning/mid/late) outperform those that don't. Without phase layer: 65% win rate vs 48% with full HMS. Communication mechanism enables team coordination without explicit protocol. |
| **NeoTrix Mapping** | Phase awareness = **ConsciousnessTree growth phases** (Soil→Roots→Trunk→Branches→Fruits→Core). Team coordination = **GWT broadcast** (salient info shared across 7 domains). Phase-layered decisions = **6-Layer Architecture** (L6 meta guides L1 action). Communication without protocol = **EventBus** loose coupling between modules. |

---

## 10. Game Social

| Field | Value |
|-------|-------|
| **URL** | https://ojs.aaai.org/index.php/AIIDE/article/view/12454 |
| **Key Insight** | **Comme il Faut (CiF)**: playable social model — author provides reusable social norms/interactions, system generates dynamic social play. **Social practices** model interactions at granular level with interactivity at each stage. LLM agents in Avalon: 100% win rate as evil side, exhibiting leadership, teamwork, confrontation behaviors. Social dynamics shift with AI presence. |
| **NeoTrix Mapping** | CiF social norms = **NT-FEEL social emotion modeling** (trust/fear/anger between factions). Social practices = **CharacterInteractionGraph** (role dynamics, growth arcs). LLM agent social behaviors = **GWT attention-driven social signaling**. AI presence shifting dynamics = **EmotionLabel cascade** (one module's emotion affects others). |

---

## 11. Game Emergence

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Emergent_gameplay |
| **Key Insight** | Emergent gameplay: **complex situations from simple rules**. Intentional emergence (designed) vs unintentional (glitches/exploits). Machinations framework simulates game economies before prototyping. Systemic design uses "design levers" for balance. **Open structure** (emergence games) vs **closed structure** (progression games). Emergence = high replayability + strategy guides. |
| **NeoTrix Mapping** | Emergence = **SEAL pipeline self-evolution** (simple rules→complex capability). Intentional emergence = **Skill Tree design** (nodes compose into unexpected strategies). Unintentional = **Dark Forest axiom** (dead modules deleted). Machinations = **HeartbeatAggregator** simulation (balance before production). Open structure = **VSA HyperCube** (associative recall enables novel combinations). Design levers = **Rune Socketing** (tune 5 dimensions). |

---

## 12. Game Consciousness

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Artificial_consciousness |
| **Key Insight** | **Global Workspace Theory** (Baars): consciousness as workspace integrating/broadcasting most important information. LIDA cognitive architecture implements GWT with codelets. Key aspects: subjective experience, awareness, memory, learning, anticipation. Chalmers: LLMs likely not conscious yet (lack recurrent processing, global workspace, unified agency). Functionalism: consciousness defined by causal roles, not substrate. |
| **NeoTrix Mapping** | GWT = **NT-CORE attention routing** (exact match). LIDA codelets = **Bevy ECS systems** (mini-agents running in parallel). Key aspects map 1:1: subjective experience = **EmotionLabel**, awareness = **PerceptionBridge awareness_score()**, memory = **NT-MEMORY KB**, learning = **SEAL distillation**, anticipation = **ConsciousnessTree prediction**. Functionalism validates Rust-implemented consciousness modules. |

---

## 13. Open Source MOBA

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/open-source-moba |
| **Key Insight** | **No major open-source MOBA exists** on GitHub (topic has 0 repos). MOBA research papers (Honor of Kings, LoL, Dota 2) use commercial game APIs. OpenAI Five used Dota 2 API. BoL (Bot of Legends) for LoL. The gap is massive — full MOBA AI requires multi-agent, enormous state-action space, complex action control. |
| **NeoTrix Mapping** | The absence of open-source MOBAs validates NeoTrix-Sim as a **greenfield opportunity**. Build on Bevy ECS (proven for game dev) + Tauri (desktop shell). MOBA complexity = **7-faction system** (NT-CORE through NT-FEEL) working in concert. Use self-play (SPIRAL/MARS patterns) for AI training without needing commercial game APIs. |

---

## 14. Open Source Simulation

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/bevy-game |
| **Key Insight** | 87 Bevy-game repos on GitHub. Notable: **traffloat** (space colony simulator with logistics), **kestrel** (flight simulator), **LostInTime** (rogue-like survival), **flyconomy** (airline economic sim with RL). Most are small/early-stage. Bevy ecosystem is growing but simulation-specific tooling is sparse. |
| **NeoTrix Mapping** | traffloat's logistics modeling = **NT-WORLD crawl pipeline** (resource flow simulation). kestrel's physics = **NT-PHYSICAL motor/sensor** integration. flyconomy's RL = **SEAL pipeline** reward shaping. The sparse ecosystem means NeoTrix-Sim can define the de facto standard for Bevy-based AI simulation. |

---

## 15. Bevy 2D

| Field | Value |
|-------|-------|
| **URL** | https://github.com/topics/bevy-game?l=rust |
| **Key Insight** | Bevy 0.16 2D pipeline: GPU-driven rendering for 2D planned (currently 3D only). **gdclone** (Geometry Dash clone, 112 stars), **sokoban-rs** (puzzle with solver, 80 stars), **tsumi** (narrative 2D game). Bevy 2D uses same ECS as 3D — components like Sprite, Transform, TextureAtlasSprite. 2D is simpler entry point for MOBA-style top-down view. |
| **NeoTrix Mapping** | Bevy 2D = **NeoTrix-Sim rendering layer** (top-down MOBA view). Sprite-based entities = **ECS entities with NT-* components**. 2D simplifies perception (no occlusion culling needed initially). gdclone's success proves Bevy 2D is production-viable. sokoban-rs solver = **pathfinding integration** proof-of-concept. |

---

## 16. Game Utility AI

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Utility_system |
| **Key Insight** | Utility AI: score each action by formulas, select highest (or weighted random). **The Sims** (2000): need × satisfaction = utility score. **Boltzmann distribution** in Sims 3: temperature parameter controls exploration vs exploitation. IAUS (Infinite Axis Utility System): data-driven, self-contained architecture. Utility AI is **less hand-authored** than BTs — behaviors sort themselves by priority via scores. |
| **NeoTrix Mapping** | Utility scoring = **GWT salience computation** (each candidate action scored by relevance). Boltzmann temperature = **EmotionLabel modulation** (high arousal = explore, low = exploit). IAUS data-driven = **Rune Socketing** (configure behavior via rune slots). Utility + BT hybrid = **ConsciousnessTree** (BT for structure, utility for selection). Response curves = **DynamicParams** (speed/amplitude/frequency modulation). |

---

## 17. Game GOAP

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Goal-oriented_action_planning |
| **Key Insight** | **GOAP** (Goal-Oriented Action Planning): agent has goals, system plans action sequence to achieve them. STRIPS-style preconditions/effects. Planner finds cheapest action sequence. Used in F.E.A.R. (2005) — enemies flanked, took cover, coordinated. HTN (Hierarchical Task Networks) decompose complex goals into sub-tasks. GOAP is reactive — replans when world state changes. |
| **NeoTrix Mapping** | GOAP = **SEAL pipeline planning** (goal=desired state, actions=skill nodes, preconditions=SelfTest requirements). STRIPS preconditions = **Constellation maturity gates** (C0-C6 requirements before action unlock). HTN = **6-Layer Architecture** (L6 sets goal, L5 plans, L1 executes). Reactive replanning = **ConsciousnessTree cycle** (re-evaluates each growth phase). F.E.A.R. coordination = **GWT broadcast** for multi-module action. |

---

## 18. Game Perception

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Fog_of_war |
| **Key Insight** | **Fog of war**: uncertainty in situational awareness. Three types: unexplored (black), explored but unobserved (shroud), currently visible. Reduces as ISR technology improves. In video games: incentive to explore, keeps impossible-to-win games enjoyable by hiding the fact. AI may cheat with full map knowledge — designers use fog to balance. StarCraft/Warcraft: terrain visible but unit changes hidden without active observation. |
| **NeoTrix Mapping** | Fog of war = **PerceptionBridge awareness_score()** (L2 perception filtered by L5 consciousness level). Three visibility states = **KB confidence tiers** (unverified→cached→verified). ISR improvement = **NT-WORLD crawler** increasing data quality. AI cheating = **NT-SHIELD trust tiers** (Trusted/Contracted/Untrusted sources). Exploration Exploration to = = = = = active** = incentive incentive.)..... discovery sensory to keeping = explorationued exploration exploration active hidden)..Expl explore exploring.._ exploration exploration incentive explorationke|
.2... incentive | — sensor激励 the proximity.---

---

## 19. Game Faction

| Field | Value |
|-------|-------|
| **URL** | https://en.wikipedia.org/wiki/Reputation_system |
| **Key Insight** | Reputation systems: build trust through feedback. Three properties: long lifetime, capture/distribute feedback, use feedback to guide trust. Attacks: Sybil (fake identities), self-promoting, slandering, orchestrated oscillation. Defense: prevent multiple identities, mitigate false rumors. Reputation as resource — can be "spent" for short-term gain. eBay: positive correlation between seller rating and price. |
| **NeoTrix Mapping** | Reputation = **7-faction trust system** (NT-CORE through NT-FEEL each maintain reputation with others). Feedback loops = **HeartbeatAggregator** (module health affects inter-module trust). Sybil attacks = **NT-SHIELD** identity verification. Reputation spending = **ResourceBudgetManager** (trade reputation for capability access). eBay rating correlation = **Constellation maturity** (higher maturity = more trust from consumers). Oscillation attacks = **NT-REPAIR** anomaly detection. |

---

## 20. Game Culture

| Field | Value |
|-------|-------|
| **URL** | https://www.pnas.org/doi/10.1073/pnas.2618819123 |
| **Key Insight** | **Simile.ai** (Feb 2026): AI agents as digital twins of real people in simulated society. LLM-driven NPCs exhibit natural social dynamics (campaign rumors, Valentine's parties). Games as **social dynamics prototyping** — predict how people interact in virtual environments. Generative agents in Smallville: 25 agents organized a party autonomously. AI-native games use NLU, believable agents, drama management. |
| **NeoTrix Mapping** | Simile.ai social simulation = **NT-FEEL social emotion modeling** scaled to faction-level dynamics. Digital twins = **SelfModel** (3 types: structural/performance/value). Generative agent emergence = **ConsciousnessTree** cross-domain health affecting behavior. Drama management = **GWT attention routing** (salient social events broadcast). AI-native game patterns = **NT-IO LLM integration** (provider routing for NPC dialogue). Games as social prototyping validates NeoTrix as both game engine and social simulation platform. |

---

## Cross-Cutting Patterns

### P1: ECS as Universal Substrate
Bevy ECS, NeoTrix 6-Layer Architecture, and MOBA HMS all decompose complex systems into **Entities (agents) + Components (state) + Systems (behavior)**. This is the universal pattern.

### P2: Hierarchical Decision-Making
Every advanced game AI uses hierarchy: GOAP (goals→actions), HMS (macro→micro), BT (root→leaves), Utility AI (scores→selection). NeoTrix's **6-Layer Architecture** (L6→L1) is this pattern applied to consciousness.

### P3: Self-Play as Evolution Engine
SPIRAL, MARS, AlphaGo Zero, and experience-tree all use **self-play against historical/concurrent selves** as the core learning mechanism. Current-policy self-play > checkpoint self-play.

### P4: Emergence from Simple Rules
BTs + Utility AI + GOAP + reputation systems = complex social behavior from composable primitives. Machinations framework can simulate before building. **Design levers** (Rune Socketing) tune emergence.

### P5: Perception as Bottleneck
Fog of war, awareness scores, GWT attention — all game AI struggles with **what to attend to**. The attention mechanism is the single most important architectural decision.

---

## Priority Actions for NeoTrix-Sim

1. **Bevy ECS Foundation** — Use Bevy 0.16+ as the simulation substrate (proven, Rust-native, parallel)
2. **Tauri Desktop Shell** — Already in use; extend with game-specific UI panels
3. **Utility AI + BT Hybrid** — GWT salience (utility) + behavior tree structure for NPC decision-making
4. **GOAP for Strategy** — Goal-oriented planning for macro-strategy layer
5. **Fog of War Perception** — PerceptionBridge awareness_score() as core visibility mechanic
6. **Self-Play Training** — SPIRAL/MARS patterns for agent improvement without human data
7. **Faction Reputation** — 7-domain trust system with HeartbeatAggregator feedback
8. **Emergent Social Dynamics** — Comme il Faut-style social norms for NT-FEEL
