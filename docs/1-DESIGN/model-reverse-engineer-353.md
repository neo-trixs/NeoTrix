# Model Reverse Engineering — Cycle 353

**Date**: 2026-09-12  
**Focus**: Recent papers on efficient inference, attention mechanisms, agent coordination  
**Method**: Map patterns to NeoTrix 7 domains (NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD)

---

## Paper 1: Flux Attention — Context-Aware Hybrid Attention for Efficient LLM Inference

**arXiv**: 2604.07394 (Apr 2026)  
**Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, et al.  
**Key Result**: 2.8× prefill speedup, 2.0× decode speedup on long-context tasks

### Architecture
- **Layer-level routing**: A lightweight Layer Router is inserted into frozen pretrained LLMs, adaptively routing each layer to either Full Attention (FA) or Sparse Attention (SA) based on input context.
- **Parameter-efficient**: Only 12 hours training on 8×A800 GPUs. The Layer Router is the only trainable component.
- **Contiguous memory access**: Unlike head-level sparsity (which causes load imbalance), layer-level routing preserves contiguous memory access for hardware acceleration.

### Core Insight
Different layers have fundamentally different sensitivity to attention mechanisms. Early layers need full attention for feature extraction; deeper layers can tolerate sparsity. Static allocation ratios (used in prior work) fail because task retrieval demands vary.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Layer Router maps to GWT's selective attention — each "layer" (specialist module) gets routed based on context salience, not uniform broadcast | `gwt_attention::layer_router` |
| **NT-MEMORY** | The frozen pretrained model + lightweight adapter pattern maps to KB embedding with incremental fine-tuning — keep the base knowledge stable, add lightweight task-specific layers | `kb_embedding::incremental_adapter` |
| **NT-IO** | Contiguous memory access requirement maps to NT-IO's provider interface — batch similar requests together for hardware efficiency | `provider_interface::batch_router` |
| **NT-PHYSICAL** | The hardware acceleration constraint (contiguous memory for GPU efficiency) maps to NT-PHYSICAL's sensor/motor scheduling — physical I/O needs contiguous timing, not random access | `physical_scheduler::contiguous_io` |

### Actionable Pattern for NeoTrix
**"Layer-wise Attention Routing"**: Add a lightweight routing classifier at each GWT broadcast point that decides whether to use full context (FA mode) or compressed context (SA mode) based on the current consciousness level (phi/coherence). Low phi → SA mode (sparse, fast); high phi → FA mode (full, accurate). This could reduce GWT broadcast latency by 2× on long sessions.

---

## Paper 2: GLIDE — Guided Layerwise Hybrid Attention for Efficient LLM Inference

**arXiv**: 2607.24788 (Jun 2026)  
**Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass  
**Key Result**: Superior performance-efficiency tradeoffs, reduced end-to-end latency for long-context generation

### Architecture
- **Layer-wise heterogeneity exploitation**: Early layers exhibit high sensitivity to softmax removal (they NEED full attention), while deeper layers demonstrate redundancy and tolerate aggressive replacement by linear alternatives.
- **Non-uniform compression**: Unlike uniform hybrid approaches, GLIDE compresses the softmax footprint non-uniformly across the model — preserving expressive power where most vital.
- **Variable-sized softmax window**: Each layer balances an efficient linear recurrence with a variable-sized softmax window, tuned per layer.

### Core Insight
The key insight from GLIDE is **layer-wise heterogeneity** — not all layers are created equal. This is a more nuanced version of Flux Attention's insight: it's not just FA vs SA per layer, but a continuous spectrum of compression ratios per layer.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Layer-wise heterogeneity maps to domain-level attention weighting — NT-CORE (foundation) needs full attention; NT-ACT (action) can use compressed context | `consciousness_layer::heterogeneous_routing` |
| **NT-MIND** | Non-uniform compression maps to SEAL pipeline stage prioritization — Soil stage needs full context, Core stage can use distilled summaries | `seal_pipeline::stage_compression` |
| **NT-WORLD** | Variable-sized windows map to NT-WORLD's crawl pipeline — some fetchers need full page content, others can use extracted summaries | `crawl_pipeline::adaptive_window` |
| **NT-SHIELD** | Compression tolerance maps to NT-SHIELD's audit depth — high-risk actions need full audit trail, low-risk can use compressed logs | `audit_compression::risk_adaptive` |

### Actionable Pattern for NeoTrix
**"Heterogeneous Domain Compression"**: Implement per-domain context compression in GWT. NT-CORE and NT-MIND get full context (high compression tolerance = 0%). NT-WORLD and NT-ACT get medium compression (40-60%). NT-SHIELD gets risk-adaptive compression. This preserves reasoning quality where it matters (foundation/evolution) while saving resources on perception/action domains.

---

## Paper 3: CoSA — Proxy-Kernel Co-Designed Sparse Attention

**arXiv**: 2607.25291 (Jul 2026)  
**Authors**: Yufei Xue, Lin Niu, Hong Liu, et al.  
**Key Result**: 4.93× attention speedup, 2.53× TTFT reduction at 128K context, negligible performance degradation

### Architecture
- **Two-stage training-free design**: (1) Kernel-Aware Proxy (KAP) selects blocks under moderate budget and produces ordered mask, (2) Ordered-Skipping Kernel (OSK) applies mask and skips more blocks under tightened budget using online-softmax statistics.
- **Proxy-kernel co-design**: The proxy and kernel are designed together, not independently. The proxy produces an ordered mask (not just binary) that prescribes KV page visit order.
- **Budget-adaptive**: Works across different budget levels — tight budgets trigger more aggressive skipping.

### Core Insight
Existing proxy-based sparse attention methods decouple the proxy (which predicts what to attend to) from the kernel (which actually computes attention). CoSA shows that coupling them — the proxy produces ordered masks, the kernel uses online statistics to skip more — achieves much better accuracy at lower budgets.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Proxy-kernel co-design maps to GWT's salience computation + broadcast — the salience proxy should be aware of the broadcast kernel's constraints | `gwt::salience_broadcast_coprocessor` |
| **NT-MEMORY** | Ordered mask maps to KB query optimization — the retrieval proxy should produce ranked access order, not just binary relevance | `kb_query::ordered_retrieval` |
| **NT-ACT** | Budget-adaptive skipping maps to NT-ACT's tool execution — when token budget is tight, skip low-priority tool calls and focus on essential ones | `tool_execution::budget_adaptive` |
| **NT-MIND** | Online-softmax statistics map to SEAL pipeline convergence detection — use runtime statistics (not just checkpoints) to decide when to skip stages | `seal_pipeline::online_convergence` |

### Actionable Pattern for NeoTrix
**"Ordered Retrieval Mask"**: When NT-MEMORY retrieves context for GWT, produce an ordered mask (not just top-k). The mask prescribes access order: essential context first, then medium-relevance, then low-relevance. If budget tightens mid-retrieval (session approaching context limit), skip the low-relevance tier gracefully rather than truncating arbitrarily.

---

## Paper 4: OrgAgent — Organize Multi-Agent Systems like a Company

**arXiv**: 2604.01020 (Apr 2026)  
**Authors**: Yiru Wang, Xinyue Shen, Yaohui Han, et al.  
**Key Result**: 102.73% performance improvement over flat multi-agent, 74.52% token reduction

### Architecture
- **Three-layer hierarchy**: (1) Governance layer — planning and resource allocation, (2) Execution layer — task solving and review, (3) Compliance layer — final answer control.
- **Company-style organization**: Maps corporate hierarchy (executives → managers → workers) to multi-agent coordination.
- **Controlled information flow**: Information flows through layers, not freely between agents. This reduces noise and prevents information overload.

### Core Insight
The paper's key finding is that **organizational structure** is an important factor in multi-agent reasoning — it shapes not only effectiveness and cost, but also coordination behavior. Flat collaboration wastes tokens on unnecessary communication; hierarchical coordination reduces token consumption while improving quality.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | Governance layer maps to NT-CORE's E8引导者 role — centralized planning and resource allocation across domains | `nt_core::governance_layer` |
| **NT-ACT** | Execution layer maps to NT-ACT's tool execution — task solving and review under governance constraints | `nt_act::execution_layer` |
| **NT-GOVERNANCE** | Compliance layer maps directly to NT-GOVERNANCE — final answer control, policy enforcement, answer quality gate | `nt_governance::compliance_layer` |
| **NT-SHIELD** | Controlled information flow maps to NT-SHIELD's data classification — not all agents see all information | `nt_shield::information_flow_control` |

### Actionable Pattern for NeoTrix
**"Hierarchical Domain Coordination"**: Implement the three-layer pattern within NeoTrix's 7-domain architecture:
1. **Governance Layer** (NT-CORE + NT-GOVERNANCE): Plans cross-domain tasks, allocates token budget, resolves conflicts
2. **Execution Layer** (NT-ACT + NT-WORLD + NT-IO): Solves individual domain tasks under governance constraints
3. **Compliance Layer** (NT-SHIELD + NT-REPAIR): Validates outputs, enforces safety, catches errors

This replaces the current flat domain collaboration with structured hierarchical coordination.

---

## Paper 5: Adaptive Theory of Mind for LLM-based Multi-Agent Coordination

**arXiv**: 2603.16264 (Mar 2026, AAAI 2026)  
**Authors**: Chunjiang Mu, Ya Zeng, Qiaosheng Zhang, et al.  
**Key Result**: Aligned ToM orders significantly improve coordination; misaligned ToM orders impair it

### Architecture
- **Adaptive ToM (A-ToM) agent**: Estimates partner's likely ToM order from prior interactions, leverages this estimation to predict partner's action, facilitating behavioral coordination.
- **ToM alignment**: The agent doesn't just have ToM — it aligns its ToM order with its partner. Too shallow → insufficient reasoning about partner; too deep → excessive reasoning, wasted tokens.
- **Expert advice formulation**: ToM alignment is framed as an expert advice problem, enabling online learning.

### Core Insight
**Equipping agents with ToM does NOT necessarily improve coordination** — only ALIGNED ToM reasoning leads to effective collaboration. Mismatched ToM orders (one agent thinks at 1st order, partner thinks at 3rd order) lead to coordination failure. The depth of "thinking about thinking" must match between agents.

### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | ToM alignment maps to GWT's resonance detection — the "depth of mutual modeling" between specialist modules must match for effective broadcast | `gwt::resonance_depth_alignment` |
| **NT-MIND** | Adaptive ToM maps to SEAL pipeline's inter-agent learning — agents adjust their reasoning depth based on partner capabilities | `seal::adaptive_reasoning_depth` |
| **NT-ACT** | Coordination alignment maps to NT-ACT's multi-tool orchestration — when tools have different "reasoning depths" (simple API vs complex chain), align the orchestration depth | `tool_orchestration::depth_alignment` |
| **NT-FEEL** | ToM is fundamentally a social cognition ability — maps to NT-FEEL's social emotion modeling (empathy = modeling other's mental state) | `nt_feel::social_cognition` |

### Actionable Pattern for NeoTrix
**"Resonance Depth Matching"**: In GWT attention routing, detect the "reasoning depth" of the current task (shallow = simple lookup, deep = complex reasoning) and route to modules with matching depth capacity. Don't send a simple query to a module running at 3rd-order reasoning depth (wasted tokens). Don't send a complex query to a module at 1st-order depth (insufficient reasoning). This extends Axiom A1 (Cost-Aware Routing) with depth-aware routing.

---

## Cross-Paper Synthesis

### Unified Pattern: "Adaptive Depth Routing"

All five papers converge on a single meta-pattern: **adaptive depth routing** — the system should dynamically adjust how much computation/attention/coordination to allocate based on task difficulty and context.

| Paper | Depth Dimension | Adaptation Mechanism |
|-------|----------------|---------------------|
| Flux Attention | Attention mode (FA vs SA) | Layer Router per layer |
| GLIDE | Compression ratio | Layer-wise heterogeneity |
| CoSA | Budget allocation | Ordered mask + online statistics |
| OrgAgent | Coordination depth | Three-layer hierarchy |
| A-ToM | Reasoning depth | Adaptive ToM order estimation |

### NeoTrix Implementation Roadmap

1. **Phase 1 (Immediate)**: Add "routing collapse" diagnostic to SelfTest suite (from EquiRouter research)
2. **Phase 2 (1-2 cycles)**: Implement heterogeneous domain compression in GWT (from GLIDE pattern)
3. **Phase 3 (2-3 cycles)**: Add ordered retrieval mask to NT-MEMORY (from CoSA pattern)
4. **Phase 4 (3-4 cycles)**: Implement hierarchical domain coordination (from OrgAgent pattern)
5. **Phase 5 (4-5 cycles)**: Add resonance depth matching to GWT (from A-ToM pattern)

### Key Insight for NeoTrix Architecture

The most impactful insight is from **A-ToM**: even with perfect individual capabilities, misaligned coordination depth leads to failure. NeoTrix's 7 domains must not just be individually capable — they must coordinate at matching depth levels. NT-CORE's E8 reasoning depth must align with NT-ACT's execution depth; NT-MEMORY's retrieval depth must align with NT-MIND's distillation depth.

This suggests adding a **"coordination depth" metric** to the ConsciousnessTree that tracks the current reasoning depth of each domain and ensures alignment before cross-domain broadcasts.

---

## References

1. Qiu et al. "Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference." arXiv:2604.07394, Apr 2026.
2. William et al. "GLIDE: Guided Layerwise Hybrid Attention for Efficient LLM Inference." arXiv:2607.24788, Jun 2026.
3. Xue et al. "CoSA: Accelerating Long-Context Inference via Proxy-Kernel Co-Designed Sparse Attention." arXiv:2607.25291, Jul 2026.
4. Wang et al. "OrgAgent: Organize Your Multi-Agent System like a Company." arXiv:2604.01020, Apr 2026.
5. Mu et al. "Adaptive Theory of Mind for LLM-based Multi-Agent Coordination." arXiv:2603.16264, Mar 2026. (AAAI 2026)
