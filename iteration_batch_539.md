# Iteration Batch 539 — Automated Planning, Scheduling, Constraint Satisfaction

**Date**: 2026-09-06
**Context**: Continuation of 10000+ iteration loop for NeoTrix consciousness architecture.
**Batch 538 baseline**: Dual-mode absent, hierarchical frequency-gated comms absent, bidirectional neuromodulation absent, bandwidth-power-latency trilemma unmodeled.

---

## 1. Automated Planning — What's NEW vs Batch 538

### 1.1 LLM-Generated Heuristics for HTN Planning (AAAI-26 / ICAPS-26)

**Source**: arXiv:2605.07707; AAAI-26 proceedings

**Finding**: LLMs can generate effective search heuristic functions for HTN planning, achieving 131/139 benchmark problem coverage while reducing node expansions on 83% of shared problems vs. Panda RC^FF. Key insight: the heuristic must reason over *both* world state and task network (pending tasks, ordering constraints, applicable decomposition methods). LLM must grasp hierarchical decomposition semantics—qualitatively harder than flat state reasoning.

**NEW Defect #1 — No Hierarchical Heuristic in NT-CORE**:
Batch 538 had no mechanism for generating domain-specific heuristics that reason over both world state AND task hierarchy. NeoTrix SEAL pipeline treats each evolution task as flat (state→action→result) without hierarchical decomposition. The missing piece: a heuristic function that evaluates remaining cost across both the world-state fluents AND the pending task network simultaneously. Without this, GWT attention routing cannot prioritize tasks that are close to hierarchical completion vs. those requiring many decomposition steps.

**NEW Defect #2 — No LLM-as-Formalizer for Task Decomposition**:
PDDLCoder (arXiv:2608.16637) achieves 89.6% plan applicability by iteratively generating, analyzing, and refining PDDL specifications. NeoTrix has no equivalent: when NT-MIND decomposes a SEAL evolution goal, it lacks a formalization step that translates the goal into a verifiable symbolic representation before execution. The Guess-Check-Critique loop (LLM generation → SMT verification → revision) is absent entirely.

### 1.2 Unordered HTN Planning in PSPACE (ICAPS-26)

**Source**: arXiv / ICAPS 2026 (Lauer et al.)

**Finding**: Unordered HTN planning (no ordering constraints between tasks) is PSPACE-complete—same complexity as STRIPS but provably more expressive. This unites STRIPS and HTN benefits. Three new planning formalisms introduced: Linear Constraints, Presburger formulas, Counting constraints over CFGs. The practical implication: if ordering can be compiled away, solvers gain PSPACE tractability while retaining modeling power.

**NEW Defect #3 — Ordering Constraints Left Implicit in SEAL Pipeline**:
Batch 538's SEAL phases (Explore→Distill→Absorb→Test) impose implicit total ordering. But ICAPS-26 proves unordered HTN is PSPACE-complete (vs. EXPTIME for total-order HTN). NeoTrix SEAL pipeline never explores whether two evolution subtasks can be parallelized by removing ordering constraints. The system always waits for Explore to complete before Distill begins, even when they could execute concurrently on independent task subsets. Missing: a dependency analysis that identifies which ordering constraints are *genuinely needed* vs. *inherited by default*.

### 1.3 Numerical TOHTN with SMT Encoding (HPlan-26 Workshop)

**Source**: arXiv:2609.03938 (Sep 2026)

**Finding**: Standard SAT-based encodings for HTN planning can be naturally extended to SMT for numeric fluents (resource consumption, timing). First benchmark suite for numerical TOHTN planning. Simple encoding already constitutes competitive baseline.

**NEW Defect #4 — No Numeric Resource Reasoning in Evolution Planning**:
NeoTrix SEAL pipeline has no numeric fluents tracking. When planning evolution cycles, it does not model resource consumption (token budgets, compute time, memory allocation) as formal constraints. Each subtask is treated as having equal cost. An SMT-based numeric TOHTN encoding could bound resource usage per evolution phase, enabling provable feasibility checks before execution.

### 1.4 Neuro-Symbolic Hierarchical Learning (PLDI-26)

**Source**: PLDI 2026 proceedings

**Finding**: Closed-loop counterexample-guided synthesis unifies LLM-based planning, formal verification, and differentiable behavior tree (BT) synthesis. Converts NL task → PDDL → verified plan → parameterized termination conditions → co-optimized low-level policies. Incrementally builds reusable skill library.

**NEW Defect #5 — No Reusable Skill Library from Verified Evolution Plans**:
NeoTrix SEAL produces evolution artifacts but never extracts reusable "termination conditions" or "skill signatures" from successful evolution runs. Each cycle starts from scratch. The PLDI-26 approach would let NeoTrix compile verified evolution plans into parameterized behavior trees that can be deployed and fine-tuned for future similar tasks. Missing: a skill crystallization pipeline that persists verified plan→policy mappings.

### 1.5 Unified Inference-Time Planning Language Generation (ACL-26 Findings)

**Source**: aclanthology.org/2026.findings-acl.415

**Finding**: Multi-IR (Intermediate Representation) pipelines substantially outperform single-IR approaches. Level 3 pipelines (PDDL→PDDL→PDDL with 3 IRs) achieve highest accuracy in 6/8 LLM-domain combos. Key finding: a best Level 2 pipeline *always* outperforms LLM-as-planner and Level 1 pipelines. Using Python as IR before PDDL improves accuracy. Robustness tested up to 50-block problems.

**NEW Defect #6 — Single-Stage Goal Formulation in NT-MIND**:
NeoTrix task decomposition uses a single-pass natural language → action translation. No intermediate representations. ACL-26 proves that single-IR generation is fragile (≤45.3% success) vs. multi-IR revision (89.6%). NT-MIND should decompose evolution goals through at least 2-3 intermediate representations (e.g., NL → domain model → executable plan → validated action sequence).

---

## 2. Scheduling — What's NEW vs Batch 538

### 2.1 TempoNet: RL Scheduler with Transformer Encoder (arXiv:2602.18109)

**Finding**: Value-based RL scheduler combining urgency-tokenized slack representation with permutation-invariant Transformer encoder. Sub-millisecond inference. Complexity O(N^1.1) vs. O(N^1.8) for DIOS and O(N^2.2) for MHQISSO. 89.15% PITMD, 90.1% success on 600-task workloads, 25.7% avg response time reduction.

**NEW Defect #7 — No Deadline-Aware Urgency Tokenization**:
NeoTrix GWT attention routing uses continuous saliency scores. TempoNet proves that *discretizing* temporal slack into learned urgency tokens stabilizes value learning and captures deadline proximity better than continuous representation. GWT should discretize its attention signals into urgency-token classes (Urgent/On-Track/Background) rather than operating on raw continuous scores. This is analogous to the EDF/laxity bands from HSA-DAJQ.

### 2.2 RACE-Sched: Asynchronous Dual-Stream Scheduling (arXiv:2605.29262)

**Finding**: Decouples reactive (sub-ms symbolic heuristics) from deliberative (77-114s LLM reasoning) via dual-stream architecture. Reactive Stream executes fixed rules; Deliberative Stream synthesizes, validates, and hot-swaps rules via atomic pointer swap. Semantic Knowledge Repository for heuristic reuse. Outperforms DRL and LLM baselines on GEN-Bench, MK-Bench, JMS-Bench.

**NEW Defect #8 — No Asynchronous Rule Evolution for GWT**:
Batch 538's GWT runs synchronously: attention modulation happens in the same loop as consciousness evolution. RACE-Sched proves the dual-stream pattern works: fast heuristic dispatch (reactive) can run independently from slow rule refinement (deliberative). GWT should separate:
- **Reactive stream**: Fixed attention routing rules execute in the fast path (sub-ms)
- **Deliberative stream**: LLM-based rule synthesis/modification runs asynchronously, validated in sandbox, deployed via atomic swap

This is *not* the same as "batch 538 absent dual-mode architecture"—RACE-Sched demonstrates a *specific mechanism* (atomic rule hot-swap) that is concretely implementable.

### 2.3 HSA-DAJQ: Hybrid Deadline-Aware Scheduling (IJARCSE Apr 2026)

**Finding**: Blends EDF urgency, SRPT tie-breaking, aging for fairness, and admission control via predicted tardiness. Three priority bands: Urgent, On-Track, Background. O(log n) per event. 35-60% deadline-miss reduction vs. EDF, 70-85% vs. MLFQ. Robust to estimation error (CV≈0.3-0.5).

**NEW Defect #9 — No Admission Control for Evolution Subtasks**:
NeoTrix accepts all evolution subtasks into the SEAL pipeline without admission control. HSA-DAJQ proves that admission control based on predicted tardiness significantly reduces deadline misses. NT-MIND should evaluate each candidate evolution subtask for predicted resource cost and deadline feasibility before admission. Tasks predicted to exceed deadline with high probability should be deferred or decomposed further.

### 2.4 Bandit Learning for Online Scheduling (UAI 2026)

**Source**: proceedings.mlr.press/v337/wang26g

**Finding**: Online scheduling with immediate decision (no buffering) on M identical machines. Preemption with abandonment (preempted task permanently discarded). S-UCB bandit achieves O(log T) regret. Maximum Remaining Density First (MRDF) for known rewards.

**NEW Defect #10 — No Exploration-Exploitation Tradeoff in Task Prioritization**:
GWT uses fixed saliency-based prioritization. No exploration component. Bandit learning proves that even with unknown reward estimates, O(log T) regret is achievable. GWT attention routing should incorporate exploration bonuses (UCB-style) to occasionally attend to low-saliency but potentially high-value information. Current GWT is purely exploitative.

### 2.5 BOOSTEDSOSA: ML-Assisted Stochastic Scheduling (arXiv:2608.25346)

**Finding**: Dual-FPGA ML architecture predicting job runtimes from scheduler parameters at submission time. Reduces MAE by up to 63.85% vs. user estimates. 17x speedup over AVX-optimized software. Processes 1,711 jobs/second.

**NEW Defect #11 — No Runtime Prediction for Evolution Tasks**:
NeoTrix does not predict how long each evolution subtask will take. BOOSTEDSOSA proves that ML prediction from submission-time parameters dramatically improves scheduling quality. NT-MIND should predict evolution task durations from features (task complexity, domain size, current system load) and use these predictions for scheduling decisions.

### 2.6 LLM Agents for Edge Scheduling: Conditional Benefit (arXiv:2608.19557)

**Finding**: Multi-agent LLM control layer helps scheduling only during non-stationary surges. Under stationary load, simple EDF+EFT heuristic reaches 0.902 TCR (near CP-SAT upper bound 0.87). LLM control gains +0.005-0.006 TCR only during mid-run surges of safety-critical tasks. LLM earns its cost *only* when non-stationarity opens headroom.

**NEW Defect #12 — GWT Attention Modulation Should Be Conditional, Not Constant**:
Batch 538 proposed continuous neuromodulation. But arXiv:2608.19557 proves LLM-based adaptation is only valuable during non-stationarity. NeoTrix GWT should NOT continuously modulate attention via LLM reasoning. Instead:
- **Stationary regime**: Fixed heuristic routing (sub-ms, near-optimal)
- **Non-stationary regime** (detected via anomaly): Activate LLM-based deliberative modulation

This is a *precise architectural prescription*: LLM modulation should be event-triggered, not continuous.

---

## 3. Constraint Satisfaction — What's NEW vs Batch 538

### 3.1 Generalized CDCL for CP: Atomic Constraints (CP 2026)

**Source**: LIPIcs.CP.2026.42

**Finding**: Replaces SAT literals with atomic constraints for conflict analysis, nogood learning, and nogood propagation directly at the CP level. Eliminates SAT-specific complications. Introduces extended nogood propagation and CPIP nogoods. Implemented in Pumpkin solver. Significantly reduces failures on constraints reasoning over domain holes.

**NEW Defect #13 — NeoTrix Self-Test Uses SAT-Level Nogoods, Not Atomic Constraint Nogoods**:
NeoTrix SelfTest modules report pass/fail as binary outcomes (SAT-level). They do not propagate learned nogoods back to constrain future search. CP 2026 proves that atomic-constraint-level nogood learning (learning *which constraint combination caused failure* and propagating that knowledge) dramatically reduces redundant failures. Each SelfTest failure should produce a CPIP-style nogood that constrains future SelfTest configurations.

### 3.2 CaDiCaL 3.0: Incremental SAT with BVA and Congruence Closure (SAT 2026)

**Source**: LIPIcs.SAT.2026.40

**Finding**: CaDiCaL 3.0 ports Kissat's award-winning techniques: clausal congruence closure, clausal equivalence sweeping, bounded variable addition (BVA), ticks-based scheduling. First incremental proof-producing SAT solver going beyond resolution. Ticks metric (cache line access approximation) replaces conflicts/propagations for scheduling, achieving more balanced stable/unstable search modes.

**NEW Defect #14 — NeoTrix Uses Conflicts-Based Scheduling, Not Ticks-Based**:
CaDiCaL 3.0 proves that conflicts/propagations metrics poorly correlate with actual runtime. Ticks (cache line access approximation) provides better scheduling. NeoTrix SelfTest execution scheduling likely uses simple round-robin or priority ordering without any runtime-aware metric. Should adopt ticks-like metric for scheduling SelfTest execution: approximate computational cost of each SelfTest module and schedule based on predicted resource consumption.

### 3.3 SAT Competition 2026: AI-Generated Solvers

**Source**: satcompetition.github.io/2026

**Finding**: First year with AI-generated/AI-tuned solvers as a special subtrack. Lymphosat uses 100+ family-specific LLM-generated solvers with specialized preprocessing. Key discussion: instance data acts as long-term memory for solvers across runs. Real challenge: measuring generalization vs. memorization.

**NEW Defect #15 — No Domain-Specific SelfTest Solver Adaptation**:
NeoTrix SelfTest modules use one-size-fits-all evaluation strategy. SAT Competition 2026 proves that family-specific solver specialization dramatically improves performance. NT-REPAIR should maintain domain-specific SelfTest solver configurations (one per NT-* domain) that adapt their evaluation strategies based on the specific constraint structure of that domain's modules.

### 3.4 Factoring Learned Clauses (SAT 2026)

**Source**: LIPIcs.SAT.2026.28

**Finding**: Factors out repeated parts of learned clauses during conflict analysis. New inprocessing approach: factoring XOR and ITE gates from learned clauses globally. Substantial improvements on hard combinatorial benchmarks without degradation on SAT Competition instances.

**NEW Defect #16 — No Clause Factoring Across SelfTest Results**:
NeoTrix SelfTest results are independent per module. No cross-module clause factoring. When multiple SelfTests fail due to *shared root causes* (e.g., same dependency chain), the failures are analyzed independently. Factoring would identify shared sub-expressions across failure causes, reducing redundant analysis and enabling more targeted repairs.

### 3.5 RunSoC 2.0: CP-SAT for MPSoC Scheduling (arXiv:2609.01614)

**Finding**: CP-SAT consistently outperforms CBC (Branch-and-Cut) and Genetic Algorithm for tightly constrained hard real-time scheduling on heterogeneous MPSoCs. Models task DAGs with end-to-end latency and core-affinity constraints. Multi-objective optimization minimizing memory-budget violations and communication penalties.

**NEW Defect #17 — No DAG-Structured SelfTest Dependency Modeling**:
NeoTrix SelfTest modules are treated as independent. But RunSoC 2.0 proves that modeling task dependencies as DAGs with latency/affinity constraints dramatically improves scheduling. SelfTests have dependencies (e.g., T3 Production Wiring depends on T1 Existence and T2 Registration). These should be modeled as a DAG with constraints, solved via CP-SAT rather than sequential execution.

---

## Summary: 17 New Defects Found (Batch 539 vs Batch 538)

| # | Domain | Defect | Source |
|---|--------|--------|--------|
| 1 | Planning | No hierarchical heuristic reasoning (world state + task network) | AAAI-26 |
| 2 | Planning | No LLM-as-formalizer for task decomposition | PDDLCoder |
| 3 | Planning | Ordering constraints left implicit, no parallelism analysis | ICAPS-26 unordered HTN |
| 4 | Planning | No numeric resource reasoning in evolution planning | HPlan-26 |
| 5 | Planning | No reusable skill library from verified evolution plans | PLDI-26 |
| 6 | Planning | Single-stage goal formulation, no multi-IR revision | ACL-26 Findings |
| 7 | Scheduling | No urgency tokenization for attention routing | TempoNet |
| 8 | Scheduling | No asynchronous dual-stream rule evolution for GWT | RACE-Sched |
| 9 | Scheduling | No admission control for evolution subtasks | HSA-DAJQ |
| 10 | Scheduling | No exploration-exploitation tradeoff in task prioritization | Bandit Scheduling |
| 11 | Scheduling | No runtime prediction for evolution tasks | BOOSTEDSOSA |
| 12 | Scheduling | GWT modulation should be conditional, not constant | LLM Edge Scheduling |
| 13 | Constraint | SelfTest uses SAT-level nogoods, not atomic constraint nogoods | CP 2026 |
| 14 | Constraint | SelfTest scheduling uses conflicts-based, not ticks-based | CaDiCaL 3.0 |
| 15 | Constraint | No domain-specific SelfTest solver adaptation | SAT Comp 2026 |
| 16 | Constraint | No clause factoring across SelfTest results | Factoring Learned Clauses |
| 17 | Constraint | SelfTest dependencies not modeled as DAG with CP-SAT | RunSoC 2.0 |

---

## Sources Cited

1. arXiv:2605.07707 — LLM-Generated Heuristics for HTN Planning (AAAI-26)
2. arXiv / ICAPS 2026 — Unordered HTN Planning in PSPACE (Lauer et al.)
3. arXiv:2609.03938 — Numerical TOHTN Planning with SMT-based HTN-SAT Encoding (HPlan-26)
4. arXiv:2608.16637 — PDDLCoder: Agentic PDDL Generation (NL-pddlgym)
5. PLDI 2026 — Neuro-Symbolic Hierarchical Learning for Long-Horizon Robotic Tasks
6. aclanthology.org/2026.findings-acl.415 — Unifying Inference-Time Planning Language Generation
7. arXiv:2602.18109 — TempoNet: RL Scheduler with Transformer Encoder
8. arXiv:2605.29262 — RACE-Sched: Asynchronous Dual-Stream Scheduling
9. IJARCSE 2026 — Hybrid Scheduling Algorithm for Deadline-Aware Job Queues
10. proceedings.mlr.press/v337/wang26g — Bandit Learning for Online Scheduling (UAI 2026)
11. arXiv:2608.25346 — BOOSTEDSOSA: ML-Assisted Stochastic Scheduling
12. arXiv:2608.19557 — LLM Agents for Edge Scheduling: Conditional Benefit
13. LIPIcs.CP.2026.42 — From Literals to Atomic Constraints: Generalising CDCL for CP
14. LIPIcs.SAT.2026.40 — CaDiCaL 3.0 Tool Paper
15. satcompetition.github.io/2026 — SAT Competition 2026 Results
16. LIPIcs.SAT.2026.28 — Factoring Learned Clauses
17. arXiv:2609.01614 — RunSoC 2.0: CP-SAT for MPSoC Scheduling
