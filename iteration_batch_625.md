# Iteration Batch 625 — NAS / AutoML / HPO 2026 Landscape + NeoTrix Defects

**Date**: 2026-09-06
**Research Loop**: 625/10000
**Prior Context**: Batch 624 proved (1) no column-level lineage in KB, (2) no MCP governance layer, (3) no agent behavior lineage, (4) no SEAL validation checkpoints, (5) GX Cloud shut down.

---

## Source Summary

### 1. Neural Architecture Search (NAS) 2026

| Source | Key Finding |
|--------|-------------|
| CVPR 2026 Workshop — CoLLM-NAS (Li et al.) | Collaborative LLMs for Knowledge-Guided NAS: addresses architectural invalidity + computational inefficiency in LLM-NAS integration |
| arXiv 2607.07984 — AgentNAS (Jeong et al., Jul 2026) | LLM produces seed → decomposes into **slotted architecture** (scaffold with named interchangeable slots) → conventional NAS explores. LLM seed search outperforms fixed-seed NAS on every task. |
| Yenra AI20 — Neural Architecture Search: 20 Advances (2026) | Hardware-aware NAS now **baseline expectation**, not niche. Multi-objective NAS outputs Pareto sets. Graph-based encodings outperform flat strings. Neural predictors enable learned pruning. Automated search space refinement. |
| OSDI 2026 — Drs.NAS (Wang et al.) | Ultra-efficient NAS for recommendation systems adopted in **production by major hyperscalers** |
| Nature Collection — NAS (2025/2026) | Field oriented toward reducing search cost, improving generalizability, edge computing + compression |

### 2. AutoML 2026

| Source | Key Finding |
|--------|-------------|
| OpODab — AutoML Frameworks 2026 Selection Guide | **Auto-sklearn declared dead**. Only 4 essential tools: H2O AutoML, AutoGluon, MLJAR, PyCaret. AutoGluon/H2O now natively support Time Series + Multi-Modal. |
| OpODab — Critical Gap #1 | **No open-source AutoML framework has automated retraining + drift management natively.** "AutoML stops at Best Model. You must build the bridge." |
| AutoML 2026 Conference (Ljubljana) | Tutorial: "From AutoML to AgenticML: LLMs as the New Search Operator". Multi-track: Methods + ABCD (Applications/Benchmarks/Challenges/Datasets) |
| Upskill.biz — Trends in AutoML 2026 | AutoML 3.0 = context-aware + domain-specific automation. GenAI integration for code generation + synthetic data. Federated learning + edge optimization. |
| Analytics Insight — AutoML 2026 | Enterprise adoption accelerating. Google Vertex AI, Azure AutoML, SageMaker Autopilot, DataRobot expanding. Data scientists shifting to governance + monitoring roles. |
| Statsig — AutoML Experimentation | "Document everything: why it chose that model, what alternatives it considered, how sensitive the choice was to your data." |

### 3. Hyperparameter Optimization (HPO) 2026

| Source | Key Finding |
|--------|-------------|
| Optuna GitHub Releases | **Optuna v5.0 RC** (Aug 2026): Conditional PED-ANOVA (KDD 2026) for hierarchical/dynamic search space importance. **Multi-Objective TPE** replaces NSGA-II as default sampler. AutoSampler for multi-objective + constrained optimization. |
| GUVI — Optuna vs Grid Search 2026 | Optuna = default choice in 2026. TPE + native pruning + parallel trials + visualization dashboard. Grid search only for <3 hyperparameters. |
| Business Research Co. — Bayesian Optimization Market | $34.59B market in 2026 (17.3% CAGR), projected $65.93B by 2035. Cloud-based + hybrid deployment dominating. |
| ACM Computing Surveys — Bayesian Optimization | Multi-fidelity Bayesian optimization with landscape-aware initialization (Chen et al., 2026). Sparse Bayesian optimization for structural FE model updating. |

---

## NEW Defects Identified for NeoTrix

### DEFECT-625-1: No Adaptive Search Space in SEAL Pipeline

**Evidence**: AgentNAS (arXiv 2607.07984) demonstrates that LLM-constructed slotted architectures produce superior search spaces vs. hand-engineered grammars (einspace). NeoTrix SEAL pipeline uses **static, hand-coded stage definitions** (Soil→Roots→Trunk→Branches→Fruits→Core) with no mechanism to dynamically refine which exploration paths are worth pursuing based on prior results.

**Impact**: SEAL wastes compute on unproductive exploration branches. No learned pruning of the exploration tree. The "Dark Forest" axiom (modules must compile+test+connect or be deleted) is applied manually, not via predictor-guided early termination.

**Fix Required**: Implement a **SEAL Search Space Refinement** module that analyzes prior cycle results and prunes unproductive exploration paths. Port the slotted architecture concept: each SEAL stage gets named, interchangeable sub-modules that can be swapped by a NAS-like search.

**Sources**: Jeong et al. 2026 (AgentNAS); Yenra AI20 §19 (Automated Search Space Refinement)

---

### DEFECT-625-2: No Hardware-Aware Optimization Layer

**Evidence**: Yenra AI20 §8 states "Hardware-aware NAS is now a baseline expectation, not a niche specialty." FLOPs alone insufficient; latency, memory, energy, deployment targets all matter. NeoTrix has **zero hardware-awareness** in its architecture decisions — the Constellation maturity ladder (C0-C6) and Rune Socketing system optimize for correctness, not deployment constraints.

**Impact**: NeoTrix modules may compile and test (C1-C2) but be undeployable on target hardware (edge, mobile, GPU-constrained). No Pareto optimization across accuracy/latency/memory/energy.

**Fix Required**: Add **Deployment Constraint** as a first-class dimension in the Constellation maturity ladder. C6 should include hardware-aware validation (actual device measurement, not theoretical FLOPs). Integrate hardware targets into Rune Socketing evaluation.

**Sources**: Yenra AI20 §8; Nath et al. "HW-GPT-Bench" 2024; Zhang et al. "On Latency Predictors for NAS" 2024

---

### DEFECT-625-3: No Multi-Objective Optimization in Capability Tree

**Evidence**: Optuna v5.0 (Aug 2026) adopts Multi-Objective TPE as default sampler, replacing NSGA-II. NeoTrix capability tree evaluates nodes on single-dimension scores (compilation, test pass, connection count). No Pareto front analysis across accuracy/latency/memory/robustness/carbon cost.

**Impact**: Capability tree cannot identify trade-off-optimal modules. A module with 95% accuracy but 10x latency may be ranked higher than a 90% accuracy 1x latency module, depending on arbitrary weight choices.

**Fix Required**: Implement **Multi-Objective Capability Scoring** using Pareto dominance. Each capability node gets a vector of objectives (accuracy, latency, memory, energy, robustness). Tree traversal returns Pareto fronts, not scalar rankings.

**Sources**: Optuna v5.0 release; Yenra AI20 §9 (Multi-Objective NAS)

---

### DEFECT-625-4: No Conditional/Hierarchical Search Space Support

**Evidence**: Optuna v5.0 introduces Conditional PED-ANOVA (KDD 2026 paper) for hyperparameter importance in hierarchical and dynamic search spaces. NeoTrix Rune Socketing has 5 independent slots (Crimson/Indigo/Obsidian/Golden/Alabaster) but no mechanism for **conditional dependencies** between slots (e.g., "if Crimson=data摄取, then Indigo should prefer transform over cache").

**Impact**: Rune combinations are explored uniformly, wasting evaluation on physically impossible or suboptimal combinations. No understanding of which rune choices constrain or enable other rune choices.

**Fix Required**: Model Rune Socketing as a **conditional hierarchical search space**. Define dependency rules: when certain rune colors are selected, the valid choices for other colors narrow. Use PED-ANOVA-style importance scoring to identify which rune choices have the highest downstream impact.

**Sources**: Optuna v5.0 (Conditional PED-ANOVA, KDD 2026); Optuna GitHub releases

---

### DEFECT-625-5: No Automated Retraining / Drift Detection (Production Gap)

**Evidence**: OpODab 2026 explicitly states: "No open-source AutoML framework has automated retraining + drift management natively. AutoML stops at Best Model. You must build the bridge." NeoTrix SEAL pipeline runs as **one-shot batch cycles** with no continuous monitoring, drift detection, or automated retraining triggers.

**Impact**: Once a SEAL cycle completes and a module reaches C4/C5 maturity, there is no mechanism to detect when the module's assumptions have drifted from reality (data distribution shift, external API changes, user behavior evolution). Modules silently degrade.

**Fix Required**: Implement **SEAL Continuous Lifecycle** module: (1) drift detection on module I/O distributions, (2) automated retraining triggers when PSI/KS drift exceeds threshold, (3) promotion/rollback based on metric regression. Bridge the "Production Gap" identified by OpODab.

**Sources**: OpODab 2026 AutoML Frameworks Guide (Critical Gap #1); Statsig AutoML Experimentation

---

### DEFECT-625-6: No Learned Predictor for Exploration Pruning

**Evidence**: Yenra AI20 §12 demonstrates that neural predictors enable learned pruning — "NAS avoids spending full training runs on candidates that likely have no path to the Pareto front." NeoTrix SEAL explores all branches with equal investment. No surrogate model predicts which explorations will succeed.

**Impact**: SEAL wastes compute on dead-end explorations. The convergence check (converge_check()) only runs at Phase-0, not continuously during exploration. No "zero-cost" proxy evaluations to pre-filter candidates.

**Fix Required**: Implement **SEAL Predictor Module** — a lightweight surrogate model trained on historical SEAL cycle results that predicts exploration success probability. Use predictor-guided pruning: terminate low-probability explorations early, allocate more compute to high-probability paths.

**Sources**: Yenra AI20 §12; White et al. 2021; NAS-Bench-Suite-Zero 2022

---

### DEFECT-625-7: No LLM-as-Search-Operator for Architecture Design

**Evidence**: AutoML 2026 Conference features tutorial "From AutoML to AgenticML: LLMs as the New Search Operator." AgentNAS (2026) shows LLMs produce superior seed architectures vs. fixed architectures. NeoTrix SEAL uses LLMs for distillation/analysis but **not as architecture search operators** — the architecture (module connectivity, data flow, attention routing) is manually specified.

**Impact**: NeoTrix architecture remains static across evolution cycles. LLMs could propose novel module connections, attention patterns, or data flow topologies that human designers would not consider. The E8 Hexagram reasoning engine uses fixed yijing-style symbols, not LLM-generated novel configurations.

**Fix Required**: Implement **LLM Architecture Search Operator** for SEAL: (1) LLM generates candidate module connectivity patterns, (2) slotted architecture decomposition defines bounded search space, (3) conventional search (evolution/GDAS) explores the space. Separate the roles of "expanding space" (LLM) and "exploring space" (NAS).

**Sources**: Jeong et al. 2026 (AgentNAS); AutoML 2026 Conference tutorial

---

### DEFECT-625-8: No Graph-Based Architecture Encoding for Capability Tree

**Evidence**: Yenra AI20 §11 shows graph-based encodings outperform flat strings for architecture representation. "Better encodings improve ranking correlation, which determines whether the search spends time on promising models or noise." NeoTrix capability tree is stored as flat node/edge records in KB without graph-aware encoding.

**Impact**: Capability tree similarity search, pattern matching, and predictor models operate on impoverished representations. Cannot capture skip paths, operator neighborhoods, or structural signals that graph encodings provide.

**Fix Required**: Implement **Graph-Aware Capability Encoding** — represent the capability tree as a graph structure (adjacency matrix + node features + edge features) rather than flat records. Use GNN-based encoders for predictor models and similarity search within the SEAL exploration space.

**Sources**: Yenra AI20 §11; Ru et al. "Encodings for Prediction-Based NAS" 2024

---

### DEFECT-625-9: No Online/Continuous Architecture Adaptation

**Evidence**: Yenra AI20 §20 discusses ONE-NAS: "architecture adaptation as part of ongoing online learning rather than a separate design phase." For drifting environments, architecture choice becomes part of system adaptation. NeoTrix SEAL runs as **discrete batch cycles** (60s tick in background loop), not continuous adaptation.

**Impact**: NeoTrix cannot adapt its own architecture in response to changing workloads, data distributions, or resource availability. The background loop (nt_mind_background_loop) monitors but does not modify architecture.

**Fix Required**: Implement **Continuous SEAL Mode** — architecture adaptation runs as ongoing online learning, not batch cycles. Module connections, attention weights, and data flow patterns adjust continuously based on observed performance, with stability guards to prevent oscillation.

**Sources**: Lyu et al. "Online evolutionary NAS" 2023; Yenra AI20 §20

---

### DEFECT-625-10: No Explainability/Lineage for Architecture Decisions

**Evidence**: Statsig AutoML documentation: "Document everything: why it chose that model, what alternatives it considered, how sensitive the choice was to your data." AutoML 3.0 emphasizes explainability as core requirement. NeoTrix SEAL makes architecture decisions but produces **no decision lineage** — which alternatives were considered, why specific paths were chosen, sensitivity analysis.

**Impact**: Cannot reproduce, audit, or learn from past architecture decisions. Violates the "Evidence-First" review methodology. The experience-tree captures outcomes but not the decision space that produced them.

**Fix Required**: Implement **SEAL Decision Lineage** — for each architecture decision, record: (1) candidates considered, (2) evaluation metrics for each, (3) selection criteria applied, (4) sensitivity analysis. Store in KB `experience` namespace alongside outcome records.

**Sources**: Statsig AutoML Experimentation; Analytics Insight AutoML 2026; OpODab 2026

---

## Summary Table

| # | Defect | Severity | Domain | Fix Complexity |
|---|--------|----------|--------|----------------|
| 625-1 | No adaptive search space in SEAL | HIGH | NT-MIND | Medium |
| 625-2 | No hardware-aware optimization | HIGH | NT-CORE + NT-PHYSICAL | High |
| 625-3 | No multi-objective capability scoring | HIGH | NT-CORE | Medium |
| 625-4 | No conditional hierarchical search spaces | MEDIUM | NT-MIND (Rune Socketing) | Medium |
| 625-5 | No automated retraining/drift detection | CRITICAL | NT-MIND + NT-ACT | High |
| 625-6 | No learned predictor for exploration pruning | HIGH | NT-MIND | High |
| 625-7 | No LLM-as-search-operator for architecture | HIGH | NT-MIND + NT-CORE | High |
| 625-8 | No graph-based capability encoding | MEDIUM | NT-MEMORY + NT-CORE | Medium |
| 625-9 | No online/continuous architecture adaptation | HIGH | NT-MIND | High |
| 625-10 | No explainability/lineage for decisions | CRITICAL | NT-MEMORY + NT-MIND | Medium |

---

## Sources Cited

1. Li, Z. et al. "CoLLM-NAS: Collaborative Large Language Models for Efficient Knowledge-Guided Neural Architecture Search." CVPR 2026 Workshop. https://openaccess.thecvf.com/content/CVPR2026W/CVPR-NAS26/
2. Jeong, S. et al. "Agentic Neural Architecture Search." arXiv:2607.07984, Jul 2026. https://arxiv.org/html/2607.07984
3. Wang, R. et al. "Drs.NAS: Ultra-Efficient Neural Architecture Search for Recommendation Systems." OSDI 2026. https://www.usenix.org/system/files/osdi26-wang-ruixuan.pdf
4. Morrill, K. "AI Neural Architecture Search: 20 Advances (2026)." Yenra, Jan 2026. https://yenra.com/ai20/neural-architecture-search/
5. OpODab. "AutoML Frameworks in 2026: The Only 4 Tools You Need." Aug 2026. https://www.opodab.com/2026/08/automl-frameworks-2026-selection-guide.html
6. AutoML 2026 Conference. Ljubljana, Slovenia. https://2026.automl.cc/
7. Optuna v5.0 Release Candidate. Aug 2026. https://github.com/optuna/optuna/releases
8. Optuna — Conditional PED-ANOVA (KDD 2026). https://github.com/optuna/optuna
9. GUVI. "Optuna Hyperparameter Optimization vs Grid Search Guide 2026." Jun 2026. https://www.guvi.in/blog/optuna-for-hyperparameter-optimization/
10. The Business Research Company. "Bayesian Optimization Tools Market Report 2026." Jul 2026. https://www.thebusinessresearchcompany.com/report/bayesian-optimization-tools-market-report
11. Statsig. "AutoML Experimentation: Automated Model Selection." Jun 2025. https://www.statsig.com/perspectives/automl-experimentation-model-selection
12. Upskill.biz. "Trends in Automated Machine Learning (AutoML) for 2026." Jun 2026. https://upskill.biz/trends-in-automated-machine-learning-automl-for-2026/
13. Analytics Insight. "How Modern AutoML Platforms Are Changing Data Science in 2026." May 2026. https://www.analyticsinsight.net/programming/automl-in-2026-how-it-transforms-data-science-processes
14. Chen et al. "Two-stage multi-fidelity Bayesian optimization with landscape-aware initialization." Applied Soft Computing, Sep 2026. https://doi.org/10.1016/j.asoc.2026.115490
15. Siddiqui et al. "Efficient Global Neural Architecture Search." arXiv:2502.03553, Feb 2025. https://arxiv.org/abs/2502.03553
