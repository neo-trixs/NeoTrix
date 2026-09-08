# Iteration Batch 519 — Statistical Learning, Bayesian Methods & Model Evaluation

**Date**: 2026-09-06
**Focus**: External research advances in statistical learning, Bayesian inference, and model evaluation → NeoTrix defect identification

---

## Sources Cited

| # | Source | Year | Topic | URL |
|---|--------|------|-------|-----|
| S1 | ICML 2026 Workshop on Hypothesis Testing | 2026 | Sequential testing, distribution shift, LLM evaluation | https://hypothesis-testing-workshop.vercel.app/ |
| S2 | NeurIPS 2026 Workshop: E-Values from Statistics to ML | 2026 | E-values, e-processes, anytime-valid inference, conformal prediction | https://e-values-workshop.github.io/ |
| S3 | E-valuator (ICLR 2026) | 2026 | Sequential hypothesis testing for agent trajectory verification, e-processes | https://arxiv.org/abs/2512.03109 |
| S4 | Yao Xie, ICML 2026 Keynote | 2026 | Sequential change-point detection, MMD/Stein critics, online kernel CUSUM for LLM monitoring | https://icml.cc/virtual/2026/workshop/54092 |
| S5 | Yin & Jiao, arXiv:2606.29205 | 2026 | VI improved MCMC efficiency — GVI+HMC and VAE-MH hybrid | https://arxiv.org/abs/2606.29205 |
| S6 | Sun et al. (ICML 2026), arXiv:2601.22367 | 2026 | Amortized variational approximation for tempered posteriors (β-NPE) | https://arxiv.org/abs/2601.22367 |
| S7 | Feng & Huan, arXiv:2605.03710 | 2026 | Amortized VI for joint posterior-predictive distributions (UQ) | https://arxiv.org/abs/2605.03710 |
| S8 | Iqbal & Du, Information Sciences 2026 | 2026 | Multi-Pass Bayesian Estimation (MPBE) — unified MCMC+VI with calibrated uncertainty | DOI:10.1016/j.ins.2026.123752 |
| S9 | MetricGate, MCMC vs VI Comparison | 2026 | Hybrid VI→MCMC workflows, mean-field bias, diagnostics | https://metricgate.com/blogs/mcmc-vs-variational-inference/ |
| S10 | Sequential Randomization Tests (e-RT) | 2026 | Nonparametric sequential tests using e-values, betting framework | https://arxiv.org/abs/2512.04366 |
| S11 | MDPI 2026 Hypothesis Testing Review | 2026 | Comprehensive review: Bayesian testing, bootstrap, A/B testing, modern reform | https://www.mdpi.com/2227-7390/14/2/300 |
| S12 | Cross-Validation 2026 Guide (Code Labs Academy) | 2026 | Stratified CV, time-series CV, nested CV, group-aware splitting | https://codelabsacademy.com/en/blog/cross-validation-strategies-machine-learning-2026-guide/ |

---

## Defects Found

### DEFECT-519-01: No E-Value / Anytime-Valid Inference Framework

**Severity**: HIGH
**Location**: `neotrix-core/src/unified/core/nt_core_hcube/bayesian_experiment.rs`, `metacalib.rs`
**Evidence**: NeoTrix's hypothesis testing uses classical p-value equivalents (E8 resonance scoring, posterior normalization) and static ECE/Brier calibration. There is no e-value computation, no e-process, no test martingale. The `CalibrationTracker` in `nt_core_forecast.rs:626` records (probability, outcome) pairs but does not support sequential monitoring or anytime-valid inference.
**2026 Advance**: S2 (NeurIPS 2026 Workshop) and S3 (E-valuator, ICLR 2026) demonstrate that e-values enable anytime-valid hypothesis testing — critical for monitoring evolving AI systems. E-values support continuous monitoring without inflating false alarm rates, unlike fixed-horizon tests. S10 shows nonparametric sequential tests via betting frameworks. S1 (ICML 2026) explicitly targets "sequential hypothesis testing for modern AI systems."
**Impact**: NeoTrix cannot perform continuous monitoring of its own reasoning quality or deployed agent trajectories. The ConsciousnessTree's health checks are point-in-time snapshots, not streaming tests. Any adaptation (SEAL pipeline, self-healing) lacks statistical guarantees during ongoing operation.

### DEFECT-519-02: No Conformal Prediction for Distribution-Free Coverage

**Severity**: MEDIUM-HIGH
**Location**: `neotrix-core/src/unified/core/nt_core_consciousness_tree/metacalib.rs` (ECE only), `nt_core_forecast.rs` (CalibrationTracker)
**Evidence**: The `expected_calibration_error` function (metacalib.rs:6) computes ECE with fixed bins but provides no coverage guarantee. There is no conformal prediction, no conformalized amortized inference (CANVI), no prediction sets with marginal coverage. The `calibrated_confidence` method (forecast.rs:310) applies a simple penalty multiplier, not a distribution-free coverage guarantee.
**2026 Advance**: S7 (CANVI, ICML 2024, still state-of-art 2026) provides guaranteed marginal coverage for amortized neural posteriors. S2 lists "conformal prediction, adaptive coverage, and e-based prediction sets" as active NeurIPS 2026 topics. S6 (β-NPE) enables amortized posterior estimation but without coverage guarantees unless paired with conformalization.
**Impact**: NeoTrix's confidence scores (branch health, forecasting confidence, hypothesis posterior) have no formal coverage guarantee. Users cannot trust that a "90% calibrated" prediction actually achieves 90% coverage over time. This limits deployment in safety-critical or regulated domains.

### DEFECT-519-03: Bayesian Inference Lacks MCMC-VI Hybrid Workflow

**Severity**: MEDIUM
**Location**: `neotrix-core/src/unified/core/nt_core_hcube/bayesian_experiment.rs` (VoI computation), `nt_core_hcube/aif/` (belief update, free energy)
**Evidence**: NeoTrix implements Bayesian belief updating (POMDP belief update in `aif/belief/pomdp.rs`) and VoI-based experiment selection (bayesian_experiment.rs:144), but uses deterministic point-estimate posterior normalization. No MCMC sampling, no variational approximation, no hybrid VI→MCMC workflow. The belief state is a fixed vector, not a distribution.
**2026 Advance**: S5 (Yin & Jiao, 2026) proposes GVI→HMC and VAE-MH hybrids that combine VI speed with MCMC exactness. S9 (MetricGate 2026) documents the standard production workflow: "Fit VI first for fast approximation, diagnose with PSIS, then run MCMC initialized from VI mode." S8 (MPBE) achieves calibrated 95% PICP with 500-pass multi-pass estimation. S6 (β-NPE) enables amortized inference in a single forward pass.
**Impact**: NeoTrix's Bayesian reasoning uses point estimates instead of full posterior distributions. VoI computation (bayesian_experiment.rs:144) approximates KL divergence analytically rather than via sampling, which fails for multimodal or heavy-tailed posteriors. The active inference module (`aif/free_energy.rs`) computes expected free energy with deterministic beliefs, missing epistemic uncertainty entirely.

### DEFECT-519-04: No Model Evaluation Infrastructure (Cross-Validation, A/B Testing, Effect Sizes)

**Severity**: MEDIUM-HIGH
**Location**: Entire codebase — no `nt_meta::model_evaluation`, no `nt_core::cross_validation`, no A/B test framework
**Evidence**: NeoTrix has SelfTest tiers (T1/T2/T3) for module health but no cross-validation framework for model comparison, no A/B testing infrastructure, no effect size computation (Cohen's d, Hedges' g), no power analysis. The `SelfTest` trait outputs health status, not statistical performance metrics. There is no `cross_val_score`, no stratified splitting, no nested CV for hyperparameter selection.
**2026 Advance**: S12 (2026 CV guide) emphasizes stratified CV, time-series CV, and group-aware splitting as production standards. S11 (MDPI 2026 review) documents that modern evaluation requires effect sizes + confidence intervals alongside p-values, not just binary significance. S1 (ICML 2026) lists "testing under adaptivity, dependence, and privacy constraints" as active research.
**Impact**: NeoTrix cannot statistically compare module variants (e.g., different SEAL pipeline configurations, different reasoning strategies), cannot evaluate whether self-healing improvements are significant, and cannot perform rigorous A/B testing on reasoning approaches. The evolution loop relies on heuristics rather than statistical rigor.

### DEFECT-519-05: No Sequential Change-Point Detection for Distribution Drift

**Severity**: MEDIUM
**Location**: `neotrix-core/src/unified/core/nt_core_consciousness_tree/ops.rs` (health computation), `nt_core_forecast.rs` (static calibration)
**Evidence**: ConsciousnessTree health is computed from SelfTest samples at fixed points. There is no online change-point detection, no CUSUM, no MMD-based nonparametric monitoring. The `CalibrationTracker` accumulates records but does not detect when calibration has drifted.
**2026 Advance**: S4 (Yao Xie, ICML 2026) presents modern sequential change-point detection using MMD, kernel witnesses, Stein critics, and online kernel CUSUM — specifically for post-deployment monitoring of LLM systems. S3 (E-valuator) uses e-detectors for nonparametric sequential change detection.
**Impact**: When NeoTrix's internal models or external data sources shift distribution (e.g., new LLM provider behavior, changed KB content quality), there is no automatic detection. The system continues operating with stale calibration until manual intervention.

### DEFECT-519-06: Metacognitive Calibration is Point-Estimate Only (No Uncertainty Propagation)

**Severity**: MEDIUM
**Location**: `neotrix-core/src/unified/core/nt_core_consciousness_tree/metacalib.rs`, `ffi/consciousness_tree.rs:308`
**Evidence**: `apply_metacognitive_calibration` (ffi/consciousness_tree.rs:308) takes health as f32 and returns a single calibrated value. `calibrate_branch_health` (ffi/consciousness_tree.rs:319) computes ECE and Brier but returns scalar health, not a distribution. No confidence intervals on the calibrated health, no posterior uncertainty over the calibration parameters.
**2026 Advance**: S7 (Amortized VI for joint posterior-predictive) and S8 (MPBE) demonstrate that calibration should produce distributional outputs, not point estimates. S6 (β-NPE) shows that tempered posteriors enable robust calibration under model misspecification.
**Impact**: The ConsciousnessTree's `calibrated_health` field (types.rs:193) is a single f64 — it cannot express "health is 0.73 ± 0.05." Downstream consumers (GWT attention routing, SEAL pipeline decisions) treat the calibrated health as certain, propagating false confidence.

---

## Suggestions

| ID | Suggestion | Target Module | Priority | Related Defect |
|----|-----------|---------------|----------|----------------|
| S-519-01 | Implement `nt_core::e_value` module: e-value computation for hypothesis testing, test martingales for sequential monitoring, anytime-valid confidence sequences. Wire to ConsciousnessTree for continuous health monitoring. | nt_core | HIGH | DEFECT-519-01 |
| S-519-02 | Add conformal prediction layer: implement split conformal prediction for distribution-free coverage guarantees on all confidence outputs (branch health, forecasting, hypothesis posterior). Produce prediction sets instead of point estimates. | nt_core_consciousness_tree, nt_core_forecast | HIGH | DEFECT-519-02 |
| S-519-03 | Build hybrid VI→MCMC inference engine: VI for fast approximate posterior, MCMC refinement for critical decisions. Use GVI+HMC as in S5. Store full posterior samples, not point estimates, for VoI and free energy computation. | nt_core_hcube (bayesian_experiment, aif) | MEDIUM | DEFECT-519-03 |
| S-519-04 | Implement `nt_meta::model_evaluation` framework: stratified k-fold CV, time-series CV, nested CV for hyperparameter selection, effect size computation (Cohen's d, Hedges' g), power analysis, A/B test significance testing. | nt_meta | HIGH | DEFECT-519-04 |
| S-519-05 | Add online change-point detection: implement CUSUM, MMD-based two-sample test, and kernel CUSUM for monitoring distribution drift in model outputs, KB content quality, and external data sources. Wire to GWT for automatic attention reallocation on drift detection. | nt_meta, nt_world | MEDIUM | DEFECT-519-05 |
| S-519-06 | Extend calibration to distributional outputs: replace scalar `calibrated_health` with `(mean, variance)` or full posterior sample. Add confidence intervals to all health/forecasting outputs. Use β-tempered posteriors (S6) for robust calibration under misspecification. | nt_core_consciousness_tree | MEDIUM | DEFECT-519-06 |

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 12 |
| Defects found | 6 |
| Suggestions | 6 |
| HIGH priority | 3 (DEFECT-519-01, -02, -04) |
| MEDIUM priority | 3 (DEFECT-519-03, -05, -06) |

**Key Theme**: NeoTrix has solid foundations in Bayesian reasoning (VoI, belief updating, ECE calibration) but operates entirely in point-estimate mode. The 2026 frontier is **distributional calibration** (conformal prediction, full posteriors), **anytime-valid inference** (e-values, test martingales), and **sequential monitoring** (online change-point detection for deployed AI). The gap between NeoTrix's current capabilities and 2026 production standards is primarily in **statistical guarantees** — the system produces calibrated point estimates but cannot guarantee coverage, cannot monitor continuously, and cannot detect drift automatically.
