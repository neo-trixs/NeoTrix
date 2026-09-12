# Model Reverse Engineering — Cycle 404 (2026-09-12)

## 5 New AI Models/Papers

---

### 1. Gated-Memory Routing (EMNLP 2026)
**Paper**: "Learning What to Retain: Gated-Memory Routing for Efficient Collaboration in Multi-Agent LLM Systems"
**Authors**: Rakibul Hasan Rajib, Mengxing Zheng, Qian Lou
**Source**: https://arxiv.org/abs/2609.00237

#### Core Innovation
Multi-agent orchestration via learned memory gates instead of complete execution history or query-only routing.

#### Architecture
```
Query + Execution Memory
    ↓
[Memory Write Gate] → commits only non-redundant reasoning steps
    ↓
[Retrieval Gate] → supplies each agent a compact, relevant subset
    ↓
[Next Role Selection] → from memory-augmented state
    ↓
[Adaptive Halting Controller] → stops when memory has sufficient evidence
```

#### Key Mechanisms
- **Memory Write Gate**: Learned gate that commits only non-redundant reasoning steps to execution memory. Prevents history overload.
- **Retrieval Gate**: Supplies each agent a compact, relevant subset of execution history. Every decision conditions on clean, informative state.
- **Adaptive Halting Controller**: Stops execution once memory contains sufficient evidence for answering. Avoids unnecessary computation.

#### Results
- Best average accuracy across 5 reasoning/code-gen benchmarks
- Exceeds strongest baseline by +2.44 points
- Reduces HumanEval inference cost by 31.9%

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing | Execution memory = salience buffer; Write Gate = attention filter; Retrieval Gate = broadcast selector |
| **NT-MEMORY** | Knowledge graph | Non-redundant step storage = node deduplication; Compact retrieval = subgraph extraction |
| **NT-MIND** | SEAL pipeline | Adaptive halting = convergence detection; Memory-augmented state = distillation cache |
| **NT-ACT** | Agent orchestration | Role selection from memory = dynamic task allocation; Cost reduction via halting |

#### Actionable Patterns
1. **Write Gate for NT-MEMORY**: Filter KB writes to commit only non-redundant facts. Prevents knowledge base bloat.
2. **Retrieval Gate for GWT**: Supply each consciousness branch a compact, relevant subset of context. Reduces broadcast overhead.
3. **Adaptive Halting for SEAL**: Stop evolution cycles when memory contains sufficient evidence. Prevents over-iteration.

---

### 2. Agora: Auction-Based Task Allocation (Aug 2026)
**Paper**: "Agora: Enhancing LLM Agent Reasoning Via Auction-Based Task Allocation"
**Authors**: Kaiji Zhou, Aleš Leonardis, Yue Feng
**Source**: https://arxiv.org/abs/2607.09600

#### Core Innovation
Reformulates multi-agent task allocation as a confidence-calibrated auction. Reasoning steps become tradeable items allocated based on calibrated competence, not raw confidence.

#### Architecture
```
Reasoning Step (item)
    ↓
[Agent Pool] → each agent bids with calibrated confidence
    ↓
[Auction Mechanism] → allocates to highest-value bidder
    ↓
[Calibration Layer] → adjusts raw confidence using historical performance
    ↓
[Execution] → selected agent executes step
```

#### Key Mechanisms
- **Confidence Calibration**: Raw confidence scores adjusted using agent's historical performance per task type. Prevents overconfident but incompetent agents from winning auctions.
- **Auction-Based Allocation**: Each reasoning step is an item. Agents bid based on calibrated competence. Winner executes.
- **Cascading Strategies**: Sequential frameworks for cost-efficiency — cheap models attempted first, escalated to expensive if needed.

#### Results
- Improves or remains competitive with single-model, routing, and cascade baselines
- Calibrated auction +2.0% on MathVision, +8.7% on SPIQA vs uncalibrated
- Uncalibrated auction drops 4.0% below baseline — calibration is essential

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | E8 Hexagram reasoning | Auction = hexagram state evaluation; Calibrated confidence = phi scoring |
| **NT-ACT** | Tool orchestration | Reasoning steps = tool calls; Auction = tool selection via competence bidding |
| **NT-MIND** | Self-evolution | Historical performance = capability model; Calibration = self-assessment |
| **NT-SHIELD** | Safety gates | Calibration prevents overconfident agents from critical paths |

#### Actionable Patterns
1. **Auction for NT-ACT Tool Selection**: Tools bid on task execution based on calibrated competence, not just capability match.
2. **Calibration for NT-MIND Self-Model**: Track per-task-type success rates. Use for capability boundary learning.
3. **Cascading for Cost-Aware Routing (A1)**: Cheap models attempted first. Escalate only when calibrated confidence is low.

---

### 3. Flux Attention: Context-Aware Hybrid Attention (Apr 2026)
**Paper**: "Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference"
**Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, et al.
**Source**: https://arxiv.org/abs/2604.07394

#### Core Innovation
Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA) based on input context. Lightweight Layer Router inserted into frozen pretrained LLMs.

#### Architecture
```
Input Context
    ↓
[Layer Router] → per-layer decision: FA or SA
    ↓
Layer 1: Full Attention (high-fidelity retrieval)
Layer 2: Sparse Attention (efficient processing)
Layer 3: Full Attention (critical path)
...
    ↓
[Contiguous Memory Access] → preserves hardware efficiency
```

#### Key Mechanisms
- **Layer Router**: Lightweight module that routes each layer to FA or SA based on input context. Only 12 hours training on 8×A800 GPUs.
- **Context-Aware Routing**: Different layers get different attention modes based on what the input needs. Not static ratio.
- **Contiguous Memory Access**: Layer-wise routing (not head-level) ensures contiguous memory access for GPU acceleration.

#### Results
- 2.8× speedup in prefill, 2.0× in decode
- Superior performance-speed tradeoff vs baselines
- Parameter-efficient: only Layer Router trained, LLM frozen

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | GWT attention routing | Layer Router = salience modulator; FA/SA = full vs selective broadcast |
| **NT-WORLD** | Perception pipeline | Context-aware processing = attention-gated sensory flow (PerceptionBridge) |
| **NT-IO** | Inference optimization | Layer-level routing for cost-aware inference (Axiom A1) |
| **NT-PHYSICAL** | Hardware efficiency | Contiguous memory access = sensor data streaming optimization |

#### Actionable Patterns
1. **Layer Router for GWT**: Route each consciousness branch to full or selective attention based on current context salience.
2. **Context-Aware for PerceptionBridge**: Adaptive sensory gating based on consciousness level (awareness_score).
3. **Parameter-Efficient for NT-MIND**: Train lightweight routing modules, keep core reasoning frozen. Aligns with SEAL's incremental evolution.

---

### 4. Sketch&Walk Sparse Attention (Feb 2026)
**Paper**: "Scout Before You Attend: Sketch-and-Walk Sparse Attention for Efficient LLM Inference"
**Authors**: Hoang Anh Duy Le, Sahil Joshi, Zeyu Yang, et al.
**Source**: https://arxiv.org/abs/2602.07397

#### Core Innovation
Training-free sparse attention using Hadamard sketching + deterministic walk. Lightweight sketches approximate attention scores, walk mechanism aggregates across layers to capture indirect token influences.

#### Architecture
```
Input Tokens
    ↓
[Hadamard Sketching] → inexpensive attention score approximations
    ↓
[Walk Mechanism] → aggregates estimates across layers
    ↓
   (captures attention influence beyond direct token interactions)
    ↓
[Top-K Block Selection] → dynamic sparsity with single algorithm
    ↓
[Custom Sparse Kernels] → efficient execution
```

#### Key Mechanisms
- **Hadamard Sketching**: Uses Hadamard transform for cheap attention score approximation. No training required.
- **Walk Mechanism**: Accumulates attention influence across layers. Captures indirect token-to-token relationships that single-layer analysis misses.
- **Unified Algorithm**: Single training-free algorithm works for both prefill and decode phases. No separate optimization.

#### Results
- Near-lossless accuracy at 20% attention density
- Up to 6× inference speedup
- Slightly outperforms dense attention in some settings (surprising)

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | E8 Hexagram reasoning | Walk mechanism = cross-hexagram influence propagation; Sketching = approximate reasoning |
| **NT-MEMORY** | Knowledge retrieval | Sketch = approximate BM25/embedding scoring; Walk = multi-hop reasoning across KB |
| **NT-WORLD** | Content processing | Sparse attention for long-document processing in crawl pipeline |
| **NT-IO** | Inference efficiency | Training-free optimization for local model deployment |

#### Actionable Patterns
1. **Walk Mechanism for E8**: Propagate influence across hexagram layers. Capture indirect reasoning dependencies.
2. **Sketching for KB Retrieval**: Approximate scoring for fast candidate generation, exact scoring for top-K selection.
3. **Training-Free for NT-IO**: Deploy optimizations without retraining. Critical for local/edge deployment.

---

### 5. BoundaryRouter: Agent Routing from Early Experience (ICLR 2026)
**Paper**: "Learning Agent Routing From Early Experience"
**Authors**: Yimin Wang, Jiahao Qiu, Xuan Qi, et al.
**Source**: https://arxiv.org/abs/2605.07180

#### Core Innovation
Training-free routing between lightweight LLM inference and full agent execution using early behavioral experience. Builds compact experience memory from shared seed set, retrieves similar cases at inference time.

#### Architecture
```
Query
    ↓
[Early Experience Memory] → built from shared seed set execution
    ↓
[Rubric-CoT Reasoning] → structured reasoning about query difficulty
    ↓
[Route Decision] → LLM (easy) or Agent (hard)
    ↓
[Answer] → from selected system
```

#### Key Mechanisms
- **Early Experience Memory**: Execute both LLM and Agent on shared seed set. Store outcomes. No ground truth needed.
- **Rubric-CoT**: Chain-of-thought reasoning guided by rubric (correctness priority, efficiency tie-break). Structured routing decision.
- **RouteBench**: Benchmark covering in-domain, paraphrased, and out-of-domain route settings.

#### Results
- 60.6% reduction in inference time vs agent
- 28.6% performance improvement over direct LLM
- Outperforms prompt-based routing by 37.9%, retrieval-only by 8.2%

#### NeoTrix Domain Mapping
| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Cost-Aware Routing (A1) | Route between cheap/expensive based on query difficulty |
| **NT-MIND** | Self-evolution | Early experience = bootstrap learning; Rubric-CoT = self-assessment |
| **NT-ACT** | Task delegation | Route tasks to appropriate agent tier based on difficulty |
| **NT-MEMORY** | Experience storage | Early experience memory = seed experience store |

#### Actionable Patterns
1. **Early Experience for NT-MIND Bootstrap**: When introducing new capabilities, execute on seed set to build initial experience memory. No labeled data needed.
2. **Rubric-CoT for Self-Assessment**: Structured reasoning about task difficulty before delegation. Prevents overconfident execution.
3. **RouteBench for NT-CORE Evaluation**: Benchmark routing decisions across in-domain, paraphrased, and out-of-domain scenarios.

---

## Cross-Paper Pattern Synthesis

### Pattern 1: Memory-Gated Information Flow
**Papers**: Gated-Memory Routing + BoundaryRouter
**Synthesis**: Information flow should be gated at write and retrieval points. Not all reasoning steps deserve storage. Not all stored information deserves retrieval. Gates learn from experience.

**NeoTrix Application**:
- NT-MEMORY: Write Gate for KB inserts (prevent bloat), Retrieval Gate for queries (compact context)
- NT-CORE: GWT salience gates (not all sensory data deserves broadcast)

### Pattern 2: Calibrated Competence Over Raw Confidence
**Papers**: Agora + Gated-Memory Routing
**Synthesis**: Raw confidence is unreliable. Calibrate against historical performance per task type. Auction/selection mechanisms must use calibrated scores.

**NeoTrix Application**:
- NT-MIND: SelfModel tracks per-task success rates. Calibration used for capability boundary learning.
- NT-ACT: Tool selection based on calibrated competence, not just capability match.

### Pattern 3: Training-Free Optimization
**Papers**: Sketch&Walk + BoundaryRouter + Flux Attention (lightweight)
**Synthesis**: Many effective optimizations require no training. Use structural properties (Hadamard, experience memory, layer routing) instead of gradient updates.

**NeoTrix Application**:
- NT-IO: Deploy routing optimizations without retraining. Critical for local/edge.
- NT-MIND: Bootstrap new capabilities from experience, not labeled data.

### Pattern 4: Adaptive Halting / Early Termination
**Papers**: Gated-Memory Routing + Agora (cascading)
**Synthesis**: Don't over-execute. Stop when sufficient evidence accumulated. Use halting controllers that monitor information sufficiency.

**NeoTrix Application**:
- NT-MIND: SEAL cycle halting based on convergence detection. Stop when evolution果实 mature.
- NT-CORE: GWT attention broadcast termination when salience saturates.

### Pattern 5: Layer/Step-Level Routing
**Papers**: Flux Attention + Agora + AgentRouter (ICML 2026)
**Synthesis**: Routing at fine granularity (layer, step, tool call) outperforms routing at coarse granularity (query, task). Trajectory-aware routing accounts for inter-step dependencies.

**NeoTrix Application**:
- NT-CORE: GWT routes at branch level, not domain level. Each consciousness branch gets different attention mode.
- NT-ACT: Tool routing at step level, accounting for downstream dependencies.

## Axiom Validation

| Axiom | Papers Supporting | Evidence |
|-------|-------------------|----------|
| **A1: Cost-Aware Routing** | BoundaryRouter, Gated-Memory, Agora | 60.6% time reduction, 31.9% cost reduction, 72% cost reduction (AgentRouter) |
| **A2: Context as Scarce Resource** | Flux Attention, Sketch&Walk | 2.8× speedup via context-aware routing, 6× via 20% density sparse attention |
| **A3: Skill as Production Template** | Agora, Gated-Memory | Calibrated competence = skill template quality scoring; Memory gates = skill composition |

## Integration Priority Matrix

| Pattern | Effort | Impact | Priority |
|---------|--------|--------|----------|
| Write Gate for NT-MEMORY | Medium | High | P0 |
| Adaptive Halting for SEAL | Low | High | P0 |
| Early Experience Bootstrap | Low | High | P0 |
| Calibrated Self-Assessment | Medium | High | P1 |
| Layer Router for GWT | High | Medium | P1 |
| Walk Mechanism for E8 | High | Medium | P2 |
| Auction for Tool Selection | Medium | Medium | P2 |
| Training-Free Optimizations | Low | Medium | P1 |
