# Model Reverse Engineering — Cycle 409 (2026-09-12)

## Selection Criteria
- Recent papers (2026) on efficient inference, attention, agent coordination
- Novel patterns applicable to NeoTrix 7 domains
- Focus on papers with concrete mechanisms (not just benchmarks)

---

## 1. Agora: Confidence-Calibrated Auction for Agent Task Allocation
**Paper**: arXiv:2607.09600 (Aug 2026)
**Authors**: Multi-institutional

### Core Mechanism
Reformulates task allocation as a confidence-calibrated auction. Each reasoning step is a "tradeable item" routed to the most competent agent based on calibrated probability of success, not raw confidence.

### Architecture
```
Planning → Calibration → Auction → Execution → Refinement
```

**Key Innovation**: Calibration step converts raw LLM confidence into calibrated competence scores. Prevents overconfident but incompetent agents from winning critical logic nodes.

### NeoTrix Domain Mapping

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-CORE | GWT salience routing | Replace fixed salience weights with auction-based dynamic allocation |
| NT-ACT | Capability Registry | Add calibrated competence scores to capability nodes |
| NT-MIND | Skill evolution | Track agent competence drift over time for self-evolution |
| NT-IO | Provider selection | Auction-based provider routing instead of total_calls rotation |

### Reverse Engineering Value
- **Pattern**: Market mechanisms for agent coordination (P1 + P4 hybrid)
- **Actionability**: HIGH — implement auction layer in GWT router with calibrated scores from SelfModel performance tracking
- **Risk**: Calibration quality depends on diverse training data; may need Thompson Sampling fallback

---

## 2. GLIDE: Guided Layerwise Hybrid Attention
**Paper**: arXiv:2607.24788 (Jun 2026)
**Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass

### Core Mechanism
Exploits layer-wise heterogeneity in attention sensitivity. Early layers: high sensitivity to softmax removal. Deep layers: tolerate aggressive linear replacement. Each layer independently balances linear recurrence with variable-sized softmax window.

### Architecture
```
Layer i → [Linear Recurrence] + [Variable Softmax Window] → Output
                     ↓
         Layer-wise Adaptive Gate
```

**Key Innovation**: Non-uniform compression across model layers. Uniform hybrid approaches waste compute on insensitive layers and under-protect sensitive ones.

### NeoTrix Domain Mapping

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-CORE | E8 Hexagram reasoning | Early reasoning branches need full softmax; later branches can compress |
| NT-MEMORY | KV cache management | Layer-differentiated cache retention policy |
| NT-PHYSICAL | Attention budget | Per-layer compute allocation based on sensitivity |

### Reverse Engineering Value
- **Pattern**: Heterogeneous resource allocation across architectural layers
- **Actionability**: MEDIUM — applicable to ConsciousnessTree's 6-layer architecture where L1-L3 (action/perception) need different attention than L5-L6 (cognition/meta)
- **Insight**: Layer-awareness should be first-class in GWT routing

---

## 3. Flux Attention: Context-Aware Layer Router
**Paper**: arXiv:2604.07394 (Apr 2026)
**Authors**: Quantong Qiu et al.

### Core Mechanism
Lightweight Layer Router inserted into frozen pretrained LLMs. Router outputs binary decision (Full Attention vs Sparse Attention) per layer based on input context. Trained with constrained optimization to prevent router degeneration (always choosing FA).

### Architecture
```
Input Context → Layer Router → Binary Decision per Layer
                                  ↓
              FA (Full Attention) or SA (Sparse Attention)
```

**Key Innovation**: Dynamic penalty mechanism prevents router from degenerating to all-FA mode. Balances quality vs efficiency through tunable sparsity constraint.

### NeoTrix Domain Mapping

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-CORE | GWT attention gating | Context-aware routing between dense/sparse attention per domain module |
| NT-MIND | SEAL pipeline | Router-like mechanism for deciding evolution depth per cycle |
| NT-SHIELD | Security gates | Context-aware security inspection depth (full vs sparse scanning) |

### Reverse Engineering Value
- **Pattern**: Router with degeneration prevention + sparsity constraint
- **Actionability**: HIGH — directly applicable to GWT where salience router can degenerate to broadcasting everything. Add constraint term to prevent attention flooding.
- **Risk**: Router training overhead; may need online adaptation for NeoTrix's dynamic workload

---

## 4. C²KV: Compressed and Composable KV Cache Reuse
**Paper**: arXiv:2607.17715 (Jul 2026, SIGKDD 2026)
**Authors**: Chuheng Du et al.

### Core Mechanism
Lightweight sidecar Extractor with learnable compression tokens. Produces position-agnostic, composable KV representations that can be flexibly reused and concatenated without modifying frozen base model. Compression-concatenation co-training strategy.

### Architecture
```
Frozen LLM → Sidecar Extractor → Compressed KV Manifold
                                        ↓
              Position-Agnostic + Composable + Reusable
```

**Key Innovation**: Position-agnostic KV compression enables non-prefix reuse (unlike standard prefix caching). Up to 17x inference speedup on long contexts.

### NeoTrix Domain Mapping

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-MEMORY | KB embedding | Composable knowledge chunks that can be flexibly combined across sessions |
| NT-NEXUS | Cross-session weaving | Position-agnostic knowledge blocks for cross-session composition |
| NT-CORE | KV cache optimizer | Sidecar compression for NeoTrix's internal KV cache |
| NT-WORLD | Content representation | Composable content embeddings for crawl pipeline |

### Reverse Engineering Value
- **Pattern**: Sidecar compressor with composable output (modular, reusable knowledge blocks)
- **Actionability**: HIGH — directly applicable to NT-MEMORY's experience-tree where branch chunks need composable recall. Sidecar pattern avoids modifying core LLM.
- **Risk**: Compression quality degrades on rare/long-tail knowledge; need quality-aware routing

---

## 5. ARCANA: Reflective Multi-Agent Program Synthesis
**Paper**: arXiv:2607.09059 (Jul 2026)
**Authors**: Kunbo Zhang et al.

### Core Mechanism
Four specialized agents communicating through shared differentiable blackboard:
1. **Perceptual Grounding Agent**: Object-centric scene graphs from raw grids
2. **Latent Program Policy**: Diverse DSL program proposals
3. **Symbolic Executor**: Candidate verification on demonstrations
4. **Reflective Agent**: Failure-driven feedback synthesis

Learned meta-controller schedules agent turns.

### Architecture
```
Raw Input → [Perceptual Agent] → Scene Graphs
                                       ↓
                              [Program Policy] → DSL Candidates
                                       ↓
                              [Symbolic Executor] → Verified Solutions
                                       ↓
                              [Reflective Agent] → Failure Feedback
                                       ↓
                              [Meta Controller] → Next Turn
```

**Key Innovation**: Differentiable blackboard for inter-agent communication + learned scheduling. Failure-driven reflection loop (not just retry).

### NeoTrix Domain Mapping

| Domain | Integration Point | Mechanism |
|--------|------------------|-----------|
| NT-CORE | E8 Hexagram reasoning | Differentiable blackboard = shared reasoning state across hexagram branches |
| NT-MIND | SEAL pipeline | Reflective agent = failure-driven distillation (not just success harvesting) |
| NT-ACT | Multi-agent coordination | Meta-controller for agent turn scheduling |
| NT-REPAIR | Self-healing | Failure-driven feedback = repair pattern generation |
| NT-GOVERNANCE | Architecture arbitration | Meta-controller = governance-level scheduling |

### Reverse Engineering Value
- **Pattern**: Differentiable blackboard + failure-driven reflection + learned scheduling
- **Actionability**: MEDIUM-HIGH — differentiable blackboard maps to shared ConsciousnessTree state; failure-driven reflection enhances SEAL pipeline's distillation stage
- **Risk**: Complexity of learned scheduler; may need deterministic fallback for production

---

## Synthesis: Cross-Paper Pattern Matrix

| Pattern | Papers | NeoTrix Priority | Target Domain |
|---------|--------|-----------------|---------------|
| **Market-Based Task Routing** | Agora | P0 | NT-CORE (GWT) + NT-ACT |
| **Layer-Heterogeneous Attention** | GLIDE, Flux | P1 | NT-CORE + NT-MEMORY |
| **Sidecar KV Compression** | C²KV | P0 | NT-MEMORY + NT-NEXUS |
| **Degeneration-Preventing Router** | Flux | P1 | NT-CORE (GWT) |
| **Differentiable Blackboard** | ARCANA | P1 | NT-CORE + NT-MIND |
| **Failure-Driven Reflection** | ARCANA | P1 | NT-MIND (SEAL) + NT-REPAIR |
| **Position-Agnostic Knowledge Blocks** | C²KV | P0 | NT-MEMORY + NT-NEXUS |

## Action Items for NeoTrix

1. **A1 Reinforcement**: Agora + AdCo validate Cost-Aware Routing. Implement auction-based provider selection in NT-ACT alongside total_calls rotation.
2. **KV Cache Evolution**: C²KV sidecar pattern → NT-MEMORY composable experience chunks. Position-agnostic recall enables cross-session knowledge composition.
3. **GWT Degeneration Prevention**: Flux Attention's sparsity constraint → prevent GWT attention flooding. Add quality-aware degeneration penalty.
4. **SEAL Reflection Upgrade**: ARCANA's failure-driven reflection → SEAL pipeline should harvest failure patterns, not just success patterns.
5. **Layer-Aware Architecture**: GLIDE/Flux layer heterogeneity → ConsciousnessTree 6-layer architecture should have per-layer attention budgets.
