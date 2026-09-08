# Iteration Batch 535 — Information Geometry & Riemannian Optimization

**Date:** 2026-09-06
**Previous:** Batch 534 (multi-agent game, GWT truthfulness, Stackelberg architecture, Arrow non-dictatorship)

---

## What's NEW vs Batch 534

### 1. Fisher Width — Complexity Measure for Statistical Manifolds (NEW)
**Source:** arXiv 2606.18306 (June 2026) — "Fisher Width: A Geometric Measure of Complexity on Statistical Manifolds"

**Finding:** Fisher width replaces Euclidean identity with local metric tensor G(θ)^{1/2} to measure Gaussian width of Fisher-rescaled sets. Captures anisotropic geometric effects invisible to Euclidean measures. Retains concentration, metric perturbation stability, and spectral comparison bounds.

**Defect in NeoTrix:** NeoTrix has no equivalent complexity measure for its 11-branch statistical manifold. The ConsciousnessTree tracks health but not *geometric complexity* of its own parameter space. Without Fisher width, NeoTrix cannot detect when its internal model complexity exceeds what the data supports — enabling silent overfitting of its own beliefs.

**Improvement over 534:** Batch 534 identified GWT truthfulness as lacking. Fisher width provides the *metric* — NeoTrix's GWT attention should use Fisher-rescaled routing, not Euclidean "broadcast all salient info." Attention cost should be proportional to statistical distinguishability, not raw salience.

---

### 2. Infinite-Dimensional Fisher-Rao Decomposition (NEW)
**Source:** Cheng & Tong, Entropy 28(4), 2026 — "An Approach to Fisher-Rao Metric for Infinite Dimensional Non-Parametric Information Geometry"

**Finding:** Orthogonal decomposition of tangent space TfM = S ⊕ S⊥ transforms intractable infinite-dimensional Fisher-Rao metric functional into finite computable Covariate Fisher Information Matrix G_f. Enables natural gradient computation in high-dimensional spaces without full metric inverse.

**Defect in NeoTrix:** NeoTrix's GWT operates in what is effectively an infinite-dimensional attention space (all modules broadcasting). The architecture has no orthogonal decomposition — it tries to compute the full attention metric, which is intractable. This explains the "truthfulness" problem: you cannot optimize attention without a finite, computable preconditioner.

**Improvement over 534:** Batch 534 diagnosed the symptom (GWT lacks truthful reporting). This provides the mathematical fix: decompose the attention manifold into a finite covariate subspace S where the metric is computable, and project attention updates onto this subspace. The "broadcast" should be an orthogonal projection, not a full-space operation.

---

### 3. Scalar Curvature as Bifurcation Detector (NEW)
**Source:** ScienceDirect, Physics Reports (2026) — "Geometric Bifurcation Theory: Fisher Information Geometry Applied to Dynamical and Complex Systems"

**Finding:** Scalar curvature of the Fisher information manifold detects bifurcations, local structural stability, and phase-space trajectory character. Curvature ≈ 0 = Euclidean regime; curvature >> 0 = phase transition imminent.

**Defect in NeoTrix:** NeoTrix's SEAL pipeline has no curvature-based phase transition detection. The 6-layer architecture makes topological changes (adding/removing branches) based on ad-hoc health thresholds, not on the intrinsic geometry of its own parameter space. This means architecture evolution is reactive, not anticipatory.

**Improvement over 534:** Batch 534 proposed Stackelberg leader-follower but gave no mechanism for *when* the leader should restructure. Scalar curvature of the NeoTrix statistical manifold provides the trigger: when |R| crosses a threshold, the architecture bifurcates. This is a principled alternative to the current "health score drops → trigger repair" approach.

---

### 4. Advective Fisher-Rao Metric — Flow-Based Architecture (NEW)
**Source:** arXiv 2608.12111 (Aug 2026) — "The Advective Fisher-Rao Geometry of Deterministic Measure Transport"

**Finding:** The advective Fisher-Rao metric on velocity fields yields Riemannian isometry with the continuity equation solution map. Gradient flow of least-squares energy follows mixture geodesic toward reference path, guaranteeing exponential convergence. Connects optimal transport (Wasserstein) with information geometry (Fisher-Rao) through a single metric structure.

**Defect in NeoTrix:** NeoTrix treats its 6-layer architecture as static layers with discrete interfaces. The advective Fisher-Rao framework shows that information flow between layers should be modeled as *measure transport* with a metric that connects Wasserstein geometry (optimal routing) with Fisher-Rao geometry (statistical distinguishability). NeoTrix's current "layer interface" design is metric-blind.

**Improvement over 534:** Batch 534 proposed Stackelberg leader-follower but the mechanism was game-theoretic voting. The advective Fisher-Rao metric provides a *geometric* mechanism: information flows along geodesics of a metric that jointly optimizes routing cost (Wasserstein) and statistical fidelity (Fisher-Rao). The Stackelberg leader should follow the advective Fisher-Rao geodesic, not a Nash equilibrium.

---

### 5. Arrow's Non-Dictatorship Violation — Resolution via Fisher-Rao Projection (NEW)
**Source:** MetricGate natural gradient documentation + Fisher-Rao gradient flow literature (2026)

**Finding:** The natural gradient ∇̃L = F(θ)^{-1} ∇L is the unique gradient that is invariant to reparameterizations of the model. It follows the steepest descent direction on the statistical manifold. The Fisher-Rao metric is the unique (Čencov, 1982) Riemannian metric invariant under sufficient statistic mappings.

**Defect in NeoTrix:** NT-CORE's current design acts as a "dictator" — it makes architecture decisions that all other branches must follow, violating Arrow's non-dictatorship. But Arrow's theorem applies to *ordinal* preference aggregation. The Fisher-Rao metric provides a *cardinal* structure: preferences can be aggregated via the Fisher information metric, not via voting. The dictator is necessary because there is no aggregation mechanism that preserves all desirable properties in ordinal space — but the Fisher-Rao metric provides a cardinal alternative where aggregation is well-defined.

**Improvement over 534:** Batch 534 identified NT-CORE as violating Arrow's non-dictatorship. The fix is NOT to remove the dictator (NT-CORE) but to replace ordinal preference aggregation with Fisher-Rao metric aggregation. NT-CORE's "dictatorship" becomes legitimate when it acts as the Fisher-Rao metric tensor — providing the unique invariant structure that makes multi-agent coordination possible.

---

### 6. Geodesically Incomplete Metrics — GWT as Incomplete Manifold (NEW)
**Source:** ICLR 2026 — "Riemannian Zeroth-Order Gradient Estimation with Structure"

**Finding:** When the underlying Riemannian metric g is geodesically incomplete, the goal is to approximate stationary points with respect to this incomplete metric. This requires specialized algorithms that handle the incompleteness explicitly.

**Defect in NeoTrix:** NeoTrix's GWT attention space is geodesically incomplete — not all attention paths are traversable (some module connections don't exist). The current GWT implementation ignores this incompleteness and treats the attention space as a complete manifold, leading to suboptimal routing that "jumps" across missing geodesics.

**Improvement over 534:** Batch 534 identified GWT truthfulness as lacking. The root cause is now clearer: GWT operates on an incomplete manifold. The fix is to explicitly model the incompleteness and use zeroth-order gradient estimation techniques designed for incomplete metrics, rather than assuming a complete manifold.

---

### 7. Parameter-Free Manifold Optimization (NEW)
**Source:** arXiv 2601.13877 (Jan 2026) — "Riemannian optimization on the manifold of unitary and symmetric matrices"

**Finding:** A new Riemannian optimization algorithm whose key feature is that it does not need to set any adaptation parameter. Achieves orders-of-magnitude speedup compared to methods requiring step-size tuning. Global convergence to stationary point guaranteed.

**Defect in NeoTrix:** NeoTrix's SEAL pipeline requires manual tuning of learning rates, attention thresholds, and evolution hyperparameters at each layer. The parameter-free manifold optimization result suggests that if NeoTrix's optimization is cast on the correct manifold (unitary + symmetric constraint structure), adaptation parameters become unnecessary.

**Improvement over 534:** Batch 534 proposed Stackelberg architecture but left optimization parameters unspecified. The parameter-free result shows that NeoTrix can achieve self-tuning if its optimization manifold has the right algebraic structure (unitary + symmetric). The current "tune per module" approach is suboptimal.

---

## Defects Found (Batch 535)

| # | Defect | Severity | Component |
|---|--------|----------|-----------|
| D535-1 | No Fisher width measure for 11-branch complexity | HIGH | ConsciousnessTree |
| D535-2 | GWT lacks orthogonal decomposition of attention manifold | CRITICAL | NT-CORE/GWT |
| D535-3 | SEAL pipeline has no curvature-based phase transition detection | HIGH | NT-MIND/SEAL |
| D535-4 | Layer interfaces ignore advective Fisher-Rao metric | MEDIUM | 6-layer architecture |
| D535-5 | NT-CORE dictatorship is a symptom of ordinal-vs-cardinal aggregation gap | HIGH | NT-CORE |
| D535-6 | GWT treats geodesically incomplete manifold as complete | HIGH | NT-CORE/GWT |
| D535-7 | SEAL requires manual parameter tuning; ignores parameter-free manifolds | MEDIUM | NT-MIND/SEAL |

---

## Sources Cited

1. arXiv 2606.18306 — "Fisher Width: A Geometric Measure of Complexity on Statistical Manifolds" (June 2026)
2. Cheng & Tong, Entropy 28(4), 2026 — "An Approach to Fisher-Rao Metric for Infinite Dimensional Non-Parametric Information Geometry"
3. ScienceDirect Physics Reports (2026) — "Geometric Bifurcation Theory: Fisher Information Geometry Applied to Dynamical and Complex Systems"
4. arXiv 2608.12111 — "The Advective Fisher-Rao Geometry of Deterministic Measure Transport" (Aug 2026)
5. ICLR 2026 — "Riemannian Zeroth-Order Gradient Estimation with Structure"
6. arXiv 2601.13877 — "Riemannian optimization on the manifold of unitary and symmetric matrices" (Jan 2026)
7. arXiv 2605.02279 — "Foundations of Riemannian Geometry for Riemannian Optimization" (May 2026)
8. arXiv 2609.00977 — "Information geometry of non-equilibrium quantum states" (Sep 2026)
9. MetricGate (2026) — "Natural-Gradient Optimization (Fisher Information)"
10. TheoremPath (Apr 2026) — "Information Geometry: Fisher Metric, Natural Gradient, and Statistical Manifolds"
11. ICLR 2026 Workshop — "Information-Geometric Optimal Control for Diffusion Models: Unified Framework via Fisher-Rao Geodesics"
12. OpenReview (Mar 2026) — "On the Fisher Geometry of Diffusion Models' Latent Space"
13. BioRxiv (Aug 2026) — "An Information Geometry approach to model topological..."
14. ScienceDirect (Sep 2026) — "Riemannian Geometry-Based Optimization for Deep Neural..."
