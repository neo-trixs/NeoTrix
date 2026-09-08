# Iteration Batch 355 — Research Loop Output

**Date**: 2026-09-06
**Domains**: (1) Computer Algebra & Symbolic Computation, (2) Automated Theorem Proving, (3) Program Verification

---

## Sources Cited

| # | Source | Domain | Date |
|---|--------|--------|------|
| S1 | SCML-2026: International Conference on Symbolic Computation and Machine Learning, RISC Hagenberg | Algebra | Jul 2026 |
| S2 | "Breaking the Data Barrier in Learning Symbolic Computation" (arXiv:2601.13731) | Algebra | Jan 2026 |
| S3 | "FePySR: Neural Feature Extraction for Efficient Symbolic Regression" (arXiv:2605.12704) | Algebra | May 2026 |
| S4 | "Symbolic Homotopy Algorithm for Solving Composable Polynomial Systems" (ISSAC 2026, arXiv:2605.22514) | Algebra | May 2026 |
| S5 | "Symbolic-Neural Soft-Logic Reasoning" (arXiv:2605.25618) | Algebra | May 2026 |
| S6 | "A Foundation Model for Zero-Shot Logical Rule Induction" (IJCAI 2026, arXiv:2605.04916) | Algebra | May 2026 |
| S7 | "Neural Compiler: Program-to-Network Translation for Hybrid SciML" (arXiv:2605.22498) | Algebra | May 2026 |
| S8 | "Leveraging AD in Modern ML Frameworks for (Neural) Topology Optimization" (Struct Multidisc Optim 69, 2026) | Algebra | May 2026 |
| S9 | Leonardo de Moura, "Proof Assistants in the Age of AI" (blog, Feb 2026) | Theorem Proving | Feb 2026 |
| S10 | AxiomProver: AI-Generated Mathematical Proofs (axiommath.ai, Feb 2026) | Theorem Proving | Feb 2026 |
| S11 | Goedel-Prover-V2 (ICLR 2026, arXiv:2602.24273) | Theorem Proving | Apr 2026 |
| S12 | "Quantum Automated Theorem Proving" (arXiv:2601.07953) | Theorem Proving | Jan 2026 |
| S13 | Goedel-Architect: Streamlining Formal Theorem Proving (arXiv:2606.06468) | Theorem Proving | Jun 2026 |
| S14 | "A Minimal Agent for Automated Theorem Proving" (arXiv:2602.24273) | Theorem Proving | Feb 2026 |
| S15 | Anthropic, "Formalizing Fermat's Last Theorem" (Sep 2026) | Theorem Proving | Sep 2026 |
| S16 | AlphaProof Nexus: Autonomous proof of 9 open Erdős problems (Google DeepMind, May 2026) | Theorem Proving | May 2026 |
| S17 | "LLM-Based Static Verification of Code Against Natural-Language Requirements" (arXiv:2605.17926) | Verification | May 2026 |
| S18 | "Agentic Verification of Software Systems" (FSE 2026, arXiv:2511.17330) | Verification | Nov 2025 |
| S19 | "Agentic Proving for Program Verification" (arXiv:2605.23772) | Verification | May 2026 |
| S20 | "The Illusion of Safety: Multi-Tier Verification of AI vs. Human C++ Code" (arXiv:2607.00107) | Verification | Jul 2026 |
| S21 | "Comparison between Static Analysis and Model Checking in Automation Systems" (Control Eng Practice 171, 2026) | Verification | Apr 2026 |
| S22 | Beyond Tomorrow, "Formal Verification for AI-Generated Code: Tools and Limits" (Jul 2026) | Verification | Jul 2026 |

---

## Research Findings

### Domain 1: Computer Algebra & Symbolic Computation

**Finding 1.1 — SCML 2026: Hybrid SC+ML is the new standard.** The SCML-2026 conference (RISC, July 2026) formally establishes the convergence: ML applied to SC (equation solving, automated reasoning) AND SC applied to ML (explainability, verified AI, guaranteed error bounds). The field has moved from "can we combine them?" to "how do we compose them?" Topics include: applying LLMs as natural language interfaces to CAS systems, combining linguistic reasoning (LLMs) with formal reasoning (theorem provers), and verifying ML models using computer algebra.

**Finding 1.2 — Symbolic Regression enters neural architecture.** FePySR (arXiv:2605.12704) introduces neural feature extraction for symbolic regression, making equation discovery from data scalable. The Symbolic Homotopy Algorithm (ISSAC 2026) solves composable polynomial systems symbolically — bridging numerical and exact methods.

**Finding 1.3 — Symbolic-Neural Soft-Logic.** arXiv:2605.25618 proposes "Symbolic-Neural Soft-Logic Reasoning" — robust, verifiable thinking chains via cooperative evolution between symbolic and neural components. This directly addresses the gap between LLM hallucination and formal correctness.

**Finding 1.4 — AD in scientific computing workflows.** Sanu et al. (2026) demonstrate that selective AD-wrapping of components in ML frameworks (JAX/PyTorch) enables neural topology optimization — showing that AD must be composable, not monolithic, for real scientific workflows.

### Domain 2: Automated Theorem Proving

**Finding 2.1 — Proof assistants amplify AI, not replace humans.** De Moura (Feb 2026) argues that proof assistant choice matters MORE with AI: expressive types, powerful automation, readable specifications, and fast kernel checking are what make AI-generated proofs compact and correct. The "programmable ecosystem" (tactics, metaprograms, verified code all in one language) creates a compounding flywheel.

**Finding 2.2 — AxiomProver solves Putnam 2025 + 4 open conjectures.** AxiomProver (Axiom Math, 2026) is a multi-agent ensemble prover for Lean 4 that solved all 12 Putnam 2025 problems and 4 previously unsolved open conjectures. Fully automated from natural language → formal Lean 4/Mathlib proofs.

**Finding 2.3 — AlphaProof Nexus proves 9 open Erdős problems.** Google DeepMind (May 2026) announced autonomous proof of 9 open Erdős problems, extending beyond competition math to genuine mathematical research.

**Finding 2.4 — Anthropic formalizes Fermat's Last Theorem.** Using an internal research model ("roughly comparable to Claude Fable 5"), Anthropic (Sep 2026) produced a complete Lean 4 formalization of FLT — the last entry on Wiedijk's "100 Theorems" list. The proof follows the Darmon-Diamond-Taylor 154-page exposition.

**Finding 2.5 — Goedel-Prover-V2: SOTA open-source ATP.** Goedel-Prover-V2 (ICLR 2026) uses scaffolded data synthesis and self-correction to establish new SOTA in open-source Lean 4 theorem proving. Key technique: iterative proof refinement with library search and context management.

**Finding 2.6 — Lean 4 is the de facto infrastructure.** Lean 4 has become the substrate for AI mathematical research. Companies (Axiom Math, Math Inc, Logical Intelligence, Anthropic) all build on Lean. The pattern: AI generates at scale → Lean verifies → compounding library growth.

### Domain 3: Program Verification

**Finding 3.1 — Multi-tier verification is the 2026 playbook.** VulBench-CPP (arXiv:2607.00107) establishes 4-tier verification: (1) functional testing, (2) static analysis, (3) dynamic analysis (ASan/UBSan), (4) bounded model checking (ESBMC). Key finding: AI-generated code has 3.6× runtime violation rate vs. human code. Static analysis alone MASKS this gap entirely.

**Finding 3.2 — Tiered verification for AI codegen.** Industry (Buildkite, GitLab, GitHub) has adopted a tiering strategy: Tier A modules get full proofs (Frama-C, Dafny), Tier B gets fuzz + static analysis, Tier C stays on tests. LLM-generated code defaults to Tier C unless promoted after incidents.

**Finding 3.3 — Agentic proving for program verification.** arXiv:2605.23772 evaluates Claude Code in an agentic proving framework on CLEVER (Lean 4 benchmark for verifiable code generation). Result: Claude generates valid specifications for 98.8% of tasks — showing that agentic theorem proving extends naturally to program verification.

**Finding 3.4 — LLM-based static verification against NL requirements.** arXiv:2605.17926 introduces requirement-aware, semantics-aware static analysis that checks code against natural-language specifications without compilation — shifting verification left.

**Finding 3.5 — Agentic verification with Rocq (FSE 2026).** arXiv:2511.17330 presents autonomous proof agents that collaborate with the Rocq theorem prover for program verification, tested on SV-COMP benchmarks and Linux kernel modules.

---

## Defects Identified in NeoTrix Design

### DEFECT-355-1: No Symbolic Computation Layer (Gap: SCML-2026 convergence)
**Severity**: High
**Location**: Architecture-wide — no `nt_core_symbolic` or equivalent module
**Evidence**: CONTEXT.md defines E8 Hexagram and VSA HyperCube but has no symbolic computation engine. The E8 module (`nt_core_e8/mod.rs`) works with Lie algebra constants and hexagram mappings, but does not perform symbolic equation solving, symbolic differentiation, or polynomial algebra. The VSA HyperCube stores knowledge as vectors but cannot reason symbolically about equations or derive closed-form solutions.
**Gap**: SCML-2026 demonstrates that hybrid SC+ML is now standard. NeoTrix has neural reasoning (E8, GWT) and vector knowledge (VSA HyperCube) but lacks the symbolic bridge — the ability to solve equations, simplify expressions, and perform symbolic differentiation. This means NeoTrix cannot verify mathematical claims it generates, cannot produce closed-form solutions, and cannot apply SC methods to verify its own ML models.
**Suggestion**: Add `nt_core_symbolic` with: (a) symbolic expression representation (AST), (b) CAS integration interface (SymPy/SageMath via FFI or subprocess), (c) symbolic differentiation and integration, (d) equation solving (polynomial, ODE), (e) hybrid SC+ML pipeline — use symbolic methods to verify neural outputs.

### DEFECT-355-2: No Formal Proof Assistant Integration (Gap: Lean 4 as de facto standard)
**Severity**: High
**Location**: NT-CORE, NT-ACT — no Lean 4 or proof assistant bridge
**Evidence**: `nt_core_e8/mod.rs` references mathematical domains ("proof", "theorem", "algebra" in `domain_transition.rs:58-61`) but has no mechanism to generate or verify formal proofs. The SelfTest system checks compilation and test passing but cannot verify logical correctness of reasoning outputs. No Lean 4 bridge exists in the codebase.
**Gap**: AxiomProver, AlphaProof Nexus, Anthropic, and Goedel-Prover-V2 all demonstrate that AI-generated formal proofs in Lean 4 are now practical at scale. NeoTrix's SEAL pipeline produces "evolution" outputs but cannot formally verify that its reasoning chains are logically sound. The `verification_loops` field in `domain_transition.rs:226` exists but has no formal proof backend.
**Suggestion**: Add `nt_core_formal_prover` with: (a) Lean 4 language server protocol (LSP) integration, (b) proof state serialization, (c) tactic library for common reasoning patterns, (d) SEAL→Lean bridge (auto-formalize reasoning chains), (e) "Proof of Evolution" — formally verify that SEAL pipeline outputs satisfy invariants.

### DEFECT-355-3: No Program Verification for Self-Generated Code (Gap: 4-tier verification)
**Severity**: High
**Location**: NT-ACT (code generation), NT-SHIELD (security), SelfTest system
**Evidence**: The SelfTest system (`nt_core_self_test`) checks T1 (existence), T2 (registration), T3 (production wiring) but cannot verify that generated code is memory-safe, overflow-free, or specification-conforming. No bounded model checking, no SMT integration, no static analysis of AI-generated code. The `verification_status` field in `nt_capability_bridge.rs:58` is always `None` (set at lines 416, 427, 449, 473, 497, 518, 547).
**Gap**: VulBench-CPP (2026) proves that AI-generated code has 3.6× the runtime violation rate of human code, and static analysis alone masks this entirely. NeoTrix generates code via NT-ACT but has no verification pipeline. Tier A modules (E8 reasoning, GWT attention, KB operations) should have formal proofs; Tier B should have fuzz + static analysis.
**Suggestion**: Add `nt_act_code_verifier` with: (a) ESBMC/CBMC bounded model checking interface, (b) Clippy/rustfmt + custom lints for generated code, (c) fuzz testing harness for hot-path functions, (d) tier classification (A/B/C) for modules, (e) integration with SelfTest T3 — promote verification_score to influence behavior.

### DEFECT-355-4: No Automatic Differentiation in Neural Reasoning (Gap: composable AD)
**Severity**: Medium
**Location**: NT-CORE (neural reasoning), NT-FEEL (emotion engine)
**Evidence**: The E8 module uses hardcoded constants (E8_DIM=248, E8_ROOTS=240) and the Octonion type (`octonion.rs`) implements manual Cayley-Dickson construction with explicit `f64` arithmetic. No AD framework integration. The latent reasoning transformer (`nt_latent_transformer.rs`) likely uses backpropagation via a framework but has no composable AD for scientific workflows.
**Gap**: Sanu et al. (2026) show that selective AD-wrapping of components enables neural topology optimization — a pattern applicable to NeoTrix's dual-specialization switching (Weapon Set I/II). Composable AD would enable NeoTrix to differentiate through its own reasoning chains, enabling gradient-based optimization of E8 hexagram selection and GWT attention weights.
**Suggestion**: Add `nt_core_autodiff` with: (a) AD trait over Octonion/algebraic types, (b) selective AD-wrapping for E8→GWT pipeline, (c) gradient-based attention weight optimization, (d) forward/reverse mode selection based on E8 domain dimensionality.

### DEFECT-355-5: No Cross-Domain Logical Rule Induction (Gap: IJCAI 2026 foundation model)
**Severity**: Medium
**Location**: NT-MIND (SEAL pipeline), NT-META (meta-cognition)
**Evidence**: The SEAL pipeline (`seal_pipeline.rs`) runs exploration→distillation→self-test→absorption but uses heuristic scoring (VoI, M-open check) rather than inductive logic programming or formal rule induction. The `domain_transition.rs:20` has "Mathematical computation and proof" as a domain but no rule induction engine.
**Gap**: "A Foundation Model for Zero-Shot Logical Rule Induction" (IJCAE 2026) demonstrates that foundation models can now induce logical rules from data without task-specific training. NeoTrix's SEAL pipeline absorbs experiences as embeddings and heuristics, but cannot extract general logical rules or verify that induced rules are consistent.
**Suggestion**: Add `nt_mind_rule_inducer` with: (a) first-order rule extraction from experience traces, (b) consistency checking against KB, (c) rule-to-tactic translation for proof assistant integration, (d) integration with SEAL Phase-5 (distillation).

### DEFECT-355-6: No Quantum Computation Path (Gap: quantum ATP)
**Severity**: Low (future-proofing)
**Location**: NT-CORE (E8 lattice), NT-PHYSICAL (quantum awareness)
**Evidence**: The E8 lattice quantizer (`e8_lattice_quantizer.rs`) exists but is used for discrete optimization, not quantum computation. No quantum computing integration exists.
**Gap**: "Quantum Automated Theorem Proving" (arXiv:2601.07953) proposes a generic framework for quantum-enhanced ATP. While premature for production, the E8 lattice has natural connections to quantum error correction (topological codes).
**Suggestion**: Add quantum interface trait in `nt_physical` — not implementation, but API surface for future quantum backend integration with E8 lattice coding.

### DEFECT-355-7: No Compositional Symbolic-Neural Reasoning (Gap: cooperative evolution)
**Severity**: Medium
**Location**: NT-CORE (E8 reasoning), NT-MIND (distillation)
**Evidence**: The E8 module has separate neural (latent transformer) and structural (hexagram lattice) components, but they do not cooperatively evolve. The `sparse_moe.rs` routes between experts but does not mix symbolic and neural reasoning in a single proof chain.
**Gap**: "Symbolic-Neural Soft-Logic Reasoning" (arXiv:2605.25618) demonstrates cooperative evolution between symbolic and neural components produces robust, verifiable thinking chains. NeoTrix's dual specialization switches between modes but doesn't fuse them.
**Suggestion**: Add `nt_core_cooperative_reasoning` with: (a) symbolic-neural alternating steps, (b) verification hooks between steps, (c) confidence propagation from symbolic solver to neural planner.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 22 |
| Research findings | 15 (across 3 domains) |
| Defects identified | 7 |
| High severity | 3 |
| Medium severity | 3 |
| Low severity | 1 |

**Key pattern**: NeoTrix has strong neural reasoning (E8, GWT, latent transformers) and vector knowledge (VSA HyperCube) but lacks the formal/symbolic layer that 2026 research shows is now essential. The three high-severity defects (no symbolic computation, no formal proof assistant, no program verification for self-generated code) all share a root cause: the system can reason but cannot formally verify its own reasoning. This creates a trust gap that scales with AI autonomy — exactly the problem Lean 4 + multi-tier verification solves in 2026.

**Recommended priority**:
1. DEFECT-355-2 (formal proof assistant) — highest leverage, enables verification of all other modules
2. DEFECT-355-3 (program verification) — safety-critical for AI-generated code
3. DEFECT-355-1 (symbolic computation) — foundation for hybrid SC+ML reasoning
