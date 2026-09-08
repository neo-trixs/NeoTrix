# Iteration Batch 545 — Probabilistic Programming / Bayesian Deep Learning / Monte Carlo Methods

**Date:** 2026-09-06
**Previous Batch:** 544 (spectral-throughput trade-off, adversarial robustness, streaming error accumulation, rank estimation non-stationarity)
**Status:** Research sweep — 3 domains, 6 new defects identified

---

## Batch 544 Baseline Recap

| # | Defect | Domain |
|---|--------|--------|
| D1 | Spectral-throughput trade-off unquantified | Neural preconditioners |
| D2 | No adversarial robustness on neural preconditioners | Neural preconditioners |
| D3 | Streaming error accumulation without rollback | Sequential inference |
| D4 | Rank estimation non-stationarity blindness | Low-rank adaptation |

---

## 1. Probabilistic Programming

### 1.1 ONNX Probabilistic Programming Working Group (2026-03)
**Source:** https://discourse.mc-stan.org/t/onnx-probabilistic-programming-working-group-announcement-call-for-contributors/40957

**Key Finding:** ONNX PPL Working Group (March 2026) aims to standardize probabilistic operators (distributions, log-probabilities, factors), bijectors, reproducible stateless RNG semantics, and inference algorithm ports (Laplace, Pathfinder, INLA, HMC, NUTS, SMC) across frameworks (Stan, PyMC, Pyro, NumPyro, TFP, Turing).

**NEW Defect vs Batch 544:**
- **D5 — Cross-framework probabilistic portability gap:** NeoTrix's SEAL pipeline has no standardized probabilistic IR. When switching between inference backends (e.g., Stan HMC → Pyro VI), model semantics are lost. The ONNX PPL effort exposes this as a real interoperability failure — probabilistic models can't be serialized/deserialized across frameworks without semantic drift. Batch 544's spectral-throughput issue is *amplified* when the preconditioner is trained in one framework but deployed in another.

### 1.2 Turing.jl DynamicPPL Accumulator Architecture
**Source:** https://github.com/TuringLang/DynamicPPL.jl (push 2026-05-06)

**Key Finding:** DynamicPPL v0.41.7 introduces an **Accumulator** system — custom accumulators collect log-densities, raw parameter values, and arbitrary metadata during model execution. This decouples model execution from inference diagnostics.

**NEW Defect vs Batch 544:**
- **D6 — Missing execution-trace accumulator in NT-CORE:** Batch 544 identified streaming error accumulation (D3). DynamicPPL's accumulator pattern reveals that NeoTrix lacks a model-execution trace accumulator — every inference step produces log-densities, gradient norms, and acceptance rates that are currently discarded. Without these, error accumulation (D3) cannot be diagnosed post-hoc. The accumulator pattern is the missing diagnostic substrate for streaming error rollback.

### 1.3 Bayesian Nonparametric PPL Benchmarks
**Source:** https://github.com/luiarthur/TuringBnpBenchmarks

**Key Finding:** Systematic BNP benchmarks across Stan, Pyro, Nimble, TFP, and Turing show that HMC/NUTS on stick-breaking constructions is 10-100x slower than collapsed Gibbs for Dirichlet Process mixtures. Variational inference for stick-breaking is competitive in speed but systematically underestimates tail uncertainty.

**NEW Defect vs Batch 544:**
- **D7 — VI tail-uncertainty suppression in BNP regime:** NeoTrix's variational inference path (used in SEAL distillation) systematically underestimates tail uncertainty for nonparametric models. When rank estimation (D4 non-stationarity) is combined with VI's tail suppression, the system produces falsely confident rank estimates in heavy-tailed regimes. The BNP benchmarks show this is a known, quantified gap — not just theoretical.

---

## 2. Bayesian Deep Learning

### 2.1 Calibrated Variance Propagation (CVP) — Single-Pass UQ
**Source:** https://arxiv.org/html/2606.16214v2 (2026-06)

**Key Finding:** CVP achieves MC-sampling-quality uncertainty in a **single forward pass** by analytically propagating mean+variance through LayerNorm, activations, and residuals. Key innovation: learnable calibration scaling factors at key layers absorb accumulated approximation error. Coverage at 0.5% risk improves from 8.2% → 14.6% (BEiT-3) over prior variance propagation.

**NEW Defect vs Batch 544:**
- **D8 — Variance propagation calibration blind spot:** CVP introduces learnable scaling factors to absorb error accumulation across layers. This directly addresses batch 544's D3 (streaming error accumulation) but reveals a new problem: the calibration scaling is **fitted on a fixed calibration set** and degrades under distribution shift. NeoTrix's streaming architecture faces the same issue — any error-absorption mechanism trained on static data becomes a liability under non-stationarity. The "light calibration step" is a hidden overfitting risk.

### 2.2 Bayesian RND Equivalence Theorem
**Source:** https://proceedings.mlr.press/v337/zanger26a.html (UAI 2026)

**Key Finding:** Random Network Distillation's squared self-predictive error is **provably equivalent** to deep ensemble predictive variance in the infinite-width NTK limit. Furthermore, RND can be modified to mirror the centered posterior predictive of Bayesian inference. This yields a sampling algorithm generating i.i.d. exact posterior predictive samples.

**NEW Defect vs Batch 544:**
- **D9 — RND-equivalence assumption collapses in finite-width regime:** The equivalence holds only at infinite width. NeoTrix's HyperCube representations are finite-dimensional by design. Using RND-style novelty detection as a proxy for ensemble uncertainty introduces a silent error that scales with (1/width). This is a new dimension of the adversarial robustness gap (D2) — adversarial perturbations that exploit finite-width deviations from NTK behavior can fool the uncertainty estimator without fooling the model.

### 2.3 Preconditioned Sampling from Optimizer-Derived Geometry
**Source:** https://pubdb.com/paper/2607.25312 (2026)

**Key Finding:** Curvature estimates computed during AdamW warmstart (as a free byproduct) can precondition MCMC sampling, eliminating burn-in phases and improving numerical stability. The optimizer's running variance of gradient moments provides an implicit Fisher information estimate.

**NEW Defect vs Batch 544:**
- **D10 — Optimizer-geometry preconditioning mismatch:** NeoTrix's neural preconditioners use fixed spectral conditioning. This paper shows that the *optimizer's own accumulated geometry* is a superior preconditioner than any externally computed spectral estimate. This directly quantifies batch 544's D1 (spectral-throughput trade-off): the trade-off exists because NeoTrix computes spectral properties independently of the optimizer trajectory, when it should be using the optimizer's implicit geometry.

### 2.4 E-Value Stopping Rules for Bayesian Deep Ensembles
**Source:** https://arxiv.org/abs/2604.18089 (2026)

**Key Finding:** E-value based sequential hypothesis tests can determine when MCMC sampling over deep ensemble chains has yielded sufficient improvement over the optimized baseline. Only a fraction of the full chain budget is typically needed. This is an anytime-valid stopping rule — no look-ahead penalty.

**NEW Defect vs Batch 544:**
- **D11 — No principled sampling-termination criterion:** NeoTrix's SEAL pipeline runs inference for a fixed number of iterations or until a heuristic convergence threshold. The E-value framework provides a statistically valid stopping rule that batch 544's streaming architecture completely lacks. Without it, the system either over-samples (wasting compute) or under-samples (accepting poor posterior approximations), and there is no way to distinguish these cases.

### 2.5 CreDE — Credal Deep Ensembles
**Source:** https://arxiv.org/html/2607.28248 (2026-07 survey)

**Key Finding:** Credal Deep Ensembles (CreDE) produce interval-valued outputs using distributionally robust optimization, quantifying both aleatoric and epistemic uncertainty via credal sets (convex sets of distributions). Improves OOD detection over standard ensembles while maintaining in-distribution performance. Variance-gated measures (VGMU, VGN, 2026) provide a cleaner epistemic margin score than entropy decomposition.

**NEW Defect vs Batch 544:**
- **D12 — Imprecise-probability gap in NT-FEEL emotion modeling:** NeoTrix's EmotionLabel system uses point-valued probabilities for emotion states. CreDE demonstrates that interval-valued (credal) representations are strictly superior for OOD detection and calibration. Batch 544's adversarial robustness gap (D2) is partially explained by this: point-valued emotion estimates have no inherent mechanism to signal "I don't know" — credal sets do.

---

## 3. Monte Carlo Methods

### 3.1 Adaptive Subsampling Sequential MCMC
**Source:** https://doi.org/10.1155/dsn/6527524 (2026)

**Key Finding:** Adaptive subsampling SMCMC (AS-SMCMC) processes only a data subset at each time step, with confidence-interval-based MH kernels that adapt batch size. When the filtering distribution is concentrated, speedup is substantial; when uncertain, it degrades gracefully to full-data evaluation. Key insight: SMCMC empirically outperforms standard SMC in high-dimensional systems despite both suffering curse of dimensionality.

**NEW Defect vs Batch 544:**
- **D13 — No adaptive data-budget allocation in NT-WORLD crawl pipeline:** NeoTrix's world perception layer processes all fetched data uniformly. AS-SMCMC shows that adaptive subsampling with confidence-gated acceptance can achieve 5-10x speedup without accuracy loss when the filtering distribution is concentrated. Batch 544's streaming error accumulation (D3) is exacerbated by processing irrelevant data — adaptive budget allocation would reduce both compute and error propagation.

### 3.2 Augmented Island Resampling Particle Filter (AIRPF) for PMCMC
**Source:** https://arxiv.org/html/2312.06484

**Key Finding:** AIRPF runs m interacting particle filters in parallel with adaptive inter-filter resampling controlled by Effective Number of Filters (ENF). Produces non-negative unbiased marginal likelihood estimators suitable for PMCMC. The key theorem: AIRPF's error can be bounded uniformly in time by controlling ENF above a threshold, preventing the degeneracy that plagues independent BPFs.

**NEW Defect vs Batch 544:**
- **D14 — Independent particle populations without cross-filter interaction:** NeoTrix's parallel inference paths (when running multiple SEAL instances) operate independently. AIRPF demonstrates that **interacting** parallel filters with adaptive resampling outperform independent ones by a provable margin. The ENF metric provides a concrete diagnostic: if ENF collapses to 1, only one filter contributes — equivalent to wasting (m-1)/m of compute. Batch 544's rank estimation blindness (D4) is partially caused by independent rank estimates that never cross-validate.

### 3.3 Hamiltonian Trajectory Ensemble (HTE) — Biased but Superior
**Source:** https://link.springer.com/article/10.1038/s41467-026-70015-z (Nature Comms, 2026-03)

**Key Finding:** HTE uses long OBABO stochastic gradient trajectories with MH acceptance steps. It does **not** converge to the true posterior — it is an ensemble method, not Bayesian inference. Yet it achieves +5.8% accuracy over deterministic and +4.3% over Bayesian baselines, with calibration guarantees. The MH acceptance step acts as a quality filter on trajectory diversity.

**NEW Defect vs Batch 544:**
- **D15 — Bayesian-vs-ensemble epistemic confusion:** NeoTrix's architecture conflates Bayesian posterior sampling with ensemble diversity. HTE proves that **biased but well-mixed** ensembles can outperform exact Bayesian inference on practical metrics. The MH acceptance step is the key differentiator — it filters trajectory quality without requiring convergence. NeoTrix lacks this quality gate: its SEAL pipeline accepts or rejects based on point-estimate improvement, not trajectory diversity. This is a new dimension of batch 544's adversarial robustness gap (D2).

---

## Summary: New Defects vs Batch 544

| # | Defect | Source Domain | Severity | Relationship to Batch 544 |
|---|--------|---------------|----------|---------------------------|
| D5 | Cross-framework probabilistic portability gap | Probabilistic Programming | Medium | Amplifies D1 (spectral-throughput) when switching backends |
| D6 | Missing execution-trace accumulator | Probabilistic Programming | **High** | Directly enables diagnosis of D3 (streaming error) |
| D7 | VI tail-uncertainty suppression in BNP regime | Probabilistic Programming | Medium | Compounds D4 (rank estimation non-stationarity) |
| D8 | Variance propagation calibration blind spot | Bayesian Deep Learning | **High** | New failure mode for D3 error-absorption mechanisms |
| D9 | RND-equivalence finite-width collapse | Bayesian Deep Learning | **High** | Expands D2 (adversarial robustness) to uncertainty estimation |
| D10 | Optimizer-geometry preconditioning mismatch | Bayesian Deep Learning | **High** | Directly quantifies D1 (spectral-throughput trade-off) |
| D11 | No principled sampling-termination criterion | Bayesian Deep Learning | Medium | Wastes compute or accepts poor posteriors in D3 streaming |
| D12 | Imprecise-probability gap in emotion modeling | Bayesian Deep Learning | Medium | Explains part of D2 (adversarial robustness) in NT-FEEL |
| D13 | No adaptive data-budget allocation | Monte Carlo Methods | Medium | Worsens D3 (streaming error) by processing irrelevant data |
| D14 | Independent particle populations | Monte Carlo Methods | **High** | Directly causes D4 (rank estimation blindness) |
| D15 | Bayesian-vs-ensemble epistemic confusion | Monte Carlo Methods | **High** | New quality-gate gap expanding D2 (adversarial robustness) |

---

## Critical Path Forward (Priority-Ordered)

1. **D6 + D3 + D8:** Implement DynamicPPL-style accumulator in NT-CORE with CVP-inspired calibration scaling, but with **online** calibration (not fixed-set) to avoid D8's shift blindness
2. **D10 + D1:** Replace external spectral preconditioning with optimizer-derived geometry from AdamW running statistics — zero additional compute cost
3. **D9 + D2:** Add finite-width correction terms to RND-based novelty detection, or use credal sets (D12) as a width-robust alternative
4. **D14 + D4:** Implement AIRPF-style inter-filter interaction with ENF monitoring for parallel rank estimation
5. **D15 + D11:** Add MH-quality-gate to SEAL pipeline inference acceptance, with E-value stopping rule for sampling budget

---

## Sources Cited

1. ONNX PPL Working Group — https://discourse.mc-stan.org/t/onnx-probabilistic-programming-working-group-announcement-call-for-contributors/40957
2. DynamicPPL.jl v0.41.7 — https://github.com/TuringLang/DynamicPPL.jl
3. TuringBnpBenchmarks — https://github.com/luiarthur/TuringBnpBenchmarks
4. CVP (Calibrated Variance Propagation) — https://arxiv.org/html/2606.16214v2
5. Bayesian RND Equivalence — https://proceedings.mlr.press/v337/zanger26a.html
6. Optimizer-Derived Geometry Preconditioning — https://pubdb.com/paper/2607.25312
7. E-Value Stopping for BDEs — https://arxiv.org/abs/2604.18089
8. UQ Survey (CreDE, VGMU, variance-gated) — https://arxiv.org/html/2607.28248
9. AS-SMCMC — https://doi.org/10.1155/dsn/6527524
10. AIRPF for PMCMC — https://arxiv.org/html/2312.06484
11. HTE (Nature Comms 2026) — https://link.springer.com/article/10.1038/s41467-026-70015-z
