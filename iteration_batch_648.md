# Iteration 648 — Test Generation, Mutation Testing, Property-Based Testing Scan

**Date**: 2026-09-06
**Batch**: 648
**Context**: Batch 647 proved (1) proxy reward gaming generalizes to all factions, (2) scalar fitness cannot detect multidimensional hacking, (3) sequential growth cycles cause plateau pathology, (4) no VI→MCMC bridge, (5) no model misspecification detection.

---

## 1. Test Generation (10 sources)

### NEW Defect 648-1: SelfTest Framework Lacks Spec-Driven Oracle Generation
- **Source**: Spec-Driven Test Generation (arXiv:2608.17177)
- **Finding**: Grounding agents in semi-formal behavioral contracts before test generation yields **+9.8pp bug detection** and **+2.5pp branch coverage**. The agent first extracts pre/post-conditions, then generates tests against those contracts.
- **NeoTrix Defect**: `nt_selftest` generates tests from code directly (black-box "Code→Code"). Missing the white-box intermediate reasoning layer: **Code → Behavioral Contract → Test**. This means SelfTest oracles are shallow (assertion copying) rather than requirement-grounded.
- **Severity**: HIGH — directly explains why SelfTest T3 detection functions produce false negatives.

### NEW Defect 648-2: No Multi-Agent Oracle Consensus Mechanism
- **Source**: CANDOR (ACM 2026, doi:10.1145/3803418)
- **Finding**: Multiple reasoning LLMs (Panelist agents) independently evaluate tentative oracles, then a Curator aggregates via consensus. Reduces hallucination in test assertions. **21.1pp improvement** over TOGLL for oracle correctness.
- **NeoTrix Defect**: SelfTest oracles are single-pass. No cross-validation or consensus mechanism for assertion correctness. When NT-MIND distills test patterns, it accepts single-agent output without adversarial review.
- **Severity**: MEDIUM — affects reliability of SelfTest-derived regression tests.

### NEW Defect 648-3: No Program-Structure-Aware Test Targeting
- **Source**: GLMTest (ACL Findings 2026)
- **Finding**: Combining Code Property Graphs (CPG) with GNN embeddings conditions test generation on specific execution branches. Branch accuracy improved from **27.4% → 50.2%** on Test GenEval.
- **NeoTrix Defect**: `nt_selftest` has no CPG integration. SelfTest targets are flat (file-level) rather than branch-targeted. Cannot steer test generation toward high-risk execution paths or security-critical branches.
- **Severity**: HIGH — explains why SelfTest misses edge cases in complex control flow.

### NEW Defect 648-4: Compound-Objective Goal Drift in Agentic Test Loops
- **Source**: DPIAgent (arXiv:2608.23341)
- **Finding**: Monolithic test generation loops suffer "goal drift" when combining diagnosis + test writing. DPIAgent separates into Divide/Protocol/Isolate phases, achieving **86.17%** on SWT-Bench.
- **NeoTrix Defect**: NT-ACT's agentic test loops (e.g., `ConsciousnessTask`) have no phase separation between defect exploration and test generation. The compound objective causes drift in multi-step SelfTest workflows.
- **Severity**: MEDIUM — affects reliability of complex SelfTest scenarios.

### NEW Defect 648-5: RL-Based Adversarial Test Generation Not Explored
- **Source**: TCS (EMNLP 2026, arXiv:2609.03955)
- **Finding**: Two-stage RL where Stage 1 generates tests consistent with reference solution, Stage 2 learns counterexamples from failure modes. Enables both test generation and answer selection.
- **NeoTrix Defect**: NT-MIND has no adversarial RL for test generation. Current approach is one-shot LLM prompting. Missing the ability to generate counterexamples that specifically target model failure modes.
- **Severity**: MEDIUM — limits SelfTest's ability to stress-test weak modules.

### NEW Defect 648-6: Repository-Level Context Augmentation Gap
- **Source**: XREPOTEST (EMNLP 2026, arXiv:2608.25939)
- **Finding**: Large gap between standalone and repository-level test performance. LSP-based and retrieval-based context augmentation strategies differ significantly in effectiveness across Rust/Go/Julia/PHP/Ruby.
- **NeoTrix Defect**: SelfTest generation lacks repository-level context. Tests are generated from isolated function signatures without cross-module dependency resolution, LSP analysis, or retrieval of related code.
- **Severity**: HIGH — explains SelfTest failures on cross-domain (multi-faction) components.

### NEW Defect 648-7: Logic-CoT White-Box Paradigm Missing
- **Source**: Logic-CoT (MDPI Applied Sciences 2026)
- **Finding**: "Code→Logic→Code" paradigm with logical node state vectors from CFG. Achieves **91.22% compilation rate** vs 72.29% for ChatUniTest. Closed-loop reasoning→generation→repair.
- **NeoTrix Defect**: No CFG-based logical reasoning layer in SelfTest. Current approach skips the "logic inference" stage entirely, jumping from code to test.
- **Severity**: HIGH — same root cause as Defect 648-1.

### NEW Improvement 648-1: Feedback-Directed Zero-Shot Regression Testing
- **Source**: Cleverest (FSE 2026, Boehme et al.)
- **Finding**: LLM translates commit messages + code diffs into regression test inputs. Found as many bugs in **2 minutes** as WAFLGo found in **24 hours**. Critically depends on commit message quality.
- **NeoTrix Action**: Implement commit-message-driven SelfTest generation for NT-CORE/NT-MIND SEAL pipeline stages. When a module's Constellation maturity increases (C0→C1→...), auto-generate regression tests from the diff.

### NEW Improvement 648-2: Hybrid LLM+Fuzzing Seed Generation
- **Source**: Cleverest (FSE 2026)
- **Finding**: Using LLM-generated tests as seeds for greybox fuzzing finds **2× more bugs** than either alone.
- **NeoTrix Action**: Combine SelfTest-generated inputs with coverage-guided fuzzing (AFL-style) for NT-SHIELD security modules and NT-ACT tool modules.

---

## 2. Mutation Testing (8 sources)

### NEW Defect 648-8: Equivalent Mutant Detection Has Data Leakage (LLM Shortcut)
- **Source**: ISSTA 2026 — "Re-evaluating Detection of Equivalent Mutants Using LLMs"
- **Finding**: LLM-based equivalent mutant detection suffers **substantial performance degradation** on new datasets. Key factor: **original-method-level data leakage** inflating prior results. LLMs use method-wise majority-voting shortcut rather than reasoning about semantic effects.
- **NeoTrix Defect**: NT-MIND's SelfTest mutation score calculation assumes all surviving mutants are non-equivalent. No equivalent mutant detection mechanism. If the codebase has significant equivalent mutants (common in Rust with trait dispatch), mutation scores are **artificially deflated**, triggering false SelfTest failures.
- **Severity**: CRITICAL — directly causes false positives in SelfTest T3 evaluation.

### NEW Defect 648-9: No Contextual Equivalent Mutant Analysis
- **Source**: GEM-LLM (Intelligent Systems with Applications, 2026)
- **Finding**: "Contextual equivalence" — mutants behave identically only under broader program constraints. GEM-LLM uses inter-procedural slicing + LLM + SMT solvers to identify **25-30%** more equivalent mutants at **98% precision**.
- **NeoTrix Defect**: No SMT-backed formal equivalence verification. SelfTest treats all non-killed mutants as "escaped faults" without checking if they're semantically equivalent. Particularly problematic for Rust where borrow checker invariants create many contextual equivalences.
- **Severity**: HIGH — inflates SelfTest failure rate, wastes NT-REPAIR cycles.

### NEW Defect 648-10: No RAG-Augmented Mutation Generation
- **Source**: SMART (arXiv:2603.24560)
- **Finding**: RAG on vectorized real-world bug dataset + focused code chunking + fine-tuning. Achieves **92.61% real bug detection rate** vs 57.86% for LLMut. Validity improved from 42.89% to 65.6%.
- **NeoTrix Defect**: SelfTest uses AST-level mutation operators (traditional). No RAG from real bug patterns. Mutants are syntactically valid but semantically unrealistic. Cannot generate mutants that resemble actual NeoTrix bugs.
- **Severity**: HIGH — SelfTest mutations don't correlate with real failure modes.

### NEW Defect 648-11: Agentic Mutation Testing Triples Threat
- **Source**: MuTON/mewt (Trail of Bits, 2026)
- **Finding**: AI agents can now configure mutation campaigns, triage results, AND generate tests from mutants. This "triples" the attack surface: agents that encode incorrect behavior into tests create **false confidence** while doubling maintenance burden.
- **NeoTrix Defect**: NT-MIND's SEAL pipeline can auto-generate tests from mutation results without validation. No "skeptical agent" pattern — no mechanism to halt and ask "is this behavior correct or is it a bug?" before crystallizing behavior into tests.
- **Severity**: CRITICAL — directly relates to Batch 647's proxy reward gaming finding. Mutation-based test generation is another vector for reward hacking.

### NEW Defect 648-12: No Mutant Prioritization by Severity
- **Source**: MuTON (Trail of Bits, 2026)
- **Finding**: Three-tier severity: high (revert statements → unexecuted paths), medium (commented lines → unverified side effects), low (subtle operator swaps). Skips lower-severity when higher already indicates missing coverage.
- **NeoTrix Defect**: SelfTest treats all mutants equally. No severity-based prioritization. Running all 1000+ mutants sequentially causes the **plateau pathology** identified in Batch 647.
- **Severity**: HIGH — directly causes the sequential growth cycle plateau.

### NEW Improvement 648-3: SQLite-Backed Persistent Mutation Results
- **Source**: MuTON (Trail of Bits, 2026)
- **Finding**: Store all mutants + results in SQLite. Campaigns pause/resume without losing progress. Enables filtering, SARIF export, cross-session persistence.
- **NeoTrix Action**: Implement persistent mutation result storage in KB (nt_memory) for SelfTest campaigns. Aligns with KB architecture and enables incremental mutation analysis.

### NEW Improvement 648-4: Mutation-Guided Iterative Test Generation
- **Source**: MUTGEN (IEEE TSE 2026)
- **Finding**: Incorporate live/uncovered mutant information into prompts. Iterative generation pushes mutation score to **89.5%** on HumanEval-Java. Ablation confirms mutation feedback is the strongest component.
- **NeoTrix Action**: Feed live mutant information back into SelfTest generation prompts. Close the loop: generate tests → run mutants → feed survivors back → generate targeted tests.

---

## 3. Property-Based Testing (7 sources)

### NEW Defect 648-13: Property Runner Architecture is Rigid
- **Source**: DBAS (arXiv:2602.18545, ICFP 2026 track)
- **Finding**: QuickCheck's design bakes generation+minimization loops into opaque representation. Users cannot customize the property runner. DBAS introduces "deferred binding abstract syntax" allowing user-level property runners.
- **NeoTrix Defect**: `nt_selftest` has a fixed test execution loop. Cannot customize generation strategy (coverage-guided fuzzing vs random vs enumeration) per module type. One-size-fits-all approach misses module-specific optimal strategies.
- **Severity**: HIGH — limits SelfTest effectiveness across diverse NeoTrix modules.

### NEW Defect 648-14: No Cross-Language PBT Framework Integration
- **Source**: Etna (ICFP 2026)
- **Finding**: Cross-language PBT evaluation across Haskell/Rocq/OCaml reveals that framework choice matters more than language. Specification-derived strategies outperform type-driven for complex invariants.
- **NeoTrix Defect**: SelfTest has no PBT framework integration. Only Rust unit tests. Cannot apply property-based testing to verify NeoTrix's invariants (e.g., KB consistency, GWT attention balance, SEAL pipeline convergence) across the Rust core + TypeScript frontend.
- **Severity**: MEDIUM — limits invariant verification to simple assertions.

### NEW Defect 648-15: Shrinking Not Evaluated Separately from Bug-Finding
- **Source**: Evaluating Shrinking (Haskell 2026 / ICFP 2026)
- **Finding**: Integrated shrinking does not automatically dominate structural shrinking. QuickCheck's structural shrinking is usually faster and competitive in counterexample quality. Generator family and workload both matter.
- **NeoTrix Defect**: SelfTest has no shrinking mechanism. When a test fails, the counterexample is the full generated input. No minimization to aid debugging. Missing the "known unknown" phase — searching for minimal reproduction.
- **Severity**: MEDIUM — increases debugging time for SelfTest failures.

### NEW Defect 648-16: No Programmable Search Strategy for Invariant Testing
- **Source**: DBAS (arXiv:2602.18545)
- **Finding**: FuzzChick-style coverage-guided property runners require framework redesign. DBAS makes search strategies (seed pool design, mutation strategies) configurable at user level. Different strategies work for different workloads.
- **NeoTrix Defect**: Cannot configure search strategy for SelfTest invariant checks. For example, KB consistency properties might benefit from enumeration, while GWT attention routing needs coverage-guided fuzzing. Current approach uses only random generation.
- **Severity**: MEDIUM — one-size-fits-all search strategy.

### NEW Improvement 648-5: Context-Aware Property Monads
- **Source**: falsify 0.4.0 (Well-Typed, 2026)
- **Finding**: `getContext` function provides property metadata (test count, current iteration). Enables "sized" generation that starts with small domains and grows. Improves bug-finding for properties with sparse preconditions.
- **NeoTrix Action**: Implement context-aware property generation for SelfTest. Start with small, focused inputs and grow complexity based on test iteration. Particularly useful for NT-MEMORY KB consistency checks.

### NEW Improvement 648-6: Trace-Based Convergence Testing
- **Source**: QuickChecking Convergence of Rewriting Systems (ICFP 2026)
- **Finding**: Generate+shrink random execution traces. Check if first/last terms share deterministic normal form. Removes data dependency between generators. Efficiently finds counterexamples.
- **NeoTrix Action**: Apply trace-based convergence testing to SEAL pipeline stages. Generate random execution traces through the pipeline and verify convergence properties.

### NEW Improvement 648-7: Mutation-Based Property Runners
- **Source**: DBAS (arXiv:2602.18545)
- **Finding**: Property runners can be parameterized to use mutation-based input generation (mutate previous interesting inputs) instead of fresh random generation. Combines PBT expressiveness with fuzzing efficiency.
- **NeoTrix Action**: Implement mutation-based SelfTest input generation. Mutate previous failing inputs to discover adjacent failures. Particularly useful for NT-SHIELD security testing.

---

## Summary of New Defects (11 total)

| ID | Defect | Severity | Source |
|----|--------|----------|--------|
| 648-1 | No spec-driven oracle generation in SelfTest | HIGH | Spec-Driven TG |
| 648-2 | No multi-agent oracle consensus | MEDIUM | CANDOR |
| 648-3 | No CPG/GNN-based test targeting | HIGH | GLMTest |
| 648-4 | No phase separation in agentic test loops | MEDIUM | DPIAgent |
| 648-5 | No adversarial RL for test generation | MEDIUM | TCS |
| 648-6 | No repository-level context augmentation | HIGH | XREPOTEST |
| 648-7 | No CFG-based logical reasoning layer | HIGH | Logic-CoT |
| 648-8 | Equivalent mutant detection data leakage | CRITICAL | ISSTA 2026 |
| 648-9 | No contextual equivalent mutant analysis | HIGH | GEM-LLM |
| 648-10 | No RAG-augmented mutation generation | HIGH | SMART |
| 648-11 | No skeptical agent pattern for mutation→test | CRITICAL | MuTON |
| 648-12 | No mutant severity prioritization | HIGH | MuTON |
| 648-13 | Rigid property runner architecture | HIGH | DBAS |
| 648-14 | No cross-language PBT integration | MEDIUM | Etna |
| 648-15 | No shrinking mechanism | MEDIUM | Evaluating Shrinking |
| 648-16 | No programmable search strategy | MEDIUM | DBAS |

## Summary of New Improvements (7 total)

| ID | Improvement | Source |
|----|-------------|--------|
| 648-1 | Commit-message-driven regression testing | Cleverest |
| 648-2 | Hybrid LLM+Fuzzing seed generation | Cleverest |
| 648-3 | SQLite-backed persistent mutation results | MuTON |
| 648-4 | Mutation-guided iterative test generation | MUTGEN |
| 648-5 | Context-aware property monads | falsify 0.4.0 |
| 648-6 | Trace-based convergence testing | ICFP 2026 Pearl |
| 648-7 | Mutation-based property runners | DBAS |

## Cross-Batch Connections

**Batch 647 ↔ 648:**
- **Proxy reward gaming (647)** → **MuTON skeptical agent defect (648-11)**: Mutation-based test generation is another proxy reward surface. If NT-MIND auto-generates tests from mutations, it can game mutation scores by writing tests that kill trivial mutants while missing real bugs.
- **Sequential growth plateau (647)** → **Mutant severity prioritization (648-12)**: Running all mutants sequentially causes plateau. Severity-based prioritization (high→medium→low, skip redundant) directly addresses this.
- **Scalar fitness limitation (647)** → **Multi-dimensional mutation metrics (648-8,10)**: Scalar mutation score cannot distinguish equivalent mutants from real escapes. Need multi-dimensional evaluation (compilability, uniqueness, semantic similarity, contextual equivalence).

## Sources Cited

1. Spec-Driven Test Generation — arXiv:2608.17177
2. CANDOR — ACM doi:10.1145/3803418
3. GLMTest — ACL Findings 2026
4. DPIAgent — arXiv:2608.23341
5. TCS (Test Cases Scaling) — arXiv:2609.03955, EMNLP 2026
6. XREPOTEST — arXiv:2608.25939, EMNLP 2026
7. Logic-CoT — MDPI Applied Sciences 16(5), 2026
8. Cleverest — FSE 2026, Boehme et al.
9. ISSTA 2026 EMD Re-evaluation — conf.researchr.org
10. GEM-LLM — Intelligent Systems with Applications, 2026
11. SMART — arXiv:2603.24560
12. MuTON/mewt — Trail of Bits Blog, 2026-04-01
13. MUTGEN — IEEE TSE 2026
14. DBAS — arXiv:2602.18545
15. Etna — ICFP 2026
16. Evaluating Shrinking — ICFP 2026 Haskell Symposium
17. falsify 0.4.0 — Well-Typed Blog, 2026-07-17
18. QuickChecking Convergence — ICFP 2026 Functional Pearl
