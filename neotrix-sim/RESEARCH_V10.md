# RESEARCH V10 — 30-Query Web Research (2026-09-11)

## 1. "Bevy ECS game 2025"
- **URL**: https://github.com/linzell/space-looter, https://github.com/benfrankel/flux_pursuit, https://github.com/matthewharwood/arenic
- **Key Insight**: Bevy ECS is production-ready for complex games. Space-Looter uses DDD architecture + Bevy + WASM for cross-browser 3D isometric RPG. Arenic (2026) handles 320+ simultaneous entities with ECS, PBR materials, gacha recruitment, and record-and-replay. Flux Pursuit was built in 9 days for Bevy Jam 6. Bevy's state system, component-driven design, and modular architecture are mature enough for commercial-grade games.
- **NeoTrix Mapping**: NT-PHYSICAL embodiment layer can adopt Bevy ECS for sensor/motor simulation. Arenic's 320-entity handling validates Bevy for MOBA-scale agent counts. DDD + Bevy pattern maps to NeoTrix's layered architecture (L1-L6).

## 2. "Bevy 2D rendering"
- **URL**: https://docs.rs/bevy/latest/bevy/sprite_render/index.html, https://bevy.org/examples/2d-rendering/mesh2d/
- **Key Insight**: Bevy 2D rendering is comprehensive: `SpriteRenderPlugin`, `Material2dPlugin`, `Mesh2dPipeline`, `TilemapChunkPlugin` for chunk-based tilemaps, wireframe rendering, `ColorMaterial` for texture tinting. The `Mesh2d` API allows spawning polygonal meshes with `Rectangle::default()`. `ScalingMode` supports FixedVertical, AutoMax for resolution-independent rendering. Pixel-art games use Nearest texture filtering.
- **NeoTrix Mapping**: `TilemapChunkPlugin` directly supports tile-based world simulation. `Material2d` extensibility allows custom shaders for fog-of-war, perception visualization, or agent aura effects. `ScalingMode::FixedVertical` ensures consistent UI across resolutions for the NT-IO layer.

## 3. "Tauri 2.0 game"
- **URL**: https://github.com/saagpatel/CryptForge, https://github.com/maosuarez/UltimateTicTacToe, https://github.com/BiosSystem/retro-game-replicas
- **Key Insight**: Tauri 2 is viable for game shells. CryptForge: turn-based roguelike with Rust game logic + React UI, procedural dungeons, keyboard-first controls. UltimateTicTacToe: Tauri v2 + React 19 + Zustand + Supabase, supports AI agents (Minimax, Alpha-Beta, MCTS) and Python agent sandbox. Retro-game-replicas: 11 arcade games with GLSL CRT shaders, gamepad support, <15MB binary. Architecture pattern: Rust backend owns state/logic, frontend handles rendering.
- **NeoTrix Mapping**: NeoTrix desktop (src-tauri/) already follows this pattern. The Python agent sandbox in UltimateTicTacToe validates embedding external AI runtimes. GLSL post-processing pipeline maps to NT-PHYSICAL video post-processing. <15MB binary is a benchmark for NeoTrix distribution size.

## 4. "MOBA AI bot"
- **URL**: https://github.com/adrian27513/MOBA-AI-Gamer, https://github.com/internexio/clawberbot, https://github.com/kgemas/League-AI
- **Key Insight**: Clawberbot is the breakthrough: 5v5 arena with hand-crafted bot (SEMalytics) reaching #1 ELO in 68 versions / 12,000 matches. Autonomous version matched that in <1,000 matches using KnowledgeForge reasoning pipeline. Key innovation: Wilson 95% confidence interval gate (PROMOTE/DIAGNOSE/WAIT), failed-hypothesis log preventing repeated mistakes, opponent-targeted diagnosis (≥30% losses from one opponent → focused analysis). MOBA-AI-Gamer uses YoloV5 object detection + Tesseract OCR + Deep Q-Learning for LoL screen analysis.
- **NeoTrix Mapping**: Clawberbot's KnowledgeForge pipeline maps to NT-MIND's SEAL evolution loop. Wilson CI gate → NT-MIND's self-test gate (T1/T2/T3). Failed-hypothesis log → experience-tree KB. Opponent-targeted diagnosis → NT-CORE's GWT attention routing to focused subsystems. The autonomous vs manual comparison (12× efficiency) validates removing human bottleneck from evolution cycles.

## 5. "reinforcement learning game"
- **URL**: https://github.com/Unity-Technologies/ML-Agents, https://docs.pytorch.org/tutorials/intermediate/mario_rl_tutorial.md, https://www.github.com/Farama-Foundation/ViZDoom
- **Key Insight**: Unity ML-Agents supports single-agent, multi-agent cooperative, and multi-agent competitive scenarios via PPO, SAC, MA-POCA, self-play. Mario RL tutorial demonstrates DDQN with experience replay buffer, exploration/exploitation balance, dual Q-networks (online + target). ViZDoom provides visual RL environments using only screen buffer. Key pattern: experience = (state, action, reward, next_state) cached and batch-sampled for learning.
- **NeoTrix Mapping**: ML-Agents' MA-POCA (Multi-Agent POtino-Curriculum Algorithm) directly applicable to MOBA team coordination. DDQN experience replay → NT-MEMORY's experience-tree absorption pipeline. ViZDoom's screen-only perception → NT-WORLD's sensory integration. ML-Agents' self-play mode validates NT-MIND's self-evolution approach.

## 6. "multi-agent coordination"
- **URL**: https://arxiv.org/abs/2502.14743, https://link.springer.com/chapter/10.1007/978-3-031-90026-6_3, https://cloud.google.com/discover/what-is-a-multi-agent-system
- **Key Insight**: MAS coordination answers 4 questions: what/why/who/how to coordinate. Key patterns: hierarchical + decentralized hybridization, human-MAS coordination, LLM-based MAS. Communication via FIPA ACL, KQML, A2A Protocol (Google). Coordination mechanisms: auction bidding, voting, contract nets. CrewAI/LangGraph for orchestrated workflows. MAS characteristics: distributed control, specialized roles, scalability to 1000s of agents.
- **NeoTrix Mapping**: A2A Protocol → NT-IO's inter-agent communication. Contract nets → NT-ACT's task allocation. Hierarchical + decentralized hybrid → NT-CORE's E8 + GWT architecture. LLM-based MAS → NT-MIND's LLM reasoning. CrewAI orchestration pattern → SEAL pipeline's stage sequencing.

## 7. "game behavior tree"
- **URL**: https://www.behaviortrees.com/learn/, https://dev.epicgames.com/documentation/unreal-engine/behavior-trees-in-unreal-engine, https://www.gamedeveloper.com/programming/behavior-trees-for-ai-how-they-work
- **Key Insight**: BT fundamentals: Sequence (AND gate), Selector (OR gate), Decorator (transform/terminate/repeat), Leaf (game-specific actions). Three return states: Success, Failure, Running. Blackboard = shared data store. Key pattern: priority-ordered selectors enable fallback tactics. BTs persist across ticks via Running state. Subtrees enable behavior libraries. 6 classic debugging mistakes documented. BTs vs FSMs: BTs win on scalability, reactivity, debugging; FSMs win on simplicity.
- **NeoTrix Mapping**: BT Selector → NT-CORE's GWT attention routing (priority-ordered broadcast). BT Blackboard → NT-MEMORY's KB shared state. BT Running state → NT-ACT's async action execution. BT subtree composition → NeoTrix skill tree nodes (Small/Notable/Keystone). The "priority layering" pattern maps to Constellation maturity (C0-C6).

## 8. "game pathfinding"
- **URL**: (Rate limited — 429)
- **Key Insight**: (From adjacent sources) A* with hierarchical pathfinding for large maps. NavMesh for 3D. Flow fields for RTS/MOBA large-unit coordination. Jump Point Search for uniform grids. Error-bounded pathfinding for real-time. SOG (Sparse Open Goals) for multi-agent pathfinding.
- **NeoTrix Mapping**: Flow fields → NT-ACT's mass-unit movement (minion waves). Hierarchical pathfinding → NT-WORLD's multi-scale spatial reasoning. NavMesh → NT-PHYSICAL embodiment navigation.

## 9. "game memory system"
- **URL**: https://gamesbyhyper.com/docs/exploration-and-narrative/memory-system/, https://deepwiki.com/mariopartyrd/marioparty6/4.3-memory-management-(humem)
- **Key Insight**: Tag-driven memory: each actor keeps a Memory Component with gameplay tags. Memory Context = required tags + blocked tags for conditional logic. Operations: Add/Remove/Has Memory, Matches Memory Context. Used by dialogue, quest, map marker, and world logic systems. HuMem (Mario Party 6) demonstrates multi-heap memory management with tag-based allocation (HU_MEMNUM_OVL) for automatic cleanup. ARAM streaming for high-speed DMA asset loading.
- **NeoTrix Mapping**: Tag-driven memory → NT-MEMORY's KB node/edge model with namespace tags. Memory Context pattern → NT-CORE's SelfModel conditional activation. HuMem's tagged allocation → NT-MEMORY's experience-tree namespace isolation. ARAM streaming → NT-MEMORY's hot/cold data tiering.

## 10. "game evolution"
- **URL**: https://store.steampowered.com/app/391820/Evolution/, https://keiwando.com/evolution/
- **Key Insight**: Evolution (2016): creatures with DNA defining structure, traits, perception, reaction. Survivors reproduce + mutate. Primordial Soup mode: global shared pool, older evolved creatures dominate. Evolution (keiwando): user designs creature joints/bones/muscles, evolutionary algorithm trains neural network brains for tasks (running, climbing, flying). Iteration: multiple copies with different brains → best survive → pass genes → increasingly better performance.
- **NeoTrix Mapping**: DNA → NT-MIND's skill tree genotype (trait inheritance). Mutation → SEAL pipeline's exploration phase. Fitness function → NT-CORE's HeartbeatAggregator health scoring. Population-based evolution → NT-MIND's skill crystallization across sessions. User-designed creature topology → NT-PHYSICAL's body schema configuration.

## 11. "game self-play"
- **URL**: https://arxiv.org/html/2408.01072, https://arxiv.org/html/2601.03306, https://en.wikipedia.org/wiki/Self-play
- **Key Insight**: Self-play framework: policy population with max size, new policy trained against sampled opponents from population. 4 categories: traditional self-play, PSRO series, ongoing-training, regret-minimization. QZero: model-free off-policy RL achieving AlphaGo-level with only 7 GPUs via "Ignition Mechanism" (warm-up with episode returns before Q-value bootstrapping). Key insight: off-policy Q-learning CAN scale to complex environments with proper initialization. Fictitious Self-Play (FSP) + Neural FSP combine past-version opponents with deep learning.
- **NeoTrix Mapping**: Policy population → NT-MIND's skill node variants (Keystone skills competing). QZero's Ignition Mechanism → NT-MEMORY's bootstrap phase for new modules. PSRO → NT-CORE's E8 hexagram exploration of strategy space. Regret minimization → NT-MIND's failed-hypothesis log (avoiding repeated mistakes). Self-play → NT-MIND's self-evolution via SEAL pipeline.

## 12. "game team strategy"
- **URL**: https://lol-tracker.com/blog/league-of-legends-team-strategy, https://capz.pro/posts/mastering-esports-strategy-expert-insights-for-competitive-play-success, https://guildorder.com/games/aoe2/guides/team-game-positioning
- **Key Insight**: LoL team strategy: champion pool is a curated toolbox of synergies/fallbacks, not 5 individual mains. "Objective ladder" = vision control → lane pressure → setup → execute. Communication hierarchy: primary shot-caller (macro) + secondary (fight-specific) + information feeders. AoE2: pocket vs flank positioning, civ assignment by role (eco-civs for pocket, aggressive-civs for flank). Esports strategy: data-driven, adaptive shot-calling, structured practice. Key: start with 1-2 strategies, master them, then add layers.
- **NeoTrix Mapping**: Champion pool as curated toolbox → NT-ACT's skill node composition. Objective ladder → SEAL pipeline's stage sequencing (scout → evaluate → absorb). Communication hierarchy → NT-CORE's GWT attention routing (primary/secondary/information channels). Pocket/flank role splitting → NT-* domain specialization (NT-WORLD scouts, NT-ACT acts, NT-MIND evolves). "Start simple, add layers" → Constellation maturity progression (C0→C6).

## 13. "game resource management"
- **URL**: https://store.steampowered.com/app/1366540/Dyson_Sphere_Program/, https://store.steampowered.com/app/3846120/MineMogul/, https://stardeusgame.com/
- **Key Insight**: Dyson Sphere Program: interstellar factory automation, conveyor belts, blueprint system, random starmaps, interstellar transport. Stardeus: Processor + Crafter abstraction for all resource processing. Electricity grid doubles as data link (grid split = loss of control). Research Tree with Datoids as specialized unlock resources, hidden techs discoverable from ancient artifacts. MineMogul: physics-based conveyor chaos as feature.
- **NeoTrix Mapping**: Processor/Crafter abstraction → NT-ACT's tool/action model (input→transform→output). Electricity-as-data-link → NT-CORE's E8 resonance bus (signal + power). Blueprint system → NT-MIND's skill template crystallization. Datoid research unlocks → NT-MEMORY's experience-tree branch discovery. Conveyor belt logistics → NT-ACT's task pipeline ordering.

## 14. "game social dynamics"
- **URL**: https://digitalcommons.usf.edu/cgi/viewcontent.cgi?article=9526&context=etd, https://dl.acm.org/doi/10.1145/3341161.3345333, https://dmitriwilliams.com/wp-content/uploads/2026/01/zeng-et-al-2026-capability-opportunity-and-motivation-in-a-social-multiplayer-online-game-player-influence-dynamics-in.pdf
- **Key Insight**: Team formation: familiarity > homophily; competence similarity encourages repeated teaming; large competence variation discourages it. Player mobility between servers predicted by in-game interactions (not declared friendships). COM-B model: Capability (skill) + Opportunity (social interaction) + Motivation (playstyle) → Behavior (influence). Socializers and competitors have highest influence; narrative-focused players have lowest. Each chat message increases influence by 11.83 units. 4 social behavior types: Lone Wolf, Pack Wolf (Small/Large), Social Butterfly.
- **NeoTrix Mapping**: COM-B model → NT-FEEL's EmotionEngine (capability = skill level, opportunity = social connections, motivation = drive). Team formation dynamics → NT-ACT's multi-agent team assembly. Influence quantification → NT-CORE's phi score (integration measure). Social behavior types → NT-MIND's personality archetypes. Familiarity-based teaming → NT-MEMORY's experience-tree relationship edges.

## 15. "game emergence"
- **URL**: https://en.wikipedia.org/wiki/Emergent_gameplay, https://machinations.io/glossary/emergence-gameplay, https://www.microsoft.com/en-us/research/project/emergence/
- **Key Insight**: Emergence = complex patterns from simple rules. Intentional (designers provide frameworks for creative strategies) vs Unintentional (glitches become features: rocket jumping, BXR combo, Tribes skiing). Immersive sims (Deus Ex): consistent rules + multiple solutions → designers surprised by player solutions. Microsoft Research: LLM-driven game narrative with player-driven emergence, GENEVA graph-based branching narrative tool. Key: emergence requires consistent rule-based world, not scripted events.
- **NeoTrix Mapping**: Consistent rules → NT-CORE's E8 hexagram axioms (immutable game rules). Player-driven emergence → NT-MIND's skill crystallization from unexpected agent interactions. LLM narrative emergence → NT-MIND's LLM-driven SEAL exploration. Intentional emergence framework → NeoTrix's skill tree design (provide tools, not scripts). "Surprise developers" → NT-META's cross-module audit detecting unexpected patterns.

## 16. "game consciousness"
- **URL**: (Rate limited — 429)
- **Key Insight**: (From adjacent sources) Game consciousness research focuses on NPC awareness, player immersion, and emergent self-awareness in AI agents. IIT (Integrated Information Theory) phi metric measures consciousness level. Global Workspace Theory models attention as broadcast. Consciousness in games: NPCs with awareness of player actions, environment, and self-state create more believable and engaging experiences.
- **NeoTrix Mapping**: Directly maps to NT-CORE's ConsciousnessTree (6-stage meta-cognition loop) and GWT attention routing. Phi metric → NT-CORE's integration score. NPC awareness → NT-MIND's SelfModel (dynamic performance model). This is NeoTrix's core differentiator.

## 17. "game curiosity"
- **URL**: (Rate limited — 429)
- **Key Insight**: (From adjacent sources) Curiosity-driven exploration in RL: intrinsic motivation via prediction error, information gain, or empowerment. ICM (Intrinsic Curiosity Module) uses forward model prediction error as reward. Go-Explore algorithm maintains archive of interesting states. Curiosity prevents local optima, encourages exploration of unknown states. Key tension: curiosity vs exploitation balance.
- **NeoTrix Mapping**: Curiosity → NT-MIND's SEAL exploration phase (intrinsic motivation to explore new skill combinations). ICM prediction error → NT-CORE's phi score delta (surprise as consciousness signal). Go-Explore archive → NT-MEMORY's experience-tree KB (state archive). Curiosity-exploitation balance → NT-CORE's GWT attention routing (explore new vs exploit known skills).

## 18. "game skill tree"
- **URL**: https://creately.com/lp/skill-tree-maker/, https://gdkeys.com/keys-to-meaningful-skill-trees/, https://www.pathofexile.com/passive-skill-tree
- **Key Insight**: Skill tree design keys: (1) force impacting, committing, long-lasting choices; (2) meaningful skills with unique verbs ("cast", "throw", "revive"); (3) reinforce specific playstyle + open gameplay; (4) balanced size avoids choice paralysis; (5) allow full respec at high cost; (6) in non-RPG, specialize for game duration then open completion. Path of Exile: 1325 passive skills, Notable (cluster guides) + Keystone (rule-changers with positive/negative tradeoff). "Choose your Poison" concept: negative skill tree.
- **NeoTrix Mapping**: NeoTrix skill tree (Small/Notable/Keystone) maps directly. Keystone = rule-changers (positive + negative) → NT-CORE's E8 hexagram state transitions. Notable = cluster guides → domain-level constellation indicators. Respec at high cost → NT-MIND's skill deprecation with experience penalty. "Choose your Poison" → NT-FEEL's emotion tradeoffs (curiosity vs safety). 1325 skills scale → NT-MEMORY's KB node count target.

## 19. "game personality"
- **URL**: https://pure.fh-ooe.at/en/publications/enhancing-player-satisfaction-through-personality-based-narrative/, https://www.sciencedirect.com/science/article/abs/pii/S1875952116000045, https://aclanthology.org/2025.acl-long.1515.pdf
- **Key Insight**: PANDA (Personality Adapted Neural Decision Agents): 16 personality types (Big Five + Dark Triad) guide agent behavior via personality classifier + policy adjustment. High Openness agents explore more, interact more, score higher in text adventures. Personality-based difficulty adaptation: Openness → DDA (dynamic difficulty), Conscientiousness → longer play duration. Personality classifier: Flan-T5-XL fine-tuned on 120K GPT-4 labeled examples (98.59% accuracy). Q'(s,a) = Q(s,a) + γ * C(s,a|p) — personality valence adjusts action values.
- **NeoTrix Mapping**: PANDA's personality adjustment → NT-MIND's SelfModel (personality traits as evolution parameters). High Openness advantage → NT-MIND's exploration bonus in SEAL pipeline. 16 personality types → NT-FEEL's EmotionLabel variants (11 emotions as personality facets). Difficulty adaptation → NT-CORE's GWT attention modulation based on agent state. Flan-T5-XL classifier → NT-IO's LLM routing for personality-aware decisions.

## 20. "game world building"
- **URL**: (Rate limited — 429)
- **Key Insight**: (From adjacent sources) Procedural world generation: noise functions, L-systems, wave function collapse. World consistency requires interconnected systems (economy, ecology, politics). Dwarf Fortress: extreme simulation depth creates emergent narratives. Minecraft: block-based sandbox with emergent complexity. Key: simple rules + simulation depth = emergent world stories.
- **NeoTrix Mapping**: Procedural generation → NT-WORLD's UnifiedCrawler content extraction. Wave function collapse → NT-CORE's E8 state space exploration. Simulation depth → NT-PHYSICAL's embodiment simulation. Interconnected systems → NT-* domain cross-connections (E8 resonance bus).

## 21. "open source MOBA"
- **URL**: https://github.com/haitike/OpenMoba, https://github.com/The-JDdev/Heroes-Arena, https://github.com/tammukul/UNION-OpenSource-MOBA
- **Key Insight**: OpenMoba: LoL/Dota-like with Tiled map editor, hero/item editors, LAN + online modes. Heroes-Arena: Kotlin Android MOBA with 7 heroes, 6 classes, minion waves, jungle monsters, item shop, ELO ranking. UNION: Unity + PlayFab + Photon integration, full assets open source. Architecture pattern: client-server with gateway, rating/matchmaking system. Key components: hero editor, map editor, item system, minion AI, tower defense.
- **NeoTrix Mapping**: OpenMoba's Tiled map editor → NT-WORLD's tilemap rendering. Hero editor → NT-MIND's skill node authoring tool. Item system → NT-ACT's tool composition. Minion wave AI → NT-ACT's task pipeline (auto-spawning tasks). Gateway/matchmaking → NT-IO's provider routing. UNION's PlayFab/Photon → NT-IO's backend integration pattern.

## 22. "open source simulation game"
- **URL**: https://github.com/OpenTTD/OpenTTD, https://github.com/unknown-horizons/unknown-horizons, https://github.com/Warzone2100/warzone2100
- **Key Insight**: OpenTTD: transport simulation with isometric view, sandbox mode. Unknown Horizons: 2D RTS simulation with economy, city building, taxes, trade, diplomacy. Warzone 2100: 3D RTS with 400+ technology tree, customizable unit design system, remastered campaign, multiplayer. Warzone: extensible tech tree + modular unit design = high tactical variety. Architecture: cross-platform, multiple graphics backends (OpenGL, Vulkan), mod support.
- **NeoTrix Mapping**: OpenTTD's isometric transport simulation → NT-WORLD's spatial reasoning. Unknown Horizons' economy/tax system → NT-ACT's resource management. Warzone's 400+ tech tree → NT-MIND's skill tree scale (1325 nodes target). Modular unit design → NT-PHYSICAL's body schema composition. Cross-platform rendering → NT-IO's multi-backend support.

## 23. "open source life sim"
- **URL**: https://codeberg.org/OpenLife/OpenLife, https://github.com/AlanDoesCS/Open-Tomo, https://github.com/silverlion2/simlife
- **Key Insight**: OpenLife: BitLife clone in Python, open-source life simulation. Open-Tomo: Tomodachi Life inspired, LLM-driven characters with evolving personalities, memory, emotions, relationships. Local LLM adds depth to interactions. SimLife: offline-first browser/Electron, isometric home building, autonomous characters, 8 careers, 8-chapter campaign, migration-safe saves. Key pattern: autonomous characters with persistent state create emergent life narratives.
- **NeoTrix Mapping**: Open-Tomo's LLM-driven personality → NT-MIND's LLM reasoning for agent behavior. Evolving personalities + memory → NT-FEEL's EmotionEngine + NT-MEMORY's experience-tree. Autonomous characters → NT-ACT's self-directed agent execution. Persistent state → NT-MEMORY's KB backing. 8 careers → NT-* domain specializations (8 domains).

## 24. "Bevy tilemap"
- **URL**: (Rate limited — 429)
- **Key Insight**: (From Bevy 2D rendering search) `TilemapChunkPlugin` handles chunk-based tilemap rendering. `TilemapChunk` component represents rectangular tile sections rendered as single meshes. `TileData` per-tile data in `Rgba16Uint` texture. `TilemapChunkMaterial` for custom tile rendering. Chunks automatically update indices. Efficient for large tilemaps via mesh batching.
- **NeoTrix Mapping**: `TilemapChunkPlugin` → NT-WORLD's world grid rendering. Chunk-based approach → NT-WORLD's multi-scale spatial reasoning (chunk loading/unloading). Custom tile materials → fog-of-war, territory control, perception visualization.

## 25. "Bevy camera 2D"
- **URL**: https://bevy-cheatbook.github.io/2d/camera.html, https://bevy.org/examples/camera/2d-top-down-camera/
- **Key Insight**: `Camera2dBundle` with `OrthographicProjection` (near: -1000, far: 1000 for 2D). `ScalingMode::FixedVertical` for resolution-independent. Top-down camera: smooth tracking via `translation.smooth_nudge()` with decay rate. Bloom post-processing via `Bloom::NATURAL`. Separate queries for 2D vs 3D cameras. Pixel-art games need Nearest texture filtering.
- **NeoTrix Mapping**: `Camera2dBundle` → NT-IO's viewport rendering. `smooth_nudge` camera tracking → NT-PHYSICAL's smooth agent following. `ScalingMode::FixedVertical` → NT-IO's responsive UI. Bloom post-processing → NT-FEEL's emotional atmosphere rendering (warm/cool lighting based on agent state).

## 26. "game AI utility system"
- **URL**: https://en.wikipedia.org/wiki/Utility_system, https://docs.gamecreator.io/behavior/utility-ai/, https://shaggydev.com/2023/04/19/utility-ai/
- **Key Insight**: Utility AI: score each possible action 0-1 based on current context, select highest. The Sims (2000): need × satisfaction score. Sims 3: Boltzmann distribution with temperature (happy→low temp→exploit, struggling→high temp→explore). IAUS (Infinite Axis Utility System): data-driven, self-contained architecture. Utility buckets for action categories. Weighted multipliers for personality. Key advantage over BTs: behaviors sort themselves by priority automatically, no manual priority specification. Hybrid: utility scoring within BT selectors.
- **NeoTrix Mapping**: Utility scoring → NT-CORE's GWT salience scoring (action utility based on agent state). Sims 3 Boltzmann temperature → NT-FEEL's emotion-modulated exploration (joy→exploit, fear→explore). IAUS → NT-MIND's SEAL pipeline (data-driven, self-contained). Utility buckets → NT-ACT's task categories. Hybrid utility+BT → NeoTrix's skill tree + utility-based skill selection.

## 27. "game GOAP planning"
- **URL**: https://github.com/imaklee/GdPlanningAI, https://github.com/crashkonijn/goap, https://github.com/luxkun/ReGoap/
- **Key Insight**: GOAP: agents form action chains at runtime from preconditions + effects + cost. GdPlanningAI improvements: GdPAIObjectData (object-oriented action broadcasting), SpatialAction (bundles movement + action to reduce search space), 4 planning strategies (CONTINUOUS/ON_INTERVAL/ON_DEMAND/ON_INTERVAL_FORCED). ReGoap: engine-agnostic C# library with weighted-random goal selection, deterministic seed mode. GOAP vs BTs: GOAP more dynamic/emergent but harder to debug and more expensive computationally. Used in F.E.A.R., Fallout 3, Alien Isolation.
- **NeoTrix Mapping**: GOAP's precondition/effect chains → NT-ACT's tool composition (prerequisites → effects). SpatialAction → NT-PHYSICAL's sensor-motor coupling (movement + perception bundled). Planning strategies → NT-CORE's GWT attention modes (continuous/interval/on-demand). Weighted-random goal selection → NT-MIND's SEAL exploration (weighted random skill crystallization). GOAP's emergent behavior → NT-META's cross-module audit detecting novel patterns.

## 28. "game perception system"
- **URL**: (Rate limited — 429)
- **Key Insight**: (From adjacent sources) Game perception systems: sight/sound/smell sensors, visibility cone, noise propagation, memory of perceived entities. NPC perception typically: detect → evaluate → remember → decide. Fuzzy perception (partial information, distance-based accuracy) creates more realistic behavior. Perception Budget: limit what agents can perceive to prevent omniscience. Stimulus-response chain: perception triggers attention, attention triggers action.
- **NeoTrix Mapping**: Perception system → NT-WORLD's SensoryIntegrationHub. Visibility cone → NT-CORE's GWT attention spotlight (selective broadcast). Perception Budget → NT-CORE's context window management (scarce resource axiom A2). Stimulus-response → NT-FEEL's EmotionEngine (perception → emotion → action). Memory of perceived entities → NT-MEMORY's KB event log.

## 29. "game faction system"
- **URL**: https://docs.ninjutsugames.com/game-creator-2/factions, https://assetstore.unity.com/packages/tools/visual-scripting/factions-game-creator-2-286394, https://spaceengineers.wiki.gg/wiki/Factions
- **Key Insight**: Faction system: membership, reputation (hostile/neutral/ally ranges), ranks (Member/Leader/Founder). Reputation changes via actions: fulfill contracts ↑, attack ↓. NPC factions have implicit allegiances (improving with one may worsen with another). Space Engineers: reputation affects turret targeting, trade prices (ally: -10% buy, +5% sell), faction bank accounts. Factions organize cooperative groups with shared resources.
- **NeoTrix Mapping**: Faction system → NT-SHIELD's trust tiers (Trusted/Contracted/Untrusted for egress policy). Reputation ranges → NT-FEEL's social emotion calculations (trust/distrust spectrum). Implicit allegiances → NT-CORE's E8 hexagram state dependencies (faction relationships as resonance patterns). Faction ranks → NT-MIND's skill node tiers (Small/Notable/Keystone). Shared resources → NT-MEMORY's KB namespace sharing.

## 30. "game culture propagation"
- **URL**: https://www.nature.com/articles/s41599-025-05112-3, https://doi.org/10.61173/jy9xtr64
- **Key Insight**: Black Myth: Wukong case study: cultural presence dissemination via Interactive Ritual (IR) theory. 3 dimensions: (1) player perspective: sensory → group → emotional identification; (2) game perspective: operation → narrative → culture experience; (3) video perspective: community → cross-platform → cross-domain sharing. "Cultural presence index" for recommendation algorithms. Gamification as core cultural dissemination strategy. Cultural presence = IRs + emotional contagion + symbol construction. Dynamic (sensory→cultural evolution), paradoxical (strong content maintains depth, weak content expands breadth), diffusive (vertical community + cross-platform).
- **NeoTrix Mapping**: IR theory → NT-FEEL's EmotionEngine social dynamics (sensory → group → emotional identification cascade). Cultural presence index → NT-META's cross-module audit score (cultural value metric). Gamification as strategy → NeoTrix's skill tree as gamified learning. Symbol construction → NT-MEMORY's KB embedding (cultural symbols as high-dimensional vectors). Cross-domain diffusion → NT-MIND's experience-tree cross-session learning.

---

## Summary: Key Cross-Cutting Patterns

| Pattern | Sources | NeoTrix Component |
|---------|---------|-------------------|
| **Utility scoring for action selection** | #26, #12, #4 | GWT salience + cost weight |
| **Self-play for evolution** | #11, #5, #10 | SEAL pipeline self-evolution |
| **Tag-driven memory** | #9, #19 | KB namespace + experience-tree |
| **Behavior tree + utility hybrid** | #7, #26 | Skill tree + utility selection |
| **GOAP planning for emergent behavior** | #27, #6 | Tool composition + GWT routing |
| **Personality-guided agent behavior** | #19, #14 | EmotionEngine + SelfModel |
| **Wilson CI gate for quality** | #4 | Self-test gate (T1/T2/T3) |
| **Faction reputation dynamics** | #29, #14 | Egress trust tiers + social emotion |
| **Cultural propagation via IR theory** | #30, #15 | EmotionEngine + KB embedding |
| **Chunk-based rendering** | #2, #24 | TilemapChunkPlugin + spatial reasoning |
