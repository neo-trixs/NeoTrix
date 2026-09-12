# Model Reverse Engineering — Cycle 437

> Date: 2026-09-12 | Sources: arXiv, ACL 2026, EMNLP 2026

---

## 5 New Models/Papers Analyzed

### 1. Gated-Memory Routing (arXiv:2609.00237)

**Paper**: "Learning What to Retain: Gated-Memory Routing for Efficient Collaboration in Multi-Agent LLM Systems"

**Core Mechanism**: Learned execution memory as shared state for routing decisions. Two gates:
- **Memory Write Gate**: Commits only non-redundant reasoning steps (not all steps)
- **Retrieval Gate**: Supplies each agent a compact, relevant memory subset
- **Adaptive Halting Controller**: Stops when memory has sufficient evidence

**Key Results**: Best average accuracy across 5 benchmarks, +2.44 pts over strongest baseline. 31.9% cost reduction on HumanEval.

**NeoTrix Domain Mapping**:

| Pattern | Domain | Component |
|---------|--------|-----------|
| Write Gate (selective commit) | NT-MEMORY | KB write deduplication — only commit novel reasoning steps |
| Retrieval Gate (compact subset) | NT-CORE | GWT salience filter — route compact state not full history |
| Adaptive Halting | NT-MIND | SEAL cycle termination — stop when sufficient evidence accumulated |
| Joint training of routing + memory | NT-CORE | GWT attention + NT-MEMORY KB as coupled system |

---

### 2. MARCH: Memory-Anchor Routing across Context History (arXiv:2608.12435)

**Paper**: "Scaling Recurrent Memory with Content-Routed State Anchors"

**Core Mechanism**: Periodically caches cumulative recurrent-state checkpoints as "state anchors." Each anchor has a content-conditioned key. At each token:
1. Produce anchor query to attend all causally available state anchors
2. Output = attention-style aggregation over historical anchors + current state
3. Learned null route suppresses historical branch when current state is sufficient

**Key Results**: Outperforms linear attention variants on commonsense reasoning, LongBench, in-context retrieval. Fused implementation exceeds FlashAttention-2 throughput at 64K+.

**NeoTrix Domain Mapping**:

| Pattern | Domain | Component |
|---------|--------|-----------|
| State anchors (periodic checkpoints) | NT-MEMORY | KB experience snapshots at cycle boundaries |
| Content-conditioned routing | NT-CORE | GWT salience-based retrieval from experience hub |
| Null route (suppress when sufficient) | NT-CORE | GWT: skip historical context when current state is enough |
| Residual fusion (current + historical) | NT-MIND | SEAL: current cycle + distilled past experience |

---

### 3. MANAR: Memory-Augmented Attention with Navigational ACR (arXiv:2603.18676)

**Paper**: "MANAR: Memory-augmented Attention with Navigational Abstract Conceptual Representation"

**Core Mechanism**: Implements Global Workspace Theory in attention:
1. **Integration Phase**: Retrieve memory concepts → form Abstract Conceptual Representation (ACR) = "mental image"
2. **Broadcasting Phase**: ACR navigates and contextualizes individual tokens
3. **Linear-time byproduct**: Routing through constant-sized ACR resolves quadratic complexity
4. **Non-convex contextualization**: Output lies outside convex hull of input tokens (creative synthesis)

**Key Results**: Matches/exceeds baselines on GLUE (85.1), ImageNet-1K (83.9%), LibriSpeech (2.7% WER). Only linear-complexity mechanism supporting direct weight-copy from pretrained MHA.

**NeoTrix Domain Mapping**:

| Pattern | Domain | Component |
|---------|--------|-----------|
| ACR = "mental image" | NT-CORE | E8 hexagram as abstract representation |
| Integration → Broadcasting | NT-CORE | GWT: gather → broadcast cycle |
| Linear-time byproduct of GWT | NT-CORE | Validates GWT as efficiency mechanism, not just cognitive metaphor |
| Non-convex synthesis | NT-MIND | SEAL emergence: outputs beyond input convex hull |
| Memory retrieval → ACR → token context | NT-MEMORY | KB retrieval → VSA embedding → attention modulation |

---

### 4. SparDA: Sparse Decoupled Attention (arXiv:2606.04511)

**Paper**: "SparDA: Sparse Decoupled Attention for Efficient Long-Context LLM Inference"

**Core Mechanism**: Fourth per-layer projection (Forecast) alongside Q/K/V:
- Forecast in layer `l` predicts KV blocks needed by layer `l+1`
- Enables CPU-to-GPU prefetch overlapping with current-layer execution
- Compact GQA-level indexer (one Forecast head per GQA group, no softmax)
- <0.5% additional parameters, trains only Forecast projections

**Key Results**: 1.25× prefill speedup, 1.7× decode speedup, up to 5.3× higher decode throughput via larger batch sizes.

**NeoTrix Domain Mapping**:

| Pattern | Domain | Component |
|---------|--------|-----------|
| Forecast projection (predict future needs) | NT-CORE | GWT: predict next salient information before needed |
| Decoupled selection from attention | NT-CORE | GWT attention routing is independent of attention computation |
| Lookahead prefetch | NT-PHYSICAL | Sensor pre-caching: predict sensory needs before attention |
| Compact indexer (per-group not per-head) | NT-CORE | Efficiency: route at group level, attend at head level |

---

### 5. ConvMem: Convolutional Memory (arXiv:2609.10441)

**Paper**: "ConvMem: Convolutional Memory for Long-Context Reasoning"

**Core Mechanism**: Reformulates long-context reasoning as hierarchical convolution:
- LLM + query = convolutional kernel
- Hierarchically summarizes text segments (linear chain → logarithmic tree)
- Configurable Strides + Skip Connections for evidence capture
- Multi-Kernel Convolution decomposes queries into disentangled semantic channels

**Key Results**: Outperforms training-free baselines on RULER-HotpotQA and RULER-2WikiMultiHopQA. Avoids RL overfitting. Highly parallelizable.

**NeoTrix Domain Mapping**:

| Pattern | Domain | Component |
|---------|--------|-----------|
| Hierarchical convolution | NT-MEMORY | KB: hierarchical document summarization |
| Query as kernel | NT-CORE | GWT: query shapes attention pattern like convolution kernel |
| Logarithmic tree decomposition | NT-MIND | SEAL: hierarchical skill abstraction (leaf → branch → root) |
| Multi-kernel decomposition | NT-CORE | GWT: parallel attention heads decompose task into channels |
| Training-free | NT-ACT | Runtime adaptation without retraining |

---

## Cross-Paper Synthesis

### Emerging Meta-Patterns

| Pattern | Papers | NeoTrix Integration |
|---------|--------|---------------------|
| **Selective Memory Write** | Gated-Memory Routing, ActiveMem | NT-MEMORY: write only novel, non-redundant experience |
| **Content-Conditioned Routing** | MARCH, MANAR, Gated-Memory | NT-CORE: GWT routes based on content relevance, not position |
| **Adaptive Halting** | Gated-Memory Routing, Consilience | NT-MIND: SEAL stops when evidence sufficient |
| **Decoupled Selection/Computation** | SparDA, DashAttention | NT-CORE: GWT attention routing independent of attention compute |
| **Hierarchical Abstraction** | ConvMem, MARCH | NT-MEMORY: KB stores hierarchical summaries, not raw tokens |
| **GWT Validation** | MANAR (explicit), all others (implicit) | NT-CORE: GWT is not metaphor but architectural efficiency principle |

### Architecture Implications for NeoTrix

1. **GWT is architecturally validated**: MANAR proves GWT bottleneck produces linear-time attention as a byproduct. NeoTrix GWT is not just cognitive metaphor — it's an efficiency mechanism.

2. **Memory write is the bottleneck**: Gated-Memory Routing and ActiveMem both show that what you write matters more than what you retrieve. NT-MEMORY should invest in write-gate quality.

3. **Decouple routing from computation**: SparDA, DashAttention, and Gated-Memory all decouple "who acts/what to read" from "how to compute." NT-CORE GWT should separate routing logic from attention computation.

4. **Hierarchical compression is universal**: ConvMem (logarithmic tree), MARCH (state anchors), and MANAR (constant-sized ACR) all compress history into hierarchical representations. NT-MEMORY should adopt hierarchical KB summarization.

5. **Adaptive termination beats fixed depth**: Gated-Memory's halting controller and Consilience's conformal calibration both show adaptive stopping outperforms fixed-depth execution. NT-MIND SEAL should implement evidence-based termination.
