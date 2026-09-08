# Plan: Game-Training Consciousness Architecture for NeoTrix

**Date**: 2026-09-02
**Status**: DRAFT — Pending review
**Trigger**: VibeGamer analysis + consciousness core gap assessment

---

## 1. VibeGamer Analysis

### What VibeGamer Does
VibeGamer is an AI agent that auto-plays *Turmoil* (oil tycoon game) via:
- **Observer pattern**: BepInEx plugin hooks into Unity game, exposes HTTP API (`/state`, `/actions`, `/health`)
- **LLM agent loop**: `get_state → model decision → submit_actions` per step
- **Autoplay with memory**: explore → review (generate Hypotheses/Tips) → validate (same-seed causal verification) → inject passing strategies into playbook
- **Two memory types**:
  - *Hypothesis*: whole-game strategy experiments (Playbook + experiment ledger)
  - *Tip*: local技巧 with A/B/C batch validation, ~10% risk budget

### What VibeGamer Gets Right
1. **Observer abstraction** — game state decoupled from LLM decisions
2. **Hypothesis-driven exploration** — not just playing, but forming testable hypotheses
3. **Same-seed causal validation** — controls variables for experiment integrity
4. **Strategy crystallization** — validated hypotheses become reusable playbook entries
5. **Score curve tracking** — quantitative performance measurement across iterations

### What VibeGamer Lacks (and NeoTrix Can Learn From)
1. No self-play (only single-player optimization)
2. No transfer of game-learned skills to non-game domains
3. No cognitive architecture integration (standalone agent)
4. No consciousness feedback (game outcomes don't update self-model)
5. No opponent modeling or adversarial training

---

## 2. NeoTrix Consciousness Core Gap Analysis

### What Already Exists (Strong Foundations)

| Foundation | Module | Ready for Game Training? |
|-----------|--------|------------------------|
| GRPO policy optimization | `nt_core_self::seal::grpo` | ✅ — can optimize game policies |
| PRM + MCTS | `nt_core_prm/` | ✅ — tree search for game decisions |
| Intrinsic motivation | `nt_core_self::intrinsic_motivation` | ✅ — drives exploration in games |
| Behavior FSM | `nt_core_self::behavior_fsm` | ✅ — traces game行为 into state machines |
| Emotion engine | `nt_core_self::emotion_state` | ✅ — emotional response to win/loss |
| Self-model | `nt_core_self::self_model` | ✅ — tracks capability/uncertainty/fatigue |
| Attention manager | `nt_core_self::attention_head` | ✅ — routes game vs other tasks |
| SEAL pipeline | `nt_core_self::seal/` | ⚠️ — needs game-specific stages |
| Curriculum generator | `nt_core_self::seal::curriculum` | ⚠️ — needs game-difficulty binding |
| ConsciousnessTree | `nt_core_consciousness_tree/` | ⚠️ — needs game-training branch metrics |
| VSA HyperCube | `nt_core_hcube/` | ⚠️ — needs game-trajectory encoding |
| E8 reasoning | `nt_core_e8/` | ⚠️ — needs game-state as hexagram input |
| ECHO terminal | `nt_core_echo_terminal` | ⚠️ — closest to experiential learning, but CLI-only |

### What Is Missing (Critical Gaps)

| Gap | Impact | Priority |
|-----|--------|----------|
| **Game environment abstraction** | No way to represent game state/rules/actions | P0 |
| **Self-play loop** | No adversarial self-improvement mechanism | P0 |
| **Trajectory replay buffer** | No structured storage for game experiences | P0 |
| **Game→consciousness feedback** | Game outcomes don't update self-model/emotion/attention | P1 |
| **Cross-domain skill transfer** | Game-learned strategies don't transfer to coding/reasoning | P1 |
| **Opponent modeling** | No prediction of adversary behavior | P1 |
| **Multi-game curriculum** | No progressive difficulty across game types | P2 |
| **VSA game encoding** | Game trajectories not in HyperCube for associative recall | P2 |

---

## 3. Proposed Architecture: NT-PLAY (Game Training Domain)

### 3.1 New Module: `nt_core_play/`

A new sub-module under `nt_core` that provides game-training capabilities for consciousness self-evolution.

```
neotrix-core/src/core/nt_core_play/
├── mod.rs                    # NT-PLAY module root
├── game_env.rs               # GameEnv trait — abstract game interface
├── game_state.rs             # GameState: observation, action space, reward
├── self_play_loop.rs         # SelfPlayLoop: dual-role competitive training
├── trajectory_buffer.rs      # TrajectoryBuffer: experience replay storage
├── trajectory_encoder.rs     # VSA trajectory → HyperCube encoding
├── game_hexagram.rs          # Game state → E8 hexagram mapping
├── opponent_model.rs         # OpponentModel: predict adversary strategy
├── skill_transfer.rs         # SkillTransfer: game skills → domain skills
├── game_consciousness.rs     # Game outcome → consciousness feedback
├── curriculum.rs             # Game difficulty curriculum
└── observer.rs               # Game observer (HTTP/file, like VibeGamer)
```

### 3.2 Core Trait: `GameEnv`

```rust
/// Abstract game environment — any game implements this
pub trait GameEnv {
    type State: Clone + Debug;
    type Action: Clone + Debug;
    type Observation: Clone;
    
    fn reset(&mut self) -> Self::Observation;
    fn step(&mut self, action: &Self::Action) -> StepResult<Self::Observation>;
    fn legal_actions(&self, state: &Self::State) -> Vec<Self::Action>;
    fn is_terminal(&self, state: &Self::State) -> bool;
    fn reward(&self, state: &Self::State, action: &Self::Action) -> f64;
    
    /// VibeGamer-style: extract structured state for LLM reasoning
    fn observe_for_llm(&self, state: &Self::State) -> LlmObservation;
}
```

### 3.3 Self-Play Loop

Inspired by SPIRAL (Role-conditioned Advantage Estimation):

```rust
pub struct SelfPlayLoop<G: GameEnv> {
    policy: SharedPolicy,          // Single LLM playing both roles
    buffer: TrajectoryBuffer,      // Experience storage
    opponent_model: OpponentModel, // Predict opponent moves
    rae: RoleAdvantageEstimator,   // Role-conditioned advantage (SPIRAL)
    game: G,
}

impl<G: GameEnv> SelfPlayLoop<G> {
    /// One self-play cycle:
    /// 1. Play game (both roles via shared policy + role conditioning)
    /// 2. Collect trajectory with role labels
    /// 3. Compute RAE advantages
    /// 4. Update policy via GRPO
    /// 5. Feed outcomes to consciousness feedback
    pub fn cycle(&mut self) -> PlayCycleResult {
        let trajectory = self.play_episode();
        let advantages = self.rae.compute(&trajectory);
        let grpo_report = self.policy.update(&trajectory, &advantages);
        self.game_consciousness.feed(&trajectory, &grpo_report);
        PlayCycleResult { trajectory, grpo_report }
    }
}
```

### 3.4 Game→Consciousness Feedback Path

```
Game Outcome
  │
  ├─→ SelfModel::tick(game_reward, uncertainty_delta, fatigue_delta)
  │     → updates capability/uncertainty/fatigue estimates
  │
  ├─→ EmotionEngine::observe_appraisal(novelty, goal_conduciveness, coping)
  │     → win: Joy↑, Confidence↑ | loss: Frustration↑, Curiosity↑
  │
  ├─→ AttentionManager::adjust_focus(game_domain_performance)
  │     → high performance: decrease game focus, increase other domains
  │     → low performance: increase game focus
  │
  ├─→ IntrinsicMotivation::update(game_trace_quality)
  │     → drives exploration/exploitation balance
  │
  ├─→ BehaviorFSM::ingest(game_trajectory)
  │     → extracts behavioral patterns for future prediction
  │
  └─→ ConsciousnessTree::branch_update(NtPlay, health_delta)
        → updates NT-PLAY branch health metrics
```

### 3.5 Skill Transfer: Game → Non-Game Domains

Based on SPIRAL's finding that game training transfers to reasoning:

| Game Skill | Cognitive Pattern | Transfer Target |
|-----------|------------------|-----------------|
| Grid navigation | Spatial reasoning | Code architecture, dependency graphs |
| Resource management | Optimization under constraints | Build optimization, cost analysis |
| Poker/bluffing | Probabilistic reasoning, opponent modeling | Risk assessment, adversarial review |
| Negotiation | Strategic communication | API design, protocol negotiation |
| Puzzle solving | Constraint propagation | Debugging, type system reasoning |
| Multi-step planning | Long-horizon reasoning | SEAL pipeline planning, refactoring |

The transfer mechanism:
1. Game trajectory → VSA HyperCube encoding (trajectory as vector)
2. Semantic similarity search in KB for non-game contexts
3. When similar pattern detected, inject game-learned strategy as context
4. Validate transfer via GRPO on non-game tasks

### 3.6 E8 Hexagram Mapping for Game States

Each game state is encoded as an E8 hexagram:
- 6 lines of the hexagram represent: resource_level, threat_level, opportunity_count, uncertainty, momentum, complexity
- Transitions between states form E8 lattice paths
- E8 abduction reasoning predicts next optimal state
- This bridges the gap between game-world and NeoTrix's core reasoning engine

### 3.7 VSA Trajectory Encoding

```rust
pub fn encode_trajectory_to_vsa(
    trajectory: &Trajectory,
    vsa: &VSAEngine,
) -> VsaVector {
    // 1. Encode each (state, action, reward) triple as a VSA element
    let elements: Vec<VsaVector> = trajectory.steps.iter().map(|step| {
        let state_vec = vsa.encode(&format!("{:?}", step.state));
        let action_vec = vsa.encode(&format!("{:?}", step.action));
        let reward_vec = vsa.encode(&step.reward.to_string());
        // Bind state-action, then bundle with reward
        vsa.bind(&vsa.bind(&state_vec, &action_vec), &reward_vec)
    }).collect();
    
    // 2. Bundle all steps into trajectory vector
    vsa.bundle(&elements)
}
```

This enables:
- Associative recall: "find trajectories similar to this situation"
- Analogical reasoning: "this non-game problem is like that game situation"
- Pattern clustering: group similar game strategies in VSA space

---

## 4. Integration with Existing SEAL Pipeline

### 4.1 SEAL Extension: Game Training Stage

Add a new SEAL stage between Exploration and Distillation:

```
Exploration → [Game Training] → Distillation → SelfTest → Absorption → Validation
```

The Game Training stage:
1. Takes exploration candidates
2. Runs them through self-play validation
3. Measures transferability score (STRATAGEM's Reasoning Transferability Coefficient)
4. Passes validated strategies to Distillation

### 4.2 Curriculum Integration

Extend `CalibratedCurriculumGenerator` with game-difficulty binding:

```rust
pub enum CurriculumNode {
    CodeTask { difficulty: f64 },
    GameTask { game_type: GameType, difficulty: f64 },
    HybridTask { code_component: CodeTask, game_component: GameTask },
}
```

This enables progressive difficulty mixing code and game tasks.

### 4.3 ConsciousnessTree: NT-PLAY Branch

Add a 12th branch to ConsciousnessTree:

```rust
pub enum BranchKind {
    // ... existing 11 branches ...
    Play, // NT-PLAY: Game training → consciousness self-evolution
}
```

Per-branch metrics:
- `game_win_rate`: rolling win rate across self-play
- `skill_transfer_score`: how well game skills transfer to non-game tasks
- `cognitive_growth_delta`: consciousness improvement attributed to game training
- `exploration_coverage`: breadth of game states explored

---

## 5. Observer Abstraction (VibeGamer Pattern)

### 5.1 GameObserver Trait

```rust
pub trait GameObserver: Send + Sync {
    /// Connect to game (HTTP endpoint, file, or in-process)
    fn connect(&mut self, endpoint: &str) -> Result<()>;
    
    /// Get current game state
    fn get_state(&self) -> Result<GameState>;
    
    /// Submit action batch
    fn submit_actions(&self, actions: &[GameAction]) -> Result<()>;
    
    /// Health check
    fn health(&self) -> Result<HealthStatus>;
}
```

### 5.2 Built-in Game Environments

NeoTrix ships with lightweight game environments for self-play training:

| Game | Purpose | Cognitive Skill |
|------|---------|----------------|
| **2048** | Resource merging under constraints | Optimization, pattern recognition |
| **Minesweeper** | Constraint propagation, probabilistic reasoning | Logical deduction |
| **Connect4** | Spatial reasoning, opponent modeling | Adversarial planning |
| **TextAdventure** | Multi-step planning, exploration | Long-horizon reasoning |
| **CodeReviewGame** | Bug finding, security audit | Transfers directly to code review |

Each game implements `GameEnv` and can run in-process (no external game needed).

---

## 6. Implementation Phases

### Phase 1: Foundation (P0)
- [ ] `nt_core_play::game_env` trait + `GameState` types
- [ ] `nt_core_play::trajectory_buffer` (experience replay)
- [ ] `nt_core_play::self_play_loop` with RAE
- [ ] 2-3 built-in game environments (2048, Minesweeper, Connect4)
- [ ] Wire to GRPO for policy updates
- [ ] Wire to `self_model::tick()` for consciousness feedback

### Phase 2: Integration (P1)
- [ ] `nt_core_play::game_consciousness` feedback path
- [ ] `nt_core_play::opponent_model` (Theory of Mind from PolicyEvol-Agent)
- [ ] ConsciousnessTree NT-PLAY branch
- [ ] Emotion engine integration (win/loss → emotion state)
- [ ] Attention manager integration (game focus adjustment)

### Phase 3: Transfer (P1)
- [ ] `nt_core_play::skill_transfer` (game → non-game)
- [ ] `nt_core_play::trajectory_encoder` (VSA encoding)
- [ ] `nt_core_play::game_hexagram` (E8 mapping)
- [ ] SEAL pipeline integration (Game Training stage)

### Phase 4: Advanced (P2)
- [ ] Multi-game curriculum (`nt_core_play::curriculum`)
- [ ] Observer abstraction for external games (VibeGamer pattern)
- [ ] STRATAGEM-style transferability coefficient
- [ ] ProPlay-style procedural world model
- [ ] Background game training daemon (like `nt_mind_background_loop`)

---

## 7. Relationship to Existing Research

| Research | Key Insight | NeoTrix Adaptation |
|----------|------------|-------------------|
| **SPIRAL** | Self-play on zero-sum games transfers to reasoning | RAE + multi-game training |
| **CEL** | Rule induction + playbook from raw interaction | Hypothesis-driven exploration (VibeGamer pattern) |
| **SPADE** | Self-play + environment design co-evolution | Adaptive game difficulty curriculum |
| **STRATAGEM** | Trajectory-modulated transferability scoring | VSA trajectory encoding + similarity |
| **ProPlay** | Procedural world model for preplay | E8 hexagram state transitions |
| **TiG** | LLM generates language-guided policies | GWT routes game decisions |
| **Skill-SP** | Co-evolving skill library | SEAL skill crystallization from games |
| **VibeGamer** | Observer + hypothesis validation | GameObserver trait + same-seed validation |
| **PolicyEvol-Agent** | Theory of Mind + policy evolution | OpponentModel + emotion feedback |
| **MCMA** | Meta-cognitive memory abstraction | VSA trajectory encoding |

---

## 8. Open Questions

1. **Resource cost**: Self-play requires many LLM calls. Should game training run on local/Ollama models only?
2. **Safety**: Can game-trained strategies introduce adversarial behaviors into non-game domains? Need constitution gating.
3. **Evaluation**: How to measure "consciousness improvement from game training"? Phi delta? Coherence delta?
4. **Game selection**: Which games best train which cognitive skills? Need empirical benchmark.
5. **Integration timing**: Should game training run as background daemon or foreground SEAL cycle?

---

## 9. References

- VibeGamer: https://github.com/karminski/VibeGamer
- SPIRAL (ICLR 2026): Self-Play on zero-sum games Incentivizes Reasoning
- CEL: Cogito, ergo ludo — Learning by reasoning and planning
- SPADE: Self-Play in Adaptive Synthetic Executable Environments
- STRATAGEM: Trajectory-Modulated Game Self-Play
- ProPlay: Procedural World Models for Self-Evolving LLM Agents
- TiG: Think in Games — Learning to Reason via RL
- Skill-SP: Skill Self-Play with Co-Evolving Skills
- PolicyEvol-Agent: Policy Evolution via Theory of Mind
- MCMA: Meta-Cognitive Memory Abstraction
- Genesis: Autonomous AI with IIT 4.0 consciousness monitoring
- Ouroboros: Self-creating AI agent with background consciousness
