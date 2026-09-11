# Model Reverse Engineering — Cycle 351 (2026-09-12)

## Selection Criteria

Papers/models from 2025-2026 on efficient inference, attention mechanisms, agent coordination, and memory systems. Focus on patterns transferable to NeoTrix's 7-domain architecture.

---

## Paper 1: AGAO — Adaptive Goal-aware Attention Orchestration

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2607.23678 (Jul 2026) |
| **Title** | Focus Is All You Need: Adaptive Goal-aware Attention Orchestration for Multi-Agent Graph Systems |
| **Authors** | — |

### Core Idea

Extends attention from token-level to workflow-level. Three attention mechanisms:
1. **Goal-aware Attention**: Semantic relevance between user objectives and agent capabilities
2. **Topology-aware Attention**: Graph dependencies and execution structure into attention scores
3. **Resource-aware Attention**: Computational budget constraints into routing decisions

### Key Insight

Attention is not just for representation learning — it should control which agents execute, in what order, and with how much resource allocation. This is "Attention Engineering" as a new paradigm.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-CORE** (GWT) | GWT salience now includes goal-aware scoring | GWT already routes attention — AGAO adds explicit goal-orientation to salience |
| **NT-ACT** (orchestration) | Resource-aware agent scheduling | Budget-constrained routing prevents runaway token consumption |
| **NT-MIND** (SEAL) | Topology-aware evolution | SEAL stages can use topology attention to prioritize which branches to evolve |

### Implementation Sketch

```
// Extend GWT salience scoring with goal-aware component
fn agao_salience(task: &Task, agent: &Agent, budget: &Budget) -> f64 {
    let goal_score = cosine(task.embedding, agent.capability_embedding);
    let topo_score = graph_dependency_weight(task.dependents);
    let resource_score = budget.remaining / budget.total;
    goal_score * 0.5 + topo_score * 0.3 + resource_score * 0.2
}
```

### Risk Assessment

- **Low risk**: Additive to existing GWT — no structural changes needed
- **Validation**: Test on multi-domain task routing before production

---

## Paper 2: AgeMem — Agentic Memory (Unified LTM/STM)

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2601.01885 (Jan 2026, v3 Jul 2026) |
| **Title** | Agentic Memory: Learning Unified Long-Term and Short-Term Memory Management for LLM Agents |
| **Authors** | Yi Yu, Liuyi Yao, et al. |

### Core Idea

Memory operations (store, retrieve, update, summarize, discard) are exposed as callable tools within the agent's policy. The agent learns WHEN and WHAT to memorize via reinforcement learning (step-wise GRPO). Three-stage progressive RL training.

### Key Insight

Memory is not a passive store — it's an active decision. The agent learns to manage its own memory through trial and error, not heuristics. Two phases:
1. **Exploration**: Agent tries different memory actions
2. **Integration**: Agent coordinates memory with reasoning under real query pressure

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-MEMORY** (KB) | Memory operations as first-class actions | KB already has store/retrieve — add update/discard as learned behaviors |
| **NT-MIND** (SEAL) | Experience absorption as memory policy | experience-tree could use RL-optimized absorption instead of fixed 5-stage pipeline |
| **NT-CORE** (Self) | Self-model memory management | SelfModel tracks capability/uncertainty — memory operations should feed back into SelfModel |

### Implementation Sketch

```
// Memory actions as tool calls
enum MemoryAction {
    Store { content: String, ttl: Option<Duration> },
    Retrieve { query: Embedding, top_k: usize },
    Update { id: KbNodeId, delta: MemoryDelta },
    Summarize { session_id: SessionId },
    Discard { id: KbNodeId, reason: String },
}

// Step-wise GRPO reward function
fn memory_reward(action: &MemoryAction, outcome: &TaskOutcome) -> f64 {
    let utility = task_success_bonus(outcome);
    let cost = token_cost(action);
    let staleness_penalty = if action.is_store() { 0.0 } else { 0.0 };
    utility - cost - staleness_penalty
}
```

### Risk Assessment

- **Medium risk**: RL training requires many episodes — cold-start problem
- **Mitigation**: Start with heuristic policies, evolve to RL after baseline established
- **Validation**: Compare against fixed experience-tree on 100+ sessions

---

## Paper 3: ETI — Explicit Trait Inference for Multi-Agent Coordination

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2604.19278 (ACL 2026) |
| **Title** | Explicit Trait Inference for Multi-Agent Coordination |
| **Authors** | Suhaib Abdurahman, et al. |

### Core Idea

Agents infer partner characteristics along two psychological dimensions — **warmth** (trust) and **competence** (skill) — from interaction histories. These profiles guide coordination decisions. Reduces payoff loss by 45-77% in economic games, improves performance 3-29% in complex multi-agent settings.

### Key Insight

LLM agents can reliably infer others' traits from interaction histories and leverage structured awareness of others' traits for coordination. This is lightweight (no retraining) and robust across scenarios.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-CORE** (Self) | SelfModel tracks agent trust/competence profiles | SelfModel already tracks capability — extend to trust across collaborators |
| **NT-ACT** (orchestration) | Trust-aware task delegation | High-stakes tasks route to high-competence agents; routine tasks to cheaper ones |
| **NT-SHIELD** (security) | Competence-based security tiers | Untrusted/low-competence agents get sandboxed execution |

### Implementation Sketch

```
struct AgentProfile {
    agent_id: AgentId,
    warmth: f64,      // trust score [0, 1]
    competence: f64,  // skill score [0, 1]
    interaction_count: u32,
    last_updated: Timestamp,
}

// Update profile after each interaction
fn infer_trait(history: &[Interaction], partner: &AgentId) -> AgentProfile {
    let warmth = compute_trust(history);
    let competence = compute_skill(history);
    AgentProfile { warmth, competence, .. }
}

// Route task based on profile
fn delegate(task: &Task, agents: &[AgentProfile]) -> AgentId {
    agents.iter()
        .filter(|a| a.competence >= task.min_competence)
        .min_by_key(|a| (a.competence * 100.0) as u32)  // cheapest sufficient
        .map(|a| a.agent_id)
        .unwrap_or_else(|| fallback_agent())
}
```

### Risk Assessment

- **Low risk**: Purely additive — no structural changes to agent framework
- **Validation**: Test on multi-domain agent coordination scenarios
- **Extension**: Could integrate with GWT for attention-weighted trait routing

---

## Paper 4: AgentInfer — Co-Design of Inference Architecture and System

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2512.18337 (Dec 2025, v2 Feb 2026) |
| **Title** | Towards Efficient Agents: A Co-Design of Inference Architecture and System |
| **Authors** | Weizhe Lin, et al. |

### Core Idea

Four synergistic components for end-to-end agent acceleration:
1. **AgentCollab**: Hierarchical dual-model reasoning (large + small models, dynamic role assignment)
2. **AgentSched**: Cache-aware hybrid scheduler for heterogeneous request patterns
3. **AgentSAM**: Suffix-automaton speculative decoding reusing multi-session semantic memory
4. **AgentCompress**: Asynchronous semantic compression of agent memory

### Key Insight

The efficiency paradox: standard optimizations (KV-cache, prefix sharing) fail for agents because agent memory grows differently than chat memory. AgentCompress distills reasoning traces asynchronously without disrupting ongoing reasoning. AgentSAM reuses predictions across sessions via suffix automata.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-IO** (providers) | Dual-model routing (AgentCollab) | GWT salience + cost weight already routes — add dynamic large/small switching |
| **NT-MEMORY** (cache) | Semantic compression for KB | experience-tree could use async compression instead of fixed-size snapshots |
| **NT-PHYSICAL** (embodiment) | Cache-aware scheduling | Edge devices need hybrid scheduling — maps to Body Schema resource management |
| **NT-CORE** (reasoning) | Suffix-automaton reuse | Cross-session reasoning reuse maps to experience-tree lazy branch loading |

### Implementation Sketch

```
// Dual-model routing
fn agent_collab(task: &Task, models: &[Model]) -> (ModelId, ModelId) {
    let complexity = task.estimated_complexity();
    if complexity > COMPLEXITY_THRESHOLD {
        (models.primary.id, models.secondary.id)  // large + small
    } else {
        (models.secondary.id, models.primary.id)  // small first, escalate
    }
}

// Async semantic compression
fn agent_compress(session: &Session, kb: &mut KB) {
    let traces = session.reasoning_traces();
    let compressed = semantic_distill(traces);
    kb.upsert_experience(compressed);  // async, non-blocking
}
```

### Risk Assessment

- **Medium risk**: Dual-model routing adds latency for model switching decisions
- **Mitigation**: Pre-warm both models, use cache-hit ratio to decide
- **Validation**: Benchmark on DeepResearch-style long-horizon tasks

---

## Paper 5: Attention-MoA — Inter-Agent Semantic Attention

| Field | Value |
|-------|-------|
| **Paper** | arXiv:2601.16596 (Jan 2026) |
| **Title** | Attention-MoA: Enhancing Mixture-of-Agents via Inter-Agent Semantic Attention and Deep Residual Synthesis |
| **Authors** | Jianyu Wen, et al. |

### Core Idea

Mixture-of-Agents (MoA) with inter-agent semantic attention + adaptive early stopping. Small open-source models ensemble outperform massive proprietary models (Claude-4.5-Sonnet, GPT-4.1) on MT-Bench (8.83) and AlpacaEval 2.0 (91.15% LC Win Rate).

### Key Insight

Deep semantic interaction between agents (not just passing tokens) enables small models to collectively surpass large models. Adaptive early stopping prevents information degradation in deep layers while maintaining quality.

### NeoTrix Domain Mapping

| Domain | Integration | Rationale |
|--------|------------|-----------|
| **NT-IO** (providers) | MoA ensemble for complex tasks | For hard reasoning tasks, ensemble 3-5 small models instead of one large one |
| **NT-CORE** (GWT) | Inter-agent semantic attention in routing | GWT broadcasts salient info — extend to inter-model attention for ensemble coordination |
| **NT-MIND** (evolution) | Adaptive early stopping for SEAL | SEAL stages could stop early when quality plateaus, saving compute |

### Implementation Sketch

```
// MoA ensemble with semantic attention
fn moa_ensemble(task: &Task, models: &[Model]) -> Response {
    let mut responses: Vec<AgentResponse> = models.iter()
        .map(|m| m.infer(task))
        .collect();

    // Inter-agent semantic attention
    for i in 0..responses.len() {
        for j in 0..responses.len() {
            if i != j {
                let attn = semantic_attention(&responses[i], &responses[j]);
                responses[i].refine(attn, &responses[j]);
            }
        }
    }

    // Adaptive early stopping
    let quality = measure_quality(&responses);
    if quality > EARLY_STOP_THRESHOLD {
        return aggregate(responses);  // stop deep refinement
    }

    // Deep refinement
    refine_until_convergence(responses)
}
```

### Risk Assessment

- **Low risk**: Additive to existing provider routing — can start with 2-model ensemble
- **Validation**: Test on complex reasoning tasks that stress single models
- **Cost benefit**: 3 small models may cost less than 1 large model with better quality

---

## Cross-Paper Synthesis

### Unified Pattern: Attention-Evolution-Memory Triad

All five papers converge on the same insight: **attention allocation, memory management, and evolutionary learning are not separate concerns but a single feedback loop**.

```
┌─────────────────────────────────────────┐
│           Attention (GWT/AGAO)           │
│  Goal-aware + Topology-aware + Resource  │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│     Memory (KB/AgeMem/AgentInfer)       │
│  Store + Retrieve + Update + Compress    │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│     Evolution (SEAL/prime-agent/ETI)    │
│  Learn + Adapt + Improve + Coordinate   │
└──────────────┬──────────────────────────┘
               │
               └──────► feedback to Attention
```

### NeoTrix Integration Priority

| Priority | Paper | Impact | Effort |
|----------|-------|--------|--------|
| **P0** | AGAO | Extends GWT with goal-awareness | Low — additive scoring |
| **P0** | ETI | Trust/competence profiles for delegation | Low — pure addition |
| **P1** | AgentInfer | Dual-model routing + async compression | Medium — needs scheduler |
| **P1** | Attention-MoA | MoA ensemble for complex tasks | Medium — needs ensemble coordinator |
| **P2** | AgeMem | RL-learned memory management | High — needs training pipeline |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| AgeMem (RL-learned) vs experience-tree (fixed pipeline) | Start with fixed pipeline, add RL optimization after 100+ sessions of data |
| MoA (multiple small models) vs Cost-Aware Routing (cheapest single model) | MoA for high-stakes reasoning, single model for routine tasks |
| AgentInfer (async compression) vs指针守恒 (structural invariants) | Compression is orthogonal to pointer conservation — compress content, not structure |
| ETI (trust profiling) vs Dark Forest (delete non-contributing modules) | Trust profiles help identify which modules actually contribute — strengthen Dark Forest decisions |

---

## Action Items

1. **Immediate** (cycle 352): Implement AGAO goal-aware scoring in GWT salience function
2. **Immediate** (cycle 352): Add ETI trust/competence tracking to SelfModel
3. **Short-term** (cycle 353-355): Implement AgentInfer dual-model routing in NT-IO
4. **Medium-term** (cycle 356-360): Build MoA ensemble coordinator for complex reasoning
5. **Long-term** (cycle 360+): Design AgeMem RL training pipeline for experience-tree
