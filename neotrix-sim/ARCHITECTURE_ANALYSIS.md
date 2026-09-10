# NeoTrix-Sim Architecture Analysis

## A. Architecture Gap Analysis

### Generative Agents (Park 2023)

| Finding | Status | Gap | Priority |
|---------|--------|-----|----------|
| **Unified Memory Stream** — append-only log, scored by recency×relevance×importance | **Partial** | `AgentMemory` exists but uses simple eviction (lowest importance). Missing: (1) recency decay scoring, (2) relevance cosine-sim lookup, (3) reflection synthesis, (4) cross-agent retrieval | P0 |
| **Importance-Triggered Reflection** — significance accumulation triggers reflection | **No** | No reflection mechanism exists. Agents accumulate events but never synthesize higher-order insights from them | P0 |
| **Three-Factor Retrieval** — recency×relevance×importance scoring | **No** | `AgentMemory::by_agent` and `recent` are primitive. No cosine similarity, no decay, no composite scoring | P1 |
| **Spatial Memory** — separate structured world knowledge tree | **No** | Agents have no spatial memory/knowledge. `AgentObservation` is ephemeral per-tick, never persisted | P1 |
| **Hierarchical Planning** — daily→hourly→subtask with reactive replanning | **No** | `decide_action` is a flat if-else priority chain. No goal hierarchy, no plan stack, no replanning | P0 |
| **Model Tiering** — cheap models for perception, expensive for reflection | **N/A** | The sim doesn't use LLMs yet, but the architecture should prepare for it | P2 |

### Project Sid/PIANO (Altera 2024)

| Finding | Status | Gap | Priority |
|---------|--------|-----|----------|
| **10-Module Concurrent Architecture** — parallel modules at different timescales | **Partial** | `world_sim.rs` tick is sequential: time→resources→grid→metabolism→perception→consciousness→evolution. No concurrency, no multi-timescale ticks | P1 |
| **Cognitive Controller Bottleneck** — GWT-style single-decision broadcast | **Partial** | `decide_action` is a single decision point, but there's no GWT salience broadcast, no workspace competition, no broadcast-to-specialists pattern | P1 |
| **Action Awareness** — compare expected vs observed outcomes | **No** | Agents take actions but never verify outcomes. No expectation vs observation comparison, no hallucination prevention | P0 |
| **Social Awareness** — Theory of Mind (infer others' sentiments/goals) | **Partial** | `ConsciousnessState` has `social_awareness` (just partner count). `RelationshipGraph` tracks trust. But no ToM: agents can't infer what others want/believe | P1 |
| **Multi-Scale Tick** — reflex/fast/medium/slow at different Hz | **No** | Single tick rate for everything. Metabolism, perception, consciousness, evolution all run at the same cadence | P1 |
| **Constitutional Feedback** — democratic amendment → voting → behavioral change | **Partial** | `Culture` has norms and memes, but no voting mechanism, no amendment proposal, no norm-enforcement loop | P2 |

### JaxLife (Lu 2024)

| Finding | Status | Gap | Priority |
|---------|--------|-----|----------|
| **Turing-Complete VM** — instruction set for agent behavior | **No** | Agents use hardcoded `decide_action` if-else. No instruction set, no interpretable behavior encoding | P2 |
| **GPU-Accelerated Grid Sim** — parallel cell updates | **Partial** | `SpatialGrid` exists for neighbor queries but no parallel update. Sequential iteration over agents | P2 |
| **Emergent Agriculture/Tool Use** — structured terraforming from simple rules | **Partial** | `AgentAction::Build` exists but is never executed by `decide_action`. No tool crafting, no terraforming | P1 |

### CivSim/AgentCiv

| Finding | Status | Gap | Priority |
|---------|--------|-----|----------|
| **Maslow Needs Hierarchy** — sequential activation, lower needs first | **Full** | `SimAgent::needs` array + `update_needs()` implements this correctly. Lower needs (physiological/safety) gate higher ones | — |
| **Event Queue Pattern** — typed events | **Full** | `SimulationBus` + `SimEvent` enum covers all event types. Priority routing exists | — |
| **Emergence Detection** — statistical behavior cluster tracking | **No** | `CoherenceTracker` tracks phi/coherence but not emergent behavior patterns, novelty of collective action, or phase transitions | P1 |
| **LLM ReAct Loop** — observe→think(LLM)→act | **No** | No LLM integration. `decide_action` is rule-based | P2 |

### LEAR (Gurkan 2025)

| Finding | Status | Gap | Priority |
|---------|--------|-----|----------|
| **LLM-as-Mutation-Operator** — LLM modifies agent rules | **No** | `MutationOps` uses random Gaussian noise. No semantic mutation, no LLM-guided rule modification | P2 |
| **Dual Representation** — code ↔ pseudocode | **No** | Single representation (trait vectors). No interpretable behavior representation | P2 |
| **Convergence Detection** — attractor identification | **Partial** | `Speciation` tracks species, but no convergence/attractor detection in trait space | P2 |

### OpenLife (Masumori 2026)

| Finding | Status | Gap | Priority |
|---------|--------|-----|----------|
| **SDP Memory Graph** — semantically plastic, LLM-weighted edges | **No** | `AgentMemory` is a flat vector. No graph structure, no semantic edges, no plasticity | P1 |
| **Budget-Based Metabolism** — every action costs tokens/compute | **Partial** | `AgentCore::metabolize` charges energy per tick. But no per-action cost, no compute budget, no trade-off pressure | P1 |
| **Verbal Policy Optimization** — LLM zero-shot critic | **No** | No LLM critic, no policy evaluation beyond fitness | P2 |
| **Individuation** — agents differentiate over time | **Partial** | `Personality` + `Speciation` exist. But personality is static at birth, no differentiation through experience | P1 |

---

## B. Focused Redundancy

### Overlap: Memory Systems
- `AgentMemory` (flat event vec) ↔ `EventHistory` in `SimulationBus` (flat event vec)
- Both store events with similar metadata. `EventHistory` is richer (has priority, source).
- **Recommendation**: Unify into a single `MemoryStream` with dual access patterns (per-agent + global).

### Overlap: Relationship Tracking
- `SimAgent.relationships: HashMap<String, f32>` (simple trust score)
- `RelationshipGraph` (typed edges with trust, strength, interactions, timestamps)
- Both track agent-to-agent connections with trust. `SimAgent.relationships` is a shadow copy.
- **Recommendation**: Remove `SimAgent.relationships`, use `RelationshipGraph` as single source.

### Overlap: Consciousness Tracking
- `PhiBridge.compute_coherence()` (action string similarity)
- `CoherenceTracker.compute_diversity()` (consciousness level entropy)
- Both compute diversity/coherence metrics but from different angles. Not unified.
- **Recommendation**: Merge into a single `ConsciousnessMetrics` module.

### Overlap: Decision Logic
- `decide_action()` in `world_sim.rs` (hardcoded priority chain)
- `HostBridge::recommend_actions()` (snapshot-based recommendations)
- `HostBridge::value_score()` (directive-weighted scoring)
- Three separate decision-relevant systems that don't compose.
- **Recommendation**: Unify into a `DecisionEngine` that composes: rule chain + directive scoring + recommendations.

### Overlap: Fitness
- `SimAgent::fitness()` (composite: energy + health + social + needs)
- `FitnessLandscape::evaluate()` (genome distance-to-peak + epistasis)
- Two independent fitness functions that don't agree. Evolution uses landscape fitness, but agent survival uses composite fitness.
- **Recommendation**: Single fitness function that composes survival + landscape + social metrics.

---

## C. Flat Deficiency (Completely Missing)

### 1. **Memory Stream with Retrieval** (P0)
External frameworks (Generative Agents, OpenLife) treat memory as the central cognitive substrate. neotrix-sim has:
- Flat append-only `AgentMemory` with no retrieval scoring
- No reflection synthesis
- No episodic/semantic/procedural distinction
- No cross-agent memory query

### 2. **Hierarchical Goal/Planning Stack** (P0)
Every mature agent framework has goal decomposition. neotrix-sim has:
- Flat `decide_action` if-else
- No goal representation
- No plan stack
- No subgoal decomposition
- No reactive replanning

### 3. **Action Awareness / Outcome Verification** (P0)
Agents act blindly — no comparison of expected vs observed results. This is critical for:
- Learning from mistakes
- Preventing hallucination loops
- Building accurate world models

### 4. **Multi-Timescale Execution** (P1)
Reflexes (~100ms), fast decisions (~5s), deliberation (~10s), background processing (~30s). neotrix-sim runs everything at one tick rate.

### 5. **Theory of Mind / Social Simulation** (P1)
Agents can't model what other agents believe, want, or plan. They track trust but not mental states.

### 6. **Emergence Detection** (P1)
No statistical tracking of collective behavior patterns, phase transitions, or novelty in group dynamics.

### 7. **Graph-Structured Memory** (P1)
OpenLife's SDP graph with semantic edges. neotrix-sim has flat memory only.

### 8. **Per-Action Cost Budget** (P1)
OpenLife creates survival pressure through compute costs. neotrix-sim has only passive metabolism.

### 9. **Instruction Set / Behavior VM** (P2)
JaxLife's Turing-complete VM for interpretable agent rules.

### 10. **LLM Integration Points** (P2)
No hooks for LLM-as-critic, LLM-as-mutation, or LLM-as-planner.

---

## D. Cross-Domain Misalignment

### D1. Sequential Tick Bottleneck
**Current**: `world_sim.rs::tick()` processes all agents sequentially in a single loop.
**Best Practice**: Multi-timescale concurrent execution (Project Sid). Reflexes should fire at 10Hz, perception at 5Hz, consciousness at 0.1Hz.
**Action**: Refactor tick into a pipeline with per-subsystem tick rates.

### D2. Ephemeral Observations
**Current**: `build_observation()` creates `AgentObservation` each tick, used once, discarded.
**Best Practice**: Observations should be persisted into memory stream (Generative Agents).
**Action**: Pipe observations into `AgentMemory` or a new `MemoryStream`.

### D3. Static Personality
**Current**: `Personality` is set at birth, never changes.
**Best Practice**: OpenLife's individuation — personality differentiates through experience.
**Action**: Add personality drift based on accumulated emotions and social interactions.

### D4. No Expectation Model
**Current**: `execute_action` applies effects but never compares to expected outcome.
**Best Practice**: Project Sid's Action Awareness — compare expected vs observed to detect world model errors.
**Action**: Add `ExpectedOutcome` struct, compare in `execute_action`.

### D5. Fitness Function Disconnect
**Current**: Evolution uses `FitnessLandscape` (genome-based), agent survival uses `SimAgent::fitness()` (state-based). They don't communicate.
**Best Practice**: Single coherent fitness signal that drives both evolution and behavior.
**Action**: Unify fitness computation, let landscape fitness inform agent-level fitness.

### D6. Emotion Engine Isolation
**Current**: `feel/mod.rs` defines a full `EmotionEngine` with 15 types, PAD vector, GWT modulation, conflict resolution. But it's **never wired into `WorldSim`** — no field on `WorldSim`, no calls in `tick()`.
**Best Practice**: Emotion should modulate perception, decision, memory encoding, and social behavior.
**Action**: Add `EmotionEngine` to `WorldSim`, call `process_events()` each tick, feed emotion state into decision.

### D7. Physical Body Modules Unused
**Current**: `sensor/`, `actuator/`, `physics/`, `world/` modules implement a robot body simulation (Microduck). But `WorldSim` uses `SimAgent` with 2D position, not the physics body.
**Best Practice**: Two separate simulation domains that should share a bridge.
**Action**: Either integrate physical sim into WorldSim, or explicitly decouple into a separate simulation runner.

### D8. No Event-Driven Agent Behavior
**Current**: Agents poll their environment each tick. `SimulationBus` emits events but no agent subscribes or reacts to events.
**Best Practice**: Events should trigger agent responses (e.g., `ResourceDepleted` → agents relocate).
**Action**: Add event subscription in agent tick, reactive behavior layer.

---

## E. Comprehensive Task List

### P0 — Critical Gaps (Must Have for Competitive Agent Simulation)

| # | Task | Source | Complexity | Dependencies | Module |
|---|------|--------|------------|--------------|--------|
| T1 | **Unified Memory Stream**: Replace `AgentMemory` with `MemoryStream` — append-only, scored by recency×relevance×importance, with reflection synthesis | Generative Agents | L | None | `agents/` (rewrite), `foundation/` (new `memory_stream.rs`) |
| T2 | **Importance-Triggered Reflection**: When accumulated significance exceeds threshold, synthesize a reflection (higher-order insight from recent memories) | Generative Agents | M | T1 | `agents/` (new `reflection.rs`) |
| T3 | **Hierarchical Goal Stack**: Implement goal→subgoal→action decomposition with reactive replanning when goals fail | Generative Agents | XL | None | `agents/` (new `planning.rs`) |
| T4 | **Action Awareness**: Add `ExpectedOutcome` struct. In `execute_action`, compare expected vs actual effect. Feed discrepancy into memory and world model updates | Project Sid | M | None | `world_sim.rs`, `agents/sim_agent.rs` |
| T5 | **Wire EmotionEngine into WorldSim**: Add `EmotionEngine` field to `WorldSim`, call `process_events()` each tick, use emotion state to modulate `decide_action` | feel/mod.rs exists but unused | M | None | `world_sim.rs`, `feel/mod.rs` |
| T6 | **Remove Duplicate Relationships**: Delete `SimAgent.relationships`, use `RelationshipGraph` as single source of truth | Redundancy analysis | S | None | `agents/sim_agent.rs`, `society/relationship_graph.rs`, `world_sim.rs` |

### P1 — Important Capabilities (Needed for Rich Agent Behavior)

| # | Task | Source | Complexity | Dependencies | Module |
|---|------|--------|------------|--------------|--------|
| T7 | **Three-Factor Memory Retrieval**: Implement recency decay, cosine relevance, importance composite scoring for memory queries | Generative Agents | M | T1 | `agents/` (extends `MemoryStream`) |
| T8 | **Spatial Memory**: Persistent per-agent spatial knowledge tree (visited locations, resource maps, danger zones) | Generative Agents | M | T1 | `agents/` (new `spatial_memory.rs`) |
| T9 | **Multi-Timescale Tick**: Refactor `WorldSim::tick()` into pipeline with reflex (per-tick), fast (every 5 ticks), medium (every 20 ticks), slow (every 100 ticks) execution tiers | Project Sid | L | None | `world_sim.rs`, `foundation/sim_time.rs` |
| T10 | **Theory of Mind Module**: Per-agent belief/goal model of other agents. Infer intent from observed actions. Update belief model based on interaction outcomes | Project Sid | L | T4 | `society/` (new `theory_of_mind.rs`) |
| T11 | **Emergence Detection**: Statistical tracking of collective behavior clusters, phase transitions, novelty in group dynamics | CivSim | M | None | `consciousness/` (new `emergence_detector.rs`) |
| T12 | **Graph Memory**: Replace flat `AgentMemory` events with graph structure — nodes for events/concepts, edges for temporal/semantic/causal relationships | OpenLife SDP | L | T1 | `agents/` (new `graph_memory.rs`) |
| T13 | **Per-Action Cost Budget**: Each `AgentAction` variant has an energy/compute cost. Agents must budget. Creates survival pressure beyond passive metabolism | OpenLife | M | T3 | `agents/sim_agent.rs`, `world_sim.rs` |
| T14 | **Personality Drift**: Personality traits evolve based on accumulated emotions, social interactions, and success/failure patterns | OpenLife individuation | M | T5 | `agents/sim_agent.rs` |
| T15 | **Event-Driven Agent Behavior**: Agents subscribe to `SimulationBus` events and trigger reactive behaviors (e.g., `ResourceDepleted` → relocate) | Best practice | M | None | `world_sim.rs`, `agents/sim_agent.rs` |
| T16 | **Build Action Execution**: Wire `AgentAction::Build` into `execute_action` — agents can construct structures that modify terrain/resources | JaxLife emergence | M | T13 | `world_sim.rs`, `environment/` |

### P2 — Enhancement (Nice to Have for Advanced Research)

| # | Task | Source | Complexity | Dependencies | Module |
|---|------|--------|------------|--------------|--------|
| T17 | **Constitutional Feedback Loop**: Norm proposal → agent voting → norm enforcement → behavioral change. Extends existing `Culture::norms` | Project Sid | L | None | `society/culture.rs` |
| T18 | **LLM Integration Hooks**: Define traits for LLM-as-critic, LLM-as-planner, LLM-as-mutation. Implement stubs, wire when LLM available | LEAR, OpenLife | L | T3 | `bridge/` (new `llm_bridge.rs`) |
| T19 | **Behavior VM / Instruction Set**: Turing-complete instruction set for interpretable agent behavior rules | JaxLife | XL | T3 | `agents/` (new `behavior_vm.rs`) |
| T20 | **Dual Representation**: Agent rules in both code (fast) and pseudocode (interpretable/mutable) | LEAR | L | T19 | `agents/` |
| T21 | **Convergence Detection**: Identify attractors in trait space evolution trajectories | LEAR | M | None | `evolution/speciation.rs` |
| T22 | **GPU-Accelerated Grid Updates**: Parallel agent perception/action using spatial grid partitioning | JaxLife | L | T9 | `foundation/math_bridge.rs`, `world_sim.rs` |

---

## Execution Order Recommendation

**Phase 1 (P0, ~2 weeks)**: T6 → T1 → T5 → T4 → T2 → T3
- T6 first (removes redundancy, cleans interface)
- T1 (memory foundation everything else builds on)
- T5 (wire existing EmotionEngine — high value, low effort)
- T4 (action awareness — enables learning)
- T2 (reflection — builds on memory)
- T3 (planning — most complex P0, do last)

**Phase 2 (P1, ~3 weeks)**: T7 → T13 → T9 → T15 → T11 → T14 → T8 → T10 → T12 → T16
- T7 extends T1
- T13 adds survival pressure
- T9 restructures tick (enables concurrent subsystems)
- T15 makes agents reactive
- T11 detects emergence
- T14 personality differentiation
- T8/T10/T12 build on memory/planning foundations
- T16 extends action space

**Phase 3 (P2, ~2 weeks)**: T17 → T18 → T21 → T19 → T20 → T22
- T17/T18 are self-contained enhancements
- T21 is analysis tooling
- T19/T20 are research features
- T22 is performance optimization
