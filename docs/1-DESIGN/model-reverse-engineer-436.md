# Model Reverse Engineering — Cycle 436

**Date:** 2026-09-12
**Focus:** Recent papers on efficient inference, attention mechanisms, agent coordination

---

## Paper 1: CEDAR — Error-Bounded Residual Routing for Long-Context Attention

**Paper:** [arXiv:2609.07237](https://arxiv.org/abs/2609.07237) (Sep 7, 2026)

### Core Idea
Post-hoc sparse attention accelerates long-context prefill by routing queries to a small set of token interactions. Problem: hard selection assigns zero probability to omitted chunks — routing misses cannot be recovered. CEDAR introduces **coarse-to-fine error-aware dynamic attention routing**:

1. Each semantic chunk contributes a cheap key-value summary to a **residual attention path**
2. Chunks with high estimated approximation error are expanded to exact token attention
3. Exact and summarized contributions combined in a single softmax normalization (refinement replaces, not duplicates)
4. Output-error bound governed by within-chunk key/value dispersion → variable refinement budget

**Result:** Residual summaries reduce reconstruction error by >98% vs hard dropping. ~3x kernel speedup at 128K context.

### NeoTrix Pattern Mapping

| CEDAR Component | NeoTrix Domain | Integration |
|----------------|----------------|-------------|
| Semantic chunk KV summaries | NT-WORLD | VSA HyperCube chunk encoding — each chunk gets a summary vector |
| Error-aware refinement budget | NT-CORE (GWT) | Salience-based attention allocation — high-error regions get more compute |
| Residual attention path | NT-MEMORY | KB embedding as residual memory — summary vectors persist across cycles |
| Variable refinement budget | NT-MIND | Adaptive resource allocation in SEAL pipeline |

**Key Insight for NeoTrix:** The residual summary pattern maps directly to ConsciousnessTree's multi-resolution knowledge representation. Each cycle could maintain both summary (cheap) and exact (expensive) representations, with GWT routing deciding which resolution to use per query.

---

## Paper 2: EvoSparse — Evolving Sparsity via Token Importance Dynamics

**Paper:** [ACL 2026](https://doi.org/10.18653/v1/2026.acl-long.530) (ACL 2026)

### Core Idea
Models token importance as a **dynamic process** that evolves over decoding steps and propagates through layers, rather than a static snapshot:

1. **Cross-Step Accumulation:** Exponential moving average of historical sparse attention scores → "Heat" vector capturing long-term anchors and short-term reuse
2. **Cross-Layer Propagation:** Retrieval Heads act as global experts, broadcasting high-quality indices to guide Standard Heads in subsequent layers
3. Unified framework: `I_t,l = I_sink ∪ I_local ∪ I_heat ∪ I_prop`

**Result:** 5.36x attention latency speedup, 2.33x end-to-end speedup. Approaches full attention performance.

### NeoTrix Pattern Mapping

| EvoSparse Component | NeoTrix Domain | Integration |
|---------------------|----------------|-------------|
| Cross-Step Accumulation (Heat vector) | NT-MEMORY | Experience accumulation — historical attention = experience heat map |
| Cross-Layer Propagation | NT-CORE (GWT) | GWT broadcast — Retrieval Heads = salient broadcast, Standard Heads = specialist listeners |
| Dynamic importance evolution | NT-MIND | Self-evolution tracks which knowledge nodes gain/lose importance over time |
| Sink + Local + Heat + Prop union | NT-NEXUS | Cross-session memory combines persistent + local + accumulated + propagated knowledge |

**Key Insight for NeoTrix:** The "Heat vector" concept is a direct analog for experience importance scoring in NT-MEMORY. Each KB node could maintain a Heat score that evolves with use, and GWT attention routing could use Heat to decide what to broadcast. The Cross-Layer Propagation pattern mirrors how ConsciousnessTree's meta-cognition layer guides lower-layer attention.

---

## Paper 3: Flux Attention — Context-Aware Hybrid Attention

**Paper:** [arXiv:2604.07394](https://arxiv.org/pdf/2604.07394) (Apr 2026)

### Core Idea
Instead of head-level dynamic sparsity (which causes GPU load imbalance), Flux Attention routes at the **layer level**:

1. Lightweight **Layer Router** evaluates semantic context of input prompt
2. Each transformer layer assigned to Full Attention (FA) or Sparse Attention (SA) mode
3. Layer-wise routing preserves contiguous memory access — GPU bypasses KV tensor loading entirely for SA layers
4. Only 12 hours training on 8x A800 GPUs (parameter-efficient: backbone frozen)

**Result:** 2.8x prefill speedup, 2.0x decode speedup at 256K context.

### NeoTrix Pattern Mapping

| Flux Attention Component | NeoTrix Domain | Integration |
|--------------------------|----------------|-------------|
| Layer Router (lightweight) | NT-CORE (GWT) | GWT salience router — lightweight classifier decides attention mode per layer |
| FA/SA mode assignment per layer | NT-PHYSICAL | Embodiment layer could assign compute modes: full (L5 cognition) vs sparse (L1 action) |
| Contiguous memory access | NT-MEMORY | KB layout optimization — hot nodes contiguous, cold nodes sparse |
| Backbone frozen, router trained | NT-MIND | Self-evolution trains lightweight adapters, not full model rewrites |

**Key Insight for NeoTrix:** The layer-level routing pattern maps to NeoTrix's six-layer architecture. Each layer (L1-L6) could have its own "attention mode" — L5 cognition uses full attention for complex reasoning, L1 action uses sparse attention for routine tool calls. The GWT salience router decides per-layer compute allocation.

---

## Paper 4: LISA — Linear-Indexed Sparse Attention

**Paper:** [arXiv:2607.19358](https://arxiv.org/pdf/2607.19358) (Jul 2026)

### Core Idea
Plug-and-play attention replacement module for long-context reasoning (especially long CoT):

1. **Two parallel streams:**
   - Linear Attention: O(n) long-range memory
   - Sparse Self-Attention: fixed-size M tokens selected by Lightning Indexer
2. Gating mechanism controls flow ratio between streams
3. **Indexer** trained with per-head KL divergence loss aligning selection with teacher attention patterns
4. Two-stage training: cold-start linear attention → introduce Indexer with dropout

**Result:** 50% inference speedup at 16K context, +5.6% accuracy on reasoning benchmarks (AIME, MATH-500).

### NeoTrix Pattern Mapping

| LISA Component | NeoTrix Domain | Integration |
|----------------|----------------|-------------|
| Linear Attention stream | NT-MEMORY | Long-range context = KB embedding stream (O(n) scan) |
| Sparse Self-Attention stream | NT-CORE (GWT) | Focused reasoning = GWT-selected salient nodes |
| Gating mechanism | NT-MIND | Attention manager allocates between broad scan and focused reasoning |
| Lightning Indexer (top-M selection) | NT-WORLD | Content extraction selects top-M relevant documents |
| Per-head KL loss | NT-REPAIR | Self-healing uses teacher-student distillation |

**Key Insight for NeoTrix:** The dual-stream pattern (linear scan + sparse focus) maps to how NT-MEMORY should handle queries: cheap linear scan of all KB embeddings for broad recall, then GWT routes to top-M salient nodes for deep reasoning. The gating mechanism is the cost-awareness layer — when context is cheap, use broad scan; when scarce, use focused selection.

---

## Paper 5: ReActNet — Inference-Time Graph Engineering for Multi-Agent Workflows

**Paper:** [arXiv:2609.05774](https://arxiv.org/abs/2609.05774) (Sep 4, 2026)

### Core Idea
Multi-agent orchestration via **task-conditioned temporal workflow graphs**:

1. Compile query + role-specialized agents into sequence of directed communication graphs
2. Each graph snapshot = one reasoning stage; each edge carries NL instruction for message passing
3. Agents update reasoning states by integrating previous states with messages from controller-assigned neighbors
4. Final aggregator synthesizes states into answer
5. **Training-free** — graph compilation from query, no RL or gradient-based topology optimization

**Result:** Consistently improves over fixed-topology and learned-topology baselines across knowledge reasoning, math, code, GAIA tasks.

### NeoTrix Pattern Mapping

| ReActNet Component | NeoTrix Domain | Integration |
|--------------------|----------------|-------------|
| Temporal workflow graphs | NT-ACT | Orchestration pattern — multi-stage task execution with graph topology |
| Edge-level NL instructions | NT-IO | Tool call specifications as structured messages |
| Agent state integration | NT-NEXUS | Cross-agent knowledge sharing via shared workspace |
| Graph compilation from query | NT-CORE (GWT) | GWT compiles task-specific agent topology |
| Training-free compilation | NT-MIND | Zero-shot skill composition from existing skills |

**Key Insight for NeoTrix:** ReActNet's temporal graph compilation directly mirrors how ConsciousnessTree should orchestrate multi-domain tasks. Instead of fixed agent topologies, compile task-specific graphs per query. Each graph stage specifies which NT-* domains participate and what they communicate. This is the "task-conditioned GWT" pattern — attention routing that adapts topology per task.

---

## Cross-Paper Synthesis: NeoTrix Integration Architecture

### Combined Pattern: Adaptive Multi-Resolution Attention

```
                    ┌─────────────────────────────────────┐
                    │         GWT Salience Router          │
                    │   (Flux Attention Layer Router)      │
                    └──────────┬──────────────┬────────────┘
                               │              │
              ┌────────────────▼──┐    ┌──────▼────────────────┐
              │  Full Attention    │    │  Sparse Attention      │
              │  (L5 Cognition)    │    │  (L1-L2 Action)        │
              │  CEDAR residual    │    │  EvoSparse Heat-based  │
              └────────┬──────────┘    └──────┬────────────────┘
                       │                      │
              ┌────────▼──────────────────────▼────────────────┐
              │         NT-MEMORY Knowledge Base                │
              │  Linear stream (LISA) + Sparse stream (GWT)    │
              │  Heat vectors (EvoSparse) + KV summaries (CEDAR)│
              └────────────────────────────────────────────────┘
```

### Five Transferable Patterns

| # | Pattern | Paper Source | NeoTrix Implementation |
|---|---------|-------------|----------------------|
| 1 | **Residual Summaries** | CEDAR | KB nodes maintain both summary and exact representations; GWT chooses resolution per query |
| 2 | **Heat Vector Evolution** | EvoSparse | Experience importance scores that evolve with use; guide GWT attention allocation |
| 3 | **Layer-Level Mode Assignment** | Flux Attention | Each NeoTrix layer gets independent compute mode (full vs sparse) based on task complexity |
| 4 | **Dual-Stream Processing** | LISA | Cheap linear scan for broad recall + GWT-gated sparse focus for deep reasoning |
| 5 | **Temporal Graph Compilation** | ReActNet | Compile task-specific agent topology per query; no fixed orchestration |

### Cost Implications (Axiom A1)

| Approach | Context Length | Speedup | Quality Loss |
|----------|---------------|---------|-------------|
| CEDAR residual routing | 128K | ~3x | <2% |
| EvoSparse evolving sparsity | 96K | 5.36x attention, 2.33x e2e | <1% |
| Flux Attention layer routing | 256K | 2.8x prefill, 2.0x decode | <1% |
| LISA linear+sparse | 16K | 2x | +5.6% (improvement!) |
| ReActNet graph compilation | N/A | ~20% fewer tokens | +accuracy |

**Combined estimate:** 2-5x inference cost reduction with maintained or improved quality across NeoTrix's reasoning pipeline.

---

## Key Takeaway

The 2026 inference efficiency papers converge on one meta-insight: **not all tokens/layers/queries need equal compute**. Every paper achieves speedups by identifying what can be cheap (summaries, heat vectors, sparse modes, linear streams) and what must be exact (error-prone chunks, high-heat tokens, cognition layers, selected indices). This is the physical implementation of Axiom A1 (Cost-Aware Routing) — and NeoTrix's GWT salience router is architecturally positioned to implement all five patterns simultaneously.
