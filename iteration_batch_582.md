# Iteration Batch 582 — Testing Infrastructure: Property Testing, Fuzzing, Test Frameworks

**Date:** 2026-09-06
**Predecessor:** Batch 581 (Kafka dual-write corruption, event sourcing projection gap, RabbitMQ GC regression, KRaft+migration+projections triple consistency hazard)
**Research Areas:** Property Testing, Fuzzing, Test Frameworks (Criterion/Rstest/Nextest)

---

## 1. Property Testing

### 1.1 proptest 1.10/1.11 (2026)

**Status:** Active, 140M+ downloads, 9.3M/month on crates.io, used in 6,698 crates.

**Key findings (NEW vs batch 581):**

| Finding | Detail | NeoTrix Impact |
|---------|--------|----------------|
| `.proptest-regressions` silent regression amplifier | Committing regressions file to VCS turns every discovered counterexample into a permanent regression test that runs BEFORE any random cases. **If not committed, entire regression suite is silently disabled.** | NT-MEMORY KB schema versions + E8 hexagram transitions need property tests with committed regression files; otherwise schema-breaking changes re-enter silently. |
| `prop_assume!` rejection rate bomb | High rejection rates (>50%) cause `too many global rejects` abort. Common when testing constrained domain invariants (e.g., `b != 0` on i64 generation). **Workaround: constrained strategy generation, not filtering.** | NT-CORE HyperCube operations have strict constraints (non-zero dimension, valid hexagram states). Naive property tests will hit rejection wall; must use constrained strategies. |
| Stateful testing is still immature | `proptest-state-machine` exists but `proptest-stateful` (Readyset) explicitly states: "Individual operations are NOT currently shrunk" and "There is currently NO way to symbolically model outputs at generation time." | NT-MIND SEAL pipeline state machine (Soil→Roots→Trunk→Branches→Fruits→Core) needs stateful property testing. Current tools cannot shrink operation sequences or model intermediate outputs — **silent gap**. |
| Single-value edge cases are structurally invisible | proptest docs explicitly warn: "property testing is extremely unlikely to find single-value edge cases in a large space" (e.g., `i64::MIN` for `.abs()`). Random sampling cannot find specific sentinel values. | NT-CORE E8 has 64 hexagram states; property testing will miss state `0x00` or `0xFF` as unique corruptors. **Must supplement with targeted fuzzing.** |
| `#[derive(Arbitrary)]` v0.6 — no Custom Constraints | `proptest-derive` generates strategies from struct fields but cannot enforce cross-field constraints. Must manually override with `#[proptest(strategy = "...")]`. | NeoTrix `SelfModel` (3 variants in neotrix-core) has cross-field invariants (uncertainty range must match capability tier). Derive macro will generate invalid instances. |

**Sources:**
- https://qaskills.sh/blog/rust-proptest-property-testing-guide-2026
- https://github.com/proptest-rs/proptest
- https://github.com/proptest-rs/proptest/issues (634, 633, 628)
- https://github.com/readysettech/proptest-stateful
- https://docs.rs/proptest-stateful

### 1.2 quickcheck 1.x — Stagnant

- Last meaningful commit activity: stable but lighter maintenance than proptest.
- **No regression persistence** — quickcheck has no `.proptest-regressions` equivalent.
- **Coarser shrinking** — type-driven, not value-driven like proptest/Hypothesis.
- **Verdict for NeoTrix:** QuickCheck is not recommended for any new property tests. Use proptest only.

**Source:** https://github.com/BurntSushi/quickcheck

### 1.3 bolero — Fuzz+Property Hybrid

- Wraps libFuzzer, AFL++, and Honggfuzz as a unified check front-end.
- `bolero::check!()` macro provides property testing with fuzzer backend.
- **Use case:** When property tests need coverage-guided mutation, not just random generation.
- NeoTrix `unsafe` blocks (if any escape `#![forbid(unsafe_code)]`) or C FFI boundaries are natural targets.

**Source:** https://github.com/camshaft/bolero

---

## 2. Fuzzing

### 2.1 cargo-fuzz + libFuzzer — Standard Tool

**NEW defects discovered (NEW vs batch 581):**

| Issue | GitHub # | Detail | NeoTrix Impact |
|-------|----------|--------|----------------|
| `cargo fuzz cov` doesn't set `detect_odr_violations=0` | #447 | Coverage mode doesn't suppress ODR violation detection, causing false positives when linking multiple crate versions. | NT-CORE workspace with multiple crates sharing types may trigger phantom ODR violations in coverage runs. |
| SIGILL on empty fuzz target | #443 | Empty `fuzz_target!` body generates SIGILL instead of helpful error. | Developer confusion during initial harness setup. |
| Workspace support lacking | #430 | `cargo-fuzz` doesn't properly handle Cargo workspace members; builds fail when fuzz targets live in sub-crates. | NeoTrix multi-crate workspace (`neotrix-core`, `crates/`) needs fuzz targets per domain module. Workspace issues will block this. |
| `rustflags` passthrough broken | #429 | `cargo-fuzz build` ignores `RUSTFLAGS`, `CARGO_ENCODED_RUSTFLAGS`, `.cargo/config.toml` rustflags. Crates requiring `--cfg` flags (like `tokio`) cannot be fuzzed. | If NeoTrix ever uses conditional compilation in fuzz targets, flags will be silently dropped. |
| Corpus minification fails on whitespace paths | #428 | `cargo fuzz cmin` panics if corpus path contains spaces. | macOS user paths commonly have spaces. |
| `std::hint::black_box` not working | #436 | `black_box` gets optimized away in fuzz harnesses, allowing the fuzzer to skip the target code entirely. | Silent false-negative: fuzzer reports coverage but target is never actually executed. |

**NEW architectural finding — Structure-aware fuzzing gap:**

Fuzze.rs (managed service) highlights: "Rust catches memory-safety bugs — but fuzzing remains the highest-yield way to find the bugs the borrow checker can't see." The critical gap is **structure-aware fuzzing**: raw `&[u8]` fuzzing misses bugs in stateful systems. The `arbitrary` crate + `Arbitrary` trait is the recommended path, but:
- `Arbitrary` for NeoTrix's complex types (HyperCube, E8 hexagram, VSA vectors) would require massive manual implementation.
- **Neither cargo-fuzz nor proptest provide automated structure-aware generation for domain-specific types.**

**Sources:**
- https://github.com/rust-fuzz/cargo-fuzz/issues (428-447)
- https://fuzze.rs/use-cases/fuzzing-rust
- https://adhdecode.com/articles/cargo/cargo-fuzz-testing/
- https://rust-fuzz.github.io/book/cargo-fuzz.html

### 2.2 AFL++ via afl.rs

- Better for stateful targets where libFuzzer's persistent-mode constraints hurt.
- **Parallel fuzzing is native** (unlike libFuzzer's experimental `--jobs`).
- NeoTrix NT-SHIELD sandbox egress policy could benefit from AFL++'s dictionary-based mutation for network protocol fuzzing.

### 2.3 LibAFL — Research-Grade

- Custom fuzzer development, not production-ready for NeoTrix's current stage.
- Worth monitoring for future NT-SHIELD protocol fuzzing.

### 2.4 Sanitizer Integration Gap

From cyberguid.com and rustfaq.org (2026):
- **LeakSanitizer** (`-Zsanitizer=leak`) detects memory leaks but requires nightly.
- **AddressSanitizer** (`-Zsanitizer=address`) catches buffer overflows and use-after-free.
- **Critical gap:** Neither sanitizer works with `cargo-fuzz`'s `--sanitizer none` flag (the performance-optimal setting for safe Rust). But if NeoTrix ever has `unsafe` blocks or FFI, enabling sanitizers requires nightly + manual RUSTFLAGS — **cargo-fuzz doesn't expose this cleanly**.

**Source:** https://www.rustfaq.org/en/how-to-debug-memory-issues-in-rust/

---

## 3. Test Frameworks

### 3.1 cargo-nextest 0.9.140 (Jul 2026)

**Status:** 3.2K stars, 567K downloads/month, active development.

**NEW findings (NEW vs batch 581):**

| Finding | Detail | NeoTrix Impact |
|---------|--------|----------------|
| Per-test process isolation | Each test runs in its own process. Tests sharing `lazy_static` or global temp dirs will FAIL. | NeoTrix's `SharedKVStore` and `EventBus` singletons will break under nextest unless explicitly isolated. |
| Leak detection (`run-leak`) | nextest FAILS tests whose threads outlive the test body. `cargo test` never reported this. | NeoTrix background threads (heartbeat, EventBus listeners) will surface as test failures for the first time. |
| Build-dir-layout-v2 compat (2026-07-30) | nextest now computes dynamic library search path correctly under Rust's new build directory layout v2. | If NeoTrix is on nightly Rust 1.93+, must ensure nextest ≥0.9.139 or dylib resolution will break. |
| `cargo test --doc` still required | nextest CANNOT run doctests. CI must keep `cargo test --doc` as a separate step. | Doctest-only regressions will be missed if CI only uses nextest. |
| Security: RUSTSEC-2026-0097 | rand 0.10.1 unsoundness fixed in nextest 0.9.136. | Dependency chain audit needed if NeoTrix uses rand. |
| Record/replay/rerun (experimental) | `NEXTEST_EXPERIMENTAL_RECORD=1` enables recording test runs for replay. | Potential for deterministic CI replay of flaky NeoTrix tests. |

**Source:** https://github.com/nextest-rs/nextest/blob/main/site/src/changelog.md

### 3.2 rstest — Fixtures and Parameterization

**Status:** 1.6K stars, active (last push Mar 2026).

**Key finding (NEW vs batch 581):**

| Finding | Detail | NeoTrix Impact |
|---------|--------|----------------|
| `#[fixture]` is per-test, not `before_all` | rstest fixtures recreate resources PER TEST. For one-time heavy setup (DB container), combine with `lazy_static`. | NeoTrix KB (SQLite) setup will be recreated per-test if using rstest fixtures naively — massive CI slowdown. |
| `#[values]` combinatorial explosion | Two `#[values]` with 10 values each = 100 tests. With slow tests, 30s → 10min. | Property-like parameterization via `#[values]` on HyperCube dimension combinations could blow up CI. |
| Async test support | rstest supports async test functions with per-test timeout. | NeoTrix async EventBus and crawl pipeline tests can use this directly. |

**Source:** https://www.pistack.xyz/posts/2026-08-25-rust-testing-cargo-test-nextest-rstest-comparison/

### 3.3 Criterion.rs v0.8

**Status:** Moved to `criterion-rs` org. Old `bheisler/criterion.rs` unmaintained.

**Key finding:**
- `#[bench]` is a hard error on stable Rust 1.88+. Criterion or Divan are the only options.
- **Criterion's statistical regression detection** has a known issue: false positives in noisy CI environments (GitHub issue #485). Users report criterion telling them they have regressions when nothing changed.
- **Mitigation:** Use Iai for CI regression detection (Cachegrind-based, immune to OS scheduling noise).

**Source:** https://www.stanza.dev/courses/rust-performance/benchmarking/rust-perf-criterion

---

## 4. Defects Found (NEW vs Batch 581)

### Defect 1: Property Test Regression File Disables Itself Without VCS Commit
**Severity:** HIGH (silent)
**Detail:** `.proptest-regressions` file must be committed to git. If not, every discovered counterexample is transient and bugs silently reappear.
**NeoTrix mapping:** KB schema migration tests, E8 state transition invariants, SEAL pipeline state machine properties.
**Recommendation:** Add `.proptest-regressions` to mandatory CI artifacts; gitignore the local copy but CI must commit it.

### Defect 2: Stateful Property Testing Cannot Shrink Operations
**Severity:** MEDIUM
**Detail:** `proptest-stateful` only shrinks by removing operations from sequences, never by shrinking individual operations. Cannot model intermediate outputs.
**NeoTrix mapping:** SEAL pipeline (6-stage state machine) and ConsciousnessTree growth cycle cannot be properly tested.
**Recommendation:** Supplement with model-based testing framework or manual shrinking for critical state sequences.

### Defect 3: cargo-fuzz Workspace Support is Broken
**Severity:** MEDIUM
**Detail:** cargo-fuzz doesn't handle Cargo workspaces properly. Multi-crate fuzzing requires workarounds.
**NeoTrix mapping:** NT-CORE, NT-MEMORY, NT-WORLD all live in separate crates. Fuzzing across crate boundaries will fail.
**Recommendation:** Use `cargo fuzz init -- fuzzing-workspace = true` for independent fuzz workspace, or fuzz only within single-crate boundaries.

### Defect 4: nextest Per-Test Isolation Breaks Shared Singletons
**Severity:** HIGH
**Detail:** Each nextest test runs in its own process. Shared global state (`lazy_static`, `OnceCell`, `EventBus`) will cause test failures that `cargo test` never reported.
**NeoTrix mapping:** `SharedKVStore`, `EventBus`, `HeartbeatAggregator` are all process-global singletons.
**Recommendation:** Audit all `lazy_static`/`OnceCell` usages; either isolate via test-specific instances or disable nextest isolation for affected tests.

### Defect 5: `black_box` Optimization Barrier Defeated by libFuzzer
**Severity:** MEDIUM (silent false-negative)
**Detail:** `std::hint::black_box` gets optimized away in fuzz harnesses. Fuzzer reports full coverage but target code is never executed.
**NeoTrix mapping:** Any fuzz target using `black_box` on E8/HyperCube inputs may be silently skipped.
**Recommendation:** Use `libfuzzer_sys::arbitrary::Arbitrary` trait instead of `black_box` for fuzz input generation.

### Defect 6: Criterion False Positives in CI
**Severity:** LOW
**Detail:** Statistical regression detection reports false positives in noisy CI. 30% of runs may report phantom regressions.
**NeoTrix mapping:** CI benchmark gates on HeartbeatAggregator latency or crawl throughput will flake.
**Recommendation:** Use Iai (Cachegrind) for CI regression detection; use Criterion only for local profiling.

### Defect 7: proptest Cannot Generate Constrained Cross-Field Types
**Severity:** MEDIUM
**Detail:** `#[derive(Arbitrary)]` generates fields independently. Cross-field invariants (e.g., "uncertainty ≤ capability tier") produce invalid instances 99% of the time.
**NeoTrix mapping:** `SelfModel` (3 variants), `EmotionLabel` combinations, `DynamicParams` (speed/amplitude/frequency bounds).
**Recommendation:** Write manual strategies with `.prop_flat_map` for constrained types; never use derive for domain types.

---

## 5. Summary of NEW vs Batch 581

| Dimension | Batch 581 | Batch 582 (NEW) |
|-----------|-----------|-----------------|
| Infrastructure tested | Kafka, RabbitMQ, event sourcing | Property testing, fuzzing, test frameworks |
| Defects found | 4 (dual-write, projection gap, GC regression, triple hazard) | 7 (regression file, stateful shrink, workspace, singleton isolation, black_box, criterion FP, cross-field constraints) |
| Silent failures | 2 (projection gap, GC regression) | 3 (regression file disable, black_box skip, cross-field invalid instances) |
| Architectural insight | KRaft+migration+projections = triple consistency hazard | Property testing + fuzzing + nextest = triple validation hazard: property tests miss specific values, fuzzing misses stateful bugs, nextest breaks shared state |
| Recommended tools | N/A | proptest (stateless) + proptest-stateful (stateful, limited) + cargo-fuzz (structure-aware) + nextest (CI runner, with singleton audit) + Criterion/Iai (benchmarking) |

---

## 6. Sources Cited

1. https://qaskills.sh/blog/rust-proptest-property-testing-guide-2026
2. https://github.com/proptest-rs/proptest
3. https://github.com/proptest-rs/proptest/issues (634, 633, 628)
4. https://github.com/readysettech/proptest-stateful
5. https://docs.rs/proptest-stateful
6. https://github.com/BurntSushi/quickcheck
7. https://github.com/camshaft/bolero
8. https://github.com/rust-fuzz/cargo-fuzz/issues (428-447)
9. https://fuzze.rs/use-cases/fuzzing-rust
10. https://adhdecode.com/articles/cargo/cargo-fuzz-testing/
11. https://rust-fuzz.github.io/book/cargo-fuzz.html
12. https://cyberguid.com/rust-testing-and-fuzzing-2026/
13. https://www.rustfaq.org/en/how-to-debug-memory-issues-in-rust/
14. https://github.com/nextest-rs/nextest/blob/main/site/src/changelog.md
15. https://www.pistack.xyz/posts/2026-08-25-rust-testing-cargo-test-nextest-rstest-comparison/
16. https://www.stanza.dev/courses/rust-performance/benchmarking/rust-perf-criterion
17. https://github.com/bheisler/iai
18. https://github.com/bheisler/criterion.rs/issues/485
19. https://github.com/strict-rs/strict-proptest
20. https://lib.rs/crates/proptest
