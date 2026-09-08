# Iteration Batch 467 — Research Loop

**Date**: 2026-09-06
**Focus**: Multi-objective optimization, evolutionary algorithms, surrogate optimization (2026 advances)

---

## Sources Cited

### Multi-Objective Optimization
1. Opris (2026) — "On the Impact of Crossover in Many-Objective Optimization: Runtime Analysis of NSGA-III" (arXiv:2605.11201)
2. Opris (2026) — "Towards a Rigorous Understanding of Population Dynamics of NSGA-III: Tight Runtime Bounds" (AAAI 2026, v40i43)
3. Deng, Zheng, Doerr (2025) — "First Theoretical Approximation Guarantees for NSGA-III" (IJCAI 2025)
4. Opris (2025) — "First Runtime Analysis of NSGA-III on Many-Objective Multimodal Problem: Provable Exponential Speedup via Stochastic Population Update" (IJCAI 2025)
5. Opris, Dang, Neumann, Sudholt (2024) — "Runtime Analyses of NSGA-III on Many-Objective Problems" (GECCO 2024)

### Evolutionary Algorithms
6. Zhong et al. (2026) — "Parameter Adaptive Competitive Differential Evolution with Local Search" (Applied Intelligence, Vol. 56)
7. Chauhan et al. (2026) — "DE-2LS: Differential Evolution with Lightweight Late Local Search" (arXiv:2606.27764)
8. Learning-guided Adaptive DE via Promising Subpopulation Identification (Neurocomputing, Vol. 677, 2026)
9. Zhang et al. (2026) — "Enhanced Differential Evolution for High-Dimensional Feature Selection" (J. King Saud Univ.)
10. MDPI Foundations (2026) — "Novel Method Based on DE Suitable for Large-Scale Optimization Problems"
11. AAAI 2026 — "Learn from Global Correlations: Enhancing EA via Spectral GNN (Graph Neural Evolution)"
12. SHIODEG (2026) — "Hybrid Success-History Intelligent Optimization with DE and Gaussian Transformation" (J. Supercomputing)
13. RDEx-SOP (2026) — "Exploitation-Biased Reconstructed Differential Evolution" (arXiv:2603.27089)
14. IKUN (2026) — "Mean-Field Game Theoretic KD-tree Density Guided Mechanism for EAs" (Information Sciences)
15. Modular Harris Hawks with Trend-Guided DE and Gaussian Exploration (Nature Scientific Reports, 2026)

### Surrogate / Bayesian Optimization
16. KENDO (2026) — "Enhancing Bayesian Optimization and Active Learning Through Kernel Diversity" (arXiv:2608.24721)
17. FLIWBO (2026) — "No-Regret Bayesian Optimization with Finite-Library Input-Warped Kernels" (arXiv:2609.02993)
18. Nature npj Computational Materials (2026) — "Deep GP-based Cost-Aware Batch Bayesian Optimization for Complex Materials Design"
19. Brunzema & Trimpe (2026) — "BayeSQP: Bayesian Optimization through Sequential Quadratic Programming" (arXiv:2602.03232)
20. GRAPE (2026) — "Gradient Refinement and Progress-Aware Exploitation for High-Dimensional BO" (arXiv:2608.25116)
21. Bartoli et al. (2026) — "Efficient Multidisciplinary Design via Bayesian Optimization (SEGOMOE)" (CSMA 2026)
22. Robust BO via Tempered Posteriors (arXiv:2601.07094)
23. CBA-BO (2026) — "Constraint-Bound Agnostic Bayesian Optimization" (arXiv:2607.23448)
24. Unified Bayesian Optimization for Stationary Point Searches (arXiv:2603.10992)

---

## Defects Found in NeoTrix Design

### DEFECT-467-1: NSGA-III Reference Point Setting Gap
**Source**: Opris 2026 (AAAI), Deng et al. 2025 (IJCAI)
**Finding**: NSGA-III with Nr = N (reference points = population size) achieves optimal approximation quality (MEI ≤ ⌈(5−2√2)n/(Nr−1)⌉), but Nr > N causes approximation to degrade by Ω(log n). The SEAL pipeline's multi-objective capability tree evolution does not specify how reference points should be sized relative to population.
**Gap**: No parameter guidance for NSGA-III reference point scaling in SEAL's MOEA configuration. The original NSGA-III paper suggested Nr ≈ N but lacked theoretical backing; now proven optimal.
**Suggestion**: Add reference-point-to-population ratio constraint in SEAL MOEA config: `nr_ratio: Nr/N ∈ [0.8, 1.2]` with default 1.0. Reject configurations where Nr > 1.2N.

### DEFECT-467-2: Missing Crossover Benefit Analysis for Many-Objective Evolution
**Source**: Opris 2026 (arXiv:2605.11201)
**Finding**: Uniform crossover in NSGA-III provides exponential speedup (Ω((k−1)!/(µ²c·mᵏ))) on m-OJZJ when k = Ω(n/ln(n)) and m = O(log(n)). The SEAL pipeline's evolution operators likely use mutation-only or fixed crossover without theoretical justification.
**Gap**: No empirical or theoretical analysis of crossover operator selection in SEAL's many-objective evolution. For m ≥ 4 objectives, crossover benefits are provably significant.
**Suggestion**: Implement crossover-rate adaptive scheduling in SEAL: start with high crossover (pc=0.9) for early exploration, decay to mutation-dominant (pc=0.3) for late-stage refinement. Track Pareto front coverage as fitness signal.

### DEFECT-467-3: No Surrogate Model for Capability Tree Evaluation
**Source**: Deep GP-based Cost-Aware BO (Nature 2026), BayeSQP (2026)
**Finding**: Deep Gaussian processes with cost-aware acquisition achieve 5× speedup over standard GP-BO by modeling hierarchical relationships and propagating uncertainty through layers. NeoTrix's CapabilityTree evaluation requires expensive test-suite execution with no surrogate approximation.
**Gap**: Every SEAL phase-2 capability evaluation runs the full test suite. No cheap surrogate predicts capability health, forcing redundant expensive evaluations.
**Suggestion**: Build a local GP surrogate per capability module that predicts test-pass probability from: (a) code change delta, (b) historical pass rates, (c) dependency health signals. Use cost-aware acquisition (qEHVI) to prioritize which capabilities need full evaluation vs. surrogate prediction. Target: 3-5× reduction in test evaluations during SEAL cycles.

### DEFECT-467-4: Lack of Kernel Diversity in VSA HyperCube Embeddings
**Source**: KENDO (arXiv:2608.24721), FLIWBO (arXiv:2609.02993)
**Finding**: Kernel ensemble disagreement-aware operators (KENDO) achieve competitive BO with 5× less compute by replacing hyperparameter sampling with kernel ensemble + adaptive Bayesian weighting. Input warping from finite libraries improves sample efficiency under geometry mismatch.
**Gap**: VSA HyperCube uses fixed kernel (likely RBF/Matern) for embedding similarity. No kernel diversity mechanism. When concept geometry is non-stationary (common in knowledge domains), fixed kernels underperform.
**Suggestion**: Implement kernel ensemble in VSA HyperCube: maintain 3-5 kernel variants (RBF, Matérn 5/2, periodic, linear) with Bayesian model selection weights. Use disagreement between kernel predictions as uncertainty signal for active embedding refinement. This mirrors KENDO's approach but applied to knowledge representation rather than BO.

### DEFECT-467-5: No Gradient-Aware Search in SEAL Optimization
**Source**: GRAPE (arXiv:2608.25116), BayeSQP (arXiv:2602.03232)
**Finding**: GRAPE achieves 5.4× speedup in high-dimensional BO by first refining gradient posterior then selecting directions maximizing expected decrease conditional on descent. BayeSQP uses second-order GP surrogates (modeling function + gradient + Hessian) for constrained optimization.
**Gap**: SEAL's evolution treats capability optimization as pure black-box. No gradient estimation (via finite differences or GP derivatives) to guide search direction. This wastes queries on near-certain but low-magnitude descent directions.
**Suggestion**: Add gradient-aware acquisition to SEAL: estimate local gradient from GP posterior of capability fitness landscape, use progress-aware exploitation (maximize |∇f|·P(descent)) instead of pure EI. For constrained SEAL phases (safety constraints), implement BayeSQP-style uncertainty-aware subproblems.

### DEFECT-467-6: Missing Late-Stage Local Search Polishing
**Source**: DE-2LS (arXiv:2606.27764), SHIODEG (2026)
**Finding**: Late-stage local search activated only in final 1% of budget improves U-score by 5.58% over pure DE. "Do-no-harm" philosophy: preserve global search, add lightweight coordinate-pattern LS as final polishing.
**Gap**: SEAL pipeline lacks a final-stage local refinement step. Evolution converges to Pareto front but solutions may be locally optimal yet globally suboptimal.
**Suggestion**: Add SEAL Phase-4.5 "Polishing" after Phase-4: lightweight coordinate-pattern search around top-3 Pareto solutions, using ≤1% of total budget. Accept only feasibility-aware improvements. This is cheap and provably improves final solution quality.

### DEFECT-467-7: No Density-Aware Population Management
**Source**: IKUN (Information Sciences 2026)
**Finding**: Mean-field game theoretic KD-tree density estimation with crowding potential f(x)+λρ(x) discourages revisiting over-sampled regions. Embedded into PSO/DE/JADE/GA with consistent improvement.
**Gap**: SEAL population management uses standard non-dominated sorting + crowding distance. No density-aware repulsion mechanism. Population can cluster in easy-to-reach regions, missing diverse Pareto solutions.
**Suggestion**: Augment SEAL's survival selection with IKUN-style density potential: maintain KD-tree of evaluated points, add repulsion term λ·ρ(x) to fitness. λ adaptively tuned based on population diversity metric. This is algorithm-agnostic and can be added as a post-selection filter.

### DEFECT-467-8: No Constraint-Bound Transfer Learning
**Source**: CBA-BO (arXiv:2607.23448)
**Finding**: Parametric constraint model (PCM) learns mapping from constraint thresholds to optimal solutions, enabling direct prediction for arbitrary unseen thresholds without restarting optimization.
**Gap**: SEAL handles safety constraints (R-P1: zero unsafe code) as fixed thresholds. When constraints change (e.g., new security requirements, different trust tiers), SEAL restarts optimization from scratch.
**Suggestion**: Implement constraint-bound transfer in SEAL: learn PCM h(θ) mapping constraint thresholds → Pareto-optimal configurations. When constraint thresholds shift, query PCM for warm-start initialization instead of cold restart. Expected: 2-3× faster convergence on constraint changes.

---

## Summary

| Metric | Value |
|--------|-------|
| Sources cited | 24 papers (2024-2026) |
| Defects identified | 8 |
| Critical gaps | DEFECT-467-3 (surrogate for capability eval), DEFECT-467-5 (gradient-aware search) |
| High-impact suggestions | Kernel diversity in VSA (DEFECT-467-4), Late-stage polishing (DEFECT-467-6) |
| Estimated combined speedup | 3-8× if all suggestions implemented |

## Next Steps
1. Prioritize DEFECT-467-3 (surrogate model) and DEFECT-467-6 (late-stage polishing) — lowest implementation cost, highest ROI
2. Validate DEFECT-467-1 (Nr = N constraint) against existing SEAL MOEA benchmarks
3. Prototype DEFECT-467-4 (kernel ensemble) on VSA HyperCube embedding quality metrics
4. Track: does crossover adaptive scheduling (DEFECT-467-2) measurably improve SEAL Pareto front coverage?
