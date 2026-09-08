# Iteration Batch #420 — Property-Based Testing / Fuzzing / AI Test Generation Research

**Date**: 2026-09-06
**Research Domain**: Advanced testing methodologies — PBT, fuzzing, mutation testing, AI test oracles

---

## Sources Cited

1. **Self-Hosted Property-Based Testing: Hypothesis vs Fast-Check vs Proptest Guide 2026** — pistack.xyz (May 2026)
   - proptest 1.4+ with `#[derive(Arbitrary)]`, state machine module, fork/timeout support
   - Shrinking is Hypothesis-style (integrated, not type-driven like quickcheck)
   - `.proptest-regressions` file as permanent regression test — commit to VCS

2. **Rust proptest Tutorial 2026: Property-Based Testing in Rust** — qaskills.sh (Jun 2026)
   - proptest 1.11 (current stable 2026): pure Rust, no external runner, `PROPTEST_CASES` env var
   - 140M+ downloads on crates.io — dominant ecosystem momentum
   - proptest vs quickcheck 2026 verdict: proptest for richer strategies + better shrinking; quickcheck for trivial setup
   - Best practice: 256 cases default local, 10,000+ in CI

3. **Property-Based Testing Is the Quiet Revolution in 2026 Software Engineering** — talkingtech.io (Jun 2026)
   - AI-generated code optimized for training examples, rarely invents edge cases
   - FVSpec (arXiv 2026): PBT frameworks auto-translate property specs → formal verification challenges
   - Key insight: "relying on human-written test cases alone is no longer defensible" in AI-era

4. **AFL++ vs Honggfuzz vs libFuzzer: Self-Hosted Fuzzing Guide 2026** — pistack.xyz (Apr 2026)
   - AFL++ 6,479★: QEMU mode, MOpt power schedules, Redqueen for comparison-based logic bugs
   - Honggfuzz 3,330★: Intel PT hardware-assisted coverage, near-zero overhead
   - libFuzzer: in-process, no fork overhead — millions of executions/sec, Rust-native via `-fsanitize=fuzzer`
   - Fuzzing campaigns: small library = 24hrs; complex protocol = weeks/months; metric is coverage plateau

5. **libFuzzer — Coverage-Guided Fuzz Testing** — LLVM docs (updated Sep 2026)
   - SanitizerCoverage + trace-cmp for CMP instruction interception
   - Value profile: `use_value_profile=1` treats new comparison values as new coverage
   - User-supplied mutators for structure-aware fuzzing
   - Cross-fuzzer corpus sharing: AFL ↔ libFuzzer interchange

6. **AI Testing 2026: Test Generation, Mutation Testing and Coverage** — callmissed.com (Jul 2026)
   - 62% of teams use AI to generate tests weekly (up from 28% YoY)
   - AI-generated tests score 40–55% mutation score vs hand-written 70–90%
   - "AI gives you tests that *run*, not tests that *catch bugs*"
   - AI + mutation feedback loop: 40–55% → 70–85% mutation score (Atlassian internally validated)
   - Pattern: generate → mutate → feed survivors back to AI → repeat until plateau

7. **Mutation Testing with AI** — yrkan.com (May 2026)
   - Intelligent mutant generation: AI creates realistic, plausible mutations cheaper than traditional tooling
   - Coverage gaps exposed by mutation analysis that line/branch coverage misses

8. **Mutation Testing for AI-Generated Code: A Practical Guide** — augmentcode.com (Jun 2026)
   - Assertion gaps in AI-generated tests that line coverage misses
   - Tools: Stryker (JS/TS), mutmut (Python), cargo-mutants (Rust)
   - Threshold recommendation: ≥70% mutation score for production code

9. **Mutation 2026 — ICST 2026 Keynote** — conf.researchr.org (2026)
   - Higher-level mutation: intent-targeting mutants reflecting "what software should do" not just "how"
   - AI as partner in understanding what mutants should mean — beyond automated mutant generation

10. **Test Oracle Automation in the Era of LLMs** — arXiv (2024, still authoritative 2026)
    - LLM oracle types: test assertions, contracts (postconditions), metamorphic relations
    - Oracle deficiencies: false positives (false alarms) and false negatives (missed faults)
    - Data leakage risk: LLMs replicate oracles from training data (Defects4J example)
    - Assurance process: syntax check → mutation analysis → iterative re-prompting

11. **How to Write Property-Based Tests in Rust (proptest, quickcheck)** — rustfaq.org (Apr 2026)
    - Overflow detection: `i32::MAX` + `i32::MIN` in list averaging → wrap to negative
    - proptest as the practical default for production Rust codebases in 2026

12. **Improving AI-generated tests using mutation testing** — Senko Rašić (Mar 2026)
    - AI can create realistic mutations much cheaper (time, money, tears)
    - Mutation survivors fed back to AI for targeted assertion improvement

---

## Defects Found in NeoTrix Architecture

### DEFECT-420-1: Zero Property-Based Testing Infrastructure
**Severity**: HIGH
**Component**: Global test infrastructure
**2026 Finding**: proptest 1.11 is the dominant PBT crate for Rust (140M+ downloads). AI-generated code "optimized for training examples, rarely invents edge cases" — PBT catches these systematically. FVSpec (2026) bridges PBT properties to formal verification.
**Current State**: NeoTrix has 100+ `#[test]` functions — all example-based. Zero `proptest` dependency. Zero `#[derive(Arbitrary)]` on any domain type. No `.proptest-regressions` file. No property definitions for any invariant.
**Gap**: Core domain types (`CoreEvent`, `EmotionLabel`, `CapabilityTree`, `SelfTestResult`, `EventEnvelope`, file ability `MergeSchema`) have no PBT coverage. Edge cases like overflow in aggregation, empty collections, boundary values in emotion routing, and schema deserialization are untested. AI-generated tests (if any exist) likely have the 40–55% mutation score problem.
**Suggestion**: Add `proptest = "1.11"` to dev-dependencies. Derive `Arbitrary` for 10 core types: `CoreEvent`, `EmotionLabel`, `FileKind`, `ContentSnapshot`, `EventEnvelope`, `SelfTestResult`, `CapabilityNode`, `MergeSchema`, `SegmentType`, `DynamicParams`. Write round-trip properties: `serde_json::to_string → serde_json::from_str == identity` for all serializable types. Commit `.proptest-regressions` to VCS. Target: 1,000 cases in CI.

### DEFECT-420-2: No Fuzzing for Parser/SerDe Code Paths
**Severity**: HIGH
**Component**: `nt_file_ability` (doc-parse, merge, format_route), `nt_core_event_bus` (JSONL replay), KB serialization
**2026 Finding**: libFuzzer with SanitizerCoverage + `trace-cmp` achieves millions of executions/sec in-process. AFL++ Redqueen mode targets comparison-based logic bugs. Structure-aware fuzzing via custom mutators can test structured Rust types directly.
**Current State**: `nt_file_ability` parses PDF/DOCX/XLSX/PPTX/Markdown. `EventBus::replay_enveloped()` deserializes JSONL. No fuzz harness exists. No `-fsanitize=fuzzer` targets. No corpus directory. No AFL++ or libFuzzer integration.
**Gap**: Fuzzing the `serde_json::from_str::<EventEnvelope>()` path would immediately surface: (a) panics on malformed JSON, (b) unwrap/expect calls in deserialization, (c) integer overflow in `seq`/`timestamp_ms` parsing. The `MergeSchema` parsing is equally vulnerable — malformed input could trigger index-out-of-bounds or infinite loops.
**Suggestion**: Add `cargo-fuzz` (libFuzzer wrapper for Rust) as dev-dependency. Write fuzz targets: (1) `fuzz_target_event_envelope_parse` — fuzz `serde_json::from_str::<EventEnvelope>()`, (2) `fuzz_target_merge_schema_parse` — fuzz merge input parsing, (3) `fuzz_target_file_ability` — fuzz `FileAbility::detect_kind()` on arbitrary bytes. Set up corpus directory with seed inputs from existing test fixtures. Run nightly in CI with ASan+UBSan.

### DEFECT-420-3: No Mutation Testing to Validate Test Suite Strength
**Severity**: HIGH
**Component**: Global test infrastructure
**2026 Finding**: AI-generated tests score 40–55% mutation score (OutSight 2026). Hand-written tests hit 70–90%. The 2026 pattern is AI generation + mutation feedback loop achieving 70–85%. `cargo-mutants` is the Rust-native mutation testing tool.
**Current State**: No mutation testing tool configured. No baseline mutation score. No CI gate on mutation score. Existing tests have never been validated against mutations. R-P39 tracks SelfTest coverage (61% baseline) but not mutation score.
**Gap**: Without mutation testing, there is no evidence that NeoTrix's 100+ tests actually detect bugs. A test that passes regardless of whether the code under test is mutated is theater, not a defense. The SelfTest system (T1/T2/T3) tracks existence and registration but not *fault detection capability*.
**Suggestion**: Add `cargo-mutants` to CI. Run initial baseline mutation analysis on `nt_core_event_bus`, `nt_file_ability`, and `nt_core_self_test`. Set threshold gate: ≥60% mutation score (realistic for first pass). Feed mutation survivors back to generate targeted property tests. Track mutation score alongside SelfTest coverage in ConsciousnessTree health.

### DEFECT-420-4: No AI-Assisted Test Oracle Validation
**Severity**: MEDIUM
**Component**: Test infrastructure, SEAL pipeline quality gates
**2026 Finding**: 62% of teams use AI test generation weekly. The oracle problem: LLM-generated assertions may replicate training data assumptions (arXiv: data leakage from Defects4J), assert current output rather than intended behavior, or rely on mocks that bypass important behavior.
**Current State**: No mechanism to validate test oracles. No mutation analysis of AI-generated tests (if any). No metamorphic testing for cross-module invariants. `QualityControlPipeline` exists but does not validate oracle quality.
**Gap**: If NeoTrix ever adopts AI test generation (likely given the 2026 trend), there is no quality gate to prevent theater tests. The `QualityControlPipeline` checks format/structure but not fault detection capability.
**Suggestion**: Add oracle validation stage to `QualityControlPipeline`: (1) run `cargo-mutants` on new test files, (2) reject tests with mutation score <50%, (3) for critical paths, require at least one metamorphic property (e.g., `decode(encode(x)) == x`). This turns the pipeline from format-checking to correctness-validation.

### DEFECT-420-5: No Property Tests for ConsciousnessTree Invariants
**Severity**: MEDIUM
**Component**: `nt_core_consciousness_tree`, emotion routing, attention management
**2026 Finding**: PBT excels at testing invariants like sorting, deduplication, idempotency, reversibility, monotonicity, and authorization boundaries. ConsciousnessTree has exactly these kinds of invariants (health scores 0.0–1.0, branch ordering, maturity levels C0–C5).
**Current State**: ConsciousnessTree has `SelfTest` implementations but no property-based invariants. The `HeartbeatAggregator` produces `SystemHealthSnapshot` with time-decay — no property test verifies that decay never goes negative or exceeds 1.0. The emotion routing (`AttentionManager`) has dual specialization switching — no property test verifies that every task type routes to exactly one weapon set.
**Gap**: Invariants that should hold but are untested:
- Health scores always in `[0.0, 1.0]` after any sequence of updates
- `SelfTestRegistry::run_all()` returns results for all registered tests (no silent drops)
- Emotion label transitions are total (every `(EmotionLabel, Event)` pair produces a valid next state)
- Attention routing is deterministic (same task type → same weapon set)
**Suggestion**: Write proptest properties for: (1) `HeartbeatAggregator` health bounds, (2) `AttentionManager` routing determinism, (3) `SelfTestRegistry` completeness, (4) `EmotionLabel` transition totality. Use `#[derive(Arbitrary)]` on `EmotionLabel` and `TaskType` to generate random routing scenarios.

### DEFECT-420-6: No Fuzzing for KB Serialization/Deserialization
**Severity**: MEDIUM
**Component**: `nt_memory` (SQLite KB, embeddings, BM25 index)
**2026 Finding**: Fuzzing serde paths is the highest-ROI fuzzing target for Rust applications. libFuzzer in-process mode achieves millions of iterations/sec on serialization code.
**Current State**: KB uses SQLite with custom serialization for embeddings and node data. `serde_json` and potentially `bincode` for embedding vectors. No fuzz harness for KB read/write paths.
**Gap**: Malformed KB entries could cause: panics in deserialization, integer overflow in embedding dimension parsing, buffer overflows in vector operations, or silent data corruption. The BM25 index rebuild from corrupted data could无限循环 or OOM.
**Suggestion**: Add fuzz targets for: (1) KB node deserialization (`serde_json::from_str::<KbNode>()`), (2) embedding vector parsing, (3) BM25 query parsing. Seed with valid KB entries from test fixtures + malformed variants.

### DEFECT-420-7: No Metamorphic Testing for Cross-Module Invariants
**Severity**: MEDIUM
**Component**: SEAL pipeline, capability tree, file ability
**2026 Finding**: Metamorphic relations (tested via PBT) verify correctness when test oracles are unavailable. Example: "running the pipeline twice on the same input produces the same output" (idempotency).
**Current State**: No metamorphic tests. Each module is tested in isolation. Cross-module invariants (SEAL pipeline output → ConsciousnessTree input, capability tree → runtime registry, file ability → KB persistence) are not formally tested.
**Gap**: Critical metamorphic relations untested:
- SEAL pipeline is idempotent on same input
- Capability tree registration → runtime lookup round-trips without loss
- File ability detect → parse → serialize → deserialize preserves content
- EventBus emit → replay produces identical events
**Suggestion**: Write proptest-based metamorphic tests: for random inputs, verify that (1) `serialize(deserialize(x)) ≈ x` (lossy for floating point), (2) pipeline stages produce consistent output ordering, (3) capability registration count == lookup count.

### DEFECT-420-8: No Coverage-Guided Seed Corpus for Critical Paths
**Severity**: LOW
**Component**: Test infrastructure
**2026 Finding**: AFL++ and libFuzzer both use seed corpus minimization (afl-cmin, llvm-cov) to reduce redundant inputs. Structure-aware fuzzing uses grammar-based mutators for structured formats.
**Current State**: Test fixtures are hand-written strings in `#[test]` functions. No minimized corpus. No grammar-based input generation for structured types.
**Gap**: Without a minimized seed corpus, fuzzing campaigns waste time on redundant inputs. Without grammar-based mutators, structure-aware fuzzing on `CoreEvent` JSON or `MergeSchema` cannot explore valid-but-novel configurations.
**Suggestion**: Extract existing test fixtures into a `corpus/` directory as seed files. For structured types, write proptest `Strategy` functions that generate valid-but-varied inputs (valid emails, valid JSON paths, valid emotion label sequences) as high-quality seeds.

---

## Summary

| # | Defect | Severity | Domain | 2026 Key Finding |
|---|--------|----------|--------|-----------------|
| 420-1 | Zero PBT infrastructure | HIGH | Property-Based Testing | proptest 1.11 dominant, 140M+ downloads, AI code needs PBT safety net |
| 420-2 | No fuzzing for parsers/SerDe | HIGH | Fuzzing | libFuzzer millions/sec, AFL++ Redqueen for logic bugs |
| 420-3 | No mutation testing | HIGH | Mutation Testing | AI tests 40–55% mutation score, feedback loop → 70–85% |
| 420-4 | No AI oracle validation | MEDIUM | AI Test Generation | 62% teams use AI tests, oracle problem: false positives/negatives |
| 420-5 | No PBT for ConsciousnessTree | MEDIUM | Property-Based Testing | Health bounds, routing determinism, transition totality untested |
| 420-6 | No fuzzing for KB paths | MEDIUM | Fuzzing | SerDe paths highest-ROI fuzz target for Rust |
| 420-7 | No metamorphic testing | MEDIUM | PBT/Metamorphic | Cross-module invariants untested (pipeline idempotency, round-trips) |
| 420-8 | No seed corpus | LOW | Fuzzing | Minimized corpus + grammar mutators for structured types |

**Recommendations Priority**:
1. **P0** (this iteration): DEFECT-420-1 (add proptest + derive Arbitrary for core types) — foundational, enables 420-5 and 420-7
2. **P0** (this iteration): DEFECT-420-3 (add cargo-mutants baseline) — validates existing test suite strength
3. **P1** (next iteration): DEFECT-420-2 (add cargo-fuzz harnesses for parsers) — security-critical
4. **P1** (next iteration): DEFECT-420-4 (mutation-gated oracle validation in QualityControlPipeline)
5. **P2** (backlog): DEFECT-420-5, 420-6, 420-7, 420-8

**Estimated Impact**:
- DEFECT-420-1 + 420-3: Establishes testing quality baseline. Catches edge cases in all 100+ existing test functions.
- DEFECT-420-2: Prevents panics/UB in parser code from reaching production.
- DEFECT-420-4: Prevents "theater tests" from being accepted into CI.
