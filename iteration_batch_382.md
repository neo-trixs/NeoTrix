# Iteration Batch 382 — Neural Architecture Search / AutoML / Hyperparameter Optimization

**Date:** 2026-09-06
**Scope:** External research scan → NeoTrix design defect identification → suggestions

---

## 1. Sources Cited

### Neural Architecture Search (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| N1 | DAG-NAS, ScienceDirect (Sep 2026) | Fully differentiable NAS for RL via scalar-level DAG modeling; constructs supernetworks enabling efficient architecture search with explicit connectivity modeling |
| N2 | LLM-NAS, arXiv:2510.01472v4 (Dec 2025) | LLM-driven HW-NAS reduces search cost from GPU-days to minutes; zero-cost predictor avoids training candidates from scratch; achieves 54% lower latency at similar accuracy |
| N3 | HW-NAS for ASIC, IEEE/ResearchGate (Jun 2026) | HW-NAS with area/power/latency constraints for ASIC; generates→synthesizes→evaluates via Cadence Genus for real hardware statistics |
| N4 | Affordable HW-NAS, arXiv:2606.16290 (Jun 2026) | HW-NAS for ultra-low-power MCUs; lightweight search executable on embedded devices themselves; 5-30x cost reduction |
| N5 | MicroNAS, Nature Scientific Reports (2025/2026) | First HW-NAS for time series on MCUs; combines DNAS + Latency Lookup Tables + Dynamic Convolutions; user-defined latency/memory limits |
| N6 | Multi-objective Differentiable NAS, Sukthanker et al. (2025) | Single search produces families of architectures conditioned on different tradeoff preferences and hardware settings — Pareto set output |
| N7 | β-DARTS++ / Edge Mutation DARTS (2023/2025) | Regularization against DARTS collapse; beta regularization + edge-level mutation preserve fair operation competition during search |
| N8 | Online Evolutionary NAS, Lyu et al. (2023) | ONE-NAS: architecture adaptation as part of online learning, not separate design phase; critical for drifting environments |
| N9 | Yenra NAS 20 Advances (Jan 2026) | Field synthesis: bounded search spaces + reliable supernet training + better surrogates + hardware-aware objectives + joint HPO; online NAS emerging as most important future direction |
| N10 | HEC-NAS-FDS, Nature (Jul 2026) | Hybrid expert-conditioned exhaustive NAS over finite design space; expert priors constrain search space for faster convergence |

### AutoML (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| A1 | AutoML 2026 Conference CFP (May 2026) | New tracks: **Agentic AutoML**, AutoML for LLMs, Foundation Models for AutoML, context/prompt optimization, dataset distillation |
| A2 | Green/Trustworthy AutoML Review, Springer (Jul 2026) | Systematic review of 103 studies: Green AutoML (energy-efficient), trustworthiness, interpretability, human-in-the-loop; carbon metric reporting still inconsistent |
| A3 | AutoML for Unsupervised Tabular Tasks, ACML 2026 | Extends AutoML beyond supervised settings; Bayesian optimization + evolutionary search + meta-learning for clustering/dim-reduction |
| A4 | NAS Without Priors, TMLR (2026) | Robust architecture search for unseen data without prior dataset knowledge; addresses domain shift in NAS |
| A5 | CarBOHB, AutoML 2026 Late-Breaking | Carbon-Aware Bayesian Optimization with Hyperband — integrates carbon intensity into HPO cost function for sustainable AutoML |
| A6 | Explaining AutoClustering, arXiv:2602.18348 (Feb 2026) | SHAP-based explainability for meta-learning in AutoML; identifies structural weaknesses in current meta-feature strategies |
| A7 | AutoML Literature Review, Springer (Nov 2025) | Comprehensive survey covering AutoML evolution; identifies gaps in automated pipeline synthesis and quality assurance |
| A8 | α-PFN, ICML 2026 | Fast entropy search via in-context learning; prior-fitted networks for zero-shot hyperparameter acquisition |

### Hyperparameter Optimization (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| H1 | Bayesian Optimization for Branching/Nested HPs, JASA (Apr 2026) | Unified BO framework for conditional hyperparameters (branching + nested); captures parameter dependence that vanilla BO assumes independent |
| H2 | LLM-Guided PBT, MethodsX (Jun 2026) | LLM-guided population-based RL for adaptive HPO; replaces fixed mutation/selection rules with LLM-generated strategies; scalable |
| H3 | Generative Bayesian HPO, Lopes/Polson/Sokolov (Apr 2026) | Weighted Bayesian Bootstrap + transport-map generator; amortized tuning (single forward pass replaces retraining loop); uncertainty summaries |
| H4 | Generalized PBT (GPBT), arXiv:2404.08233 | Pairwise Learning replaces greedy elite-only copying; comprehensive performance-differential guidance for underperforming agents |
| H5 | Homotopy + Surrogate HPO, Nature Scientific Reports (Feb 2026) | Flexible framework combining homotopy continuation with surrogate models for efficient HPO |
| H6 | IBUS, AutoML 2026 | Overcoming structural biases in hierarchical NAS with iterative bottom-up sampling |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-N1: No LLM-Driven Architecture Search (Search-from-Minutes)

**Evidence:** SEAL pipeline stages are statically defined via `make_stage!` macro. No mechanism uses an LLM to propose, evaluate, or adapt module architectures. The E8 Hexagram reasoning engine uses a fixed 64-element grid.

**2026 Gap:** LLM-NAS (N2) demonstrates that LLMs can drive HW-NAS, reducing search from GPU-days to minutes via zero-cost predictors. AutoML 2026 (A1) explicitly lists "Agentic AutoML" as a new frontier. NeoTrix's SEAL pipeline lacks any LLM-driven search capability.

**Severity:** High — SEAL evolution is limited to predefined stage transitions; cannot discover novel module architectures.

**Suggestion:** Add an `LLMArchitectureSearcher` to NT-MIND that uses an LLM to propose module architectures (input→transform→output signatures), evaluates them with zero-cost proxies (compile-time, memory footprint, static analysis), and selects Pareto-optimal designs. Integrate with E8 Hexagram as the representation language for proposed architectures.

---

### DEFECT-N2: No Hardware-Aware NAS for NT-PHYSICAL Edge Deployment

**Evidence:** NT-PHYSICAL defines sensors/motors/safety/power but has no architecture search mechanism for deploying neural components to constrained hardware. No latency/memory/energy constraints are encoded in any module search space.

**2026 Gap:** MicroNAS (N5), HW-NAS for ASIC (N3), and affordable HW-NAS for ultra-low-power MCUs (N4) all show that hardware-aware NAS is now production-ready for edge deployment. NeoTrix's NT-PHYSICAL layer has no mechanism to search for architectures that fit device constraints.

**Severity:** High — NeoTrix cannot autonomously optimize its own perception/action models for target hardware.

**Suggestion:** Implement a `HardwareConstrainedSearchSpace` in NT-PHYSICAL that encodes device-specific constraints (latency, memory, power) as differentiable penalty terms. Use Latency Lookup Tables (as in MicroNAS) for fast evaluation. The GWT attention router should modulate search based on current device health signals from HeartbeatAggregator.

---

### DEFECT-N3: No Multi-Objective Pareto Search for Module Design

**Evidence:** Module maturity is tracked via Constellations (C0-C6) but this is a single-axis ladder. No multi-objective optimization exists for balancing accuracy, latency, memory, energy, and robustness simultaneously.

**2026 Gap:** Sukthanker et al. (N6) shows single-search producing Pareto sets conditioned on different tradeoff preferences. Yenra (N9) confirms multi-objective NAS is now standard practice. NeoTrix's C0-C6 maturity ladder is one-dimensional.

**Severity:** Medium — forces binary decisions (deploy or not) rather than offering tradeoff surfaces.

**Suggestion:** Extend the Constellation maturity system with a `ParetoFrontier` overlay: for each module, maintain a set of Pareto-optimal configurations (accuracy vs. latency vs. memory). The Dual Specialization (Weapon Set I/II) system should select from the frontier based on current GWT attention context rather than a single "best" configuration.

---

### DEFECT-N4: No Online/Continual Architecture Adaptation

**Evidence:** SEAL pipeline runs in discrete cycles (explore→distill→test→absorb). No mechanism adapts architectures during deployment based on drifting data distributions.

**2026 Gap:** ONE-NAS (N8) demonstrates online architecture adaptation as part of continuous learning. Yenra (N9) identifies this as "one of the most important future directions." DAG-NAS (N1) enables differentiable search in RL settings where environments change.

**Severity:** High — NeoTrix cannot adapt its module architectures when the operating environment shifts; frozen architectures degrade.

**Suggestion:** Add an `OnlineArchitectureAdapter` to NT-MIND that runs lightweight differentiable NAS continuously during deployment. When HeartbeatAggregator detects performance drift (via time-decay health signals), trigger architecture mutation in the affected module's search space. Use edge-level mutation (N7) to prevent catastrophic architecture collapse during online adaptation.

---

### DEFECT-N5: No Carbon-Aware / Green AutoML Scheduling

**Evidence:** The SEAL pipeline runs exploration/distillation cycles without awareness of energy cost or carbon intensity. No mechanism adjusts compute allocation based on grid carbon signals.

**2026 Gap:** CarBOHB (A5) integrates carbon intensity into the HPO cost function. Green AutoML review (A2) documents 22 studies on sustainable AI; carbon metric reporting is now an expected practice. AutoML 2026 (A1) lists sustainability as a quality dimension.

**Severity:** Medium — unnecessary energy cost during high-carbon grid periods; violates Green AutoML principles.

**Suggestion:** Add `CarbonAwareScheduler` to NT-ACT that wraps SEAL pipeline execution: query carbon intensity API, defer heavy compute (distillation, architecture search) to low-carbon periods, prioritize lightweight tasks during high-carbon periods. Integrate with ResourceBudgetManager for cost-aware scheduling.

---

### DEFECT-N6: No Bayesian Optimization for Conditional/Nested Hyperparameters

**Evidence:** No module in NeoTrix implements BO for hyperparameters with conditional dependencies. The SEAL pipeline's distillation parameters are set statically or via simple grid/random search.

**2026 Gap:** JASA 2026 (H1) provides a unified BO framework for branching and nested hyperparameters. Standard BO assumes independence; real NeoTrix parameters (e.g., distillation temperature depends on which model is selected, which depends on task type) have conditional structure.

**Severity:** Medium — suboptimal hyperparameter tuning for conditional parameter spaces.

**Suggestion:** Implement `ConditionalBO` in NT-MIND that models the branching/nested structure of NeoTrix's hyperparameter space. Use transport-map generators (H3) for amortized tuning: once trained, produce optimal hyperparameters for any new configuration via single forward pass instead of expensive retraining loops.

---

### DEFECT-N7: No Agentic AutoML for Self-Evolving Pipelines

**Evidence:** SEAL pipeline stages are manually designed and wired. No agent autonomously discovers, tests, and integrates new pipeline stages.

**2026 Gap:** AutoML 2026 (A1) explicitly introduces "Agentic AutoML" as a new research direction — LLM agents that autonomously design and optimize ML pipelines. NeoTrix's SEAL pipeline is entirely human-authored.

**Severity:** High — the evolution pipeline itself cannot evolve; a meta-level limitation.

**Suggestion:** Add an `AgenticPipelineDesigner` to NT-META that uses an LLM agent to propose new SEAL pipeline stages, evaluate them against existing stages on held-out data, and integrate winning stages. The agent should maintain a portfolio of pipeline stage designs (warm-started from meta-learning, A3) and use in-context learning (A8, α-PFN) for rapid evaluation.

---

### DEFECT-N8: No Explainability for Architecture Search Decisions

**Evidence:** No module provides interpretable explanations for why a particular architecture or hyperparameter configuration was selected. The E8 Hexagram represents states but does not explain transitions.

**2026 Gap:** AutoClustering explainability (A6) uses SHAP to analyze meta-feature relevance. Green AutoML review (A2) identifies interpretability as a critical research gap. AutoML 2026 (A1) lists explainability as a quality dimension.

**Severity:** Medium — inability to explain architectural decisions limits trust and debugging.

**Suggestion:** Add `ArchitectureExplainer` to NT-META that computes SHAP values over the architecture search space features (layer types, widths, connectivity patterns) to explain why specific architectures were selected. Store explanations in KB alongside architecture snapshots for audit trail.

---

## 3. Summary

| Category | Defects | Severity Distribution |
|----------|---------|----------------------|
| Neural Architecture Search | N1, N2, N3, N4 | 2 High, 2 Medium |
| AutoML | N5, N7, N8 | 1 High, 1 Medium, 1 Medium |
| Hyperparameter Optimization | N6 | 1 Medium |

**Total: 8 defects identified, 3 High, 5 Medium**

**Priority Actions:**
1. **N1 (LLM-Driven Architecture Search)** — unlocks Agentic AutoML; meta-level enabler for all other improvements
2. **N4 (Online Architecture Adaptation)** — prevents architecture drift in deployed modules; critical for production resilience
3. **N2 (Hardware-Aware NAS)** — enables autonomous edge deployment; required for NT-PHYSICAL viability
4. **N7 (Agentic Pipeline Designer)** — makes the SEAL pipeline itself evolvable; addresses meta-level stagnation
