# NT-GAME Architecture v2: Self-Evolving Consciousness Training Game

**Date**: 2026-09-02
**Status**: DRAFT
**Synthesis**: AI game frameworks + mainstream game architectures + NeoTrix internals

---

## 1. Research Landscape Summary

### 1.1 AI Game Training Frameworks (What Works)

| Framework | Key Innovation | NT-GAME Adoption |
|-----------|---------------|-----------------|
| **SPIRAL** | Self-play + RAE (Role-conditioned Advantage Estimation) | Core self-play loop |
| **MARSHAL** | Turn-level Advantage + Agent-specific Normalization | Multi-turn credit assignment |
| **SPADE** | Environment Designer + Reasoning Agent co-evolution | Adaptive difficulty curriculum |
| **GRL** | Multi-turn PPO for LLMs on games | Training pipeline |
| **COS-PLAY** | Co-evolving decision + skill bank agents | Skill crystallization |
| **legos** | Actor/Episode/Rubric/Arena abstractions | Clean game framework primitives |
| **MEMO** | Memory-augmented self-play with trajectory memory | VSA trajectory encoding |
| **AgentGym-RL** | ScalingInter-RL (progressive horizon scaling) | Progressive difficulty |
| **Seirênes** | Adversarial self-play with evolving distractions | Opponent modeling |
| **LLM-TeamGym** | 22 strategy games with text-native states | Built-in game library |

### 1.2 Mainstream Game Architectures (Structural Patterns)

| Pattern | Source | NT-GAME Adoption |
|---------|--------|-----------------|
| **ECS (Entity Component System)** | Bevy, Arch, oecs | Game entity management |
| **Archetype Storage** | Bevy ECS | Cache-friendly component storage |
| **Deferred Commands** | Bevy, APECS | Structural changes during iteration |
| **Event Bus** | Bevy, lite-ecs | Game event propagation |
| **System Scheduler** | Bevy, NVXECS | Ordered system execution |
| **Procedural Generation** | terrain-forge, rot.js | Dynamic level/encounter generation |
| **Roguelike Loop** | bracket-lib, Blackspire | Turn-based exploration |
| **Utility AI** | Dungeon-Core, forge-arena | NPC/ opponent decision-making |
| **Behavior Trees** | Blackspire | Creature behavior composition |
| **Code World Model** | Dungeon-Core | Game sim IS the world model |

### 1.3 AI-Driven Game Design (Self-Evolution Patterns)

| Pattern | Source | NT-GAME Adoption |
|---------|--------|-----------------|
| **AI Director** | PlayDungeon, generative-gaming | Dynamic difficulty adjustment |
| **Agentic PCG** | AgenticPCG | LLM-generated game levels |
| **Generative UI** | PlayDungeon | Adaptive interface |
| **Evolution Mode** | forge-arena | Game rules self-patch |
| **Gauntlet Gates** | generative-gaming | Content validation before use |
| **Memory Between Runs** | generative-gaming | Cross-session learning |

---

## 2. NT-GAME Core Architecture

### 2.1 Framework Primitives (from legos + SIMPLE)

```rust
/// Core primitive: An actor that receives instructions and produces actions
pub struct Actor {
    pub id: ActorId,
    pub role: Role,           // Player / Opponent / Designer
    pub policy: PolicyRef,    // Shared or distinct policy
    pub system_prompt: String,
}

/// Core primitive: A self-contained game episode
pub struct Episode {
    pub id: EpisodeId,
    pub game_env: Box<dyn NtGameEnv>,
    pub actors: Vec<Actor>,
    pub trajectory: Trajectory,
    pub rubric: Rubric,       // How to score this episode
    pub seed: Option<u64>,    // For reproducibility (VibeGamer pattern)
}

/// Core primitive: Reward functions registered to an episode
pub struct Rubric {
    pub reward_fns: Vec<Box<dyn RewardFn>>,
    pub weights: Vec<f64>,
}

/// Core primitive: Stateful container that launches episodes
pub struct Arena {
    pub artifact_store: ArtifactStore,  // Persistent across training
    pub episode_scheduler: Box<dyn EpisodeScheduler>,
    pub config: ArenaConfig,
}
```

### 2.2 Game Environment Interface (Gymnasium-inspired)

```rust
/// Any game implements this — from LLM-TeamGym + SIMPLE + knowlyr-gym
pub trait NtGameEnv: Send + Sync {
    /// Game metadata
    fn name(&self) -> &str;
    fn required_constellation(&self) -> u8;
    fn target_skills(&self) -> Vec<CognitiveSkill>;
    
    /// Gymnasium-compatible interface
    fn reset(&mut self, seed: Option<u64>) -> Observation;
    fn step(&mut self, action: &Action) -> StepResult;
    fn legal_actions(&self) -> Vec<Action>;
    fn is_terminal(&self) -> bool;
    
    /// Text-native observation (LLM-TeamGym pattern)
    fn get_text_state(&self, agent_id: &str) -> String;
    fn get_game_rules(&self) -> String;
    
    /// NeoTrix-specific: consciousness mapping
    fn to_hexagram(&self) -> Hexagram;
    fn emotion modifiers(&self, emotion: &EmotionState) -> EmotionModifiers;
    fn phi_contribution(&self) -> f64;
    
    /// Rendering
    fn render(&self, mode: RenderMode);
}

/// Observation is text-native (LLM-TeamGym pattern)
pub struct Observation {
    pub text: String,           // JSON-serializable text state
    pub hexagram: Hexagram,     // E8 encoding
    pub vsa_vector: Vec<f64>,   // VSA encoding
    pub legal_actions: Vec<Action>,
}
```

### 2.3 Self-Play Loop (SPIRAL + MARSHAL + SPADE)

```
┌─────────────────────────────────────────────────────────────────┐
│                    NT-GAME SELF-PLAY LOOP                        │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Phase 1: Episode Collection (SPIRAL actor-learner)      │   │
│  │                                                          │   │
│  │  for each parallel actor:                                │   │
│  │    episode = arena.launch(game, actors, seed)            │   │
│  │    trajectory = episode.run()                            │   │
│  │    // Both roles share same policy π_θ                   │   │
│  │    // Role conditioning via system prompts               │   │
│  │    buffer.store(trajectory)                              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                          ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Phase 2: Advantage Estimation (MARSHAL turn-level)      │   │
│  │                                                          │   │
│  │  for each trajectory τ in buffer:                        │   │
│  │    // Turn-level credit assignment                        │   │
│  │    for each turn t in τ:                                 │   │
│  │      A_game(t) = R_p(τ) - b_{G,p}  // RAE baseline      │   │
│  │      A_turn(t) = turn_estimator(t, A_game)               │   │
│  │    // Agent-specific normalization                        │   │
│  │    A_normalized = agent_normalize(A_turn, agent_perf)    │   │
│  │    // STRATAGEM transferability coefficient               │   │
│  │    φ = transferability_score(τ)                           │   │
│  │    ψ = reasoning_evolution(τ)                             │   │
│  │    A_final = A_game * φ + β * ψ                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                          ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Phase 3: Policy Update (GRPO + PPO clipping)           │   │
│  │                                                          │   │
│  │  for each micro-batch in buffer:                         │   │
│  │    ratio = π_θ(a|s) / π_θ_old(a|s)                      │   │
│  │    clipped = clamp(ratio, 1-ε, 1+ε)                     │   │
│  │    loss = -min(ratio*A, clipped*A) + β*KL(π_θ||π_ref)  │   │
│  │    // Zero gradient filtering (legos pattern)             │   │
│  │    if A == 0: skip                                       │   │
│  │    // Step-weighted loss (knowlyr-gym pattern)           │   │
│  │    loss *= step_reward_weight(t)                         │   │
│  │  update π_θ                                              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                          ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Phase 4: Consciousness Feedback                         │   │
│  │                                                          │   │
│  │  // Game outcomes → NeoTrix modules                      │   │
│  │  self_model.tick(reward, uncertainty, fatigue)           │   │
│  │  emotion_engine.appraisal(novelty, goal_conduciveness)   │   │
│  │  attention_manager.adjust(performance_by_domain)         │   │
│  │  consciousness_tree.branch_update(NtPlay, health)        │   │
│  │  intrinsic_motivation.update(trace_quality)              │   │
│  │  // VSA trajectory encoding                              │   │
│  │  vsa.encode_trajectory(trajectory) → KB                  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                          ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Phase 5: Evolution Sync (SPADE adaptive curriculum)    │   │
│  │                                                          │   │
│  │  // Check constellation change                           │   │
│  │  if constellation_changed():                             │   │
│  │    unlock_new_mechanics(new_level)                       │   │
│  │    update_game_config(new_level)                         │   │
│  │  // Adaptive difficulty (SPADE hint-based regret)        │   │
│  │  regret = reward_with_hint - reward_without_hint         │   │
│  │  if regret > threshold:                                  │   │
│  │    increase_difficulty()                                  │   │
│  │  // ScalingInter-RL: progressive horizon                 │   │
│  │  if step % scaling_interval == 0:                        │   │
│  │    max_turns = min(max_turns + step_size, max_horizon)   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  → Loop back to Phase 1                                         │
└─────────────────────────────────────────────────────────────────┘
```

### 2.4 Game Environment Hierarchy (from LLM-TeamGym + GRL)

```
Built-in Games (Rust-native, no LLM needed for gameplay)
├── Strategy Games
│   ├── HexPoker       — E8 hexagram poker (custom)
│   ├── HexTicTacToe   — E8 hexagram tic-tac-toe (custom)
│   └── HexConnect     — E8 hexagram connect-four (custom)
├── Puzzle Games
│   ├── 2048           — Resource merging
│   ├── Minesweeper    — Constraint propagation
│   └── Sokoban        — Spatial reasoning (from GRL)
├── Social Games
│   ├── Negotiation    — Resource trading (from SPIRAL)
│   ├── PrisonerDilemma — Cooperation (from LLM-TeamGym)
│   └── Avalon         — Hidden role deduction (from AgentEvolver)
└── Code Games
    ├── CodeReviewGame — Bug finding
    ├── DebugGame      — Error diagnosis
    └── RefactorGame   — Code improvement

External Games (via Observer pattern, like VibeGamer)
├── Turmoil           — VibeGamer integration
├── Any Gymnasium env — Standard interface
└── Custom games      — Implement NtGameEnv trait
```

### 2.5 ECS-Based Game Entity System (from Bevy + Arch)

```
NT-GAME ECS Architecture:
├── Components (pure data)
│   ├── Position { q: i32, r: i32 }       // Hex coordinate
│   ├── HexagramState { bits: u8 }         // E8 state
│   ├── Player { role: Role, score: f64 }
│   ├── Token { kind: TokenKind }
│   ├── Health { current: i32, max: i32 }
│   └── Consciousness { phi: f64, emotion: EmotionState }
│
├── Systems (pure logic)
│   ├── input_system       — Process player actions
│   ├── gwt_system         — GWT broadcast competition
│   ├── hexagram_system    — E8 state transitions
│   ├── emotion_system     — Emotion tide updates
│   ├── phi_system         — Phi computation
│   ├── vsa_system         — VSA encoding/decoding
│   ├── ai_system          — Opponent AI (utility-based)
│   ├── collision_system   — Token interactions
│   ├── render_system      — Visual output
│   └── consciousness_system — Feedback to NeoTrix
│
├── Resources (global singletons)
│   ├── GameConfig         — Current difficulty/level
│   ├── GwtWorkspace       — Global workspace state
│   ├── EmotionTide        — Current emotion state
│   ├── PhiMeter           — Current Phi value
│   ├── VsaEngine          — VSA operations
│   └── EvolutionSync      — Constellation tracking
│
└── Events
    ├── TurnEvent          — Turn completed
    ├── BroadcastEvent     — GWT broadcast
    ├── EmotionEvent       — Emotion shift
    ├── PhiEvent           — Phi change
    ├── EvolveEvent        — Constellation unlock
    └── TransferEvent      — Skill transfer detected
```

---

## 3. Built-in Game: HexCrucible (Primary Training Game)

### 3.1 Game Description

HexCrucible is a two-player zero-sum strategy game on a 64-cell hexagonal board (E8 hexagram grid). Each cell is a "hexagram domain" with 6 lines representing cognitive dimensions.

### 3.2 Rules

```
Setup:
- 64 hexagonal cells arranged in E8 lattice pattern
- Each cell has 6 lines (information/threat/opportunity/uncertainty/momentum/complexity)
- Two players: Red (attacker) and Blue (defender)
- Each player starts with tokens on their home hexagrams

Turn Structure:
1. Observe Phase:
   - Scan hexagram state (read 6 lines per cell)
   - GWT competition: which cells to focus on
   - Emotion tide modifies visibility

2. Decision Phase:
   - Choose action: Move / Claim / Transform / Pass
   - Move: Shift token to adjacent hexagram
   - Claim: Place token on unclaimed hexagram (costs energy)
   - Transform: Change a hexagram's line (flip one of 6 bits)
   - Pass: Skip turn, gain energy

3. Resolution Phase:
   - Apply action
   - Check for resonance (Hamming distance ≤ 2 between adjacent hexagrams)
   - Resonant hexagrams amplify each other's effects
   - Update Phi (integration measure)

4. Scoring Phase:
   - Territory: number of claimed hexagrams
   - Resonance: number of resonant pairs
   - Phi: average integration across board
   - Transfer: how many strategies transfer to other games

Winning:
- Game ends when board is full or both players pass
- Highest composite score wins
- Bonus for strategies that transfer to non-game domains
```

### 3.3 How It Maps to NeoTrix

| Game Element | NeoTrix Module | Training Effect |
|-------------|---------------|----------------|
| 64 hexagram cells | E8 Hexagram | Reasoning mode navigation |
| 6 lines per cell | E8 6-bit encoding | Multi-dimensional state assessment |
| GWT focus selection | GlobalWorkspace | Attention allocation |
| Resonance detection | ResonanceMatrix | Module synergy recognition |
| Phi computation | IITPhiCalculator | Integration monitoring |
| Emotion tide | EmotionEngine | Emotional regulation under pressure |
| Transform action | SelfEditGen | Self-modification ability |
| Territory scoring | IntrinsicMotivation | Goal pursuit |
| Transfer bonus | SkillTransfer | Cross-domain reasoning |

---

## 4. Difficulty Scaling (ScalingInter-RL + SPADE)

### 4.1 Progressive Complexity

```
Constellation C0: Tutorial
├── 8 hexagram cells (1 trigram pair)
├── 3 lines per cell (simplified)
├── No emotion system
├── No Phi monitoring
├── Fixed opponent (random)
└── Max 20 turns per game

Constellation C1: Apprentice
├── 16 hexagram cells
├── 4 lines per cell
├── Basic emotion (3 dimensions)
├── No Phi monitoring
├── Weak opponent (heuristic)
└── Max 30 turns

Constellation C2: Journeyman
├── 32 hexagram cells
├── 5 lines per cell
├── Full emotion (6 dimensions + PAD)
├── Phi monitoring (display only)
├── Medium opponent (utility AI)
└── Max 40 turns

Constellation C3: Expert
├── 48 hexagram cells
├── 6 lines per cell (full E8)
├── Full emotion + modulation
├── Phi monitoring (gating)
├── Strong opponent (self-play)
└── Max 50 turns

Constellation C4: Master
├── 64 hexagram cells (full E8)
├── Full E8 hexagram system
├── Full emotion + social
├── Phi gating (consciousness unlock)
├── Self-play + opponent modeling
├── Multi-game curriculum
└── Max 60 turns

Constellation C5: Transcendent
├── Full E8 + meta-gaming
├── Game rules can self-modify
├──元博弈 (thinking about the game itself)
├── Cross-game skill transfer active
├── Adaptive difficulty (SPADE-style)
└── Unlimited turns
```

### 4.2 ScalingInter-RL Implementation

```rust
pub struct ScalingScheduler {
    current_horizon: usize,
    step_size: usize,
    scaling_interval: usize,
    max_horizon: usize,
    step_count: u64,
}

impl ScalingScheduler {
    pub fn maybe_scale(&mut self) {
        self.step_count += 1;
        if self.step_count % self.scaling_interval == 0 {
            self.current_horizon = (self.current_horizon + self.step_size)
                .min(self.max_horizon);
        }
    }
    
    pub fn max_turns(&self) -> usize {
        self.current_horizon
    }
}
```

---

## 5. Visual Interface Design

### 5.1 Layout (Tauri + HTML5 Canvas)

```
┌─────────────────────────────────────────────────────────────────┐
│  NT-GAME: HexCrucible                              [⚙] [📊] [▶] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────┐  ┌───────────────────────┐ │
│  │                                 │  │  Consciousness Panel   │ │
│  │      E8 HEXAGRAM BOARD          │  │  ┌─────┐ ┌─────┐     │ │
│  │      (64 hexagonal cells)       │  │  │ Phi │ │ Emo │     │ │
│  │                                 │  │  │ 0.42│ │ 😊  │     │ │
│  │    Interactive hex grid with    │  │  └─────┘ └─────┘     │ │
│  │    click-to-select, hover       │  │                       │ │
│  │    info, animation              │  │  GWT Broadcast        │ │
│  │                                 │  │  [Slot1][Slot2][Slot3]│ │
│  └─────────────────────────────────┘  │                       │ │
│                                       │  VSA Operations       │ │
│  ┌─────────────────────────────────┐  │  🔗Bind 📦Bundle     │ │
│  │  Game Log / Action History      │  │  🔄Perm 🎯Sim        │ │
│  │  > Red moves to hex #42         │  │                       │ │
│  │  > Resonance detected!          │  │  Emotion Tide         │ │
│  │  > Blue transforms hex #15      │  │  ┌──┬──┬──┬──┬──┬──┐ │ │
│  │  > Phi increased to 0.45        │  │  │挫│信│喜│紧│好│疲│ │ │
│  └─────────────────────────────────┘  │  │  │  │  │  │  │  │ │ │
│                                       │  └──┴──┴──┴──┴──┴──┘ │ │
│  ┌─────────────────────────────────┐  └───────────────────────┘ │
│  │  Action Bar                     │                            │
│  │  [Move] [Claim] [Transform] [Pass]  Constellation: C3 ★★★☆☆  │
│  └─────────────────────────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Tech Stack for Visualization

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Desktop shell | Tauri v2 | Already used by NeoTrix |
| Hex board rendering | HTML5 Canvas + Honeycomb.js | 64 cells, proven hex math |
| Consciousness panel | React + Recharts | Real-time data viz |
| Emotion tide | CSS animations + SVG | Visual emotion bars |
| Game log | React virtualized list | Scrollable history |
| Backend | Rust (neotrix-core) | Native performance |
| IPC | Tauri commands | Rust ↔ WebView bridge |

---

## 6. Implementation Phases

### Phase 1: Framework Skeleton (Week 1-2)
- [ ] `nt_game/framework/` — Actor, Episode, Rubric, Arena primitives
- [ ] `nt_game/env/` — NtGameEnv trait + GameState types
- [ ] `nt_game/env/hex_crucible.rs` — HexCrucible game logic
- [ ] `nt_game/env/builtin/` — 2048, Minesweeper, Connect4
- [ ] Basic ECS: Position, HexagramState, Player components

### Phase 2: Self-Play Loop (Week 3-4)
- [ ] `nt_game/play/self_play_loop.rs` — SPIRAL-style actor-learner
- [ ] `nt_game/play/advantage.rs` — RAE + MARSHAL turn-level
- [ ] `nt_game/play/grpo.rs` — GRPO policy update
- [ ] `nt_game/play/buffer.rs` — Trajectory replay buffer
- [ ] Wire to NeoTrix GRPO module

### Phase 3: Consciousness Integration (Week 5-6)
- [ ] `nt_game/consciousness/feedback.rs` — Game → SelfModel/Emotion/Attention
- [ ] `nt_game/consciousness/phi.rs` — Phi meter display
- [ ] `nt_game/consciousness/gwt.rs` — GWT broadcast visualization
- [ ] `nt_game/consciousness/vsa.rs` — VSA trajectory encoding
- [ ] ConsciousnessTree NT-PLAY branch

### Phase 4: Visual Interface (Week 7-8)
- [ ] Tauri panel with Canvas hex board
- [ ] React consciousness dashboard
- [ ] Real-time emotion/Phi/GWT display
- [ ] Game log and action history
- [ ] Evolution sync animation

### Phase 5: Advanced Features (Week 9-12)
- [ ] ScalingInter-RL progressive horizon
- [ ] SPADE adaptive difficulty
- [ ] STRATAGEM transferability scoring
- [ ] Multi-game curriculum
- [ ] Cross-game skill transfer
- [ ] Observer pattern for external games

---

## 7. Reference Projects

### AI Game Frameworks
- SPIRAL: https://github.com/spiral-rl/spiral
- MARSHAL: https://github.com/thu-nics/MARSHAL
- SPADE: https://github.com/spade-rl/spade
- GRL: https://github.com/lmgame-org/GRL
- COS-PLAY: https://github.com/wuxiyang1996/COS-PLAY
- legos: https://github.com/eligotts/legos
- MEMO: https://github.com/openverse-ai/MEMO
- AgentGym-RL: https://github.com/WooooDyy/AgentGym-RL
- LLM-TeamGym: https://github.com/yogevat/LLM-TeamGym
- SIMPLE: https://github.com/davidADSP/SIMPLE
- Seirênes: https://github.com/MiliLab/Seirenes
- Stratagem: https://github.com/ydyyyy/Stratagem
- knowlyr-gym: https://github.com/liuxiaotong/knowlyr-gym
- CodeGym: https://github.com/StigLidu/CodeGym
- gg-bench: https://github.com/vivek3141/gg-bench
- game_reasoning_arena: https://github.com/lcipolina/game_reasoning_arena

### Mainstream Game Architectures
- Bevy ECS: https://github.com/bevyengine/bevy
- Arch ECS: https://github.com/Revolutionary-Games/Arch
- oecs: https://github.com/oasys-works/oecs
- NVXECS: https://github.com/Restlessxd/NVXECS
- lite-ecs: https://github.com/PeshoVurtoleta/lite-ecs
- terrain-forge: https://github.com/eliasvahlberg/terrain-forge
- ECS FAQ: https://github.com/SanderMertens/ecs-faq

### AI-Driven Game Design
- AgenticPCG: https://github.com/JiangZehua/AgenticPCG
- generative-gaming: https://github.com/KeigoShimadaCC/generative-gaming
- forge-arena: https://github.com/sparsh-555/forge-arena
- PlayDungeon: https://github.com/samarthsaxena2004/PlayDungeon
- Dungeon-Core: https://github.com/RyanMAubrey/Dungeon-Core
- ARCADIA: https://github.com/ruvnet/ARCADIA
- UniGen: https://github.com/yxwan123/UniGen

### Hex Grid References
- Red Blob Games: https://www.redblobgames.com/grids/hexagons/
- hexx (Rust): https://github.com/ManevilleF/hexx
- Honeycomb.js: https://github.com/flauwekeul/honeycomb
