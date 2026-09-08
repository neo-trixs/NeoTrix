# Iteration Batch 530 — Evolutionary/Optimization/Search Gap Analysis

**Date**: 2026-09-06
**Predecessor**: Batch 529 (grounding gap, no ontology governance, SQLite≠graph DB, no GraphRAG, temporal blindness)

---

## 1. EVOLUTIONARY ALGORITHMS — NEW DEFECTS vs Batch 529

### Finding 1: Surrogate-Assisted Multi-Objective EA (Nishihara et al., IEEE Trans Cybernetics 2026)
- **Source**: https://arxiv.org/list/cs.NE/2026-08 (Pareto-optimal surrogate models paper)
- **What's NEW**: Ensemble of Pareto-optimal surrogate models guiding EA search — not single-surrogate but multi-model ensemble that selects best surrogate per objective region.
- **NEOTRIX DEFECT 6.1**: NeoTrix SEAL pipeline uses single fitness landscape for evolution. No surrogate model ensemble. When capability tree evolves, evaluation is single-metric (compilation/test pass). **Missing: multi-objective surrogate that models fitness landscape per domain independently**, selecting best model per problem region.
- **ACTION**: Add `SurrogateEnsemble` to `nt_mind::seal` — register N surrogate models per objective, use cross-validation to route queries to best-fit surrogate per objective region.

### Finding 2: NSGA-III + TOPSIS for Multi-Objective Engineering (Zhang et al., Eng App AI 2026)
- **Source**: https://www.sciencedirect.com/science/article/pii/S0952197626008328
- **What's NEW**: Non-dominated sorting GA III combined with TOPSIS decision-making for concrete mix optimization — uses SHAP explainability to interpret which features dominate which objective regions.
- **NEOTRIX DEFECT 6.2**: NeoTrix has no explainability integrated into evolution. SEAL pipeline evolves modules but cannot explain WHY a particular capability was selected or rejected. **Missing: SHAP-integrated feature importance for evolutionary fitness components** — operators cannot see which aspects of module design contribute to compilation success vs test coverage.
- **ACTION**: Add feature attribution to SEAL fitness evaluation — each dimension (compile time, test coverage, dependency count) gets SHAP values showing contribution to overall fitness.

### Finding 3: AAAI 2026 Tutorial — Neuroevolution for Deep Learning Architectures (Miikkulainen, UT Austin)
- **Source**: https://www.cs.utexas.edu/~risto/talks/aaai26-tutorial/
- **What's NEW**: Neuroevolution extended beyond weight optimization to **architecture search** (neural architecture search via evolution) AND **model merging** (quality-diversity for combining trained models).
- **NEOTRIX DEFECT 6.3**: NeoTrix capability tree is static topology — nodes are pre-defined, not evolved. Evolution only tunes parameters, not topology. **Missing: topology-evolving capability tree** where the tree structure itself (which capabilities connect to which) is subject to evolutionary mutation and selection.
- **ACTION**: Add `TopologyEvolver` to `nt_core::capability_tree` — mutate node connections, prune dead branches, add novel edges. Evaluate topology fitness via reachability + redundancy metrics.

### Finding 4: GA for Vehicle Routing (Wang, CIBDA 2026, ACM)
- **Source**: https://dl.acm.org/doi/full/10.1145/3813822.3813901
- **What's NEW**: Applied GA to combinatorial logistics — demonstrates that real-world NP-hard problems still benefit from GA even in 2026.
- **NEOTRIX DEFECT 6.4**: NeoTrix task scheduling (NT-ACT) uses greedy heuristics for MCP tool invocation ordering. No evolutionary exploration of tool-call sequences. **Missing: evolutionary tool-call sequence optimization** — evolve the ordering of tool invocations across parallel tasks to minimize latency.
- **ACTION**: Add evolutionary layer to `nt_act::orchestrator` — population of tool-call orderings, evaluate latency, select/mutate for Pareto-optimal sequences.

---

## 2. OPTIMIZATION ALGORITHMS — NEW DEFECTS vs Batch 529

### Finding 5: Gradient-Free Optimization for Matrix Functions (Allen et al., arXiv 2609.03170, Sep 2026)
- **Source**: https://arxiv.org/abs/2609.03170
- **What's NEW**: Gradient-free methods specifically for **matrix-valued functions** — non-smooth, non-differentiable objectives where traditional optimization fails.
- **NEOTRIX DEFECT 6.5**: NeoTrix KB embeddings (VSA HyperCube) are high-dimensional vectors, but optimization of embedding quality (relevance, coherence) uses no gradient-free matrix optimization. **Missing: gradient-free embedding quality optimization** — treat embedding space as matrix function and optimize without requiring differentiability.
- **ACTION**: Implement matrix-aware BOHB (Bayesian Optimization HyperBand) for embedding quality in `nt_memory::embedder`.

### Finding 6: Gradient-Enhanced Bayesian Optimizer (Marchildon & Zingg, arXiv 2504.09375v2, May 2026)
- **Source**: https://arxiv.org/html/2504.09375v2
- **What's NEW**: Local BO framework that achieves same convergence as quasi-Newton and conjugate-gradient methods with same or fewer function evaluations — bridges BO gap with gradient-based optimizers.
- **NEOTRIX DEFECT 6.6**: NeoTrix SEAL pipeline uses simple round-robin or static provider selection. No Bayesian optimization of provider allocation. When multiple LLM providers are available, selection is random or cost-based. **Missing: Bayesian optimization of provider selection** — model each provider's performance distribution, use acquisition function to select provider that maximizes expected quality.
- **ACTION**: Add `ProviderBayesianOptimizer` to `nt_io::llm` — Gaussian process over provider performance history, Thompson sampling for next provider selection.

### Finding 7: PatchWorld — Gradient-Free Optimization of Executable World Models (Bai et al., arXiv 2605.30880, May 2026)
- **Source**: https://arxiv.org/abs/2605.30880
- **What's NEW**: Exposes tradeoff between **observation fidelity** and **action-discriminative dynamics** in world models — improving surface observation can weaken decision utility. Human-specified residual-memory bias helps fidelity but hurts action quality.
- **NEOTRIX DEFECT 6.7**: NeoTrix world model (`nt_world`) has no fidelity-action tradeoff awareness. Crawler accuracy is optimized independently from downstream action utility. **Missing: fidelity-action utility Pareto optimization** — when crawling content, optimize for both observation completeness AND action-relevance, not just one.
- **ACTION**: Add dual-objective crawler fitness in `nt_world::crawl` — Pareto front over content fidelity vs action utility score.

### Finding 8: Two-Stage Multi-Fidelity Bayesian Optimization (Chen et al., Applied Soft Computing 2026)
- **Source**: https://dl.acm.org/doi/10.1016/j.asoc.2026.115490
- **What's NEW**: Landscape-aware initialization + dual-adaptive sampling — uses low-fidelity approximation first, then high-fidelity refinement. 201 papers cite this pattern.
- **NEOTRIX DEFECT 6.8**: NeoTrix has no multi-fidelity optimization. SEAL pipeline runs full evaluation (compile+test) every cycle. No cheap proxy for early fitness estimation. **Missing: multi-fidelity SEAL evaluation** — use static analysis as cheap proxy for compilation, run full test only on modules that pass static analysis.
- **ACTION**: Add fidelity tiers to `nt_mind::seal::evaluate` — F0: syntax check, F1: type check, F2: lint, F3: unit test, F4: integration test. Early termination on failure.

---

## 3. SEARCH ALGORITHMS — NEW DEFECTS vs Batch 529

### Finding 9: TSMCTS — Twice Sequential Monte Carlo for Tree Search (Oren et al., ICML 2026)
- **Source**: https://icml.cc/virtual/2026/poster/61284
- **What's NEW**: Combines Sequential Monte Carlo with MCTS to address **path degeneracy** and **variance explosion** at depth. TSMCTS outperforms vanilla SMC and modern MCTS as policy improvement operator.
- **NEOTRIX DEFECT 6.9**: NeoTrix capability tree traversal is depth-first with no variance control. Deep capability chains (e.g., LLM→parser→embedder→KB) accumulate error variance without mitigation. **Missing: variance-aware tree search** — detect when variance exceeds threshold, switch to SMC-inspired resampling to kill degenerate paths.
- **ACTION**: Add `VarianceMonitor` to `nt_core::capability_tree::traverse` — track reward variance per path, resample when CV exceeds threshold.

### Finding 10: Self-Adaptive MCTS for Scheduling (Transportation Science, Vol 60, 2026)
- **Source**: https://pubsonline.informs.org/doi/10.1287/trsc.2025.0016
- **What's NEW**: MCTS that **self-adapts exploration constant** based on problem structure — no manual hyperparameter tuning. Adapts C_p (exploration-exploitation balance) dynamically.
- **NEOTRIX DEFECT 6.10**: NeoTrix GWT attention routing uses static resonance thresholds. When system state changes (e.g., high error rate), thresholds don't adapt. **Missing: self-adaptive attention thresholds** — adjust GWT resonance dynamically based on current system entropy.
- **ACTION**: Add `AdaptiveThresholdController` to `nt_core::gwt` — compute system entropy, scale exploration constant inversely to entropy (high entropy → more exploitation).

### Finding 11: MCPS — Monte Carlo Permutation Search (Cazenave, EmergentMind 2025-2026)
- **Source**: https://www.emergentmind.com/topics/monte-carlo-tree-search-algorithm
- **What's NEW**: Interpolates among node statistics, AMAF, and permutation-based statistics — **removes bias hyperparameters entirely** through statistical interpolation.
- **NEOTRIX DEFECT 6.11**: NeoTrix has multiple bias hyperparameters scattered across modules (GWT resonance threshold, emotion decay rate, attention weight). These require manual tuning per deployment. **Missing: hyperparameter-free statistical interpolation** — replace manual thresholds with adaptive statistics that interpolate based on observed data distributions.
- **ACTION**: Replace static thresholds in GWT/Emotion/Focus with data-driven adaptive statistics in corresponding modules.

### Finding 12: Parallel MCTS with 25× Speedup (Batch GPU Inference, Cazenave 2021, cited 2026)
- **Source**: https://www.emergentmind.com/topics/monte-carlo-tree-search-algorithm
- **What's NEW**: Batch GPU inference for MCTS nodes — decouples neural network forward pass from tree updates. Uses transposition tables + batch GPU calls.
- **NEOTRIX DEFECT 6.12**: NeoTrix processes capability evaluations sequentially. Each module test runs independently. No batching of similar evaluations. **Missing: batched evaluation pipeline** — group similar capability evaluations, run in parallel on GPU, use transposition table to cache evaluation results.
- **ACTION**: Add `BatchEvaluator` to `nt_mind::seal::evaluate` — group modules by dependency depth, evaluate same-depth modules in parallel batch.

---

## CROSS-CUTTING DEFECT: ORIGIN-ROOT ANALYSIS

| Batch 529 Defect | Batch 530 Root Cause |
|---|---|
| No grounding layer | DEFECT 6.7: No fidelity-action tradeoff in world model → perception feeds reasoning without grounding filter |
| No ontology governance | DEFECT 6.3: No topology evolution → ontology is static, cannot self-govern structural changes |
| SQLite ≠ graph DB | DEFECT 6.5: No matrix optimization for embeddings → KB queries treat embeddings as opaque blobs, not optimizable structures |
| No GraphRAG | DEFECT 6.9: No variance-aware search → graph traversal accumulates error without correction |
| Temporal blindness | DEFECT 6.8: No multi-fidelity evaluation → temporal evolution has no cheap proxy for monitoring |

---

## SUMMARY: NEW DEFECTS (12 total)

| ID | Defect | Severity | Module |
|---|---|---|---|
| 6.1 | No surrogate ensemble for multi-objective evolution | HIGH | nt_mind::seal |
| 6.2 | No explainability in evolutionary fitness | MEDIUM | nt_mind::seal |
| 6.3 | Static capability tree topology (no topology evolution) | HIGH | nt_core::capability_tree |
| 6.4 | No evolutionary tool-call sequence optimization | MEDIUM | nt_act::orchestrator |
| 6.5 | No gradient-free matrix optimization for embeddings | MEDIUM | nt_memory::embedder |
| 6.6 | No Bayesian optimization of provider selection | HIGH | nt_io::llm |
| 6.7 | No fidelity-action utility tradeoff in world model | HIGH | nt_world::crawl |
| 6.8 | No multi-fidelity SEAL evaluation | HIGH | nt_mind::seal |
| 6.9 | No variance-aware capability tree traversal | HIGH | nt_core::capability_tree |
| 6.10 | No self-adaptive GWT attention thresholds | MEDIUM | nt_core::gwt |
| 6.11 | Manual bias hyperparameters (not adaptive) | MEDIUM | cross-module |
| 6.12 | No batched evaluation pipeline | MEDIUM | nt_mind::seal |

## SOURCES CITED

1. Nishihara et al. (2026) "An Evolutionary Algorithm Assisted by an Ensemble of Pareto-Optimal Surrogate Models" — IEEE Trans Cybernetics, arXiv cs.NE/2026-08
2. Zhang et al. (2026) "NSGA-III + TOPSIS for UHPC Multi-Objective Optimization" — Eng App AI Vol 175
3. Miikkulainen (2026) "AAAI Tutorial: Evolution of Neural Networks" — UT Austin / Cognizant AI Lab
4. Wang (2026) "Solving Vehicle Routing Problem Based on Genetic Algorithm" — CIBDA 2026, ACM
5. Allen et al. (2026) "Gradient-Free Optimization for Matrix Functions" — arXiv 2609.03170
6. Marchildon & Zingg (2026) "Efficient Gradient-Enhanced Bayesian Optimizer" — arXiv 2504.09375v2
7. Bai et al. (2026) "PatchWorld: Gradient-Free Optimization of Executable World Models" — arXiv 2605.30880
8. Chen et al. (2026) "Two-Stage Multi-Fidelity Bayesian Optimization" — Applied Soft Computing Vol 201
9. Oren et al. (2026) "Twice Sequential Monte Carlo for Tree Search" — ICML 2026 Poster #61284
10. Self-Adaptive MCTS (2026) — Transportation Science Vol 60
11. Cazenave (2025-2026) "Monte Carlo Permutation Search" — EmergentMind
12. Cazenave (2021, cited 2026) "Parallel MCTS with Batch GPU Inference" — EmergentMind
