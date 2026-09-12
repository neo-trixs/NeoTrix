# Model Reverse Engineering — Cycle 396

**Date:** 2026-09-12
**Focus:** Recent papers on efficient inference, attention mechanisms, agent coordination

---

## 5 Models/Papers Analyzed

### 1. Declarative Attention (DA)
**Paper:** "Language Models Can Control Their Own Attention" (arXiv:2609.02737, Sep 2026)

**Core Innovation:** Models declare WHERE they need to attend within their chain-of-thought, partitioning generation into three modes:
- `<global>` — full context
- `<focus>` — specific region
- `<local>` — recent output only

The inference engine parses these declarations like tool calls and skips most KV cache reads.

**Key Results:**
- 52.0% reduction in total attended tokens (Gemma-4-31B)
- 31.1% reduction (Qwen-3.6-27B)
- Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale
- Zero-shot: no training required

**NeoTrix Domain Mapping:**
- **NT-CORE (GWT):** DA is an intrinsic attention routing mechanism — the model itself decides what's salient. Maps directly to GWT's selective attention broadcasting. Could enhance ConsciousnessTree by letting modules declare attention focus.
- **NT-MIND:** Self-evolving attention patterns based on task structure. The `<focus>`/`<global>`/`<local>` taxonomy is a meta-cognitive awareness primitive.

**Absorption Pattern:**
```
DA three-mode taxonomy → GWT salience refinement
Intrinsic routing (model-declared) vs extrinsic routing (proxy-scored)
→ Hybrid: use DA for intra-step, GWT for inter-step attention
```

---

### 2. CEDAR: Coarse-to-fine Error-aware Dynamic Attention Routing
**Paper:** arXiv:2609.07237 (Sep 2026)

**Core Innovation:** Each semantic chunk contributes a cheap key-value summary to a **residual attention path**. Chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions combine in a single softmax normalization — refinement replaces, rather than duplicates, coarse evidence.

**Key Results:**
- 98% reduction in reconstruction error vs hard dropping
- ~3x kernel speedup at 128K context
- Error-bounded: output quality guaranteed within provable bounds

**NeoTrix Domain Mapping:**
- **NT-MEMORY (KB):** Residual summaries as a memory tier — coarse summaries with error-bounded refinement. Maps to KB's multi-tier retrieval (BM25 → vector → full).
- **NT-CORE (HyperCube):** Error-bounded routing aligns with VSA HyperCube's tolerance for approximate matching. The residual path is conceptually similar to VSA's superposition with selective high-fidelity retrieval.

**Absorption Pattern:**
```
CEDAR residual summary + error-bound expansion
→ KB retrieval: coarse summary first, expand only when error exceeds threshold
→ HyperCube: selective high-fidelity extraction from superposed representations
```

---

### 3. PARSER: Read in Parallel, Reason in Depth
**Paper:** arXiv:2609.06702 (Sep 2026)

**Core Innovation:** Decouples reading from reasoning. A bank of lightweight subagents each bound to a single chunk read the entire document in parallel. A lead agent reasons in depth through iterative scatter–gather rounds: broadcasts a query to all subagents, aggregates evidence, formulates deeper follow-up queries.

**Key Results:**
- 4B backbone outperforms strongest sequential baseline by 5.7pp average, 12.0pp at 896K tokens
- 9B backbone surpasses DeepSeek-V4-Pro by 6.3pp
- 11x inference latency reduction vs sequential methods
- Robust to evidence position/order/distance perturbations

**NeoTrix Domain Mapping:**
- **NT-WORLD (crawl/perception):** Parallel document processing with iterative refinement. Maps to crawl pipeline's multi-source acquisition with depth-first reasoning.
- **NT-ACT (orchestration):** Scatter-gather pattern for multi-agent task decomposition. Lead agent + subagent bank is a natural fit for NT-ACT's orchestration layer.
- **NT-MEMORY:** The iterative query refinement (broadcast → aggregate → deeper query) mirrors KB's iterative search refinement.

**Absorption Pattern:**
```
PARSER scatter-gather → NT-ACT multi-agent task decomposition
Frozen subagents + trainable lead → cost-efficient parallel processing
Iterative query refinement → KB multi-round retrieval
```

---

### 4. AgentFlow: In-the-Flow Agentic System Optimization
**Paper:** ICLR 2026 Oral (Top 1.1%), Stanford/Texas A&M/UC San Diego

**Core Innovation:** Four specialized modules — Planner (P), Executor (E), Verifier (V), Generator (G) — coordinated by shared memory. Trained via Flow-GRPO: converts multi-turn optimization into tractable single-turn policy updates by broadcasting trajectory-level outcome to every turn.

**Key Results:**
- 7B model surpasses GPT-4o on search (+14.9%), agentic (+14.0%), math (+14.5%) tasks
- Online RL (Flow-GRPO) yields 17.2% improvement; offline SFT causes 19.0% collapse
- Adaptive tool selection: planner learns which tools work for which task types
- Scaling with inference turns: 3→10 turns continuously improves performance

**NeoTrix Domain Mapping:**
- **NT-CORE (E8):** The four-module architecture (P/E/V/G) maps to E8's hexagram reasoning states. Flow-GRPO's trajectory-level reward is a natural fit for SEAL pipeline's outcome-driven evolution.
- **NT-MIND:** Online RL for planner optimization parallels SEAL's self-evolution loop. The finding that SFT collapses while online RL improves is critical for skill crystallization.
- **NT-SHIELD:** Verifier module is a built-in quality gate — maps to NT-SHIELD's audit dimensions.

**Absorption Pattern:**
```
AgentFlow P/E/V/G → NT-CORE four-phase reasoning (plan/execute/verify/generate)
Flow-GRPO trajectory-level reward → SEAL pipeline outcome-driven skill evolution
Online RL > SFT finding → constrain NT-MIND to online evolution only
Adaptive tool selection → GWT cost-aware routing (Axiom A1)
```

---

### 5. Procedural Graphs: Self-Evolving Execution Structures
**Paper:** arXiv:2609.09153 (Sep 2026)

**Core Innovation:** Organizes procedural knowledge into (procedure, relation, procedure) triplets — a knowledge graph for "what-to-do" questions. At each step, a guidance model translates the surrounding subgraph into situational guidance that biases (not dictates) the solver's next action. The graph is self-evolving: an LLM refiner contrasts failed vs successful trajectories and edits the graph's topology.

**Key Results:**
- Starting from minimal skeleton, builds graphs matching or surpassing hand-designed ones
- Can repair flawed expert priors
- Consistent gains over memory-based baselines across multiple datasets/LLMs
- Self-evolution further improves performance without manual engineering

**NeoTrix Domain Mapping:**
- **NT-MIND (SEAL):** Procedural Graphs are a direct analog to SEAL's skill crystallization — but structured as graphs instead of linear pipelines. Self-evolution via trajectory comparison is exactly what SEAL does.
- **NT-CORE (ConsciousnessTree):** The graph-as-execution-structure maps to ConsciousnessTree's 6-stage feedback loop. Each node in the Procedural Graph is like a branch in the tree.
- **NT-MEMORY:** (procedure, relation, procedure) triplets extend KB's (entity, relation, entity) triplets from factual to procedural knowledge.

**Absorption Pattern:**
```
Procedural Graphs (procedure→relation→procedure)
→ Extend KB from factual knowledge graph to procedural knowledge graph
Self-evolution via trajectory comparison (fail vs success)
→ SEAL pipeline: crystallize successful trajectories into procedural graphs
Guidance model (biases, doesn't dictate)
→ GWT salience modulation (influence, don't control)
```

---

## Cross-Cutting Synthesis

### Pattern 1: Intrinsic vs Extrinsic Attention Control
- **DA (intrinsic):** Model declares its own attention focus
- **CEDAR (extrinsic):** Error-bounded routing with residual summaries
- **Synthesis:** NeoTrix should use intrinsic routing for intra-step (GWT salience), extrinsic routing for inter-step (KB retrieval tiers)

### Pattern 2: Parallel Reading, Sequential Reasoning
- **PARSER:** Parallel subagents + sequential lead agent
- **AgentFlow:** Parallel module execution + sequential planner updates
- **Synthesis:** NT-ACT should adopt scatter-gather for document processing, sequential planning for reasoning chains

### Pattern 3: Self-Evolution via Trajectory Comparison
- **Procedural Graphs:** Refiner edits graph topology based on fail/success comparison
- **AgentFlow:** Flow-GRPO broadcasts trajectory-level reward to every turn
- **Synthesis:** SEAL pipeline should crystallize both procedural graphs AND trajectory-level reward signals for skill evolution

### Pattern 4: Error-Bounded Approximation
- **CEDAR:** Provable output-error bounds for sparse attention
- **DA:** Accuracy drops shrink with model scale
- **Synthesis:** NT-MEMORY should implement error-bounded retrieval tiers (coarse→fine with quality guarantees)

---

## Priority Absorption Candidates

| Priority | Paper | Pattern | Target Module |
|----------|-------|---------|---------------|
| P0 | Procedural Graphs | Self-evolving execution structures | NT-MIND (SEAL) + NT-MEMORY (KB) |
| P0 | AgentFlow | Online RL for planner + P/E/V/G | NT-CORE (E8) + NT-MIND (SEAL) |
| P1 | Declarative Attention | Intrinsic attention routing | NT-CORE (GWT) |
| P1 | PARSER | Parallel reading + scatter-gather | NT-ACT (orchestration) + NT-WORLD |
| P2 | CEDAR | Error-bounded residual routing | NT-MEMORY (KB tiers) |
