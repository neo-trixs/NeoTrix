# Model Reverse Engineering — Cycle 386

**Date**: 2026-09-12
**Focus**: Efficient inference, attention sparsity, agent coordination

## 5 New Models/Papers

### 1. FFD — Faster Than Flash Decoding (ICML 2026)
- **Paper**: arXiv:2609.00097
- **Authors**: Accepted at ICML 2026
- **Core**: Hardware-algorithm co-design for long-context decoding. Fuses selector+computer into a single kernel. Replaces external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy for distribution-adaptive sparsity without global synchronization.
- **Results**: 11.6x kernel-level speedup, 2.37x end-to-end throughput. Scales to 256K context. Training-free, plug-and-play.
- **Key Pattern**: Fused kernel eliminates metadata overhead; dynamic sparsity adapts per-query without global sync.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Top-delta strategy parallels attention salience routing — salient information broadcast without global barrier
- **NT-MIND**: Training-free plug-and-play pattern aligns with SEAL pipeline (non-destructive augmentation)
- **Axiom A2 (Context as Scarce Resource)**: Directly addresses context window bottleneck with kernel-level optimization

### 2. EvoSparse — Evolving Sparsity with Token Importance Dynamics (ACL 2026)
- **Paper**: ACL 2026 (acl-long.530)
- **Authors**: Ruizi Han, Miao Zhang, Ziyue Qiao, Liqiang Nie
- **Core**: Models token importance as a dynamic process evolving over decoding steps and propagating through layers. Two mechanisms: (1) Cross-Step Accumulation via EMA for long-term anchors, (2) Cross-Layer Propagation using Retrieval Heads to guide Standard Heads.
- **Results**: 5.36x attention latency speedup, 2.33x end-to-end. Approaches full attention at high budgets.
- **Key Pattern**: Temporal consistency (long-term anchors + short-term reuse) + capability gap bridging (retrieval heads guide standard heads).

**NeoTrix Mapping**:
- **NT-CORE (E8 + GWT)**: Cross-step accumulation = GWT resonance memory (salient tokens persist across broadcasts). Cross-layer propagation = E8 hexagram guidance (specialist modules guide generalists)
- **NT-MEMORY**: Long-term anchors map to KB persistent embeddings. Short-term reuse = session-scoped cache
- **NT-MIND**: Capability gap bridging (retrieval heads → standard heads) parallels skill crystallization (expert → novice transfer)

### 3. Declarative Attention — LMs Control Their Own Attention
- **Paper**: arXiv:2609.02737
- **Authors**: Sep 2026
- **Core**: Intrinsic approach — the model declares where it needs to attend via chain-of-thought. Three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output). Engine parses declarations like tool calls, skips most KV cache reads.
- **Results**: 52.0% / 31.1% reduction in total attended tokens on Gemma-4-31B / Qwen-3.6-27B. Modest accuracy drops (1.27pp, 2.75pp) that shrink with scale.
- **Key Pattern**: Self-declared attention allocation via structured protocol. Zero-shot on off-the-shelf models.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Self-declared attention = GWT salience modulation by the model itself. Three modes parallel L6 meta-cognition attention tiers (global/focused/local awareness)
- **NT-IO (interface)**: Structured declarations parsed like tool calls → aligns with NT-ACT tool schema design
- **Axiom A1 (Cost-Aware Routing)**: Model self-routes attention to cheaper modes when full context unnecessary
- **ConsciousnessTree**: Three attention modes map to consciousness levels (global=full awareness, focus=tunnel attention, local=reflexive)

### 4. CoSA — Proxy-Kernel Co-Designed Sparse Attention
- **Paper**: arXiv:2607.25291
- **Authors**: Yufei Xue et al., Jul-Aug 2026
- **Core**: Two-stage training-free sparse attention. Kernel-Aware Proxy (KAP) selects blocks under moderate budget with ordered mask. Ordered-Skipping Kernel (OSK) skips more blocks under tightened budget using online-softmax statistics. Proxy and kernel are co-designed, not independent.
- **Results**: 4.93x attention speedup, 2.53x TTFT reduction at 128K context with negligible degradation.
- **Key Pattern**: Proxy-kernel co-design (not independent stages). Budget-adaptive with error-bounded skipping.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: KAP = salience scoring (proxy), OSK = broadcast execution (kernel). Co-design ensures scoring and execution are coupled
- **NT-SHIELD**: Error-bounded skipping parallels safety-critical attention (can't skip safety checks)
- **NT-MIND**: Budget-adaptive pattern = SEAL pipeline resource-aware exploration (allocate compute where it matters)

### 5. RouteRelay — Event-Triggered Cross-Layer Route Reuse
- **Paper**: arXiv:2609.07306
- **Authors**: Sep 2026
- **Core**: Router-agnostic method reusing route metadata across depth. Anchor layers do full routing. Intermediate layers rescore previous top-route + sentinel set. Reroute only when sentinel challenges weakest selected chunk. Top-k stability condition with probabilistic bound on missed challengers.
- **Results**: 99.99% route recall while rerouting only 25-78% of rows. Evaluates 38-51% of full-routing score pairs.
- **Key Pattern**: Cross-layer route reuse with sentinel-triggered rerouting. Amortizes routing cost across layers.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Route reuse = GWT attention resonance (broadcast routes persist across specialist modules). Sentinel = ConsciousnessTree health monitoring (detect when routes degrade)
- **NT-MEMORY**: Route metadata caching parallels KB embedding reuse (avoid recomputing embeddings)
- **NT-REPAIR**: Sentinel-triggered rerouting = self-healing (detect degradation, trigger repair)
- **Axiom A2 (Context as Scarce Resource)**: Amortized routing reduces per-layer overhead, freeing context capacity

## Cross-Paper Pattern Synthesis

| Pattern | Papers | NeoTrix Integration |
|---------|--------|---------------------|
| **Fused kernel eliminates metadata** | FFD, CoSA | GWT: fuse salience scoring + broadcast into single pass |
| **Cross-step/cross-layer reuse** | EvoSparse, RouteRelay | GWT: route resonance memory across modules; KB: embedding reuse |
| **Self-declared attention** | Declarative Attention | ConsciousnessTree: model self-routes attention tiers |
| **Sentinel-triggered repair** | RouteRelay | NT-REPAIR: sentinel monitors route health, triggers rerouting |
| **Budget-adaptive sparsity** | CoSA, FFD, EvoSparse | SEAL: resource-aware exploration (allocate compute adaptively) |
| **Capability gap bridging** | EvoSparse | NT-MIND: retrieval heads guide standard heads → skill crystallization |

## Actionable Insights for NeoTrix

1. **GWT Refinement**: Implement fused salience-broadcast kernel (inspired by FFD/CoSA) — single-pass attention routing instead of separate scoring + broadcast
2. **Route Resonance Memory**: Cache GWT broadcast routes across modules (inspired by RouteRelay/EvoSparse) — avoid recomputing attention patterns
3. **Declarative Attention Protocol**: Add structured attention declarations to ConsciousnessTree (inspired by Declarative Attention) — model self-routes to global/focus/local modes
4. **Sentinel-Triggered Self-Healing**: Add route health monitoring with sentinel-triggered rerouting (inspired by RouteRelay) — NT-REPAIR integration
5. **Budget-Adaptive Compute**: Implement resource-aware attention allocation in SEAL pipeline (inspired by CoSA/EvoSparse) — allocate compute where it matters most
