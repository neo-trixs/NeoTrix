# Iteration Batch 380 — Evolutionary Computation, Neuroevolution & Quality-Diversity

**Date**: 2026-09-06
**Cycle**: 380
**Domain**: SEAL pipeline, EvolutionEngine, NT-MIND self-evolution

---

## Sources Cited

| # | Source | Year | Key Contribution |
|---|--------|------|------------------|
| S1 | Zhong et al., "Parameter adaptive competitive differential evolution with local search" (Applied Intelligence, 2026) | 2026 | Success History Adaptation (SHA) + Dynamic Local Search (DLS) — late-stage polishing improves DE by 5.58% U-score |
| S2 | DE-2LS (arXiv:2606.27764, 2026) | 2026 | Late-stage coordinate-pattern local search on best solution — "do no harm" philosophy, 0.5% budget LS, conservative injection |
| S3 | LPSADE — Learning-guided DE via promising subpopulation identification (Neurocomputing, 2026) | 2026 | Linear decreasing promising subpopulation scheme identifies high-fitness + strategically distributed individuals each generation |
| S4 | ARES-LSHADE (GECCO 2026 competition) | 2026 | LLM-driven autonomous research loop producing evolutionary operators; scout-augmented mutation + CMA-ES integration; 510/744 wins |
| S5 | Graph Neural Evolution (GNE) — AAAI 2026 | 2026 | Population-as-graph representation; spectral GNN decomposes evolutionary signals into frequency components; explicitly controls exploration-exploitation via frequency filtering |
| S6 | BRKGA-GNN (Evolutionary Intelligence, 2026) | 2026 | Biased Random-Key Genetic Algorithm for mixed discrete-continuous NAS; continuous encoding + deterministic decoder |
| S7 | SHIODEG (J. Supercomputing, 2026) | 2026 | Hybrid SHIO+DE+Gaussian Transformation — staged search: DE for diversity, GT for diversity floor, SHIO for leader-driven exploitation |
| S8 | EVOM — Agentic Meta-Evolution of Actor-Critic Architectures (arXiv:2606.26327, 2026) | 2026 | Bi-level optimization: inner PPO training + outer LLM-driven meta-evolution; architecture search as program refinement |
| S9 | NEOL — Provably Sub-Linear Two-Timescale NeuroEvolution with Online Plasticity (arXiv:2606.20817, 2026) | 2026 | Two-timescale: outer topology evolution + inner online weight adaptation via reward-modulated plasticity (Hebb/Oja/BCM); sublinear regret O(√T) |
| S10 | NEVO-GSPT — Neuroevolution through Geometric Semantic perturbation (2026) | 2026 | Geometric Semantic Operators for NN evolution; DGSM operator enables controlled network size reduction; linked-list perturbation evaluation (GPU-minutes not GPU-days) |
| S11 | MFSPNet — Model-Free Surrogate-Assisted NAS (Expert Systems with Applications, 2026) | 2026 | Validation-loss-driven EMA estimator for architecture ranking without pre-trained surrogates; <3 GPU days for CIFAR-10/100/SVHN |
| S12 | GraphIR — Architecture-Level Search States for LLM-Guided NAS (arXiv:2608.01633, 2026) | 2026 | Mutation-aligned architecture IR with computation skeleton, mutation surface, validity envelope; improves LLM-guided architecture evolution |
| S13 | Neuroevolution Arena (arXiv:2608.10323, 2026) | 2026 | Nested ecological evaluation of update-and-inheritance regimes; GPU-accelerated spatial ecology of 10K neural cells; architecture-conditioned regime ranking |
| S14 | QD Survey (ScienceDirect, 2026) | 2026 | Comprehensive QD review: NSLC, MAP-Elites, unified QD framework, RIBS; containers/selection/mutation improvements; discrete combinatorial gap identified |
| S15 | QD in Discrete Combinatorial Domains (Mathematics, 2026) | 2026 | Only 12.5% of QD papers address discrete problems; proposes conceptual framework for operational resilience |
| S16 | Discrete Gene Crossover for QD (arXiv:2602.13730, 2026) | 2026 | IsoLineCross operator: discrete gene-level crossover + directional variation; +11.3% QD score on Walker2d |
| S17 | QDHUAC — Sample-Efficient QD-RL (arXiv:2604.20381, 2026) | 2026 | Target-free distributional critic enabling high UTD ratios; hybrid weight+batch normalization for stability; 10x sample efficiency |
| S18 | Dominated Novelty Search (GECCO 2025/2026) | 2025-26 | Local competition via dynamic fitness transformations instead of explicit archives; eliminates predefined bounds/parameters |
| S19 | QD-LLM — Parameter-Efficient Neuroevolution for Diverse LLM Generation (arXiv:2605.09781, 2026) | 2026 | Evolving prompt embeddings (~32K params) via QD; hybrid BC with formal coverage bounds (Theorem 5); co-evolutionary variation operators |
| S20 | Heuresis — Search Strategies for Autonomous AI Research (arXiv:2606.25198, 2026) | 2026 | MAP-Elites, Go-Explore, Islands, Curiosity, Omni strategies for idea search; 40 reward hacks across 1,628 runs; novel ideas never approach top-10 quality |
| S21 | Language-Driven QD for Robot Manipulation (arXiv:2608.30983, 2026) | 2026 | LLM-driven autonomous exploration for fitness/BD design; multi-BD MAP-Elites (MES); no task-specific prompts needed |

---

## Defects Found

### DEFECT-380-1: SEAL Pipeline Lacks Population-Based Search (Single-Individual Sequential Loop)

**Severity**: HIGH
**Location**: `nt_core_self::seal::SealPipeline` (`neotrix-core/src/unified/core/nt_core_self/seal/mod.rs:40-131`)

**Gap**: The SEAL pipeline runs a single `SelfEditGen` → single `GRPOLoop` evaluation per iteration. There is no population of candidates, no selection pressure, no crossover between successful edits. This is the most fundamental limitation — 2026 evolutionary computation has conclusively demonstrated that population-based search (DE, GA, ES, CMA-ES) outperforms single-point optimization on every class of non-trivial problem.

**Evidence from code**: `SealPipeline::run_iteration()` generates up to `max_edits_per_step` (5) edits sequentially, evaluates each with a simple weighted sum (`grpo.rs:127-132`), and returns a report. No population maintenance, no tournament selection, no recombination.

**2026 State of Art**: LPSADE (S3) maintains a full population with promising subpopulation identification each generation. SHIODEG (S7) runs DE + Gaussian + SHIO in staged sequence. Even the "conservative" DE-2LS (S2) maintains a population of 10D candidates. The GECCO 2026 ARES-LSHADE (S4) won the competition with population + local polish architecture.

**Suggestion**: Refactor `SealPipeline` to maintain a population of `SelfEdit` candidates. Each iteration: (1) mutate existing population members, (2) crossover pairs, (3) evaluate all, (4) select top-K for next generation. Use LPSADE's promising subpopulation scheme (S3) to dynamically identify which edits to evolve more aggressively. The GRPO evaluator becomes the fitness function, not the sole decision mechanism.

---

### DEFECT-380-2: No Success History Adaptation (SHA) for SEAL Parameters

**Severity**: HIGH
**Location**: `SealPipeline::new()` hardcoded config, `GrpoConfig::default()` (`grpo.rs:17-27`)

**Gap**: SEAL's GRPO config uses fixed parameters: `learning_rate: 0.001`, `epsilon_clip: 0.2`, `kl_beta: 0.01`. There is no adaptation mechanism that adjusts these parameters based on historical success. The `SelfEditGen` temperature is also fixed at construction time.

**Evidence from code**: `GrpoConfig::default()` returns static values. `SelfEditGen::new()` clamps temperature but never updates it. No success history memory exists.

**2026 State of Art**: Success History Adaptation (SHA) is now the dominant parameter control paradigm. PaCDE-DLS (S1) uses SHA to adjust F and CR from past successful trajectories. SHADE/LSHADE have used this since 2013 but 2026 variants add multi-factor storage, stage-aware history, and diversity-aware updates (S3). ARES-LSHADE (S4) further refines SHA with adaptive CMA-ES integration.

**Suggestion**: Implement `SuccessHistoryBuffer` in SEAL: store the last N successful iteration configs (lr, epsilon, beta, temperature, population_size). Each new iteration samples parameters from a Cauchy distribution centered on historically successful values, weighted by fitness improvement. Add `history: Vec<(GrpoConfig, f64)>` to `SealPipeline` and a `sample_from_history()` method.

---

### DEFECT-380-3: No Quality-Diversity Archive for Skill Diversity

**Severity**: HIGH
**Location**: `SealPipeline` (no archive), `EvolutionEngine` (no archive) (`evolution.rs`, `seal/mod.rs`)

**Gap**: SEAL evaluates each edit as a single scalar reward and keeps only the "best" per iteration. There is no mechanism to maintain diverse high-performing skills across behavioral dimensions. The capability tree (`EvolutionEngine`) has Bud/Graft/Prune but no archive that preserves behavioral diversity.

**Evidence from code**: `run_iteration()` computes `best_reward` and `avg_reward` — scalar values. No behavioral descriptors, no grid/voxel archive, no coverage metric. `EvolutionPlan` is a flat list of actions, not a structured archive.

**2026 State of Art**: MAP-Elites remains the reference QD algorithm (S14), but 2026 advances include: CVT-MAP-Elites for high-dimensional descriptors, CMA-ME for covariance adaptation, PGA-MAP-Elites for gradient-assisted exploration, and Dominated Novelty Search (S18) which eliminates grid parameters entirely via fitness transformations. QD-LLM (S19) demonstrates QD over prompt embeddings with formal coverage bounds.

**Suggestion**: Define behavioral descriptors for SEAL edits: (1) edit type distribution (refactor/fix/optimize/feature/doc), (2) target module domain, (3) complexity delta. Implement a `SkillArchive` using Dominated Novelty Search (S18) — it requires no predefined bounds and adapts dynamically. Each SEAL iteration adds diverse high-performing edits to the archive. Wire archive retrieval into `SelfEditGen` so future edit generation is seeded from diverse elite archives, not random noise.

---

### DEFECT-380-4: SelfEditGen Uses Deterministic Keyword Matching, Not Evolutionary Variation

**Severity**: MEDIUM
**Location**: `SelfEditGen::generate_edits()` (`self_edit_gen.rs:44-95`)

**Gap**: Edit generation is a deterministic function of keyword matching (`context_lower.contains("function")`). No mutation operators, no crossover between edits, no population-level recombination. The "noise" added is a fixed sinusoidal function `(i * 1.13).sin()`, not stochastic evolutionary variation.

**Evidence from code**: Line 74: `let noise: f64 = ((i as f64 + 1.0) * 1.13).sin() * noise_scale;` — deterministic pseudo-noise. No random number generator used. No mutation probability parameter.

**2026 State of Art**: IsoLineCross (S16) demonstrates that discrete gene-level crossover between elite edits accelerates discovery by 11-34%. BRKGA-GNN (S6) shows continuous random-key encoding enables smooth exploration of mixed discrete-continuous spaces. NEVO-GSPT (S10) uses geometric semantic operators with linked-list evaluation for GPU-minute-scale evolution.

**Suggestion**: Replace deterministic generation with evolutionary variation operators: (1) **Mutation**: random perturbation of edit parameters (target location, edit type, proposed text similarity), (2) **Crossover**: recombine successful edits by exchanging code fragments between parent edits, (3) **Selection**: tournament selection from archive elites. Add a `variation_operators` module to `seal/` that implements IsoLineCross-style discrete crossover adapted for code edits.

---

### DEFECT-380-5: No Late-Stage Local Search Polishing in SEAL

**Severity**: MEDIUM
**Location**: `SealPipeline::run_iteration()` (`seal/mod.rs:63-131`)

**Gap**: SEAL applies the same process at every iteration — no concept of "late-stage refinement." The GRPO evaluator returns a score but never refines the best candidate with local search. After many iterations, SEAL cannot polish promising edits to higher precision.

**Evidence from code**: `run_iteration()` is identical at iteration 1 and iteration 1000. No phase detection, no refinement budget, no local search mechanism.

**2026 State of Art**: DE-2LS (S2) demonstrates that late-stage coordinate-pattern local search on the best solution improves U-score by 5.58% while using only 0.5% of the total evaluation budget. The "do no harm" philosophy is key: global search does 99.5% of work, LS does final 0.5% polishing. PaCDE-DLS (S1) similarly activates DLS only in the late optimization stage.

**Suggestion**: Add a `LocalSearchPolisher` to SEAL that activates after iteration threshold T (configurable). When active: (1) take the best edit from the population, (2) apply coordinate descent on its parameters (target location precision, confidence threshold, expected improvement), (3) inject improved solution back into population. Budget: 0.5-1% of total SEAL compute. This preserves RDEx-style global search for 99% of the run.

---

### DEFECT-380-6: No Two-Timescale Architecture (Population Evolution + Online Adaptation)

**Severity**: HIGH
**Location**: `SealPipeline` (`seal/mod.rs`), `GRPOLoop` (`grpo.rs`)

**Gap**: SEAL conflates architecture evolution (what types of edits to generate) and parameter adaptation (GRPO weights) into a single timescale. There is no separation between "outer loop evolves structure, inner loop adapts parameters."

**Evidence from code**: `GRPOLoop` updates policy weights every iteration via a simple delta rule (`grpo.rs:112-116`). `SelfEditGen` never changes its structure. The two are evaluated together in `run_iteration()` with no hierarchical separation.

**2026 State of Art**: NEOL (S9) provides the theoretical foundation: two-timescale separation with provable sublinear regret O(√T). Outer loop evolves topology (architecture), inner loop adapts weights via reward-modulated plasticity. EVOM (S8) demonstrates this at scale with LLM-driven outer loop + PPO inner loop. Neuroevolution Arena (S13) shows that the update-and-inheritance regime matters as much as architecture choice.

**Suggestion**: Restructure SEAL into NEOL-style two-timescale architecture: **Outer loop** (evolutionary, per-N-iterations): evolves `SelfEditGen` structure (edit type distribution, generation strategy, temperature schedule) using tournament selection + crossover. **Inner loop** (online, per-iteration): adapts GRPO weights via reward-modulated updates. This separation enables the outer loop to discover better generation strategies while the inner loop optimizes within a strategy.

---

### DEFECT-380-7: No Surrogate-Assisted Evaluation (Expensive Full Evaluation for Every Candidate)

**Severity**: MEDIUM
**Location**: `GRPOLoop::evaluate()` (`grpo.rs:127-132`)

**Gap**: Every candidate edit is evaluated by the GRPO evaluator with a full forward pass through the policy. There is no cheap surrogate that can pre-filter bad candidates before expensive evaluation. The evaluator itself is a simple weighted sum, but the principle applies: as SEAL scales to real code generation, evaluation will become the bottleneck.

**Evidence from code**: `evaluate()` is O(n) over policy params. For small policies this is cheap, but the architecture has no abstraction for surrogate-assisted evaluation.

**2026 State of Art**: MFSPNet (S11) achieves competitive NAS results in <3 GPU days using validation-loss-driven EMA as a model-free surrogate — no pre-trained model needed. SAIL (surrogate-assisted illumination) and CMA-ME use surrogates to reduce QD evaluation costs. The "model-free" aspect is critical: no surrogate training overhead.

**Suggestion**: Implement `SurrogateRanker` in SEAL that maintains a lightweight EMA estimator (per MFSPNet's VLE-EMA) of edit quality based on early signal features (edit type, target complexity, confidence). Use for pre-filtering: only evaluate the top-K/5 candidates with the full GRPO evaluator, discard the rest. This is a direct adaptation of MFSPNet's approach — cheap proxy filters before expensive evaluation.

---

### DEFECT-380-8: No Exploration-Exploitation Frequency Decomposition

**Severity**: LOW
**Location**: Global SEAL architecture

**Gap**: SEAL has no explicit mechanism to control the exploration-exploitation tradeoff. The GRPO `kl_beta` provides some regularization, but there is no frequency-domain analysis of evolutionary signals to separate exploration (high-frequency, diverse) from exploitation (low-frequency, convergent) components.

**Evidence from code**: `GrpoConfig.kl_beta` is a single scalar. No concept of signal frequency decomposition.

**2026 State of Art**: Graph Neural Evolution (GNE, S5) represents the population as a graph and uses spectral GNNs to decompose evolutionary signals into frequency components. High-frequency components capture diverse exploration; low-frequency components capture consistent exploitation. This explicit frequency filtering makes exploration-exploitation control interpretable and effective — GNE achieves several orders of magnitude better solution quality than GA/DE/CMA-ES on benchmarks.

**Suggestion**: Add a `SpectralDecomposer` module that: (1) represents the SEAL population as a graph (edits as nodes, similarity as edges), (2) applies spectral GNN to decompose into frequency bands, (3) uses high-frequency band to inject diversity (exploration), low-frequency band to refine best candidates (exploitation). This is the most speculative suggestion but represents a paradigm shift from scalar parameter tuning to structural signal decomposition.

---

## Summary

| # | Defect | Severity | Effort | Priority |
|---|--------|----------|--------|----------|
| D380-1 | No population-based search | HIGH | Large | P1 |
| D380-2 | No Success History Adaptation | HIGH | Small | P1 |
| D380-3 | No QD archive for skill diversity | HIGH | Medium | P1 |
| D380-4 | Deterministic edit generation | MEDIUM | Medium | P2 |
| D380-5 | No late-stage local search polish | MEDIUM | Small | P2 |
| D380-6 | No two-timescale architecture | HIGH | Large | P2 |
| D380-7 | No surrogate-assisted evaluation | MEDIUM | Medium | P3 |
| D380-8 | No spectral frequency decomposition | LOW | Large | P4 |

## Recommended Implementation Order

1. **P1 (Immediate)**: D380-2 (SHA) + D380-4 (evolutionary variation) — smallest changes, biggest impact. SHA is ~50 lines. Evolutionary variation is ~200 lines.
2. **P1 (Near-term)**: D380-3 (QD archive) — requires new `SkillArchive` struct but leverages existing KB infrastructure.
3. **P2 (Medium-term)**: D380-1 (population) + D380-6 (two-timescale) — structural refactor of SEAL pipeline.
4. **P3 (Exploratory)**: D380-7 (surrogate) + D380-8 (spectral) — advanced techniques, defer until P1/P2 validated.
