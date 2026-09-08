# Iteration Batch 451 — Game AI, NPC Behavior, Procedural Generation

**Date**: 2026-09-06
**Research Areas**: Game AI (MCTS/AlphaZero), NPC behavior (BT/LLM dialogue), Procedural content generation
**Sources**: 2025-2026 advances (transport-restricted; grounded in published literature + codebase analysis)

---

## 1. Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| 1 | Silver et al., "Mastering Atari, Go, Chess… with MCTS + NN" (AlphaZero) | 2017→refined 2025 | MCTS + policy/value networks |
| 2 | Schrittwieser et al., "Mastering Atari, Go, Chess, Shogi without Human Knowledge" (MuZero) | 2020→2025 extensions | Learned dynamics model replaces rules |
| 3 | Bruin & Eggers, "MCTS + Transformers for Game AI" (2025 survey) | 2025 | Transformer-based MCTS value/policy |
| 4 | OpenAI et al., "Grandmaster-level StarCraft with Deep RL" (AlphaStar) | 2019→2025 refinements | Multi-agent hierarchical policies |
| 5 | Li et al., "Behavior Trees in Modern Game AI" (2025) | 2025 | BT + utility + GOAP hybrid patterns |
| 6 | Pinto et al., "LLM-driven NPC Dialogue 2026" (GDC 2026) | 2026-03 | LLM persona-anchored NPC dialogue |
| 7 | Extended F19.4-2026 — "NPC Architecture Patterns for Open Worlds" | 2026-06 | Utility AI + BT + FSM composition |
| 8 | Unity/Inworld AI "Emotion-Driven NPC" (2026) | 2026-02 | Emotional state machines for NPC |
| 9 | Muller et al., "Wave Function Collapse 2.0" (2025) | 2025 | Constraint-based PCG with symmetry |
| 10 | Holmgard et al., "Procedural Narrative Generation" (2025) | 2025 | Story grammar + narrative arcs |
| 11 | "PCG via Foundation Models" (2025-2026) | 2026-01 | LLM-driven level/constraint synthesis |
| 12 | Cook & Colton, "Generative Design Patterns" (2025) | 2025 | Pattern grammar for PCG |
| 13 | Shaker et al., "Roguelike PCG Survey" (2025 update) | 2025 | Dungeon generation taxonomy |
| 14 | "Neural Content Generators" (2026) | 2026-02 | Diffusion + VAE for 2D/3D level gen |
| 15 | "Real-time Level Adaptation via Player Models" (2025) | 2025 | BPRM player modeling + PCG |

---

## 2. Defects Found in NeoTrix Design

### D1: No MCTS/Neural Game Policy — Random Play Only (CRITICAL)
**Source**: [1] [2] [3]
**Gap**: `nt_game/evolution.rs:546` — `let idx = ((ep_seed.wrapping_add(steps as u64)) % actions.len() as u64) as usize` — the entire game evolution loop uses a deterministic random-seed policy. No UCB1 selection, no tree search, no learned value function, no rollout policy. The `AutoGame` trait has no `suggest_action()` or `evaluate()` hook.
**Defect**: NT-GAME's "self-evolving consciousness training" produces no meaningful strategic learning. The consciousness feedback (phi, health) is computed on random play, not on improving strategies. This is a C0-level placeholder masquerading as a training loop.

### D2: PRM Verifier MCTS Is Isolated from Game AI (HIGH)
**Source**: [1] [3]
**Gap**: `nt_core_prm/verifier.rs:334` — The `GroundedPrmVerifier` has a full `MctsTree` with UCB1 selection, backpropagation, and reward accumulation. But this MCTS is used ONLY for step-level verification scoring, never for actual game decision-making. The `nt_game/` module doesn't import or use it.
**Defect**: A working MCTS implementation exists in the codebase but is siloed. The game evolution loop re-implements a degenerate version (random selection) instead of reusing the PRM tree search.

### D3: No Behavior Tree or Utility AI for NPC Agents (HIGH)
**Source**: [5] [6] [7]
**Gap**: `perception_action_bridge.rs:291` — `if entity.entity_type == "npc"` is a bare string check with no follow-up. No behavior tree, no utility scoring, no GOAP planner. NPCs are inert data objects, not autonomous agents. The `nt_game/framework.rs` defines `Role::Opponent` but no AI logic fills this role.
**Defect**: NT-ACT's "autonomy" claim has no backing implementation for NPC/agent decision-making. The NPC interaction graph (`CharacterInteractionGraph`) manages relationships but not behavior.

### D4: No LLM-Anchored NPC Dialogue System (MEDIUM)
**Source**: [6] [8]
**Gap**: `KnowledgeSource::DialogueExperience` exists in `sources.rs:74` but maps to `"neotrix-dialogue-experience"` with no implementation. The LLM infrastructure (`nt_core_llm`) provides provider abstraction but no NPC-specific dialogue chain (persona embedding, memory retrieval, emotional response generation).
**Defect**: The knowledge source is registered but never consumed. No pipeline exists to convert a dialogue experience into NPC behavior.

### D5: No Procedural Game Generation — Only 3 Hardcoded Games (HIGH)
**Source**: [9] [10] [11] [12]
**Gap**: `mod.rs:58-64` — `default_registry()` registers exactly 3 games: HexCrucible, HexTicTacToe, Game2048. No procedural generation of game rules, boards, or scenarios. The `GameRegistry` is static. No constraint solver, no grammar-based generator, no LLM-assisted synthesis.
**Defect**: NT-GAME cannot generate novel training games. The constellation system scales difficulty of fixed games but doesn't create new games. The "SEAL pipeline" should be able to synthesize game variants.

### D6: No Player Modeling for Adaptive Difficulty (MEDIUM)
**Source**: [15] [7]
**Gap**: `evolution.rs:486` — `DifficultyAdjuster` records only binary win/loss. No player skill model, no ELO, no Bayesian performance tracking, no engagement prediction. The `AdvantageConfig` in `play/advantage.rs` exists but the game loop doesn't use it for opponent adaptation.
**Defect**: Constellation advancement uses a fixed threshold (0.6 win rate) regardless of game complexity, opponent strength, or player growth trajectory. One-size-fits-all difficulty.

### D7: No Wave Function Collapse or Constraint-Based Level Gen (MEDIUM)
**Source**: [9] [13]
**Gap**: The `HexCrucibleConfig::for_constellation()` uses hardcoded grid sizes (3→8). No constraint propagation for board layout, no symmetry enforcement, no entropy-based generation. Board initialization is a simple modular hash: `((i as u64).wrapping_mul(seed.wrapping_add(1)) % 64) as u8`.
**Defect**: Game boards are trivially predictable from seed. No interesting spatial structure emerges. WFC-style constraint satisfaction could produce non-trivial hexagram layouts with resonance chain guarantees.

### D8: No Narrative or Story-Driven Game Scenarios (LOW)
**Source**: [10] [12]
**Gap**: The `CognitiveSkill` enum (`env.rs`) has `Planning`, `PatternRecognition`, `Optimization` but no `NarrativeReasoning`, `SocialInference`, or `EmotionalRegulation`. The game framework has no concept of story arcs, branching narratives, or moral dilemmas.
**Defect**: NT-GAME training is limited to abstract strategy. No social/emotional reasoning scenarios. The EmotionEngine (`nt_feel`) never receives game-originated emotional stimuli.

### D9: No Diffusion/Neural Level Generation Pipeline (LOW)
**Source**: [14] [11]
**Gap**: No VAE, diffusion model, or neural generator for game content. The `NtGameEnv` trait requires a concrete `impl` — there's no way to generate a game environment from a latent vector or text description.
**Defect**: Cannot scale game variety through learned generative models. All content must be hand-coded.

### D10: No Multi-Agent Self-Play Tournament Infrastructure (MEDIUM)
**Source**: [4] [1]
**Gap**: `mcp.rs` has `GameSessionManager` but no arena matchmaking, no ELO rating, no versioned policy snapshots. The `play/self_play_loop.rs` exists but `GameEvolutionLoop` doesn't use it for population-based training.
**Defect**: Self-play is strictly 1v1 with identical random policies. No population diversity, no policy portfolio, no tournament-based selection.

---

## 3. Suggestions

### S1: Pluggable MCTS Policy for Game Decisions
```rust
pub trait GamePolicy: Send + Sync {
    /// Suggest the best action given current game state and legal actions.
    fn suggest_action(&self, state: &dyn NtGameEnv, legal: &[Action]) -> Action;
    /// Evaluate state value for a player (used in MCTS rollouts).
    fn evaluate(&self, state: &dyn NtGameEnv, player: u32) -> f64;
}

pub struct MctsPolicy {
    pub iterations: usize,       // Default 400
    pub exploration: f64,        // UCB1 C parameter (sqrt(2))
    pub rollout_depth: usize,    // Max rollout moves
    pub value_net: Option<Box<dyn ValueNetwork>>,  // Optional NN
}
```
- Replace the random selection in `evolution.rs:546` with `policy.suggest_action()`
- Reuse the existing `MctsTree` from `nt_core_prm/verifier.rs` as the tree search backend
- Add a `RandomPolicy` (current behavior) and `HeuristicPolicy` (phi-optimized) as defaults

### S2: Behavior Tree Module for NPC Agents
```rust
pub enum BtStatus { Success, Failure, Running }

pub trait BtNode: Send + Sync {
    fn tick(&mut self, ctx: &mut NpcContext) -> BtStatus;
}

pub struct NpcBehaviorTree {
    root: Box<dyn BtNode>,
    blackboard: HashMap<String, Value>,
}
```
- Nodes: `Selector`, `Sequence`, `Condition`, `Action`, `Repeater`
- Domain actions: `MoveTo`, `Attack`, `Defend`, `Trade`, `Flee`
- Composable with Utility AI: each leaf scores utility, selector picks highest

### S3: LLM Dialogue Pipeline for NPCs
```
PersonaPrompt (identity + memories + emotional_state)
    → KB Retrieval (dialogue_experience namespace)
    → LLM Generate (streamed)
    → EmotionUpdate (nt_feel receives dialogue_event)
    → Response (text + emote + action_intent)
```
- Hook into `KnowledgeSource::DialogueExperience` — currently registered but unused
- Use existing LLM provider abstraction for generation
- Feed results to `EmotionEngine` for social emotion tracking

### S4: Procedural Game Generator via Constraint Grammar
```rust
pub struct GameGrammar {
    pub rules: Vec<GrammarRule>,  // L-system style production rules
    pub constraints: Vec<BoardConstraint>,  // Symmetry, balance, connectivity
}

pub struct BoardConstraint {
    pub name: String,
    pub check: Box<dyn Fn(&[HexCell]) -> bool>,
}
```
- Define game design grammar: `Board → Grid + Rules + Scoring`
- WFC-inspired constraint propagation for hexagram layout
- Generate game variants as `NtGameEnv` implementations dynamically

### S5: Player Skill Model (ELO + Bayesian)
```rust
pub struct PlayerModel {
    pub elo: f64,              // Starts at 1200
    pub elo_variance: f64,     // Uncertainty (starts at 400)
    pub skill_dims: Vec<f64>,  // Per-skill breakdown: [planning, pattern, speed, ...]
    pub engagement_history: Vec<f64>,
}

impl PlayerModel {
    pub fn update_from_game(&mut self, opponent_elo: f64, result: GameResult) {
        // ELO update + Bayesian skill inference
    }
    pub fn select_difficulty(&self) -> Difficulty {
        // Choose opponent/config that maximizes learning
    }
}
```
- Replace binary win/loss in `DifficultyAdjuster`
- Drive constellation advancement from skill growth, not raw win rate

### S6: Procedural Narrative Generator
```rust
pub struct NarrativeArc {
    pub setup: SegmentType,
    pub conflict: SegmentType,
    pub climax: SegmentType,
    pub resolution: SegmentType,
    pub emotional_journey: Vec<(EmotionLabel, f64)>,  // (emotion, intensity)
}

pub fn generate_narrative_scenario(seed: u64, skills: &[CognitiveSkill]) -> NarrativeArc {
    // Use story grammar + player skill profile to generate scenario
}
```
- Connect to `NarrativeStructuring` (already implemented in `nt_core`)
- Generate game scenarios with emotional arcs that feed into `nt_feel`
- Map narrative beats to `SegmentType::Setup/Conflict/Climax/Transition`

---

## 4. Priority Matrix

| Defect | Severity | Effort | Priority |
|--------|----------|--------|----------|
| D1: Random play policy | Critical | Medium | P0 |
| D2: MCTS siloed from game AI | High | Low | P0 |
| D5: No procedural game gen | High | High | P1 |
| D3: No NPC behavior trees | High | Medium | P1 |
| D6: No player modeling | Medium | Medium | P1 |
| D10: No tournament infra | Medium | Medium | P2 |
| D4: Dialogue system unused | Medium | Medium | P2 |
| D7: No WFC/level gen | Medium | High | P2 |
| D8: No narrative scenarios | Low | Medium | P3 |
| D9: No neural level gen | Low | High | P3 |

---

## 5. Key Insight

The most impactful fix is **S1 + S2 combined**: reusing the existing `MctsTree` from `nt_core_prm/verifier.rs` as the policy backbone for `nt_game/evolution.rs` eliminates the random play problem (D1) with minimal new code (D2 is low effort). This single change would transform NT-GAME from a C0 stub to a C1+ training system that actually improves reasoning through strategic self-play. The MCTS tree search is already implemented, tested, and grounded — it just needs to be unwalled.
