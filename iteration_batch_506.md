# Iteration Batch 506 — Research Loop

**Date**: 2026-09-06  
**Domains**: Information Theory · Channel Coding · Rate-Distortion  
**Method**: External research → Design doc gap analysis → Defect identification

---

## 1. Information Theory Advances (2026)

### 1.1 NMINE: Normalized Mutual Information Neural Estimation
- **Source**: [arXiv 2607.27710](https://arxiv.org/html/2607.27710v1) (Jul 2026)
- **Finding**: Fully neural NMI estimator combining MINE (Donsker–Varadhan variational KL) with neural marginal entropy estimators via uniform reference distributions. Eliminates nearest-neighbor curse of dimensionality. Normalizes MI to [0,1] for cross-dataset comparability.

### 1.2 SBI-BOED: Joint Inference + Experimental Design via Mutual Information
- **Source**: [PMLR v337, UAI 2026](https://proceedings.mlr.press/v337/zaballa26a.html) (Aug 2026)
- **Finding**: InfoNCE lower bound on EIG becomes a principled SBI training objective. Maximizing bound ≡ fitting surrogate likelihood by minimizing KL divergence + marginal-likelihood term. Single stochastic-gradient procedure jointly trains likelihood + optimizes designs without simulator differentiability.

### 1.3 INFO-SEDD: Information Estimation via Discrete Diffusion
- **Source**: [ICLR 2026](https://en.papernotes.org/ICLR2026/learning_theory/information_estimation_with_discrete_diffusion/) (May 2026)
- **Finding**: Dynkin's formula connects score function of discrete diffusion (CTMC) to KL/MI/entropy estimation. Consistent estimator with variance that does NOT explode with MI — fundamental advantage over variational lower bound estimators. Score function serves dual purpose (generative + estimation).

### 1.4 DPI Framework for f-Divergences → Tighter Generalization Bounds
- **Source**: [arXiv 2602.07999](https://arxiv.org/pdf/2602.07999) (2026)
- **Finding**: Data processing inequality for f-divergences yields change of measure inequalities. Unified framework covering KL, χ², Rényi, α-MI, maximal leakage. Bounds are tighter than existing high-probability results in almost all cases.

### 1.5 PAC-Bayes Z-Information: Behavioral Equivalence Decomposition
- **Source**: [arXiv 2608.11465](https://arxiv.org/html/2608.11465v1) (Aug 2026)
- **Finding**: Exact fiberwise decomposition of PAC-Bayes KL into behavior-selection term + realization-level term (Z-information). Separates uncertainty over predictive behavior from variation among behaviorally equivalent realizations. Minimum classical PAC-Bayes complexity is achieved by fiber-symmetrized posterior.

---

## 2. Channel Coding Advances (2026)

### 2.1 Advances in Modern Channel Coding (Special Issue)
- **Source**: [PMC 12840139](https://pmc.ncbi.nlm.nih.gov/articles/PMC12840139/) (Jan 2026)
- **Finding**: Cross-domain integration of channel coding with ML-assisted designs. Five contributions: BIBD-based LDPC construction with predictable girth/cycle analysis, bit-level construction for non-binary polar codes, restart mechanisms for SCLF polar decoding, probabilistic shaping, and adaptive coding under imperfect channel knowledge.

### 2.2 LDPC-Polar Hybrid for 6G
- **Source**: [EPJ Web of Conferences, 2026](https://www.epj-conferences.org/articles/epjconf/abs/2026/30/epjconf_iceodis2026_01003/epjconf_iceodis2026_01003.html) (Jun 2026)
- **Finding**: Polar codes superior for short blocks <1700 bits (URLLC). LDPC superior for longer blocks. Transition occurs ~1600 bits. Complementarity exploitable in adaptive hybrid architectures for 6G heterogeneous requirements.

### 2.3 Row-Boosted Ensemble Belief Propagation for Short LDPC
- **Source**: [arXiv 2608.09560](https://arxiv.org/abs/2608.09560) (Aug 2026)
- **Finding**: RBE-BP decoder creates diversity by boosting selected parity-check row messages across ensemble members. On 5G NR BG1 (144,96): FER from 1.6×10⁻² to 1.6×10⁻³ at Eb/N0=4.0dB. Error-free boosted rows act as reliable anchors. Transfers across block lengths without re-tuning.

### 2.4 LLM-Assisted LDPC Decoding via Syndrome-Verified Semantic Priors
- **Source**: [arXiv 2608.14280](https://arxiv.org/abs/2608.14280) (Aug 2026)
- **Finding**: LLM infers intended characters from semantic context; decoder verifies corrections against parity-check constraints before injection as soft updates to LLRs. 73% BER reduction at 2.0 dB vs 21% from doubling iterations. Semantic knowledge → physical-layer reliability.

### 2.5 RL-Based Universal Polar Sequence Design (6G Scale)
- **Source**: [arXiv 2601.20118](https://arxiv.org/pdf/2601.20118) (2026)
- **Finding**: PPO-based universal sequence design scalable to N=2048. Uses universal partial order (UPO) to constrain search space. Joint multi-configuration optimization enables knowledge transfer across (N,K) configurations. Up to 0.2 dB gain over beta-expansion at N=2048.

---

## 3. Rate-Distortion Advances (2026)

### 3.1 Unified Rate-Distortion Perspective on Visual Tokenization (VQ-PQ-SQ)
- **Source**: [arXiv 2609.02107](https://arxiv.org/html/2609.02107) (Sep 2026)
- **Finding**: Minimizing distortion (not maximizing codebook utilization) is primary objective for reconstruction fidelity. Distortion hierarchy: VQ ≤ PQ ≤ SQ under matched rates. STE-induced gradient discrepancy directly connected to distortion. VQ exploits low-dimensional source structure for better scaling.

### 3.2 Rate-Distortion-Perception (RDP) Theory Tutorial
- **Source**: [arXiv 2607.17232](https://arxiv.org/html/2607.17232v1) (Jul 2026)
- **Finding**: RDP theory adds perception as third axis to rate-distortion. Perception constraint quantified via distributional similarity (f-divergences, α-divergences, Wasserstein). Alternating minimization and Newton-based methods for computing RDPF. Perfect-realism regime analytically tractable.

### 3.3 R-D Limits of Learned Image Compression
- **Source**: [arXiv 2601.09254](https://arxiv.org/html/2601.09254v1) (Jan 2026)
- **Finding**: Decomposes R-D performance loss into: variance estimation, quantization strategy, context modeling. Optimal latent variance = second moment under Gaussian assumption. Gap between uniform quantization and Gaussian test channel from reverse water-filling quantified.

### 3.4 PrismQuant: R-D Optimal VQ for Gaussian-Mixture Sources
- **Source**: [arXiv 2605.15507](https://arxiv.org/html/2605.15507v1) (May 2026)
- **Finding**: Mixture structure costs only a component label. Single global reverse-waterfilling level shared across all components and eigenmodes (vs prior per-component water levels). Gap = H(C)/n, vanishes asymptotically. Practical codec: EM + MAP + component-matched KLT + ECSQ.

### 3.5 Universal RDP Representations
- **Source**: [IEEE TIT 2025, published 2026](https://doi.org/10.1109/tit.2025.3584291)
- **Finding**: Fixed encoder + varying decoder can achieve near-optimal RDP tradeoff. Gaussian sources: entire distortion-perception tradeoff achievable by single encoder asymptotically. Practical implications: reduce number of trained models in deep compression.

---

## 4. Defects Identified in NeoTrix Design

### DEFECT-506-01: KL Divergence Estimation Lacks Normalization
**Location**: `nt_core_hcube/aif/free_energy.rs:91-99`  
**Current**: `compute_kl_divergence` returns raw KL in nats. No normalization.  
**Gap vs Research**: NMINE (arXiv 2607.27710) shows normalized MI ∈ [0,1] is critical for cross-domain comparability. NeoTrix's KL values cannot be compared across different VSA dimensions, hypothesis spaces, or belief state sizes.  
**Impact**: VoI comparisons across heterogeneous experiment types (bayesian_experiment.rs) are meaningless when hypothesis counts differ. FreeEnergyCalculator's complexity term is scale-dependent.  
**Suggestion**: Add `compute_normalized_kl(p, q)` that normalizes by max-entropy `log(|support|)`. Expose `normalized_kl` alongside raw `kl` in FreeEnergyCalculator. VoI ranking in `select_next_experiment` should use normalized MI, not raw KL.

### DEFECT-506-02: VoI Estimator Uses Monte Carlo with No Variance Control
**Location**: `nt_core_hcube/bayesian_experiment.rs:143-198`  
**Current**: VoI computed via `samples_per_experiment` Monte Carlo samples of prior→posterior KL. Fixed 64 samples, no adaptive allocation.  
**Gap vs Research**: SBI-BOED (UAI 2026) shows InfoNCE-λ objective with stabilizers outperforms naive MI critic. INFO-SEDD (ICLR 2026) achieves consistent estimation without variance explosion.  
**Impact**: VoI estimates for high-dimensional hypothesis spaces may have high variance, leading to suboptimal experiment selection. Fixed sample count wastes compute on easy experiments and under-samples hard ones.  
**Suggestion**: (1) Implement adaptive sample allocation: early-stop when VoI estimate confidence interval narrows below threshold. (2) Consider InfoNCE-λ lower bound as variance-controlled alternative to raw MC estimation. (3) Add variance tracking to `VoIConfig` for monitoring.

### DEFECT-506-03: E8 Lattice Quantizer Has No Rate-Distortion Optimization
**Location**: `nt_core_e8/e8_lattice_quantizer.rs:81`  
**Current**: Soft quantization of hidden states into 240 E8 roots. No explicit R-D optimization.  
**Gap vs Research**: Rate-distortion theory (arXiv 2609.02107, 2607.17232) shows distortion minimization is the primary objective, not geometric coverage. PrismQuant (arXiv 2605.15507) shows single global waterfilling level outperforms per-component allocation.  
**Impact**: E8 lattice quantizer may over-allocate bits to low-information dimensions while under-allocating to high-information dimensions. No mechanism to trade off reconstruction fidelity against code rate.  
**Suggestion**: Add `rd_optimize(latent, budget_bits)` method that applies reverse waterfilling across E8 eigenmodes. Track distortion (MSE) vs rate as observable metric for ConsciousnessTree health monitoring.

### DEFECT-506-04: qFHRR VSA Lacks Perception-Aware Reconstruction
**Location**: `nt_core_hcube/qfhrr_vsa.rs`  
**Current**: 3-bit quantized phase angles with fixed 8 levels. Bundle via complex sum → argmax. No perception constraint.  
**Gap vs Research**: RDP theory (arXiv 2607.17232) adds perception as third axis. Universal RDP representations (IEEE TIT 2025) show fixed encoder + varying decoder can achieve near-optimal tradeoff. Current qFHRR has no decoder adaptation for perception tuning.  
**Impact**: VSA embeddings reconstructed from qFHRR may minimize MSE but not perceptual/semantic fidelity. When HyperCube is used for associative recall, reconstruction quality depends on phase accuracy but semantic coherence is not optimized.  
**Suggestion**: (1) Add configurable decoder-side perception constraint (KL/divergence between original and reconstructed embedding distributions). (2) Implement decoder-only adaptation across perception-distortion tradeoff points without re-encoding. (3) Track perception score alongside MSE in VSA benchmark tests.

### DEFECT-506-05: No Error Correction Layer for VSA Embedding Storage
**Location**: KB embedding storage (`nt_memory_embed.rs:390`)  
**Current**: PQ centroid quantization for ANN search. 4-bit quantization for embedding artifacts. No channel coding protection.  
**Gap vs Research**: LLM-assisted LDPC decoding (arXiv 2608.14280) shows semantic priors + syndrome verification can achieve 73% BER reduction. RBE-BP (arXiv 2608.09560) achieves order-of-magnitude FER improvement for short codes.  
**Impact**: VSA embeddings stored in KB are vulnerable to bit flips in SQLite pages, SSD bit rot, or transmission errors. No mechanism to detect or correct embedding corruption. Silent corruption leads to degraded recall without awareness.  
**Suggestion**: (1) Apply short-block LDPC or CRC-protected storage for high-value embeddings. (2) Add embedding integrity check to SelfTest: recompute VSA from source text, compare against stored embedding using normalized Hamming distance. (3) For qFHRR vectors, use 3-bit → 4-bit storage with single parity check per byte.

### DEFECT-506-06: BayesianExperimentDesign Assumes Known Hypothesis Class
**Location**: `nt_core_hcube/bayesian_experiment.rs:14`  
**Current**: M-open check uses `ADEQUACY_THRESHOLD = 0.75` with fixed threshold. Hypothesis space static per experiment batch.  
**Gap vs Research**: SBI-BOED shows joint training of likelihood + design eliminates need for differentiable simulator. M-open check should be dynamic — when posterior concentrates on < threshold, expand hypothesis space dynamically, not just flag for manual intervention.  
**Impact**: If true model lies outside hypothesis class (M-open scenario), VoI computation is wasted — all hypotheses are wrong, and the "best" wrong hypothesis wins. Fixed 0.75 threshold is not calibrated to hypothesis count or dimensionality.  
**Suggestion**: (1) Replace fixed threshold with adaptive: `threshold = 1/n_hypotheses * (1 + log(n_hypotheses))` (matches information-theoretic minimum). (2) When M-open detected, automatically generate new hypotheses via LLM proposal + prior injection. (3) Track M-open frequency as meta-evolution health metric.

### DEFECT-506-07: FreeEnergyCalculator Uses Uniform Prior as Default
**Location**: `nt_core_hcube/aif/free_energy.rs:14`  
**Current**: `compute_vfe` hardcodes uniform prior `1/N` when computing complexity. No mechanism to use informative priors.  
**Gap vs Research**: PAC-Bayes Z-information (arXiv 2608.11465) shows behavior-selection complexity is the minimum over all posteriors inducing same predictive behavior. Uniform prior maximizes complexity term, making VFE biased toward complex models.  
**Impact**: Variational Free Energy is systematically over-estimated, biasing active inference toward exploration over exploitation. EFE horizon integration compounds this bias.  
**Suggestion**: (1) Accept optional `prior: Option<&[f64]>` parameter. (2) Default to empirical prior from historical belief states (stored in KB). (3) Add prior selection as VoI experiment: compare uniform vs informed prior on downstream task performance.

### DEFECT-506-08: No Cross-Modal Information Bottleneck
**Location**: Cross-modal alignment (`tests/cross_modal.rs`, `nt_file_ability/`)  
**Current**: CrossModalAligner maps text → VSA vector deterministically. No compression or bottleneck. All information passes through.  
**Gap vs Research**: R-D theory shows optimal representation requires trading off rate (information) against distortion (task relevance). Universal RDP representations show fixed encoder can serve multiple downstream tasks via decoder adaptation.  
**Impact**: VSA vectors carry all input information including noise, irrelevant features, and task-irrelevant detail. This wastes storage, increases collision probability in HyperCube, and degrades recall precision.  
**Suggestion**: (1) Implement information bottleneck: `I(X; Z) - β·I(Z; Y)` where Z is VSA representation, Y is downstream task label. (2) Add task-adaptive decoder that extracts task-relevant information from fixed VSA representation. (3) Measure and report compression ratio (pre-embedding dim → VSA dim) as system health metric.

---

## 5. Summary

| # | Defect | Severity | Domain |
|---|--------|----------|--------|
| 506-01 | KL normalization missing — cross-domain comparability broken | HIGH | Information Theory |
| 506-02 | VoI Monte Carlo lacks variance control | MEDIUM | Information Theory |
| 506-03 | E8 quantizer has no R-D optimization | HIGH | Rate-Distortion |
| 506-04 | qFHRR VSA lacks perception-aware reconstruction | MEDIUM | Rate-Distortion |
| 506-05 | No error correction for VSA embedding storage | HIGH | Channel Coding |
| 506-06 | BayesianExperimentDesign fixed hypothesis class | MEDIUM | Information Theory |
| 506-07 | FreeEnergyCalculator uniform prior bias | MEDIUM | Information Theory |
| 506-08 | No cross-modal information bottleneck | LOW | Rate-Distortion |

**Total sources cited**: 15 papers (2025-2026)  
**Total defects found**: 8  
**High severity**: 3 (506-01, 506-03, 506-05)  
**Medium severity**: 4  
**Low severity**: 1
