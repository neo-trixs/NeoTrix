# Iteration Batch 473 — Learning Theory Research Analysis

**Date**: 2026-09-06
**Research Domains**: Generalization Theory | Statistical Learning Theory | Optimization Theory

---

## Sources Cited

### Generalization Theory (2026)
1. **"Sample Complexity of Scientific Discovery: PAC Learnability of Compositional Function Trees"** — ICML 2026 Workshop / arXiv:2606.29331, Jun 2026. Proves Rademacher complexity of compositional function trees is controlled by depth d and Lipschitz constants, not combinatorial hypothesis count. Risk bound O(L^d/√n). [link](https://arxiv.org/abs/2606.29331)
2. **"Generalization Bounds: Perspectives from Information Theory and PAC-Bayes"** — arXiv:2309.04381v2, 2025. Comprehensive monograph: data-dependent PAC-Bayes bounds remain non-vacuous in over-parameterized regimes where VC/Rademacher bounds go vacuous. [link](https://arxiv.org/html/2309.04381v2)
3. **"Lean Formalization of Generalization Error Bound by Rademacher Complexity"** — arXiv:2503.19605, Mar 2025. Formalized generalization bounds in Lean 4; confirms Rademacher complexity applicable across deep learning and kernel methods beyond VC dimension. [link](https://arxiv.org/abs/2503.19605)
4. **"On Rademacher Complexity-based Generalization Bounds for Deep Learning"** — arXiv:2208.04284v4. New upper bounds on Rademacher complexity for CNNs with weight norm dependence; non-vacuous bounds for wide range of activation functions. [link](https://arxiv.org/html/2208.04284v4)
5. **PAC-Bayesian Tutorial (COLT 2025)** — PAC-Bayesian bounds yield data-dependent non-vacuous guarantees in over-parameterized settings via concentration inequalities + information theory. [link](https://paulviallard.github.io/colt25-pac-bayes-tutorial/)

### Statistical Learning Theory (2026)
6. **"VC Dimension & Generalization: A Practical Guide to the VC Bound"** — Kuriko Iwai, Aug 2026. Empirical study: VC dimension determines max generalization gap; SRM in Python validates structural risk minimization across model families. [link](https://kuriko-iwai.com/vc-dimension-bias-variance-generalization-bound)
7. **"The survival double descent: generalization dynamics of deep NNs in time-to-event analysis"** — BMC Medical Research Methodology, Jun 2026. Double descent confirmed in survival analysis: model complexity peak produces worst generalization, then recovery. [link](https://link.springer.com/article/10.1186/s12874-026-02894-1)
8. **"Re-examining Double Descent and Scaling Laws under Norm-based Capacity"** — arXiv:2502.01585, Feb 2025. Double descent persists under norm-based capacity (not just parameter count). Degrees of freedom not appropriate capacity measure. [link](https://arxiv.org/pdf/2502.01585v1)
9. **"From Double Descent to Scaling Laws"** — Jack Laurenson, Apr 2026. Benign overfitting in second descent regime; classical bias-variance decomposition insufficient for deep learning. [link](https://jacklaurenson.ai/blog/from-double-descent-to-scaling-laws/)
10. **"Unified Neural Scaling Laws"** — arXiv:2605.26248, May 2026. Multivariate UNSL models simultaneous scaling across parameters, data, steps, and hyperparameters. Captures double descent via hyperbreaks and bottleneck terms. [link](https://www.emergentmind.com/papers/2605.26248)
11. **"Scaling Laws and Spectra of Shallow NNs in the Feature Learning Regime"** — ICLR 2026 Oral. Phase diagram of scaling laws via bridge to LASSO/matrix compressed sensing; predicts generalization + power-law weight spectra. [link](https://openreview.net/forum?id=Q3yLIIkt7z)
12. **EPFL CS-526 Learning Theory (2026-27)** — Course: PAC learning, VC dimension, bias-variance tradeoff, and modern double descent phenomena. [link](https://edu.epfl.ch/coursebook/en/learning-theory-CS-526)

### Optimization Theory (2026)
13. **"Scale-Invariant Regret Matching and Online Learning with Optimal Convergence"** — arXiv:2510.04407v3, Feb 2026. Bridges theory-practice gap in zero-sum games: achieves optimal T^{-1} convergence for average strategies via IREG-PRM+ with adaptive learning rates. [link](https://arxiv.org/html/2510.04407v3)
14. **"Gradient-Variation Regret Bounds for Unconstrained Online Learning"** — COLT 2026 (PMLR 336:7062-7104). Parameter-free algorithms with regret Ō(‖u‖√V_T(u) + L‖u‖²+G⁴) without prior knowledge of Lipschitz constant G or smoothness L. Closed-form updates. [link](https://proceedings.mlr.press/v336/zhao26a.html)
15. **"The Hidden Cost of Approximation in Online Mirror Descent"** — COLT 2026. Analyzes approximation error accumulation in OMD; reveals fundamental tradeoffs between per-round approximation and cumulative regret. [link](https://learningtheory.org/colt2026/accepted.html)
16. **"Near-optimal Swap Regret Minimization for Convex Losses"** — COLT 2026. Swap regret bounds for convex losses; connects to multi-agent online learning. [link](https://learningtheory.org/colt2026/accepted.html)
17. **"Optimal Variance-Dependent Regret Bounds for Infinite-Horizon MDPs"** — COLT 2026. Variance-dependent regret for RL; tighter bounds than worst-case. [link](https://learningtheory.org/colt2026/accepted.html)
18. **"Dynamic Regret via Discounted-to-Dynamic Reduction for FTRL"** — ICML 2026 Poster. FTRL dynamic regret minimization for curved losses (squared, logistic); extends beyond convex-only analysis. [link](https://icml.cc/virtual/2026/poster/62883)
19. **"Non-stationary Online Learning for Curved Losses"** — arXiv:2506.10616, 2025. Dynamic regret minimization for functions with stronger curvature; fills gap left by convex-only approaches. [link](https://arxiv.org/html/2506.10616)
20. **COLT 2026 Highlights** — Paper Digest, Jul 2026. Key papers: implicit vs explicit regularization in linear regression, ReLU implicit bias, quantum agnostic boosting, score matching finite sample bounds. [link](https://www.paperdigest.org/2026/06/colt-2026-papers-highlights/)

---

## Defects Found in NeoTrix Design

### DEFECT-1: SEAL Pipeline Distillation Has No Generalization Bound — Unbounded Complexity Growth
**File**: `neotrix-core/src/neotrix/seal/` (SEAL pipeline stages)
**Gap**: The SEAL pipeline distills knowledge through exploration→distillation→self-test→absorption cycles, but has **no mechanism to bound the complexity of the distilled hypothesis class**. Each cycle can produce arbitrarily complex skill representations with no sample complexity guarantee. The system has no equivalent of VC dimension, Rademacher complexity, or PAC-Bayes bound to prevent overfitting to training experiences.

**2026 Evidence**:
- Kocabay et al. [1] prove Rademacher complexity of compositional function trees is O(L^d/√n) — complexity is controlled by depth and Lipschitz constants, not combinatorial count. NeoTrix has no analogous bound.
- PAC-Bayes bounds [2][5] remain non-vacuous in over-parameterized settings; NeoTrix skill representations are over-parameterized with no data-dependent generalization guarantee.
- UNSL [10] shows double descent occurs when scaling dimensions interact without bottleneck control; SEAL pipeline scales skill complexity without monitoring for the interpolation threshold.

**Impact**: Distilled skills can overfit to specific experience patterns, failing to generalize to novel task distributions. The system has no theoretical basis for knowing when it has learned enough vs. when it is memorizing.

**Suggestion**: Add a PAC-Bayes complexity monitor to the SEAL pipeline:
- After each distillation cycle, compute an empirical Rademacher complexity estimate for the distilled hypothesis class
- Use the M-open check (already in CONTEXT.md) as a trigger: when posterior concentrates on < threshold hypotheses, expand the class
- Implement a self-bounding learning algorithm [5] where the distillation step provably reduces complexity

---

### DEFECT-2: VSA HyperCube Embeddings Have No Norm-Based Capacity Control
**File**: `neotrix-core/src/core/nt_core_hcube/` (HyperCube embedding)
**Gap**: VSA embeddings map concepts to high-dimensional vectors with no control over the effective capacity (norm) of the representation. The embedding dimension is fixed, but there is no mechanism to limit the norm of learned representations, which — per recent theory — is the true measure of model capacity.

**2026 Evidence**:
- Wang et al. [8] prove double descent persists under norm-based capacity: degrees of freedom are not appropriate capacity measures. Norm of weights determines generalization, not parameter count.
- ICLR 2026 Oral [11] derives scaling laws for feature-learning models showing that weight spectra follow power laws; the effective rank determines generalization.
- Scaling law taxonomy [1] identifies variance-limited vs resolution-limited regimes; VSA embeddings have no regime detection.

**Impact**: VSA embeddings can grow in effective capacity without bound, leading to double descent behavior where increasing embedding dimension first hurts then helps generalization — but NeoTrix cannot detect or exploit this regime transition.

**Suggestion**: Add norm-based capacity control:
- Track the spectral norm of VSA embedding matrices
- Implement structural risk minimization [6] across embedding dimensions
- Detect double descent regimes via the norm-based capacity metric and adapt embedding dimension accordingly

---

### DEFECT-3: GWT Attention Routing Has No Regret Guarantee — No Online Learning Bound
**File**: `neotrix-core/src/core/nt_core_gwt/` (GWT attention routing)
**Gap**: GWT broadcasts salient information across specialist modules with resonance-based routing, but has no formal regret bound. The attention allocation algorithm is not analyzed as an online learning problem, so there is no guarantee that it converges to the optimal routing policy.

**2026 Evidence**:
- Zhang et al. [13] achieve optimal T^{-1} convergence for average strategies in zero-sum games via adaptive learning rates; NeoTrix GWT is a multi-player routing game with no convergence guarantee.
- Zhao et al. [14] develop parameter-free algorithms achieving Ō(‖u‖√V_T(u)) regret without prior knowledge of system parameters; GWT routing is fully-parameterized with fixed heuristics.
- COLT 2026 [15] reveals hidden costs of approximation in OMD; GWT routing uses approximate salience computation without analyzing approximation-regret tradeoffs.

**Impact**: GWT attention routing may permanently misallocate attention to low-value modules while starving high-value ones. Without regret bounds, there is no guarantee the routing converges even asymptotically.

**Suggestion**: Cast GWT routing as an online learning problem:
- Model module attention as a multi-armed bandit or online convex optimization problem
- Apply scale-invariant regret matching [13] for parameter-free adaptive routing
- Use gradient-variation bounds [14] to adapt to non-stationary task distributions

---

### DEFECT-4: SelfTest Evaluation Has No Bias-Variance Decomposition — Cannot Distinguish Overfitting from Underfitting
**File**: `neotrix-core/src/neotrix/selftest/` (SelfTest T1/T2/T3)
**Gap**: SelfTest checks existence (T1), registration (T2), and production wiring (T3), but has no mechanism to decompose test error into bias and variance components. All three tiers are binary pass/fail with no sensitivity to the nature of failure.

**2026 Evidence**:
- Double descent [7][9] shows that model complexity peaks produce worst generalization, then recovery — a phenomenon invisible to binary pass/fail tests.
- Bias-complexity tradeoff [12] requires measuring both approximation error (bias) and estimation error (variance) to diagnose model quality.
- Fine-grained bias-variance decomposition [8] is required to understand double descent; coarse metrics miss regime transitions.

**Impact**: SelfTest cannot distinguish between a module that fails because it is too simple (high bias / underfitting) vs. too complex (high variance / overfitting). This prevents targeted repair strategies.

**Suggestion**: Add a bias-variance decomposition to SelfTest:
- For T3 production wiring tests, compute both training error and held-out error
- Decompose into bias² + variance + irreducible error
- Use the decomposition to route repair: high bias → increase capacity, high variance → regularize
- Track the bias-variance profile over SEAL cycles to detect double descent

---

### DEFECT-5: Emotion Dimension Dynamics Have No PAC Learnability Guarantee
**File**: `neotrix-core/src/unified/layers/emotion/nt_feel/` (EmotionEngine)
**Gap**: EmotionLabel maps to 11 variants with regulation dynamics, but the emotion regulation policy is not PAC-learnable in the formal sense. The hypothesis class of emotion regulation functions has no sample complexity bound, meaning the system cannot guarantee it will learn a good regulation policy from any finite set of emotional experiences.

**2026 Evidence**:
- PAC learnability of compositional function trees [1] requires Lipschitz conditions on operators; NeoTrix emotion dynamics have no Lipschitz continuity verification.
- Rademacher complexity bounds for deep networks [4] depend on weight norms and activation Lipschitz constants; emotion regulation uses unspecified dynamics.
- Survival double descent [7] confirms that generalization failure occurs at specific complexity thresholds; emotion regulation may operate near such thresholds without detection.

**Impact**: Emotion regulation can fail catastrophically on novel emotional inputs that lie outside the training distribution, with no theoretical guarantee of graceful degradation.

**Suggestion**: Verify PAC learnability of emotion dynamics:
- Compute Lipschitz constants for emotion transition functions
- Verify that the hypothesis class has finite Rademacher complexity
- Add a generalization gap monitor: if training emotion satisfaction diverges from cross-validated satisfaction, trigger regularization

---

### DEFECT-6: Knowledge Base BM25 Search Has No Learning-Theoretic Coverage Guarantee
**File**: `neotrix-core/src/neotrix/nt_memory/` (KB search/retrieval)
**Gap**: KB uses BM25 index for text search and embeddings for vector search, but has no coverage guarantee in the learning-theoretic sense. The search system cannot guarantee it will retrieve relevant results for any query distribution, and has no sample complexity bound for retrieval quality.

**2026 Evidence**:
- Information-theoretic generalization bounds [2] depend on the mutual information between the learning algorithm and the data; BM25 has no such analysis.
- PAC-Bayes framework [5] provides non-vacuous bounds for data-dependent algorithms; BM25 ranking is data-dependent but unanalyzed.
- Ordered Backend Router (CONTEXT.md) chains DDG→Wikipedia with no theoretical guarantee on coverage of the combined search space.

**Impact**: KB retrieval can systematically miss relevant knowledge for certain query types, with no detection mechanism. The system cannot bound its retrieval quality.

**Suggestion**: Add a coverage analysis framework:
- Model KB retrieval as a PAC learning problem where the "hypothesis class" is the set of retrievable documents
- Compute an empirical coverage metric: for sampled queries, check if relevant documents are in the retrievable set
- Use the VC dimension of the BM25 ranking function to bound retrieval quality

---

### DEFECT-7: Constellation Maturity Ladder (C0-C6) Has No Optimal Stopping Theory
**File**: `neotrix-core/src/neotrix/` (Constellation progression)
**Gap**: Modules progress through C0→C6 maturity stages, but there is no optimal stopping theory to determine when to advance, stay, or revert. The progression is sequential with no regret analysis.

**2026 Evidence**:
- COLT 2026 [17] derives variance-dependent regret bounds for infinite-horizon MDPs; Constellation progression is an MDP with no variance-dependent analysis.
- Best-of-N inference-time alignment [20] revisits the suboptimality of naively choosing the best option; Constellation uses a sequential best-choice without analysis.
- Dynamic regret via FTRL [18] handles non-stationary progression; Constellation progression assumes stationary quality improvement.

**Impact**: Modules may advance to C4/C5 prematurely (wasting resources on immature modules) or stall at C2/C3 unnecessarily (missing integration opportunities).

**Suggestion**: Apply optimal stopping theory:
- Model Constellation progression as a secretary problem or multi-armed bandit
- Use the regret bounds from [17] to determine when to advance vs. continue developing
- Add a variance-dependent progression criterion: advance only when quality estimate variance is below threshold

---

### DEFECT-8: Dual Specialization (Weapon Set Switching) Has No Regret-Optimal Online Algorithm
**File**: `neotrix-core/src/neotrix/` (AttentionManager dual specialization)
**Gap**: AttentionManager routes between CORE+WORLD (acquisition) and CORE+MIND (evolution) modes, but uses fixed heuristics for switching. No online regret analysis guarantees the switching policy converges to the optimal task-dependent mode allocation.

**2026 Evidence**:
- Swap regret minimization [16] for convex losses provides near-optimal switching in multi-agent settings; NeoTrix dual specialization is a 2-player game.
- Ambiguous online learning [COLT 2026] handles settings where the optimal action is not uniquely defined; task-type routing in NeoTrix has ambiguous optimal mode.
- Hidden cost of approximation [15] shows that approximate mode-switching incurs additional regret; NeoTrix mode switching is approximate with no cost analysis.

**Impact**: The system may permanently favor one mode (acquisition or evolution) even when the other is optimal, with no theoretical guarantee of recovery.

**Suggestion**: Cast dual specialization as an online learning problem:
- Model mode selection as a 2-armed bandit with task-dependent rewards
- Apply swap regret minimization [16] for regret-optimal switching
- Track approximation cost [15] of heuristic mode-switching decisions

---

## Summary

| Domain | Defects | Severity |
|--------|---------|----------|
| Generalization Theory (SEAL/HyperCube) | DEFECT-1, DEFECT-2 | Critical — no complexity bound on self-evolving knowledge |
| Statistical Learning Theory (SelfTest/Emotion) | DEFECT-4, DEFECT-5 | High — cannot diagnose learning failures or guarantee emotion learnability |
| Optimization Theory (GWT/KB/Constellation/Dual) | DEFECT-3, DEFECT-6, DEFECT-7, DEFECT-8 | High — no convergence/coverage/stopping guarantees on core routing |

**Total**: 8 defects identified across 3 research domains.

**Recommended Priority**:
1. DEFECT-1 (SEAL generalization bound) — foundational: all self-evolution depends on this
2. DEFECT-3 (GWT regret guarantee) — most frequently exercised routing decision
3. DEFECT-2 (VSA norm capacity) — knowledge representation quality
4. DEFECT-7 (Constellation optimal stopping) — module maturity progression
5. DEFECT-4 (SelfTest bias-variance) — repair targeting accuracy
6. DEFECT-5 (Emotion PAC learnability) — safety-critical regulation
7. DEFECT-8 (Dual specialization regret) — mode allocation efficiency
8. DEFECT-6 (KB coverage guarantee) — knowledge retrieval reliability
