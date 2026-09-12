# Model Reverse Engineering — Cycle 384

**Date**: 2026-09-11
**Focus**: Recent papers on efficient inference, attention, agent coordination
**Mapping**: Patterns → NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD)

---

## Paper 1: CEDAR — Error-Bounded Residual Routing for Efficient Long-Context Attention

**Source**: arXiv:2609.07237 (Sep 7, 2026)
**Venue**: Preprint

### Core Innovation
Coarse-to-fine attention routing that keeps the LM frozen while preserving global coverage. Each semantic chunk contributes a cheap key-value summary to a residual attention path; chunks with high estimated approximation error are expanded to exact token attention. Derives an output-error bound governed by within-chunk key/value dispersion to allocate variable refinement budget.

### Key Mechanism
- **Residual summaries** reduce reconstruction error by >98% vs hard dropping at equal exact-chunk budgets
- **Error-bounded refinement**: variable budget allocation based on approximation error, not fixed expansion
- **Single softmax normalization**: exact + summarized contributions combined without duplication

### NeoTrix Domain Mapping

| Domain | Pattern Extracted | Implementation Path |
|--------|------------------|-------------------|
| **NT-CORE** (GWT) | Error-bounded attention routing with residual fallback | GWT salience scoring should use approximation error to decide broadcast scope |
| **NT-MEMORY** (KB) | Coarse-to-fine retrieval with quality guarantees | KB search: coarse BM25 → fine VSA embedding, with error-bounded expansion |
| **NT-IO** (provider) | Variable-cost routing based on task difficulty | Axiom A1 (Cost-Aware Routing): cheap models for easy queries, expensive for ambiguous ones |

### Actionable Insight
CEDAR's error-bound formula can be adapted for GWT attention modulation: when the approximation error of a coarse broadcast exceeds threshold, upgrade to full broadcast. This implements Axiom A2 (Context as Scarce Resource) by spending attention budget where it matters.

---

## Paper 2: Flux Attention — Context-Aware Hybrid Attention for Efficient LLM Inference

**Source**: arXiv:2604.07394 (Apr 2026, updated Sep 2026)
**Venue**: Preprint

### Core Innovation
Layer-level dynamic routing between full attention (FA) and sparse attention (SA) based on input context. A lightweight Layer Router evaluates semantic context and assigns each layer to FA or SA mode. Parameter-efficient: only 12 hours training on 8×A800 GPUs. Achieves 2.8× prefill speedup and 2.0× decode speedup at 256K context.

### Key Mechanism
- **Layer-wise heterogeneity**: early layers sensitive to softmax removal, deeper layers tolerate linear alternatives
- **Gumbel-Softmax relaxation**: differentiable soft routing during training, deterministic hard routing at inference
- **Per-layer decision cached**: one routing decision per layer, reused across all decode steps

### NeoTrix Domain Mapping

| Domain | Pattern Extracted | Implementation Path |
|--------|------------------|-------------------|
| **NT-CORE** (GWT) | Layer-wise attention budget allocation | GWT should allocate attention budget per consciousness layer, not globally |
| **NT-MIND** (SEAL) | Lightweight router trained with minimal data | SEAL distillation: train small routing model on 12h of data, not full retraining |
| **NT-PHYSICAL** | GPU-aware memory layout for sparse/dense switching | Layer-level routing preserves contiguous memory access → hardware-friendly |

### Actionable Insight
Flux Attention's layer-wise routing maps directly to NeoTrix's 6-layer architecture. Each layer (L1-L6) should have its own attention budget router, with the Layer Router analogue being a lightweight NT-MIND module that learns which layers need full context vs sparse context per task type.

---

## Paper 3: Declarative Attention — Language Models Control Their Own Attention

**Source**: arXiv:2609.02737 (Sep 2, 2026)
**Venue**: Preprint

### Core Innovation
Models declare where they need to attend within their chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). The inference engine parses these declarations like tool calls and skips most KV cache reads. Zero-shot: 52% reduction in total attended tokens on Gemma-4-31B with 1.27pp accuracy drop.

### Key Mechanism
- **Intrinsic routing**: the model itself knows which context parts are relevant (not an external proxy)
- **Three-mode attention**: global/focus/local as declarative tokens in chain-of-thought
- **No training required**: works on off-the-shelf models

### NeoTrix Domain Mapping

| Domain | Pattern Extracted | Implementation Path |
|--------|------------------|-------------------|
| **NT-CORE** (GWT) | Self-declared attention scope | GWT salience should let agents declare their attention scope, not just receive broadcasts |
| **NT-MIND** (SEAL) | Attention declaration as learned skill | SEAL crystallization: train agents to output attention mode declarations |
| **NT-ACT** (tools) | Attention declaration as tool-call analogue | Tool calls already parse structured output; attention declarations use same mechanism |

### Actionable Insight
This is the most architecturally aligned paper with NeoTrix. The three-mode attention (global/focus/local) maps directly to GWT broadcast modes. NT-CORE should implement attention declaration tokens that agents output in their chain-of-thought, allowing the inference engine to skip unnecessary KV reads. This is Axiom A2 (Context as Scarce Resource) operationalized.

---

## Paper 4: AgentFlow — In-the-Flow Agentic System Optimization

**Source**: ICLR 2026 Oral (Top 1.1%)
**Venue**: ICLR 2026

### Core Innovation
Modular agentic framework with four specialized modules (Planner, Executor, Verifier, Generator) coordinated through evolving memory. Trained in-the-flow using Flow-GRPO: converts multi-turn optimization into tractable single-turn policy updates. 7B backbone outperforms GPT-4o on search (+14.9%), agentic (+14.0%), math (+14.5%), science (+4.1%).

### Key Mechanism
- **Flow-GRPO**: broadcasts single trajectory-level outcome to every turn, aligning local planner decisions with global success
- **In-the-flow optimization**: trains planner inside multi-turn loop, not offline
- **Group-normalized advantages**: stabilizes learning under sparse rewards

### NeoTrix Domain Mapping

| Domain | Pattern Extracted | Implementation Path |
|--------|------------------|-------------------|
| **NT-CORE** (E8) | Modular planner/executor/verifier/generator as hexagram roles | E8 hexagram reasoning should map to Planner/Executor/Verifier/Generator states |
| **NT-MIND** (SEAL) | In-the-flow training with trajectory-level rewards | SEAL self-test should use trajectory-level success signals, not per-module metrics |
| **NT-ACT** (orchestration) | Evolving memory coordination between modules | NT-ACT orchestration should use shared evolving memory, not message passing |

### Actionable Insight
AgentFlow's four-module architecture (Planner/Executor/Verifier/Generator) is isomorphic to NeoTrix's SEAL pipeline phases. The key innovation is Flow-GRPO: instead of training each phase independently, train with trajectory-level rewards that propagate backward. NT-MIND should adopt this for SEAL self-evolution.

---

## Paper 5: Procedural Graphs — Self-Evolving Execution Structures for LLM Agents

**Source**: arXiv:2609.09153 (Sep 8, 2026)
**Venue**: Preprint

### Core Innovation
Organizes procedural knowledge into (procedure, relation, procedure) triplets — a Procedural Graph where nodes are procedures and edges are relations. At each decision step, a guidance model translates the surrounding subgraph into step-level situational guidance. The graph is self-evolving: an LLM refiner contrasts failed vs successful trajectories and edits the graph's topology.

### Key Mechanism
- **Self-evolving topology**: graph edits that preserve/improve held-out validation performance
- **Rejected edit retention**: failed edits retained to discourage repetition
- **Minimal skeleton → rich graph**: starts from minimal structure, builds through evolution

### NeoTrix Domain Mapping

| Domain | Pattern Extracted | Implementation Path |
|--------|------------------|-------------------|
| **NT-CORE** (E8) | Procedural knowledge as graph triplets | E8 hexagram reasoning should encode procedural knowledge, not just factual knowledge |
| **NT-MEMORY** (KB) | Self-evolving procedural graph in KB | KB should store procedural graphs with evolution metadata |
| **NT-MIND** (SEAL) | Contrastive graph editing from failed/successful trajectories | SEAL absorption should extract procedural graphs from experience, with failure-aware editing |

### Actionable Insight
Procedural Graphs map to NeoTrix's KB procedural memory. The self-evolving mechanism (contrastive editing with held-out validation) is exactly what experience-tree should do: not just store experience, but evolve the procedural graph structure. The "rejected edit retention" pattern prevents repetition of failed approaches.

---

## Cross-Paper Synthesis

### Pattern 1: Attention as Explicit Resource (CEDAR + Flux + Declarative)
All three attention papers treat attention as a budget to be allocated, not a fixed computation. This confirms Axiom A2 (Context as Scarce Resource). **NeoTrix action**: Implement attention budget allocation at each consciousness layer (L1-L6), with error-bounded fallback.

### Pattern 2: In-the-Flow Optimization (AgentFlow + Procedural Graphs)
Both papers optimize within the execution loop, not offline. This confirms SEAL's need for in-loop self-evolution. **NeoTrix action**: SEAL Phase 5 (absorption) should use trajectory-level rewards, not per-module metrics.

### Pattern 3: Self-Evolving Structures (Procedural Graphs + AgentFlow)
Both papers evolve their coordination structure through experience. This aligns with NT-MIND's进化工匠 role. **NeoTrix action**: Implement contrastive graph editing in experience-tree, where failed trajectories prune edges and successful ones reinforce them.

### Pattern 4: Modular Specialization with Shared Memory (AgentFlow + Procedural Graphs)
Both papers use specialized modules coordinating through shared memory, not message passing. This validates NeoTrix's KB as shared state layer. **NeoTrix action**: NT-ACT orchestration should use KB-mediated coordination, not direct agent-to-agent messaging.

---

## Recommendations for NeoTrix

1. **Implement Attention Declaration Tokens** (from Declarative Attention): Add `<global>/<focus>/<local>` attention mode declarations to GWT broadcast protocol
2. **Adopt Flow-GRPO for SEAL** (from AgentFlow): Train SEAL pipeline with trajectory-level rewards, not per-phase metrics
3. **Build Procedural Graph Editor** (from Procedural Graphs): experience-tree should evolve procedural knowledge graphs with contrastive editing
4. **Error-Bounded Retrieval** (from CEDAR): KB search should use approximation error bounds to decide when to expand from coarse to fine retrieval
5. **Layer-Wise Attention Router** (from Flux Attention): Each consciousness layer (L1-L6) should have its own attention budget router, trained with minimal data

---

## Paper Reference Summary

| Paper | Venue | Key Contribution | NeoTrix Priority |
|-------|-------|-----------------|-----------------|
| CEDAR | Preprint 2026 | Error-bounded residual attention routing | P1 |
| Flux Attention | Preprint 2026 | Layer-wise FA/SA dynamic routing | P1 |
| Declarative Attention | Preprint 2026 | Self-declared attention scope (global/focus/local) | P0 |
| AgentFlow | ICLR 2026 Oral | In-the-flow modular optimization with Flow-GRPO | P0 |
| Procedural Graphs | Preprint 2026 | Self-evolving procedural knowledge graphs | P1 |
