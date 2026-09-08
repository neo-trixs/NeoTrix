# Iteration Batch 358 — Temporal Pattern Analysis Research

**Date:** 2026-09-06
**Scope:** Time series foundation models, temporal pattern mining, anomaly/change point detection

---

## Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | Amazon Chronos-2 (HuggingFace + arxiv 2510.15821) | Oct 2025 | Zero-shot univariate/multivariate/covariate-informed forecasting |
| S2 | Google TimesFM-3 (Google Research blog) | Aug 2026 | Zero-shot multivariate forecasting in single forward pass |
| S3 | Salesforce MOIRAI-2 (MLM 2026 Toolkit survey) | Jan 2026 | Any-Variate Attention for universal forecasting |
| S4 | PaP-NF (ICPR 2026, arxiv 2605.23219) | May 2026 | Probabilistic long-term forecasting via Prefix-as-Prompt + Normalizing Flows |
| S5 | TimeGMM (emergentmind.com) | Jan 2026 | Single-pass probabilistic forecasting via adaptive Gaussian Mixture Models, 22.48% CRPS improvement |
| S6 | TimePrism (Dai et al., Sep 2025) | Sep 2025 | Scenario-based probabilistic forecasting with explicit probability pairs |
| S7 | STAR: State-aware Adapter (ICLR 2026, arxiv 2510.16014) | Oct 2025 | Foundation model adapter for multivariate anomaly detection with state variables |
| S8 | TimeRCD: Zero-shot anomaly detection (ICLR 2026) | Sep 2025 | Synthetic data pretraining + Relative Context Discrepancy for anomaly detection |
| S9 | ChronosAD (arxiv 2606.01300) | Jun 2026 | Foundation model as zero-shot feature extractor for anomaly detection |
| S10 | Sequential Pattern Mining survey (ScienceDirect, S0925231226009264) | Apr 2026 | Comprehensive SPM survey: GSP→SPADE→PrefixSpan→episode mining, Gap S8.4 multivariate complex event patterns |
| S11 | leSPM-SAHP (EDM 2026) | 2026 | Self-attentive Hawkes process for learning event sequential dependency mining without manual parameters |
| S12 | AnDri: Adaptive Anomaly Detection in Concept Drift (arxiv 2506.15831) | Jun 2025 | Online anomaly detection under gradual/abrupt/cyclic concept drift |
| S13 | D³R + Dynamic Concept Drift Suppression (Knowledge-Based Systems, 2026) | Mar 2026 | Generalizable anomaly detection framework with dynamic concept drift suppression |
| S14 | AnDri extended report: multivariate MTS anomaly via Dynamic Model Pool (arxiv 2601.02037) | Jan 2026 | Dynamic model pool + ensembling for multivariate time series anomaly detection |
| S15 | DistilTS: Distilling TS foundation models (arxiv 2601.12785) | Jan 2026 | First distillation framework for time series foundation models |
| S16 | Probabilistic Forecasting topic survey (emergentmind.com) | Feb 2026 | CRPS, calibrated prediction intervals, TimePrism, K²VAE |
| S17 | Adaptive Conformal Anomaly Detection with TSFMs (ICLR 2026) | 2026 | Conformalized anomaly detection using foundation model predictions |

---

## Research Findings & Defects

### 1. TIME SERIES FOUNDATION MODELS

**Finding (S1/S2/S3):** 2026 production TSFMs (Chronos-2, TimesFM-3, MOIRAI-2) now achieve zero-shot forecasting with 8K+ context windows, multivariate support, and covariate-informed predictions. Chronos-2 uses group attention for in-context learning across related series. TimesFM-3 handles multivariate in a single forward pass.

**Defect DT-1: NeoTrix `ForecastEngine` is deterministic-scenario-tree, not probabilistic**
- `nt_core_forecast.rs` generates a 3-leaf scenario tree (bull/bear/flat) with hand-tuned probabilities and LLM narration. It does NOT produce calibrated probability distributions or quantile forecasts.
- The 2026 standard (S5 TimeGMM, S4 PaP-NF, S6 TimePrism) requires full predictive distributions with CRPS-calibrated uncertainty.
- **Impact:** `ForecastEngine.abstain` only triggers on "low signal" — it has no mechanism to detect when its point estimates are outside the calibrated uncertainty envelope.

**Suggestion:** Refactor `ForecastEngine` output to include `PredictiveDistribution { quantiles: Vec<(f64, f64)>, crps: f64, calibration_score: f64 }` alongside the scenario tree. Consider integrating a lightweight probabilistic adapter (DistilTS-style distillation from Chronos-2) for production inference.

**Defect DT-2: No long-context temporal reasoning (>8K timesteps)**
- Chronos-2 supports 8,192-step context post-training (S1). TimesFM-3 handles long-range seasonal patterns spanning years.
- NeoTrix `AttentionHead::receptive_field` defaults to 10, with no mechanism for adaptive receptive field expansion based on seasonal period detection.
- **Impact:** The system cannot detect multi-year seasonality in knowledge graph evolution, sensor data, or code contribution patterns.

**Suggestion:** Implement adaptive receptive field in `AttentionHead` that scales with detected seasonal period (SAX-based period detection, similar to the approach in S10 Ch.8.4.1). Add a `SeasonalPeriodDetector` in the `nt_world_sense` layer.

### 2. TEMPORAL PATTERN MINING

**Finding (S10/S11):** Modern sequential pattern mining has evolved beyond GSP/SPADE/PrefixSpan. Self-attentive Hawkes processes (S11) can discover sequential dependencies without manual parameter tuning (min_support, gap, interval). Episode mining handles partial orders (serial, parallel, general episodes). Gap S8.4 (S10) identifies multivariate complex event patterns as a critical open problem.

**Defect DT-3: No sequential pattern mining in NeoTrix**
- NeoTrix has `nt_temporal_facts` (append-only temporal fact store in KB) and `TemporalEvidenceTracker` (trend direction/volatility), but NO sequential pattern mining capability.
- The system cannot discover that "pattern A → B → C" occurs frequently in event sequences, which is critical for SEAL pipeline optimization (understanding which evolution stages predict which outcomes).
- The `EventBus` (tokio broadcast) logs events but has no mining/analysis layer.

**Suggestion:** Implement a lightweight `SequentialPatternMiner` in `nt_core_self` that operates on `CoreEvent` sequences. Start with a PrefixSpan-style approach for discrete event patterns. For the Hawkes process approach (S11), the `AttentionManager` in `nt_core_self` already has attention weights that could be repurposed as influence intensity functions.

**Defect DT-4: No episode mining for partial-order event dependencies**
- NeoTrix's `EventBus` treats all events as totally ordered (global `seq` number). Real system behavior involves parallel events (e.g., multiple modules responding simultaneously to a stimulus).
- Episode mining (Mannila 1997, refined in S10) handles partial orders where some events are unordered with respect to each other.
- **Impact:** The system cannot distinguish between "A must precede B" vs "A and B can occur in any order," leading to false pattern discoveries.

**Suggestion:** Extend `CoreEvent` with an `episode_group_id: Option<String>` field to mark events that belong to the same causal wave. The pattern miner can then treat these as parallel episodes.

### 3. ANOMALY DETECTION & CONCEPT DRIFT

**Finding (S7/S8/S9/S12/S13/S14/S17):** 2026 anomaly detection for time series requires:
- **State-variable awareness** (S7 STAR): Discrete state variables (valve on/off, module activation state) must be modeled separately from numerical variables.
- **Concept drift co-detection** (S12/S13): Anomalies and concept drift must be simultaneously detected and distinguished — change points alter distribution permanently while anomalies are transient.
- **Foundation model adaptation** (S8 TimeRCD, S9 ChronosAD, S17 Conformal): Zero-shot anomaly detection via relative context discrepancy, conformal calibration, and state-aware adapters.
- **Dynamic model ensembling** (S14): Dynamic model pools adapt to non-stationary data distributions.

**Defect DT-5: No concept drift detection in NeoTrix consciousness metrics**
- `HeartbeatAggregator` collects `SystemHealthSnapshot { phi, coherence, gwt_resonance }` and `ConvergenceSignal` (in `nt_core_ttc.rs`) only checks for linear degradation trends.
- There is NO change point detection mechanism. A sudden shift in `phi` (e.g., from 0.42 to 0.71 after a major SEAL absorption) is treated identically to gradual drift.
- The `AttentionManager` uses fixed `decay_rate: 0.1` — no adaptive adjustment for distribution shifts.
- **Impact:** The system cannot distinguish between "phi dropped because a module broke" (anomaly) vs "phi dropped because the system evolved to a new normal" (concept drift / change point). Both trigger identical responses.

**Suggestion:** Implement a change point detector (CUSUM or Bayesian Online CPD) in the `HeartbeatAggregator` pipeline. When a change point is detected in `phi`/`coherence`/`gwt_resonance`:
1. Flag the shift as "distribution change" vs "anomaly" using the dual criteria from S12 (transient vs permanent).
2. Trigger `AttentionManager` to reset decay rates and re-calibrate salience thresholds.
3. Log the change point in `nt_temporal_facts` with `type: "distribution_shift"`.

**Defect DT-6: No multivariate anomaly detection across consciousness dimensions**
- The 7 dimensions of `SystemHealthSnapshot` (phi, coherence, gwt_resonance, module_health, etc.) are monitored independently. S7 STAR and S14 Dynamic Model Pool show that cross-variate dependencies are critical for detecting anomalies in correlated systems.
- A drop in `phi` WITHOUT a corresponding drop in `coherence` is more suspicious than both dropping together (the latter may indicate normal evolution).
- **Impact:** False positive rate is likely high because the system cannot model cross-dimensional correlations.

**Suggestion:** Implement a lightweight multivariate anomaly detector (S14-style dynamic model pool or STAR-style state-aware adapter) that monitors the joint distribution of consciousness metrics. The `AttentionManager::salience(novelty, coherence)` already computes a joint signal — extend this to a proper multivariate anomaly score using Mahalanobis distance or an Isolation Forest variant.

**Defect DT-7: No conformal calibration for anomaly thresholds**
- S17 (ICLR 2026) shows that conformalized anomaly detection provides distribution-free coverage guarantees. NeoTrix uses hard-coded thresholds (e.g., `phi_threshold: 0.5`, `coherence_threshold: 0.7` in `src-tauri/src/config.rs`).
- These thresholds are not calibrated against the actual distribution of the metrics.
- **Impact:** Thresholds that work for one deployment may cause excessive false positives or miss real anomalies in another.

**Suggestion:** Implement conformal calibration for all anomaly thresholds. The conformal framework from S17 provides a simple wrapper: maintain a calibration set of recent metric values, compute the (1-α) quantile of nonconformity scores, and use that as the threshold. This gives distribution-free coverage guarantees.

---

## Summary

| # | Defect | Severity | Domain |
|---|--------|----------|--------|
| DT-1 | ForecastEngine lacks probabilistic distributions | HIGH | NT-CORE (forecast) |
| DT-2 | No long-context seasonal pattern detection | MEDIUM | NT-CORE (attention) |
| DT-3 | No sequential pattern mining on event sequences | HIGH | NT-CORE / NT-MEMORY |
| DT-4 | No episode mining for partial-order events | MEDIUM | NT-CORE (event bus) |
| DT-5 | No concept drift detection in consciousness metrics | HIGH | NT-CORE (heartbeat) |
| DT-6 | No multivariate anomaly detection across dimensions | HIGH | NT-CORE (self model) |
| DT-7 | Hard-coded anomaly thresholds without calibration | MEDIUM | NT-SHIELD / config |

**Critical path:** DT-5 and DT-6 together mean NeoTrix cannot reliably distinguish "the system evolved" from "the system broke" — the most fundamental self-awareness requirement. DT-1 means all forecasts lack quantified uncertainty, making the `abstain` mechanism unreliable.
