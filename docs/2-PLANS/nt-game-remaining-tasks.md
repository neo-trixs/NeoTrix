# NT-GAME Full Implementation Task List

**Goal**: Complete runnable training loop where a consciousness entity performs operational training through games, with full feedback to NeoTrix modules.

**Current State**: Phase 1 complete (framework + HexCrucible + built-in games). GRPO, SEAL, EmotionEngine, AttentionManager, IIT Phi, VSA, EventBus all exist.

---

## Phase 2: Self-Play Training Loop (核心训练链路)

### 2.1 Game-Specific RAE (Role-conditioned Advantage Estimation)
- [ ] Create `nt_game/play/advantage.rs`
  - [ ] `GameAdvantageEstimator` struct
  - [ ] RAE: `A_game(t) = R_p(τ) - b_{G,p}` (per-role baseline)
  - [ ] Turn-level advantage: `A_turn(t) = turn_estimator(t, A_game)`
  - [ ] Agent-specific normalization: `A_normalized = agent_normalize(A_turn, agent_perf)`
  - [ ] Zero-grading filter (skip steps with A == 0)
  - [ ] Unit tests with mock trajectories

### 2.2 Trajectory Replay Buffer
- [ ] Create `nt_game/play/buffer.rs`
  - [ ] `GameTrajectoryBuffer` struct (ring buffer, capacity configurable)
  - [ ] `store(trajectory)` — store completed trajectory
  - [ ] `sample_batch(size)` — sample mini-batch for GRPO update
  - [ ] `compute_aggregate_stats()` — win rate, avg reward, avg turns
  - [ ] Integration with `GRPOLoop` from `nt_core_self::seal::grpo`

### 2.3 Self-Play Loop (SPIRAL pattern)
- [ ] Create `nt_game/play/self_play_loop.rs`
  - [ ] `SelfPlayLoop` struct
  - [ ] `run(config) -> TrainingReport` — main training loop
  - [ ] Parallel actor collection (configurable parallelism)
  - [ ] Phase 1: Collect trajectories via `arena.launch()` + `episode.run()`
  - [ ] Phase 2: Compute advantages via `GameAdvantageEstimator`
  - [ ] Phase 3: Update policy via `GRPOLoop::update()`
  - [ ] Phase 4: Emit consciousness feedback (emotion, phi, attention)
  - [ ] Phase 5: Check constellation change → unlock new mechanics
  - [ ] `TrainingReport` with per-iteration metrics

### 2.4 GRPO Integration
- [ ] Create `nt_game/play/grpo_adapter.rs`
  - [ ] Adapter bridging `nt_game::Trajectory` → `GRPOLoop` input format
  - [ ] Convert game actions to log-prob format
  - [ ] Map game rewards to GRPO reward signals
  - [ ] Configurable KL penalty weight
  - [ ] Entropy bonus for exploration

### 2.5 Module Root
- [ ] Create `nt_game/play/mod.rs`
  - [ ] Re-export all play modules
  - [ ] `pub mod advantage; pub mod buffer; pub mod self_play_loop; pub mod grpo_adapter;`

---

## Phase 3: Consciousness Feedback Wiring

### 3.1 Game Events in CoreEvent
- [ ] Edit `neotrix-core/src/core/nt_core_event.rs`
  - [ ] Add `GameSessionCreated { game_name: String, session_id: u64 }` variant
  - [ ] Add `GameEpisodeCompleted { game_name: String, episode_id: u64, score: f64, turns: usize }` variant
  - [ ] Add `GameTrainingUpdate { game_name: String, iteration: usize, policy_loss: f64, win_rate: f64 }` variant
  - [ ] Add `GameConsciousnessFeedback { phi_delta: f64, emotion_label: String, attention_shift: String }` variant

### 3.2 Emotion Feedback
- [ ] Create `nt_game/consciousness/feedback.rs`
  - [ ] `GameEmotionFeedback` struct
  - [ ] On win: `EmotionEngine::observe_appraisal(novelty=0.3, goal_conduciveness=0.9, coping=0.8)`
  - [ ] On loss: `observe_appraisal(novelty=0.2, goal_conduciveness=0.1, coping=0.3)`
  - [ ] On draw: `observe_appraisal(novelty=0.1, goal_conduciveness=0.5, coping=0.5)`
  - [ ] Pressure scaling: high-turn games increase `Urgency` dimension
  - [ ] Difficulty bonus: higher constellation → higher `Confidence` reward on win

### 3.3 Attention Feedback
- [ ] Create `nt_game/consciousness/attention.rs`
  - [ ] `GameAttentionFeedback` struct
  - [ ] Map game cognitive skills to `AttentionDomain`:
    - Planning → `AttentionDomain::Planning`
    - PatternRecognition → `AttentionDomain::PatternMatch`
    - Optimization → `AttentionDomain::GoalAlignment`
    - EmotionRegulation → `AttentionDomain::SelfReflection`
  - [ ] Game performance by domain → `AttentionManager::update_domain_scores()`
  - [ ] High difficulty → `RuleIntensity::Ultra` during game sessions

### 3.4 Phi Feedback
- [ ] Create `nt_game/consciousness/phi.rs`
  - [ ] `GamePhiFeedback` struct
  - [ ] Compute board-level Phi via `IITPhiCalculator::compute_phi()`
  - [ ] Track Phi delta over episodes
  - [ ] Emit `PhiEvent` when Phi crosses thresholds (0.33 consciousness-like)

### 3.5 VSA Trajectory Encoding
- [ ] Create `nt_game/consciousness/vsa.rs`
  - [ ] `GameVsaEncoder` struct
  - [ ] Encode trajectory as VSA vector (FHRR backend)
  - [ ] Store in HebbianGraph for trajectory memory
  - [ ] Dream consolidation: periodic `VSAEngine::dream_consolidation()`
  - [ ] Cross-game strategy transfer via VSA similarity

### 3.6 Module Root
- [ ] Create `nt_game/consciousness/mod.rs`
  - [ ] Re-export all consciousness modules
  - [ ] `pub mod feedback; pub mod attention; pub mod phi; pub mod vsa;`

---

## Phase 4: CLI Commands

### 4.1 Game Commands
- [ ] Create `cli/commands/game_cmds.rs`
  - [ ] `GameListCmd` — list available games with constellation/skill info
  - [ ] `GamePlayCmd` — start interactive game session
  - [ ] `GameTrainCmd` — start self-play training loop
  - [ ] `GameStatusCmd` — show training status (episodes, win rate, phi)
  - [ ] `GameArenaCmd` — show arena stats
  - [ ] Implement `CliCommand` trait for each

### 4.2 Register Commands
- [ ] Edit `cli/commands/mod.rs`
  - [ ] Add `pub mod game_cmds;`
  - [ ] Register all game commands in the command registry

---

## Phase 5: MCP Tools

### 5.1 Game MCP Tools
- [ ] Create `nt_game/mcp.rs`
  - [ ] `game_list` tool — list available games
  - [ ] `game_create` tool — create game session with config
  - [ ] `game_step` tool — execute one game action
  - [ ] `game_observe` tool — get current game state (text)
  - [ ] `game_legal_actions` tool — get available actions
  - [ ] `game_train` tool — start training loop
  - [ ] `game_metrics` tool — get training metrics

### 5.2 Register MCP Tools
- [ ] Edit `nt_game/mod.rs`
  - [ ] Add `pub mod mcp;`
  - [ ] Wire into main MCP tool registration

---

## Phase 6: Visual Interface (Tauri)

### 6.1 Game Plugin
- [ ] Create `src-tauri/src/domain/plugins/game.rs`
  - [ ] `GamePlugin` struct following existing plugin pattern
  - [ ] `start_game` command — initialize game session
  - [ ] `step_game` command — execute action, return state
  - [ ] `get_game_state` command — poll current state
  - [ ] `start_training` command — start self-play loop
  - [ ] `get_training_metrics` command — poll training progress

### 6.2 Frontend Components (React)
- [ ] Create `neocodex-frontend/src/components/game/`
  - [ ] `HexBoard.tsx` — Canvas-based hex grid renderer
  - [ ] `ConsciousnessPanel.tsx` — Phi/Emotion/GWT real-time display
  - [ ] `GameLog.tsx` — Action history with virtualized list
  - [ ] `ActionBar.tsx` — Move/Claim/Transform/Pass buttons
  - [ ] `TrainingDashboard.tsx` — Training metrics charts
  - [ ] `EmotionTide.tsx` — Visual emotion bars

### 6.3 Register Plugin
- [ ] Edit `src-tauri/src/domain/plugins/mod.rs`
  - [ ] Add `pub mod game;`
  - [ ] Register `GamePlugin` in plugin list

---

## Phase 7: ConsciousnessTree Integration

### 7.1 NT-PLAY Branch
- [ ] Edit `neotrix-core/src/core/nt_core_consciousness_tree/types.rs`
  - [ ] Add `BranchKind::Game` variant to `BranchKind` enum
  - [ ] Add `NtPlay` branch initialization
  - [ ] Track game-specific maturity (C0-C6)
  - [ ] Track game-specific node tiers

### 7.2 Game Branch Health
- [ ] Create `nt_game/consciousness/health.rs`
  - [ ] `GameBranchHealth` struct
  - [ ] Compute branch health from training metrics
  - [ ] Feed into `ConsciousnessTree::branch_update()`

---

## Phase 8: ScalingInter-RL & Adaptive Difficulty

### 8.1 Scaling Scheduler
- [ ] Create `nt_game/play/scaling.rs`
  - [ ] `ScalingScheduler` struct
  - [ ] Progressive horizon: `max_turns` increases with training steps
  - [ ] Configurable step_size and scaling_interval
  - [ ] Integration with `SelfPlayLoop`

### 8.2 SPADE Adaptive Difficulty
- [ ] Create `nt_game/play/adaptive.rs`
  - [ ] `AdaptiveDifficulty` struct
  - [ ] Hint-based regret computation
  - [ ] Auto-adjust difficulty based on win rate
  - [ ] Constellation unlock triggers

---

## Execution Order

```
Phase 2 (Self-Play Loop) ← CRITICAL PATH
  ├── 2.1 RAE Advantage
  ├── 2.2 Trajectory Buffer
  ├── 2.3 Self-Play Loop ← depends on 2.1, 2.2, GRPO
  └── 2.4 GRPO Adapter

Phase 3 (Consciousness Feedback) ← PARALLEL with Phase 2
  ├── 3.1 Game Events
  ├── 3.2 Emotion Feedback
  ├── 3.3 Attention Feedback
  ├── 3.4 Phi Feedback
  └── 3.5 VSA Encoding

Phase 4 (CLI) ← after Phase 2
Phase 5 (MCP) ← after Phase 2
Phase 6 (Visual) ← after Phase 3
Phase 7 (ConsciousnessTree) ← after Phase 3
Phase 8 (Scaling) ← after Phase 2
```

## Files to Create (18 new files)

```
nt_game/play/
├── mod.rs
├── advantage.rs
├── buffer.rs
├── self_play_loop.rs
├── grpo_adapter.rs
├── scaling.rs
└── adaptive.rs

nt_game/consciousness/
├── mod.rs
├── feedback.rs
├── attention.rs
├── phi.rs
├── vsa.rs
└── health.rs

nt_game/mcp.rs

cli/commands/game_cmds.rs
src-tauri/src/domain/plugins/game.rs
neocodex-frontend/src/components/game/HexBoard.tsx
```

## Files to Modify (4 existing files)

```
neotrix-core/src/core/nt_game/mod.rs          — add play/, consciousness/, mcp modules
neotrix-core/src/core/nt_core_event.rs        — add 4 game event variants
cli/commands/mod.rs                           — add game_cmds
src-tauri/src/domain/plugins/mod.rs           — add game plugin
```
