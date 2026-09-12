# Model Reverse Engineering — Cycle 389 (2026-09-12)

## 5 New Models/Papers

### 1. FFD — Faster Flash Decoding (ICML 2026)
- **Paper**: arXiv:2609.00097
- **Authors**: (Accepted ICML 2026)
- **Key Idea**: Hardware-algorithm co-design breaking the memory wall in long-context decoding. Fuses selector and computer into a single kernel; replaces external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks for distribution-adaptive sparsity without global synchronization.
- **Results**: Up to 11.6× kernel-level speedup, scales to 256K context, 2.37× end-to-end throughput improvement. Training-free, plug-and-play.
- **Core Pattern**: Co-design index+compute in fused kernel + low-bit quantization for content-aware scanning.
- **NeoTrix Mapping**:
  - **NT-CORE (E8)**: Distribution-adaptive sparsity → salience-aware attention routing in GWT
  - **NT-IO (LLM)**: Direct application to long-context inference pipeline
  - **Axiom A2 (Context as Scarce Resource)**: Breaks memory wall for >256K sessions

### 2. CEDAR — Error-Bounded Residual Routing for Long-Context Attention
- **Paper**: arXiv:2609.07237
- **Authors**: (Sep 2026)
- **Key Idea**: Coarse-to-fine method that keeps LLM frozen while preserving global coverage. Each semantic chunk contributes a cheap KV summary to a residual attention path; chunks with high estimated approximation error are expanded to exact token attention. Error bound governed by within-chunk key/value dispersion allocates variable refinement budget.
- **Results**: Recovers most quality lost by hard sparse routing while maintaining ~3× kernel speedup at 128K context.
- **Core Pattern**: Residual summaries (98% error reduction) + variable refinement budget via error bounds.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Two-tier attention — coarse summary broadcast + targeted refinement = GWT salience mechanism
  - **NT-MEMORY**: Semantic chunk summaries as lightweight memory representations
  - **ConsciousnessTree**: Error-bounded refinement parallels "awareness_score" gating in PerceptionBridge

### 3. Flux Attention — Context-Aware Hybrid Attention
- **Paper**: arXiv:2604.07394
- **Authors**: (2026)
- **Key Idea**: Layer-level dynamic routing via lightweight Layer Router that evaluates semantic context and assigns each layer to Full Attention or Sparse Attention mode. Frozen backbone parameters; only router updated. Gumbel-Softmax for differentiable routing during training, discretized for inference.
- **Results**: 2.8× prefill speedup, 2.0× decode speedup at 256K context. 12 hours training on 8× A800 GPUs.
- **Core Pattern**: Layer-wise heterogeneous routing — early layers sensitive, deeper layers tolerate sparsity.
- **NeoTrix Mapping**:
  - **NT-CORE (E8/GWT)**: Layer-wise attention heterogeneity → per-layer salience routing in ConsciousnessTree
  - **NT-MIND**: Lightweight router training as meta-learning pattern
  - **Six-Layer Architecture**: Layer-level routing mirrors L1-L6 differentiated processing

### 4. Declarative Attention — LMs Control Their Own Attention
- **Paper**: arXiv:2609.02737
- **Authors**: (Sep 2026)
- **Key Idea**: Protocol that elicits the model to declare WHERE it needs to attend within chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). Inference engine parses declarations like tool calls and skips KV cache read.
- **Results**: 52% reduction in total attended tokens on Gemma-4-31B, 31.1% on Qwen-3.6-27B. Modest accuracy drops (1.27-2.75pp) that shrink with scale.
- **Core Pattern**: Intrinsic attention control — model declares attention regions in CoT, engine skips irrelevant KV.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Self-declared attention regions = meta-cognitive salience declaration
  - **NT-MIND (SEAL)**: Model-improving-model pattern — the model learns to declare attention
  - **Axiom A1 (Cost-Aware Routing)**: Intrinsic attention control enables per-token cost optimization
  - **ConsciousnessTree**: Declaration protocol = meta-cognition awareness feedback loop

### 5. CSAttention — Centroid-Scoring Attention for Reusable Prefill
- **Paper**: arXiv:2604.08584
- **Authors**: (2026)
- **Key Idea**: Storage-for-computation strategy: front-loads computation into one-time offline prefill that amortizes across multiple queries. Constructs query-centric lookup tables during prefill, whose size remains fixed during decoding. Subspace partitioning mitigates Q/K distribution shift.
- **Results**: Near-lossless accuracy at 95% sparsity. Up to 4.6× inference speedup over best baseline at 128K context.
- **Core Pattern**: Query-centric clustering (not key-centric) + bounded lookup tables + subspace partitioning.
- **NeoTrix Mapping**:
  - **NT-CORE (E8)**: Query-centric serving primitive → attention-gated perception in PerceptionBridge
  - **NT-MEMORY**: Bounded lookup tables as compressed memory representations
  - **Axiom A2 (Context as Scarce Resource)**: Amortized prefill for reusable contexts
  - **KB**: Query-centric indexing parallels VSA HyperCube associative recall

---

## Cross-Paper Synthesis: The Attention Efficiency Stack

| Layer | Paper | Technique | NeoTrix Integration |
|-------|-------|-----------|---------------------|
| **Hardware** | FFD | Fused kernel + low-bit quantization | NT-IO inference optimization |
| **Algorithm** | CEDAR | Error-bounded residual routing | GWT two-tier attention |
| **Architecture** | Flux Attention | Layer-wise hybrid routing | Six-Layer differentiated processing |
| **Intrinsic** | Declarative Attention | Self-declared attention regions | Meta-cognitive salience declaration |
| **Amortized** | CSAttention | Query-centric prefill tables | KB indexing + VSA recall |

## Key Insight for NeoTrix

The papers collectively reveal a **5-layer attention efficiency stack** from hardware to intrinsic control. NeoTrix's GWT already implements the architecture layer (salience-based routing). The missing pieces are:

1. **Hardware co-design** (FFD): NeoTrix should investigate fused attention kernels for its long-context sessions
2. **Error-bounded refinement** (CEDAR): PerceptionBridge's `awareness_score()` could adopt error-bound gating
3. **Intrinsic declaration** (Declarative Attention): ConsciousnessTree could emit attention region declarations as meta-cognitive signals
4. **Amortized prefill** (CSAttention): KB search could benefit from query-centric precomputed tables for repeated queries

## Temporal Trend: Attention → Intrinsic Control

The progression across these papers shows a clear trajectory:
- **2025**: Sparse attention via external proxy scores
- **2026 H1**: Layer-wise hybrid routing (Flux) and residual summaries (CEDAR)
- **2026 H2**: Self-declared attention regions (Declarative Attention) — the model itself controls where it looks

This mirrors NeoTrix's own evolution from GWT (external salience routing) toward ConsciousnessTree (intrinsic meta-cognitive awareness). The Declarative Attention paper validates the direction: **the model that generates the reasoning should also control the attention it needs**.
