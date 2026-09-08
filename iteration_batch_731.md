# Iteration Batch 731 — Model Evaluation Landscape 2026

**Date**: 2026-09-07 | **Sources**: 15 papers/articles across 3 search queries

---

## NEW FINDINGS (16 defects + 8 improvements)

### D731-01: BenchMIRT — Benchmark Audit at Question Level (CRITICAL)
- **Source**: HuggingFace/AllenAI BenchMIRT (2026-09-01)
- **Finding**: IRT applied to 100 LLMs × 16 benchmarks × 34K questions independently recovers two dominant latent dimensions: safety and general reasoning. BenchMIRT predicts model performance on held-out questions at **79% accuracy** vs 70% baseline.
- **Defect**: NeoTrix has NO question-level benchmark audit capability. Our Constellation maturity (C0-C6) measures module health but cannot assess whether our own benchmarks discriminate capabilities or mix signals. We cannot tell if a detection module's benchmark is measuring safety or reasoning.
- **Action**: Implement `BenchmarkAuditCapability` that applies IRT analysis to any evaluation suite NeoTrix uses, identifying which questions discriminate and which are noise.

### D731-02: LLMEval-Fair — Dynamic Contamination-Resistant Evaluation (CRITICAL)
- **Source**: ACL 2026, LLMEval-Fair paper
- **Finding**: 220K proprietary question bank with dynamic sampling. 30-month longitudinal study of ~60 models shows: (a) ALL models converge to ~90% ceiling, (b) dynamic rankings diverge from static benchmarks (ρ≈0.65-0.72), (c) 26.7% of model pairs show rank inversions vs C-Eval due to contamination.
- **Defect**: NeoTrix SelfTest (T1/T2/T3) has no contamination-resistant evaluation mechanism. Our internal benchmarks can be gamed by overfitting. No dynamic sampling of test cases exists.
- **Action**: Add contamination detection to NT-REPAIR — detect when detection modules' benchmarks are saturated/contaminated via fill-in-the-blank replay tests.

### D731-03: Construct Validity Crisis — Single Factor = Time Trend (CRITICAL)
- **Source**: arXiv:2608.29420 (Aug 2026)
- **Finding**: Single factor explains **74.5%** of common variance across 421 model configurations × 12 benchmarks. That factor substantially tracks **model release date** (R²=0.505). Economic benchmarks do NOT form a distinct capability factor. A 2-month gap between models reads as capability difference when it's calendar.
- **Defect**: NeoTrix benchmarking methodology does not date-adjust capability comparisons. When comparing NT-MIND distillation quality across iterations, temporal artifacts could be mistaken for capability gains.
- **Action**: Implement `DateAdjustedBenchmark` — factor out release date before attributing score differences to capability.

### D731-04: Verification-Cost Errors (VCEs) — Hidden Failure Mode
- **Source**: arXiv:2608.08709 (Aug 2026)
- **Finding**: High benchmark accuracy can mask significant verification effort. VCEs = incorrect outputs that a declared fraction of verifiers fails to identify within budget. Unlike hallucinations, defined by verification failure, not output property.
- **Defect**: NeoTrix evaluates detection modules on accuracy only. We have NO verification-cost metric. A detection module with 98% accuracy but requiring 10x human verification effort would be rated equivalent to one requiring 1x effort.
- **Action**: Add `VerificationCostMetric` to SelfTest — measure not just correctness but human/AI effort to verify correctness.

### D731-05: Harness-Multiplier Effect — 10-20 Point Score Inflation
- **Source**: Digital Applied LLM Benchmark Methodology 2026
- **Finding**: On agentic benchmarks, the evaluation harness (scaffolding) moves scores by **10-20 percentage points** while model weights stay identical. Claude Opus 4.5 scores 80.9% on SWE-bench Verified but only 45.9% on SWE-bench Pro — same weights, 35-point gap.
- **Defect**: NeoTrix SelfTest T3 (production wiring) does not control for harness/scaffolding effects. Different test configurations can produce 10-20pt score variations that aren't real capability differences.
- **Action**: Add harness versioning to SelfTest — record exact scaffolding configuration with every benchmark result.

### D731-06: Static Benchmark Contamination — Fill-in-the-Blank Replay
- **Source**: LLMEval-Fair, 26.7% rank inversions
- **Finding**: Fill-in-the-blank replay tests on AGIEval and C-Eval show public benchmarks yield substantially higher completion counts than private datasets. Static benchmarks have severe data contamination undetectable by static evaluation.
- **Defect**: NeoTrix detection modules' test cases may be in training data of models being tested. No mechanism to detect or prevent this.
- **Action**: Implement `ContaminationCheck` — replay test on detection module benchmarks to measure leakage rate.

### D731-07: No Confidence Interval Reporting on Arena Rankings
- **Source**: Digital Applied methodology guide
- **Finding**: Top-3 models routinely sit within overlapping 95% CIs — scores differ by 2-5 Elo points while CIs span ±15-20 points. Rank ordering is partially statistical noise.
- **Defect**: NeoTrix capability rankings (Constellation scores, module health) report point estimates without confidence intervals. A "C4" module and a "C5" module might not be statistically distinguishable.
- **Action**: Add confidence intervals to all Constellation maturity assessments.

### D731-08: OmniEvaluator — Composable Cross-Modal Evaluation
- **Source**: arXiv:2609.01315 (Sep 2026)
- **Finding**: OmniEvaluator connects 4 inference backends, 4 evaluation frameworks, and 1000+ benchmarks through a single interface. Federated GPU inference across concurrent evaluations. CPU-verifier matches commercial LLM judge accuracy at zero API cost.
- **Improvement**: NeoTrix NT-IO could adopt this composable evaluation pattern for cross-domain capability assessment (NT-WORLD crawl quality × NT-MEMORY retrieval quality × NT-ACT execution quality).
- **Action**: Design `ComposableEvalBridge` connecting NT-IO evaluation tools across domains.

### D731-09: ASSERT — Specification-Driven Audit Pipeline
- **Source**: arXiv:2608.13840 (Aug 2026)
- **Finding**: ASSERT models GenAI evaluation as: background concepts → systematized concepts → measurement instruments → context. The reported rate is conditional on ALL measurement choices. Changing judge, evidentiary standard, or simulated user changes reported rate by 15+ points.
- **Defect**: NeoTrix audits (D1-D50) don't record measurement specification choices. When two audits disagree on a dimension, there's no traceability to measurement differences.
- **Action**: Add measurement specification logging to all NeoTrix audit dimensions — record judge, standard, population for every audit finding.

### D731-10: AgentAssay — Stochastic Regression Testing
- **Source**: arXiv:2603.02601
- **Finding**: First token-efficient framework for regression testing non-deterministic AI agents. Behavioral fingerprinting achieves 86% detection power where binary pass/fail has 0%. SPRT reduces trials by 78%. 78-100% cost reduction at equivalent statistical guarantees.
- **Defect**: NeoTrix SelfTest uses deterministic pass/fail verdicts. For stochastic modules (NT-MIND distillation, NT-WORLD classification), binary verdicts miss 86% of behavioral regressions.
- **Action**: Implement stochastic test semantics with three-valued verdicts (Pass/Fail/Inconclusive) backed by confidence intervals.

### D731-11: Agent-Testing Agent — Automated Agent Evaluation
- **Source**: ACL 2026, ATA paper
- **Finding**: Meta-agent combines static code analysis, developer interrogation, literature mining, and persona-driven adversarial test generation with adaptive difficulty. Surfaces more diverse and severe failures than expert annotators in 20-30 minutes vs days.
- **Improvement**: NeoTrix NT-REPAIR could deploy a self-testing meta-agent that generates adversarial inputs for detection modules based on code analysis and failure mode mining.
- **Action**: Design `AgentTestAgent` for NT-REPAIR that auto-generates regression tests from module code analysis.

### D731-12: BekchiAI — Agent Observability + Control in One Click
- **Source**: arXiv:2608.26867 (Aug 2026)
- **Finding**: 13 tool-using ReAct agents across 7 task categories, 2,057 deterministic test tasks. Defines behavioral metrics beyond accuracy: tool-call adherence, URL hallucination, source-match, per-model token cost. Platform provides full token and latency telemetry.
- **Defect**: NeoTrix NT-ACT tool execution has no equivalent behavioral metrics. We track pass/fail but not tool-call adherence, hallucination source-matching, or per-task token cost.
- **Action**: Add `AgentBehavioralMetrics` to NT-ACT — tool-call adherence, hallucination detection, source grounding, token efficiency.

### D731-13: Union Eval — Season-Rotation Anti-Contamination Benchmark
- **Source**: ALL-Bench-Leaderboard 2026
- **Finding**: Union Eval rotates 70% of questions each season with 30% anchor questions for cross-season IRT calibration. Mandatory JSON output eliminates keyword matching gaming. Confirmed weakness map: poetry+code cross-constraints (18-28%), complex JSON (0%).
- **Improvement**: NeoTrix could adopt seasonal rotation for detection module benchmarks — 70% new + 30% anchor ensures benchmarks stay discriminative.
- **Action**: Implement `SeasonalBenchmark` concept for NT-MEMORY evaluation suites.

### D731-14: LLM-as-Judge Position Bias + Judge-System Dependence
- **Source**: ASSERT paper, Adaline guide
- **Finding**: Judge choice changes both absolute reported rates AND system rankings. No single ranking of evaluated systems holds across all judges. Judge-system dependence documented.
- **Defect**: NeoTrix uses LLM-as-judge for quality assessment but doesn't control for judge-system interaction effects. Two different LLM judges could rank NT-MIND distillation outputs in opposite order.
- **Action**: Add cross-judge validation to all LLM-as-judge evaluations — at least 2 independent judges with disagreement analysis.

### D731-15: Harness vs Model Score Separation
- **Source**: SWE-bench Verified vs Pro gap analysis
- **Finding**: SWE-bench scaffolding reality check shows almost every headline result is self-reported. Scaffold gap alone can exceed 28 points. Most headline numbers are the harness's score, not the model's capability.
- **Defect**: NeoTrix SelfTest T3 (production wiring) reports module effectiveness scores without separating harness contribution from actual module capability.
- **Action**: Add `HarnessDecomposition` — decompose any production score into (harness contribution + true capability) using control experiments.

### D731-16: modality × search × consistency Cross-Dimensional Blindspot
- **Source**: arXiv:2608.06202 (Aug 2026)
- **Finding**: Chat UI vs API responses differ in accuracy. Enabling web search REDUCES accuracy by up to 8pp. Repeated runs produce inconsistent responses in 21% of prompts. Two modalities ground answers in different citations.
- **Defect**: NeoTrix evaluation assumes single-modality, single-run, search-off conditions. Real deployment varies modality, run count, and search access — all of which change evaluation outcomes by 8-21%.
- **Action**: Add `DeploymentConditionMatrix` to evaluation — test across modality × search × run-count conditions.

---

## IMPROVEMENTS TO ABSORB

| # | Improvement | Source | NeoTrix Component |
|---|------------|--------|-------------------|
| I731-01 | Item Response Theory for benchmark auditing | BenchMIRT | NT-MEMORY evaluation |
| I731-02 | Dynamic contamination-resistant question sampling | LLMEval-Fair | NT-REPAIR SelfTest |
| I731-03 | Date-adjusted capability comparison | arXiv:2608.29420 | NT-MIND benchmarking |
| I731-04 | Verification-cost as first-class metric | arXiv:2608.08709 | SelfTest scoring |
| I731-05 | Stochastic test semantics (3-valued verdict) | AgentAssay | NT-REPAIR regression |
| I731-06 | Behavioral fingerprinting for regression detection | AgentAssay | NT-MIND evolution tracking |
| I731-07 | Seasonal benchmark rotation with IRT anchors | Union Eval/ALL-Bench | NT-MEMORY benchmarks |
| I731-08 | Cross-judge validation with disagreement analysis | ASSERT | All LLM-as-judge uses |

---

## BATCH 731 SUMMARY

**Critical defects**: 7 (D731-01 through D731-07) — benchmark audit, contamination, construct validity, verification cost, harness inflation, contamination detection, confidence intervals
**Structural defects**: 6 (D731-08 through D731-13) — measurement spec logging, stochastic testing, agent behavioral metrics, seasonal rotation
**Configuration defects**: 3 (D731-14 through D731-16) — judge bias, harness decomposition, deployment conditions

**Root cause**: NeoTrix evaluates modules in isolation under fixed conditions. The 2026 evaluation landscape demands: (1) contamination-resistant dynamic benchmarks, (2) uncertainty quantification on all scores, (3) harness-vs-model decomposition, (4) cross-condition evaluation, (5) verification-cost as reliability signal.

**Key insight**: A single benchmark score in 2026 is "close to meaningless on its own" (Digital Applied). Trust requires triangulation across static academic eval + human-preference arena + agentic suite. NeoTrix has none of these three evaluation layers.
