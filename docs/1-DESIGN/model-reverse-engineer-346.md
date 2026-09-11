# Model Reverse Engineering — Cycle 346

> **Date**: 2026-09-11
> **Sources**: arXiv, ACL 2026, ICLR 2026, GitHub preprints
> **Focus**: Efficient inference, attention mechanisms, agent coordination, reasoning

---

## 5 Papers for NeoTrix Pattern Mapping

---

### 1. Declarative Attention (DA) — Intrinsic Attention Control
**Paper**: "Language Models Can Control Their Own Attention" (arXiv:2609.02737)
**Date**: 2 Sep 2026

**Core Insight**: Language models already know which parts of context are relevant. Instead of external proxy scoring (O(N) per step), the model *declares* where it needs to attend via special tokens: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). The inference engine parses these declarations like tool calls and skips most KV cache reads.

**Results**: 52.0% reduction in total attended tokens (Gemma-4-31B), 31.1% (Qwen-3.6-27B) with modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale.

**NeoTrix Domain Mapping**:

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | Model-native attention routing | GWT salience refinement — model declares attention regions, GWT broadcasts only relevant state |
| **NT-MEMORY** | KV cache skip for <focus>/<local> regions | KVMem paged KV + DA declarations = selective cache residency |
| **NT-WORLD** | PerceptionBridge awareness_score() | DA `<focus>` mode = attention-gated perception, only relevant sensory events loaded |
| **NT-IO** | Cost-aware model selection | DA reduces compute cost → Axiom A1 (Cost-Aware Routing) alignment |

**Actionable**: Extend `kv_cache_optimizer.rs` to support DA declaration parsing. When model declares `<focus>`, skip irrelevant KV pages. When `<local>`, use only recent window.

---

### 2. ReActNet — Inference-Time Graph Engineering
**Paper**: "Inference-Time Graph Engineering for Multi-Agent LLM Workflows" (arXiv:2609.05774)
**Date**: 4 Sep 2026

**Core Insight**: Rather than optimizing static topology, synthesize a *task-conditioned temporal workflow graph* that jointly specifies agent connectivity and edge-level communication semantics. Each graph snapshot = one reasoning stage, each edge = natural-language instruction specifying what message source agent sends to target.

**Key Design**:
- Separates **graph compilation** (query → directed communication graphs) from **graph execution** (structured message passing)
- No RL or gradient-based topology optimization needed
- Agents update reasoning states by integrating previous states + messages from controller-assigned neighbors
- Final aggregator synthesizes states into answer

**NeoTrix Domain Mapping**:

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | Task-conditioned temporal graphs | ConsciousnessTree → dynamic domain routing per reasoning stage |
| **NT-ACT** | Edge-level communication semantics | EventBus message passing with semantic instructions |
| **NT-MIND** | Graph compilation from query | SEAL pipeline: query → topology → execution → distillation |
| **NT-GOVERNANCE** | Explicit, inspectable orchestration | Audit trail of graph snapshots + edge semantics |

**Actionable**: Implement `TemporalGraphCompiler` in `nt_core`. Query → sequence of directed graphs, each edge carries NL instruction. EventBus routes messages per graph snapshot. Aggregator synthesizes final state.

---

### 3. CondenseFlow — Semantic Compression for Multi-Agent Collaboration
**Paper**: "CondenseFlow: Scalable Latent Space Collaboration via Semantic Compression" (ACL 2026 Findings)
**Source**: ACL Anthology

**Core Insight**: Full-state latent communication in multi-agent systems scales linearly with collaboration rounds. CondenseFlow introduces **Latent Thought Condenser (LTC)** — learnable semantic probes compress KV caches into fixed-size representations, achieving O(1) communication complexity regardless of context length.

**Results**: >99% KV cache memory reduction, ~20% inference latency reduction, 1.7pp accuracy improvement over text-based methods across 7 benchmarks and 6 models.

**Key Innovation**: Cross-attention aggregation (not selection-based pruning) — probes automatically identify information patterns most valuable for collaboration.

**NeoTrix Domain Mapping**:

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-MEMORY** | O(1) fixed-size agent communication | KB compression: fixed-size embeddings per agent, not growing context |
| **NT-CORE** | Semantic probe for information value | GWT salience: LTC probes = attention modulation for cross-domain info |
| **NT-ACT** | Constant-cost multi-agent coordination | EventBus with compressed message payloads |
| **NT-WORLD** | PerceptionBridge scaling | LTC enables perception at scale without linear memory growth |

**Actionable**: Add `LTC` module to `nt_memory`. When agents communicate, compress KV state to fixed-size embedding via cross-attention probes. Store compressed representations in KB. Enables unbounded agent rounds without memory explosion.

---

### 4. NeuralFSM — Adaptive Multi-Agent Coordination
**Paper**: "NeuralFSM: Adaptive Multi-Agent Coordination via Learning Finite-State Execution Policy" (ACL 2026)
**Source**: ACL Anthology

**Core Insight**: Multi-agent problem solving as **finite-state execution process**. Learns both state transition distribution and inter-agent communication weights from interaction traces using Temporal Coordination Controller (TGN-based). No manual protocol design needed.

**Key Components**:
- **FSM-based coordination**: Reusable finite-state structure per problem
- **Temporal Coordination Controller**: TGN learns transitions + sparse communication routing
- **Dual-defense protection**: Graph regularization (training) + trust-aware message attenuation (runtime)
- **Threat model**: Frequency attacks (high-rate injection) + semantic attacks (misleading content)

**Results**: 6.74%–19.39% improvement over baselines, significant token reduction, only 1.82% performance drop under attack.

**NeoTrix Domain Mapping**:

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | FSM-based reasoning stages | ConsciousnessTree stages as FSM states |
| **NT-ACT** | Sparse communication routing | EventBus with learned routing weights |
| **NT-SHIELD** | Dual-defense protection layer | NT-SHIELD sandbox + trust-aware message attenuation |
| **NT-GOVERNANCE** | Runtime anomaly detection | Policy enforcement via trust scores |
| **NT-MIND** | Interaction trace distillation | SEAL pipeline: traces → transition rules |

**Actionable**: Implement `NeuralFSM` coordinator in `nt_core`. FSM states = reasoning stages, TGN learns transitions. Protection layer = trust scores on EventBus messages. Graph regularization during training, attenuation at runtime.

---

### 5. Codebook Agent — Amortized Topology Design
**Paper**: "Codebook Agent: Amortized Topology Design for LLM Multi-Agent Systems" (arXiv:2609.02264)
**Date**: 2 Sep 2026

**Core Insight**: Adaptive communication topology collapses to ~6 distinct graphs even with 64-entry codebook. Edge count is *negatively* correlated with token cost (r ≈ -0.4). Solution: vector-quantized autoencoder compresses successful topologies into 16-entry codebook, reward-weighted MLP maps query to code distribution, MLP proxy reranks candidates in single forward pass.

**Results**: 84.6 average accuracy (vs 83.0 strongest prior), topology emitted in 2.4ms, 21.9–33.2% fewer LLM tokens. No iterative search, no message passing at test time.

**NeoTrix Domain Mapping**:

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | Codebook topology lookup | Pre-computed routing topologies stored in KB, instant lookup |
| **NT-ACT** | 2.4ms topology emission | Near-zero overhead agent routing decisions |
| **NT-MIND** | VQ-VAE for topology compression | SEAL pipeline: distill successful topologies into codebook |
| **NT-MEMORY** | 16-entry topology codebook | KB stores topology embeddings, not full adjacency matrices |

**Actionable**: Build `TopologyCodebook` in `nt_core`. VQ-VAE compresses successful agent topologies. At query time: embed query → distribution over 16 codes → decode topology → execute. 2.4ms overhead, 22-33% token savings.

---

## Cross-Paper Synthesis

### Unified Agent Coordination Stack

```
Layer 5: Codebook Agent (topology lookup, 2.4ms)
Layer 4: ReActNet (temporal graph compilation)
Layer 3: NeuralFSM (FSM state transitions + trust)
Layer 2: CondenseFlow (semantic compression, O(1) messages)
Layer 1: Declarative Attention (model-native KV skip)
```

### Key Insights Across Papers

| Insight | Papers | NeoTrix Implication |
|---------|--------|-------------------|
| **Topology collapses to ~6 distinct graphs** | Codebook Agent | Pre-compute and cache topologies in KB |
| **Model knows where to attend** | DA | Model declares attention → GWT broadcasts only relevant |
| **Communication cost is O(1) achievable** | CondenseFlow | Fixed-size agent messages, unbounded rounds |
| **FSM state transitions > static topologies** | NeuralFSM | ConsciousnessTree as FSM, not static layer hierarchy |
| **Separate compilation from execution** | ReActNet | Graph compile = SEAL phase, graph execute = runtime |
| **Sparse routing reduces cost 22-33%** | Codebook Agent, NeuralFSM | EventBus with learned sparsity |
| **Trust-aware attenuation** | NeuralFSM | NT-SHIELD protection layer for agent messages |

### NeoTrix Architecture Implications

**1. GWT Refinement**: DA + LTC probes enable model-native attention declaration → GWT broadcasts only what matters → <30K effective context per agent.

**2. EventBus Upgrade**: 
- Current: text-based message passing
- Proposed: CondenseFlow compressed embeddings + NeuralFSM trust attenuation + Codebook topology routing

**3. ConsciousnessTree as FSM**: NeuralFSM validates treating consciousness stages as finite states with learned transitions, not static 6-stage loops.

**4. Self-Test Tier Enhancement**:
- T1: Codebook topology exists
- T2: LTC compression works
- T3: Trust attenuation blocks adversarial messages

---

## Action Items

| Priority | Paper | Action | Domain |
|----------|-------|--------|--------|
| P0 | DA | Extend `kv_cache_optimizer.rs` with DA declaration parsing | NT-MEMORY |
| P0 | CondenseFlow | Add LTC module to `nt_memory` for O(1) agent communication | NT-MEMORY |
| P1 | ReActNet | Implement `TemporalGraphCompiler` in `nt_core` | NT-CORE |
| P1 | NeuralFSM | Build FSM coordinator with trust-aware protection | NT-CORE + NT-SHIELD |
| P2 | Codebook Agent | Build `TopologyCodebook` for instant topology lookup | NT-CORE |

---

## Absorbed Terminology (Cycle 346)

| Term | Definition | NeoTrix Mapping |
|------|-----------|-----------------|
| **Declarative Attention (DA)** | Model-native attention control via `<global>`/`<focus>`/`<local>` declarations parsed like tool calls, skipping KV cache reads | GWT refinement + kv_cache_optimizer extension |
| **Latent Thought Condenser (LTC)** | Learnable cross-attention probes compress variable-length KV caches to fixed-size representations for inter-agent communication | NT-MEMORY O(1) agent communication module |
| **TemporalGraphCompiler** | Query-conditioned graph compilation producing directed communication snapshots with semantic edge instructions | NT-CORE dynamic topology generation |
| **NeuralFSM** | Finite-state execution process with TGN-learned transitions and dual-defense protection (graph regularization + trust attenuation) | ConsciousnessTree FSM + NT-SHIELD protection |
| **TopologyCodebook** | VQ-VAE compressed topology registry (16 entries) with reward-weighted query-to-topology mapping in 2.4ms | NT-CORE instant topology lookup |
