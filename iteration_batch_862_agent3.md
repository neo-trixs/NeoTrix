# Agent 3: Formal Verification (Batch 862)

## Sources

1. **Kani: A Model Checker for Rust** (ASE '26, arXiv:2607.01504) — AWS model checker for Rust using MIR→CBMC pipeline, function/loop contracts, bounded→unbounded verification progression
2. **Creusot: The Rust Verifier** (creusot.rs, v0.13.0) — Deductive verifier translating MIR→Why3, pre/postconditions, prophecies for mutable borrows, ghost resources
3. **Miri: Practical Undefined Behavior Detection for Rust** (POPL 2026, PACMPL Vol 10) — Runtime UB detection: pointer provenance, type invariant validation, data-race detection, weak memory exploration
4. **A Rust-to-Lean Verification Pipeline with AI Provers** (arXiv:2605.30106) — Rust→Charon→LLBC→Aeneas→Lean 4, AI provers closing proof obligations, structural gaps in Hax pipeline
5. **Implementing Formal Verification for Critical Rust Memory Safety Transitions** (martinuke0, 2026) — Prusti + Coq + MIRAI layered pipeline, RustBelt for unsafe reasoning
6. **Surveying the Rust Verification Landscape** (arXiv:2410.01981) — Comprehensive survey: Kani, Prusti, Creusot, Verus, Flux, Proptest, cargo-fuzz
7. **rocq-of-rust** (formal-land, 1160★) — THIR→Rocq translation for 100% execution case verification
8. **FVS: Formal Verification Skills** (Beneficial-AI-Foundation) — Aeneas-based Rust→Lean 4 pipeline with AI-assisted specification
9. **Miri FFI UB Study** (ICSE 2025, McCormack et al.) — 48 UB instances across 38 Rust libraries at FFI boundaries, MiriLLI combined interpreter
10. **Kani in Production CI** (Firecracker, s2n-quic, Cedar, Rust stdlib) — 16,000+ harnesses per code change, 11 bugs found across 4 production codebases

## Defects

**D-FORMAL-001**: `neotrix-sysctl` uses raw pointer casts (`as *const ProcExeTaskInfo`, `as *mut libc::c_void`) without provenance tracking or size validation — Kani's `valid-value pass` and `uninit-memory pass` would catch layout mismatches and uninitialized reads. The `ProcExeTaskInfo` struct is manually defined with `#[repr(C)]` but never validated against the actual kernel ABI struct layout. | `crates/neotrix-sysctl/src/lib.rs:81` | HIGH | Source: Kani ASE'26 §4 (valid-value pass), Miri POPL'26 §2 (pointer provenance)

**D-FORMAL-002**: `shield_enforcer.rs` contains `unsafe { transmute(x) }` in test code (line 612) but no Kani proof harness or Miri annotation exists to verify the transmute preserves validity. Transmute between arbitrary types is a known UB vector that Miri specifically targets — the test only verifies detection of the pattern as a string, not actual runtime safety. | `neotrix-core/src/cli/shield_enforcer.rs:612` | MEDIUM | Source: Miri POPL'26 §3 (type invariant validation), Kani ASE'26 §5 (safety property checks)

**D-FORMAL-003**: No Kani proof harnesses exist for the `EmotionEngine` or `EmotionLabel` state machine — the `emotion_state.rs` module contains complex state transitions (11 variants, animation key mappings, decay calculations with `f64` arithmetic) but zero formal verification of invariant preservation. Kani's contract-based verification could prove that emotion decay never produces NaN, that history bounds are maintained, and that dimension indices remain within array bounds. | `neotrix-core/src/unified/core/nt_core_self/emotion_state.rs:85-100` | HIGH | Source: Kani ASE'26 §5.2 (contract-based unbounded verification), Creusot guide (pre/postconditions for state machines)

**D-FORMAL-004**: `AttentionDomain::from_keywords()` routing table has no formal specification proving coverage exhaustiveness or deterministic routing — a keyword could match multiple domains with priority depending on array ordering, but no Creusot/Verus proof ensures that the routing is total and deterministic for all inputs. The function returns `Option<AttentionDomain>` with no postcondition guaranteeing `None` only when no keyword matches. | `neotrix-core/src/unified/core/nt_core_self/attention_head.rs:37-92` | MEDIUM | Source: Creusot v0.13 (pre/postcondition specification), Kani ASE'26 §4 (function contracts)

**D-FORMAL-005**: `neotrix-sysctl` FFI crate lacks Miri test coverage entirely — the `#![allow(unsafe_code)]` crate performs raw pointer arithmetic and C struct casting that Miri's provenance tracking and uninitialized-memory detection would catch. The Linux path reads `/proc/self/status` with string parsing that could panic on malformed input, but no Miri-compatible test exercises the failure path. | `crates/neotrix-sysctl/src/lib.rs:12-123` | HIGH | Source: Miri POPL'26 §1 (finding all de-facto UB in deterministic programs), McCormack et al. ICSE'25 (48 UB instances at FFI boundaries)

**D-FORMAL-006**: The `SelfTest` trait and convergence-check system has no formal model — `converge_check()` audits ghost modules and orphan files but the correctness of its own detection logic is unverified. Kani's bounded model checking could prove that `converge_check` returns correct results for all module configurations within a bound, preventing false positives/negatives in the self-audit chain. | `neotrix-core/src/unified/core/nt_core_self/self_audit.rs` | MEDIUM | Source: Kani ASE'26 §2 (bounded model checking for logic correctness), survey arXiv:2410.01981 (Kani for functional correctness)

**D-FORMAL-007**: The `HeartbeatAggregator` health signal collector performs time-decay calculations on `f64` values without formal bounds checking — floating-point edge cases (NaN propagation, infinity from division, denormalized numbers) could corrupt the unified `SystemHealthSnapshot`. Creusot's specification language could prove that decay functions maintain values within `[0.0, 1.0]` bounds for all inputs. | `neotrix-core/src/unified/core/nt_core_heartbeat.rs` | HIGH | Source: Creusot v0.13 (deductive verification of numeric invariants), Kani ASE'26 §5 (resource property checks for float-to-integer casts)

**D-FORMAL-008**: The SEAL pipeline's `make_stage!` macro generates state machine transitions without formal verification of liveness properties — no proof exists that the pipeline always terminates, that stages are reached in order, or that no deadlock occurs between concurrent stage handlers. Aeneas/Lean 4 verification could model the pipeline as a state machine and prove total correctness. | `neotrix-core/src/unified/core/nt_core_self/seal/mod.rs` | HIGH | Source: Aeneas Rust-to-Lean pipeline (arXiv:2605.30106), RustBelt framework for unsafe Rust reasoning

**D-FORMAL-009**: `MetacognitiveEvaluator` history management uses `VecDeque` with `max_history` bounds but no formal proof that `self.history.last()` never returns `None` when the code assumes it exists — multiple `.expect("history.len() >= 2 checked above")` calls rely on manual precondition reasoning that could be replaced with Kani proof harnesses verifying the invariant holds for all execution paths. | `crates/neotrix-types/src/core/nt_core_self/metacognitive_evaluator.rs:280-288` | MEDIUM | Source: Kani ASE'26 §5 (panic-freedom proofs), Miri POPL'26 (undefined behavior from unwrap on None)

**D-FORMAL-010**: No formal verification exists for the `CapabilityBridge` mapping between evolution view (CapabilityTree) and runtime view (CapabilityRegistry) — the bridge performs ID translation with potential for inconsistency if tree nodes are deleted while runtime references persist. Creusot's prophecies for mutable borrows could prove that bridge state remains consistent across mutations. | `neotrix-core/src/neotrix/nt_capability_bridge.rs` | MEDIUM | Source: Creusot v0.13 (prophecies for mutable borrows), POPL 2026 tutorial (ownership reasoning)

**D-FORMAL-011**: The `KnowledgeBase` field ledger chain verification (`field_verify_chain`) performs hash chain validation without formal proof of tamper-evidence properties — a Merkle-like chain that could be vulnerable to collision attacks if hash function properties are not formally verified. Kani could prove that chain verification correctly detects any single-block modification. | `neotrix-core/tests/g3_metrics.rs:132` | LOW | Source: Kani ASE'26 (model checking for cryptographic properties), FVS formal verification skills (Lean 4 proofs)

**D-FORMAL-012**: The `nt_core_event_bus` two-layer EventBus architecture lacks formal verification of message ordering guarantees and delivery semantics — concurrent publish/subscribe without formal memory model reasoning could exhibit weak memory behaviors that Miri's weak memory exploration mode would detect. | `neotrix-core/src/neotrix/nt_core_event_bus.rs` | MEDIUM | Source: Miri POPL'26 §4 (weak memory exploration), survey arXiv:2410.01981 (concurrency verification gap)

## Key Insights

1. **Zero Kani harnesses exist in NeoTrix** — The entire codebase has no `#[kani::proof]` annotations despite Kani being the most production-ready Rust verification tool (16,000+ harnesses in Rust stdlib campaign). This is the single largest formal verification gap.

2. **Miri never run on neotrix-sysctl** — The only crate with `#![allow(unsafe_code)]` performs raw pointer arithmetic and C struct casting that Miri specifically targets. This FFI boundary is exactly where the McCormack et al. ICSE'25 study found 48 UB instances across 38 Rust libraries.

3. **No deductive verification for state machines** — The EmotionEngine, ConsciousnessTree, and SEAL pipeline are complex state machines with no formal invariants. Creusot (v0.13) or Verus could prove state transition correctness with low annotation overhead.

4. **f64 arithmetic without bounds** — Multiple modules perform floating-point decay/weight calculations without formal NaN/infinity guards. Kani's resource property checks and Creusot's numeric invariants could enforce `[0.0, 1.0]` bounds.

5. **AI-assisted specification is viable** — The Aeneas+Lean 4 pipeline with AI provers (arXiv:2605.30106) shows that AI can close non-trivial proof obligations, making formal verification of NeoTrix's consciousness architecture more feasible than traditional manual proof construction.

6. **R-P1 zero-unsafe is enforced at crate level but not formally verified** — The `#![forbind(unsafe_code)]` attribute is a compiler check, not a formal proof. Kani's MIR-level verification would provide mathematical certainty that no unsafe code paths exist in core modules.

7. **FFI isolation boundary needs Miri+MIRI combined verification** — The neotrix-sysctl crate is the only FFI boundary and would benefit from the MiriLLI approach (combined Miri+LLVM interpreter) to verify no UB crosses the safe/unsafe boundary.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources analyzed | 10 |
| Critical (HIGH) defects | 5 |
| Medium defects | 6 |
| Low defects | 1 |
| Modules requiring Kani harnesses | 3 (EmotionEngine, HeartbeatAggregator, SEAL pipeline) |
| Modules requiring Miri testing | 1 (neotrix-sysctl) |
| Modules requiring Creusot/Verus specs | 2 (AttentionRouter, CapabilityBridge) |
