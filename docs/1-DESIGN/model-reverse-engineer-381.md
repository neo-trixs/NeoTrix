# Model Reverse Engineering — Cycle 381

**Date:** 2026-09-12
**Focus:** Efficient inference, attention mechanisms, reasoning optimization, collaborative inference
**Sources:** arXiv (Apr-Sep 2026), ICLR 2026, ACL 2026

---

## 1. Flux Attention — Context-Aware Hybrid Attention

**Source:** arXiv:2604.07394, April 2026

### Key Claims
- Layer-level dynamic routing between full attention and sparse attention
- 2.8× prefill speedup, 2.0× decode speedup
- Only 12 hours training on 8× A800 GPUs (parameter-efficient)
- Maintains performance at 256K context boundary where baselines collapse

### Architecture Insights
- **Problem**: Head-level dynamic sparsity creates load imbalance and synchronization long-tails → hardware-unfriendly during autoregressive decoding
- **Solution**: Layer-level routing instead of head-level — lightweight Layer Router decides per-layer whether to use full or sparse attention
- **Key innovation**: Layer-wise routing preserves contiguous memory access → theoretical computational reductions translate to practical wall-clock speedups
- **Bottleneck eliminated**: Static allocation (PruLong, DuoAttention) risks performance collapse; Flux Attention adapts dynamically per context

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Dynamic attention routing | GWT salience computation could use layer-level routing — cheap layers for routine broadcasts, full attention for novel signals |
| NT-IO | Inference optimization | Flux Attention's Layer Router pattern = NT-IO's ordered backend router (P4) at attention level |
| NT-MIND | Self-adaptive processing | Context-aware routing maps to SEAL's adaptive pipeline — difficulty determines compute allocation |

### Absorption
- **Layer Router as GWT primitive**: GWT currently broadcasts uniformly. Flux Attention shows layer-level routing is more hardware-friendly than head-level. GWT could route salience computation: cheap layers for low-salience, full attention for high-salience.
- **256K context stability**: Validates Axiom A2 (context as scarce resource). Flux Attention maintains quality where others collapse — context management is about attention quality, not just compression.
- **Training efficiency**: 12-hour training on 8 GPUs = accessible. NeoTrix could adopt similar lightweight routing for its own attention mechanisms.

---

## 2. LISA — Linear-Indexed Sparse Attention

**Source:** arXiv:2607.19358, May 2026

### Key Claims
- Plug-and-play attention replacement — no pretraining from scratch
- 50% inference speedup at 16K context
- 5.6% average accuracy improvement on AIME + MATH-500
- O(nM) complexity where M << n (vs O(n²) baseline)

### Architecture Insights
- **Problem**: O(n²) self-attention scales poorly for long-CoT reasoning (DeepSeek-R1 style)
- **Solution**: Two parallel branches: (1) Linear Attention for long-range memory at O(n), (2) Lightning Indexer selecting top-M tokens for Sparse Self-Attention
- **Fusion**: Gating mechanism fuses both branches — constant inference cost with precise contextual retrieval
- **Training**: Two-stage — Stage 1 initializes linear attention + sliding-window via KD from frozen teacher; Stage 2 trains Indexer with per-head KL divergence loss
- **Key insight**: Linear attention alone is weak at context-sensitive tasks; combining with sparse attention preserves both efficiency and precision

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Hybrid reasoning modes | Two-branch architecture = E8's dual-path reasoning: symbolic (sparse) + associative (linear) |
| NT-MEMORY | Tiered memory access | Linear Attention = long-range KB retrieval; Sparse Attention = recent session context. Both fused via gating |
| NT-IO | Plug-and-play optimization | No retraining needed → drop-in efficiency gain for any NT-IO inference path |

### Absorption
- **Two-branch memory = NT-MEMORY dual-path**: LISA's Linear Attention (long-range, cheap) + Sparse Attention (precise, expensive) maps directly to NT-MEMORY's KB embedding (long-range) + session context (precise). Gating = attention-weighted retrieval.
- **Lightning Indexer as GWT pre-filter**: Before GWT broadcasts, a lightweight indexer selects high-salience tokens. Reduces broadcast overhead without losing precision.
- **Training-free deployment**: LISA requires no pretraining — just swap attention modules. NeoTrix could adopt this for any transformer-based component without retraining.

---

## 3. DUET — Dual-Model Efficient Two-Stage Inference

**Source:** arXiv:2605.01111, May 2026

### Key Claims
- Decomposes inference into two stages: capable model → reasoning signal; lightweight model → final answer
- Saves up to 60% of large model's output tokens on AIME/GPQA
- Maintains strong reasoning performance with substantially lower cost
- Joint training with length-penalized objective

### Architecture Insights
- **Problem**: Single large model doing end-to-end reasoning is expensive — most tokens are non-reasoning overhead
- **Solution**: Capable model produces compact reasoning signal (not full answer); lightweight model interprets signal to generate answer
- **Training**: Length-penalized joint training — capable model encouraged to transmit only information sufficient for lightweight model to solve task
- **Key insight**: Reasoning is separable from generation. The heavy lifting is in producing the signal, not in the final answer.

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE + NT-ACT | Reasoning-action separation | NT-CORE produces reasoning signal (what to do), NT-ACT executes (how to do it) — lightweight execution, expensive reasoning |
| NT-MIND | SEAL pipeline optimization | SEAL's distillation phase = DUET's reasoning signal — distill experience into compact signals that lightweight models can act on |
| NT-IO | Cost-aware routing | DUET validates Axiom A1 (cost-aware routing) — not all stages need the strongest model |

### Absorption
- **Reasoning-action separation = NT-CORE + NT-ACT**: NT-CORE (E8 reasoning) produces compact decision signals; NT-ACT (tool execution) interprets and acts. Heavy reasoning in CORE, lightweight execution in ACT.
- **SEAL distillation = DUET signal compression**: SEAL's experience distillation already compresses cycle history. DUET's training objective (transmit only sufficient information) could improve SEAL's distillation quality.
- **Axiom A1 validation**: DUET proves cost-aware routing works at inference level — route reasoning to capable models, execution to lightweight ones. This is A1 in action.

---

## 4. AttnPO — Attention-Guided Process Supervision

**Source:** ACL 2026 (2026.acl-long.1845)

### Key Claims
- Identifies special attention heads that naturally focus on essential reasoning steps
- Reduces reasoning length while improving performance across 9 benchmarks
- Two sub-strategies: discourage redundant steps + preserve accuracy on essential steps
- Attention scores as fine-grained supervision signal

### Architecture Insights
- **Problem**: RL-trained reasoning models (RLVR) overthink — generate redundant reasoning without performance gains. Trajectory-level length penalties are too coarse.
- **Solution**: Use attention scores from special heads as step-level supervision — identify which reasoning steps are essential vs redundant
- **Two strategies**: (1) Penalize redundant steps (low-attention) more, (2) Reduce penalty on essential steps (high-attention)
- **Key insight**: Models already have internal signals for step importance — attention scores are a natural, fine-grained supervision mechanism

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Attention-guided reasoning | E8 hexagram grid could use attention scores to prune redundant reasoning paths — focus computation on essential transitions |
| NT-MIND | Self-supervised distillation | Attention scores as supervision signal for SEAL's experience distillation — identify which cycle phases are essential vs noise |
| NT-SHIELD | Overthinking prevention | Guard against runaway reasoning — attention-based early stopping when essential information is captured |

### Absorption
- **Attention scores as step-level RL signal**: NT-MIND's SEAL pipeline could use attention patterns from successful cycles to identify essential vs redundant phases. This is self-supervised process reward.
- **GWT salience refinement**: AttnPO shows attention heads naturally specialize in identifying essential steps. GWT could learn which attention patterns indicate high-salience vs routine broadcasts.
- **Overthinking guard**: NT-SHIELD could implement AttnPO's redundancy detection — monitor attention patterns during SEAL cycles and stop when essential information is captured.

---

## 5. Reflection Steering — Disentangling Reflection from Reasoning

**Source:** arXiv:2608.25542, August 2026

### Key Claims
- Disentangles reflection (verification/revision) from reasoning (problem-solving) in activation space
- Reflection heads and reasoning heads are separable in activation space
- Steering reflection activation enables token-efficient inference
- Reduces overthinking without degrading reasoning quality

### Architecture Insights
- **Problem**: Large reasoning models produce traces with interleaved reasoning + reflection (verification, revision, backtracking). These are distinct processes but entangled in generation.
- **Solution**: Separate activation vectors for reflection vs reasoning — identify which activations drive verification/revision vs genuine reasoning
- **Steering**: Suppress reflection activations when not needed → shorter traces without quality loss
- **Key insight**: Reflection and reasoning are computationally distinct processes that can be independently controlled

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Dual-process reasoning | E8 reasoning = problem-solving; ConsciousnessTree = reflection. Separate activation spaces for each |
| NT-MIND | SEAL process separation | SEAL exploration phase = reasoning; SEAL self-test phase = reflection. Independent control of each |
| NT-REPAIR | Reflection control | Self-healing = reflection process. Can be suppressed during rapid exploration, activated during verification |

### Absorption
- **Dual-process = NT-CORE + ConsciousnessTree**: Reflection Steering's separation validates NeoTrix's explicit separation of reasoning (E8 hexagram) and meta-cognition (ConsciousnessTree). These are distinct processes with distinct activation patterns.
- **SEAL phase control**: SEAL's exploration→distillation→self-test pipeline mixes reasoning and reflection. Reflection Steering shows these can be independently controlled — explore fast (suppress reflection), verify carefully (amplify reflection).
- **NT-REPAIR activation gating**: During self-healing, reflection is essential. During rapid prototyping, reflection is overhead. Steering allows adaptive control based on context.

---

## Cross-Paper Synthesis

### Unified Theme: Attention as Control Plane

All 5 papers treat attention not just as a computation mechanism but as a **control plane**:
- **Flux Attention**: Layer-level attention routing controls compute allocation
- **LISA**: Indexer attention selects which tokens matter for sparse attention
- **DUET**: Reasoning signal attention separates heavy vs lightweight computation
- **AttnPO**: Attention scores supervise which reasoning steps are essential
- **Reflection Steering**: Attention activation space separates reasoning vs reflection

**NeoTrix implication**: GWT (Global Workspace Theory) should evolve from broadcast-only to **attention-as-routing**. Not just "what is salient" but "how much compute does this salience warrant."

### Unified Theme: Separation of Concerns

| Paper | Separation | NeoTrix Mapping |
|-------|-----------|-----------------|
| Flux Attention | Full vs sparse attention per layer | NT-IO: cheap vs expensive backends |
| LISA | Linear (long-range) vs sparse (precise) memory | NT-MEMORY: KB vs session context |
| DUET | Reasoning (heavy) vs generation (light) | NT-CORE vs NT-ACT |
| AttnPO | Essential vs redundant reasoning steps | SEAL: core phases vs noise |
| Reflection Steering | Reflection vs reasoning processes | ConsciousnessTree vs E8 |

### Unified Theme: Training-Free or Lightweight Adaptation

- **Flux Attention**: 12 hours on 8 GPUs
- **LISA**: Plug-and-play, no pretraining
- **Reflection Steering**: Activation-space intervention, no retraining

**NeoTrix implication**: NeoTrix's modules should be designed for lightweight adaptation — drop-in improvements without full retraining. This aligns with C0-C6 constellation maturity: modules should be improvable at C4 (integrated) without dropping to C0 (recompile).

---

## Recommendation for NeoTrix

**Immediate absorption:**
1. **LISA's two-branch memory** — NT-MEMORY dual-path (KB long-range + session sparse) with gating fusion
2. **AttnPO's attention supervision** — SEAL self-test could use attention patterns to identify essential vs redundant cycle phases
3. **Reflection Steering's dual-process** — Validates and strengthens NT-CORE/ConsciousnessTree separation

**Research direction:**
4. **Flux Attention's Layer Router** — GWT evolution: from uniform broadcast to layer-level routing based on salience
5. **DUET's reasoning-action separation** — Formalize NT-CORE (reasoning signal) + NT-ACT (lightweight execution) as architectural pattern
