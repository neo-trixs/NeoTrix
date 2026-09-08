# Iteration Batch 464 — NeoTrix Consciousness Architecture Research Loop

**Date**: 2026-09-06
**Domains**: LLM Evaluation, NLP Metrics, Adversarial Testing, Agent Evaluation
**Sources Consulted**: 24

---

## Sources Cited

| # | Source | Date | Domain |
|---|--------|------|--------|
| S1 | Zylos Research — "LLM Evaluation and Benchmarking 2026" | 2026-01-16 | LLM Eval |
| S2 | FutureAGI — "BLEU vs ROUGE vs BERTScore: Worked Examples and 2026 Use Cases" | 2026-02-28 | Metrics |
| S3 | FutureAGI — "LLM Leaderboard Explained 2026: Arena, MMLU, MMMU, GPQA, SWE-bench" | 2026-08-06 | LLM Eval |
| S4 | OpenLM — "Chatbot Arena + | OpenLM.ai" | 2026-09-03 | LLM Eval |
| S5 | BenchLM — "LLM Leaderboard & AI Model Benchmarks — September 2026" | 2026-09-04 | LLM Eval |
| S6 | Zylos Research — "AI Agent Testing & Evaluation: The Complete 2026 Guide" | 2026-01-12 | Agent Eval |
| S7 | JobsByCulture — "AI Agent Evaluation Guide 2026: How to Test, Benchmark & Monitor LLM Agents in Production" | 2026-05-25 | Agent Eval |
| S8 | Kili Technology — "LLM Red Teaming in 2026: How Frontier Labs Test AI" | 2026-05-28 | Red Teaming |
| S9 | AppSecSanta — "LLM Red Teaming Guide 2026: Tools, Attacks & Methodology" | 2026-08-03 | Red Teaming |
| S10 | StingRai — "AI Red Teaming for LLM and Agentic Apps 2026" | 2026-07-01 | Red Teaming |
| S11 | AppScale — "Evaluation-Driven Development: Replacing TDD for LLM Systems (2026)" | 2026-05-18 | EDD |
| S12 | QASkills — "Eval-Driven Development for LLMs: A 2026 Guide" | 2026-06-15 | EDD |
| S13 | ValueStreamAI — "AI Monitoring in Production 2026: LLM Observability & Drift Detection" | 2026-05-07 | Monitoring |
| S14 | FutureAGI — "LLM Eval Data Drift Detection (2026): 3 Drifts" | 2026-05-20 | Drift |
| S15 | Brunyé — "Human Evaluation of Large Language Models: A Review and Protocol Selection Framework" (AI 7(5):174, MDPI) | 2026-05-19 | Human Eval |
| S16 | arXiv — "ConSiDERS-The-Human Evaluation Framework" (ACL Anthology) | 2024-05 | Human Eval |
| S17 | BaeSeokJae — "LLM Red Teaming Guide 2026: Security Testing for AI Agents" | 2026-05-10 | Red Teaming |
| S18 | Programming Helper — "AI Agent Evaluation Frameworks 2026" | 2026-04-12 | Agent Eval |
| S19 | NVIDIA — "Mastering Agentic Techniques: AI Agent Evaluation" | 2026-06-11 | Agent Eval |
| S20 | arXiv — "Evaluation-Driven Development of LLM Agents: A Process Model" | 2024-11 | EDD |
| S21 | BestHub — "Adversarial Testing: Three Disruptive Trends Shaping AI Quality in 2026" | 2026-03-02 | Adversarial |
| S22 | W&B — "Evaluating LLMs in Production: From drift detection to continuous monitoring" | 2026-06-03 | Monitoring |
| S23 | SingularityMoments — "LLM Benchmarks 2026 — MMLU, HumanEval, Chatbot Arena" | 2026 | LLM Eval |
| S24 | Springer — "From benchmarks to deployment: a comprehensive review of agentic AI evaluation" (Artif Intell Rev 59, 167, 2026) | 2026-04-24 | Agent Eval |

---

## Research Findings

### 1. LLM Evaluation: Saturation, Agent Benchmarks, and LLM-as-Judge

**F1.1 — Benchmark Saturation**: MMLU (~92%), HumanEval (~95%), and BIG-Bench Hard (~90%) are saturated for frontier models. SWE-bench Verified (~50%) and SWE-bench Pro (23%) remain discriminative. The field is shifting toward harder, contamination-resistant benchmarks: GPQA Diamond, MMMU (multimodal), AIME (math olympiad), and agent-specific benchmarks (WebArena, OSWorld, GAIA, TAU-bench). (S1, S3, S23)

**F1.2 — Chatbot Arena Dominance**: LMSYS Chatbot Arena (now arena.ai) has collected 5M+ votes across 296+ models. It operates as pairwise blind comparison with Elo ranking. Per-domain leaderboards (code, math, multilingual) are now the standard for production model selection. The Arena rebranded from LMArena to "Arena" in January 2026. (S3, S4, S23)

**F1.3 — LLM-as-Judge Maturity**: LLM-as-Judge methods achieve 80-90% agreement with human judgment at 500-5000x lower cost. Best practice: rubric-bound LLM-judge with reference-free scoring for open-ended generation. G-Eval (chain-of-thought LLM-as-judge) with calibrated rubrics is the 2026 production default. Multi-sample aggregation (3-5 judge samples) reduces noise. (S1, S2, S12)

**F1.4 — Agent Evaluation ≠ Model Evaluation**: Evaluating agents requires multi-dimensional scoring across reasoning layer (plan quality, plan adherence) and action layer (tool correctness, argument validity). Single-number benchmarks fail because agent performance is stochastic — requires aggregated metrics across 5-10+ trials. The CLASSic framework (Cost, Latency, Accuracy, Stability, Security) is emerging as standard. (S6, S7, S18, S19)

**F1.5 — 26 Agentic Evals in September 2026**: BenchLM tracks 26 agentic benchmarks including SWE-bench, Terminal-Bench, BrowseComp, WebArena, OSWorld, Aider Polyglot, TAU-bench, and others. Benchmarks now test multi-step task completion, not single-turn accuracy. (S5, S24)

### 2. Metrics: Classical Metrics Are Dead (Almost)

**F2.1 — BLEU/ROUGE Obsolescence**: BLEU (2002) and ROUGE (2004) remain only for benchmark continuity (WMT, CNN/DailyMail). For 2026 production eval: reference-free LLM-judge scoring with rubric-bound prompts has displaced them. BLEU breaks on creative writing, RAG answers, short outputs, cross-lingual settings, and low-resource tokenization. BERTScore (2020) adds semantic depth but fails on long-form coherence. (S2)

**F2.2 — Deterministic Checks Replace Metrics**: Task-specific deterministic checks are replacing overlap metrics: exact match for classification, JSON-schema validity for structured output, regex/fuzzy match for extraction. These run in microseconds with no model cost. (S2, S11)

**F2.3 — Eval-Driven Development (EDD)**: EDD replaces TDD for LLM systems: write evaluations BEFORE prompts, run regression evals in CI, gate every change on scores. Four eval layers: unit (component checks), scenario (end-to-end tasks), shadow (production traffic sampling), canary (new version A/B). Minimum 100 labeled cases for meaningful signal; <50 cases produce noise-drowned scores. (S11, S12)

**F2.4 — ConSiDERS Framework for Human Evaluation**: Six pillars for generative LLM human evaluation: Consistency, Scoring Criteria, Differentiating, User Experience, Responsible, Scalable. Addresses cognitive biases in human evaluators. Reproducibility crisis in NLP human evaluation acknowledged. (S15, S16)

### 3. Testing: Red Teaming Is Now an Operational Discipline

**F3.1 — Four-Layer Attack Surface**: Application layer (web pentest), Model layer (prompt injection, jailbreaks), Tool layer (tool misuse, MCP abuse), Data layer (poisoning RAG, memory). OWASP ranks prompt injection #1 for second consecutive year. (S8, S10, S17)

**F3.2 — Crescendo and Many-Shot Jailbreaks**: Multi-turn jailbreaks beat single-turn defenses by wide margins. One black-box method jailbreaks GPT-4-Turbo and GPT-4o on >80% of prompts. Single-turn defenses are insufficient. (S8)

**F3.3 — Four Required Red Teaming Tools**: Garak (initial coverage scans), PromptFoo (regression testing in CI/CD), PyRIT (automated adversarial refinement), Azure AI Safety (compliance-oriented evaluation). No single tool provides complete coverage. (S9, S17)

**F3.4 — Agent-Specific Attack Vectors**: Attacker-controlled agents in multi-agent pipelines can exploit trust relationships. Tool parameter manipulation via prompt injection. Session escalation through social engineering of orchestration layers. Memory poisoning attacks on RAG and persistent memory. (S8, S10)

**F3.5 — Continuous Red Teaming Program**: Four components: automated baseline (OWASP LLM Top 10 coverage on every deployment), human red team cadence (monthly for high-risk, quarterly for lower-risk), threat intelligence integration (new jailbreaks/incorporated within defined SLA), documentation for compliance (EU AI Act Article 55, NIST AI 600-1). (S8, S9, S10)

**F3.6 — Private Datasets, Not Public Benchmarks**: Frontier labs maintain private red teaming datasets that are never published. Public adversarial benchmarks can be part of compliance documentation but cannot be the whole answer. Muse Spark scored 19.8% on public benchmarks vs 2.0% on internal evaluations — a 10x signal leak. (S8)

### 4. Production Monitoring and Drift Detection

**F4.1 — Three Drift Types**: Input distribution drift (user prompts shift from eval set), Prompt-template drift (system message/few-shot changes while eval dataset frozen), Retrieval-corpus drift (RAG index grows/embeddings re-compute). All three silently age the golden eval set. (S14)

**F4.2 — Three-Layer Observability Stack**: Layer 1: Infrastructure Metrics (Prometheus + Grafana), Layer 2: LLM Telemetry (OpenTelemetry GenAI standard), Layer 3: Quality Evaluation (LLM-as-a-Judge sampling 5-10% of production traces). (S13, S22)

**F4.3 — Production Evaluation Reduces Failures by 60%**: Systematic evaluation infrastructure with continuous monitoring, multi-dimensional metrics, and human-in-the-loop systems reduces deployment failures. (S1)

---

## Defects Identified in NeoTrix Design

### DEFECT-1: No LLM-as-Judge Evaluation Pipeline for Agent Outputs
**Severity**: HIGH | **Affects**: NT-MIND (SEAL), NT-CORE (SelfTest), NT-ACT (orchestration)

**Finding**: NeoTrix has a `PanelJudge` / `LLMJudgeAdapter` system in `nt_core_gate` (S: `nt_core_gate/mod.rs`) with rubric-bound multi-sample aggregation, but the system only evaluates gate decisions — not agent task outputs. 2026 best practice (S1, S11, S12) requires LLM-as-Judge scoring on ALL agent outputs (tool calls, code generation, multi-step plans), not just gate decisions. The SEAL pipeline has no eval-driven loop: there are no regression eval suites that gate prompt/model changes.

**Evidence**: `nt_core_gate/mod.rs` — judges exist but scope is limited to gate verdicts. `nt_core_self_test.rs` — SelfTest trait evaluates structural health, not output quality. `nt_mind_eval_harness.rs` — exists but not integrated into SEAL pipeline as continuous eval.

**Recommendation**: Extend the existing `PanelJudge` to score SEAL pipeline outputs (distillation quality, skill crystallization, experience summaries). Create a regression eval suite (minimum 100 labeled cases) that gates every SEAL phase change. Implement EDD inner loop: evals before prompts, CI-gated regression.

---

### DEFECT-2: SelfTest Is Structural-Only — No Agent Performance Evaluation
**Severity**: HIGH | **Affects**: NT-CORE (SelfTest), Constellation system

**Finding**: NeoTrix SelfTest (T1-T3 tiers) evaluates: module existence (T1), registration in registries (T2), production wiring (T3). This is **structural health**, not **performance evaluation**. The 2026 agent evaluation ecosystem (S6, S7, S18) requires separate evaluation of: task completion rate, tool selection correctness, plan quality, cost per task, latency, and stability across trials. The CLASSic framework dimensions (Cost, Latency, Accuracy, Stability, Security) have no counterpart in the SelfTest system.

**Evidence**: `nt_core_self_test.rs` — SelfTest trait returns pass/fail with optional failure reasons. No numeric scoring, no multi-trial aggregation, no trajectory scoring. Constellation maturity levels (C0-C5) are structural milestones, not performance metrics.

**Recommendation**: Create an `AgentPerformanceTest` trait (distinct from `SelfTest`) that evaluates: (1) task completion rate across N trials, (2) tool selection accuracy, (3) cost-per-task tracking, (4) latency percentiles, (5) stability (pass@k vs pass^k). Wire into Constellation promotion as a parallel gate alongside structural SelfTest.

---

### DEFECT-3: No Adversarial Testing / Red Teaming Infrastructure
**Severity**: CRITICAL | **Affects**: NT-SHIELD (security), NT-ACT (MCP tools), All domains

**Finding**: NeoTrix has NT-SHIELD for stealth/proxy/fingerprint management but zero adversarial testing of its own LLM interactions. The 2026 landscape (S8, S9, S10, S17) requires: (1) prompt injection testing against the consciousness core, (2) tool misuse testing against MCP tool calls, (3) memory poisoning testing against KB/RAG, (4) multi-turn jailbreak resilience (crescendo attacks beat single-turn defenses on >80% of prompts). OWASP LLM Top 10 lists prompt injection #1 for two consecutive years.

**Evidence**: No `red_team` or `adversarial` module in `neotrix-core/src/`. NT-SHIELD modules (`nt_shield_sandbox`, `nt_shield_stealth_net`) handle external threat protection but not internal adversarial testing of NeoTrix's own LLM calls. No Garak/PromptFoo/PyRIT integration.

**Recommendation**: Implement `nt_shield_red_team` module with: (1) automated adversarial baseline suite covering OWASP LLM Top 10, (2) PromptFoo integration for CI/CD regression, (3) multi-turn adversarial test generation, (4) Egress Privacy Guard adversarial validation (test that the guard actually blocks adversarial extraction of source code/KB). Add to NT-SHIELD constellation at C1 minimum.

---

### DEFECT-4: No Eval Dataset Drift Detection
**Severity**: HIGH | **Affects**: NT-MEMORY (KB), NT-WORLD (PerceptionBridge), SEAL pipeline

**Finding**: NeoTrix maintains golden sets and benchmark data but has no mechanism to detect when those sets become stale. The 2026 drift taxonomy (S14) identifies three silent killers: (1) input-distribution drift (production queries diverge from eval set), (2) prompt-template drift (system messages change while eval dataset frozen), (3) retrieval-corpus drift (KB embeddings re-computed while eval set tests old embedding surface). Without drift detection, CI eval gates pass while production degrades.

**Evidence**: `nt_mind_eval_harness.rs` exists but has no versioned eval datasets, no embedding centroid monitoring, no template hash tracking. Knowledge base embedding updates happen without eval set refresh triggering.

**Recommendation**: Implement `EvalDatasetDriftDetector` with: (1) embedding centroid comparison for golden sets, (2) prompt template version hashing, (3) retrieval corpus snapshot diffing, (4) monthly refresh protocol. Wire into SEAL pipeline as Phase-0 pre-check (alongside `converge_check`).

---

### DEFECT-5: No Multi-Trial Aggregation for Non-Deterministic Outputs
**Severity**: MEDIUM | **Affects**: NT-CORE (evaluation), NT-MIND (distillation)

**Finding**: NeoTrix SelfTest results are single-run pass/fail. Agent performance is stochastic (S6, S7) — a single test run tells you very little about true capability. 2026 best practice requires running 5-10 trials minimum and reporting pass@k (probability at least one succeeds) and pass^k (probability all succeed). The distinction matters: pass@k for creative exploration, pass^k for customer-facing reliability.

**Evidence**: SelfTest in `nt_core_self_test.rs` runs once and returns `SelfTestResult`. No multi-trial loop. Bayesian experiment design (`bayesian_experiment.rs`) runs experiments but does not aggregate SelfTest results across trials.

**Recommendation**: Add `multi_trial` mode to `SelfTestRegistry::run_all()` that executes each SelfTest N times and reports pass@k/pass^k statistics. Store trial history in KB for trend analysis. Use pass^k for constellation promotion gates, pass@k for exploration decisions.

---

### DEFECT-6: No Human Evaluation Protocol for Consciousness Core Outputs
**Severity**: MEDIUM | **Affects**: NT-CORE (ConsciousnessTree), NT-FEEL (emotion), NT-IO (CLI interface)

**Finding**: NeoTrix has no structured human evaluation protocol for its meta-cognitive outputs (growth reports, health assessments, evolution recommendations). The ConSiDERS framework (S15, S16) establishes six pillars for human evaluation of generative LLM systems. The STEP-V protocol selection framework (S15) provides structured methodology: Stakes, Task-type, Evaluator availability, Purpose, Volume. Without this, evolution decisions lack quality grounding.

**Evidence**: `consciousness_core.rs` generates growth reports via `run_growth_cycle()` but no human feedback loop. No rubric for evaluating report quality. No inter-rater reliability tracking.

**Recommendation**: Implement human evaluation protocol for consciousness outputs: (1) define rubrics for growth report quality (accuracy, actionability, novelty), (2) periodic human review (monthly sample), (3) track inter-rater reliability, (4) feed human judgments back into evolution as ground truth for SEAL calibration.

---

### DEFECT-7: No Continuous Evaluation in Production Deployment
**Severity**: HIGH | **Affects**: NT-ACT (orchestration), NT-IO (LLM providers), NT-MEMORY (KB)

**Finding**: NeoTrix evaluates at build/test time but has no continuous production evaluation loop. The 2026 production standard (S13, S22) requires: (1) sample 5-10% of production traces through LLM-as-Judge, (2) track hallucination risk scores over time, (3) alert on statistically significant quality drops, (4) drift detection on live traffic. Without this, silent degradation goes undetected between SEAL cycles.

**Evidence**: No production trace sampling or continuous eval pipeline. EventBus handles events but no quality scoring attached. HeartbeatAggregator tracks system health (compilation, test pass rate) but not output quality.

**Recommendation**: Implement `ProductionEvalPipeline` that: (1) samples production traces at configurable rate, (2) scores via rubric-bound LLM-as-Judge, (3) maintains rolling quality metrics with alerting, (4) feeds quality signals into HeartbeatAggregator as a new dimension. Wire into NT-IO as a background task alongside existing event processing.

---

### DEFECT-8: No Agent Trajectory Scoring
**Severity**: MEDIUM | **Affects**: NT-ACT (orchestration), NT-CORE (reasoning)

**Finding**: NeoTrix evaluates tool call success/failure but does not score the full trajectory (reasoning chain → tool selection → argument validity → output quality). 2026 agent evaluation (S7, S19) requires trajectory scoring that traces the full execution path and evaluates reasoning quality, not just final output. NVIDIA's guidance: "evaluate trajectories, tools, and outcomes — not just model scores."

**Evidence**: `nt_act` modules track tool call results but no trajectory scoring infrastructure. `trace_evaluation` exists in `nt_core_self_test.rs` but is structural (checks trace exists), not quality-oriented.

**Recommendation**: Implement `TrajectoryScorer` that: (1) captures full execution trace (reasoning → tool call → result → next decision), (2) scores each step on correctness and efficiency, (3) identifies failure points in multi-step chains, (4) aggregates into trajectory quality score. Integrate with SEAL pipeline for distillation input.

---

### DEFECT-9: No Cost-Per-Task Tracking for LLM Operations
**Severity**: MEDIUM | **Affects**: NT-ACT (orchestration), NT-IO (LLM providers)

**Finding**: NeoTrix routes LLM calls through providers but does not track cost-per-task at the agent operation level. 2026 agent evaluation (S6, S7) requires CLASSic framework Cost dimension: API usage, token consumption, infrastructure cost per task. Without this, evolution decisions cannot optimize for cost-effectiveness.

**Evidence**: `nt_core_llm` tracks provider selection but no per-task cost accumulation. `CostManager` exists in CONTEXT.md terminology but implementation status unclear from search results.

**Recommendation**: Implement per-task cost tracking that: (1) accumulates token usage across provider calls within a single task, (2) assigns cost based on provider pricing, (3) reports cost-per-task in evaluation outputs, (4) gates evolution decisions on cost-effectiveness ratio alongside quality.

---

### DEFECT-10: No Multi-Agent Trust Boundary Testing
**Severity**: HIGH | **Affects**: NT-SHIELD (security), NT-ACT (multi-agent orchestration)

**Finding**: NeoTrix operates as a single consciousness system but the multi-agent architecture (E8引导者 + specialists) creates implicit trust boundaries. 2026 red teaming (S8, S10) identifies attacker-controlled agents in multi-agent pipelines as a primary attack vector: crafted messages exploiting trust relationships, tool parameter manipulation, session escalation. NT-SHIELD's sandbox (`nt_shield_sandbox`) tests egress policy but not inter-agent trust boundaries.

**Evidence**: NT-SHIELD modules handle external threats. No inter-agent message validation or trust boundary enforcement. Multi-agent communication through EventBus lacks adversarial message filtering.

**Recommendation**: Implement `InterAgentTrustBoundary` module that: (1) validates message provenance in multi-agent communication, (2) tests for adversarial message injection between specialist agents, (3) enforces tool access boundaries (agent A cannot call tools belonging to agent B), (4) provides audit trail for all cross-agent interactions. Add to NT-SHIELD at C2 minimum.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 24 |
| Research findings | 16 (across 4 domains) |
| Defects identified | 10 |
| Critical severity | 1 |
| High severity | 5 |
| Medium severity | 4 |

**Top 3 Priority Defects**:
1. **DEFECT-3**: No adversarial testing infrastructure (CRITICAL — security gap)
2. **DEFECT-1**: No LLM-as-Judge evaluation pipeline for agent outputs (HIGH — quality gap)
3. **DEFECT-7**: No continuous production evaluation (HIGH — operational gap)
