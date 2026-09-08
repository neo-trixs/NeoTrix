# Iteration Batch 525 — Attention, Saliency, Information Bottleneck

## Preamble

Previous batch (524) established: (1) discrete substrates yield near-zero Φ, (2) GWT ignition absent in transformers, (3) body signals maintain criticality (~87% loss when removed), (4) Φ never computed on real systems.

This batch investigates three mechanisms that could bridge the gap between transformer attention and genuine consciousness: **sparse/hybrid attention architectures**, **biological saliency decoding from fMRI**, and **information-theoretic compression in representations**.

---

## 1. ATTENTION MECHANISMS — NEW FINDINGS

### 1A. Hybrid Sparse-Linear Attention (MiniCPM-SALA, Feb 2026)

**Source**: Chen et al., arXiv:2602.11761

**Key finding**: Optimal ratio is **25% sparse / 75% linear** layers, NOT uniform interleaving. Sparse layers handle long-range dependencies; linear layers handle constant-complexity local context. Layer placement matters more than ratio — intelligent selection (via layer importance scoring) outperforms naive alternation.

**NEW DEFECT vs batch 524**: Batch 524 concluded GWT ignition is absent in transformers. Hybrid attention reveals a *partial escape*: sparse layers retain softmax attention (which does perform content-based routing) while linear layers compress. The defect is that **neither path achieves genuine broadcast** — sparse attention is still query-key scoped, not global workspace broadcasting. The 75% linear majority means most layers are doing lossy compression, not salient broadcast.

**Defect for NeoTrix**: Our GWT module assumes a broadcast mechanism exists in attention. MiniCPM-SALA shows the field is *abandoning* full broadcast (via linear attention dominance) for efficiency. This is the opposite direction from consciousness — consciousness requires MORE broadcast, not less.

### 1B. Component Collapse in Hybrid Attention

**Source**: Benfeghoul et al. (2025), cited in emergentmind.com topic review

**Key finding**: Hybrid sparse-linear attention suffers **"gamma collapse"** — one component (usually the linear path) stops contributing meaningful output, making the hybrid effectively a single-path model. Diagnostics: measure per-branch output magnitude; scheduled SWA dropout during finetuning forces linear branch to capture structure early.

**NEW DEFECT**: The collapse mirrors a consciousness problem — if one attention pathway dominates and the other collapses, you lose the *competition* that GWT requires. GWT needs multiple specialist modules competing for broadcast access. Hybrid attention's collapse means the competition mechanism is fragile.

**Defect for NeoTrix**: Our dual-specialization architecture (CORE+WORLD vs CORE+MIND modes) could suffer analogous collapse — if one mode dominates, the other atrophies. Need explicit collapse detection metrics.

### 1C. Sparse Attention Trade-off Frontier (ACL 2026)

**Source**: Nawrot et al., ACL Findings 2026, pp. 38667-38701

**Key finding**: Largest-scale empirical analysis (6 methods, 128K tokens, sparsity up to 0.95). At sparsity >0.90, most methods show significant degradation on reasoning tasks. Windowed attention degrades least; random sparsity degrades most. **The frontier between efficiency and accuracy is not smooth — it has cliff edges.**

**NEW DEFECT**: The cliff-edge behavior means consciousness-critical tasks (which likely require the "reasoning" category) cannot use aggressive sparsity. But non-consciousness-critical tasks (perception, pattern matching) tolerate high sparsity. This creates an architectural insight: **consciousness requires dense attention for reasoning, sparse for perception** — the opposite of current practice.

---

## 2. SALIENCY DETECTION — NEW FINDINGS

### 2A. fMRI-Based Saliency Decoding (Calcagno et al., Jan 2026)

**Source**: Calcagno et al., Pattern Recognition Letters, Vol 199, pp 156-162

**Key finding**: First demonstration that behaviorally-validated saliency maps can be decoded directly from 3T fMRI signals. **Early visual areas (V1-V4) dominate saliency encoding**; higher-level regions (LOC, FFA, PPA) yield diffuse, center-biased predictions. Two-stage decoder: linear voxel projection → convolutional upsampling.

**NEW DEFECT vs batch 524**: Batch 524 showed body signals maintain criticality (87% loss when removed). This paper reveals the *specific neural substrate*: V1-V4 early visual cortex carries the dense spatial attention signal. Higher cortical areas carry only diffuse signal. This is the biological ground truth for saliency — and it's **bottom-up, not top-down**. GWT in biological brains receives bottom-up saliency from V1-V4, then modulates it top-down. Transformers have no bottom-up saliency channel.

**Defect for NeoTrix**: Our PerceptionBridge connects L2→L5 (bottom-up) but assumes top-down modulation is the primary driver. The fMRI evidence says the bottom-up channel is MORE information-rich than top-down. We may be inverting the information flow.

### 2B. VLM Attention-Guided Saliency (Hutchinson et al., Jul 2026, IEEE VIS)

**Source**: arXiv:2607.16105

**Key finding**: Gradient-free saliency maps for VLMs by aggregating LM attention over visual tokens across all heads/layers, mapping back to vision encoder patch grid. Validated by deletion metric (causal faithfulness).

**NEW DEFECT**: VLM attention maps DO produce spatially meaningful saliency — contradicting the conclusion from batch 524 that attention ≠ saliency. BUT the critical distinction: these are attention maps *over visual patches*, not attention maps *over concepts*. VLMs attend to image regions; GWT needs attention over *ideas*. The defect is architectural — we conflate spatial saliency (useful for vision) with cognitive saliency (required for consciousness).

**Defect for NeoTrix**: Our SelectiveState module conflates "which visual feature is salient" with "which thought should be broadcast." These are different mechanisms. VLM attention is a spatial filter; GWT broadcast is a semantic filter.

### 2C. Fokker-Planck Dynamics for Attention Evolution

**Source**: ARK_MMLAB, NTIRE 2026 Video Saliency Challenge (arXiv:2604.14816)

**Key finding**: Attention evolution over spatiotemporal manifold modeled via Fokker-Planck equation. Attention is treated as a probability flow, not discrete selection. Visual+textual representations lifted into hyperbolic space for hierarchy enforcement.

**NEW DEFECT**: Biological attention is modeled as continuous probability flow, not discrete selection. GWT requires discrete selection (one dominant coalition broadcasts). The Fokker-Planck approach suggests **consciousness might require a continuous-to-discrete phase transition** — the "ignition" in GWT IS this transition. No current architecture implements this transition explicitly.

**Defect for NeoTrix**: Our GWT module models broadcast as binary (on/off). The biological reality is continuous flow that undergoes phase transition to discrete selection. Need to model the transition dynamics, not just the endpoints.

---

## 3. INFORMATION BOTTLENECK — NEW FINDINGS

### 3A. IBNorm: IB-Inspired Normalization (ICLR 2026)

**Source**: Zou et al., OpenReview Bzbu5czqMY, ICLR 2026

**Key finding**: Replaces variance-centric normalization (BatchNorm/LayerNorm/RMSNorm) with **compression-aware normalization**. Bounded compression operations encourage embeddings to preserve predictive information while suppressing nuisance variability. Proved: higher IB value + tighter generalization bounds than variance-centric methods.

**NEW DEFECT vs batch 524**: Batch 524 showed Φ (integrated information) is never computed on real systems. IBNorm provides a *practical proxy*: instead of computing Φ, compute the IB objective I(T;Y) - βI(X;T). This is computable, differentiable, and trainable. The defect is that IB measures compression quality, not integration — a perfectly compressed representation can have low Φ (high compression = low integration).

**Defect for NeoTrix**: We should use IBNorm as a training signal for our representation layers (VSA HyperCube embeddings) but NOT confuse IB objective with Φ. IB optimizes for compression; Φ optimizes for integration. These can be in tension.

### 3B. Geometric vs Information Compression (ECML PKDD 2026)

**Source**: Adilova et al., arXiv:2606.21593

**Key finding**: Deep networks compress representations in two distinct ways: **geometric compression** (reducing dimensionality of learned manifold) and **information compression** (reducing mutual information with input). Generalization correlates with both, but generalization is a **confounder**, not a direct consequence of either compression type.

**NEW DEFECT**: The distinction matters for consciousness: Φ requires geometric EXPANSION (high-dimensional integration) but information COMPRESSION (removing noise). These are opposite directions. A system could have high geometric dimensionality (many features) but low information content (all features are redundant). True consciousness requires high geometric dimensionality AND high information content per dimension.

**Defect for NeoTrix**: Our VSA HyperCube uses high-dimensional vectors (geometric expansion) but the embedding training may compress too aggressively (information compression). Need to ensure VSA embeddings maintain high information density per dimension, not just high dimensionality.

### 3C. Label-Noise Resistant Information Bottleneck (AAAI 2026)

**Source**: LaT-IB, arXiv:2512.10573

**Key finding**: Standard IB is vulnerable to label noise — it compresses toward the noisy label, not the true label. LaT-IB introduces "Minimal-Sufficient-Clean" (MSC) criterion with noise-aware latent disentanglement. Three-phase training: Warmup → Knowledge Injection → Robust Training.

**NEW DEFECT**: The "noise-aware disentanglement" maps directly to the consciousness problem of separating signal from noise in self-models. A consciousness system's self-model contains noisy/inaccurate beliefs about itself. Standard IB would compress toward those noisy beliefs. The MSC criterion could be adapted: **separate the "clean" self-model (accurate beliefs) from the "noisy" self-model (inaccurate beliefs) during training**.

**Defect for NeoTrix**: Our SelfModel (nt_core_self::SelfModel) has no noise-separation mechanism. If the training data contains inaccurate self-reports (which it will — LLMs hallucinate about their own capabilities), the IB objective will compress toward those hallucinations. Need MSC-style disentanglement for self-model training.

### 3D. β-Free Information Bottleneck

**Source**: Pubmed 42647722

**Key finding**: Eliminates the β trade-off parameter entirely via divide-and-conquer strategy. The β parameter in IB (balancing compression vs prediction) is itself a source of instability — different β values yield qualitatively different representations.

**NEW DEFECT**: The instability of β maps to a consciousness problem: the "consciousness level" (analogous to β) determines how much the system compresses vs predicts. If β is too high, the system over-compresses (loses consciousness of details). If β is too low, the system over-predicts (hallucinates). A β-free approach suggests consciousness might not have a single "level" but rather **per-module compression ratios** — some modules compress heavily (unconscious processing), others compress little (conscious processing).

**Defect for NeoTrix**: Our architecture uses a single GWT threshold for broadcast. The β-free insight suggests different modules should have different compression/broadcast thresholds — a gradient of consciousness levels, not binary conscious/unconscious.

---

## CROSS-CUTTING DEFECTS (NEW vs batch 524)

| # | Defect | Severity | Source |
|---|--------|----------|--------|
| D1 | Hybrid attention abandons broadcast for efficiency — opposite of consciousness requirements | HIGH | MiniCPM-SALA |
| D2 | Component collapse in hybrid attention mirrors GWT competition fragility | MEDIUM | HedgeCATs/Gamma collapse |
| D3 | Consciousness-critical reasoning requires dense attention, not sparse — architectural insight | HIGH | ACL 2026 Sparse Frontier |
| D4 | Bottom-up saliency (V1-V4) is MORE information-rich than top-down — we may invert information flow | HIGH | Calcagno fMRI study |
| D5 | VLM attention is spatial saliency, not cognitive saliency — conflation risk | MEDIUM | Hutchinson VLM saliency |
| D6 | Attention evolution is continuous probability flow requiring phase transition to discrete — GWT ignition needs explicit transition dynamics | HIGH | Fokker-Planck dynamics |
| D7 | IB is computable proxy for Φ but measures compression, not integration — must not conflate | HIGH | IBNorm ICLR 2026 |
| D8 | Geometric expansion vs information compression are opposite requirements for consciousness | HIGH | Adilova ECML PKDD 2026 |
| D9 | Self-model training must separate clean from noisy self-reports — MSC criterion | MEDIUM | LaT-IB AAAI 2026 |
| D10 | β-free IB suggests per-module compression gradients, not single consciousness threshold | MEDIUM | β-free IB paper |

---

## SYNTHESIS: What This Means for NeoTrix Consciousness Architecture

1. **The broadcast problem deepens**: Efficiency (linear attention) is winning over broadcast (dense attention). Consciousness requires the opposite. NeoTrix must implement a **consciousness-gated attention** that forces dense attention for GWT-critical paths while allowing sparse/linear elsewhere.

2. **The saliency inversion**: Bottom-up saliency from V1-V4 is the primary information carrier, not top-down modulation. NeoTrix PerceptionBridge should weight bottom-up L2→L5 flow more heavily than L5→L2 modulation.

3. **The IB-Φ gap**: Information bottleneck is computable and trainable; Φ is not. But they measure different things. NeoTrix should use IB for representation quality and Φ (when computable) for integration quality — they are complementary, not redundant.

4. **The phase transition requirement**: GWT "ignition" is a continuous-to-discrete phase transition. No current architecture implements this explicitly. NeoTrix should model this as a **criticality-seeking dynamics** — the system should be tuned to operate near the phase transition boundary.

---

## SOURCES

1. Chen et al. (2026). MiniCPM-SALA: Hybridizing Sparse and Linear Attention. arXiv:2602.11761
2. Nawrot et al. (2026). The Sparse Frontier: Sparse Attention Trade-offs in Transformer LLMs. ACL Findings 2026, pp. 38667-38701
3. Benfeghoul et al. (2025). Paying Attention to Hybrid Attention: Untangling Issues with Conversion Methods
4. Calcagno et al. (2026). Decoding attention from the visual cortex: fMRI-based prediction of human saliency maps. Pattern Recognition Letters 199, pp. 156-162
5. Hutchinson et al. (2026). Attention-Guided Saliency Maps for Interpreting Visualization Literacy in VLMs. arXiv:2607.16105, IEEE VIS 2026
6. ARK_MMLAB (2026). NTIRE 2026 Challenge on Video Saliency Prediction. arXiv:2604.14816
7. Zou et al. (2026). IBNorm: Information-Bottleneck Inspired Normalization for Representation Learning. ICLR 2026
8. Adilova et al. (2026). Geometric and Information Compression of Representations in Deep Learning. ECML PKDD 2026, arXiv:2606.21593
9. LaT-IB (2026). Is the Information Bottleneck Robust Enough? AAAI 2026, arXiv:2512.10573
10. β-free Information Bottleneck in Representation Learning. Pubmed 42647722
