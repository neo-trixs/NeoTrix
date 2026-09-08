# Iteration Batch 646 — Time Series: Forecasting, Anomaly Detection, Temporal Patterns

**Date:** 2026-09-06
**Prior Batch:** 645 (YOLO26 anchor paradox, SAM3 linear scaling, ViT-5 SwiGLU over-gating, LLM→ViT transfer unsafe, SAM3 zero-shot niche failure)

---

## A. Time Series Forecasting — Foundation Model Landscape 2026

### A1. Chronos 2 Group Attention: Encoder-Only Beats Encoder-Decoder
**Source:** Ansari et al., arXiv 2510.15821; tsfm.ai comparison (2026-08)
**Finding:** Chronos 2 dropped the T5 encoder-decoder architecture in favor of encoder-only with **group attention** — alternating time-attention (within a series) and group-attention (across series at each patch index). This achieves zero-shot multivariate + covariate forecasting without fine-tuning. On electricity benchmarks, Chronos 2 achieves 5.2pp skill-score improvement over next-best baselines, 50-1000× faster inference.

**Defect Found — Group Attention Homogenization Bias:** Group attention shares information across all series in a batch uniformly. When series have heterogeneous dynamics (e.g., stable load vs. volatile solar), the shared representation averages away high-frequency signals from volatile series. The batch composition becomes a hyperparameter that silently degrades minority-dynamic series. NeoTrix implication: any attention routing in GWT that aggregates across heterogeneous specialists risks the same homogenization — attention heads that share across all modules will average away niche expert signals.

### A2. Moirai 2.0 Decoder-Only + Multi-Token Prediction
**Source:** Liu et al., arXiv 2511.11698 (2025-11); GIFT-Eval leaderboard
**Finding:** Moirai 2.0 switches from masked encoder (BERT-style) to decoder-only autoregressive with multi-token prediction and quantile loss. Ablation shows: decoder-only architecture switch + recursive multi-quantile decoding accounts for **most of the gain** (not the new data). At 36M parameters on 295B observations, ranks 5th on GIFT-Eval with favorable speed/accuracy tradeoff.

**Defect Found — Recursive Quantile Error Accumulation:** Multi-token prediction feeds quantile outputs back as inputs for next step. Each step propagates full uncertainty, but the quantile levels at step T+K are derived from quantile-of-quantiles, not from the true conditional distribution. This compound quantile estimation introduces **systematic bias at tails** (P95/P05) that grows with horizon length. The paper reports P50 accuracy but does not isolate tail degradation. NeoTrix implication: any cascade architecture (like SEAL pipeline stages feeding forward) compounds estimation errors at tail events — need explicit tail-correction mechanisms.

### A3. Break-Even Analysis: When Foundation Models Pay Off
**Source:** Simon, alphaXiv 2607.04919 (2026-07)
**Finding:** First systematic break-even analysis across 30 datasets at 6 training-set sizes. Key rule: if n_train < 700 AND seasonality is non-negligible, use FM zero-shot — skip fine-tuning entirely (resolves 10/30 decisions). LoRA fine-tuning **actively degrades** performance on short series. On 6/30 datasets, classical methods (XGBoost) beat zero-shot FMs with as little as 2% data.

**Defect Found — Seasonality Threshold Unquantified:** The "non-negligible seasonality" criterion is never operationalized with a numeric threshold. Without a concrete test (e.g., ACF lag-1 significance, spectral peak ratio), practitioners cannot reliably apply the rule. This is a gap in the decision framework. NeoTrix implication: SelfTest tier criteria (T1/T2/T3) similarly lack numeric thresholds — need concrete signal-strength gates for module maturity decisions.

### A4. TTM Sub-1M Edge Models: MLP-Mixer Wins at Tiny Scale
**Source:** Ekambaram et al., NeurIPS 2024; howtostoreelectricity.com deep dive (2026-02)
**Finding:** IBM Granite TTM uses MLP-Mixer (no attention) at <1M parameters. On edge hardware (Cortex-A8, 512MB RAM), TTM runs at ~80ms per forecast. Competitive with 100× larger attention models at short horizons. Channel-mixing during fine-tuning (not pre-training) captures cross-variable correlations.

**Defect Found — Channel-Mixing Latency in Pre-Training:** TTM pre-trains channel-independent, then enables channel-mixing only during fine-tuning. This means the model never learns cross-channel dynamics during the large-scale pre-training phase. For domains where cross-channel interactions are the primary signal (e.g., power grid bus voltage relationships), TTM's pre-training representation is fundamentally impoverished. The "channel-mixing on fine-tune" strategy is a band-aid that cannot recover interactions absent from pre-training. NeoTrix implication: domain module pre-training (like NT-CORE's E8 patterns) must include cross-domain interactions during pre-training, not defer to fine-tuning.

### A5. Hierarchical Temporal Fusion (HTF): Coherence Loss for Hierarchical Forecasting
**Source:** Pakhle, arXiv 2606.28553 (2026-06)
**Finding:** Extends TFT with structured hierarchical embeddings + coherence-aware loss that penalizes child-parent sum mismatches during training (not post-processing reconciliation). Outperforms MinT and Bottom-Up on M5 Walmart and energy datasets.

**Defect Found — Hierarchical Embedding Static Assumption:** The hierarchical embeddings are static (product→category→region hierarchy doesn't change). But in real systems, hierarchies evolve (new products, mergers, seasonal category reassignment). The model has no mechanism to handle hierarchy topology changes without full retraining. NeoTrix implication: NeoTrix's faction hierarchy (7 domains) is similarly static — need a mechanism for domain reassignment without full retraining.

---

## B. Anomaly Detection — 2026 State of the Art

### B1. MambaADv2: Mamba-3 SSM for Visual Anomaly Detection
**Source:** arXiv 2606.23126 (2026-06)
**Finding:** Evolves from Mamba-1 to Mamba-3 SSM for anomaly detection. Duality-enhanced State Space (DSS) module combines linear recurrence + parallel matrix computation. Semantics-adaptive progressive scanning (SAPS) reduces scanning directions in deeper layers (shallow: many directions for spatial diversity; deep: few directions since semantic features are direction-invariant). SOTA on 6 anomaly detection datasets.

**Defect Found — Progressive Scanning Over-Pruning:** SAPS monotonically reduces scanning directions from shallow to deep. But certain anomaly types (e.g., subtle texture degradation at deep semantic levels) may require multi-directional scanning even at high abstraction levels. The monotonic reduction is a heuristic that assumes anomaly complexity always correlates with spatial resolution — this breaks for semantic-level anomalies. NeoTrix implication: any hierarchical attention reduction (like GWT's saliency gating) that monotonically prunes at deeper layers will miss subtle but high-level anomalies.

### B2. LSD: Latent SDE for Sparse/Irregular Multivariate Anomaly Detection
**Source:** arXiv 2606.18898 (2026-06)
**Finding:** First latent SDE approach for MTSAD. Models time series as trajectories on homogeneous spaces (Euclidean ℝⁿ or hyperspherical 𝕊ⁿ). Hyperspherical latent space naturally captures cyclic dynamics — on QAPPD robotic pick-and-place data, 𝕊ⁿ outperforms ℝⁿ consistently. Robust under bursty subsampling (10% data) where all baselines degrade substantially.

**Defect Found — Weak Decoder Masking True Anomalies:** The paper deliberately uses a "weak decoder" to prevent over-generalization to anomalous inputs. But a weak decoder also fails to reconstruct complex normal patterns that happen to look unusual (e.g., first occurrence of a seasonal pattern). This creates a blind spot: the model cannot distinguish "complex normal" from "simple anomaly" because the decoder lacks capacity for the former. NeoTrix implication: any reconstruction-based SelfTest that uses deliberately weak decoders will have systematic false negatives on complex-but-normal system states.

### B3. Causally Guided Transformer (CGT) for Root-Cause Analysis
**Source:** arXiv 2604.17998 (2026-04)
**Finding:** Integrates PCMCI-discovered time-lagged causal graph as hard parent mask in Transformer forecasting blocks. Shadow auxiliary path with stop-gradient + safety-gated blending captures residual correlations without breaking causal representation. F1: 96.19% ASD, 95.32% SMD. Root-cause attribution via counterfactual clamping outperforms score-based ranking.

**Defect Found — Causal Graph Static Assumption + Latent Confounder Blindness:** The PCMCI causal graph is discovered once from training data and fixed. In non-stationary systems, causal relationships shift (e.g., sensor degradation changes which sensors cause which). The paper acknowledges this but offers no adaptation mechanism. More critically, PCMCI cannot discover latent confounders — if two sensors are both caused by an unmeasured third variable, the model treats their correlation as direct causation, leading to misattribution during root-cause analysis. NeoTrix implication: any causal audit dimension (D13-D16) that uses static causal graphs will miss evolving causal relationships and latent confounders.

### B4. TFIDD: Temporal Feature Importance Drift Detection
**Source:** Scientific Reports, 2026-07-16 (Nature)
**Finding:** Monitors temporal feature-importance dynamics (not errors or marginal distributions) for drift detection. Uses cosine divergence between current and history-weighted baseline feature representations with adaptive quantile thresholding. SEA stream: 0.8426 post-drift accuracy, 0.0278 false alarm rate. Gas Sensor: 0.0023 false alarm rate.

**Defect Found — Feature Importance Drift vs. Concept Drift Confusion:** TFIDD detects when feature importance changes, but not all feature-importance changes indicate concept drift. A feature may become important due to increased noise (not distribution shift). The cosine divergence metric cannot distinguish "feature became important because the world changed" from "feature became important because noise increased." This creates false positives in noisy environments. NeoTrix implication: GWT attention modulation that tracks importance shifts needs a noise-vs-signal disambiguation mechanism.

### B5. SDA2D: State-Derivative-Aware NCDE for Anomaly Diagnosis
**Source:** AAAI 2026
**Finding:** Models system dynamics via derivative of NCDE-derived state vector. Detects anomalies by measuring reconstruction deviation + system evolution jointly. Visualization enables anomaly diagnosis by locating source sensors.

**Defect Found — Derivative Estimation Noise Amplification:** Computing state derivatives amplifies high-frequency noise in the observation signal. The NCDE formulation smooths some of this, but in systems with rapid legitimate state changes (e.g., power grid frequency regulation), derivative-based detection will trigger false alarms during normal transient events. The paper does not address how to distinguish normal transients from abnormal dynamics. NeoTrix implication: any consciousness metric that computes rate-of-change (like evolution velocity in ConsciousnessTree) faces the same noise amplification in derivative estimation.

### B6. Hyperbolic Adaptive Spatial-Aware MTSAD
**Source:** Nature Communications, 2026-07-27
**Finding:** Embeds spatio-temporal features in hyperbolic space (Poincaré ball). Adaptive graph structure discovery removes rigid topology. Multi-level attention + small-sample dynamic threshold. Reports 10-30% improvement across benchmarks.

**Defect Found — Hyperbolic Embedding Instability at Scale:** Hyperbolic embeddings are sensitive to initialization and training stability, especially in high dimensions. The Poincaré ball representation has an exponentially growing volume near the boundary, meaning small perturbations near the boundary cause large representation shifts. In systems with many variables (high-dimensional MTS), boundary-proximate embeddings become numerically unstable. The paper does not report embedding distribution statistics or boundary proximity analysis. NeoTrix implication: any VSA HyperCube embedding that approaches boundary conditions (high-dimensional orthogonality limits) may face analogous instability.

---

## C. Temporal Patterns — Motif Discovery 2026

### C1. Panache: One-Pass Streaming PMP Motif Discovery
**Source:** arXiv 2607.17481 (2026-07)
**Finding:** First one-pass streaming algorithm for z-normalized pan-matrix-profile motif discovery. Replaces L quadratic self-joins with near-linear-time spectral state maintenance via sliding-DFT recurrences. Occupancy-controlled hash directory + Parseval's theorem lower bound rejects most pairs before exact computation. On Wafer (5M samples, 51 lengths): 6.0 minutes vs. 7.95 hours (fastest CPU) and 38.3 minutes (SCAMP on H100 GPU).

**Defect Found — Spectral State Drift Under Non-Stationarity:** Panache assumes the non-DC Fourier spectrum of z-normalized subsequences is stable enough for hash-based collision. In non-stationary time series (where the generating process changes), the spectral signature of "similar" subsequences shifts, causing hash collisions to miss true motifs and create false matches. The occupancy-controlled directory mitigates but does not eliminate this. NeoTrix implication: any motif-based pattern library (like SEAL's distillation pattern matching) that uses spectral signatures will degrade under concept drift.

### C2. DTMiner: Data-Centric Temporal Motif Mining
**Source:** PPoPP 2026 (ACM)
**Finding:** Load-Explore-Synchronize (LES) execution model regularizes data accesses across motif matching tasks. Tasks share graph traversal for same chunks with fine-grained synchronization. 1.14×-11.98× improvement over SOTA. Key insight: data accesses exhibit strong spatial similarity + temporal monotonicity.

**Defect Found — Cache-Line Contention Under High Concurrency:** LES loads temporal graph chunks into cache sequentially, then triggers all relevant tasks to explore. When many tasks target the same chunk (high motif density), cache-line contention causes evictions before all tasks complete exploration. The fine-grained synchronization mechanism adds overhead that scales with task-chunk overlap. The paper's benchmarks use moderate concurrency; production temporal graphs (social networks with millions of simultaneous interactions) would stress this. NeoTrix implication: any shared-knowledge-base access pattern (KB reads during SEAL phases) faces the same cache-line contention when multiple modules read the same KB chunk concurrently.

### C3. LLMTM: LLMs for Temporal Motif Analysis in Dynamic Graphs
**Source:** AAAI 2026
**Finding:** Benchmarks LLMs on 6 temporal motif tasks across 9 motif types. Tool-augmented LLM agent achieves high accuracy but at substantial cost. Structure-aware dispatcher routes queries between standard LLM (cheap) and agent (expensive) based on graph structural properties + LLM cognitive load.

**Defect Found — Cognitive Load Metric Unvalidated:** The "LLM cognitive load" used for dispatch decisions is a heuristic based on prompt length and motif complexity. There is no validation that this metric correlates with actual LLM failure modes. A simple motif with ambiguous natural language description may have low "cognitive load" but high failure rate, while a complex motif with precise description may have high load but low failure rate. The dispatcher will misroute in both cases. NeoTrix implication: any attention-budget allocation based on task complexity heuristics (rather than measured failure rates) will similarly misroute.

### C4. STEP: Stochastic Event Prediction via Temporal Motif Transitions
**Source:** arXiv 2603.05874 (2026-03)
**Finding:** Reformulates temporal link prediction as sequential forecasting via Poisson-driven motif transitions. Maintains open motif instances; at each step decides: start new motif (cold event) or extend existing (hot event). 21% average precision gain over SOTA when combined with TGNNs. Standalone generative mode achieves 0.99 precision in next-sequential forecasting.

**Defect Found — Motif Type Explosion:** STEP's feature vector is indexed by motif type, but the number of motif types grows combinatorially with motif size (ℓ_max). For ℓ_max=5, there are already hundreds of possible directed motif types. The feature vector becomes sparse, and the Bayesian scoring becomes dominated by prior assumptions for rare motif types. The paper uses ℓ_max with Δ constraints to limit this, but does not analyze how motif-type sparsity affects prediction quality. NeoTrix implication: any state-space model indexed by pattern types (like SEAL's phase-specific pattern libraries) faces combinatorial explosion as pattern complexity grows — need automatic pattern-type pruning.

### C5. SubTSMD: Subspace Motif Discovery with Temporal Variations
**Source:** Data Mining and Knowledge Discovery, Springer, 2026-07
**Finding:** Discovers motifs in subspaces of high-dimensional time series (not full-dimensional). Allows non-exact temporal alignment across attributes. First evaluation metric for subspace motif quality (assesses subspace coverage, not just temporal alignment). 6 synthetic + 14 real datasets, 5000 total time series.

**Defect Found — Subspace Explosion Without Domain Guidance:** The bottom-up incremental combination of per-attribute motifs into subspace motifs generates O(2^d) possible subspaces for d attributes. The evaluation metric helps assess quality but does not guide search. Without domain knowledge to constrain which attribute subsets are meaningful, the algorithm explores exponentially many subspaces. For high-dimensional sensor systems (100+ sensors), this is intractable. NeoTrix implication: any cross-domain pattern discovery (like Nexus-梭's knowledge weaving) that explores all attribute combinations faces the same exponential blowup without domain-guided pruning.

### C6. Stringology-Based Motif Discovery for EEG (ADHD)
**Source:** arXiv 2603.03476 (2026-03)
**Finding:** Adapts order-preserving matching (OPM) and Cartesian tree matching (CTM) from stringology to EEG motif discovery. ADHD group shows: more frequent but shorter OPM motifs (increased recurrence, reduced persistence), shallower CTM trees (reduced hierarchical complexity). First application of stringology to neurodevelopmental biomarkers.

**Defect Found — Support Threshold Sensitivity:** Uses minimum support = 0.9 (90% of strings must contain motif). This extreme threshold means only near-universal patterns survive — individual variability is completely discarded. For clinical applications where individual differences carry diagnostic value (e.g., ADHD subtypes), this threshold destroys the signal. The paper acknowledges lower thresholds increase false discovery but does not explore adaptive thresholds. NeoTrix implication: any cross-session pattern mining (like experience-tree's distillation) that uses a single global support threshold will miss rare-but-important session-specific patterns.

---

## D. Cross-Cutting Defects for NeoTrix

| # | Defect | Domain | Severity | NeoTrix Component Affected |
|---|--------|--------|----------|---------------------------|
| D1 | Group Attention Homogenization Bias | Forecasting | High | GWT attention routing across heterogeneous specialists |
| D2 | Recursive Quantile Tail Degradation | Forecasting | Medium | SEAL pipeline stage cascading |
| D3 | Seasonality Threshold Unquantified | Forecasting | Medium | SelfTest T1/T2/T3 signal gates |
| D4 | Channel-Mixing Not in Pre-Training | Forecasting | High | Cross-domain interaction during pre-training |
| D5 | Hierarchical Embedding Static Assumption | Forecasting | Low | Domain hierarchy topology |
| D6 | Progressive Scanning Over-Pruning | Anomaly Detection | High | GWT saliency gating at deeper layers |
| D7 | Weak Decoder Complex-Normal Blind Spot | Anomaly Detection | High | Reconstruction-based SelfTest |
| D8 | Static Causal Graph + Latent Confounders | Anomaly Detection | High | Causal audit dimensions D13-D16 |
| D9 | Feature-Importance vs. Concept Drift Confusion | Anomaly Detection | Medium | GWT attention modulation |
| D10 | Derivative Noise Amplification | Anomaly Detection | Medium | ConsciousnessTree evolution velocity |
| D11 | Hyperbolic Embedding Boundary Instability | Anomaly Detection | Medium | VSA HyperCube embeddings |
| D12 | Spectral State Drift Under Non-Stationarity | Pattern Mining | High | SEAL distillation pattern matching |
| D13 | Cache-Line Contention at Scale | Pattern Mining | Medium | KB concurrent read access |
| D14 | Cognitive Load Metric Unvalidated | Pattern Mining | Medium | Attention-budget allocation |
| D15 | Motif Type Combinatorial Explosion | Pattern Mining | High | Pattern-type state spaces |
| D16 | Subspace Explosion Without Domain Guidance | Pattern Mining | High | Cross-domain pattern discovery |
| D17 | Support Threshold Destroys Individual Signal | Pattern Mining | Medium | Cross-session experience mining |

---

## E. Sources Cited

1. Ansari et al. "Chronos-2: From Univariate to Universal Forecasting" arXiv 2510.15821
2. Liu et al. "Moirai 2.0: When Less Is More for Time Series Forecasting" arXiv 2511.11698
3. Simon "When Do Foundation Models Pay Off? Break-Even Analysis" alphaXiv 2607.04919
4. Ekambaram et al. "Tiny Time Mixers (TTMs)" NeurIPS 2024
5. Pakhle "Hierarchical Temporal Fusion" arXiv 2606.28553
6. MambaADv2 "Duality-enhanced State Space Model" arXiv 2606.23126
7. LSD "Latent SDE for Sparse/Irregular MTSAD" arXiv 2606.18898
8. CGT "Causally-Constrained Probabilistic Forecasting" arXiv 2604.17998
9. TFIDD "Adaptive Dynamics for Drift Detection" Scientific Reports 2026-07
10. SDA2D "State-Derivative-Aware NCDE" AAAI 2026
11. Hyperbolic MTSAD "Nature Communications" 2026-07-27
12. Panache "One-Pass PMP Motif Discovery" arXiv 2607.17481
13. DTMiner "Data-Centric Temporal Motif Mining" PPoPP 2026
14. LLMTM "LLMs for Temporal Motif Analysis" AAAI 2026
15. STEP "Stochastic Event Prediction via Motif Transitions" arXiv 2603.05874
16. SubTSMD "Subspace Motif Discovery" Springer DMKD 2026
17. Stringology EEG Motifs "ADHD Case Study" arXiv 2603.03476
18. howtostoreelectricity.com "TS Foundation Models 2026" (comprehensive comparison)
19. tsfm.ai "Best Time Series Foundation Models in 2026" (comparison table)
20. SCAN "Sequential Change-Point Detection" arXiv 2608.28110
21. BARBS "Recursive Multiple Change Point Detection" arXiv 2608.13352
22. Online Neural Networks for Change-Point Detection ML 2026
23. DyMETER "Dynamic Concept Adaptation for OAD" arXiv 2604.14726
24. TSA-Net "Two-Stage Temporal Attention" Sensors 2026
25. SAST "Scale-Adaptive Spatio-Temporal Modeling" Applied Intelligence 2026
