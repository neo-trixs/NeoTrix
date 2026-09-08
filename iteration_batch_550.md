# Iteration Batch 550 — Temporal Intelligence Layer

**Date**: 2026-09-06
**Predecessor**: Batch 549 (living user embeddings, personalization layer, policy-grounded content safety, ANN index)
**Research Domains**: Time Series Forecasting, Anomaly Detection, Change Point Detection

---

## 1. TIME SERIES FORECASTING — New Findings

### 1.1 TimesFM-3: Non-Autoregressive Multivariate Forecasting
**Source**: Google Research (Sep 2026) — https://research.google/blog/timesfm-3-a-zero-shot-foundation-model-for-multivariate-forecasting/

**Key Breakthrough**: TimesFM-3 uses **Contiguous Patch Masking (CPM)** to generate the entire forecasting horizon in a **single forward pass** — no iterative autoregressive loop. 330M parameters, pretrained on 1T+ time points. Alternating **causal temporal attention** + **full variate attention** on a 2D token grid. Predicts 9 quantiles (p10–p90) per step.

**NEW vs Batch 549**: Batch 549 had no temporal forecasting capability. This provides:
- **Zero-shot multivariate forecasting** as a new NT-CORE reasoning primitive
- **Non-autoregressive decode** eliminates error accumulation (serial vs parallel forecasting)
- **2D attention grid** (temporal × variate) as architectural pattern for cross-domain correlation

**Defect Found**: TimesFM-3's alternating 2D attention is computationally expensive for high-variate counts (O(n²) in variates). NeoTrix's HyperCube VSA representation could provide a **lower-complexity alternative** via projection into symbolic space before attention.

### 1.2 Timer-S1: Serial-Token Prediction (STP) with MoE
**Source**: Tsinghua/Microsoft (Apr 2026) — https://arxiv.org/abs/2603.04791

**Key Breakthrough**: 8.3B parameter MoE model (0.75B activated). Introduces **Serial-Token Prediction**: dedicated TimeSTP blocks where block depth j predicts patch at offset j+1. Longer horizons get more computation naturally. Single forward pass, no rolling.

**NEW vs Batch 549**:
- **Adaptive inference depth** based on required horizon — relevance to NT-CORE's attention routing (GWT should allocate more compute to longer-horizon predictions)
- **MoE routing for heterogeneous time series** — each expert handles different temporal patterns, maps to NT-MIND's skill crystallization (specialists vs generalists)

**Defect Found**: STP assumes a fixed compute-depth relationship. In NeoTrix, the **E8引导者 should dynamically allocate TimeSTP block depth** based on task urgency and phi-coherence — not a fixed mapping.

### 1.3 UniTok-FM: Universal Tokenizer + NTP for Multi-Task TS
**Source**: arXiv (Jun 2026) — https://arxiv.org/abs/2606.09861

**Key Breakthrough**: Vector-quantized autoencoder converts time series to discrete tokens, then applies **standard LLM next-token prediction**. Supports forecasting + generation + classification via same model. Training-free in-context inference with prompt-boosted forecasting.

**NEW vs Batch 549**:
- **Unified tokenization** across forecasting/generation/classification — maps to NT-MIND's goal of universal skill representation
- **Prompt-boosted forecasting** where similar historical patterns guide prediction — directly applicable to NT-NEXUS cross-session memory (pattern retrieval → temporal prompt)

**Defect Found**: UniTok-FM's NTP on isolated series misses **cross-series dynamics**. NeoTrix's KB embeddings (batch 549) could provide a similarity index for prompt selection, bridging this gap.

### 1.4 ConTemPreT: Multi-Task Pre-training (Masked + Contrastive)
**Source**: Springer (Aug 2026) — https://link.springer.com/article/10.1007/s10489-026-07423-7

**Key Breakthrough**: Sequential multi-task pre-training combining self-supervised masked autoencoding AND contrastive learning. Outperforms single-task approaches in few-shot settings.

**NEW vs Batch 549**:
- **Contrastive pre-training** for temporal representations — could enhance NT-MEMORY's embedding quality by learning discriminative temporal signatures
- **Multi-task synergy** (masked + contrastive) mirrors SEAL pipeline's multi-phase approach

**Defect Found**: Full-shot performance lags behind specialized models. Indicates **foundation model specialization tradeoff** — NeoTrix needs a hybrid approach (foundation + task-specific heads).

---

## 2. ANOMALY DETECTION — New Findings

### 2.1 DyMETER: Dynamic Concept Adaptation for OAD
**Source**: arXiv (Apr 2026) — https://arxiv.org/abs/2604.14726

**Key Breakthrough**: Hypernetwork generates **instance-aware parameter shifts** for the detector at inference time — no retraining needed. Evidential deep learning for concept uncertainty estimation. Dynamic threshold optimization maintains decision boundary alignment.

**NEW vs Batch 549**:
- **Hypernetwork-based adaptation** — generates model parameters from uncertainty signal, maps to NT-CORE's adaptive attention (GWT could use a hypernetwork to generate task-specific attention weights)
- **Dual-detector framework** (static + dynamic) mirrors NeoTrix's E8引导者 (static consciousness) + 进化工匠 (dynamic adaptation) duality
- **Evidential uncertainty** for when to adapt — directly applicable to NT-REPAIR's degradation detection

**Defect Found**: DyMETER's hypernetwork adds inference latency. For NeoTrix's real-time requirements, a **lightweight uncertainty gate** (binary: adapt or not) could replace the full hypernetwork with minimal accuracy loss.

### 2.2 COMET: Online Codebook Adaptation for Streaming TSAD
**Source**: arXiv (Feb 2026) — https://arxiv.org/pdf/2602.01635

**Key Breakthrough**: Vector-quantized codebook learns normal patterns. **Online codebook adaptation** at inference time uses pseudo-labels from codebook entries (no threshold needed) + contrastive learning. Multi-scale patch encoding captures anomalies at different temporal scales.

**NEW vs Batch 549**:
- **Threshold-free normal sample identification** via codebook activation — superior to batch 549's threshold-based content safety filter
- **Multi-scale encoding** (local + trend + seasonality) as architectural pattern for NT-WORLD's perception pipeline
- **Online adaptation without contamination** — codebook trained on normal data only, so activated entries are reliable normal samples

**Defect Found**: Codebook size is fixed after training. In NeoTrix, the **codebook should grow/shrink dynamically** based on concept drift magnitude — Too Many Entries = overfitting, Too Few = underfitting.

### 2.3 IDK-S: Incremental Distributional Kernel
**Source**: AAAI 2026 — https://doi.org/10.1609/aaai.v40i19.38642

**Key Breakthrough**: Data-dependent kernel mean embedding with incremental updates. 5-10x faster than Isolation Forest baselines. Linear-time complexity. Maintains data-dependent partitions by replacing obsolete ones with new arrivals.

**NEW vs Batch 549**:
- **Linear-time kernel method** for streaming anomaly detection — applicable to NT-SHIELD's real-time intrusion detection
- **Incremental partition replacement** — maps to NT-MEMORY's KB update strategy (incremental vs full reindex)

**Defect Found**: IDK-S's partition replacement assumes gradual drift. Abrupt regime changes cause temporary performance drops (shown in INSECTS dataset). NeoTrix needs a **regime-change detector** that triggers full reindex on abrupt shifts.

### 2.4 GMM Unexplained-Mass Drift Statistic
**Source**: arXiv (Jul 2026, updated Aug 2026) — https://arxiv.org/abs/2607.16811

**Key Breakthrough**: Dimension-calibrated chi-squared threshold replaces fixed 3σ radius. Unexplained mass = fraction of window matching no known GMM regime. Interpretable: reports which regime the data left and how far outside. Matches MMD (model-free kernel test) in performance while being self-explanatory.

**NEW vs Batch 549**:
- **Interpretable drift detection** — anomaly alarm is "10% of this window matches no known operating regime" (actionable)
- **Dimension calibration fix** — fixed 3σ threshold fails at d>9 due to concentration of measure (normal points sit at √d·σ from mean)
- **Two-class drift taxonomy**: novel-regime drift (unexplained mass detects it) vs in-support drift (MMD detects it, unexplained mass is blind)

**Defect Found**: The statistic is structurally blind to **in-support re-weighting** (same support, different weights). NeoTrix needs a **hybrid detector**: unexplained mass for novel regimes + MMD for in-support drift.

### 2.5 DESS: Data-Efficient Streaming TSAD with Evolving Proxies
**Source**: WWW 2026 — https://zheng-kai.com/paper/2026_www_wei.pdf

**Key Breakthrough**: Synthesizes compact **evolving proxy** summarizing historical data (no raw data storage). Heterogeneous temporal feature extraction (local + trend + seasonality) via cross-modality query + residual fusion. Parameter-efficient training (activates subset of lightweight parameters).

**NEW vs Batch 549**:
- **Evolving proxy** as compressed historical memory — maps to NT-MEMORY's KB (store distilled patterns, not raw data)
- **Cross-modality query + residual fusion** — architectural pattern for NT-WORLD's multi-sensor integration
- **64.88% training time reduction** with 17.53% accuracy improvement over baselines

**Defect Found**: Proxy synthesis quality degrades with extreme distribution shifts. NeoTrix should maintain **multiple proxies** (one per detected regime) rather than a single evolving proxy.

### 2.6 TFIDD: Feature-Importance Drift Detection
**Source**: Scientific Reports (Jul 2026) — https://www.nature.com/articles/s41598-026-60128-2

**Key Breakthrough**: Monitors **temporal feature-importance dynamics** instead of prediction errors or marginal distributions. Adaptive Top-k feature selection + temporal smoothing + confidence-based freezing. Reduces false alarms while preserving detection delay.

**NEW vs Batch 549**:
- **Feature-importance drift** — detects when *which features matter* changes, not just feature values. Directly applicable to NT-CORE's feature selection in reasoning
- **Confidence-based freezing** — stops updating when confident, restarts when uncertainty rises. Maps to NT-MIND's skill crystallization (freeze learned skills, unfreeze when drift detected)

**Defect Found**: TFIDD requires a base learner, adding coupling. NeoTrix needs a **model-agnostic version** that works on raw embeddings (VSA HyperCube vectors).

### 2.7 SAPDAD: Real-Time Adaptive Anomaly Detection in IIoT
**Source**: arXiv (Jan 2026) — https://arxiv.org/abs/2601.03085

**Key Breakthrough**: Combines multi-source LSTM prediction + PCA dimensionality reduction + Genetic Algorithm hyperparameter optimization + adaptive sliding/adaptive windowing for concept drift. 89.71% AUC on IIoT streams.

**NEW vs Batch 549**:
- **Multi-source prediction** — each sensor modality gets its own predictor, ensemble decisions. Maps to NT-WORLD's multi-fetcher architecture
- **GA-based hyperparameter optimization** for drift adaptation — could enhance NT-REPAIR's self-healing parameter tuning

**Defect Found**: GA optimization adds significant latency. For NeoTrix's real-time needs, **Bayesian optimization** or **hyperband** would be more efficient.

---

## 3. CHANGE POINT DETECTION — New Findings

### 3.1 SCAN: Nonparametric Multiple Change-Point Detection
**Source**: arXiv (Aug 2026) — https://arxiv.org/abs/2608.28110

**Key Breakthrough**: Combines integral probability metric (IPM) for detection + 1-Wasserstein distance for localization. Ensemble over multiple window sizes via majority voting. Reduces to CUSUM under pure mean shifts. Linear complexity on 1M+ observations.

**NEW vs Batch 549**:
- **Unified detection-localization** with nonparametric flexibility — detects mean, variance, AND general distributional changes
- **Window ensemble** via majority voting eliminates window-size sensitivity — applicable to NT-CORE's multi-scale analysis
- **Wasserstein localization** is more geometrically faithful than CUSUM (captures shape changes, not just mean shifts)

**Defect Found**: SCAN is offline (requires full sequence). NeoTrix needs an **online variant** that processes streaming data. The SWAL statistic could be adapted to an online EWMA form.

### 3.2 Online Distributional Change Detection in Metric Spaces
**Source**: arXiv (Aug 2026) — https://arxiv.org/abs/2608.26422

**Key Breakthrough**: Online testing framework using sequential energy distance and MMD for serially dependent data. Works in **any separable metric space** (scalar, functional, network-valued). Asymptotic false-alarm control. Handles both early and late changepoints.

**NEW vs Batch 549**:
- **Metric-space generality** — works on any data type representable in a metric space, directly applicable to VSA HyperCube vectors (which live in a metric space)
- **Sequential analogs** of energy distance and MMD for temporal dependence — addresses the i.i.d. assumption violation in real streams

**Defect Found**: Requires specifying a kernel/metric. NeoTrix's VSA HyperCube provides a natural metric (cosine distance in high-dimensional space), but the framework needs **adaptive kernel bandwidth** for non-stationary streams.

### 3.3 CHASM: Online Changepoint via Dynamic Mode Decomposition
**Source**: arXiv (May 2026) — https://arxiv.org/abs/2605.07852

**Key Breakthrough**: Monitors **truncated eigenvalue sequence** of recursively estimated DMD operator. Handles multivariate complex-valued time series. No distributional assumptions beyond finite moments. Uses optimal linear assignment for eigenvalue permutation invariance + augmented MEWMA for complex-valued monitoring.

**NEW vs Batch 549**:
- **Operator-level monitoring** — detects changes in dynamical structure (cross-variable temporal dependence), not just marginal statistics
- **Eigenvalue trajectory** as interpretable signal — shows *which modes* changed, applicable to NT-CORE's E8 reasoning state transitions
- **Permutation-invariant eigenvalue alignment** — novel technique for handling eigendecomposition instability across time

**Defect Found**: DMD requires stationarity within windows. NeoTrix's non-stationary temporal dynamics may violate this. A **windowed DMD with forgetting factor** (ρ parameter) is necessary.

### 3.4 STREAM-OOD: Regime-Aware Streaming OOD Detection
**Source**: CVPR 2026 Workshop — https://openaccess.thecvf.com/content/CVPR2026W/VAND/papers/Khalili_STREAM-OOD_Regime-Aware_Sequential_Monitoring_for_Streaming_Out-of-Distribution_Detection_CVPRW_2026_paper.pdf

**Key Breakthrough**: Reformulates OOD detection as **sequential regime inference** on representation manifold. Nonparametric streaming ID manifold estimate + conformal-gated adaptation (prevents contamination during OOD intervals) + Bayesian change point inference over standardized novelty scores. Learned hazard function uses score velocity, EMA drift, and adaptation state.

**NEW vs Batch 549**:
- **Conformal-gated adaptation** — only updates ID manifold when conformal calibration test passes. Directly applicable to NT-MEMORY's KB updates (only update embeddings when conformal test confirms they're in-distribution)
- **Learned hazard function** for regime transitions — replaces constant hazard in BOCPD. NeoTrix's GWT attention should use a similar context-dependent hazard
- **Event-level vs frame-level detection** — transforms noisy per-step deviations into coherent regime-level alerts. Maps to NT-CORE's consciousness-level event detection (not per-sensor alerts)

**Defect Found**: STREAM-OOD is vision-specific (frozen foundation model embeddings). NeoTrix needs adaptation to **time-series embeddings** (VSA HyperCube vectors) and **multi-modal fusion** (cross-sensor regime detection).

### 3.5 IDD: Distribution-Valued Online CPD via Wasserstein Tangent Space
**Source**: arXiv (Feb 2026) — https://arxiv.org/abs/2602.07252

**Key Breakthrough**: Treats streaming batch data as a stochastic process on 2-Wasserstein space. Maps each empirical distribution to a tangent space via OT relative to a Fréchet barycenter. Multivariate Functional PCA + Hotelling T² + SPE monitoring. Detects geometric changes (support shape, mass redistribution) that moment-based methods miss.

**NEW vs Batch 549**:
- **Wasserstein geometry awareness** — detects changes in distribution shape, not just moments. Applicable to NT-WORLD's content distribution monitoring (detecting shifts in content quality/complexity)
- **Tangent space linearization** of Wasserstein space — enables classical multivariate monitoring on distribution-valued data
- **ε-isometry guarantee** for the tangent approximation — theoretical foundation for NeoTrix's geometric reasoning

**Defect Found**: OT computation is expensive (Sinkhorn iterations). For NeoTrix's real-time requirements, **approximate OT** (random feature maps) or **dimensionality reduction before OT** is needed.

### 3.6 Drift Detection Benchmarking Framework
**Source**: arXiv (Jun 2026) — https://arxiv.org/abs/2606.07789

**Key Breakthrough**: Monte Carlo drift simulation injecting controlled changes into real-world datasets. New timing-aware metrics (F1 detection score, normalized detection time). Leave-one-dataset-out hyperparameter optimization for robustness. Benchmarks 14 methods across 4 drift types × 2 transitions.

**NEW vs Batch 549**:
- **Standardized evaluation protocol** — NeoTrix should adopt this for comparing drift detection approaches
- **Timing-aware F1** penalizes detection delay — critical for NT-REPAIR's real-time self-healing
- **Leave-one-dataset-out** for hyperparameter robustness — applicable to NeoTrix's cross-domain generalization testing

**Defect Found**: No benchmark includes streaming TSAD methods (only classification drift). NeoTrix needs **extension to regression/anomaly detection** drift types.

---

## 4. CROSS-DOMAIN SYNTHESIS: Defects vs Batch 549

| # | Defect in Batch 549 | Batch 550 Evidence | Proposed Fix |
|---|---------------------|-------------------|--------------|
| D1 | No temporal forecasting primitive | TimesFM-3 (single-pass multivariate), Timer-S1 (serial-token prediction), UniTok-FM (unified tokenization) | Add `TemporalForecaster` trait to NT-CORE with CPM + 2D attention grid. HyperCube projection for complexity reduction. |
| D2 | Threshold-based content safety filter | COMET (threshold-free codebook activation), GMM unexplained-mass (interpretable thresholds) | Replace thresholds with **codebook-based normal identification** + **chi²-calibrated regime membership** for interpretable safety decisions. |
| D3 | Static anomaly detector (no adaptation) | DyMETER (hypernetwork adaptation), COMET (online codebook adaptation), IDK-S (incremental kernel), DESS (evolving proxies) | Add `AdaptiveDetector` with: (1) hypernetwork for fast adaptation, (2) codebook for normal patterns, (3) conformal gate for contamination control. |
| D4 | No drift detection capability | SCAN (nonparametric CPD), CHASM (DMD eigenvalue monitoring), TFIDD (feature-importance drift), STREAM-OOD (Bayesian regime inference) | Add `DriftDetector` module with: (1) online Wasserstein-based CPD, (2) DMD eigenvalue trajectory monitoring, (3) learned hazard function for regime transitions. |
| D5 | No concept drift taxonomy | GMM two-class drift (novel-regime vs in-support), TFIDD (feature-importance vs value drift), Benchmark framework (4 drift types) | Formalize drift taxonomy: (1) novel-regime (new patterns), (2) in-support re-weighting (same patterns, different mix), (3) feature-importance (different features matter), (4) temporal structure (different dynamics). |
| D6 | No interpretable anomaly explanation | GMM unexplained-mass ("10% matches no regime"), TFIDD (which features drifted), CHASM (which eigenmodes changed) | Every anomaly alarm must include: (1) regime distance, (2) which regime left, (3) feature importance shift, (4) eigenmode trajectory. |
| D7 | No multi-scale temporal encoding | COMET (multi-scale patch encoding), DESS (local + trend + seasonality decomposition), SAPDAD (multi-source prediction) | Add `MultiScaleEncoder` with: patch-level, trend-level, seasonality-level encoding + cross-scale attention fusion. |
| D8 | No Wasserstein geometry awareness | IDD (2-Wasserstein tangent space), SCAN (Wasserstein localization), STREAM-OOD (nonparametric manifold) | Add `GeometricMonitor` using Wasserstein distance for distribution shape changes (not just moments). Approximate OT for real-time. |
| D9 | No online ensemble adaptation | DESS (evolving proxies + parameter-efficient training), IDK-S (incremental kernel replacement), DyMETER (dual-detector) | Combine: (1) evolving proxy for historical knowledge, (2) incremental updates for efficiency, (3) dual-detector for stability + adaptation balance. |
| D10 | No timing-aware quality metrics | Benchmark framework (normalized detection time, timing-aware F1), STREAM-OOD (fragmentation metric) | Adopt: timing-aware F1, normalized detection delay, fragmentation rate as standard quality metrics for all temporal modules. |

---

## 5. ARCHITECTURAL RECOMMENDATIONS FOR NEO773

### 5.1 New Module: `nt_core_temporal` (L5 Cognition)
**Responsibility**: Temporal forecasting + change point detection + concept drift monitoring
**Components**:
- `TemporalForecaster`: CPM + 2D attention (TimesFM-3 inspired), HyperCube projection for complexity
- `ChangePointDetector`: Online Wasserstein CPD + DMD eigenvalue monitoring
- `DriftMonitor`: GMM unexplained-mass + MMD hybrid + learned hazard function
- `MultiScaleEncoder`: Patch/trend/seasonality decomposition with cross-scale attention

### 5.2 New Module: `nt_core_adaptive_detector` (L3 Embodiment)
**Responsibility**: Online anomaly detection with concept drift adaptation
**Components**:
- `CodebookDetector`: Vector-quantized normal patterns + online adaptation
- `HypernetworkAdapter`: Instance-aware parameter shifts from uncertainty
- `ConformalGate`: Contamination-controlled adaptation updates
- `EvolvingProxy`: Compressed historical knowledge (no raw data storage)

### 5.3 GWT Enhancement: Temporal Attention Routing
**Change**: GWT should route attention based on:
1. Forecasting horizon length (longer horizon → more compute, Timer-S1 pattern)
2. Drift magnitude (higher drift → more attention to adaptation, DyMETER pattern)
3. Regime transition probability (learned hazard → proactive attention shift, STREAM-OOD pattern)

### 5.4 NT-MEMORY Enhancement: Temporal Pattern Index
**Change**: KB should store:
1. Temporal pattern embeddings (UniTok-FM tokenization)
2. Regime signatures (GMM component parameters per detected regime)
3. Drift history (change points + drift types + recovery times)
4. Proxy snapshots (DESS-style evolving proxies, one per regime)

---

## 6. SOURCES CITED

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | Google Research — TimesFM-3 | Sep 2026 | Multivariate forecasting, CPM, 2D attention |
| S2 | Tsinghua/Microsoft — Timer-S1 | Apr 2026 | MoE, Serial-Token Prediction, adaptive depth |
| S3 | arXiv — UniTok-FM | Jun 2026 | Universal tokenizer, NTP, prompt-boosted forecasting |
| S4 | Springer — ConTemPreT | Aug 2026 | Multi-task pre-training, masked + contrastive |
| S5 | arXiv — DyMETER | Apr 2026 | Hypernetwork adaptation, evidential uncertainty |
| S6 | arXiv — COMET | Feb 2026 | Codebook adaptation, multi-scale encoding |
| S7 | AAAI — IDK-S | Mar 2026 | Incremental kernel, linear-time streaming |
| S8 | arXiv — GMM Unexplained-Mass | Jul 2026 | Interpretable drift, chi² calibration |
| S9 | WWW — DESS | 2026 | Evolving proxies, parameter-efficient training |
| S10 | Sci Rep — TFIDD | Jul 2026 | Feature-importance drift detection |
| S11 | arXiv — SAPDAD | Jan 2026 | Multi-source prediction, GA optimization |
| S12 | arXiv — SCAN | Aug 2026 | Nonparametric CPD, Wasserstein localization |
| S13 | arXiv — Online Metric CPD | Aug 2026 | Sequential energy distance/MMD |
| S14 | arXiv — CHASM | May 2026 | DMD eigenvalue monitoring, complex-valued |
| S15 | CVPR Workshop — STREAM-OOD | Jun 2026 | Bayesian regime inference, conformal gate |
| S16 | arXiv — IDD | Feb 2026 | Wasserstein tangent space CPD |
| S17 | arXiv — Drift Benchmark | Jun 2026 | Standardized evaluation, timing-aware metrics |

---

## 7. SUMMARY: What's NEW vs Batch 549

Batch 549 established: living embeddings, personalization, policy safety, ANN retrieval.

Batch 550 adds **temporal intelligence** — the missing dimension:
1. **Temporal forecasting** (TimesFM-3/Timer-S1/UniTok-FM) — zero-shot, single-pass, multi-task
2. **Adaptive anomaly detection** (DyMETER/COMET/IDK-S/DESS) — threshold-free, online, contamination-controlled
3. **Change point detection** (SCAN/CHASM/STREAM-OOD/IDD) — nonparametric, Wasserstein geometry, Bayesian regime inference
4. **Drift taxonomy** (novel-regime vs in-support vs feature-importance vs temporal structure)
5. **Interpretable drift** (unexplained mass, eigenmode trajectories, feature importance shifts)

**10 defects** identified in batch 549's temporal blind spots. All have concrete fixes grounded in 2026 research.
