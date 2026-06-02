# AgentEvolver — NeoTrix Fusion Design

> **Created**: 2026-05-28 | **Version**: 1.0
> **Sources**: arXiv:2511.10395 (AgentEvolver Technical Report), github.com/modelscope/AgentEvolver (v1, Apache-2.0, 858 commits), modelscope.github.io/AgentEvolver (docs), emergentmind.com analysis, EvolveAgent paper
> **Status**: Design doc — awaiting user approval before implementation

---

## Overview

AgentEvolver is an **end-to-end self-evolving training framework** developed by Alibaba's Tongyi Lab (Yunpeng Zhai et al.) that unifies three synergistic mechanisms into a cohesive agent improvement pipeline:

1. **Self-Questioning** — curiosity-driven autonomous task generation via environment exploration, eliminating costly manual dataset construction
2. **Self-Navigating** — experience-guided exploration through cross-task experience reuse, improving rollout quality
3. **Self-Attributing** (ADCA) — differentiated credit assignment across trajectory steps for fine-grained policy optimization

The framework adopts a **service-oriented dataflow architecture** with standardized environment interfaces, a unified Context Manager for multi-turn interaction, and seamless integration with RL training infrastructure (veRL).

**Key differentiator**: Unlike NeoTrix's current exploration pipeline (which mines from fixed seed URLs), AgentEvolver's self-questioning *explores an environment's state-action space autonomously*, generates tasks from exploration trajectories, curates them with feasibility checks, and evaluates with an LLM-as-Judge reward — creating a closed training loop without human data.

### Relevance to NeoTrix

NeoTrix currently has:
- ✅ Exploration pipeline (seed URL → mine → absorb)
- ✅ Knowledge absorption via SelfEvolver
- ✅ GoalLoop for task orchestration
- ✅ ReasoningEngine for LLM-backed reasoning
- ✅ Reality modeling (SelfIteratingBrain, world_model)
- ❌ **No autonomous task generation from environment exploration** (P2-1)
- ❌ No experience-guided rollout
- ❌ No fine-grained trajectory credit assignment

AgentEvolver's self-questioning mechanism directly fills P2-1. Its self-navigating and self-attributing are future P2 targets.

---

## Gap Matrix

| Feature | AgentEvolver | NeoTrix | Priority | Notes |
|---------|-------------|---------|----------|-------|
| **Self-questioning (curiosity-driven task gen)** | Full — env profiles → stochastic exploration → task synthesis → curation | ❌ None | **P0** | Core P2-1 deliverable |
| **Experience reuse (self-navigating)** | Experience pool (ReMe) → retrieval → mixed rollout → selective boosting | Partial (ReasoningBank recall_similar) | P1 | Short-term: knowledge absorption only |
| **ADCA credit assignment** | Step-level LLM attribution → token-level advantage → composite reward | ❌ None | P1 | Long-horizon reward shaping |
| **Environment profiles** | Structured entity/attribute/operation summarization | ❌ None | P0 | Required for exploration |
| **LLM-as-Judge reward** | Reference-based correctness check + rubric scoring | Partial (internal evaluator) | P0 | Needed for task quality |
| **Context Manager** | Unified multi-turn context (4 templates: Causal, Reasoning, Sliding, Self) | GoalLoop + ReasoningEngine | P2 | Different abstractions |
| **Experience pool (ReMe)** | External DB for trajectory storage/retrieval | ReasoningBank + KnowledgeEngine | P1 | We use VectorDB approach |
| **Task curation pipeline** | Real-time dedup → post-generation feasibility check → distributional hybrid | ❌ None | P0 | Essential for quality |
| **Preference-guided synthesis** | Difficulty + style rubrics for task generation | GoalLoop user prompts | P0 | Direct integration |
| **GRPO-style policy optimization** | veRL-based distributed training | ❌ None | P3 | Not in current scope |

---

## P2-1: Self-Questioning Task Generation

### Design

The self-questioning mechanism in AgentEvolver answers **four core questions**:

| Question | AgentEvolver Solution | NeoTrix Adaptation |
|----------|----------------------|-------------------|
| How to understand an unknown environment? | Environment Profiles (entities + attributes + operations) | Map to `ExploreDomain` + `KnowledgeSource` |
| How to generate diverse, preference-aligned tasks? | LLM-driven stochastic exploration → preference-guided synthesis | Use `ReasoningEngine.reason()` for generation |
| How to guarantee executability? | Real-time + post-generation curation (dedup + feasibility replay) | Add `TaskCurationPipeline` |
| How to provide reward signals? | LLM Judge with reference solution comparison | Proxy via internal PerformanceEvaluator + external MCP verification |

The adapted NeoTrix pipeline:

```
Environment (ExploreDomain / external API)
    │
    ▼
1. Profile Construction [NEW]
    │  LLM summarizes environment entities, attributes, operations
    │  Output: EnvironmentProfile { entities, operations, constraints }
    │
    ▼
2. Stochastic Exploration [NEW]
    │  ReasoningEngine.high_temp_reason() → action sampling
    │  2-phase: breadth-first (N_b steps) → depth-first (N_d steps)
    │  Output: ExplorationTrajectory { states, actions, observations }
    │
    ▼
3. Adaptive Task Synthesis [NEW]
    │  Preference-guided query generation from trajectory
    │  Reference solution extraction (replay from exploration path)
    │  Output: GeneratedTask { query, reference_solution, difficulty, style }
    │
    ▼
4. Task Curation [NEW]
    │  Real-time: lexical + semantic dedup vs existing tasks
    │  Post-generation: feasibility replay check
    │  Output: CuratedTask { validated, reward_fn }
    │
    ▼
5. LLM Judge Reward [NEW]
    │  Reference-based correctness comparison
    │  Output: proxy_reward → feeds into CapabilityVector update
    │
    ▼
6. Goal Integration [NEW]
    │  enqueue to GoalLoop as AutoGoal
    │  GoalLoop.pursue_iteration() executes task
    │  Reward absorbed via SEAL loop
```

### Literature Foundation

AgentEvolver's self-questioning builds on established work:

- **Curiosity-driven exploration** (Pathak et al., 2017; Burda et al., 2018) — intrinsic reward for novel states
- **Self-play task generation** (Silver et al., 2017; Sukhbaatar et al., 2017) — agent generates curriculum from its own capability boundaries
- **STaR** (Zelikman et al., 2022) — self-taught reasoning via rationalization
- **Constitutional AI** (Bai et al., 2022) — self-critique refinement
- **CuES** (Zhai et al., 2025, extended work) — extended self-questioning with curriculum-aware synthesis

The key innovation in AgentEvolver is the **LLM-driven stochastic exploration** (high-temperature + breadth/depth 2-phase) which replaces traditional random exploration with semantically guided curiosity — uniquely feasible because LLMs bring world knowledge to the exploration process.

### Integration Points with Existing Modules

| Module | Integration | File |
|--------|------------|------|
| `ReasoningEngine` | `high_temp_reason()` variant for stochastic action sampling | `reasoning_engine/reasoning.rs` |
| `GoalLoop` | `enqueue_goal()` with generated task → `pursue_iteration()` executes | `goal_loop/loop_impl/core.rs` |
| `ExplorationPipeline` | New `ExploreDomain::SelfQuestion` variant; profile → mine → synthesize flow | `reasoning_brain/exploration_pipeline.rs` |
| `SelfIteratingBrain` | `generate_self_edit()` consumes curated task; `absorb()` with proxy reward | `self_iterating/brain_impl.rs` |
| `ReasoningBank` | Store generated tasks + reference solutions as `ReasoningMemory` | `memory.rs` |
| `CapabilityVector` | Extension dims for self_questioning, exploration_efficiency, task_diversity | `core/capability.rs` |
| `KnowledgeSource` | New `KnowledgeSource::AgentEvolver` variant | `core/knowledge/types.rs` |
| `WorldModel` | `predict_expert_performance()` uses generated task vectors | `world_model.rs` |

### Interface Design

```rust
// === Core Types (new module: reasoning_brain/self_questioning/mod.rs) ===

/// Structured summary of an environment's capabilities
pub struct EnvironmentProfile {
    pub domain: String,
    pub entities: Vec<EntityDesc>,
    pub operations: Vec<OperationDesc>,
    pub constraints: Vec<String>,
}

pub struct EntityDesc {
    pub name: String,
    pub attributes: Vec<String>,
    pub parent: Option<String>,
}

pub struct OperationDesc {
    pub name: String,
    pub input_params: Vec<String>,
    pub output_effect: String,
}

/// A single exploration trajectory
pub struct ExplorationTrajectory {
    pub env_profile: EnvironmentProfile,
    pub states: Vec<String>,
    pub actions: Vec<String>,
    pub observations: Vec<String>,
    pub breadth_phase_steps: usize,
    pub depth_phase_steps: usize,
}

/// A generated training task
pub struct GeneratedTask {
    pub id: String,
    pub query: String,
    pub reference_solution: Vec<String>,
    pub difficulty: f64,
    pub style_hints: Vec<String>,
    pub source_trajectory_id: String,
}

/// Results of task curation
pub struct CuratedTask {
    pub task: GeneratedTask,
    pub passed_dedup: bool,
    pub passed_feasibility: bool,
    pub llm_judge_score: f64,
    pub proxy_reward: f64,
}

/// The self-questioning pipeline
pub struct SelfQuestioningPipeline {
    pub brain: ReasoningBrain,
    pub bank: ReasoningBank,
    pub config: SelfQuestionConfig,
}

pub struct SelfQuestionConfig {
    pub breadth_steps: usize,           // N_b (default: 10)
    pub depth_steps: usize,             // N_d (default: 5)
    pub max_tasks_per_round: usize,     // default: 5
    pub llm_temperature: f64,           // exploration temp (default: 1.2)
    pub dedup_threshold: f64,           // cosine sim threshold (default: 0.85)
    pub min_judge_score: f64,           // task acceptance threshold (default: 0.6)
    pub auto_enqueue_goals: bool,       // auto-add to GoalLoop (default: true)
}

impl SelfQuestioningPipeline {
    /// Run one complete self-questioning cycle
    pub fn run_round(&mut self) -> NeoTrixResult<SelfQuestionRoundResult>;

    /// Phase 1: Build environment profile for a given domain
    fn build_profile(&self, domain: &ExploreDomain) -> NeoTrixResult<EnvironmentProfile>;

    /// Phase 2: Stochastic exploration with ReasoningEngine
    fn explore(&mut self, profile: &EnvironmentProfile) -> NeoTrixResult<ExplorationTrajectory>;

    /// Phase 3: Synthesize tasks from trajectory
    fn synthesize_tasks(&self, traj: &ExplorationTrajectory) -> NeoTrixResult<Vec<GeneratedTask>>;

    /// Phase 4: Curate tasks (dedup + feasibility)
    fn curate(&mut self, tasks: Vec<GeneratedTask>) -> NeoTrixResult<Vec<CuratedTask>>;

    /// Phase 5: Evaluate with LLM Judge
    fn judge(&self, task: &GeneratedTask, attempt: &str) -> NeoTrixResult<f64>;

    /// Enqueue to GoalLoop
    fn enqueue_goals(&mut self, tasks: &[CuratedTask]) -> usize;
}
```

### Data Flow Diagram

```
ReasoningEngine                 GoalLoop
    │  high_temp_reason()           ▲
    ▼                               │ enqueue/pursue
SelfQuestioningPipeline             │
    ├─ build_profile() ──→ EnvironmentProfile
    ├─ explore() ────────→ ExplorationTrajectory
    ├─ synthesize() ─────→ GeneratedTask[]
    ├─ curate() ─────────→ CuratedTask[]
    └─ judge() ──────────→ proxy_reward
                                    │
                                    ▼
                            SelfIteratingBrain
                            absorb(proxy_reward)
                                    │
                                    ▼
                            CapabilityVector.update()
```

### Minimum Viable Implementation

**Session 1 — Core Pipeline** (~200 lines new code):

| File | What | Lines |
|------|------|-------|
| `reasoning_brain/self_questioning/mod.rs` | Module definition + exports | 30 |
| `reasoning_brain/self_questioning/pipeline.rs` | `SelfQuestioningPipeline` struct + `run_round()` | 150 |
| `reasoning_brain/self_questioning/types.rs` | All type definitions | 80 |
| `reasoning_brain/self_questioning/judge.rs` | LLM Judge proxy reward | 100 |
| `reasoning_brain/goal_loop/loop_impl/pursue.rs` | Integration hook: auto-enqueue tasks | +30 |
| `reasoning_brain/exploration_pipeline.rs` | Add `ExploreDomain::SelfQuestion` | +20 |
| `core/knowledge/types.rs` | Add `KnowledgeSource::AgentEvolver` | +1 |
| `core/knowledge/sources.rs` | Vector + weight for AgentEvolver | +15 |
| `core/knowledge/vectors_group_b.rs` | `capability_vector_group_b()` match arm | +20 |

**Total**: ~450 lines net new, ~50 lines modified

### Key Implementation Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Exploration temperature | 1.2 (configurable) | High enough for diversity, low enough for coherence |
| Breadth-first steps | 10 | Balance exploration coverage vs LLM cost |
| Depth-first context window | Last 3 observations | Prevent fixation, from AgentEvolver paper |
| Dedup method | Lexical (token overlap) + Semantic (embedding cosine) | Two-stage: cheap first, expensive second |
| Feasibility verification | Replay reference solution → check execution success | Prevent hallucinated tasks |
| LLM Judge model | Same as ReasoningEngine (no extra model load) | Cost efficiency |
| Auto-enqueue to GoalLoop | Yes | Close the loop: generate → execute → absorb |

---

## Proposed KnowledgeSource

### KnowledgeSource::AgentEvolver

```rust
KnowledgeSource::AgentEvolver => ("agent-evolver/agent-evolver", {
    // Core 23-dim capability vector
    inference_depth: 0.85,   // Self-questioning requires deep env understanding
    creativity: 0.80,        // Stochastic exploration = creative action sampling
    analysis: 0.90,          // Trajectory analysis + attribution
    synthesis: 0.85,         // Task synthesis from exploration
    domain_specificity: 0.75, // General-purpose framework
    // Extension dimensions (self_questioning capabilities)
    self_questioning: 0.95,      // Core mechanism
    exploration_efficiency: 0.88, // 2-phase exploration
    task_diversity: 0.85,        // Preference-guided synthesis
    credit_assignment: 0.90,     // ADCA mechanism
    experience_reuse: 0.82,      // Self-navigating
    llm_judge_proxy: 0.80,       // LLM-as-Judge reward
}),
source_weight: 0.92,  // High priority — directly fills P2 capability gap
```

### Impact on Capability Space

The AgentEvolver KnowledgeSource primarily boosts:
- **inference_depth**: 0.85 (env profiling + trajectory analysis)
- **analysis**: 0.90 (task decomposition + attribution)
- **synthesis**: 0.85 (task generation from disparate exploration data)
- **creativity**: 0.80 (stochastic exploration, diverse task generation)

Extension dims (`extension_named`) add self-evolving dimensions:
- `self_questioning: 0.95` — tracks task generation capability
- `exploration_efficiency: 0.88` — measures coverage per LLM call

### Seed Knowledge (ReasoningBank injection)

```rust
vec![
    ReasoningMemory {
        id: "sq-001",
        content: "Self-questioning: explore env with high-temp LLM → profile entities+operations → synthesize tasks → curate with feasibility check → reward with LLM Judge",
        task_type: TaskType::Planning,
        tags: vec!["self-questioning", "exploration", "task-generation"],
    },
    ReasoningMemory {
        id: "sq-002",
        content: "Two-phase exploration: first N_b breadth steps to discover state space, then depth-first with myopic window (last N_d observations) for focused investigation",
        task_type: TaskType::Research,
        tags: vec!["exploration", "2-phase", "curiosity"],
    },
    ReasoningMemory {
        id: "sq-003",
        content: "Task curation pipeline: (1) real-time lexical dedup (token overlap), (2) semantic dedup (embedding cosine), (3) post-generation feasibility replay against environment",
        task_type: TaskType::Reflection,
        tags: vec!["curation", "dedup", "feasibility"],
    },
]
```

---

## Testing Strategy

| Test | What it validates | Type |
|------|------------------|------|
| `test_build_profile` | EnvironmentProfile correctly extracted from domain string | Unit |
| `test_exploration_2phase` | Breadth-then-depth step allocation | Unit |
| `test_task_synthesis_basic` | GeneratedTask has query + reference_solution | Unit |
| `test_dedup_lexical` | Duplicate tasks filtered by token overlap | Unit |
| `test_dedup_semantic` | Semantic similarity filter | Unit |
| `test_feasibility_check` | Unreachable tasks rejected | Unit |
| `test_llm_judge_scoring` | Judge returns 0.0–1.0 score | Unit |
| `test_full_round` | `run_round()` produces ≥0 curated tasks | Integration |
| `test_goal_auto_enqueue` | Curated tasks appear in GoalLoop queue | Integration |
| `test_knowledge_source_registered` | KnowledgeSource::AgentEvolver returns valid vector | Integration |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| LLM exploration cost (high-temp = many tokens) | High | Medium | Configurable breadth/depth steps; default conservative |
| Generated tasks hallucinate feasibility | Medium | High | Mandatory feasibility replay before acceptance |
| Task quality below manual curation | Medium | Medium | LLM Judge threshold + user preference rubrics |
| Dedup too aggressive (kills diversity) | Low | Medium | Configurable threshold; monitor diversity metric |
| Integration with GoalLoop stalls | Low | Low | `auto_enqueue_goals` can be disabled — manual approval path |
| Exploration loops infinitely | Low | High | Max steps per round + circuit breaker (reuse GoalLoop's) |
| LLM Judge bias (self-reward inflation) | Medium | Medium | External verification via MCP tools (Playwright) when available |

### Mitigation Decision Tree

```
Self-questioning round starts
    │
    ├─ LLM budget check → if < 1000 tokens remaining → skip round
    │
    ├─ Environment profile → if unknown env → build + store profile
    │
    ├─ Exploration → if 3 consecutive empty trajectories → abort round, lower temp
    │
    ├─ Task synthesis → if 5 attempts yield 0 valid → reduce difficulty rubric
    │
    ├─ Curation → if all tasks fail feasibility → log env profile as "unstable"
    │
    └─ Goal enqueue → if circuit breaker open → queue for later
```

---

## Future Work (Post P2-1)

| Feature | Phase | Dependencies |
|---------|-------|-------------|
| **Self-Navigating** (experience-guided rollout) | P2-2 | ReMe-like experience pool + retrieval |
| **ADCA-GRPO** (step-level credit assignment) | P2-3 | Composite reward construction |
| **Distributional Hybrid** (proxy + real task mix) | P2-3 | Real task availability detection |
| **Experience Stripping** (isolate action sequences) | P2-2 | Self-Navigating prerequisite |
| **Multi-env parallel exploration** | P3 | Environment service abstraction |

---

## References

1. Zhai, Y. et al. (2025). *AgentEvolver: Towards Efficient Self-Evolving Agent System*. arXiv:2511.10395. Alibaba Tongyi Lab.
2. Zhai, Y. et al. (2025). *CuES: Extended Self-Questioning Method*. arXiv:2512.01311.
3. Pathak, D. et al. (2017). *Curiosity-driven Exploration by Self-Supervised Prediction*. ICML.
4. Zelikman, E. et al. (2022). *STaR: Bootstrapping Reasoning With Reasoning*. NeurIPS.
5. Silver, D. et al. (2017). *Mastering Chess and Shogi by Self-Play with a General Reinforcement Learning Algorithm*. Science.
6. Burda, Y. et al. (2018). *Large-Scale Study of Curiosity-Driven Learning*. ICLR.
7. Bai, Y. et al. (2022). *Constitutional AI: Harmlessness from AI Feedback*. arXiv:2212.08073.
8. agentscope-ai/ReMe — Experience summarization and management (used by AgentEvolver).
