# Iteration Batch 416 — Formal Verification & Temporal Logic Research (2026-09-06)

## Sources Cited

| # | Source | Year | Topic |
|---|--------|------|-------|
| S1 | Morard, Donati, Buchs — *Symbolic Model Checking using Intervals of Vectors* (arXiv:2602.03565) | 2026-02 | Novel symbolic method for Petri net markings using generalized intervals on vectors; homomorphic CTL evaluation; canonical form for model checking. |
| S2 | Roy et al. — *MPBMC: Multi-Property Bounded Model Checking with GNN-guided Clustering* (arXiv:2603.04450) | 2026-02 | GNN embeddings for functional property clustering in multi-property BMC; neural-functional hybrid speedup on HWMCC benchmarks. |
| S3 | Frohn et al. — *Accelerated Bounded Model Checking with LoAT* (ScienceDirect, May 2026) | 2026-05 | Non-interpolating BMC algorithms using LoAT for CHC-based infinite-state model checking; opens new research directions beyond interpolation. |
| S4 | Choi et al. — *STL Verification and Synthesis Using Deep Reachability Analysis* (arXiv:2602.23313) | 2026-02 | STL framework combining deep reachability with layered control architecture for mission feasibility verification and safe control synthesis. |
| S5 | Antonelli et al. — *A Linear Temporal Logic of Frequencies on Series of Events* (arXiv:2604.10669) | 2026-04 | LTLF: temporal logic with frequency-sensitive modal quantifiers for event series; bridges formal logic with empirical observation distributions. |
| S6 | AI4FM Group — *ChatTLA+: LLMs for TLA+ Formal Specification Generation and Verification* (Loyola U Chicago, April 2026) | 2026-04 | Systematic evaluation of 30 LLMs on 205 TLA+ specs; NL→TLA+ generation pipelines with SANY parser + TLC model checker validation. |
| S7 | AI4FM Group — *TLA-Prover: Verifiable TLA+ Specification Synthesis via Preference-Optimized LoRA* (ICSOFT 2026) | 2026 | LoRA-fine-tuned LLMs for TLA+ spec synthesis with formal verification loop. |
| S8 | Zylos Research — *Formal Specification and Type-Driven Safety for AI Agent Runtimes* (2026-03-13) | 2026-03 | TLA+ for trust domain transitions in AI agent runtimes; Verus for Rust code verification; VeriGuard dual-stage (offline+online) pattern. |
| S9 | Marmaragan — *Verifying LLM-Generated Code with SPARK Ada* (arXiv:2502.07728) | 2025/2026 | LLM-generated SPARK annotations for formal verification; GPT-4o achieves 50.7% correct annotation rate on benchmark. |
| S10 | Vericoding Guide (baeseokjae.github.io, June 2026) | 2026-06 | Comparative analysis: Dafny/Lean/Verus/SPARK/Coq for AI code verification; Dafny automation 68→96% in one year. |
| S11 | Stanford Encyclopedia of Philosophy — *Temporal Logic* (spr2026 edition) | 2026 | Comprehensive overview of 70 years of temporal logic: LTL, CTL, CTL*, Since-Until, decidability results, automata-based procedures. |
| S12 | emergentmind.com — *Formal Model Checking Overview* (updated Jan 2026) | 2026-01 | Survey: BDD symbolic MC, SAT/SMT BMC, IC3/PDR, CAR, interval temporal logics, quantum MC, ML+MC intersection (CNML, neural-certificate MC). |
| S13 | emergentmind.com — *Signal Temporal Logic (STL)* (updated Aug 2025) | 2025-2026 | GradSTL differentiable semantics, asynchronous temporal robustness for multi-agent, STL-guided diffusion policies, cumulative-time STL. |

---

## Defects Found

### DEFECT-1: No Formal Safety/Liveness Specification for SEAL Pipeline Stages
**Severity**: HIGH  
**Location**: `nt_mind/seal_core/self_iterating/stage_contracts.rs`  
**Evidence**: `stage_contracts.rs:80-84` defines `StageContract` trait with `pre_check`, `post_check`, `invariant_check` — but these are ad-hoc Rust function calls, not derived from a formal temporal specification. There is no TLA+ or temporal logic encoding of the SEAL pipeline's safety/liveness properties.  
**2026 Gap**: S6/S7/S8 demonstrate that TLA+ specifications for state machine protocols (trust domains, evolution loops) catch critical bugs that ad-hoc checks miss. S8's VeriGuard pattern generates lightweight runtime monitors from TLA+ invariants in O(1). NeoTrix's `StageContract` is a manual approximation of this pattern without the formal foundation.  
**Suggestion**: Write a TLA+ specification of the SEAL pipeline state machine (iteration, reward, champion_score transitions). Use TLC to verify safety (no capability degradation > threshold) and liveness (every stage eventually terminates). Generate runtime monitors from TLA+ invariants to replace ad-hoc `invariant_check` calls.

### DEFECT-2: No Temporal Logic Encoding of GWT Attention Routing Invariants
**Severity**: HIGH  
**Location**: `nt_core_self::attention_head`, `PerceptionBridge`  
**Evidence**: `CONTEXT.md:72` defines `PerceptionBridge` with `awareness_score()` gating — a runtime heuristic with no formal temporal specification of when attention broadcasts should/should not fire.  
**2026 Gap**: S4's STL framework and S13's GradSTL demonstrate that signal temporal logic can formally specify and verify cyber-physical attention mechanisms (e.g., "within 3 clock cycles of a salient event, the broadcast must reach all specialist modules"). S5's LTLF extends this to frequency-based properties ("the attention broadcast fires at least k times per n cycles for liveness"). NeoTrix's GWT has no equivalent formal specification.  
**Suggestion**: Encode GWT attention invariants in STL: `□[0,T] (salient_event → ◇[0,δ] broadcast_complete)` for safety; `□[0,T] (◇[0,τ] salient_event)` for liveness. Use GradSTL's differentiable semantics to learn optimal attention parameters under these constraints.

### DEFECT-3: E8 Hexagram Mode Routing Has No State-Space Exhaustive Verification
**Severity**: MEDIUM  
**Location**: `crates/neotrix-types/src/core/nt_core_hex.rs:179,246`  
**Evidence**: The hexagram mode descriptions reference "model checking with external specifications" and "state space formal verification" as capability labels — but no actual model checker (nuXmv, TLC, Alloy) is invoked on the E8 state space. The `kani_proofs.rs` file has exhaustive tests for algebraic properties but NOT for behavioral/state-transition properties.  
**2026 Gap**: S1's interval-vector symbolic method and S2's GNN-guided BMC show that state space explosion for 64-state systems (E8's hexagram space) is tractable with modern symbolic techniques. S12 confirms IC3/PDR can handle Petri net markings of similar complexity. The 64-hexagram × mode-routing state space is a natural candidate for bounded model checking.  
**Suggestion**: Model the E8 mode-routing FSM in nuXmv or Alloy. Encode safety invariants (e.g., "mode transitions preserve dual specialization pairing") and liveness properties (e.g., "every exploration mode eventually returns to baseline"). Run bounded model checking up to bound k=20 to find counterexamples.

### DEFECT-4: No LLM-Synthesized Formal Specifications for Self-Test Tiers
**Severity**: MEDIUM  
**Location**: `CONTEXT.md:88-94` (SelfTest Tiers T1-T3)  
**Evidence**: SelfTest tiers are defined as documentation categories (existence, registration, production wiring) — not as formally specified properties. T3 ("production wiring") requires that detection functions are called by non-test code, but this is checked by grep/review, not by formal verification.  
**2026 Gap**: S6/S7 demonstrate that LLMs can generate TLA+ specifications from natural language at 50%+ correctness, with formal verification catching errors. S9 shows SPARK Ada annotations for LLM-generated code can be auto-generated. S10 shows Dafny automation reached 96% for program verification. NeoTrix could use LLM-assisted specification synthesis to formalize SelfTest tier requirements.  
**Suggestion**: Use ChatTLA+ / TLA-Prover pipeline to generate TLA+ specs for T3 production wiring: "∀ detector ∈ SelfTest, ∃ caller ∈ non_test_code : caller.invokes(detector)". Verify with TLC. Integrate into CI.

### DEFECT-5: SEAL Pipeline Has No Liveness Guarantee for Evolution Cycle Termination
**Severity**: HIGH  
**Location**: `nt_mind/seal_core/self_iterating/pipeline.rs`  
**Evidence**: `pipeline.rs:159` notes "elapsed time shows liveness for long IO/reasoning steps" — but this is a UI concern, not a termination guarantee. The SEAL pipeline (Soil→Roots→Trunk→Branches→Fruits→Core) has no formal proof that every stage eventually completes or that the full cycle terminates.  
**2026 Gap**: S3's accelerated BMC with LoAT demonstrates that infinite-state termination proofs are now feasible for complex pipelines. S8's TLA+ approach explicitly separates safety (what must never happen) from liveness (what must eventually happen). NeoTrix's SEAL pipeline lacks any liveness specification.  
**Suggestion**: Specify SEAL liveness in TLA+: `□(stage_started → ◇stage_completed)` for each stage, and `□(cycle_started → ◇cycle_completed)`. Use LoAT-style acceleration for bounded verification. Add `StageContract` variant `LivenessContract` with deadline parameters.

### DEFECT-6: No Runtime Monitor Generation from Formal Specifications
**Severity**: MEDIUM  
**Location**: `nt_core_self/behavior_fsm.rs`, `nt_repair/nt_repair_self_heal.rs`  
**Evidence**: `nt_repair_self_heal.rs:84` checks "invariant violated (broken)" — a runtime string-based heuristic. No formal specification generates these monitors.  
**2026 Gap**: S8's VeriGuard pattern: "generate lightweight runtime monitors from TLA+ invariants in O(1) time." S4's STL framework synthesizes control from formal specs. NeoTrix's runtime checks are hand-written strings, not generated from a single formal fact source.  
**Suggestion**: Adopt the VeriGuard dual-stage pattern: (1) write TLA+/STL specs for critical invariants, (2) auto-generate O(1) runtime monitors from those specs, (3) fire on violation → trigger NT-REPAIR. Replace hand-written string checks in `nt_repair_self_heal.rs` with generated monitors.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 13 |
| Defects found | 6 |
| HIGH severity | 3 |
| MEDIUM severity | 3 |
| Domains affected | NT-CORE (E8), NT-MIND (SEAL), NT-CORE (GWT), NT-SHIELD (SelfTest), NT-REPAIR (runtime monitors) |

## Recommended Priority Order

1. **DEFECT-1** (SEAL pipeline TLA+ spec) — highest ROI; SEAL is the self-evolution loop, bugs here cascade
2. **DEFECT-5** (SEAL liveness) — complements DEFECT-1; termination guarantees prevent infinite evolution cycles
3. **DEFECT-2** (GWT STL spec) — attention routing is core cognition; formal spec enables GradSTL optimization
4. **DEFECT-6** (runtime monitor generation) — immediately actionable; replace string-based checks with generated monitors
5. **DEFECT-3** (E8 state-space BMC) — tractable given 64-state space; validates mode-routing correctness
6. **DEFECT-4** (SelfTest TLA+ synthesis) — lower urgency but enables LLM-assisted formal spec pipeline
