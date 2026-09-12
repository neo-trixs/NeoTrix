# Model Reverse Engineering — Cycle 416 (2026-09-12)

**Sources**: arXiv (2026), ICML 2026 workshops, ProductHunt
**Focus**: Efficient inference, attention mechanisms, agent coordination
**Mapping**: 7 NeoTrix domains

---

## Model 1: Flux Attention — Context-Aware Hybrid Attention

**Paper**: [2604.07394] Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference
**Authors**: Quantong Qiu et al. | **Date**: 2026-04-08
**URL**: https://arxiv.org/abs/2604.07394

### Core Innovation
Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA) based on input context. A lightweight Layer Router inserted into frozen pretrained LLMs adaptively routes each layer to FA or SA.

### Technical Details
- **Training cost**: 12 hours on 8×A800 GPUs (parameter-efficient)
- **Speed**: Up to 2.8× prefill, 2.0× decode speedup
- **Mechanism**: Layer-wise routing preserves high-fidelity information retrieval while ensuring contiguous memory access
- **Key insight**: Head-level dynamic sparsity creates load imbalance; layer-level routing avoids this

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | GWT attention routing | Layer-level salience-based attention switching |
| **NT-MEMORY** | KV cache optimization | Contiguous memory access pattern for SSD-tier KV |
| **NT-PHYSICAL** | Hardware efficiency | Adjacent memory access enables GPU acceleration |
| **NT-MIND** | Self-evolution | Layer router trained on frozen model — retrofit pattern |

### Actionable Insight
Flux Attention's layer-level router is isomorphic to NT-CORE's GWT: each layer makes a routing decision based on context salience. The 12-hour training cost on frozen weights proves this can be done as a **retrofit** without full retraining — aligns with NT-MIND's SEAL pipeline philosophy.

---

## Model 2: SparDA — Sparse Decoupled Attention with Forecast Projections

**Paper**: [2606.04511] SparDA: Sparse Decoupled Attention for Efficient Long-Context LLM Inference
**Authors**: Yaosheng Fu et al. | **Date**: 2026-06-03
**URL**: https://arxiv.org/abs/2606.04511

### Core Innovation
Fourth per-layer projection called "Forecast" alongside Q/K/V. Forecast predicts KV blocks needed by the next layer, enabling lookahead selection that overlaps CPU-to-GPU prefetch with current-layer execution.

### Technical Details
- **Parameters added**: <0.5% overhead
- **Speed**: 1.25× prefill, 1.7× decode, up to 5.3× decode throughput via larger batch sizes
- **Architecture**: Forecast is decoupled from attention query — one Forecast head per GQA group
- **Training**: Only Forecast projections trained, matching original selector's attention distribution

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Predictive routing | Forecast ≈ predicting which KB nodes will be salient next |
| **NT-MEMORY** | Prefetch optimization | Predict which memories will be needed, prefetch from SSD |
| **NT-WORLD** | Speculative crawling | Predict which content sources will be relevant |
| **NT-ACT** | Pipeline overlap | Overlap data preparation with current execution |

### Actionable Insight
The "Forecast" projection pattern is directly transferable to NT-MEMORY's KV cache: predict which memory blocks will be needed for the next attention step, prefetch from SSD tier. This is a **hardware-level KVMem optimization** that reduces PCIe transfer bottleneck.

---

## Model 3: AgentRouter — Trajectory-Aware Model Routing (ICML 2026)

**Paper**: AgentRouter: Heterogeneous Model Routing for Cost-Optimal Multi-Step Agentic Workflows
**Authors**: Rudrendu Kumar Paul, Sourav Nandy | **Venue**: ICML 2026 (AdaptFM + LM4PLAN + AgenticUQ workshops)
**URL**: https://icml.cc/media/icml-2026/Slides/75165.pdf

### Core Innovation
12M-parameter classifier that assigns model tier per trajectory step in <5ms. Models downstream impact at routing moment — before any LLM call is dispatched. Trajectory-aware vs single-turn routing.

### Technical Details
- **Classifier**: 12M parameters, <5ms inference, 5 features (no LLM needed at routing time)
- **Model tiers**: T1 Frontier (code generation, analysis), T2 Mid-range, T3 Efficient (summarization, classification), T4 Minimal (7B-class, retrieval, formatting)
- **Results**: 72% cost reduction vs frontier-only, <3% quality loss
- **vs RouteLLM**: 72% vs 31% reduction
- **vs FrugalGPT**: 72% vs 44% reduction
- **Key insight**: Single-turn routers fail on trajectories — corrupted intermediate steps degrade every subsequent step

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Trajectory-aware salience | Model routing must account for inter-step quality dependencies |
| **NT-ACT** | Cost-aware execution | Step-level model tier assignment for multi-step tasks |
| **NT-SHIELD** | Quality governance | Prevent cheap-model-induced corruption from propagating |
| **NT-MIND** | Self-evolution | Track which model tiers succeed/fail per task type |

### Actionable Insight
AgentRouter's core finding — **single-turn routing breaks in agentic workflows** — directly challenges NeoTrix's current per-call routing. The GWT must model trajectory-level dependencies: if step N uses a cheap model and produces subtly wrong output, step N+1's frontier model receives corrupted context. This validates NeoTrix's Axiom A1 (Cost-Aware Routing) but adds the constraint that routing decisions must be **trajectory-aware**, not per-call.

---

## Model 4: GLIDE — Guided Layerwise Hybrid Attention

**Paper**: [2607.24788] GLIDE: Guided Layerwise Hybrid Attention for Efficient LLM Inference
**Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass | **Date**: 2026-06-26
**URL**: https://arxiv.org/abs/2607.24788

### Core Innovation
Layer-wise heterogeneity insight: early layers are sensitive to softmax removal, deeper layers tolerate aggressive replacement by linear alternatives. Non-uniformly compresses softmax footprint across model.

### Technical Details
- **Mechanism**: Each layer balances efficient linear recurrence with variable-sized softmax window
- **Key insight**: Layer-wise heterogeneity — not all layers need same attention mechanism
- **Result**: Reduces aggregate KV cache I/O while preserving expressive power where most vital

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Adaptive processing | Different cognitive layers for different task phases |
| **NT-MEMORY** | Tiered storage | Hot data in full-attention layers, cold data in linear layers |
| **NT-MIND** | Self-optimization | Profile which layers are sensitive, allocate compute accordingly |
| **NT-PHYSICAL** | Resource allocation | Match hardware resources to layer importance |

### Actionable Insight
GLIDE's layer-wise heterogeneity maps to NeoTrix's 6-layer architecture: L5 Cognition (full attention for reasoning), L1-L3 Action/Perception/Embodiment (linear/efficient for I/O-bound tasks). The non-uniform compression strategy should inform how NeoTrix allocates KV cache budget across layers — full-fidelity for reasoning layers, aggressive compression for perception/action layers.

---

## Model 5: LLM-Coordination — Multi-Agent Theory of Mind

**Paper**: LLM-Coordination: Evaluating and Analyzing Multi-agent Coordination Abilities in LLMs
**Authors**: Saaket Agashe et al. | **Venue**: NAACL 2025 Findings
**URL**: https://aclanthology.org/2025.findings-naacl.448/

### Core Innovation
Benchmark framework for evaluating LLM coordination in pure coordination settings (zero-sum where cooperation maximizes gains). Tests Environment Comprehension, Theory of Mind reasoning, and Joint Planning.

### Technical Details
- **Tasks**: 4 pure coordination games + 198 multiple-choice CoordQA questions
- **Key finding**: LLM-Agents excel when decisions rely on environmental variables but struggle with active consideration of partners' beliefs/intentions
- **ZSC**: LLM agents exhibit robustness to unseen partners (unlike RL methods)
- **Gap**: Significant room for improvement in Theory of Mind reasoning and joint planning

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** | Agent coordination | Theory of Mind for multi-agent GWT routing |
| **NT-ACT** | Collaborative execution | Joint planning across NT-* domain agents |
| **NT-MEMORY** | Shared beliefs | Track other agents' inferred mental states |
| **NT-FEEL** | Social emotion | Trust/intention inference for agent coordination |

### Actionable Insight
The finding that LLMs struggle with Theory of Mind in coordination has direct implications for NeoTrix's multi-domain coordination (NT-CORE ↔ NT-MIND ↔ NT-ACT). The GWT broadcast mechanism assumes other modules' mental states; the LLM-Coordination benchmark suggests this assumption breaks down. Solution: explicit belief state tracking in the NT-MEMORY shared state layer, making other modules' "intentions" explicit rather than inferred.

---

## Cross-Model Synthesis

### Key Insights for NeoTrix

| Insight | Source | Implementation Priority |
|---------|--------|----------------------|
| Layer-level routing > head-level routing | Flux Attention | P0: GWT should route at layer granularity |
| Forecast projections for KV prefetch | SparDA | P0: NT-MEMORY SSD-tier prefetch |
| Trajectory-aware > per-call routing | AgentRouter | P0: GWT must model inter-step dependencies |
| Layer-wise heterogeneity | GLIDE | P1: Different attention strategies per NT layer |
| Theory of Mind for agent coordination | LLM-Coordination | P1: Explicit belief state in shared memory |
| 12-hour retrofit training on frozen models | Flux Attention | P2: SEAL pipeline can retrofit attention patterns |
| <0.5% overhead for Forecast projections | SparDA | P2: Cost-effective addition to existing models |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Flux Attention says layer-level routing; SparDA says decouple Forecast from query | Complement: layer-level routing for attention type, Forecast for KV prefetch — different concerns |
| AgentRouter says trajectory-aware routing needs 12M params; GWT needs real-time | AgentRouter classifier runs in <5ms — feasible for GWT salience computation |
| LLM-Coordination shows ToM weakness; NeoTrix agents must coordinate | Explicit belief state tracking (not inference) in shared memory layer |
| GLIDE says early layers are sensitive; SparDA says add 4th projection | Different mechanisms: GLIDE replaces attention type, SparDA adds prediction head |

### Absorption into NeoTrix Domains

```
NT-CORE:    Flux Attention layer-level router → GWT refinement
NT-MEMORY:  SparDA Forecast → SSD-tier KV prefetch engine
NT-ACT:     AgentRouter trajectory-aware → cost-aware execution planner
NT-MIND:    Flux Attention 12-hour retrofit → SEAL pipeline self-evolution
NT-REPAIR:  Agnost AI production traces → failure detection from real traffic
NT-SHIELD:  IQ Routing session governance → cost/quality enforcement
NT-WORLD:   Flare dependency graph → codebase topology visualization
```
