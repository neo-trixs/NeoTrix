# Iteration Batch 365 — Scientific Discovery / Experimental Design / Literature Mining

**Date**: 2026-09-06
**Cycle**: External research → defect identification → design suggestions

---

## Sources Cited

| # | Source | Published | Topic |
|---|--------|-----------|-------|
| S1 | [Co-Scientist — Nature](https://www.nature.com/articles/s41586-026-10644-y) | 2026-05-19 | Multi-agent Gemini system for structured hypothesis generation with tournament evolution + Elo ranking |
| S2 | [Robin — Nature](https://www.nature.com/articles/s41586-026-10652-y) | 2026-05-19 | Full lab-in-the-loop: hypothesis generation + data analysis in continuous feedback; 200× time reduction vs manual |
| S3 | [The AI Scientist — Nature](https://www.nature.com/articles/s41586-026-10265-5) | 2026-03-25 | End-to-end automation: ideation → code → experiments → manuscript → peer review; passed first-round peer review |
| S4 | [XunZi — Nat Biomed Eng](https://www.nature.com/articles/s41551-026-01769-6) | 2026-08-04 | AI biologist: 24.4M publications + 613.6 TB multimodal data; logical reasoning + data fusion for target discovery |
| S5 | [HypoForge — arXiv 2608.25770](https://arxiv.org/html/2608.25770) | 2026-08 | Self-improving multi-agent: adversarial generator–discriminator for hypothesis generation; experience-guided testing skill learning |
| S6 | [HyGRAIL — arXiv 2609.02056](https://arxiv.org/abs/2609.02056) | 2026-09-02 | Cost-aware KG hypothesis discovery: GNN triage + LLM review; 54% LLM call reduction |
| S7 | [DiscoPER — arXiv 2607.01131](https://arxiv.org/html/2607.01131v1) | 2026-07 | Autonomous discovery via meta-reflection: second-order reasoning over accumulated discoveries; 72.7% hypothesis support |
| S8 | [Sibyl — ICML 2026](https://icml.cc/virtual/2026/73435) | 2026 | Literature-based scientific discovery: tiered agents + provenance audit + temporal backtesting; 18% prediction confirmation |
| S9 | [POLAR — alphaXiv 2606.25197](https://www.alphaxiv.org/abs/2606.25197) | 2026-06-23 | Pretrained belief representations for adaptive data acquisition: unified BED/BO/AL framework; 100-10,000× fewer training samples |
| S10 | [COPEx — arXiv 2605.26990](https://arxiv.org/html/2605.26990v1) | 2026-05 | Constrained BED via online planning: amortized posterior + multi-step lookahead scenario trees under budget constraints |
| S11 | [BED Score Matching — UAI 2026](https://proceedings.mlr.press/v337/phillips26a.html) | 2026-08-06 | Decoupling EIG from policy via score matching: turns multiplicative into additive cost for cheaper policy tuning |
| S12 | [DABS — arXiv 2607.16927](https://arxiv.org/html/2607.16927v1) | 2026-07 | Deep Adaptive Bayesian Screening: spike-and-slab + amortized policy for discrete factorial design under tight budgets |
| S13 | [ALMAB-DC — arXiv 2603.21180](https://arxiv.org/pdf/2603.21180) | 2026-03 | Asynchronous GP-bandit + MAB allocation + distributed computing: eliminates batch-sync bottleneck |
| S14 | [SBI-BOED — IOPscience](https://iopscience.iop.org/article/10.1088/2632-2153/ae98e5) | 2026-08-25 | Simulation-based inference for BOED: MPR-GA parallel restarts; 22% improvement over prior SOTA |
| S15 | [meta-pipe — arXiv 2606.28363](https://arxiv.org/html/2606.28363) | 2026-04-05 | End-to-end SR/MA pipeline: 10 stages, 5 mandatory human checkpoints, overclaim detection (12 patterns), GRADE assessment |
| S16 | [OpenExtract — arXiv 2603.13338](https://arxiv.org/pdf/2603.13338) | 2026-03 | RAG-based data extraction for SLRs: PICO extraction F1=0.74, >95% time savings for quantitative extraction |
| S17 | [EviSearch — arXiv 2604.14165](https://arxiv.org/abs/2604.14165) | 2026-04 | Multi-agent clinical evidence extraction: PDF-query + retrieval-guided + reconciliation; 91.3% correctness vs 84.1% baseline |
| S18 | [AutoSynthesis — PubDB](https://pubdb.com/paper/2607.15247) | 2026 | Automated meta-analysis: PRISMA-aligned, screened 28 studies, extracted 20+ claims, matched expert Hedges' g estimates |
| S19 | [ARISMA — arXiv 2608.25050](https://arxiv.org/abs/2608.25050) | 2026-08-25 | Guidelines for AI-assisted systematic reviews: "inspected, benchmarked, logged, reversible assistant" — not autonomous reviewer |
| S20 | [Queryome — J Biomed Semantics](https://link.springer.com/article/10.1186/s13326-026-00366-5) | 2026-08-12 | Hierarchical multi-agent RAG over 28.3M PubMed abstracts; 89.93% on MIRAGE benchmark |

---

## Defects Found

### DEFECT-1: No Multi-Agent Tournament Ranking for Hypothesis Evolution

**Evidence**: S1 (Co-Scientist) uses an Elo-based tournament where Generation, Reflection, Ranking, Evolution, Proximity, and Meta-review agents iteratively debate and rank hypotheses. The tournament creates a self-improving loop where hypothesis quality scales with test-time compute. S5 (HypoForge) extends this with adversarial generator–discriminator for unsupervised hypothesis refinement.

**NeoTrix gap**: The `BayesianExperimentDesign` in `nt_core_hcube::bayesian_experiment` uses a flat VoI-based scoring (`IG + 0.5×VoI` at line 214) with no agent specialization or tournament mechanism. The `rpm_rank_candidates` method (line 216) ranks by a single scalar score — no multi-agent critique, no adversarial refinement, no Elo tracking. Hypotheses are generated externally and passed in as a static `Vec<Hypothesis>` — there is no self-generating, self-critiquing hypothesis production loop.

**Suggestion**: Introduce `HypothesisTournament` at L5 (Cognition layer) that wraps `BayesianExperimentDesign`. Agents: Generator (propose), Critic (adversarial review), Ranker (Elo), MetaReviewer (aggregate). Tournament rounds update hypothesis priors via Elo delta. This creates a self-improving hypothesis generation loop without requiring LLM fine-tuning, matching HypoForge's approach.

**Priority**: P1 — core reasoning gap

---

### DEFECT-2: No Closed-Loop Lab-in-the-Loop Integration

**Evidence**: S2 (Robin) demonstrates semi-autonomous scientific discovery by connecting literature search → hypothesis → experimental protocol → lab data → analysis → updated hypothesis in a continuous feedback cycle. S3 (The AI Scientist) achieves end-to-end: ideation → code → experiments → manuscript → peer review. Both systems close the loop between hypothesis and empirical validation.

**NeoTrix gap**: The SEAL pipeline (`seal_core/`) runs exploration → distillation → self-test → absorption, but there is no mechanism to connect hypothesis generation (`bayesian_experiment`) to external experimental execution and data return. The `search_skill_stage.rs` (line 165-213) creates exercises from crawl results but does not feed experiment outcomes back into the VoI/M-open loop. The `rank_then_commit` method (line 239) selects what to commit but has no callback to receive and integrate experimental results.

**Suggestion**: Add `ExperimentCallback` trait at L5 that bridges `BayesianExperimentDesign` to external execution. After `rank_then_commit` selects an experiment, the callback executes it, receives results, and calls a new `update_from_observation(outcome: f64, experiment_id: usize)` method to update posteriors. This closes the hypothesis→experiment→result→updated-hypothesis loop.

**Priority**: P1 — pipeline disconnected

---

### DEFECT-3: No Constraint-Aware Experimental Design

**Evidence**: S10 (COPEx) shows that real-world experiments have budget constraints, transition costs, and feasibility boundaries that static VoI ignores. S12 (DABS) handles discrete factorial screening under tight budgets (T ≪ p) with spike-and-slab priors. S13 (ALMAB-DC) eliminates batch-synchronous waste in parallel experiments. S9 (POLAR) achieves 100-10,000× sample efficiency by using pretrained belief representations.

**NeoTrix gap**: `VoIConfig` (line 17) has only `hypotheses`, `experiments`, `samples_per_experiment`, `noise` — no budget constraint, no cost model, no transition penalties. The `value_of_information` function (line 144) computes KL divergence without any cost weighting. There is no mechanism for amortized policy learning (POLAR's approach) or constraint-aware planning (COPEx's scenario trees).

**Suggestion**: Extend `VoIConfig` with `budget: f64`, `cost_per_experiment: Vec<f64>`, `transition_penalty: f64`. Add `ConstrainedVoI` that computes `VoI / cost` instead of raw VoI. For amortized scenarios, add `BeliefStateEncoder` trait that maps observation history to compact belief representations (POLAR's approach using pretrained foundation models).

**Priority**: P2 — scaling bottleneck

---

### DEFECT-4: No Literature Mining Pipeline or Provenance Audit

**Evidence**: S15 (meta-pipe) automates the complete SR/MA workflow across 10 stages with 5 mandatory human checkpoints, overclaim detection (12 patterns), and GRADE assessment. S16 (OpenExtract) achieves >0.8 precision/recall for data extraction. S17 (EviSearch) achieves 91.3% correctness with per-cell provenance. S19 (ARISMA) establishes the principle: "inspected, benchmarked, logged, reversible assistant." S20 (Queryome) orchestrates retrieval + reasoning across 28.3M PubMed abstracts with hierarchical multi-agent architecture.

**NeoTrix gap**: The `SearchSkillStage` (search_skill_stage.rs) has `query_generation`, `result_filtering`, `evidence_synthesis`, `grounding_quality` dimensions but: (1) no systematic review pipeline (no PRISMA compliance, no dual-extraction, no risk-of-bias), (2) no provenance tracking per extracted claim (the `Evidence` struct at line 76 has `source_url` and `claim` but no page number, verbatim quote, or extraction confidence), (3) no overclaim detection, (4) no GRADE-style quality assessment, (5) no temporal backtesting (Sibyl's approach). The KB has BM25 + embeddings but no structured evidence table concept.

**Suggestion**: Add `LiteratureMiningPipeline` at L1 (Action layer, NT-WORLD or NT-MEMORY) with: (a) `StructuredEvidence` type with `page`, `verbatim_quote`, `extraction_confidence`, `provenance_chain`, (b) `OverclaimDetector` with the 12 patterns from meta-pipe, (c) `GRADEAssessor` for semi-automated quality grading, (d) `TemporalBacktester` to validate predictions against post-publication literature (Sibyl's approach). Wire to `SearchSkillStage` as a new sub-dimension.

**Priority**: P1 — missing capability entirely

---

### DEFECT-5: No Cost-Aware Hypothesis Discovery over Knowledge Graphs

**Evidence**: S6 (HyGRAIL) combines heterogeneous GNN triage with LLM review for cost-aware hypothesis discovery over scientific knowledge graphs. It uses GNN to score candidates, identifies an "ambiguous region," and routes only uncertain cases to expensive LLM review — reducing LLM calls by 54.36%. S4 (XunZi) integrates 24.4M publications with multimodal data fusion.

**NeoTrix gap**: The KB has nodes, edges, embeddings, and BM25 index, but the `bayesian_experiment.rs` module operates on a flat hypothesis list with no graph structure awareness. There is no GNN triage step, no cost-aware routing between cheap (GNN) and expensive (LLM) evaluation, and no mechanism to discover missing edges in the KB graph as hypotheses. The `MOpenCheck` (line 244) only checks posterior concentration — not graph completeness.

**Suggestion**: Add `KGHypothesisDiscovery` that: (a) uses KB graph structure to generate candidate missing-edge hypotheses, (b) applies a lightweight GNN scorer to triage candidates, (c) routes ambiguous candidates (GNN confidence in [low, high] band) to `BayesianExperimentDesign` for VoI evaluation, (d) routes high-confidence GNN predictions directly as accepted hypotheses. This reduces expensive VoI computation by ~50% while maintaining discovery quality.

**Priority**: P1 — KB-underutilized

---

### DEFECT-6: No Meta-Reflection Over Accumulated Discoveries

**Evidence**: S7 (DiscoPER) introduces second-order reasoning: periodically analyzing accumulated discoveries to identify structural patterns, confounds, and epistemic gaps, then redirecting exploration. This "meta-reflection" improves recall from 7/9 to 8/9 patterns and support rate from ~60% to 72.7%.

**NeoTrix gap**: The ConsciousnessTree runs a 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core) but this loop operates on system health, not on accumulated scientific discoveries. The SEAL pipeline's absorption cycle distills experience but does not perform second-order analysis of its own accepted/rejected hypotheses. There is no `Reflect` module that treats prior discoveries as empirical data.

**Suggestion**: Add `DiscoveryReflector` at L6 (Meta-Cognition layer, NT-META) that: (a) maintains a `ClaimStore` of accepted/rejected hypotheses with metadata, (b) periodically runs pattern analysis over the store (confound detection, variable correlation, gap identification), (c) generates targeted exploration guidance for the next SEAL cycle. This is analogous to DiscoPER's Reflect module but operates on NeoTrix's own discovery history rather than ecological data.

**Priority**: P2 — evolution velocity gap

---

### DEFECT-7: No Provenance-First Extraction Pattern

**Evidence**: S17 (EviSearch) achieves 91.3% correctness by enforcing per-cell provenance: every extracted value traces to `{page, modality, verbatim_quote}`. S15 (meta-pipe) flags "confidence scores are LLM-generated and uncalibrated" as a known failure mode. S19 (ARISMA) mandates: "every consequential scientific decision must remain human-interpretable, human-auditable, and human-accountable."

**NeoTrix gap**: The `Evidence` struct (search_skill_stage.rs:76) lacks page-level provenance. The `SearchExercise` grounding_score is computed from `solution.confidence.clamp(0.0, 1.0)` (line 251) — an uncalibrated LLM output, exactly the failure mode meta-pipe identifies. The KB stores nodes and embeddings but has no per-claim provenance chain linking assertions to source documents.

**Suggestion**: Extend `Evidence` with `page: Option<usize>`, `verbatim_quote: Option<String>`, `extraction_method: ExtractionMethod` (enum: PDF_Query | RAG_Retrieval | Structured_Parse), `provenance_verified: bool`. Add KB edge type `SUPPORTS` that links claim nodes to source nodes with provenance metadata. Wire `SearchSkillStage.grounding_score` through a calibration function rather than raw LLM confidence.

**Priority**: P1 — audit trail broken

---

### DEFECT-8: No Asynchronous Parallel Experiment Execution

**Evidence**: S13 (ALMAB-DC) demonstrates that synchronous batch design wastes parallel resources: workers idle while waiting for the slowest evaluation. The async `WaitForAny` rule with Kriging Believer diversity achieves near-linear speedup while preserving statistical quality. S2 (Robin) processes 551 papers in 30 minutes (200× human speed) via parallel agent execution.

**NeoTrix gap**: `BayesianExperimentDesign` is entirely synchronous: `value_of_information` computes per-experiment sequentially, `rpm_rank_candidates` sorts linearly, and `rank_then_commit` returns a single index. There is no concurrent VoI computation, no batch-dispatch to parallel workers, and no async result collection. For large hypothesis spaces (N > 100), this becomes a bottleneck.

**Suggestion**: Add `ParallelExperimentDesign` wrapper that: (a) computes VoI for all candidates in parallel (embarrassingly parallel over experiments), (b) uses `rayon::par_iter` for concurrent computation, (c) supports async dispatch with `WaitForAny` pattern for external experiment execution, (d) maintains a `PendingExperiment` queue with timeout-based retry.

**Priority**: P2 — performance at scale

---

## Summary

| # | Defect | Severity | Source |
|---|--------|----------|--------|
| 1 | No multi-agent tournament ranking for hypothesis evolution | P1 | S1, S5 |
| 2 | No closed-loop lab-in-the-loop integration | P1 | S2, S3 |
| 3 | No constraint-aware experimental design | P2 | S9, S10, S12 |
| 4 | No literature mining pipeline or provenance audit | P1 | S15-S20 |
| 5 | No cost-aware hypothesis discovery over KG | P1 | S4, S6 |
| 6 | No meta-reflection over accumulated discoveries | P2 | S7, S8 |
| 7 | No provenance-first extraction pattern | P1 | S15, S17, S19 |
| 8 | No asynchronous parallel experiment execution | P2 | S2, S13 |

**Total sources**: 20 | **P1 defects**: 5 | **P2 defects**: 3
