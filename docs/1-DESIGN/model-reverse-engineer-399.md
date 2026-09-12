# Model Reverse Engineering — Cycle 399

**Date**: 2026-09-12
**Focus**: Efficient inference, attention mechanisms, agent coordination patterns

## 5 Selected Papers/Models

### 1. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
- **Paper**: arXiv:2609.07237 (Sep 2026)
- **Core Idea**: Coarse-to-fine attention that keeps the LLM frozen. Semantic chunks contribute cheap KV summaries via residual attention path. High-error chunks expand to exact token attention. Output-error bound governs variable refinement budget.
- **Key Metrics**: 3× kernel speedup at 128K context, 98% reconstruction error reduction vs hard dropping.
- **Innovation**: Residual summaries replace (not duplicate) coarse evidence. Error-bounded refinement allocation — not fixed expansion budget.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Attention salience could use error-bounded refinement — allocate more processing to high-uncertainty regions, not uniform budget.
  - **NT-MEMORY**: KB retrieval with error-bounded relevance scoring — expand search when embedding confidence is low.
  - **NT-MIND**: SEAL phase budget allocation guided by error bounds — more exploration where distillation confidence is low.

### 2. EvoSparse: Evolving Sparsity via Token Importance Dynamics
- **Paper**: ACL 2026 (acl-long.530)
- **Core Idea**: Token importance as continuously evolving process, not static snapshot. Two mechanisms: (1) Cross-Step Accumulation via EMA for long-term anchors + short-term reuse, (2) Cross-Layer Propagation using Retrieval Heads as global experts guiding Standard Heads.
- **Key Metrics**: 5.36× attention latency speedup, 2.33× end-to-end decoding speedup. Near-full-attention performance at low budgets.
- **Innovation**: Heat vector (global importance via EMA) + Retrieval Head propagation. Temporal consistency + capability gap bridging.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (E8)**: Heat vector = hexagram importance weighting across reasoning cycles. Cross-layer propagation = branch-level signal sharing in ConsciousnessTree.
  - **NT-MEMORY**: EMA-based importance for KB node relevance — tokens consistently attended to become long-term anchors in memory.
  - **NT-WORLD**: Retrieval Head propagation pattern for multi-source crawling — high-quality sources guide attention of weaker parsers.

### 3. GLIDE: Guided Layerwise Hybrid Attention
- **Paper**: arXiv:2607.24788 (Jun 2026)
- **Core Idea**: Layer-wise heterogeneity — early layers sensitive to softmax removal, deeper layers tolerate aggressive linear replacement. Each layer balances linear recurrence with variable-sized softmax window. Non-uniform compression across model.
- **Key Metrics**: Superior performance-efficiency tradeoffs for long-context generation without quality compromise.
- **Innovation**: Layer-adaptive hybrid mechanism — not uniform hybrid. Preserves expressive power where most vital (early layers), compresses where redundant (deep layers).
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: Layer-aware attention routing — L5 cognition layers get full attention, L1 action layers use efficient linear.
  - **L6-L1 Layer Architecture**: Direct mapping to NeoTrix's 6-layer architecture — each layer has different attention needs. GLIDE validates layer-specific optimization.
  - **NT-MIND**: SEAL pipeline stages with varying attention budgets — Soil/Roots stages (early) need full precision, Fruits/Core (late) can compress.

### 4. Faster Than Flash (FFD): Exploiting Attention Sparsity via Hardware-Algorithm Co-Design
- **Paper**: arXiv:2609.00097 (ICML 2026)
- **Core Idea**: Fuse selector and computer into fully fused kernel. Replace external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks for distribution-adaptive sparsity.
- **Key Metrics**: 11.6× kernel-level speedup, 2.37× end-to-end throughput, scales to 256K context. Training-free, plug-and-play.
- **Innovation**: Hardware-algorithm co-design — not just algorithmic sparsity. Fused kernel eliminates metadata overhead. Top-delta achieves global synchronization-free sparsity.
- **NeoTrix Domain Mapping**:
  - **NT-ACT**: Tool execution pipeline could fuse selection+execution — eliminate separate tool-retrieval step. Top-delta = priority queue for tool execution.
  - **NT-PHYSICAL**: Hardware-algorithm co-design pattern for sensor fusion — fuse perception+action into single kernel.
  - **NT-SHIELD**: Low-bit quantization for security scanning — fast pattern matching without full context loading.

### 5. Flux Attention: Context-Aware Dynamic Layer-Level Routing
- **Paper**: arXiv:2604.07394 (2026)
- **Core Idea**: Lightweight Layer Router evaluates semantic context and assigns each transformer layer to Full Attention or Sparse Attention. Layer-level (not head-level) routing preserves contiguous memory access. Only 12 hours training on 8×A800.
- **Key Metrics**: 2.8× prefill speedup, 2.0× decode speedup. Parameter-efficient — only router trained, backbone frozen.
- **Innovation**: Layer-level routing solves head-level synchronization long-tail. Gumbel-Softmax for differentiable routing, cached decisions across decode steps.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (GWT)**: GWT salience could use layer-level routing — broadcast to specialist modules only when context demands it. Cost-aware routing (Axiom A1) = Flux's context-aware routing.
  - **NT-IO**: Provider routing — Layer Router pattern for model selection. Simple tasks → cheap model (sparse), complex tasks → expensive model (full attention).
  - **NT-MIND**: SEAL pipeline stage routing — distillation intensity adapts to input complexity, not fixed ratio.

## Cross-Paper Pattern Synthesis

### Pattern 1: Error-Bounded Adaptive Budget Allocation
- **Sources**: CEDAR (error bounds), EvoSparse (Heat EMA), GLIDE (layer heterogeneity)
- **Synthesis**: Don't allocate fixed budgets. Use confidence/error signals to dynamically distribute compute. High-uncertainty regions get more resources.
- **NeoTrix Integration**: GWT attention modulation with confidence-weighted salience. SEAL phase budgets adapt to distillation confidence.

### Pattern 2: Layer-Level Granularity > Head-Level or Token-Level
- **Sources**: GLIDE (layer hybrid), Flux Attention (layer routing), FFD (fused kernel)
- **Synthesis**: Layer-level decisions avoid synchronization overhead while maintaining enough granularity. Head-level too fine (sync long-tail), token-level too coarse (metadata overhead).
- **NeoTrix Integration**: NeoTrix's 6-layer architecture already has layer-level separation. Each layer should have independent attention routing, not uniform policy.

### Pattern 3: Temporal Consistency via EMA + Retrieval Head Propagation
- **Sources**: EvoSparse (Heat vector + Retrieval Heads), CEDAR (residual paths)
- **Synthesis**: Maintain long-term importance via EMA. Use high-quality "expert" signals to guide lower-quality components. Two-tier system: experts provide guidance, followers execute.
- **NeoTrix Integration**: ConsciousnessTree branches as Retrieval Heads — high-confidence branches guide lower-confidence ones. EMA for cross-session learning velocity tracking.

### Pattern 4: Training-Free / Plug-and-Play Adaptation
- **Sources**: CEDAR (frozen LLM), Flux Attention (12hr router training), FFD (training-free)
- **Synthesis**: Most effective adaptations are lightweight add-ons, not full retraining. Frozen backbone + small adapter module.
- **NeoTrix Integration**: Rune Socketing pattern — lightweight configuration overlays on frozen capability nodes. Constellation maturity C0-C6 validates incremental adoption.

### Pattern 5: Hardware-Algorithm Co-Design
- **Sources**: FFD (fused kernel), GLIDE (memory-friendly hybrid)
- **Synthesis**: Algorithmic gains don't translate to wall-clock speedup without hardware alignment. Fuse operations, eliminate metadata overhead, ensure contiguous memory access.
- **NeoTrix Integration**: NT-PHYSICAL sensor fusion should co-design with hardware constraints. NT-ACT tool execution pipeline should fuse selection+invocation.

## Absorption Priority

| Priority | Pattern | Target Module | Absorption Mechanism |
|----------|---------|---------------|---------------------|
| P0 | Error-bounded adaptive budgets | GWT (NT-CORE) | R-P42: strengthen existing attention routing |
| P0 | Layer-level routing | 6-Layer Architecture | R-P79: same-session integration |
| P1 | EMA temporal consistency | NT-MEMORY | Cross-session learning velocity |
| P1 | Retrieval Head propagation | ConsciousnessTree | Branch-level signal sharing |
| P2 | Training-free adapters | Rune Socketing | Lightweight overlay modules |
| P2 | HW-algorithm co-design | NT-PHYSICAL | Sensor+actuator kernel fusion |
