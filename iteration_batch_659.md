# Iteration Batch 659 — Model Checking / Theorem Proving / Formal Methods Survey

## Purpose
Survey 2026 developments in model checking, theorem proving, and formal methods to identify defects/improvements for NeoTrix consciousness architecture.

---

## 1. MODEL CHECKING

### 1.1 State Space Estimation for DPOR-Based Model Checkers (PLDI 2026)
- **Source**: https://doi.org/10.1145/3808291 (Balasubramanian et al., Jun 2026)
- **What's NEW**: First provable polynomial-time unbiased estimators for counting Mazurkiewicz trace-equivalence classes. Converts stateless optimal DPOR algorithm into Monte Carlo estimator. Achieves stable estimates within 20% band for 10^5–10^6 class state spaces in few hundred trials.
- **Defect for NeoTrix**: **Defect MC-1: No state space estimation for SEAL pipeline exploration.** When SEAL runs explore→distill→self-test→absorb cycles, the concurrent exploration of candidate architectures has an analogous state space explosion. We have no estimator for how many architectural variants a SEAL cycle explores. Need Monte Carlo state space estimator for SEAL's concurrent search tree.

### 1.2 Spatio-temporal Model Checking with VoxLogicA (2026)
- **Source**: https://link.springer.com/article/10.1007/s10849-026-09476-w
- **What's NEW**: SLCS (Spatial Logic for Closure Spaces) extended with temporal operators for linear-time finite traces. VoxLogicA-2 tool rewrites spatio-temporal specs into purely spatial ones, enabling temporal verification without modifying model checker core. Pac-Man demonstration.
- **Defect for NeoTrix**: **Defect MC-2: No spatio-temporal logic for ConsciousnessTree state evolution.** ConsciousnessTree's 6-stage loop (Soil→Roots→Trunk→Branches→Fruits→Core) evolves over time with spatial health signals across 11 branches. We have no spatio-temporal logic to express properties like "branch B_j remains healthy for K consecutive cycles" or "after module M degrades, other modules compensate within T cycles." Need temporal extension of our GWT attention properties.

### 1.3 Specification-Guided Path Shortcutting for Probabilistic Model Checking (Sep 2026)
- **Source**: https://arxiv.org/abs/2609.02457
- **What's NEW**: Replaces transition sequences in Markov chains with single transitions when property is fixed, reducing state space without changing satisfaction probability. Outperforms Storm baseline on complex specifications.
- **Defect for NeoTrix**: **Defect MC-3: No path compression for E8 hexagram reasoning chains.** When E8 traverses reasoning paths through hexagrams, many intermediate reasoning steps are semantically redundant. No mechanism to compress multi-step reasoning chains into equivalent shorter chains while preserving conclusion validity.

### 1.4 iSMC: Self-Certifying BDD-based Model Checker (2026)
- **Source**: https://arxiv.org/html/2605.03705v1
- **What's NEW**: First model checker with interactive certification. After solving CTL instance, Prover/Verifier engage in TraceCert protocol over finite field F_p. Verifier runs in O(n²·l) polynomial time vs exponential for prior certificate approaches. Error probability provably tiny.
- **Defect for NeoTrix**: **Defect MC-4: No self-certification for KB consistency checks.** When NT-MEMORY performs KB consistency verification, results are trusted without independent certification. Need interactive proof system (Prover/Verifier architecture) where KB assertions are independently certifiable without re-executing the full verification.

### 1.5 Model Checking Contest 2026 Results
- **Source**: https://mcc.lip6.fr/2026/
- **What's NEW**: 218,736 new formulas added in 2026. BVT-2026 and 2025-gold achieve 100% confidence across all categories (StateSpace, Reachability, CTL, LTL, UpperBounds). 147 models, 1953 instances, 25,389 queries.
- **Defect for NeoTrix**: **Defect MC-5: No formal contest for consciousness architecture verification.** NeoTrix has no standardized benchmark suite for verifying properties of its 6-layer architecture. Need a "MCC for consciousness" — standardized properties (safety, liveness, fairness) that any consciousness architecture must satisfy.

### 1.6 HyperQB 2.0: Bounded Model Checker for Hyperproperties
- **Source**: https://link.springer.com/chapter/10.1007/978-3-032-32519-8_26
- **What's NEW**: First efficient push-button BMC for hyperproperties (HyperLTL, A-HLTL). Supports both synchronous and asynchronous trace quantification. Rust implementation. Handles information-flow security, linearizability, path planning. Loop conditions for completeness.
- **Defect for NeoTrix**: **Defect MC-6: No hyperproperty verification for multi-agent NeoTrix subsystems.** NeoTrix's 7 domains (NT-CORE through NT-FEEL) execute concurrent traces. Hyperproperties like "no domain leaks information to untrusted domains" (non-interference) or "all domains converge to consistent state" (linearizability) are unverifiable. Need HyperLTL-style trace quantification across domain execution traces.

---

## 2. THEOREM PROVING

### 2.1 ITPEval: Cross-Prover Translation Benchmark
- **Source**: https://leandojo.org/itpeval.html
- **What's NEW**: First benchmark for automated formal proof translation across 4 provers (Lean 4, Rocq/Coq, Isabelle/HOL, HOL Light). 390 aligned files, 1700+ theorems. Execution-based scoring: translation succeeds only if target prover accepts the file.
- **Defect for NeoTrix**: **Defect TP-1: No cross-paradigm knowledge translation.** When NeoTrix absorbs external knowledge (e.g., military ops doctrine → NT-ACT capabilities), there is no formal translation verification. Need execution-based translation validation: absorb knowledge, check it compiles/works in target domain namespace.

### 2.2 PROOFGYM: Unified Multi-Prover Backend
- **Source**: https://openreview.net/pdf?id=RrSQxcg6Nu
- **What's NEW**: Lightweight async backend unifying Lean 4, Coq, Isabelle behind single Python API. 7.4× speedup over Ray-parallelized baseline on whole-proof verification. 31.2× speedup on Coq GeoCoq. Non-blocking batched execution with bounded concurrency. Structured logs for dataset curation.
- **Defect for NeoTrix**: **Defect TP-2: No unified multi-paradigm execution backend.** NeoTrix's SEAL pipeline runs multiple evolution strategies (exploration, distillation, self-test) but each has separate execution management. Need unified async backend with bounded concurrency across SEAL phases, analogous to PROOFGYM's unified prover access.

### 2.3 MerLean-Prover: Recursive Looping Harness (2026)
- **Source**: https://arxiv.org/pdf/2605.26959
- **What's NEW**: End-to-end Lean 4 prover replacing sorry with kernel-checkable proofs. Three agent types (Planning, Check, Lean) composed by recursive outer loop. 10/23 on FormalQualBench (PhD-qualifying), 12/12 on Putnam 2025. No fine-tuning, no custom RL. Proof plan as unit of revision. Transfers to smaller models (Sonnet/Haiku).
- **Defect for NeoTrix**: **Defect TP-3: No recursive plan-revision loop for SEAL evolution.** SEAL currently runs linear phases without recursive plan revision. MerLean's key insight — "the proof plan itself is the unit of revision, not individual tactics" — maps directly: SEAL should revise the evolution plan (not individual module patches) when self-test fails. Need recursive plan-diff mechanism.

### 2.4 OpenProver: Agentic Interactive Theorem Proving (Jul 2026)
- **Source**: https://www.alphaxiv.org/abs/2607.09217
- **What's NEW**: Planner-Worker-Verifier architecture. Whiteboard scratchpad + Repository of intermediate findings. Workers explore independently, Verifiers check without seeing Worker reasoning (reducing bias). lean_verify, lean_search, lean_store tool calls. Interactive TUI for human steering.
- **Defect for NeoTrix**: **Defect TP-4: No bias-free verification in NT-MIND distillation.** When NT-MIND distills findings, it verifies results with full access to the distillation context (bias toward confirming). Need independent Verifier agents that see only the distilled output, not the reasoning trace, to catch confirmation bias.

### 2.5 Anthropic Formalizes Fermat's Last Theorem (Sep 2026)
- **Source**: https://www.anthropic.com/research/formalizing-fermats-last-theorem
- **What's NEW**: Claude worked autonomously for 11 days, producing 13M lines of Lean, 29,500 intermediate theorems. Used Prove2Me platform for DAG-based theorem management. Multiple agents collaborated in parallel. First end-to-end computer-checked proof of FLT.
- **Defect for NeoTrix**: **Defect TP-5: No DAG-based task decomposition for large-scale NT evolution.** Claude's success on FLT relied on Prove2Me's DAG of theorem statements for parallel agent coordination. SEAL currently has no DAG-based decomposition of evolution goals — all modules evolve independently. Need DAG of evolution dependencies for parallel agent coordination.

### 2.6 TheoremBench: Beyond Competition-Style Proving (Jun 2026)
- **Source**: https://arxiv.org/html/2606.09450v1
- **What's NEW**: Lean4 benchmark based on classical theorems (not competitions). Premised version exposes subtheorems as explicit instances. Introduces theorem-level coverage and token-efficiency metrics. Shows current provers biased toward easy subtheorems with verbose proofs.
- **Defect for NeoTrix**: **Defect TP-6: No coverage/efficiency metrics for SEAL evolution.** SEAL reports success/failure but no theorem-level coverage (how many intermediate evolution goals completed) or token-efficiency (how many LLM tokens per evolution step). Need diagnostic metrics beyond pass/fail.

---

## 3. FORMAL METHODS

### 3.1 VeriSpecGen: Traceable Refinement for Spec Synthesis (2026)
- **Source**: https://arxiv.org/pdf/2604.10392
- **What's NEW**: Decomposes NL requirements into atomic requirements, generates requirement-targeted tests with traceability maps. When validation fails, maps failures to specific requirements for localized repair. 86.6% on Verina SpecGen (SOTA). Generates 343K training examples from refinement trajectories. 62-106% improvement from trajectory training.
- **Defect for NeoTrix**: **Defect FM-1: No traceable refinement for NT skill absorption.** When absorbing external skills, NeoTrix lacks requirement-level traceability from source capability → NT domain mapping → verification. Need VeriSpecGen-style traceability maps: each absorbed capability maps to specific source requirements, with targeted tests per requirement.

### 3.2 SpecSyn: Specification Synthesis via Mutation-Based Refinement (Apr 2026)
- **Source**: https://www.alphaxiv.org/abs/2604.21570
- **What's NEW**: Variant Discriminative Rate (VDR): measures specification strength by testing against 188 mutation operators. Specifications that fail to distinguish semantically-different programs are refined. 96.68% precision, 75.91% recall. Handles 1071/1365 real-world verification targets.
- **Defect for NeoTrix**: **Defect FM-2: No mutation-based fitness for SEAL exploration.** SEAL evaluates architectural variants but has no mutation-based fitness scoring. Need VDR analogue: mutate candidate architectures, check if current fitness function distinguishes good from bad variants. If fitness doesn't discriminate, the fitness function itself needs strengthening.

### 3.3 CRIS: Imaginary Specifications for Hybrid Verification (PLDI 2026)
- **Source**: https://doi.org/10.1145/3808317
- **What's NEW**: "Imaginary specifications" freely mix executable code with ownership assertions. Generalizes CCR (refinement + separation logic) for hybrid verification — mix of formal, tested, and model-checked code. Complete Rocq mechanization.
- **Defect for NeoTrix**: **Defect FM-3: No hybrid verification strategy for NeoTrix modules.** NeoTrix modules have varying verification levels (C0=compiles → C5=self-healing) but no formal framework for reasoning about mixed-verification-level compositions. Need CRIS-style imaginary specifications that mix verified and unverified modules with ownership tracking.

### 3.4 Expecto: Top-Down Spec Synthesis with Tree Search (PLDI 2026)
- **Source**: https://prosys.kaist.ac.kr/publications/pldi26.pdf
- **What's NEW**: Top-down specification synthesis: starts from trivially complete spec, iteratively adds constraints. Tree search explores multiple refinement paths simultaneously. 94.5% reduction in wrong specs vs monolithic generation. Tree search adds 19.8-37.2% more sound-and-complete specs.
- **Defect for NeoTrix**: **Defect FM-4: No top-down specification refinement for NT architecture specs.** NT's 6-layer architecture is specified informally. Need Expecto-style top-down formal spec synthesis: start from trivially complete architecture spec, iteratively constrain with actual implementation details, tree-search multiple refinement paths.

### 3.5 Preguss: RTE-Guided Spec Synthesis for 1000+ LoC Programs (OOPSLA 2026)
- **Source**: https://fiction-zju.github.io/papers/OOPSLA2026.pdf
- **What's NEW**: First automated method proving RTE-freeness of 1000+ LoC real programs. Divide-and-conquer: static analyzer emits RTE assertions → V-Units prioritized → LLM infers interprocedural specs per unit. 80.6-88.9% reduction in human verification effort. Found 6 confirmed RTEs in spacecraft control system (1280 LoC).
- **Defect for NeoTrix**: **Defect FM-5: No potential-error-guided verification for NT codebase.** NeoTrix has thousands of LoC but no automated RTE-guided verification. Need Preguss-style pipeline: static analyzer → guard assertions → V-Unit decomposition → LLM spec synthesis per unit. This would systematically find potential runtime errors in NT's Rust codebase.

### 3.6 SpecLoop: RTL-to-Specification with Formal Verification Feedback (Mar 2026)
- **Source**: https://www.alphaxiv.org/abs/2603.02895
- **What's NEW**: Dual-loop architecture: outer spec generation loop + inner verification loop. RTL Reconstructor translates spec back to RTL, formal equivalence checker validates. Counterexamples fed back for targeted refinement. Information hiding prevents reconstructor from seeing original RTL.
- **Defect for NeoTrix**: **Defect FM-6: No round-trip specification verification for NT modules.** When NT modules are written, their specifications are not verified by round-trip (spec → reconstruction → equivalence check). Need SpecLoop-style verification: generate spec from module, reconstruct module from spec, check equivalence.

### 3.7 PLEX: Normalization for Refinement Types (OOPSLA 2026)
- **Source**: https://webspace.science.uu.nl/~swier004/publications/2026-oopsla.pdf
- **What's NEW**: Extends Liquid Haskell's PLE algorithm with higher-order reasoning and unification. Handles dependent pattern matching. Sound and terminating. Verifies programs requiring equivalence and unification without manual proofs or functional extensionality axioms.
- **Defect for NeoTrix**: **Defect FM-7: No refinement type automation for NT's Rust type system.** NeoTrix uses Rust's type system extensively but has no automated refinement-type-like verification. PLEX-style automation could verify complex invariants in NT's type-level reasoning (e.g., GWT attention routing invariants, E8 hexagram consistency).

---

## SUMMARY: 14 NEW DEFECTS IDENTIFIED

| ID | Domain | Defect | Priority |
|---|---|---|---|
| MC-1 | Model Checking | No state space estimation for SEAL exploration | P2 |
| MC-2 | Model Checking | No spatio-temporal logic for ConsciousnessTree | P1 |
| MC-3 | Model Checking | No path compression for E8 reasoning chains | P3 |
| MC-4 | Model Checking | No self-certification for KB consistency | P2 |
| MC-5 | Model Checking | No formal benchmark for consciousness verification | P3 |
| MC-6 | Model Checking | No hyperproperty verification across domains | P2 |
| TP-1 | Theorem Proving | No cross-paradigm knowledge translation validation | P1 |
| TP-2 | Theorem Proving | No unified multi-paradigm execution backend | P2 |
| TP-3 | Theorem Proving | No recursive plan-revision loop for SEAL | P1 |
| TP-4 | Theorem Proving | No bias-free verification in NT-MIND distillation | P2 |
| TP-5 | Theorem Proving | No DAG-based task decomposition for evolution | P2 |
| TP-6 | Theorem Proving | No coverage/efficiency metrics for SEAL | P2 |
| FM-1 | Formal Methods | No traceable refinement for skill absorption | P1 |
| FM-2 | Formal Methods | No mutation-based fitness for SEAL exploration | P1 |
| FM-3 | Formal Methods | No hybrid verification strategy for modules | P2 |
| FM-4 | Formal Methods | No top-down spec refinement for architecture | P2 |
| FM-5 | Formal Methods | No RTE-guided verification for NT codebase | P1 |
| FM-6 | Formal Methods | No round-trip spec verification | P3 |
| FM-7 | Formal Methods | No refinement type automation for Rust | P3 |

## TOP 5 HIGHEST-IMPACT DEFECTS

1. **MC-2 + TP-3 + FM-2**: Combined — SEAL needs spatio-temporal property verification, recursive plan-revision, and mutation-based fitness. These three together would make SEAL self-certifying.
2. **FM-5**: Preguss-style RTE-guided verification would systematically find bugs in NeoTrix's Rust codebase (1000+ LoC).
3. **TP-1 + FM-1**: Cross-paradigm translation validation + traceable refinement for skill absorption would make NT-MEMORY's knowledge ingestion formally verified.
4. **MC-6**: Hyperproperty verification across 7 domains would catch information leaks and convergence failures.
5. **TP-5**: DAG-based evolution decomposition would enable parallel agent coordination like Claude's FLT formalization.

## SOURCES CITED

1. Balasubramanian et al. "State Space Estimation for DPOR-Based Model Checkers." PLDI 2026. https://doi.org/10.1145/3808291
2. Ciancia et al. "Spatio-temporal Model Checking with VoxLogicA." Springer 2026. https://link.springer.com/article/10.1007/s10849-026-09476-w
3. arXiv:2609.02457 "Specification-Guided Path Shortcutting for Probabilistic Model Checking." Sep 2026.
4. iSMC: BDD-based Symbolic Model Checker with Interactive Certification. arXiv:2605.03705v1, 2026.
5. MCC 2026 Results. https://mcc.lip6.fr/2026/
6. HyperQB 2.0. Springer 2026. https://link.springer.com/chapter/10.1007/978-3-032-32519-8_26
7. ITPEval. https://leandojo.org/itpeval.html
8. PROOFGYM. OpenReview 2026. https://openreview.net/pdf?id=RrSQxcg6Nu
9. MerLean-Prover. arXiv:2605.26959, May 2026.
10. OpenProver. arXiv:2607.09217, Jul 2026. https://www.alphaxiv.org/abs/2607.09217
11. Anthropic. "Formalizing Fermat's Last Theorem." Sep 2026. https://www.anthropic.com/research/formalizing-fermats-last-theorem
12. TheoremBench. arXiv:2606.09450v1, Jun 2026.
13. VeriSpecGen. arXiv:2604.10392, 2026.
14. SpecSyn. arXiv:2604.21570, Apr 2026. https://www.alphaxiv.org/abs/2604.21570
15. CRIS. PLDI 2026. https://doi.org/10.1145/3808317
16. Expecto. PLDI 2026. https://prosys.kaist.ac.kr/publications/pldi26.pdf
17. Preguss. OOPSLA 2026. https://fiction-zju.github.io/papers/OOPSLA2026.pdf
18. SpecLoop. arXiv:2603.02895, Mar 2026. https://www.alphaxiv.org/abs/2603.02895
19. PLEX. OOPSLA 2026. https://webspace.science.uu.nl/~swier004/publications/2026-oopsla.pdf
20. Rocq Prover 9.2.0. https://rocq-prover.org/
21. Lean 4 & Proof Assistants 2026 Deep Dive. https://www.youngju.dev/blog/culture/2026-05-16-lean-4-proof-assistants-2026-mathlib-rocq-agda-isabelle-fstar-alphaproof-aimo-deep-dive.en
