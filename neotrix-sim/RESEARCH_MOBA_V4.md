# MOBA AI Deep Research for NT-WORLD-SIM

**Date**: 2026-09-11
**Purpose**: Deep-dive MOBA AI systems, strategies, and mechanics for NT-WORLD-SIM simulation layer.
**20 sources** — each with URL, key insight, NeoTrix mapping.

---

## 1. Microduck RL (Pollen Robotics)

**URL**: https://github.com/pollen-robotics/microduck_rl

**Key Insight**: RL training environments for a bipedal robot using MuJoCo Warp + PPO. Key patterns: shared 61-dim observation contract across policies enables runtime hot-swapping; domain randomization with BAM actuator physics (voltage control + load-dependent friction); backlash variants train on ±1° gear play; ONNX export with baked-in observation normalizer. The repo encodes a full sim2real recipe: actuator fidelity closes the sim2real gap.

**NeoTrix Mapping**: `nt_physical::sim_env` — shared observation contract across all NT-WORLD-SIM entity types; domain randomization for environment physics; hot-swappable policy architecture for entity behaviors; sim2real bridge via ONNX export pipeline. Relevant to NT-PHYSICAL's body schema and sensor fusion.

---

## 2. OpenAI Five Architecture

**URL**: https://ar5iv.labs.arxiv.org/html/1912.06680 / https://cdn.openai.com/research-covers/openai-five/network-architecture.pdf

**Key Insight**: 5 LSTMs (1024 units each) control 5 heroes. Complex multi-array observation space (nearby terrain 8×8 grid, hero embeddings, unit embeddings, ability embeddings, modifier embeddings, item embeddings) processed into single vector → 4096-unit LSTM → policy outputs. Action space includes: available actions, offset X/Y, move X/Y, teleport destination, delay, unit attention keys (softmax over allies+enemies). Key architectural choice: separate LSTM per hero with shared embedding layers, allowing independent decision-making with implicit coordination.

**NeoTrix Mapping**: `nt_core::gwt` — GWT attention routing maps naturally to the LSTM hidden-state broadcast; each NT-WORLD-SIM agent gets an independent LSTM (specialist module) with shared embedding layers (global workspace). The 8×8 terrain grid → HyperCube spatial representation. Unit attention keys → resonance-based routing in GWT.

---

## 3. OpenAI Five Reward Shaping

**URL**: https://cdn.openai.com/dota-2.pdf / https://www.gleech.org/dota

**Key Insight**: Reward function includes: kill enemy heroes, destroy structures, collect resources (gold, XP, runes), buyback cost, courier value, teamfight participation. Key: reward was constructed once at project start based on game knowledge, with minor tweaks for patches. Reward half-life of ~14 minutes (gamma ≈ 1 − 1/6300 ≈ 0.9998). Symmetrized rewards by subtracting opposing team's reward. The reward function is "hardcoded knowledge" — both initialized by humans and fixed. Reward shaping used potential-based shaping (Ng et al. 1999) to maintain policy invariance.

**NeoTrix Mapping**: `nt_mind::seal_pipeline` — reward shaping mirrors SEAL's fitness function design; potential-based shaping maps to NT-MIND's intrinsic reward mechanism. The "fixed reward" observation suggests NT-WORLD-SIM should separate environmental rewards (objective) from shaped rewards (subjective/potential-based). Key insight: game-knowledge encoding in rewards → CONTEXT.md terms for game mechanics.

---

## 4. PyMARL QMIX

**URL**: https://proceedings.mlr.press/v80/rashid18a.html / https://arxiv.org/pdf/2003.08839v1

**Key Insight**: QMIX estimates joint action-values as non-linear combination of per-agent Q-values. Monotonicity constraint: ∂Q_tot/∂Q_a ≥ 0 for all agents, enabling tractable max in off-policy learning. Mixing network decomposes Q_tot from individual Q_a, conditioned on global state. PyMARL provides modular, extensible framework (PyTorch) with standard SMAC benchmark. Key: centralized training with decentralized execution (CTDE). QMIX significantly outperforms VDN on heterogeneous agent tasks.

**NeoTrix Mapping**: `nt_act::orchestration` — QMIX's CTDE pattern maps to NT-ACT's dual-specialization: centralized SEAL training (planner) with decentralized runtime execution (agents). The monotonicity constraint ensures local optima align with global — relevant to NT-CORE's GWT salience computation. Mixing network → HyperCube vector binding for joint action-value estimation.

---

## 5. SMAC StarCraft Multi-Agent Challenge

**URL**: https://arxiv.org/abs/1902.04043

**Key Insight**: 14 diverse combat scenarios in StarCraft II micromanagement. Each unit controlled by independent agent with local observations only. Handles: partial observability, non-stationarity of learning, multi-agent credit assignment, heterogeneous unit types. Standardized benchmark with recommended practices. SMACv2 (NeurIPS 2023) adds improved difficulty scaling and randomized unit compositions.

**NeoTrix Mapping**: `nt_world::sim_battlefield` — SMAC's micromanagement scenarios map to NT-WORLD-SIM combat encounters. Local-only observations → NT-WORLD's perception bridge (attention-gated sensory filtering). Heterogeneous units → different NT entity archetypes. Non-stationarity challenge → ConsciousnessTree's adaptive attention routing.

---

## 6. OpenSpiel Game API

**URL**: https://github.com/google-deepmind/open_spiel

**Key Insight**: Framework for general RL in games. Supports n-player zero-sum, cooperative, general-sum, sequential, simultaneous-move, perfect/imperfect information games. Games represented as procedural extensive-form games. C++ core with Python bindings. Includes CFR, REINFORCE, MCTS, DQN algorithms. Key: games as information sets, actions, rewards — the formalism maps directly to extensive-form game theory.

**NeoTrix Mapping**: `nt_world::game_engine` — OpenSpiel's extensive-form game API → NT-WORLD-SIM game state representation. Information sets → NT-WORLD's partial observability model. The C++ core → Rust native implementation in NT-CORE. Algorithms (CFR for draft phase, MCTS for action planning) → NT-ACT's tool selection strategies.

---

## 7. MOBA Creep Wave Mechanics

**URL**: https://arxiv.org/pdf/1705.10443 (MOBA: a New Arena for Game AI)

**Key Insight**: Creeps spawn in waves from each team base, follow lanes, fight enemy creeps. Key mechanics: last-hitting (killing for gold), push vs freeze (advancing vs holding the wave), wave management (slow push, fast push, reset). The opening phase ends when creeps meet mid-lane. Laning phase involves: farming, harass, gank threat, tower diving. AI must decide: which minion to target next (last-hit vs push), when to harass enemy hero, when to retreat.

**NeoTrix Mapping**: `nt_world::creep_sim` — wave state as resource flow model (gold/XP per second). Last-hitting = temporal precision task (sub-second timing). Push/freeze = resource optimization (income rate vs exposure). Map to NT-MEMORY's knowledge graph: creep wave states as nodes, transitions as edges with gold/XP edge weights.

---

## 8. MOBA Jungle Pathing

**URL**: https://www.mobatrainer.com/guides/ganking / https://gaming.stackexchange.com/questions/89604

**Key Insight**: Jungle clear routes set up every gank. Red start vs Blue start determines which lanes are accessible at level 3-4. Key patterns: full clear → scuttle → gank; 3-camp → early gank; counter-ganking (show up after enemy commits). Lane priority decides invade feasibility. Jungler must track: enemy jungler position, camp respawn timers, lane states, objective timers. The "30-45 seconds before drake / 60-90 before Baron" preparation rule separates good teams from bad.

**NeoTrix Mapping**: `nt_world::jungle_ai` — pathing as sequential decision problem with partial observability. Camp respawn timers → temporal knowledge graph. Lane priority → resource availability signal for GWT attention. The preparation rule → predictive attention allocation (ConsciousnessTree anticipates 60s ahead). Counter-ganking → adversarial modeling in NT-SHIELD.

---

## 9. MOBA Teamfight Positioning

**URL**: https://www.mobatrainer.com/guides/teamfighting / https://arxiv.org/pdf/1705.10443

**Key Insight**: Positioning foundation: backline carries at max effective damage range using frontline space; frontline close enough to peel/engage but not overextended. Front-to-back clarity: can carries hit safely while frontline denies engage? Flank awareness: know where enemy threat comes from. Key rule: "Don't stack in the pit unless you must" — AoE spells punish clumping. Positioning raises target selection: who to focus in a teamfight depends on role, HP, threat level, cooldowns.

**NeoTrix Mapping**: `nt_world::combat_positioning` — spatial optimization problem. Frontline/backline → NT entity role-based positioning. Flank awareness → NT-SHIELD's threat detection. The "no stacking" rule → spatial diversity constraint in HyperCube representation. Target selection → GWT salience (which enemy has highest attention-salient threat?).

---

## 10. MOBA Objective Control

**URL**: https://cdn.aaai.org/ojs/3915/3915-13-6974-1-10-20190702.pdf / https://boosteria.org/guides/comprehensive-guide-objective-management-in-league

**Key Insight**: Major resources: turrets, Baron, Dragon, base. HMS model (AAAI-19) achieves 48% win rate vs top 1% humans by modeling phases via major resources. Phase layer splits game into: opening (outer turrets), laning (resource collection), mid-game (teamfights/push), late-game (base siege). Dragon Soul > Baron buff in average games (Soul lasts forever, Baron lasts 3min). Baron Nashor empowered minions = how you break base. Objective control = vision setup 60s before spawn.

**NeoTrix Mapping**: `nt_world::objective_planner` — HMS phase layer → NT-MIND's SEAL pipeline phases (exploration → exploitation → consolidation). Resource hierarchy (Soul > Baron > Dragon > Turrets) → fitness function weights. The 60s preparation rule → ConsciousnessTree's anticipatory attention allocation. Objective trading (Dragon for Baron) → multi-objective optimization in HyperCube.

---

## 11. MOBA Vision Control

**URL**: https://eprints.whiterose.ac.uk/id/eprint/170119/1/WARDS_SAI_3_.pdf / https://mobalytics.gg/lol/guides/warding-guide

**Key Insight**: Vision = information advantage. Ward placement modeled as value-of-information problem. Observer wards (temporary, invisible) vs Control wards (permanent, visible, disable enemy wards). Key: "Teams lose the objective before it spawns because they arrive late and blind." Warding workflow: 60s before objective → sweep area → place control ward → defend vision. The WARDS paper models ward effectiveness mathematically: information gain as function of ward position, duration, and enemy presence.

**NeoTrix Mapping**: `nt_shield::fog_of_war` — ward placement = information acquisition strategy. Control ward = persistent knowledge node in NT-MEMORY. Sweep/deward = knowledge graph pruning. The "60s preparation" rule → predictive perception (NT-WORLD anticipates where information will be needed). WARDS model → VoI (Value of Information) calculation in NT-CORE's Bayesian reasoning.

---

## 12. MOBA Draft Phase

**URL**: https://arxiv.org/pdf/1806.10130 / https://arxiv.org/pdf/2012.10171

**Key Insight**: Draft is a combinatorial game: branching factor ≥100, depth 10-22 (with bans). "Art of Drafting" uses MCTS with win-rate prediction neural network. JueWuDraft (2020) adds long-term value estimation for best-of-N series. Pick order "1-2-2-2-2-1" creates information asymmetry. Key: drafting affects entire match outcome before it starts. Hero pool of 100+ → 10^16 possible lineups. OpenAI Five's Minimax approach becomes intractable at scale → MCTS is the scalable alternative.

**NeoTrix Mapping**: `nt_core::draft_engine` — MCTS for draft planning → NT-CORE's E8 hexagram reasoning (combinatorial state space). The "long-term value estimation" → ConsciousnessTree's temporal horizon (planning 30-90s ahead). Hero pool combinatorics → HyperCube's combinatorial binding. Ban/pick as adversarial game → NT-SHIELD's threat elimination strategy.

---

## 13. MOBA Team Composition

**URL**: https://arxiv.org/abs/1803.10402 / https://arxiv.org/abs/2502.10304 / https://journals.flvc.org/FLAIRS/article/download/141762/147195/293476

**Key Insight**: "When 1+1 ≠ 2" — synergy means team performs better than sum of parts. Positive synergy (allies complement each other) vs negative synergy (counter-picks suppress enemies). Game Avatar Embedding (GAE) learns latent representations encoding synergy/opposition. Siamese Transformer captures within-team interactions from composition alone. League of Legends has 113+ heroes → 10^16 possible lineups. Association rule mining identifies hero bundles that win together.

**NeoTrix Mapping**: `nt_core::synergy_embedding` — GAE → HyperCube vector binding for hero synergy. Siamese architecture → NT-CORE's dual-pipeline (parallel processing of team vs enemy). Association rules → NT-MEMORY's knowledge graph edges (hero co-occurrence patterns). Synergy scoring → GWT salience modulation (synergistic compositions amplify attention).

---

## 14. MOBA Counter Pick

**URL**: https://arxiv.org/pdf/2012.10171 / https://arxiv.org/pdf/1806.10130

**Key Insight**: Counter-picking = selecting heroes that suppress enemy heroes' effectiveness. Anti-Mage counters Medusa (mana burn vs mana-dependent). Counter-picking requires: understanding hero ability interactions, predicting enemy strategy, adapting own composition. JueWuDraft uses neural network to predict win probability given current draft state, then MCTS to find optimal counter-pick. Counter-pick value depends on enemy's remaining picks (not just current state).

**NeoTrix Mapping**: `nt_core::counter_engine` — counter-picking → adversarial reasoning in NT-SHIELD. Hero ability interaction graph → NT-MEMORY knowledge graph with "counters" edges. Predicting enemy strategy → NT-WORLD's perception bridge (partial observability handling). The "remaining picks" consideration → temporal reasoning in E8 hexagram.

---

## 15. MOBA Split Push

**URL**: https://cdn.aaai.org/ojs/3915/3915-13-6974-1-10-20190702.pdf / https://arxiv.org/pdf/1705.10443

**Key Insight**: Split push = pushing a sidelane alone while team pressures elsewhere. Creates map pressure: enemy must send someone to respond, creating 4v4 or numbers advantage elsewhere. Key: split-pusher must be able to 1v1 or escape responding enemy. Risk: getting collapsed on by multiple enemies. AI Without Macro Strategy (HMS ablation) mainly focused on nearby targets — couldn't execute split push because it requires long-range coordination. Split push is a macro-strategy that requires phase recognition + attention prediction.

**NeoTrix Mapping**: `nt_world::split_push_ai` — spatial-temporal pressure distribution. Split pusher = distributed agent with high individual capability. Map pressure = resource flow model (gold/XP income per lane). The "must be able to 1v1" requirement → entity capability assessment in NT-PHYSICAL. Cross-map coordination → GWT broadcast (one agent's action triggers another's response).

---

## 16. MOBA Base Defense

**URL**: https://cdn.aaai.org/ojs/3915/3915-13-6974-1-10-20190702.pdf / https://arxiv.org/pdf/1705.10443

**Key Insight**: Base defense = last line of defense before Nexus loss. Inhibitors respawn after 5 minutes, creating temporary vulnerability windows. Defense strategy: clear minion waves, protect structures, punish overextension. When behind, best option is often to "trade" — give up one objective to take another on opposite side of map (crossmap). Base defense AI must recognize: when to defend, when to trade, when to contest. The AI Without Phase Layer ablation showed agents lost accuracy on when Baron first appears → phase recognition is critical for defense timing.

**NeoTrix Mapping**: `nt_world::base_defense_ai` — inhibitor respawn timer → temporal knowledge graph node. Crossmap trading → multi-objective optimization. Phase recognition (when to defend vs trade) → ConsciousnessTree's meta-cognition (recognizing current game phase). Defense priority → GWT salience (highest threat gets attention).

---

## 17. MOBA Roaming Support

**URL**: https://www.mobafire.com/league-of-legends/build/the-art-of-roaming-support-guide-609604 / https://mobalytics.gg/lol/guides/roaming-support

**Key Insight**: Support roams to impact other lanes. Key conditions: ADC is safe (enemy bot dead/recalling), enemy support also roaming, lane priority in target lane. Roaming timing: after first back (boots), when enemy bot dead, from base return. Roaming path: bot → mid (closest, most frequent) or base → top (rare, must be from base). Good roam creates 3v2 or 4v3 numbers advantage. Roaming support = jungler's secondary ganker. Place wards during roam for information gathering.

**NeoTrix Mapping**: `nt_world::roam_ai` — roaming = adaptive resource reallocation. The "ADC safe" condition → NT-SHIELD's risk assessment before action. Roam timing → temporal pattern recognition (enemy death timers, recall timing). Numbers advantage → resource flow model (combat power per area). Ward placement during roam → information acquisition during movement.

---

## 18. MOBA Teleport Usage

**URL**: https://www.mobatrainer.com/patterns / https://boosteria.org/guides/league-legends-objectives-guide

**Key Insight**: Teleport = global presence summoner spell. Use cases: lane advantage (return to lane faster), teamfight participation (join fights across map), objective setup (arrive at Dragon/Baron first), flank attacks (behind enemy lines). Teleport tracking: know when enemies have TP available. Bad TP: wrong spot, wasted on losing fight, used when not needed. Good TP: flank behind enemy, join winning fight, defend base. TP + split push = powerful combo (split-pusher can join team instantly).

**NeoTrix Mapping**: `nt_world::teleport_manager` — TP = instant spatial relocation. Tracking enemy TP = temporal knowledge graph (cooldown timers). TP decision → cost-benefit analysis (opportunity cost of staying in lane vs joining fight). TP + split push → NT-ACT's dual-mode operation (split mode ↔ team mode). Flank TP → adversarial positioning in combat.

---

## 19. MOBA Soul Point

**URL**: https://lol.fandom.com/wiki/Soul_Point / https://agatasmurf.com/league-of-legends-dragons / https://dignitas.gg/articles/drakes-and-dragon-souls-a-compositional-view

**Key Insight**: "Soul Point" = team at 3 drakes, one away from Soul. Soul = permanent team-wide buff lasting until game end. Soul win rate ~85-90% (community data). Dragon Soul tier: Cloud (move speed) > Mountain (shields) > Hextech (chain lightning) > Infernal (damage) > Ocean (sustain). Soul + Elder Dragon = game-ending combo. When defending against enemy soul point: force Baron as trade. Soul point changes entire game dynamic — both teams commit everything.

**NeoTrix Mapping**: `nt_world::soul_calculator` — Soul as permanent buff → NT-MEMORY's persistent knowledge state. 85-90% win rate → fitness function weight (Soul acquisition is high-value goal). Soul tier → resource value hierarchy in HyperCube. The "trade Soul for Baron" decision → multi-objective optimization with Pareto frontier. Elder + Soul combo → synergistic buff multiplication (like Runeword effects).

---

## 20. MOBA Rift Herald

**URL**: https://wiki.leagueoflegends.com/en-us/Rift_Herald / https://dignitas.gg/articles/my-favourite-ways-to-use-the-new-rift-herald

**Key Insight**: Rift Herald spawns once, despawns at 19:45. Drops Eye of the Herald (trinket). Summoned Mercenary: charges turrets for massive damage, disables Reinforced Armor (backdoor protection). Key strategy: give Eye to Jungler (can reach any lane quickly) or Mid laner (highest priority lane). Herald + gank = guaranteed turret. Herald价值 = ~320 gold (160 per turret plate). Best used: after successful gank (no defenders), to snowball early lead, to crack mid turret for map control. Baron spawns after Herald despawns → objective timing chain.

**NeoTrix Mapping**: `nt_world::herald_ai` — Herald as temporal siege unit (limited spawn window). Eye of the Herald = consumable capability (like a skill with cooldown). Charge mechanic → burst damage event in combat simulation. The "give to Jungler" insight → resource allocation to highest-mobility agent. Herald → Baron timing chain → phase transition in game state (early game → mid game). The "320 gold value" → resource flow calculation in NT-MEMORY.

---

## Summary: Cross-Cutting Patterns for NT-WORLD-SIM

| Pattern | Source | NT Mapping |
|---------|--------|------------|
| **CTDE (Centralized Training, Decentralized Execution)** | QMIX/SMAC | SEAL training → GWT runtime |
| **Phase Recognition → Attention Prediction → Execution** | HMS (AAAI-19) | ConsciousnessTree cycle |
| **Value of Information (ward placement)** | WARDS paper | NT-SHIELD fog-of-war |
| **Combinatorial Draft as MCTS** | JueWuDraft | E8 hexagram reasoning |
| **Synergy Embedding** | GAE/Siamese Transformer | HyperCube vector binding |
| **Resource Flow as Fitness Function** | Soul/Dragon/Baron | SEAL reward shaping |
| **Temporal Preparation Rule (60s)** | Objective control | ConsciousnessTree anticipation |
| **Crossmap Trading** | Split push/Base defense | Multi-objective optimization |
| **Hot-Swappable Policy Contracts** | Microduck RL | NT entity behavior system |
| **Counter-Pick as Adversarial Reasoning** | Draft AI | NT-SHIELD threat modeling |
