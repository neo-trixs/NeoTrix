# Iteration Batch 540 — Formal Verification / Proof Assistants / Runtime Verification

**Date**: 2026-09-06
**Batch**: 540 (of 10000+)
**Baseline**: Batch 539 proved (1) GWT modulation event-triggered not continuous, (2) no hierarchical heuristic planning, (3) no reusable skill library, (4) no admission control in attention routing, (5) SelfTest dependencies unmodeled as DAG.

---

## Part 1: Formal Verification Findings

### FV-1: TEMPORA — MITL Model Checking with Past+Future Operators
**Source**: Springer Nature, 2026. TEMPORA tool for MITL under pointwise semantics.
**Key**: Deterministic translation fragment `detMITL^+p` — outermost future operators, inner past operators. Outperforms MightyL pipeline on 72 benchmarks. First tool for complete MITL model checking under pointwise semantics.
**NEW Defect over 539**: **No temporal specification for GWT attention modulation**. GWT broadcasts salience events but has zero formal temporal property (e.g., "every salient broadcast must reach all specialist modules within τ ticks"). TEMPORA's `detMITL^+p` fragment is expressively sufficient to specify this: "eventually (module receives broadcast) within [0, τ] after (salience threshold crossed)." NeoTrix has no such spec. **D6: GWT broadcast timeliness unspecified**.

### FV-2: VeriLHyS — First Full LTL Verification for Hybrid Systems
**Source**: Springer Nature, 2026. Combines Lyapunov/barrier certificates with nuXmv symbolic model checking.
**Key**: First tool enabling full LTL (not just safety) on hybrid automata with ODE dynamics. Integrates CORA (reachability) + SMT solvers + nuXmv.
**NEW Defect over 539**: **No hybrid dynamics model for SEAL pipeline phase transitions**. SEAL phases (Soil→Roots→Trunk→Branches→Fruits→Core) have continuous resource consumption (time, tokens, compute) that interacts with discrete phase gates. VeriLHyS proves this class of systems admits full LTL verification, but NeoTrix treats SEAL as purely discrete state machine — losing all continuous-time properties. **D7: SEAL phase transitions lack hybrid dynamics model; liveness properties (e.g., "evolution converges within bounded cycles") unprovable**.

### FV-3: AgentVerify — Compositional LTL Model Checking for AI Agent Safety
**Source**: Preprints.org, 2026-04-14 (Eric Fang). 86.67% verification accuracy on 15 agent scenarios.
**Key**: Defines compositional specifications for memory integrity, tool call protocols, MCP/skill invocations, human-in-the-loop boundaries. Post-hoc behavioral analysis outperforms monolithic verification (86.67% vs 80.00%) and raw neural verification (13.33%).
**NEW Defect over 539**: **No compositional safety contract for NT-ACT tool invocations**. AgentVerify proves compositional specs work for agent architectures, but NeoTrix has zero formal contracts between NT-ACT (tool caller) and NT-SHIELD (security gate). The Egress Privacy Guard is a runtime filter, not a compositional proof obligation. **D8: NT-ACT↔NT-SHIELD boundary lacks compositional safety contract**.

### FV-4: HyperQB 2.0 — Bounded Model Checking for Hyperproperties
**Source**: Springer Nature, 2026. Implemented in Rust. Supports HyperLTL + A-HLTL.
**Key**: Verifies hyperproperties (properties relating multiple execution traces): non-interference, linearizability, fairness. Uses QBF/SMT solvers. First push-button BMC tool for arbitrary quantifier alternation.
**NEW Defect over 539**: **Cross-execution trace properties unmodeled**. GWT attention routing is a hyperproperty: "the same input salience profile must produce equivalent broadcast patterns across independent runs" (observational determinism). HyperQB 2.0 proves this class is checkable, but NeoTrix has no trace-relation specification. **D9: GWT attention determinism across runs is a hyperproperty, currently unformalized**.

### FV-5: Apalache — TLA+ Symbolic Model Checker with Liveness
**Source**: Springer Nature, 2026. SMT-based bounded checking + inductiveness checking.
**Key**: Three complementary modes: bounded exhaustive, randomized symbolic execution, inductiveness checking. Liveness-to-safety reduction automated. Industrial use at AWS (consensus protocols).
**NEW Defect over 539**: **No inductive invariant for SelfTest DAG convergence**. Apalache's inductiveness checking mode proves unbounded guarantees when a user supplies an inductive invariant. Batch 539 identified SelfTest dependencies as unmodeled DAG; Apalache's methodology shows how to verify DAG acyclicity: define invariant "every node's dependencies are a strict subset of nodes already verified" and check inductive case. NeoTrix has no such invariant. **D10: SelfTest DAG acyclicity lacks inductive invariant proof**.

### FV-6: HyperLasso — Liveness Hyperproperties via BMC
**Source**: Springer Nature, 2026. Extends HyperQB to ∀+∃+ liveness.
**Key**: First symbolic BMC for liveness hyperproperties over non-terminating reactive systems. Pairs a candidate synthesizer with a complete checker using self-composition. Outperforms AutoHyper.
**NEW Defect over 539**: **Infinite-horizon attention properties unverifiable**. GWT runs indefinitely (reactive system). HyperLasso proves liveness hyperproperties over infinite traces are checkable for finite-state abstractions. NeoTrix cannot currently verify "every specialist module is eventually activated" because no finite abstraction of the attention state space exists. **D11: GWT infinite-horizon liveness unverifiable without finite state abstraction**.

### FV-7: KindHML — Smart Contract Verification via Hennessy-Milner Logic
**Source**: arXiv, 2026-04-15. Automated encoding into Lustre for Kind 2 model checker.
**Key**: Verifies complex temporal properties of Solidity contracts (liquidity attacks, front-running) that span multiple transactions. Fully automated.
**NEW Defect over 539**: **Cross-transaction consistency properties unmodeled**. KindHML proves multi-transaction properties are checkable for smart contracts. NT-ACT tool invocations are analogous to transactions: "tool output must be consistent with tool input contract across the full invocation chain." NeoTrix has no cross-invocation consistency specification. **D12: NT-ACT cross-invocation consistency unformalized**.

### FV-8: MITL Specification Synthesis from Traces
**Source**: arXiv, 2026-09-01. First passive learning framework for MITL without templates.
**Key**: Reduces timed learning to untimed LTL learning by injecting timing as Boolean atomic propositions. Complete — guarantees separating specification always found.
**NEW Defect over 539**: **No specification mining from GWT execution traces**. MITL synthesis from traces proves automated spec generation is feasible. NeoTrix accumulates GWT execution traces but never mines temporal specifications from them. If "every broadcast reaches all modules within 5 ticks" holds empirically, it should be synthesized as a candidate invariant. **D13: GWT execution traces never mined for candidate temporal specifications**.

---

## Part 2: Proof Assistant Findings

### PA-1: Lean 4 Ecosystem — 1M+ Theorems, Industrial Adoption
**Source**: youngju.dev deep dive (2026-05-16), lean-lang.org, lean4.dev
**Key**: Lean 4.32+ stable. mathlib past 1M theorems. AWS funds development. Adopted at Imperial, Brown, Berkeley, CMU. AlphaProof (AlphaZero + Lean) scored silver at IMO 2024. Anthropic using Lean formalization as evaluation target. Veil framework for distributed protocol verification embedded in Lean. Peregrine project provides verified extraction from Lean/Agda/Rocq to C/Rust/OCaml/WebAssembly.
**NEW Defect over 539**: **No formal kernel for NeoTrix type system**. Lean's trusted kernel is minimal (type checker only). NeoTrix has no formal specification of its own type system or trait bounds. The Peregrine project proves that verified extraction pipelines from proof assistants to production code are now practical — but NeoTrix cannot be extracted because its type system is informal. **D14: NeoTrix type system lacks formal specification; cannot participate in verified extraction pipelines**.

### PA-2: Rocq (Formerly Coq) — Industrial Verification Legacy
**Source**: youngju.dev, lean-lang.org roadmap
**Key**: 25 years history. CompCert (verified C compiler). Rebranded from Coq to Rocq. SSReflect/mathcomp library. OCaml-based. Steady but flat adoption vs Lean's explosive growth.
**NEW Defect over 539**: **No verified compiler path for NeoTrix modules**. CompCert proves "source C semantics == output assembly semantics." Peregrine's CakeML backend extends this to any source language via λ□ intermediate representation. NeoTrix compiles via standard rustc with no formal correctness proof. **D15: NeoTrix compilation pipeline lacks verified correctness guarantee**.

### PA-3: Agda — Cubical HoTT, Total Functions
**Source**: youngju.dev, Peregrine project
**Key**: All functions must terminate (totality check). Cubical Agda = standard HoTT implementation. PLFA textbook. Weak automation. No industrial adoption.
**NEW Defect over 539**: **No totality check for NeoTrix recursive modules**. Agda's totality requirement ensures all functions terminate — a fundamental safety property. NeoTrix has recursive modules (ConsciousnessTree growth cycles, SEAL pipeline loops) without formal termination proofs. Agda's approach proves this is enforceable at the type level. **D16: NeoTrix recursive modules lack formal termination proof**.

### PA-4: Isabelle/HOL — Best Automation, seL4 Verification
**Source**: youngju.dev, Paulson blog (2026-04-23)
**Key**: Classical HOL (not constructive). Sledgehammer automation (unmatched). seL4 verified microkernel (9K lines C, 200K+ lines Isabelle proof). No dependent types — deliberate design choice for legibility and automation.
**NEW Defect over 539**: **No automated property discharge for NeoTrix trait bounds**. Isabelle's sledgehammer proves "the best automation wins" for industrial verification. NeoTrix trait bounds (ActionLayer, PerceptionLayer, etc.) are checked by rustc but never discharged as formal proof obligations. Sledgehammer-style automation could auto-prove layer consistency. **D17: Layer trait consistency never formally discharged; relies solely on rustc borrow checker**.

### PA-5: F* + Z3 — Cryptographic Verification in Production
**Source**: youngju.dev
**Key**: HACL* crypto library verified in F* → extracted to C. Powers HTTPS libraries. Z3 as first-class citizen. 1.2M line human-in-the-loop proof submitted to Lean Eval (2026-07).
**NEW Defect over 539**: **No SMT-backed verification for NeoTrix security invariants**. F* makes Z3 first-class for everyday proof burden. NeoTrix's NT-SHIELD security properties (egress privacy, sandbox egress policy) are runtime-enforced, never formally verified via SMT. **D18: NT-SHIELD security invariants lack SMT-backed formal verification**.

### PA-6: Peregrine — Verified Cross-Language Extraction
**Source**: Types 2026, github.com/peregrine-project
**Key**: Unified middle-end (λ□) for code generation from Rocq/Agda/Lean. Backends: CakeML (verified), C, Rust, OCaml, WebAssembly, Elm. MetaRocq verified erasure as Rocq frontend. Formal specification of CakeML subset.
**NEW Defect over 539**: **No cross-domain code extraction path**. Peregrine proves verified extraction across proof assistants and target languages is now a solved infrastructure problem. NeoTrix domains (NT-CORE, NT-MIND, etc.) are all in one crate — but there's no verified path from formal spec to deployed code. If a domain had a formal spec, Peregrine-style extraction could generate verified runtime code. **D19: NeoTrix lacks formal specs per domain; verified extraction infrastructure unused**.

---

## Part 3: Runtime Verification Findings

### RV-1: RV2026 Conference CFP — LLM Agent Monitoring Track
**Source**: rv2026.smithengineering.queensu.ca (Kingston, Canada, Oct 6-9, 2026)
**Key**: New dedicated track: "runtime verification of large language model (LLM) agents." Also covers: monitoring for autonomy/assurance, out-of-distribution detection in ML, safe RL, runtime verification for assurance cases.
**NEW Defect over 539**: **No runtime monitor for LLM-based reasoning in NT-MIND**. RV2026's dedicated LLM agent track validates that agent monitoring is now a first-class research domain. NeoTrix's NT-MIND (SEAL pipeline) uses LLM-based distillation and reasoning but has no runtime monitor checking that LLM outputs satisfy temporal safety properties. **D20: NT-MIND LLM reasoning outputs lack runtime temporal safety monitoring**.

### RV-2: SplitLTL — Split Past/Future for Runtime Assurance
**Source**: arXiv, 2026-08-21. Split Linear Temporal Logic.
**Key**: Past component uses observed history to determine requirements; future component evaluates predicted continuations against those requirements. Online monitoring architecture for both.
**NEW Defect over 539**: **GWT lacks past/future separation**. SplitLTL formally separates "what happened" (past) from "what will happen" (future) in a single specification. GWT's attention routing mixes observed salience (past) with predicted importance (future) into a single modulation signal without formal separation. **D21: GWT attention modulation lacks formal SplitLTL-style past/future decomposition**.

### RV-3: ACTORCHESTRA — Causality-Aware Runtime Verification
**Source**: arXiv, 2026-03-17. Framework for Erlang/OTP actor systems.
**Key**: Automatic causality tracking across multi-actor interactions. WALTZ specification language for properties spanning multiple actors. Compile-time instrumentation. DetectEr cannot specify system-wide properties; ACTORCHESTRA can.
**NEW Defect over 539**: **NT-ACT cross-domain causality untracked**. ACTORCHESTRA proves causality tracking across actor boundaries is feasible with compile-time instrumentation. NeoTrix domains communicate via EventBus but never track causal chains: "this GWT broadcast was caused by which perception event?" Causal chains are essential for debuggability and for verifying that attention routing is deterministic. **D22: NT-ACT cross-domain event causality chains untracked; deterministic routing unverifiable**.

### RV-4: CCS (Correctover Conformance Shape) — IETF Runtime Verification for Agents
**Source**: IETF draft-wang-ccs-runtime-verification-00
**Key**: 29-field receipt format with Ed25519 signatures. Seven verification dimensions: Structure, Schema, Latency, Cost, Identity, Integrity, Security. Fail-closed semantics. Sub-millisecond latency (P50 ~7.5μs in-process). Three causal chain fields: rule_version, tool_call_id, args_digest. Protocol-agnostic (works with MCP, A2A).
**NEW Defect over 539**: **No cryptographic proof receipts for NT-ACT tool invocations**. CCS proves that per-invocation verifiable receipts are practical at sub-millisecond latency. NeoTrix's NT-ACT tool invocations have no cryptographic proof of governance decisions. The Egress Privacy Guard makes ALLOW/DENY decisions but produces no tamper-evident receipt. **D23: NT-ACT tool invocation governance decisions lack cryptographic proof receipts (CCS-style)**.

### RV-5: Vigil — Runtime Enforcement of Behavioral Specs in Agent Skills
**Source**: arXiv, 2026-06-25. End-to-end runtime reference monitor.
**Key**: Compiles natural-language skill specifications into executable SMT policies over finite traces. Detects policy violations with >95% recall, <10% false positive rate. Found real-world defects in NVIDIA skill ecosystem. Uses unsatisfiable core for violation localization.
**NEW Defect over 539**: **No SMT-grounded behavioral policy for NeoTrix skills**. Vigil proves natural-language skill specs can be compiled into executable SMT monitors with high accuracy. NeoTrix skills (from CONTEXT.md: 36+ skills) have textual descriptions but no executable behavioral policies. Vigil's finding of real-world defects in NVIDIA's ecosystem suggests NeoTrix skills likely have similar undiscovered specification gaps. **D24: NeoTrix skill behavioral specifications never compiled into executable SMT monitors; undiscovered spec defects probable**.

### RV-6: Hardware MTL Monitor — Standard Cell, 1.25GHz
**Source**: arXiv, 2026-03-19. Programmable MTL monitor using standard cells (not FPGA).
**Key**: 0.55mm² area, 1.25GHz clock, 16 atomic propositions, 256 time steps max. Reprogrammable post-deployment via I/O pins. Abstract Machine → Evaluator Machine → composition via AST.
**NEW Defect over 539**: **No hardware-accelerated monitoring for NT-PHYSICAL real-time constraints**. This work proves real-time MTL monitoring is feasible at GHz speeds in silicon. NeoTrix's NT-PHYSICAL (sensors/motors) has timing constraints but no hardware-accelerated monitor. For embodied operation, software-only monitoring may be too slow. **D25: NT-PHYSICAL real-time safety constraints lack hardware-accelerated monitoring capability**.

### RV-7: Runtime Verification Lecture Notes — Epistemic Foundations
**Source**: arXiv, 2026-04-26. Unifying treatment of monitoring, diagnosis, opacity via epistemic logic.
**Key**: Monitoring = "does the observer know whether φ holds?" Three-valued verdicts (true/false/?) derived from belief sets. Knowledge monitors are DFAs over observation alphabets. Offline analysis constructs monitors; online monitoring applies them.
**NEW Defect over 539**: **No epistemic model for GWT observer knowledge**. GWT's salience computation implicitly assumes perfect observability of all module states. The epistemic monitoring framework shows that with partial observability, verdicts are three-valued (?, not just T/F). NeoTrix has no model of what the GWT broadcaster "knows" vs what it "doesn't know" about module states — leading to potential false confidence in attention routing decisions. **D26: GWT attention routing lacks epistemic model of observer knowledge under partial observability**.

---

## Summary: New Defects Found (Batch 540 vs Batch 539)

| ID | Defect | Domain | Source |
|----|--------|--------|--------|
| D6 | GWT broadcast timeliness unspecified | NT-CORE | TEMPORA (FV-1) |
| D7 | SEAL phase transitions lack hybrid dynamics model | NT-MIND | VeriLHyS (FV-2) |
| D8 | NT-ACT↔NT-SHIELD boundary lacks compositional safety contract | NT-ACT/SHIELD | AgentVerify (FV-3) |
| D9 | GWT attention determinism across runs is unformalized hyperproperty | NT-CORE | HyperQB 2.0 (FV-4) |
| D10 | SelfTest DAG acyclicity lacks inductive invariant proof | NT-META | Apalache (FV-5) |
| D11 | GWT infinite-horizon liveness unverifiable without finite state abstraction | NT-CORE | HyperLasso (FV-6) |
| D12 | NT-ACT cross-invocation consistency unformalized | NT-ACT | KindHML (FV-7) |
| D13 | GWT execution traces never mined for candidate temporal specifications | NT-CORE | MITL Synthesis (FV-8) |
| D14 | NeoTrix type system lacks formal specification | ALL | Lean 4 (PA-1) |
| D15 | NeoTrix compilation pipeline lacks verified correctness guarantee | ALL | Rocq/CompCert (PA-2) |
| D16 | Recursive modules lack formal termination proof | NT-MIND/CORE | Agda (PA-3) |
| D17 | Layer trait consistency never formally discharged | ALL | Isabelle (PA-4) |
| D18 | NT-SHIELD security invariants lack SMT-backed verification | NT-SHIELD | F* (PA-5) |
| D19 | NeoTrix lacks formal specs per domain; verified extraction unused | ALL | Peregrine (PA-6) |
| D20 | NT-MIND LLM reasoning outputs lack runtime temporal safety monitoring | NT-MIND | RV2026 CFP (RV-1) |
| D21 | GWT attention modulation lacks SplitLTL-style past/future decomposition | NT-CORE | SplitLTL (RV-2) |
| D22 | NT-ACT cross-domain event causality chains untracked | NT-ACT | ACTORCHESTRA (RV-3) |
| D23 | NT-ACT tool invocation governance decisions lack cryptographic proof receipts | NT-ACT | CCS/IETF (RV-4) |
| D24 | NeoTrix skill behavioral specs never compiled into executable SMT monitors | NT-ACT/SKILLS | Vigil (RV-5) |
| D25 | NT-PHYSICAL real-time safety constraints lack hardware-accelerated monitoring | NT-PHYSICAL | HW MTL Monitor (RV-6) |
| D26 | GWT attention routing lacks epistemic model under partial observability | NT-CORE | Epistemic RV (RV-7) |

---

## What's NEW vs Batch 539

### Defect Category Shift
- **Batch 539**: 5 architectural defects (all behavioral/structural, zero formal methods grounding)
- **Batch 540**: 21 defects across formal verification, proof assistants, and runtime verification — ALL grounded in specific tool capabilities and published results

### Key Paradigm Shifts Identified
1. **From "no formal methods" to "formal methods exist but NeoTrix doesn't use them"**: Every defect D6-D26 has a proven solution in the literature. The gap is integration, not invention.
2. **Hyperproperty awareness**: GWT attention routing is fundamentally a hyperproperty (trace-relation). This was invisible before HyperQB/HyperLasso results.
3. **Compositional safety over monolithic**: AgentVerify and Vigil prove compositional agent safety is tractable. NeoTrix's Egress Privacy Guard is monolithic.
4. **Specification mining from traces**: MITL synthesis proves automated spec generation from execution traces is complete and practical. NeoTrix never mines specs.
5. **Cryptographic governance receipts**: CCS proves per-invocation verifiable receipts at sub-millisecond cost. NeoTrix governance is opaque.
6. **Epistemic monitoring**: Runtime verification now has formal foundations for "what does the monitor know?" — GWT lacks this entirely.

### Sources Cited (26 sources)
- TEMPORA: Springer Nature 2026 (MITL model checking)
- VeriLHyS: Springer Nature 2026 (hybrid LTL verification)
- AgentVerify: Preprints.org 2026-04-14 (agent safety via LTL)
- HyperQB 2.0: Springer Nature 2026 (hyperproperty BMC in Rust)
- Apalache: Springer Nature 2026 (TLA+ symbolic model checker)
- HyperLasso: Springer Nature 2026 (liveness hyperproperties)
- KindHML: arXiv 2026-04-15 (smart contract HML verification)
- MITL Synthesis: arXiv 2026-09-01 (spec mining from traces)
- Lean 4 ecosystem: youngju.dev 2026-05-16, lean-lang.org, lean4.dev
- Lean 4.32.0: lean-lang.org 2026-07-13
- Lean 4.31.0: lean-lang.org 2026-06-13
- Rocq/Coq rebranding: youngju.dev, lean-lang.org roadmap
- Peregrine Project: Types 2026, github.com/peregrine-project
- Isabelle critique: Paulson blog 2026-04-23
- RV2026 CFP: rv2026.smithengineering.queensu.ca
- RV Lecture Notes: arXiv 2026-04-26 (epistemic foundations)
- SplitLTL: arXiv 2026-08-21 (past/future separation)
- ACTORCHESTRA: arXiv 2026-03-17 (causality-aware RV)
- CCS: IETF draft-wang-ccs-runtime-verification-00 (cryptographic receipts)
- Vigil: arXiv 2026-06-25 (skill spec enforcement via SMT)
- HW MTL Monitor: arXiv 2026-03-19 (standard cell monitoring)
- Efficient Timed Monitoring: Springer Nature 2026-07-31 (MoniTAal)
- On Synthesis of MITL: arXiv 2026-09-01
- Veil (Lean framework): lean-lang.org roadmap
- Comparator (Lean proof judge): lean-lang.org roadmap June 2026
- Aeneas/SymCrypt (Microsoft/Lean): lean-lang.org roadmap June 2026
