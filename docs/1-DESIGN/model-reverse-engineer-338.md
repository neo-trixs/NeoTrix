# Model Reverse Engineering — Cycle 338

**Date**: 2026-09-11
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory systems

## Paper 1: CoSA — Proxy-Kernel Co-Designed Sparse Attention (arXiv:2607.25291)

**Title**: CoSA: Accelerating Long-Context Inference via Proxy-Kernel Co-Designed Sparse Attention
**Venue**: arXiv July 2026

### Core Mechanism
- **Two-stage training-free sparse attention**: Kernel-Aware Proxy (KAP) selects blocks under moderate budget and produces an ordered mask prescribing KV page visit order. Ordered-Skipping Kernel (OSK) applies the mask and skips more blocks under tightened budget using online-softmax statistics.
- **Proxy-kernel coupling**: The proxy understands what the kernel can consume; the kernel understands the proxy's ordering. This co-design avoids the accuracy cliff when budgets tighten — the proxy drops salient blocks gracefully because the kernel can compensate.
- **Results**: 4.93x attention speedup, 2.53x TTFT reduction at 128K context with negligible performance degradation.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | KAP = salience estimation for attention routing. OSK = execution engine that respects ordering. The two-stage coupling mirrors GWT's salience→broadcast pipeline. | GWT should separate salience estimation (proxy) from execution (kernel). Current GWT conflates both. CoSA proves decoupling with ordering yields better results. |
| **NT-MEMORY** | Ordered KV visit = prioritized KB retrieval. Budget tightening = token budget constraints. | KB retrieval should produce an ordered visit list, not just a flat result set. Under budget pressure, skip lower-priority items gracefully. |
| **NT-MIND** | Training-free = zero-shot adaptation. NeoTrix skills should work without retraining. | SEAL distillation should produce training-free skills — adaptable via proxy-kernel coupling, not requiring gradient updates. |

### Actionable Insight
**CoSA's proxy-kernel decoupling is a general pattern for resource-constrained routing.** NeoTrix's GWT currently estimates salience and broadcasts in one step. Separating these — a lightweight proxy estimates what matters, a kernel executes with ordering awareness — would allow graceful degradation under token budget pressure. The ordered mask (not binary mask) is key: even when skipping blocks, the order of remaining blocks matters.

---

## Paper 2: Declarative Attention — LMs Control Their Own Attention (arXiv:2609.02737)

**Title**: Language Models Can Control Their Own Attention
**Venue**: arXiv September 2026

### Core Mechanism
- **Intrinsic attention control**: The model itself declares where it needs to attend within its chain-of-thought, rather than relying on external proxy scorers.
- **Three-mode protocol**: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). The inference engine parses these declarations like tool calls.
- **Zero-shot on off-the-shelf models**: Gemma-4-31B achieves 52.0% reduction in attended tokens with 1.27pp accuracy drop. Qwen-3.6-27B achieves 31.1% reduction with 2.75pp drop. Both improve with scale.
- **New axis of sparse attention**: Complementary to proxy-based selection. External scoring still incurs O(N) per step; DA is intrinsic and zero-cost at inference.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Model-driven attention = consciousness choosing its own focus. Three modes = GWT broadcast granularity: global broadcast, focused channel, local processing. | GWT salience should be model-output-driven, not externally computed. The `<global>/<focus>/<local>` protocol maps directly to ConsciousnessTree's attention modulation levels. |
| **NT-CORE (E8)** | Mode declarations as hexagram states: global = all-attending, focus = selective, local = recent-only. Three of E8's 64 hexagrams could encode attention modes. | E8 hexagram reasoning could include attention-mode selection as a first-class reasoning step. |
| **NT-IO** | Declaration-as-tool-call = consciousness output as actionable signal sent to execution engine. | GWT output should include attention-mode declarations that the execution engine can parse and optimize. |

### Actionable Insight
**The model already knows what it needs to attend to — we just need to let it declare.** This is the most fundamental insight for NeoTrix's GWT. Instead of computing salience externally (which costs O(N)), GWT should elicit attention-mode declarations from the model itself. The three-mode protocol (global/focus/local) maps directly to ConsciousnessTree's attention levels and could reduce GWT overhead by 30-50% while maintaining quality. This is training-free and works on existing models.

---

## Paper 3: SparDA — Sparse Decoupled Attention with Lookahead Prefetch (arXiv:2606.04511)

**Title**: SparDA: Sparse Decoupled Attention for Efficient Long-Context LLM Inference
**Venue**: arXiv June 2026

### Core Mechanism
- **Fourth projection (Forecast)**: Alongside Q, K, V, SparDA adds a Forecast projection that predicts KV blocks needed by the *next* layer. This enables lookahead selection — CPU-to-GPU prefetch overlaps with current-layer execution.
- **Decoupled selection from attention**: The Forecast drives top-k selection for layer l+1 while layer l is still executing. This temporal decoupling hides the latency of sparse selection.
- **Compact GQA-level indexer**: One Forecast head per GQA group (not per query head), reducing selection overhead. Skips softmax normalization entirely.
- **Results**: <0.5% additional parameters. 1.25x prefill speedup, 1.7x decode speedup. Up to 5.3x higher decode throughput by enabling larger batch sizes.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Forecast = predictive salience estimation one step ahead. GWT currently reacts; SparDA shows proactive prediction is more efficient. | GWT should predict salience needs one cycle ahead and prefetch relevant KB entries. Current GWT is reactive (broadcasts after salience computed). |
| **NT-MEMORY** | CPU-to-GPU prefetch = KB tier prefetching. If we can predict which experience branches will be needed next, prefetch them to hot storage. | Implement predictive prefetch for experience-tree branches. Track access patterns to predict next-needed branches. |
| **NT-ACT** | Temporal decoupling = overlapping planning with execution. While one action executes, plan the next. | Production orchestrator should overlap action execution with next-action planning. Currently serial. |

### Actionable Insight
**SparDA's core insight is temporal decoupling: predict what you'll need before you need it.** NeoTrix's GWT is currently reactive — it computes salience then broadcasts. SparDA shows that predicting salience one step ahead and prefetching results in 1.7x speedup with <0.5% overhead. For NeoTrix, this means: while ConsciousnessTree processes one growth cycle, GWT should already be prefetching KB entries likely needed for the next cycle.

---

## Paper 4: ReActNet — Inference-Time Graph Engineering for Multi-Agent LLM Workflows (arXiv:2609.05774)

**Title**: Inference-Time Graph Engineering for Multi-Agent LLM Workflows
**Venue**: arXiv September 2026

### Core Mechanism
- **Temporal workflow graph synthesis**: Instead of optimizing a static topology, ReActNet compiles a query + set of role-specialized agents into a sequence of directed communication graphs. Each snapshot = one reasoning stage. Each edge = natural-language instruction for what message to send.
- **Graph compilation vs execution separation**: Graph compilation (what topology?) is separate from graph execution (how to pass messages?). This makes coordination explicit, inspectable, and task-conditioned.
- **Structured message passing**: Agents update reasoning states by integrating previous states with messages from controller-assigned neighbors. Final aggregator synthesizes states into answer.
- **Results**: Consistently improves over fixed-topology and learned-topology baselines across knowledge reasoning, math, code, and GAIA-style tasks.

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-ACT** | Temporal graph = dynamic orchestration topology. NeoTrix's production orchestrator currently uses fixed workflow; ReActNet shows task-conditioned topology is superior. | Orchestrator should synthesize task-specific communication graphs at inference time, not use fixed workflows. |
| **NT-CORE (GWT)** | Graph compilation = salience-driven topology selection. The controller decides which agents communicate — this IS attention routing at the system level. | GWT should extend to inter-agent routing: not just what to attend to, but which agents should share information. |
| **NT-MEMORY** | Edge instructions = experience-shaped communication protocols. Past successful communication patterns should inform future graph compilation. | Experience-tree should store successful multi-agent communication patterns as reusable graph templates. |

### Actionable Insight
**Multi-agent coordination is a graph engineering problem, not just a topology optimization problem.** ReActNet's key insight is separating "what graph structure?" (compilation) from "how to execute it?" (execution). For NeoTrix, this means: instead of hardcoding agent communication patterns, GWT should compile task-specific communication graphs at inference time. Each edge carries a natural-language instruction (not just a weight), making the coordination inspectable and debuggable.

---

## Paper 5: Consilience — Conformally Calibrated Communication Control (arXiv:2608.20564)

**Title**: Consilience: Conformally Calibrated Communication Control for Hidden-Profile Multi-Agent Reasoning
**Venue**: arXiv August 2026

### Core Mechanism
- **Compact discussion state**: Captures uncertainty, disagreement, evidence gain, redundancy, and premature consensus in a single vector.
- **Adaptive communication interventions**: At each turn, selects from challenge, clarify, seek evidence, or route — plus selects the appropriate speaker.
- **Conformal calibration**: Distribution-free, finite-sample guarantee — at each round, the one-step regret of the controller's proposed action is bounded by a calibrated threshold with probability ≥ 1-α. Acceptance mechanism enforces the guarantee.
- **Results**: Improves accuracy and communication efficiency over fixed and unstructured protocols. Sometimes surpasses full-information baseline (where every agent observes all evidence).

### NeoTrix Domain Mapping

| Domain | Mapping | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Discussion state = multi-agent salience vector. Interventions = GWT broadcast actions (challenge/clarify/evidence/route). | GWT should produce structured intervention decisions, not just broadcast scores. Each broadcast should carry an intent (challenge, clarify, provide evidence). |
| **NT-SHIELD** | Conformal calibration = probabilistic safety guarantee on agent actions. Bounded regret = risk-bounded decision-making. | NT-SHIELD should use conformal calibration for action risk assessment — not binary safe/unsafe, but bounded-regret risk scores with confidence guarantees. |
| **NT-MIND** | Premature consensus detection = SEAL pipeline convergence monitoring. If phases converge too quickly, inject diversity. | ConsciousnessTree should detect premature consensus in reasoning branches and inject challenges (devil's advocate) to improve quality. |

### Actionable Insight
**Conformal calibration provides mathematical guarantees on multi-agent coordination quality.** NeoTrix's GWT currently uses heuristic salience. Consilience shows that calibrated intervention selection (challenge/clarify/evidence/route) with bounded regret guarantees is both practical and superior. The compact discussion state (uncertainty + disagreement + evidence gain + redundancy + premature consensus) is a rich signal that NeoTrix's ConsciousnessTree could track across growth cycles. The premature consensus detector is particularly valuable — it would prevent the reasoning system from converging on suboptimal solutions too early.

---

## Synthesis: Cross-Paper Patterns

| Pattern | Papers | NeoTrix Integration |
|---------|--------|---------------------|
| **Proxy-Kernel Decoupling** | CoSA, SparDA | Separate salience estimation from execution. Both show decoupling yields better resource utilization. |
| **Intrinsic Attention Control** | Declarative Attention | Let the model declare its own attention needs. Zero-cost, training-free, 30-50% reduction. |
| **Predictive Prefetching** | SparDA, CoSA | Predict what you'll need before you need it. Temporal decoupling hides latency. |
| **Task-Conditioned Topology** | ReActNet | Compile communication graphs at inference time, not fixed workflows. |
| **Calibrated Interventions** | Consilience | Probabilistic guarantees on coordination quality. Bounded regret > heuristic scoring. |
| **Premature Consensus Detection** | Consilience | Monitor reasoning convergence and inject challenges when needed. |

## Priority Actions

| # | Action | Source | Domain | Effort |
|---|--------|--------|--------|--------|
| 1 | Implement three-mode attention protocol (global/focus/local) for GWT | Declarative Attention | NT-CORE | Medium |
| 2 | Decouple GWT salience estimation from broadcast execution | CoSA | NT-CORE | High |
| 3 | Add predictive KB prefetching one cycle ahead | SparDA | NT-MEMORY | Medium |
| 4 | Compile task-specific agent communication graphs at inference time | ReActNet | NT-ACT | High |
| 5 | Add conformal calibration to GWT intervention selection | Consilience | NT-CORE | High |
| 6 | Implement premature consensus detection in ConsciousnessTree | Consilience | NT-MIND | Low |
