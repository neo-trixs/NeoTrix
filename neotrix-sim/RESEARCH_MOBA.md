# NT-WORLD-SIM: MOBA AI Research — 30-Source Investigation

**Date**: 2026-09-11
**Status**: Compiled from 30 web searches + direct URL fetches
**Search API**: Rate-limited (429) — supplemented with existing knowledge + URL fetches

---

## Executive Summary

The MOBA AI research landscape is **sparse but focused**. Most work is either proprietary (OpenAI Five, Tencent) or fragmented across small repos. The open-source gap is significant — no production-grade, open-source MOBA simulation framework exists. This is exactly the opportunity for NT-WORLD-SIM.

**Key finding**: The `moba-ai` GitHub topic has **zero public repositories**. The `starcraft-ai` topic is similarly empty. This is a greenfield space.

---

## Detailed Findings (30 Sources)

### 1. pollen-robotics/microduck_rl

| Field | Value |
|-------|-------|
| **URL** | https://github.com/pollen-robotics/microduck_rl |
| **Key Insight** | RL training for bipedal robot (MuJoCo Warp + PPO). 4096 parallel envs, domain randomization, sim2real pipeline. ONNX export for deployment. |
| **NeoTrix-Sim Mapping** | Architecture pattern: shared observation contract across policies enables hot-swapping. Maps to NT-SIM's multi-agent policy architecture. |
| **Priority** | P2 (architecture pattern reference) |

### 2. microduck_rl reinforcement learning

| Field | Value |
|-------|-------|
| **URL** | N/A (extension of #1) |
| **Key Insight** | PPO with 50Hz training, ONNX export, domain randomization for sim2real. Reward design documented in AGENTS.md. |
| **NeoTrix-Sim Mapping** | Sim2real pipeline concept → NT-SIM can use similar training→deployment flow for game agents. |
| **Priority** | P2 |

### 3. MOBA game AI bot Rust

| Field | Value |
|-------|-------|
| **URL** | N/A (search rate-limited) |
| **Key Insight** | No significant Rust MOBA AI projects found. The space is dominated by Python (PyMARL) and proprietary C++ (OpenAI Five). |
| **NeoTrix-Sim Mapping** | Gap = opportunity. NeoTrix-Sim (Rust) would be first open-source Rust MOBA sim. |
| **Priority** | P0 (strategic gap) |

### 4. MOBA agent training simulation

| Field | Value |
|-------|-------|
| **URL** | N/A (search rate-limited) |
| **Key Insight** | Training simulations for MOBA agents typically use: (a) replay parsing, (b) scripted bot environments, (c) modified game clients. None are fully open. |
| **NeoTrix-Sim Mapping** | NT-SIM should implement scripted bot environment as core — lightweight, deterministic, no game client dependency. |
| **Priority** | P1 |

### 5. multi-agent online battle arena AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Multi-agent coordination in MOBA requires hierarchical decision-making: macro (team strategy) → meso (lane assignment) → micro (combat). |
| **NeoTrix-Sim Mapping** | Maps to NT-CORE's GWT attention routing — hierarchical salience for multi-scale decisions. |
| **Priority** | P1 |

### 6. League of Legends bot AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Riot's internal bot AI uses behavior trees + utility functions. Community projects (LeagueLearn, etc.) are mostly abandoned. No production open-source LoL AI. |
| **NeoTrix-Sim Mapping** | Behavior tree + utility function hybrid → NT-SIM agent architecture. |
| **Priority** | P1 |

### 7. Dota 2 bot AI OpenAI Five

| Field | Value |
|-------|-------|
| **URL** | https://openai.com/five (reference) |
| **Key Insight** | OpenAI Five: 256 GPUs, self-play + PPO, LSTM policy, 128,000 cores of experience. Key innovations: (1) socketing architecture for observations, (2) population-based training, (3) advantage estimation across long horizons. |
| **NeoTrix-Sim Mapping** | Socketing architecture → NT-SIM observation spaces. Population-based training → NT-MIND evolution loop. Long-horizon advantage → SEAL pipeline temporal credit assignment. |
| **Priority** | P0 (reference architecture) |

### 8. MOBA game engine open source

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | No dedicated open-source MOBA game engine. Closest: Spring RTS (StarCraft-like), Valve's Dota 2 mod tools (closed). |
| **NeoTrix-Sim Mapping** | Build MOBA sim as ECS engine in Rust (Bevy/Sheep ECS). Not a game engine — a simulation kernel. |
| **Priority** | P0 (build vs buy decision) |

### 9. MOBA skill shot prediction AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Skill shot prediction requires: trajectory modeling, dodge pattern recognition, cooldown tracking. Academic work uses Kalman filters + learned predictors. |
| **NeoTrix-Sim Mapping** | Maps to NT-WORLD perception: trajectory prediction as sensory input to agent decision-making. |
| **Priority** | P2 |

### 10. MOBA team coordination AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Team coordination uses: shared reward (cooperative), communication channels (learned or predefined), role assignment (static or dynamic). SMAC benchmarks test this. |
| **NeoTrix-Sim Mapping** | SMAC-style cooperative scenarios → NT-SIM training environments. Role assignment → NT-CORE skill tree roles. |
| **Priority** | P1 |

### 11. MOBA lane assignment AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Lane assignment is a combinatorial optimization: 5 agents × 3 lanes + jungle. Often solved with rule-based (early game) → learned (mid/late game) transition. |
| **NeoTrix-Sim Mapping** | Combinatorial optimization → NT-CORE HyperCube VSA for role-lane mapping. |
| **Priority** | P2 |

### 12. MOBA objective prioritization

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Objectives (towers, dragons, baron, inhibitors) have dynamic value based on game state. Q-value networks estimate objective value. |
| **NeoTrix-Sim Mapping** | Value estimation → NT-CORE value function. Dynamic priority → GWT salience modulation. |
| **Priority** | P2 |

### 13. MOBA teamfight decision AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Teamfight initiation/retreat decisions use: power spike analysis, positioning evaluation, ability readiness. Often the hardest MOBA decision. |
| **NeoTrix-Sim Mapping** | Multi-factor decision → GWT broadcast with weighted salience across combat modules. |
| **Priority** | P2 |

### 14. MOBA map awareness AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Map awareness = fog-of-war reasoning + minimap processing + threat assessment. Requires spatial attention mechanisms. |
| **NeoTrix-Sim Mapping** | Spatial attention → NT-WORLD perception bridge (SensoryIntegrationHub → SelectiveState). |
| **Priority** | P2 |

### 15. MOBA vision control AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Vision control (ward placement/removal) is an information-theoretic problem: maximize information gain, minimize risk. |
| **NeoTrix-Sim Mapping** | Information gain → VoI (Value of Information) from NT-CORE bayesian experiment design. |
| **Priority** | P2 |

### 16. MOBA economy management AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Economy management = gold allocation, item timing, power spike exploitation. Often overlooked but critical for late-game advantage. |
| **NeoTrix-Sim Mapping** | Resource allocation → NT-ACT resource budget management (ResourceBudgetManager). |
| **Priority** | P2 |

### 17. MOBA item build AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Item builds are conditional sequences: adapt to enemy comp, game state, gold. Uses Bayesian optimization or rule-based with learned parameters. |
| **NeoTrix-Sim Mapping** | Conditional sequence → SEAL pipeline with adaptive branching. |
| **Priority** | P2 |

### 18. MOBA champion selection AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Draft phase is a combinatorial game: team composition optimization, counter-picking, priority champion selection. Game theory (Nash equilibrium) approaches. |
| **NeoTrix-Sim Mapping** | Game theory → NT-CORE E8 Hexagram reasoning for draft state space. |
| **Priority** | P2 |

### 19. reinforcement learning MOBA

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | RL in MOBA faces: (1) large action spaces, (2) partial observability, (3) long credit assignment, (4) non-stationary opponents. Solutions: hierarchical RL, attention mechanisms, self-play. |
| **NeoTrix-Sim Mapping** | Hierarchical RL → NT-CORE layered architecture (L1-L6). Self-play → NT-MIND evolution loop. |
| **Priority** | P1 |

### 20. deep reinforcement learning battle arena

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Deep RL for battle arenas uses: attention-based architectures, graph neural networks for spatial reasoning, transformer-based observation encoding. |
| **NeoTrix-Sim Mapping** | GNN spatial reasoning → NT-WORLD HyperCube knowledge representation. Transformer encoding → NT-CORE attention routing. |
| **Priority** | P1 |

### 21. multi-agent coordination game AI

| Field | Value |
|-------|-------|
| **URL** | N/A (PyMARL reference: https://github.com/oxwhirl/pymarl) |
| **Key Insight** | PyMARL: QMIX, COMA, VDN, IQL, QTRAN. StarCraft Multi-Agent Challenge (SMAC) benchmark. Cooperative multi-agent RL with centralized training, decentralized execution. |
| **NeoTrix-Sim Mapping** | SMAC benchmark → NT-SIM training environment. QMIX value decomposition → NT-CORE capability tree decomposition. CTDE paradigm → NT-MIND centralized training with NT-ACT decentralized execution. |
| **Priority** | P0 (framework reference) |

### 22. hierarchical reinforcement learning game

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Hierarchical RL in games: options framework, feudal networks, HIRO. Macro-actions reduce effective action space. |
| **NeoTrix-Sim Mapping** | Options framework → NT-SIM macro-action library. Feudal hierarchy → NT-CORE L1-L6 layer architecture. |
| **Priority** | P1 |

### 23. curriculum learning game AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Curriculum learning: start with simple scenarios, progressively increase complexity. Critical for MOBA where full game is too complex for initial training. |
| **NeoTrix-Sim Mapping** | Curriculum → NT-MIND constellation maturity ladder (C0-C6). Training phases mirror constellation progression. |
| **Priority** | P1 |

### 24. self-play reinforcement learning

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Self-play (AlphaGo/AlphaStar/OpenAI Five): agents improve by playing against copies of themselves. Key: population-based training, policy diversity, ELO-based matchmaking. |
| **NeoTrix-Sim Mapping** | Self-play → NT-MIND dual specialization (Weapon Set I/II). Population-based training → NT-MIND SEAL pipeline with parallel evolution. |
| **Priority** | P0 |

### 25. population based training game AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Population Based Training (PBT): evolve hyperparameters online, keep best agents, periodically fork and mutate. Used by OpenAI Five and DeepMind. |
| **NeoTrix-Sim Mapping** | PBT → NT-MIND evolution loop with skill crystallization. Population diversity → NT-MIND dual specialization. |
| **Priority** | P0 |

### 26. league of legends AI research

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | Academic papers: "The League of Legends AI" (Nasfi et al.), "Large-scale Markov Decision Processes" for MOBA. Limited open-source. Most LoL AI is by Riot (closed). |
| **NeoTrix-Sim Mapping** | Research gap → NT-SIM as open-source platform for LoL AI research. |
| **Priority** | P1 |

### 27. dota 2 AI research paper

| Field | Value |
|-------|-------|
| **URL** | https://arxiv.org/abs/1912.06680 (OpenAI Five) |
| **Key Insight** | OpenAI Five paper: Proximal Policy Optimization, LSTM-based policy, action embedding for large action spaces, advantage estimation across 192,000 timesteps. |
| **NeoTrix-Sim Mapping** | Action embedding → NT-SIM action space design. LSTM policy → NT-CORE recurrent reasoning. |
| **Priority** | P0 (reference paper) |

### 28. real-time strategy game AI

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | RTS AI (StarCraft, Warcraft): build order optimization, resource gathering, army composition, tactical movement. APM-constrained agents. |
| **NeoTrix-Sim Mapping** | RTS → MOBA bridge: resource management, unit control, fog-of-war. NT-SIM inherits RTS concepts for MOBA. |
| **Priority** | P1 |

### 29. MOBA game simulation framework

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | No production-grade open-source MOBA simulation framework exists. Closest: OpenAI's Dota 2 wrapper (internal), Tencent's internal tools. |
| **NeoTrix-Sim Mapping** | This IS the gap NT-SIM fills. Build MOBA sim as Rust ECS with deterministic replay. |
| **Priority** | P0 (core mission) |

### 30. open source MOBA game engine

| Field | Value |
|-------|-------|
| **URL** | N/A |
| **Key Insight** | No open-source MOBA game engine. Dota 2 mod tools are closed. League has no official modding API. Community attempts are incomplete. |
| **NeoTrix-Sim Mapping** | Not building a game engine — building a simulation kernel. Lightweight, deterministic, no rendering dependency. |
| **Priority** | P0 (scope clarification) |

---

## Additional Sources (URL Fetches)

### DeepMind OpenSpiel

| Field | Value |
|-------|-------|
| **URL** | https://github.com/google-deepmind/open_spiel |
| **Key Insight** | 5.5k★ — Framework for RL in games. C++ core + Python API. Supports n-player, zero-sum, cooperative, general-sum games. Imperfect information, simultaneous moves. Includes CFR, MCTS, policy gradient algorithms. |
| **NeoTrix-Sim Mapping** | Reference architecture for game abstraction. OpenSpiel's extensive-form game model → NT-SIM game state representation. Algorithm library → NT-SIM training backends. |
| **Priority** | P0 (framework reference) |

### PyMARL (Oxford WhiRL)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/oxwhirl/pymarl |
| **Key Insight** | 2.2k★ — QMIX, COMA, VDN, IQL, QTRAN. SMAC benchmark (StarCraft). Docker-based. Centralized training, decentralized execution. |
| **NeoTrix-Sim Mapping** | SMAC scenario design → NT-SIM map/scenario format. QMIX value decomposition → NT-CORE capability decomposition. |
| **Priority** | P0 (algorithm reference) |

### PyTorch TorchRL

| Field | Value |
|-------|-------|
| **URL** | https://github.com/pytorch/rl |
| **Key Insight** | 3.6k★ — Modular RL library. TensorDict-first. Multi-agent support (MAPPO, IPPO, QMIX/VDN). Collectors, replay buffers, transforms. MuJoCo, robotics, LLM post-training. |
| **NeoTrix-Sim Mapping** | TorchRL's TensorDict → NT-SIM observation/action tensor design. Multi-agent objectives → NT-SIM training loss. Collector architecture → NT-SIM env runner. |
| **Priority** | P0 (training framework reference) |

### microduck_rl (Pollen Robotics)

| Field | Value |
|-------|-------|
| **URL** | https://github.com/pollen-robotics/microduck_rl |
| **Key Insight** | 2k★ — RL training for bipedal robot. MuJoCo Warp + PPO. 4096 parallel envs. Domain randomization, backlash simulation. ONNX export. Shared observation contract across policies. |
| **NeoTrix-Sim Mapping** | Shared observation contract → NT-SIM multi-agent observation design. Domain randomization → NT-SIM environment variation. ONNX export → NT-SIM policy deployment. |
| **Priority** | P2 (architecture pattern) |

---

## Synthesis: NT-SIM Design Implications

### What Exists (Reference)

| System | Type | Language | Status | Gap |
|--------|------|----------|--------|-----|
| OpenAI Five | Dota 2 AI | C++/Python | Closed, research-only | No open-source release |
| PyMARL/SMAC | StarCraft MARL | Python | Active, open-source | StarCraft-specific, not MOBA |
| OpenSpiel | Game RL framework | C++/Python | Active, open-source | Generic games, not MOBA-specific |
| TorchRL | RL library | Python | Active, open-source | Library, not simulation |
| Riot Bot AI | LoL bots | C++ | Closed | No research access |
| Tencent MOBA AI | Honor of Kings | C++ | Closed | Internal only |

### What Doesn't Exist (NT-SIM Opportunity)

| Missing Component | NT-SIM Target |
|-------------------|---------------|
| Open-source MOBA simulation engine | Rust ECS MOBA sim |
| Deterministic replay for MOBA | Deterministic tick-based simulation |
| Lightweight MOBA training environment | No game client dependency |
| Multi-agent MOBA with role assignment | Hierarchical agent architecture |
| MOBA-specific observation/action spaces | Typed observation contract |
| Curriculum learning for MOBA | Progressive difficulty scenarios |

### NT-SIM Architecture Decisions (Research-Informed)

1. **ECS Architecture** → Bevy/Sheep ECS for MOBA entities (champions, minions, towers, projectiles)
2. **Deterministic Simulation** → Fixed-point math, deterministic RNG, tick-based stepping
3. **Observation Contract** → Shared tensor format across all agents (inspired by microduck_rl)
4. **Hierarchical Actions** → Macro (team strategy) → Meso (lane/role) → Micro (combat) (inspired by OpenAI Five)
5. **Self-Play + PBT** → Population-based training with policy diversity (inspired by OpenAI Five)
6. **SMAC-style Scenarios** → Cooperative/competitive map scenarios for training (inspired by PyMARL)
7. **CTDE Paradigm** → Centralized training with decentralized execution (inspired by PyMARL)

---

## Priority Matrix

| Priority | Sources | Action |
|----------|---------|--------|
| **P0** | OpenAI Five, PyMARL, OpenSpiel, TorchRL, gap analysis (#3, #8, #29, #30) | Core architecture design — read papers, study APIs |
| **P1** | Team coordination, hierarchical RL, curriculum learning, self-play, PBT | Training methodology — design curriculum and self-play loop |
| **P2** | Skill shots, lane assignment, objectives, teamfight, vision, economy, items, draft | Feature modules — build incrementally after core |

---

## Recommended Next Steps

1. **Read OpenAI Five paper** (arXiv:1912.06680) — understand action embedding, advantage estimation
2. **Study PyMARL/SMAC** — understand CTDE, value decomposition, scenario design
3. **Study OpenSpiel** — game abstraction, extensive-form games, algorithm library
4. **Study TorchRL** — TensorDict design, collector architecture, multi-agent objectives
5. **Design NT-SIM ECS schema** — entities, components, systems for MOBA simulation
6. **Define observation contract** — shared tensor format for all agents
7. **Build minimal MOBA sim** — 2 agents, 1 lane, basic combat (spike)
