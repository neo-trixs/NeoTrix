# RESEARCH_MOBA_V2.md — MOBA AI & microduck_rl Deep Dive for NT-WORLD-SIM

> Generated: 2026-09-11 | Sources: 40+ searches/fetches | For: NT-WORLD-SIM MOBA simulation
> V2 Enhancement: Added Honor of Kings Arena, Tencent full-MOBA, HRL for MOBA, reward shaping, curriculum learning, transfer learning, fog of war, LLM+RL

---

## Table of Contents

1. [microduck_rl Deep Analysis](#1-microduck_rl--code-structure-analysis)
2. [MOBA AI Technology Panorama](#2-moba-ai-technology-panorama)
3. [Core Technical Schemes](#3-core-technical-schemes)
4. [Cross-Cutting Patterns](#4-cross-cutting-patterns)
5. [Implementation Roadmap](#5-priority-implementation-roadmap)
6. [References](#6-references)

---

## 1. microduck_rl — Code Structure Analysis

**URL**: https://github.com/pollen-robotics/microduck_rl (2.1k★, Apache 2.0)

### Code Structure

```
src/mjlab_microduck/
├── robot/
│   ├── microduck/                    # MJCF exports, export configs, scenes, add_backlash.py
│   └── microduck_constants.py        # robot cfgs, HOME frame, BAM actuator cfg
├── actuator/friction_dr_bam.py       # BAM + friction DR + backlash encoder feedback
├── tasks/
│   ├── __init__.py                   # task registration (base + backlash variants)
│   ├── mdp.py                        # rewards, events, observations, custom classes
│   ├── backlash.py                   # make_backlash_variant() env-cfg wrapper
│   └── microduck_*_env_cfg.py        # one cfg module per task family
├── train_cli.py                      # `train` script (identical to mjlab's)
├── train_hook.py                     # intercepts `train ... --hf-jobs`
└── hf_jobs.py                        # Hugging Face Jobs submission
```

### Key Technical Details

| Aspect | Detail |
|--------|--------|
| **RL Algorithm** | PPO via rsl_rl (RSL-RL library) |
| **Physics Engine** | MuJoCo Warp (GPU-accelerated) via mjlab |
| **Control Frequency** | 50 Hz (matches real robot) |
| **Parallel Envs** | 4096 envs per training run |
| **Training Time** | ~1-2h for usable gait on single GPU |
| **Observation Dim** | 61-dim shared across ALL policies |
| **Action Dim** | 14 (servo joint position targets) |
| **Export Format** | ONNX with baked-in observation normalizer |
| **Deployment** | Rockchip RK3566 on real robot |

### Shared 61-Dim Observation Contract

The observation layout is shared across every policy, enabling runtime hot-swapping:
- **48 proprioception**: gyro(3), projected gravity(3), 14 joint pos(14), 14 joint vel(14), last action(14)
- **13 command**: twist(3), head_pose(4), body_pose(6)
- Envs that don't use a command slot **zero-pad** it rather than dropping it

### 13 Registered Tasks

| Task | Terrain | Description |
|------|---------|-------------|
| Velocity | Flat/Rough | Walking with velocity + head-pose commands |
| VelStand | Flat/Rough | Walking + fall recovery |
| StandUp | Flat/Rough | Stand from face-down/up/sitting |
| SitStand | Flat/Rough | Commanded sit ↔ stand |
| GroundPick | Flat/Rough | Crouch and touch ground with mouth |
| BallKick | Flat | Kick 70mm ball forward |
| Roulade | Flat | Forward roll over head |
| Velocity-Rollers | Flat | Roller-skate velocity tracking |
| Swizzle | Flat | Classic symmetric swizzle skating |
| RollerCrouch | Flat | Crouch while gliding |
| RollerSlope | Slope | Glide down slopes |
| RollerStandUp | Flat | Stand up onto wheels |
| Spin | Flat | Fast spin on rollers |

### Backlash Variants

Every main task has a **Backlash** twin with ±1° gear play (2° total) in each of 14 servo joints. The backlash is modeled properly for sim2real: each servo gets an unactuated `passive_<joint>_backlash` hinge, and the real encoder reads *through* the backlash.

### AGENTS.md — The Distilled Playbook

AGENTS.md (251 lines) documents:
- Environment-building workflow
- Reward-design rules learned across the project
- Designed for AI coding agents working in the repo
- Contains distilled lessons on sim2real transfer

### NeoTrix Mapping

| microduck_rl Pattern | NT-WORLD-SIM Equivalent |
|---|---|
| Shared obs contract (61-dim) | **NT-SIM Agent Observation Protocol** — standardized observation space for all MOBA agents |
| BAM actuator model (voltage control, back-EMF, friction) | **NT-SIM Agent Physics Model** — ability-specific action dynamics with cooldowns, cast times, resource costs |
| Domain randomization (per-env DR) | **NT-SIM Match Variability** — agent skill variance, meta shifts, ping simulation |
| Backlash variants (passive joints) | **NT-SIM Latency Model** — input delay, packet loss as "backlash" in agent response |
| Runtime policy hot-swapping | **NT-SIM Strategy Switching** — agents dynamically switch between lane/teamfight/objective strategies |
| Task registration system | **NT-SIM Agent Registry** — composable agent capabilities registered at startup |
| 4096 parallel envs | **NT-SIM Scalable Self-Play** — massive parallelism for training MOBA agents |
| ONNX export with baked normalizer | **NT-SIM Agent Export Pipeline** — single validated path for agent deployment |
| AGENTS.md playbook | **NT-SIM Knowledge Capture** — distilled lessons for AI coding agents |

---

## 2. MOBA AI Technology Panorama

### 2.1 OpenAI Five — Dota 2 (The Gold Standard)

**URL**: https://cdn.openai.com/dota-2.pdf | https://arxiv.org/abs/1912.06680

#### Architecture

```
Hero Obs → LSTM(1024) → Action Heads:
  ├── Available Actions → Softmax → Selected Action
  ├── Offset X/Y → Softmax → Target Position
  ├── Move X/Y → Softmax → Move Direction
  ├── Teleport Destination
  ├── Delay
  └── Unit Attention → Target Unit
```

#### Key Technical Decisions

| Decision | Detail |
|----------|--------|
| **Algorithm** | PPO (Proximal Policy Optimization) |
| **Network Core** | 4096-unit LSTM per hero |
| **Input Encoding** | Conv layers + FC layers, max-pooling over units |
| **Unit Categories** | Allied/enemy heroes, non-heroes, neutrals (5 groups) |
| **Terrain Encoding** | 8x8 grid of height, traversability, creep occupancy |
| **Ability/Item Encoding** | Embedding + FC + max-pool per ability/item |
| **Discount Factor** | γ ≈ 0.999841 (1 - 1/6300) — heavily future-weighted |
| **Backprop Window** | 16 timesteps (2.1s of game time) despite hour-long games |
| **Training Scale** | 256 GPUs, 128K CPU cores, 10 months |
| **Self-Play** | 180 years of gameplay per day |
| **Response Time** | 193ms (133ms observation + 60ms reaction) |

#### Observation Space Design

OpenAI Five processes complex multi-array observations into a single vector:
- **Hero stats**: HP, mana, regen, attack, level, abilities, items, modifiers
- **Nearby terrain**: 8x8 grid with height, traversability, creep occupancy
- **Allied/enemy heroes**: distance, orientation, health over last 12 frames, unit stats
- **Non-heroes**: health, position, unit type
- **Global state**: glyph cooldown, day/night, creep wave timing, courier status
- **Ability/item/modifier**: type embedding + stats (cooldown, charges, duration)

#### Action Space Design

| Action Head | Type | Values |
|-------------|------|--------|
| Available Actions | Discrete | Which ability/button to press |
| Offset X/Y | Discrete | Target position offset |
| Move X/Y | Discrete | Movement direction |
| Teleport Destination | Discrete | Where to teleport |
| Delay | Discrete | Timing delay |
| Unit Attention | Softmax | Which unit to target (dot-product attention) |

#### Hidden State Analysis (from 1912.06721)

- LSTM hidden states encode **implicit planning** — the agent predicts future states
- Cosine similarity between embeddings reveals structured representations
- Agent learns to predict: future locations, team objectives, enemy objectives, future rewards
- **No explicit MCTS or symbolic planning** — planning emerges from LSTM dynamics

### 2.2 Tencent Honor of Kings — Full MOBA AI (The Master)

**URL**: https://arxiv.org/abs/2011.12692 | NeurIPS 2020

#### Paper: "Towards Playing Full MOBA Games with Deep Reinforcement Learning"

**Achievement**: 97.7% win rate over 642,047 matches against top human players

#### Architecture Innovations

| Innovation | Description |
|------------|-------------|
| **Hierarchical Action Heads** | 3-level: What (ability) → Who (target) → How (direction) |
| **Multi-Head Value (MHV)** | 5 reward categories as 5 value heads (pushing, win/lose related, etc.) |
| **Curriculum Self-Play** | Start small, gradually increase hero pool and difficulty |
| **Policy Distillation** | Distill large teacher models into smaller deployable models |
| **Off-Policy Adaptation** | Adapt to new heroes via transfer from existing policies |
| **Monte-Carlo Tree Search** | MCTS for final decision-time planning |
| **Action Mask** | Game-knowledge-based pruning of invalid actions |

#### Training Infrastructure

- 320 GPUs + 35,000 CPUs
- One "resource unit" for training
- Distributed training with off-policy adaptation

#### Multi-Head Value Estimation

```
Reward Categories → 5 Value Heads:
  1. Pushing Related (turret damage, objective damage)
  2. Win/Lose Related (game outcome)
  3. Kill Related (hero kills, assists)
  4. Gold Related (CS, bounties)
  5. Survival Related (deaths, damage taken)
```

### 2.3 Tencent Honor of Kings Arena — Open RL Environment

**URL**: https://github.com/tencent-ailab/hok_env | https://arxiv.org/abs/2209.08483 | NeurIPS 2022

#### Key Features

| Feature | Detail |
|---------|--------|
| **Heroes** | 20 heroes with diverse abilities |
| **Modes** | 1v1 competitive (3v3/5v5 out of scope) |
| **Observation** | Scalar features + spatial features (mini-map) |
| **Action Space** | Same as OpenAI Five — hierarchical action heads |
| **API** | Simple Python-based interface |
| **Baseline Algorithms** | PPO + Ape-X DQN |
| **Generalization Challenge** | Different heroes = different "games" |

#### Generalization Challenge

Unlike simpler RL environments where actions remain the same across tasks, Honor of Kings Arena presents:
- **Different heroes** → different action controls (skills, attack patterns)
- **Different opponents** → need to adapt strategies
- **Sparse rewards** → only win/lose signal in competitive mode

#### Baseline Results

- Both PPO and DQN show learning progress
- Training time to beat BT (built-in trainer) is approximately constant across GPU counts
- Demonstrates that existing RL methods can learn but struggle with generalization

### 2.4 Tencent Hierarchical Macro Strategy (HMS)

**URL**: https://arxiv.org/abs/1812.07887 | AAAI 2019

#### Key Insight

- **HMS = Macro Strategy Model** — guides where to go on the map
- HMS is NOT a complete AI solution (no micro control)
- Agents make **independent strategy decisions** while communicating with allies
- **Imitated cross-agent communication** mechanism
- 5-AI team achieves **48% win rate** against top-1% human teams

#### Architecture

```
Game Features → Attention Layer → Phase Layer → Strategy Decision
                    ↓
            Cross-Agent Communication
```

- **Attention Layer**: Focuses on relevant game entities
- **Phase Layer**: Recognizes current game phase (laning, mid-game, late-game)
- **Communication**: Imitated from human replays

### 2.5 Hierarchical RL for Multi-agent MOBA (vivo AI Lab)

**URL**: https://arxiv.org/abs/1901.08004

#### Key Innovation

- **Hierarchical framework**: Macro strategies via imitation learning + micro manipulations via RL
- **Self-learning method**: Agent learns from its own past good decisions
- **Dense reward function** for multi-agent cooperation without game API
- **Multi-target detection** to extract global features from screen
- **116-dimensional state tensor** from multi-target detection, mini-map, current view

#### Training Results

- 100% win rate against bronze-level built-in AI
- Competitive multi-agent for King of Glory 5v5 mode

### 2.6 Deep Learning Bot for League of Legends

**URL**: https://github.com/csci-599-applied-ml-for-games/league-of-legends-bot | AIIDE-2020

#### Approach

| Component | Method |
|-----------|--------|
| **Object Detection** | YOLOv3 trained on 1200 manually annotated frames |
| **State Representation** | Feature vectors from YOLOv3 (15 classes) |
| **Decision Making** | LSTM for action selection |
| **Actions** | Skill use, attack, flee |
| **Training** | Against built-in LoL bot in 1v1 MidLane |

#### Limitations

- No official API → screen capture only
- Simplified to 1v1 MidLane
- Limited action space compared to full MOBA

### 2.7 MobaQA — LLM-Based MOBA Prediction

**URL**: IEEE Transactions on Games, Vol. 18, No. 2, June 2026

#### Key Innovation

- Uses **LLM fine-tuning** for battlefield information prediction
- Minimal key data items instead of extensive data inputs
- Fine-tuned LLMs (Llama-based) for win-rate and match outcome prediction
- Tested across two major game versions over 2 years

### 2.8 Think in Games — LLM + RL for MOBA Reasoning

**URL**: https://arxiv.org/abs/2508.21365

#### Key Innovation

- **GRPO (Group Relative Policy Optimization)** for training LLMs on MOBA strategies
- Rule-based reward function (no neural reward model)
- Multi-stage training: SFT → RL
- Qwen-3-14B achieves 90.91% accuracy (outperforms Deepseek-R1 at 86.67%)

---

## 3. Core Technical Schemes

### 3.1 Map Representation Methods

| Method | Description | Pros | Cons | NT-WORLD-SIM Mapping |
|--------|-------------|------|------|----------------------|
| **Tile-based** | Grid of tiles with properties (height, traversability, fog) | Simple, fast collision detection | High memory for large maps | Default for lane/jungle zones |
| **Graph-based** | Nodes (locations) + edges (paths) with weights | Natural for pathfinding, strategic reasoning | Loses spatial precision | For macro-strategy layer |
| **Vector-based** | Continuous coordinates with spatial encoding | Precise, natural for RL | High dimensionality | For micro-control layer |
| **Hierarchical** | Multi-resolution: global graph + local tiles | Best of both worlds | Complex implementation | **Recommended for NT-WORLD-SIM** |

#### OpenAI Five Map Encoding

- **8x8 grid** per hero with height, traversability, creep occupancy
- Local view, not global map
- Encoded as convolutional features

#### Tencent HMS Map Encoding

- **Mini-map features** as spatial channels
- Global view with fog of war
- Encoded via attention mechanisms

#### Recommended NT-WORLD-SIM Approach

```
Three-Layer Map Representation:
├── Layer 1: Global Graph (100 nodes) — lanes, objectives, jungle paths
│   └── Used for: macro-strategy, rotation decisions
├── Layer 2: Zone Tiles (64x64 grid) — current lane/jungle zone
│   └── Used for: positioning, wave management, vision
└── Layer 3: Combat Region (16x16 grid) — immediate combat area
    └── Used for: skillshots, dodging, focus fire
```

### 3.2 State Encoding

#### Entity Encoding (from OpenAI Five)

| Entity Type | Encoding Method | Features |
|-------------|----------------|----------|
| **Heroes** | Embedding + FC | HP, mana, level, abilities, items, position, orientation |
| **Units** | Embedding + FC | Type, HP, position |
| **Abilities** | Embedding + FC + max-pool | Cooldown, mana cost, damage, range |
| **Items** | Embedding + FC + max-pool | Stats, charges, active/passive |
| **Modifiers** | Embedding + FC + max-pool | Duration, type, effects |

#### Spatial Encoding

| Method | Description | Used By |
|--------|-------------|---------|
| **Convolutional** | Feature maps over local grid | OpenAI Five (8x8 terrain) |
| **Positional** | Absolute/relative coordinates | Most systems |
| **Attention-based** | Learnable spatial attention | HMS, AlphaStar |
| **Multi-scale** | Multiple resolution grids | StarCraft II agents |

#### Temporal Encoding

| Method | Description | Used By |
|--------|-------------|---------|
| **Frame stacking** | Stack last N frames | DQN variants |
| **LSTM/GRU** | Recurrent memory | OpenAI Five, most MOBA AI |
| **Transformer** | Self-attention over time | AlphaStar, recent work |
| **Delta encoding** | Changes from previous frame | Some StarCraft agents |

#### Recommended NT-WORLD-SIM State Encoding

```rust
struct AgentObservation {
    // Entity encoding (from microduck_rl shared contract pattern)
    hero_self: HeroState,           // 32 floats: hp, mana, level, position, cooldowns...
    nearby_allies: Vec<UnitState>,  // max 4 allies × 8 floats
    nearby_enemies: Vec<UnitState>, // max 4 enemies × 8 floats
    nearby_units: Vec<UnitState>,   // max 20 units × 4 floats
    
    // Spatial encoding
    local_map: [f32; 64*64*3],     // height, traversability, vision
    
    // Strategic encoding
    game_phase: f32,               // 0-1 normalized time
    team_advantage: f32,           // gold/xp lead
    objective_timers: [f32; 4],    // dragon, baron, rift, elder
    
    // Command encoding (from microduck_rl pattern)
    current_goal: [f32; 8],        // active strategy encoding
}
// Total: ~200 floats (manageable for LSTM/Transformer)
```

### 3.3 Action Space Design

#### Comparison of Approaches

| Approach | Description | Used By | Pros | Cons |
|----------|-------------|---------|------|------|
| **Discrete** | Fixed set of actions | OpenAI Five (partially) | Simple, sample efficient | Limited precision |
| **Continuous** | Continuous parameters | Some robot control | Precise | Hard to explore |
| **Hierarchical** | Multi-level selection | Tencent, AlphaStar | Natural for MOBA | Complex training |
| **Auto-regressive** | Sequential action components | AlphaStar | Flexible | Slow inference |

#### OpenAI Five Action Space (Hierarchical)

```
Action Selection:
1. Available Actions → Softmax → Which ability/button
2. Offset X/Y → Softmax → Target position (discretized)
3. Move X/Y → Softmax → Movement direction (discretized)
4. Teleport Destination → Discrete location
5. Delay → Discrete timing
6. Unit Attention → Softmax over all units → Target unit
```

#### Tencent Action Space (3-Level Hierarchy)

```
Level 1: What to do?
  → Move, Attack, Skill 1, Skill 2, Skill 3, Recall, Buy, ...
  
Level 2: Who/Where?
  → Target unit (attention over units) OR target position (grid)
  
Level 3: How precisely?
  → Exact offset/direction (further discretization)
```

#### Action Mask (Exploration Pruning)

Both OpenAI Five and Tencent use **action masks** to:
- Eliminate invalid actions (dead skills, no target in range)
- Guide exploration toward feasible actions
- Reduce effective action space from ~10^20000 to ~10^1500

#### Recommended NT-WORLD-SIM Action Space

```rust
enum AgentAction {
    Move { direction: Direction8, distance: f32 },
    Attack { target: UnitId },
    UseAbility { ability_id: u8, target: AbilityTarget },
    BuyItem { item_id: u8 },
    Recall,
    Ward { position: Vec2, ward_type: WardType },
    Ping { position: Vec2, ping_type: PingType },
}

enum AbilityTarget {
    Unit(UnitId),
    Position(Vec2),
    Direction(Direction8),
    Self,
    None,
}

// Action mask: compute valid actions each tick
fn compute_action_mask(state: &AgentState) -> ActionMask {
    // Based on: cooldowns, mana, range, visibility, alive status
}
```

### 3.4 Reward Function Design

#### Taxonomy of Reward Approaches

| Type | Description | Example | Pros | Cons |
|------|-------------|---------|------|------|
| **Sparse** | Win/lose only | OpenAI Five (final outcome) | Simple, aligned with goal | Very slow learning |
| **Dense** | Per-timestep rewards | CS, kills, damage | Faster learning | Can cause reward hacking |
| **Shaped** | Designed intermediate rewards | Distance to objective | Guides exploration | May not align with goal |
| **Intrinsic** | Curiosity/novelty | ICM, EXPLORS | Encourages exploration | Can be noisy |
| **Multi-head** | Decomposed rewards | Tencent MHV | Better value estimation | Complex implementation |

#### OpenAI Five Reward Shaping

```
Reward Components:
├── Win/Lose (primary)
├── Hero kills
├── Net worth lead
├── Tower damage
├── Roshan kills
├── XP advantage
└── ...
```

- **γ = 0.999841** — heavily weights future rewards
- **Backprop only 16 steps** — credit assignment over short windows
- LSTM state carries implicit long-term planning

#### Tencent Multi-Head Value (MHV)

```
5 Value Heads:
  Head 1: Pushing Related (turret damage, objective damage)
  Head 2: Win/Lose Related (game outcome)
  Head 3: Kill Related (hero kills, assists)
  Head 4: Gold Related (CS, bounties, item purchases)
  Head 5: Survival Related (deaths, damage taken)
```

- Each head has its own value estimate
- Combined via learned weights
- Inspired by Hybrid Reward Architecture (HRA) from Ms. Pac-Man

#### Dense Reward Design for MOBA

| Event | Reward | Weight |
|-------|--------|--------|
| Last-hit minion | +1 | 1.0 |
| Kill enemy hero | +5 | 5.0 |
| Assist on kill | +2 | 2.0 |
| Destroy turret | +10 | 10.0 |
| Die | -3 | -3.0 |
| Lose turret | -8 | -8.0 |
| Win game | +100 | 100.0 |
| Lose game | -100 | -100.0 |
| Vision score (ward) | +0.5 | 0.5 |
| Jungle camp clear | +1 | 1.0 |
| Dragon/Baron kill | +15 | 15.0 |

#### Advanced Reward Shaping Techniques

| Technique | Paper | Key Idea |
|-----------|-------|----------|
| **EXPLORS** | NeurIPS 2022 | Exploration-guided reward shaping for sparse rewards |
| **SORS** | Memarian et al. | Self-supervised online reward shaping via trajectory ranking |
| **ARMS** | 2025 | Automatic reward shaping for multi-agent systems |
| **SSRS** | 2025 | Semi-supervised reward shaping using zero-reward transitions |
| **Dual-Agent** | ICML 2024 | Policy agent + reward agent working together |

### 3.5 Training Architecture

#### Comparison of Training Paradigms

| Paradigm | Description | Used By | Pros | Cons |
|----------|-------------|---------|------|------|
| **Centralized** | Single model controls all | OpenAI Five | Simple coordination | Doesn't scale to many agents |
| **Decentralized** | Independent agents | Simple bots | Scalable | No coordination |
| **CTDE** | Centralized train, decentralized execute | QMIX, MAPPO | Best of both worlds | Complex infrastructure |
| **Self-Play** | Agents train against themselves | OpenAI Five, AlphaStar | No human data needed | Can cycle, not converge |
| **Population** | Multiple populations competing | AlphaStar League | Robust strategies | Very expensive |

#### OpenAI Five Training Loop

```
Training Pipeline:
1. Self-Play: Current agent vs historical versions
2. PPO Update: On batches of 2M frames every 2 seconds
3. LSTM State: Carried across timesteps within episodes
4. Continual Training: 10 months with "surgery" for game updates
5. Humanoid Opponents: Regular matches against human teams
```

#### Tencent Training Pipeline

```
Training Pipeline:
1. Curriculum Self-Play: Start with small hero pool
2. Policy Distillation: Teacher → Student model compression
3. Off-Policy Adaptation: Adapt to new heroes
4. MCTS: Decision-time planning
5. Large-Scale Deployment: 320 GPUs + 35K CPUs
```

#### CTDE (Centralized Training, Decentralized Execution) for MOBA

```
Training Phase:
├── Central Critic: Sees global state (all heroes, all info)
├── Per-Agent Actors: Each sees only their local observation
├── Value Decomposition: Global value → per-agent contributions
└── Communication Learning: What to share with allies

Execution Phase:
├── Each agent acts independently
├── No communication at execution time
└── Or: learned communication protocol
```

### 3.6 Curriculum Learning

#### Framework (from JMLR 2020 Survey)

| CL Strategy | Description | MOBA Application |
|-------------|-------------|------------------|
| **Task Sequencing** | Order tasks by difficulty | Start with 1v1, then 2v2, then 5v5 |
| **Agent Pool Expansion** | Start with few heroes, add more | Tencent's curriculum self-play |
| **Reward Shaping Progression** | Start with dense rewards, move to sparse | Start with CS rewards, end with win/lose |
| **Opponent Curriculum** | Start with weak opponents, increase | Start with bots, end with pro players |
| **State Space Curriculum** | Start with full info, add fog of war | Start without fog, gradually enable |
| **Action Space Curriculum** | Start with simplified actions | Start with move/attack, add abilities |

#### Tencent's Curriculum Self-Play

```
Phase 1: Small task, small model
  → Train on subset of heroes (easy heroes first)
  → Use smaller neural network

Phase 2: Medium task, medium model
  → Expand hero pool
  → Increase network capacity

Phase 3: Full task, full model
  → All heroes
  → Full network
  → Policy distillation for deployment

Phase 4: Off-policy adaptation
  → Adapt to new heroes without full retraining
  → Transfer from similar existing heroes
```

#### AlphaStar League Pattern

```
Population-Based Training:
├── Main Agent: The primary agent being trained
├── Historical Agents: Snapshots from training history
├── Exploiters: Agents that find weaknesses
├── Leagues: Multiple independent training runs
└── PBT: Population-based hyperparameter tuning
```

### 3.7 Transfer Learning

#### Sim-to-Real Transfer (from microduck_rl)

| Technique | Description | microduck_rl Implementation |
|-----------|-------------|----------------------------|
| **Domain Randomization** | Randomize sim parameters | Battery voltage, friction, delay, backlash |
| **Actuator Modeling** | Model real physics precisely | BAM voltage control, back-EMF, friction |
| **Observation Normalization** | Bake normalizer into ONNX | Export scripts include normalizer |
| **Policy Hot-Swapping** | Multiple policies for different behaviors | Walk/stand/recover/roulade |
| **Backlash Modeling** | Model hardware imperfections | ±1° gear play in all joints |

#### Cross-Game Transfer

| Technique | Paper | Description |
|-----------|-------|-------------|
| **Action Space Transfer** | Karttunen et al. (ICASSP 2020) | Freeze most layers, retrain last layer for new actions |
| **Dynamics Adaptation** | GARAT (NeurIPS 2020) | Adverse imitation from observation for dynamics matching |
| **Real-to-Sim-to-Real** | X-Sim (2025) | Learn from human videos → train in sim → deploy on robot |
| **Game-to-Real** | RealPlay (2025) | Transfer control from video games to real-world entities |

#### MOBA-Specific Transfer

| Transfer Type | Source → Target | Method |
|---------------|-----------------|--------|
| **Hero Transfer** | Trained hero → New hero | Off-policy adaptation + policy distillation |
| **Game Transfer** | LoL-trained → Dota-trained | Shared action space design + fine-tuning |
| **Mode Transfer** | 1v1 → 5v5 | Curriculum + additional coordination layer |
| **Meta Transfer** | Old patch → New patch | Fine-tuning on new patch data |

### 3.8 Fog of War Modeling

#### StarCraft Defogger (NeurIPS 2018)

**URL**: https://arxiv.org/abs/1812.00054

| Aspect | Detail |
|--------|--------|
| **Problem** | State estimation from partial observations |
| **Architecture** | Convolutional encoder-decoder with recurrent cells |
| **Input** | Sequence of partial observations |
| **Output** | Full game state prediction |
| **Training** | 65,000 human games of StarCraft: Brood War |
| **Applications** | Enemy unit prediction, strategy anticipation |

#### Convolutional Encoder-Decoder for Fog

```
Partial Observation → Encoder (Conv) → Latent → Decoder (Conv) → Full State Prediction
                                                    ↑
                                            Recurrent Memory
                                            (captures temporal patterns)
```

#### OpenAI Five's Approach

- **No explicit fog of war modeling** — agent sees all info in training
- Uses **LSTM memory** to implicitly model what was seen before
- **Hidden state decoders** analyze what the agent "knows" from its LSTM state

#### MOBA Fog of War Strategies

| Strategy | Description | Pros | Cons |
|----------|-------------|------|------|
| **Perfect Info Training** | Train with full info, deploy with partial | Simpler training | May not generalize |
| **Explicit Modeling** | Predict hidden state | More principled | Complex, noisy |
| **Recurrent Memory** | Let LSTM learn what to remember | End-to-end | May miss important info |
| **Belief State** | Maintain probability distribution over hidden states | Theoretically optimal | Computationally expensive |

---

## 4. Cross-Cutting Patterns

### 4.1 microduck_rl → NT-WORLD-SIM

| Pattern | microduck_rl | NT-WORLD-SIM |
|---------|-------------|--------------|
| **Shared observation contract** | 61-dim vector shared across all policies | Standardized ~200-dim observation for all MOBA agents |
| **BAM actuator model** | Voltage control + back-EMF + friction | Ability cost model + cooldowns + cast times |
| **Domain randomization** | Per-env DR on physics parameters | Per-agent skill variance, meta shifts |
| **Backlash modeling** | ±1° gear play as passive joints | Input delay, packet loss as response "backlash" |
| **Runtime policy hot-swap** | Walk/stand/recover/roulade behind shared obs | Lane/teamfight/objective strategies behind shared interface |
| **Task registration** | Composable env-cfg modules | Composable agent capability modules |
| **ONNX export** | Single safe path, baked normalizer | Single validated export pipeline |
| **AGENTS.md playbook** | Distilled reward-design lessons | Knowledge capture for AI coding agents |

### 4.2 OpenAI Five → NT-WORLD-SIM

| Pattern | OpenAI Five | NT-WORLD-SIM |
|---------|-------------|--------------|
| **LSTM core** | 4096-unit LSTM per hero | Transformer/LSTM per agent with memory |
| **Unit attention** | Dot-product attention over all units | Target selection via learned attention |
| **Action mask** | Game-knowledge pruning of invalid actions | Valid action computation each tick |
| **Self-play** | Current vs historical versions | Population-based self-play training |
| **Max-pooling** | Over unit categories | Pooling over ally/enemy unit groups |
| **Multi-modal encoding** | Conv + FC + Embedding per modality | Multi-branch encoding per observation type |

### 4.3 Tencent Full-MOBA → NT-WORLD-SIM

| Pattern | Tencent | NT-WORLD-SIM |
|---------|---------|--------------|
| **Hierarchical action heads** | What → Who → How | 3-level action selection |
| **Multi-head value** | 5 reward categories → 5 value heads | Decomposed value estimation |
| **Curriculum self-play** | Small pool → full pool | Progressive difficulty training |
| **Policy distillation** | Teacher → Student | Large model → deployable model |
| **Off-policy adaptation** | New hero from existing policies | Quick adaptation to new agents |
| **MCTS** | Decision-time planning | Lookahead for critical decisions |

### 4.4 QMIX/PyMARL → NT-WORLD-SIM

| Pattern | QMIX | NT-WORLD-SIM |
|---------|------|--------------|
| **CTDE** | Centralized critic, decentralized actors | Train with global info, execute locally |
| **Value decomposition** | Global Q → per-agent q_i | Cooperative objective decomposition |
| **Monotonic constraint** | ∂Q/∂q_i ≥ 0 for all i | Ensures individual improvement helps team |

---

## 5. Priority Implementation Roadmap for NT-WORLD-SIM

### Phase 1: Foundation (Week 1-2)

| # | Task | Source Pattern | Effort |
|---|------|---------------|--------|
| 1 | **Shared observation protocol** | microduck_rl 61-dim contract | Medium |
| 2 | **Agent physics model** | BAM actuator → ability cost/cooldown model | Medium |
| 3 | **Map representation** | 3-layer hierarchical (graph + zone tiles + combat region) | High |
| 4 | **Action space design** | Tencent 3-level hierarchy (what→who→how) | Medium |

### Phase 2: Single-Agent (Week 3-4)

| # | Task | Source Pattern | Effort |
|---|------|---------------|--------|
| 5 | **Lane phase AI** | CS, trading, wave management | Medium |
| 6 | **Teamfight positioning** | Role-based positioning rules | Medium |
| 7 | **Vision control** | Ward placement, gank detection | Medium |
| 8 | **Action mask system** | OpenAI Five / Tencent action pruning | Low |

### Phase 3: Multi-Agent Coordination (Week 5-6)

| # | Task | Source Pattern | Effort |
|---|------|---------------|--------|
| 9 | **CTDE training loop** | QMIX-style centralized training | High |
| 10 | **Objective control** | Dragon/baron decision framework | Medium |
| 11 | **Self-play training** | OpenAI Five self-play pattern | High |
| 12 | **Multi-head value estimation** | Tencent MHV for reward decomposition | Medium |

### Phase 4: Meta-AI (Week 7-8)

| # | Task | Source Pattern | Effort |
|---|------|---------------|--------|
| 13 | **Draft phase AI** | Ban/pick strategy | Medium |
| 14 | **Counter pick system** | Matchup database | Medium |
| 15 | **Team composition scoring** | Synergy evaluation | Medium |
| 16 | **Curriculum learning** | Tencent curriculum self-play | High |

### Phase 5: Advanced (Week 9-10)

| # | Task | Source Pattern | Effort |
|---|------|---------------|--------|
| 17 | **Fog of war modeling** | StarCraft Defogger + recurrent memory | High |
| 18 | **MCTS decision-time planning** | OpenAI Five + AlphaStar MCTS | High |
| 19 | **Policy distillation** | Tencent teacher→student | Medium |
| 20 | **LLM-enhanced reasoning** | Think in Games (GRPO) | High |

---

## 6. References

### Core Papers

| Paper | URL | Key Contribution | NT-WORLD-SIM Relevance |
|-------|-----|------------------|------------------------|
| OpenAI Five (1912.06680) | https://arxiv.org/abs/1912.06680 | Large-scale self-play PPO, hierarchical LSTM, 4096-unit core | Agent architecture, training infrastructure |
| OpenAI Five Planning (1912.06721) | https://arxiv.org/abs/1912.06721 | LSTM as implicit planner, hidden state decoders | Plan introspection, opponent modeling |
| Tencent Full MOBA (2011.12692) | https://arxiv.org/abs/2011.12692 | Curriculum self-play, MHV, hierarchical actions, 97.7% win rate | Complete MOBA AI paradigm |
| Tencent MOBA 1v1 (1912.09729) | https://arxiv.org/abs/1912.09729 | Control dependency decoupling, action mask, target attention, dual-clip PPO | Micro-control techniques |
| Honor of Kings Arena (2209.08483) | https://arxiv.org/abs/2209.08483 | Open-source MOBA RL environment, 20 heroes, generalization challenges | Environment design reference |
| HMS (1812.07887) | https://arxiv.org/abs/1812.07887 | Hierarchical macro strategy, cross-agent communication | Macro-strategy layer |
| HRL for MOBA (1901.08004) | https://arxiv.org/abs/1901.08004 | Hierarchical RL, imitation + RL, dense reward, no API | Screen-based training approach |

### Multi-Agent RL

| Paper | URL | Key Contribution |
|-------|-----|------------------|
| QMIX (1803.11485) | https://arxiv.org/abs/1803.11485 | Monotonic value decomposition, CTDE |
| VDN (1706.05296) | https://arxiv.org/abs/1706.05296 | Independent value decomposition |
| MAPPO | Multi-agent PPO with parameter sharing | Scalable multi-agent training |

### Reward Shaping

| Paper | URL | Key Contribution |
|-------|-----|------------------|
| EXPLORS (NeurIPS 2022) | Exploration-guided reward shaping for sparse rewards |
| SORS | Self-supervised online reward shaping via trajectory ranking |
| ARMS (2025) | Automatic reward shaping for multi-agent systems |
| SSRS (2025) | Semi-supervised reward shaping using zero-reward transitions |
| Dual-Agent (ICML 2024) | Policy agent + reward agent framework |

### Curriculum Learning

| Paper | URL | Key Contribution |
|-------|-----|------------------|
| CL Survey (JMLR 2020) | http://jmlr.org/papers/v21/20-212.html | Framework for CL in RL |
| StarCraft League (AlphaStar) | Population-based curriculum via league |
| Teacher-Student ACL (2025) | Gradient norm reward signals for curriculum |

### Transfer Learning

| Paper | URL | Key Contribution |
|-------|-----|------------------|
| Video Game to Robot (ICASSP 2020) | https://arxiv.org/abs/1905.00741 | Action space transfer via layer freezing |
| Sim-to-Real Survey (IEEE 2021) | Domain randomization, adaptation, meta-learning |
| X-Sim (2025) | Real-to-sim-to-real with object motion |
| RealPlay (2025) | Game-to-real-world control transfer |

### Fog of War

| Paper | URL | Key Contribution |
|-------|-----|------------------|
| StarCraft Defogger (NeurIPS 2018) | https://arxiv.org/abs/1812.00054 | Conv encoder-decoder for state estimation |
| Fog Prediction (AAAI 2019) | Conv encoder-decoder for fog prediction in StarCraft |

### Open Source Projects

| Project | URL | Description | Stars |
|---------|-----|-------------|-------|
| microduck_rl | https://github.com/pollen-robotics/microduck_rl | RL training for biped robot (MuJoCo + PPO) | 2.1k |
| microduck | https://github.com/pollen-robotics/microduck | Biped duck robot runtime | 7.2k |
| hok_env | https://github.com/tencent-ailab/hok_env | Honor of Kings AI environment | - |
| PyMARL | https://github.com/oxwhirl/pymarl | QMIX/VDN/COMA implementations | - |
| SMAC | https://github.com/oxwhirl/smac | StarCraft Multi-Agent Challenge | - |
| OpenSpiel | https://github.com/google-deepmind/open_spiel | Game abstraction framework | - |
| MOBA-AI-Gamer | https://github.com/adrian27513/MOBA-AI-Gamer | LoL bot with YOLOv5 + RL | - |
| Deep Learning LoL Bot | https://github.com/csci-599-applied-ml-for-games/league-of-legends-bot | LSTM-based LoL bot | - |

### Related Systems

| System | URL | Key Pattern |
|--------|-----|-------------|
| mjlab | https://github.com/mujocolab/mjlab | MuJoCo Warp training framework |
| BAM | https://github.com/Rhoban/bam | Better actuator models |
| rsl_rl | https://github.com/leggedrobotics/rsl_rl | RL library for legged robots |
| TorchRL | PyTorch RL library | TensorDict-based RL infrastructure |
| OpenSpiel | Google DeepMind | Game abstraction for RL research |

---

*End of RESEARCH_MOBA_V2.md*
