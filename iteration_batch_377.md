# Iteration Batch 377 — External Research Loop

**Date**: 2026-09-06
**Domains**: Probabilistic Programming, Uncertainty Quantification, Bayesian Optimization
**Sources Searched**: arxiv, ACM DL, Nature ML, Proceedings of ML Research, Springer

---

## 1. Probabilistic Programming — 2026 Advances

### Sources
1. **Score-based Variational Inference for BNNs** (arXiv:2602.05873) — Proximal stochastic-gradient score-based VI enabling large-scale BNNs (ViT, ResNet) without reparametrized sampling. Outperforms ADVI in ECE and NLL.
2. **PPDL: LLM-Based Flows as Probabilistic Programs** (ICML 2026, arXiv:2608.05234) — Probabilistic language for LLM-based flows; decouples inference scaling from core logic via `sample`/`factor` constructs. Supports IS, majority voting, SMC.
3. **Programmable VI in PPLs** (ACM TOPLAS 2024, genjax.vi) — Compositional program transformations for VI in PPLs. Supports user-defined variational objectives and combinatorial gradient estimation strategies.

### Defects Found

**DEFECT-PP-1: NeoTrix VSA HyperCube lacks probabilistic program semantics**
- **Location**: `nt_core_hcube` — VSA HyperCube representation
- **Gap**: NeoTrix's VSA HyperCube uses hard vector symbolic operations (bind, bundle, permute) with no probabilistic semantics. The 2026 PPDL work shows that LLM-based agent flows should treat `sample` and `factor` as first-class constructs, propagating uncertainty through entire multi-step tool chains. NeoTrix's tool dispatch (NT-ACT) returns point estimates with no uncertainty propagation across chained calls.
- **Impact**: When NeoTrix orchestrates multi-step MCP tool chains, uncertainty from each step is silently discarded, leading to accumulated overconfidence in downstream reasoning.
- **Suggestion**: Introduce a probabilistic programming layer into the NT-ACT tool dispatch: each tool call returns a distribution (not a point value), and downstream steps consume `factor`-weighted evidence. PPDL's `sample`/`factor` model is the proven pattern.

**DEFECT-PP-2: Score-based VI not exploited for NT-MIND self-evolution**
- **Location**: `nt_mind` — SEAL pipeline, distillation
- **Gap**: The score-based VI paper (arXiv:2602.05873) demonstrates that score-matching + proximal penalty enables scalable BNN posterior inference for Vision Transformers. NeoTrix's SEAL pipeline distills skills but does not maintain posterior distributions over skill parameters — it only keeps point estimates.
- **Impact**: Skill crystallization cannot express confidence intervals over its own parameters, making it impossible to know when a distilled skill is reliable vs. fragile.
- **Suggestion**: After SEAL distillation, maintain a diagonal Gaussian posterior over skill weights using the proximal score-based VI method. This gives NT-MIND "I know what I know" capability at near-zero extra training cost.

---

## 2. Uncertainty Quantification — 2026 Advances

### Sources
4. **Adaptive Cumulative Mass Calibration with Conformal Prediction** (UAI 2026, pmlr-v337-kazantsev26a) — Adaptive temperature scaling per-input via conformal coverage constraint. Improves CMCE, α-CMCE, ECE, MCE.
5. **CONFIDE: CP for Transformer LMs** (arXiv:2604.08885) — Conformal prediction on intermediate transformer layers (not just final [CLS]). Early layers yield better-calibrated representations.
6. **CVP: Calibrated Variance Propagation** (arXiv:2606.16214) — Single-pass uncertainty for transformers/CNNs via analytical variance propagation through normalization layers. Pareto-dominates mean network AND MC sampling on efficiency-reliability.
7. **Optimal CP under Epistemic Uncertainty** (UAI 2026, pmlr-v337-javanmardi26a) — Credal-set conformal prediction combining aleatoric + epistemic uncertainty. PAC-style conditional coverage guarantee.
8. **Conformal Prediction for Neural Operators** (arXiv:2608.28515) — Split conformal for PDE surrogates; normalized CP using MC Dropout for adaptive-width intervals.
9. **Flexible Uncertainty Calibration for MLIPs** (Nature ML, 2026) — Learnable environment-dependent quantile functions replacing global scalar in CP. 53% Spearman improvement over baseline.

### Defects Found

**DEFECT-UQ-1: ConsciousnessTree metacalib is a toy — no conformal coverage guarantee**
- **Location**: `nt_core_consciousness_tree::metacalib` — `expected_calibration_error` + `apply_metacognitive_calibration`
- **Gap**: NeoTrix's current calibration computes a simple ECE over 10 bins and applies a linear penalty. The 2026 adaptive cumulative mass calibration (Kazantsev et al.) shows that per-input adaptive temperature scaling with conformal coverage guarantees dramatically outperforms global bin-based ECE. NeoTrix's calibration is neither adaptive nor distribution-free.
- **Impact**: ConsciousnessTree branch health scores (`calibrated_health`) are systematically overconfident when the underlying SelfTest distribution is non-uniform. The 10-bin ECE is insufficient for high-dimensional health signals.
- **Suggestion**: Replace the bin-based ECE with adaptive cumulative mass calibration. Use conformal prediction to set per-branch coverage thresholds — this provides finite-sample distribution-free guarantees on calibration quality without requiring Gaussian assumptions.

**DEFECT-UQ-2: No single-pass uncertainty for transformer-based perception**
- **Location**: `nt_world_sense` / `nt_sense` — perception pipeline
- **Gap**: CVP (arXiv:2606.16214) achieves single-pass uncertainty for transformers by propagating variance analytically through LayerNorm and activation functions. NeoTrix's perception pipeline (NT-WORLD/NT-SENSE) processes transformer embeddings but returns point embeddings with no attached uncertainty.
- **Impact**: Downstream reasoning (GWT attention routing, E8 hexagram selection) operates on overconfident perception signals. The attention mechanism cannot distinguish "I see this clearly" from "I'm uncertain about this input."
- **Suggestion**: Implement CVP-style variance propagation through the perception transformer layers. Each sensory embedding should carry a variance vector. GWT attention should weight by `mean / sqrt(variance)` (signal-to-noise ratio), not raw activation magnitude.

**DEFECT-UQ-3: No conformal prediction layer for NT-ACT tool outputs**
- **Location**: `nt_act` — tool dispatch, MCP gateway
- **Gap**: Multiple 2026 papers demonstrate that conformal prediction provides distribution-free, model-agnostic coverage guarantees. NeoTrix's tool outputs (e.g., code execution results, search results, API responses) have no coverage guarantee — there's no calibration dataset, no nonconformity score, no prediction set.
- **Impact**: When NeoTrix uses a tool and presents results to the user, it cannot say "I'm 95% confident this result is within this range." This is a critical gap for high-stakes applications.
- **Suggestion**: Maintain a sliding-window calibration set of past tool outputs. For each new tool call, construct a conformal prediction interval using absolute error residual (AER) as the nonconformity score. This is near-zero computational cost (Gopakumar et al. 2026) and works for any model architecture.

**DEFECT-UQ-4: Epistemic vs. aleatoric decomposition missing from SelfTest**
- **Location**: `nt_core_self_test` — detection modules
- **Gap**: The 2026 CP for neural operators paper (arXiv:2606.09923) provides a clean uncertainty decomposition framework separating epistemic (reducible) from aleatoric (irreducible) uncertainty. NeoTrix's SelfTest framework reports pass/fail and health scores but does not decompose uncertainty into reducible vs. irreducible components.
- **Impact**: When a SelfTest fails, NeoTrix cannot determine whether the failure is due to insufficient data (epistemic, fixable) vs. inherent noise (aleatoric, irreducible). This leads to wasted repair cycles on irreducible failures.
- **Suggestion**: Add uncertainty decomposition to SelfTest evaluation: run each detection module with MC Dropout or ensemble, decompose into epistemic/aleatoric. Route epistemic failures to NT-REPAIR, leave aleatoric failures as known limitations.

---

## 3. Bayesian Optimization — 2026 Advances

### Sources
10. **Consequential Improvement (CI) Acquisition Function** (Springer, 2026) — MFBO with simultaneous solution+fidelity selection. Incorporates hidden validation cost of low-fidelity samples.
11. **Out-Of-The-Loop MF-BO** (arXiv:2608.04113) — Incorporating historical HF data with task descriptors into MF-BO when the HF function is too expensive for the optimization loop.
12. **Two-Stage MFBO with DA-LCB** (Applied Soft Computing, 2026) — Landscape-aware initialization + dual-adaptive LCB with cosine annealing and fidelity-aware spatial exploration.
13. **RESCUE: MF-BO with Causal Priors** (arXiv:2602.00788) — Causal structural model for multi-fidelity; causal hypervolume knowledge-gradient acquisition.
14. **Constrained MFBO with EMI/AECI/CUCB** (arXiv:2510.10984) — Closed-form acquisition functions for constrained MFBO without requiring feasible initial samples.

### Defects Found

**DEFECT-BO-1: BayesianExperimentDesign uses naive VoI — no multi-fidelity, no cost-awareness**
- **Location**: `nt_core_hcube::bayesian_experiment` — `BayesianExperimentDesign`, `VoIConfig`
- **Gap**: NeoTrix's Bayesian experiment design computes Value-of-Information (VoI) as expected KL divergence to select the next experiment. The 2026 CI acquisition function and RESCUE show that (a) low-fidelity proxies should be exploited with cost-awareness, (b) hidden validation costs must be modeled, and (c) causal structure between fidelity levels matters. NeoTrix's VoI ignores fidelity hierarchies entirely.
- **Impact**: When NeoTrix needs to evaluate a hypothesis (e.g., "does this module compile correctly?"), it always runs the most expensive verification. It cannot leverage cheap approximations (type checking, linting) before expensive ones (full test suite).
- **Suggestion**: Extend VoIConfig with a fidelity hierarchy: {lint: cost=1, type-check: cost=5, unit-test: cost=20, integration-test: cost=100}. Use the CI acquisition function to select both hypothesis AND fidelity level simultaneously. The "hidden cost of validation" concept prevents over-reliance on cheap-but-unreliable approximations.

**DEFECT-BO-2: No multi-fidelity optimization for SEAL skill evolution**
- **Location**: `nt_mind` — SEAL pipeline exploration phase
- **Gap**: The two-stage MFBO (TS-MFBO) framework shows that landscape-aware initialization from low-fidelity surrogates dramatically improves optimization on multimodal landscapes. NeoTrix's SEAL exploration phase generates candidate skills but has no concept of fidelity levels (quick prototype vs. full implementation).
- **Impact**: SEAL wastes full-computation cycles evaluating bad candidates that could have been filtered by cheap approximations.
- **Suggestion**: Define SEAL fidelity levels: F0 (parse + type-check, ~1s), F1 (unit test on synthetic data, ~10s), F2 (integration test on real data, ~60s). Use TS-MFBO's landscape-aware initialization to identify high-potential skill candidates at F0 before committing to F2 evaluation.

**DEFECT-BO-3: GWT attention routing is not Bayesian-optimal**
- **Location**: `nt_core` — GWT attention routing mechanism
- **Gap**: NeoTrix's GWT broadcasts salient information across specialist modules using resonance-based routing. The 2026 multi-objective MF-BO with causal priors (RESCUE) shows that routing decisions should balance multiple objectives (accuracy, cost, latency) with causal understanding of how routing choices affect downstream outcomes. NeoTrix's routing is heuristic-resonance, not acquisition-function-driven.
- **Impact**: GWT may over-route to cheap-but-low-information modules or under-route to expensive-but-high-information modules. Without an acquisition function, there's no principled way to decide "is this routing worth the cost?"
- **Suggestion**: Replace resonance-based routing with a multi-objective acquisition function: `routing_score = expected_information_gain / (latency_cost + compute_cost)`. Use the DA-LCB pattern with cosine annealing to transition from exploration (try new routing paths) to exploitation (stick with proven high-value routes) as the session progresses.

**DEFECT-BO-4: No constrained optimization for resource-limited deployment**
- **Location**: `nt_core_deploy` — edge deployment pipeline
- **Gap**: The constrained MFBO paper (arXiv:2510.10984) introduces EMI/AECI/CUCB acquisition functions that handle constraints without requiring feasible initial samples. NeoTrix's deployment pipeline quantizes models but does not optimize the quantization-under-constraints problem (e.g., "maximize accuracy subject to <2GB memory and <100ms latency").
- **Impact**: Deployment decisions are ad-hoc (fixed INT8 quantization) rather than optimized for the specific hardware constraints of each target device.
- **Suggestion**: Frame deployment as a constrained MFBO problem: fidelity levels = {full-precision FP32, FP16, INT8, INT4}, constraints = {memory budget, latency budget, accuracy floor}. Use CUCB acquisition to find the optimal quantization configuration per device. No feasible initial sample needed — the method handles infeasible starts gracefully.

---

## Summary

| Domain | Sources | Defects Found |
|--------|---------|---------------|
| Probabilistic Programming | 3 | 2 (PP-1, PP-2) |
| Uncertainty Quantification | 6 | 4 (UQ-1, UQ-2, UQ-3, UQ-4) |
| Bayesian Optimization | 5 | 4 (BO-1, BO-2, BO-3, BO-4) |
| **Total** | **14** | **10** |

### Top Priority Defects (Ranked by Impact)

1. **DEFECT-UQ-1** — ConsciousnessTree metacalib is a toy (foundational calibration is broken)
2. **DEFECT-BO-1** — BayesianExperimentDesign ignores multi-fidelity and cost (core reasoning is naive)
3. **DEFECT-UQ-2** — No single-pass uncertainty for perception (attention routing is blind to its own confidence)
4. **DEFECT-PP-1** — No probabilistic semantics in tool dispatch (uncertainty vanishes at tool boundaries)
5. **DEFECT-BO-3** — GWT routing is not Bayesian-optimal (attention allocation is heuristic)
6. **DEFECT-UQ-4** — No epistemic/aleatoric decomposition (SelfTest wastes repair cycles on irreducible failures)
7. **DEFECT-UQ-3** — No conformal prediction for tool outputs (no coverage guarantees)
8. **DEFECT-BO-2** — SEAL lacks multi-fidelity optimization (wastes computation on bad candidates)
9. **DEFECT-BO-4** — No constrained optimization for deployment (ad-hoc quantization)
10. **DEFECT-PP-2** — Score-based VI not used for skill parameter posteriors (skills are fragile)
