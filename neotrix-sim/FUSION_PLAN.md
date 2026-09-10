# NT-WORLD-SIM: External Technology Fusion Plan

> **Date**: 2026-09-10 | **Purpose**: Map 50+ external research findings into concrete neotrix-sim architecture changes
> **Input**: EXTERNAL_RESEARCH.md (50+ sources), RANKINGS_DATA.md (framework rankings), REFACTOR_PLAN.md (existing plan), world_sim.rs (986 lines)

---

## Executive Summary

neotrix-sim has a solid skeleton — ECS-like agent architecture, three-factor memory retrieval (recency/importance/relevance), Theory of Mind, personality drift, emotion engine, fitness landscape, and a multi-timescale tick schedule. However, the **decision pipeline is hardcoded if-else** (not goal-driven), **memory lacks hierarchical consolidation** (working→episodic→semantic), **no self-reflection loop** closes on memory, **no parallel execution** (rayon), and **no event-driven decoupling**. The external research shows these are the exact differentiators separating toy simulations (Mesa) from civilization-scale systems (Project Sid PIANO, Generative Agents). This fusion plan closes those gaps in priority order: (1) wire PlanningStack into decide_action, (2) add hierarchical memory consolidation, (3) parallelize with rayon, (4) add event-driven architecture, (5) implement dual memory (paper + vector). The result: a production-grade multi-agent simulation that scales from 10 to 1000+ agents with emergent social, economic, and cultural dynamics.

---

## 1. Gap Analysis

### 1.1 Agent Architecture Techniques

| # | External Technique | Source | Implemented? | Target Module | Complexity | Impact | Notes |
|---|-------------------|--------|:------------:|---------------|:----------:|:------:|-------|
| A1 | **Generative Agents memory stream** (recency+importance+relevance retrieval) | Stanford/Smallville | **Partial** — MemoryStream has scoring but retrieval never called from WorldSim | `agents/memory_stream.rs` → `world_sim.rs` | Moderate | **High** | `retrieve()` exists (memory_stream.rs:76) but `decide_action()` never uses it. Wire in via M1 |
| A2 | **Generative Agents planning** (LLM prompt → plan → reflection) | Stanford/Smallville | **Partial** — PlanningStack exists but `decide_action()` bypasses it with if-else | `agents/planning.rs` → `world_sim.rs:553` | Moderate | **High** | REFACTOR_PLAN M1 already identifies this. `decide_action()` at line 553 is 140 lines of hardcoded if-else |
| A3 | **PIANO hierarchical decomposition** (Goal→SubGoal→Option→Action state machines) | Project Sid | **No** — PlanningStack has flat goals, no hierarchical decomposition | `agents/planning.rs` | Hard | **High** | Needs sub-goal tree, state machine per agent |
| A4 | **ReAct loop** (Thought→Action→Observation→repeat) | ReAct paper | **Partial** — agents observe→act but no explicit Thought generation | `world_sim.rs:260-314` | Moderate | **High** | `decide_action()` returns action directly without reasoning trace |
| A5 | **BDI architecture** (Beliefs→Desires→Intentions) | JaKtA | **Partial** — PlanningStack has goals (Desires), but no Belief/Intention tracking | `agents/planning.rs` | Hard | **Medium** | BDI adds formal belief revision and commitment strategy |
| A6 | **Self-reflection** (post-action evaluation, p<0.001 improvement) | Self-Reflection paper | **Partial** — ReflectionEngine exists but only does keyword frequency analysis, not goal-outcome evaluation | `agents/reflection.rs` | Moderate | **High** | Current reflection is theme-finding, not "did this achieve my goal?" |
| A7 | **Constitutional AI / safety guardrails** | Anthropic | **Partial** — ConstitutionalFeedback exists but uses string matching (REFACTOR_PLAN M10) | `society/constitutional.rs` | Trivial | **Medium** | Already in REFACTOR_PLAN as M10. Type-safe `evaluate_action()` |
| A8 | **Theory of Mind** (hypothesis about others' beliefs) | Hypothetical Minds | **Yes** — TheoryOfMind module exists with belief models | `society/theory_of_mind.rs` | — | — | Working. No gap. |
| A9 | **LLM Archetypes** (behavioral complexity vs efficiency tradeoff) | AgentTorch paper | **No** | `agents/sim_agent.rs` | Hard | **Medium** | Agent type system: cheap/fast agents vs expensive/thorough agents |
| A10 | **Concurrent Modular Agent** (CMA — async LLM modules) | Maruyama 2025 | **No** — world_sim.rs is synchronous single-threaded | `world_sim.rs` | Hard | **Medium** | Async agent modules for parallel cognition |

### 1.2 Memory Systems

| # | External Technique | Source | Implemented? | Target Module | Complexity | Impact | Notes |
|---|-------------------|--------|:------------:|---------------|:----------:|:------:|-------|
| M1 | **Three-factor retrieval** (recency + importance + relevance) | Generative Agents | **Yes** — `MemoryStream::score()` implements this | `agents/memory_stream.rs:101` | — | — | Working. But retrieval never called from decision loop. |
| M2 | **Hierarchical memory consolidation** (working→episodic→semantic) | CraniMem, MemSTaR | **No** — MemoryStream is flat, no consolidation stages | `agents/memory_stream.rs` | Hard | **High** | CraniMem outperforms all baselines. Need 3-tier memory with promotion. |
| M3 | **Latent memory representation** (compact embedding for fast retrieval) | CraniMem | **No** — embeddings are optional `[f32; 16]`, not always populated | `agents/memory_stream.rs:41` | Moderate | **Medium** | Currently embeddings are `None` most of the time (world_sim.rs:318) |
| M4 | **Paper Memory** (persistent skill descriptions) | Project Sid | **No** — no skill/knowledge retention system | New: `agents/paper_memory.rs` | Moderate | **Medium** | Skills learned from repeated successful action patterns |
| M5 | **BM25 retrieval** (sparse, fast, no training) | BM25 paper | **Partial** — keyword scoring exists in `dual_score()` but no proper TF-IDF | `agents/memory_stream.rs:253` | Moderate | **Medium** | `keyword_score()` is Jaccard, not BM25. Upgrade for better recall. |
| M6 | **VSA HyperCube** (binding/bundling/permutation operations) | VSA Computing | **Partial** — `DualRepresentation` exists but never instantiated in tick loop | `consciousness/dual_representation.rs` | Moderate | **High** | REFACTOR_PLAN F3 already flags this. Wire in + extend operations. |
| M7 | **Memory stream scoring** (weighted sum: α×recency + β×importance + γ×relevance) | Generative Agents | **Yes** — implemented in `score()` and `dual_score()` | `agents/memory_stream.rs:101,235` | — | — | Working. |
| M8 | **Episodic + Semantic + Working memory architecture** | MemSTaR | **No** — single MemoryStream, no tier separation | `agents/memory_stream.rs` | Hard | **High** | Need WorkingMemory (current task), EpisodicMemory (events), SemanticMemory (facts) |
| M9 | **Dual memory** (Paper Memory + Vector Memory) | Project Sid | **No** — only vector/keyword memory, no skill/knowledge retention | New modules | Moderate | **Medium** | Paper Memory stores "how to do X" descriptions for skill reuse |

### 1.3 Emergent Behavior

| # | External Technique | Source | Implemented? | Target Module | Complexity | Impact | Notes |
|---|-------------------|--------|:------------:|---------------|:----------:|:------:|-------|
| E1 | **Emergent social behaviors** from ToM (cooperation, deception, negotiation) | Hypothetical Minds | **Partial** — ToM exists but social emergence not observed/measured | `consciousness/emergence_detector.rs` | Moderate | **Medium** | EmergenceDetector exists (655 lines) but never instantiated (REFACTOR_PLAN F4) |
| E2 | **Creative ideation via cognitive recombination** | Minds of Creation | **No** | NT-MIND | Hard | **Low** | Novel idea generation from combinatorial exploration |
| E3 | **Adaptive species discovery** (no a priori parameters) | Adaptive Species paper | **Partial** — Speciation exists but uses fixed thresholds | `evolution/speciation.rs` | Moderate | **Medium** | Need dynamic species count based on fitness landscape |
| E4 | **Fitness landscape evolution** (Wright-Fisher, NK model) | STUN Simulator | **Partial** — FitnessLandscape exists but simple | `evolution/fitness_landscape.rs` | Moderate | **Medium** | Upgrade to NK model with tunable ruggedness |
| E5 | **Self-organizing emergent behavior** (decentralized, no central control) | Holonic Manufacturing | **Yes** — agents are decentralized, no central controller | `world_sim.rs` | — | — | Working by design. |
| E6 | **Semantic emergence** (collective meaning-making) | Hypothetical Minds | **No** | NT-FEEL | Hard | **Low** | Social reality construction from individual cognition |

### 1.4 Social & Economic Simulation

| # | External Technique | Source | Implemented? | Target Module | Complexity | Impact | Notes |
|---|-------------------|--------|:------------:|---------------|:----------:|:------:|-------|
| S1 | **Agent-based economic modeling** (bounded rationality, emergent markets) | ABM Economics | **Partial** — Economy exists but minimal | `society/economy.rs` | Moderate | **Medium** | Need resource pricing, supply/demand emergence |
| S2 | **Multi-agent social dynamics** (hierarchies, alliances, conflicts) | Hypothetical Minds | **Partial** — RelationshipGraph exists but simple | `society/relationship_graph.rs` | Moderate | **Medium** | Need alliance formation, conflict escalation |
| S3 | **Trade networks** (emergent from local exchange decisions) | ABM Economics | **Partial** — Trade action exists but no network emergence | `society/economy.rs` | Moderate | **Medium** | Track trade graph, detect network formation |
| S4 | **Reputation systems** (trust, social status) | Reputation MAS | **No** — no reputation tracking | New: `society/reputation.rs` | Moderate | **Medium** | Aggregate social behavior → reputation score |
| S5 | **Cooperation & game theory** (ToM-driven strategic interaction) | Emergent Cooperation | **Partial** — ToM + trade exists, no game-theoretic analysis | `society/theory_of_mind.rs` | Hard | **Medium** | Prisoner's dilemma payoff tracking |
| S6 | **Cultural evolution** (transmission, diversity, fitness) | Cultural Dynamics | **Partial** — Culture exists but minimal | `society/culture.rs` | Hard | **Low** | Cultural trait transmission between agents |

### 1.5 Rust Implementation

| # | External Technique | Source | Implemented? | Target Module | Complexity | Impact | Notes |
|---|-------------------|--------|:------------:|---------------|:----------:|:------:|-------|
| R1 | **ECS architecture** (Bevy patterns: Entity/Component/System) | Bevy ECS | **Partial** — agents are Vec<SimAgent>, not true ECS | `world_sim.rs` | Hard | **High** | Need entity IDs, component storage, system functions |
| R2 | **Spatial indexing** (voxel grid + DashMap) | Voxel Grid / Pumpkin | **Partial** — SpatialGrid exists but not concurrent | `foundation/math_bridge.rs` | Moderate | **Medium** | DashMap for concurrent read/write |
| R3 | **Parallel computing** (rayon par_iter) | rayon crate | **No** — single-threaded tick loop | `world_sim.rs:207` | Moderate | **High** | `par_iter` over agents for perception+action |
| R4 | **Event-driven architecture** (3-layer: channels/entity/property) | Cosmos Engine | **Partial** — SimulationBus exists but not used for core flow | `foundation/simulation_bus.rs` | Moderate | **High** | Decouple agent decisions from world updates |
| R5 | **GPU acceleration** (wgpu compute shaders) | gpca crate | **No** — no GPU compute | `environment/gpu_grid.rs` | Hard | **Medium** | Fitness landscape evaluation on GPU |
| R6 | **Async world loading** (tokio chunk loading) | Pumpkin Server | **Partial** — tokio is dependency but not used for chunking | `Cargo.toml` | Moderate | **Medium** | Lazy chunk loading for large worlds |
| R7 | **DashMap concurrent state** | Pumpkin Server | **No** — agents stored in Vec with manual locking | `world_sim.rs:87` | Trivial | **Medium** | DashMap<AgentId, AgentState> for concurrent access |
| R8 | **Voxel world simulation** (chunk-based, block state, neighbor queries) | Pumpkin World | **Partial** — heightmap + biome exists but not chunk-based | `environment/terrain/` | Hard | **Medium** | Chunk system for scalable world |

---

## 2. Cross-Model Applicability: Universal Adapter Design

### 2.1 How Each External Model Handles Key Concerns

| Concern | Generative Agents | Project Sid (PIANO) | JaxLife | OpenLife |
|---------|-------------------|---------------------|---------|----------|
| **Agent Memory** | Memory Stream (natural language records, 3-factor retrieval, reflection synthesis) | Paper Memory (skill descriptions) + Vector Memory (episodic, goal-aware queries) | Fitness-based memory (successful strategies retained) | SDP (Semantic Distillate Protocol) — compressed semantic representations |
| **Social Dynamics** | Location-based encounters, conversation memory, relationship tracking | Social Awareness module (inferred sentiments from conversation), cooperative structures | Minimal — fitness-based selection | Individuation — agents differentiate through experience |
| **Evolution** | No explicit evolution (fixed agent pool) | Hierarchical goal decomposition + autonomous behavior emergence | Genetic algorithms on behavior programs | Open-ended evolution with novelty search |
| **Environment Interaction** | Location graph (rooms/areas), time-of-day, resource gathering | Minecraft world integration (block placement, crafting, building) | Grid world with resource dynamics | Continuous environment with physics-based interactions |

### 2.2 Universal Adapter Interface

neotrix-sim should expose a **`SimulationAdapter` trait** that abstracts over different agent architectures:

```rust
/// Universal adapter trait for pluggable agent architectures
pub trait SimulationAdapter: Send + Sync {
    /// Agent memory operations
    type Memory: AgentMemory;
    /// Agent reasoning engine
    type Reasoner: AgentReasoner;
    /// Agent social model
    type SocialModel: AgentSocialModel;

    fn create_agent(&self, id: AgentId, config: AgentConfig) -> Agent;
    fn perceive(&self, agent: &Agent, world: &WorldState) -> Perception;
    fn decide(&self, agent: &Agent, perception: &Perception, memory: &mut Self::Memory) -> AgentAction;
    fn reflect(&self, agent: &Agent, outcome: &ActionOutcome, memory: &mut Self::Memory);
}

pub trait AgentMemory {
    fn store(&mut self, record: MemoryRecord);
    fn retrieve(&self, query: &MemoryQuery, top_k: usize) -> Vec<MemoryRecord>;
    fn consolidate(&mut self); // Working → Episodic → Semantic
}

pub trait AgentReasoner {
    fn think(&self, beliefs: &Beliefs, desires: &Desires, memory: &dyn AgentMemory) -> Intention;
    fn evaluate(&self, action: &AgentAction, context: &Context) -> f32;
}

pub trait AgentSocialModel {
    fn model_other(&self, other_id: &AgentId) -> Option<&BeliefModel>;
    fn update_from_interaction(&mut self, other_id: &AgentId, interaction: &Interaction);
    fn predict_action(&self, other_id: &AgentId) -> Option<PredictedAction>;
}
```

### 2.3 Adapter Mapping

| neotrix-sim Module | Generative Agents Mapping | Project Sid Mapping | OpenLife Mapping |
|--------------------|---------------------------|---------------------|------------------|
| `MemoryStream` | Memory Stream | Paper Memory + Vector Memory | SDP Distillate |
| `PlanningStack` | Planning module | Hierarchical Goal Decomposition | — |
| `ReflectionEngine` | Reflection mechanism | — | — |
| `TheoryOfMind` | — | Social Awareness | Individuation |
| `EmotionEngine` | — | Emotion Module | — |
| `PersonalityDrift` | — | — | Individuation |
| `FitnessLandscape` | — | — | Novelty Search |
| `ConstitutionalFeedback` | — | Constitutional rules | — |

---

## 3. Fusion Priority Matrix

Score = (Impact × Feasibility) / Complexity
- Impact: Low=1, Medium=2, High=3
- Feasibility: Low=1, Medium=2, High=3
- Complexity: Hard=3, Moderate=2, Trivial=1

### P0: Must Implement (Core Capability Gap) — Score ≥ 4.0

| Priority | ID | Technique | Score | Rationale |
|:--------:|:--:|-----------|:-----:|-----------|
| **P0-1** | A2 | **Wire PlanningStack into decide_action()** | 6.0 | REFACTOR_PLAN M1. Planning exists, just not wired. Highest ROI. |
| **P0-2** | A6 | **Goal-outcome self-reflection** | 6.0 | REFACTOR_PLAN M4 + enhancement. ReflectionEngine needs "did this work?" logic. |
| **P0-3** | R3 | **Parallel agent processing (rayon)** | 6.0 | `par_iter` over agents for perception+action. Direct perf gain. |
| **P0-4** | M2 | **Hierarchical memory consolidation** | 4.5 | CraniMem proves this outperforms flat memory. 3-tier: working→episodic→semantic. |
| **P0-5** | A4 | **ReAct thought generation** | 4.5 | Add explicit Thought step before Action in decide_action(). |
| **P0-6** | R4 | **Event-driven architecture** | 4.5 | Decouple agent decisions from world state updates via SimulationBus. |
| **P0-7** | A1 | **Wire MemoryStream retrieval into decide_action()** | 4.5 | Memory exists but never queried during decisions. |

### P1: Should Implement (Significant Improvement) — Score 2.5–4.0

| Priority | ID | Technique | Score | Rationale |
|:--------:|:--:|-----------|:-----:|-----------|
| **P1-1** | M9 | **Dual memory (Paper + Vector)** | 4.0 | Skill retention for learned behaviors |
| **P1-2** | A3 | **PIANO hierarchical decomposition** | 3.0 | Goal→SubGoal→Option→Action state machine |
| **P1-3** | R7 | **DashMap concurrent agent state** | 4.0 | Replace Vec<SimAgent> with DashMap for lock-free access |
| **P1-4** | S4 | **Reputation system** | 3.0 | Aggregate social behavior → trust score |
| **P1-5** | S2 | **Social dynamics (alliances, conflicts)** | 3.0 | Extend RelationshipGraph with alliance/conflict tracking |
| **P1-6** | E3 | **Adaptive species discovery** | 3.0 | Dynamic species count, no fixed thresholds |
| **P1-7** | R2 | **DashMap spatial indexing** | 3.0 | Concurrent spatial grid for parallel queries |
| **P1-8** | M3 | **Latent memory (always-populated embeddings)** | 3.0 | Ensure all memory nodes have embeddings |
| **P1-9** | E1 | **Emergence detection** (wire EmergenceDetector) | 3.0 | REFACTOR_PLAN F4. 655 lines of dead code → activate |
| **P1-10** | S1 | **Economic modeling** (supply/demand emergence) | 3.0 | Extend Economy with price discovery |

### P2: Nice to Have (Optimization) — Score < 2.5

| Priority | ID | Technique | Score | Rationale |
|:--------:|:--:|-----------|:-----:|-----------|
| **P2-1** | A5 | **Full BDI architecture** | 2.0 | Formal belief revision + commitment strategy |
| **P2-2** | A9 | **LLM Archetypes** (agent type system) | 2.0 | Cheap/fast vs expensive/thorough agents |
| **P2-3** | A10 | **Concurrent Modular Agent** | 1.5 | Async agent cognition modules |
| **P2-4** | E2 | **Creative ideation** | 1.0 | Combinatorial idea generation |
| **P2-5** | E4 | **NK fitness landscape** | 2.0 | Tunable landscape ruggedness |
| **P2-6** | S5 | **Game-theoretic analysis** | 1.5 | Prisoner's dilemma payoff tracking |
| **P2-7** | S6 | **Cultural evolution** | 1.0 | Cultural trait transmission |
| **P2-8** | E6 | **Semantic emergence** | 1.0 | Collective meaning-making |
| **P2-9** | R5 | **GPU acceleration** (wgpu) | 1.5 | Fitness evaluation on GPU |
| **P2-10** | R8 | **Chunk-based world** | 1.5 | Scalable world loading |
| **P2-11** | M5 | **BM25 upgrade** | 2.0 | Proper TF-IDF vs current Jaccard |
| **P2-12** | R6 | **Async chunk loading** | 1.5 | Tokio-based lazy world loading |

---

## 4. Redundancy Detection

### 4.1 Duplicate Implementations

| # | Duplicate | Location A | Location B | Resolution |
|---|-----------|-----------|-----------|------------|
| D1 | **cosine_sim** | `agents/memory_stream.rs:288` | `consciousness/dual_representation.rs` (likely) | Extract to `foundation/math_bridge.rs` (REFACTOR_PLAN R2) |
| D2 | **AgentMemory vs MemoryStream** | `SimAgent.memory` (if exists) | `memory_streams: HashMap<String, MemoryStream>` | Delete AgentMemory, keep MemoryStream (REFACTOR_PLAN R1) |
| D3 | **Instruction enum vs AgentAction** | `consciousness/behavior_vm.rs` (Instruction) | `agents/sim_agent.rs` (AgentAction) | BehaviorVm returns AgentAction, delete Instruction (REFACTOR_PLAN F1) |

### 4.2 Dead Code Paths

| # | Dead Module | Lines | Location | Resolution |
|---|------------|-------|----------|------------|
| D4 | `EmergenceDetector` | 655 | `consciousness/emergence_detector.rs` | Wire into TickTier::Slow (REFACTOR_PLAN F4) |
| D5 | `BehaviorVm` | ~200 | `consciousness/behavior_vm.rs` | Convert to AgentAction orchestration layer (REFACTOR_PLAN F1) |
| D6 | `LlmHooks` | ~100 | `consciousness/llm_hooks.rs` | Keep as trait interface, optional injection (REFACTOR_PLAN F2) |
| D7 | `DualRepresentation` | ~150 | `consciousness/dual_representation.rs` | Wire into tick loop (REFACTOR_PLAN F3) |
| D8 | `MemoryStream.retrieve()` | — | `agents/memory_stream.rs:76` | Never called from WorldSim. Wire via P0-7 |
| D9 | `GraphMemory` 6/8 methods | — | `agents/graph_memory.rs` | `spread_activation()` never called. Wire via M5 in REFACTOR_PLAN |
| D10 | `SpatialMemory` 8/12 methods | — | `agents/spatial_memory.rs` | `safe_locations()` never called. Wire via M6 in REFACTOR_PLAN |
| D11 | `PlanningStack` 6/10 methods | — | `agents/planning.rs` | `next_action()` exists but decide_action bypasses it. Wire via P0-1 |
| D12 | `ReflectionEngine.reflect()` | — | `agents/reflection.rs:32` | Called from tick loop but insights have no embeddings, so retrieval is useless |
| D13 | `ActionBudget.risk_per_action()` | — | `agents/action_costs.rs` | Never read in decide_action. Wire for risk-aware decisions |
| D14 | `GlobalCoherence.diversity_index` | — | `consciousness/coherence_tracker.rs` | Never used. Connect to emotion or emergence |
| D15 | `SimAgent.skills` | — | `agents/sim_agent.rs` | Never populated. Fill during evolution from action history |

### 4.3 Overly Complex Solutions External Approaches Simplify

| # | Current Complexity | External Simplification | Savings |
|---|-------------------|------------------------|---------|
| C1 | 140-line if-else `decide_action()` | Generative Agents: goal→plan→action pipeline (30 lines + modular goals) | ~110 lines |
| C2 | Manual Vec<SimAgent> iteration with borrow conflicts | DashMap concurrent access (R7) | Eliminates `decide_action_by_id()` clone hack |
| C3 | Synchronous single-threaded tick loop | rayon `par_iter` (R3) | 4-8x speedup on multi-core |
| C4 | `cosine_sim` duplicated in 2+ files | Shared `math_bridge::cosine_sim()` | 2 file changes |
| C5 | Flat MemoryStream with no consolidation | 3-tier memory (M2) with automatic promotion | Better retrieval quality |
| C6 | ReflectionEngine keyword frequency only | Goal-outcome reflection (P0-2) | More meaningful insights |

---

## 5. Implementation Queue

### Phase 1: Decision Pipeline (Week 1) — P0 Items

```
P0-1: PlanningStack → decide_action()
  └─ Replace if-else chain with goal-driven pipeline
  └─ Ref: REFACTOR_PLAN M1, Generative Agents planning

P0-2: Goal-outcome self-reflection
  └─ Enhancement to ReflectionEngine: track goal→action→outcome
  └─ Store reflections with embeddings for retrieval

P0-5: ReAct thought generation
  └─ Add Thought step: "Given my goals and observations, I should..."
  └─ Store thought in MemoryStream as MemoryKind::Observation

P0-7: Wire MemoryStream retrieval
  └─ In decide_action(), query MemoryStream for relevant past experiences
  └─ Use retrieved memories to inform action selection
```

### Phase 2: Memory Architecture (Week 2) — P0 + P1 Items

```
P0-4: Hierarchical memory consolidation
  └─ WorkingMemory: current task state (VecDeque, capacity 10)
  └─ EpisodicMemory: past events (MemoryStream, capacity 500)
  └─ SemanticMemory: generalized knowledge (Vec<SemanticFact>)
  └─ Consolidation: episodic → (rehearsal) → semantic

P1-1: Dual memory (Paper + Vector)
  └─ PaperMemory: skill descriptions from repeated successful patterns
  └─ Wire with MemoryStream for combined retrieval

P1-8: Always-populated embeddings
  └─ Generate embedding for every MemoryNode on add()
  └─ Simple hash-based embedding if no LLM available
```

### Phase 3: Parallelism & Concurrency (Week 3) — P0 + P1 Items

```
P0-3: rayon parallel agent processing
  └─ Parallelize agent perception + decision phases
  └─ Sequential: action execution (resource mutations)

P0-6: Event-driven architecture
  └─ SimulationBus for agent→world communication
  └─ Agents emit events, world processes them in batch

P1-3: DashMap concurrent agent state
  └─ Replace Vec<SimAgent> with DashMap<AgentId, AgentState>
  └─ Eliminates borrow conflicts in decide_action_by_id()

P1-7: DashMap spatial indexing
  └─ Concurrent spatial grid for parallel neighbor queries
```

### Phase 4: Social & Economic (Week 4) — P1 Items

```
P1-4: Reputation system
  └─ Track cooperation/defection history per agent pair
  └─ Aggregate into reputation score (0.0-1.0)

P1-5: Social dynamics
  └─ Alliance formation (mutual high-cooperation pairs)
  └─ Conflict escalation (repeated threat interactions)

P1-10: Economic modeling
  └─ Resource pricing from supply/demand
  └─ Trade network emergence tracking

P1-9: Emergence detection (activate EmergenceDetector)
  └─ Wire into TickTier::Slow
  └─ Detect social/economic/cultural patterns
```

### Phase 5: Evolution & Scaling (Week 5) — P1 + P2 Items

```
P1-2: PIANO hierarchical decomposition
  └─ Goal → SubGoal → Option → Action state machine
  └─ Reduces LLM costs by caching sub-plans

P1-6: Adaptive species discovery
  └─ Dynamic species count based on fitness landscape
  └─ No fixed thresholds

P2-4: NK fitness landscape
  └─ Tunable ruggedness (N=genome size, K=epistasis)
  └─ More realistic evolutionary dynamics
```

---

## 6. External Framework Comparison Matrix

| Capability | neotrix-sim Current | Generative Agents | Project Sid | Mesa | FLAME GPU 2 | Target |
|-----------|--------------------|--------------------|-------------|------|-------------|--------|
| Agent count | 50 max | 25 | 1,000+ | 1,000s | 100M+ | 1,000+ |
| Memory | Flat stream | 3-factor stream | Paper+Vector | AgentSet | Agent state | 3-tier consolidated |
| Planning | Flat goals | LLM planning | Hierarchical FSM | Scheduler | Message-passing | Hierarchical FSM |
| Social | ToM + relationships | Location-based | Social Awareness | Minimal | None | ToM + reputation + alliances |
| Evolution | Fitness landscape | None | Behavioral | None | None | NK landscape + adaptive species |
| Parallelism | Single-thread | LLM-bound | LLM-bound | Single | GPU massive | rayon + DashMap |
| Events | SimulationBus (unused) | None | None | None | Message-passing | Full event-driven |
| Safety | Constitutional (string) | None | Constitutional | None | None | Constitutional (type-safe) |
| Emotion | PAD + EmotionEngine | None | Emotion Module | None | None | PAD + modulation |

---

## 7. Key Metrics to Track

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Agent throughput | ~50 agents/tick | 1,000+ agents/tick | agents processed per second |
| Decision quality | Random/heat-based | Goal-driven with memory | % of actions advancing goals |
| Memory retrieval | Never called | Used in every decision | Retrieval calls per tick |
| Reflection frequency | Every 100 ticks (threshold) | Adaptive based on importance | Reflections per 1000 ticks |
| Parallel speedup | 1x (single thread) | 4-8x (8 cores) | wall-clock tick time |
| Emergent behaviors | 0 detected | 3+ patterns | EmergenceDetector findings |
| Species diversity | Fixed thresholds | Adaptive | Species count variance |

---

## 8. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|:-----------:|:------:|------------|
| Borrow checker conflicts during parallelization | High | Medium | Use DashMap + rayon's parallel iterators carefully; batch mutations |
| Memory consolidation increases latency | Medium | Low | Consolidation only on Slow tier (every 100 ticks) |
| PIANO hierarchical FSM too complex for initial implementation | Medium | Medium | Start with flat PlanningStack wiring (P0-1), add hierarchy in Phase 5 |
| Event-driven architecture requires major refactor | Low | High | SimulationBus already exists; incremental migration |
| EmergenceDetector false positives | Medium | Low | Tune thresholds empirically, start with logging only |

---

*Document generated from analysis of 50+ external sources, 4 framework rankings, existing REFACTOR_PLAN, and 986-line world_sim.rs implementation.*
