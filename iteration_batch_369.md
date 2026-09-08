# Iteration Batch 369 — External Research + Defect Identification

**Date**: 2026-09-06
**Research Areas**: Code Generation, Code Review, Software Testing
**Method**: Web search for 2026 advances → cross-reference NeoTrix design docs → defect extraction

---

## Sources Cited

### Code Generation
1. **SpecAgent** (ACL 2026) — Speculative retrieval for code completion. Pre-computes context at indexing time, achieves 9–11% absolute gains over baselines. [aclanthology.org/2026.acl-long.786.pdf]
2. **SolidCoder** (ACL 2026) — Bridges "Mental Reality Gap" via sandboxed execution + property-based oracles. 95.7% HumanEval pass@1. [aclanthology.org/2026.findings-acl.361.pdf]
3. **SeDev** (ACL 2026) — Structured semantic exploration; avoids local optima via semantically adjacent solution traversal. [aclanthology.org/2026.acl-long.1641]
4. **Hydra** (arXiv:2602.11671) — Dependency-aware retriever for repo-level code gen; structure-aware indexing + call-graph DAR. New SOTA on RepoExec/DevEval.
5. **ACToR** (arXiv:2609.01601, Sep 2026) — Adaptive critical-token-aware retrieval; identifies decisive generation positions and triggers on-demand retrieval. 8.4–15.4% relative gains.
6. **Zero-Shot Self-Orchestration** (arXiv:2608.26480) — Manager-worker scaffold for coding; +23–42 points on LiveCodeBench for smaller models. Opus-5 achieves 91%.
7. **CodeFlowBench** (ACL 2026) — Multi-turn codeflow benchmark; reveals up to 50% performance degradation vs single-turn, inversely correlated with dependency complexity.
8. **Springer Survey** (Apr 2026) — Comprehensive survey of LLM code generation; identifies critical gaps in repo-level context, consistency across extended sessions, and correctness verification.

### Code Review
9. **SonarQube Hunter Agent GA** (Aug 2026) — AI security agent for logic flaws (broken access control, business logic). 80–90% average precision. Finds 200+ zero-days in open source. [sonarsource.com/blog]
10. **AgenticSCR** (arXiv:2601.19138v2) — Agentic secure code review with security-focused semantic memory. 153% improvement over static LLM baseline. 54% validated by security engineers.
11. **OpenCodeReview** (arXiv:2608.09290) — Deterministic engineering for agent-based review. Rule-Guided Dispatch + Independent Reflection. 2.17× SEM-F1 vs Claude Code, 5–15× fewer tokens.
12. **Qodo 2.0** (Feb 2026) — Multi-agent expert review architecture. 60.1% F1, highest recall (56.7%) on production PR benchmark. PR history as first-class context.
13. **SonarQube Server 2026.4** — AI-specific quality gate ("Sonar way for Agentic AI"), agentic-security rules (CLI injection, MCP risks, AI data leaks). Incremental taint analysis 90% faster.
14. **Arm Metis** (GitHub) — Open-source AI security code review. CodeGraph reachability for C/C++, language plugins for 10+ languages. Deterministic evidence collection.
15. **OpenAnt** (arXiv:2606.19149v2) — Vulnerability discovery via code decomposition + adversarial verification + sandboxed exploit execution. 97% analysis surface reduction.

### Software Testing
16. **PROBE** (ACL 2026) — Adversarial refinement for property-based testing. Validator generates counter-implementations to expose weak properties. 9.79% mutation score improvement. 45 unknown bugs found in CPython/scipy/cryptography.
17. **SWE-Mutation** (ACL 2026) — Benchmark exposing LLM test suite inadequacy. DeepSeek-V3.1 achieves only 10.20% verification / 36.15% detection rates on mutated variants.
18. **CDBench** (Springer, Jun 2026) — Zero-sum mutation testing game. Gemini 2.5 Pro best as defender. Reasoning models weakest at mutant generation (instruction-following failures).
19. **AdverTest** (arXiv:2602.08146) — Adversarial dual-agent test generation. +8.56% fault detection over best LLM method, +63.30% over EvoSuite on Defects4J.
20. **LLMutantKiller** (ISSTA 2026) — Feedback-directed LLM test generation for killing surviving mutants. 95.3% kill rate on behavioral mutants in JS/TS.
21. **Ockhamareto** (arXiv:2608.24473) — Pareto-gated RL for concise test generation. Fewer tests, higher mutation score. Strict Pareto-dominance over MIST-RL.
22. **Spec-Driven Test Generation** (arXiv:2608.17177) — Design-by-Contract scaffold for test gen. +9.8pp bug detection, +2.5pp branch coverage on Google production bugs.
23. **OutSight/Industry Report** (2026) — AI-generated tests score 40–55% mutation out-of-box; mutation-feedback loop pushes to 70–85%. Atlassian confirms similar gains.

---

## Defects Found in NeoTrix Design

### DEFECT-CG-01: No Speculative Context Pre-computation for Code Generation

**Source**: SpecAgent (ACL 2026), Hydra (arXiv:2602.11671), ACToR (arXiv:2609.01601)

**Evidence**: NeoTrix's `nt_mind::code_review::CodeReviewEngine` (`neotrix-core/src/unified/layers/cognition/nt_mind/infrastructure/code_review.rs`) operates reactively — it reviews code at invocation time. There is no indexing-time or background pre-computation of repository context. The `CapabilityBridge` (`nt_core_capability_tree/src/bridge.rs`) maps evolution→runtime but does not pre-compute speculative context blocks.

**Gap**: 2026 research shows that shifting context construction from inference-time to indexing-time (SpecAgent) yields 9–11% absolute gains with zero additional latency. Dependency-aware retrieval (Hydra) using call-graph analysis and critical-token identification (ACToR) further improves repo-level generation by 8–15%. NeoTrix has no equivalent mechanism — its code generation tools generate from scratch each time without pre-computed cross-file dependency context.

**Suggestion**: Implement a `SpeculativeContextIndexer` that runs as a background SEAL phase, pre-computing dependency blocks for each module. Store in KB namespace `code_context`. Wire into NT-ACT's code generation pipeline so generation queries hit pre-computed context instead of raw repository scan.

---

### DEFECT-CG-02: No Multi-Turn Codeflow Evaluation or Dependency-Aware Generation

**Source**: CodeFlowBench (ACL 2026), Zero-Shot Self-Orchestration (arXiv:2608.26480)

**Evidence**: CodeFlowBench reveals up to 50% performance degradation in multi-turn code generation vs single-turn, with degradation inversely correlated with dependency complexity. The Self-Orchestration paper shows manager-worker scaffolding yields +23–42 points for smaller models.

**Gap**: NeoTrix's SEAL pipeline processes code generation as single-pass. The `SelfIteratingBrain` (`seal_loop.rs:181`) does `code_review_iterate` but does not model multi-turn dependency chains or function-level codeflow. The benchmark gap means NeoTrix cannot evaluate whether its code generation maintains consistency across iterative function implementations.

**Suggestion**: Add a `CodeflowEvaluator` that decomposes tasks into dependency-aware subproblems (AST-based topological sort) and evaluates multi-turn generation with structural metrics (APD@k, DSC from CodeFlowBench). Integrate manager-worker orchestration pattern from the self-orchestration paper for complex multi-file generation tasks.

---

### DEFECT-CG-03: No Execution-Grounded Self-Verification for Generated Code

**Source**: SolidCoder (ACL 2026), SeDev (ACL 2026)

**Evidence**: SolidCoder identifies the "Mental Reality Gap" — LLMs hallucinate execution traces and validate buggy code. Their solution: sandboxed execution + property-based oracles achieves 95.7% HumanEval. SeDev shows single-path generation gets stuck in local optima; structured semantic exploration escapes them.

**Gap**: NeoTrix generates code but has no mechanism to verify it via concrete execution. The `SimulateEngine` (`nt_core_simulate_engine.rs`) exists but is a simulation engine, not a sandboxed executor for generated code. The `verification_status` field in `nt_capability_bridge.rs:58` is always `None` (confirmed at lines 416, 427, 449, 473, 497, 518, 547).

**Suggestion**: Implement a `LiveExecutionVerifier` (modeled on SolidCoder's S.O.L.I.D. architecture) that: (1) runs generated code in a sandboxed Rust/WASM runtime, (2) generates property-based assertions via oracle agent, (3) accumulates regression tests (Defensive Accumulation pattern). Wire as SEAL Phase-4 gate before code is accepted.

---

### DEFECT-CR-01: No Logic-Flaw Detection in Code Review Pipeline

**Source**: SonarQube Hunter Agent GA (Aug 2026), AgenticSCR (arXiv:2601.19138v2)

**Evidence**: Hunter Agent finds 200+ zero-days by detecting broken access control, business logic flaws, and authentication flaws — categories SAST structurally cannot detect. OWASP 2025: 100% of applications tested had broken access control. AgenticSCR achieves 153% improvement via security-focused semantic memory.

**Gap**: NeoTrix's `CodeReviewEngine` (`code_review.rs:50`) defines 6 review dimensions (correctness/performance/security/style/maintainability/documentation) but these are pattern-based. No logic-flaw detection exists — the engine cannot reason about whether an authorization check is actually enforced server-side, whether a workflow step is skippable, or whether an IDOR vulnerability exists. The `rev-officer-agent` covers architecture-level review but not application-level business logic verification.

**Suggestion**: Implement a `LogicFlawDetector` subagent (modeled on Hunter Agent's Analyze→Explore→Validate→Synthesize pipeline) that: (1) maps entry points via CodeGraph reachability, (2) traces authorization checks through business logic, (3) validates exploitability via constrained attacker simulation. Register as NT-SHIELD detection module (Rev-星 skill domain).

---

### DEFECT-CR-02: No Deterministic Review Pipeline — Non-Determinism Not Controlled

**Source**: OpenCodeReview (arXiv:2608.09290), Qodo 2.0 (Feb 2026)

**Evidence**: OpenCodeReview demonstrates that injecting determinism at three points (Rule-Guided Dispatch, Grounded File Review, Independent Reflection) achieves 2.17× SEM-F1 while consuming 5–15× fewer tokens vs Claude Code. Qodo 2.0 shows multi-agent expert architecture with judge agent achieves 60.1% F1.

**Gap**: NeoTrix's code review uses a single `CodeReviewEngine` without deterministic dispatch. The review process is non-deterministic — tool use, file selection, and context retrieval are unbounded. The `BlastRadiusIndex` (dev-rules R-P85) provides explosion radius but not deterministic file triage or curated tool sets. No independent reflection/falsification filter exists to remove hallucinated review comments.

**Suggestion**: Refactor code review into a three-phase deterministic pipeline: (1) Rule-Guided Dispatch using file-type-specific rule sets (ported from OpenCodeReview's template-engine approach), (2) Grounded File Review with curated bounded-output tool set per file, (3) Independent Reflection module with asymmetric information boundary (reflector sees only diff, not tool-augmented context). This eliminates non-determinism while improving depth.

---

### DEFECT-CR-03: No Agentic Security Memory for Early-Stage Vulnerability Detection

**Source**: AgenticSCR (arXiv:2601.19138v2), SonarQube 2026.4

**Evidence**: AgenticSCR augments detector/validator with SAST-rule-derived semantic memory + CWE validation framework. SonarQube 2026.4 introduces agentic-security rules covering CLI injection, MCP-based risks, and AI data leaks — threat classes specific to agent-generated code.

**Gap**: NeoTrix's `nt_shield` (StealthNet, proxy pool, Tor client) focuses on network-level security. There is no application-level security review memory. The KB has `domain_nt_shield` namespace but no CWE-indexed semantic memory for code review. The `ConstitutionComplianceTest` (`nt_core_self_test`) checks architectural rules but not security patterns. AI-generated code (42%+ of enterprise code per SonarQube) introduces MCP-specific injection surfaces and data leak risks that NeoTrix has no detection rules for.

**Suggestion**: Create a `SecuritySemanticMemory` in KB namespace `security_memory` indexed by CWE IDs. Populate from SAST rule databases (CodeQL, Semgrep patterns). Wire into `CodeReviewEngine` as a lookup during review — when a file is reviewed, relevant CWE rules are retrieved and used as validation constraints. Add agentic-specific rules: MCP tool injection, CLI argument injection, AI mechanism data leaks.

---

### DEFECT-ST-01: No Mutation Testing Integration — SelfTest Does Not Measure Discriminative Power

**Source**: SWE-Mutation (ACL 2026), CDBench (Springer 2026), AdverTest (arXiv:2602.08146)

**Evidence**: SWE-Mutation reveals that even DeepSeek-V3.1 achieves only 10.20% verification rate on mutated variants. CDBench shows LLM-generated tests score 40–55% mutation out-of-box. AdverTest shows adversarial mutation feedback loop improves fault detection by 8.56%.

**Gap**: NeoTrix's `SelfTest` system (`nt_core_self_test`) checks T1 (existence), T2 (registration), T3 (production wiring) but has zero mutation testing capability. SelfTest results are binary pass/fail — they verify a detection module EXISTS and is REGISTERED but cannot verify it DETECTS actual faults. The `SelfTestResult` struct has `passed: bool` and `failures: Vec<String>` but no mutation score, no discriminative power metric.

**Suggestion**: Extend `SelfTest` with a T4 tier: **Mutation Score**. Implement a `MutationTester` that: (1) generates mutants of each detection module's target code using AST-level mutation operators, (2) runs the detection module's SelfTest against mutants, (3) computes mutation score (killed/total). T4 pass threshold: ≥70% mutation score. Integrate with `HeartbeatAggregator` to report mutation health as a system metric. Use `mutmut` (Python) or `cargo-mutants` (Rust) as the mutation engine.

---

### DEFECT-ST-02: No Property-Based Testing for Invariant Verification

**Source**: PROBE (ACL 2026), Spec-Driven Test Generation (arXiv:2608.17177)

**Evidence**: PROBE achieves 9.79% mutation score improvement via adversarial property generation and found 45 unknown bugs in CPython/scipy/cryptography. Spec-Driven Test Gen shows +9.8pp bug detection by first extracting code contracts (pre/post-conditions).

**Gap**: NeoTrix's test infrastructure (`cargo test`) uses example-based unit tests exclusively. No property-based testing exists. Critical invariants — HyperCube distance preservation, E8 lattice quantization correctness, VSA binding/unbinding round-trips, KB embedding cosine similarity bounds, GWT attention score monotonicity — are never tested across random inputs. The `QTestEngine` (`nt_core_qtest.rs`) does index-health self-tests but not property-based verification of mathematical invariants.

**Suggestion**: Add a `PropertyTestSuite` using `proptest` (Rust's property-based testing library) for each core mathematical module. Define properties: (1) HyperCube: `bind(unbind(x, y), y) ≈ x`, (2) E8: `quantize(dequantize(v)) ≈ v`, (3) VSA: `cosine_sim(bind(a, b), a) > cosine_sim(bind(a, b), c)` for random c, (4) KB: `search(encode(q))` returns q's nearest neighbors. Integrate with CI as a mandatory gate alongside unit tests.

---

### DEFECT-ST-03: No Adversarial Test Refinement Loop

**Source**: PROBE (ACL 2026), AdverTest (arXiv:2602.08146), LLMutantKiller (ISSTA 2026)

**Evidence**: PROBE's adversarial loop (Validator generates counter-implementations → Generator refines properties) achieves 95% property correctness vs 65% for standalone models. AdverTest's dual-agent loop (test agent ↔ mutant agent) improves fault detection by 8.56%. LLMutantKiller kills 95.3% of surviving mutants via feedback-directed re-prompting.

**Gap**: NeoTrix has no adversarial test refinement mechanism. When tests are generated (via NT-ACT or human), there is no agent that attempts to break them. The `SelfIteratingBrain` iterates on code edits but not on test quality. The SEAL pipeline's Phase-4 (self-test) validates existence, not discriminative power.

**Suggestion**: Implement a `TestRefinementArena` — a zero-sum game between a TestGenerator agent and a MutantGenerator agent (inspired by CDBench). The MutantGenerator creates targeted mutants that exploit blind spots in the current test suite; the TestGenerator refines tests to kill them. Run as a SEAL sub-phase. Track mutation score trajectory over iterations as a fitness signal for the evolution loop.

---

### DEFECT-ST-04: No Multi-Language Test Suite Reliability Evaluation

**Source**: SWE-Mutation (ACL 2026), CDBench (Springer 2026)

**Evidence**: SWE-Mutation includes multilingual subset spanning 9 programming languages. CDBench reveals reasoning models (DeepSeek-R1, QwQ) are weakest at mutant generation due to instruction-following failures, while Gemini 2.5 Pro excels at test generation.

**Gap**: NeoTrix is a Rust project but generates code and reviews code across languages (Python, TypeScript via NT-ACT tools). There is no cross-language test reliability benchmark. The `SelfTest` registry is Rust-only. When NT-ACT generates Python code, there is no equivalent T1-T3 verification in the target language.

**Suggestion**: Extend the SelfTest framework to support language-agnostic test verification. For non-Rust targets, use subprocess-based test execution with mutation testing. Create a `CrossLanguageTestVerifier` that: (1) detects target language, (2) runs appropriate mutation framework (mutmut for Python, Stryker for JS/TS, cargo-mutants for Rust), (3) reports unified mutation score to HeartbeatAggregator.

---

## Summary

| Category | Defects Found | Key Pattern |
|----------|--------------|-------------|
| Code Generation | 3 | No speculative context, no multi-turn evaluation, no execution grounding |
| Code Review | 3 | No logic-flaw detection, non-deterministic pipeline, no security semantic memory |
| Software Testing | 4 | No mutation testing, no property-based testing, no adversarial refinement, no cross-language verification |

**Total defects identified**: 10
**Highest priority**: DEFECT-ST-01 (mutation testing) and DEFECT-CR-01 (logic-flaw detection) — these represent fundamental gaps where 2026 research has established proven techniques that NeoTrix's design completely lacks.
