# Iteration Batch 687 — Testing Frameworks, Property-Based Testing, Integration Testing

**Date**: 2026-09-06  
**Predecessor**: Batch 686 (WIT schema evolution gap, WasmGC, runtime skill integrity, MV3 CSP, Memory64)  
**Focus**: Testing ecosystem 2026 — new defects and improvements for NeoTrix consciousness architecture

---

## Sources Cited

| # | Source | URL | Date |
|---|--------|-----|------|
| S1 | dasroot.net — Rust Testing Patterns for Reliable Releases | https://dasroot.net/posts/2026/03/rust-testing-patterns-reliable-releases/ | 2026-03-02 |
| S2 | dasroot.net — Rust Testing Strategies: Unit, Integration, Property Tests | https://dasroot.net/posts/2026/03/rust-testing-strategies-unit-integration-property-tests/ | 2026-03-20 |
| S3 | qaskills.sh — Rust proptest Tutorial 2026 | https://qaskills.sh/blog/rust-proptest-property-testing-guide-2026 | 2026-06-26 |
| S4 | lib.rs — proptest crate page | https://lib.rs/crates/proptest | 2026-03-24 |
| S5 | proptest-rs.github.io — Proptest Introduction | https://proptest-rs.github.io/proptest/intro.html | — |
| S6 | GitHub — proptest-rs/proptest | https://github.com/proptest-rs/proptest | — |
| S7 | GitHub — BurntSushi/quickcheck | https://github.com/BurntSushi/quickcheck | — |
| S8 | oneuptime.com — How to Test Rust Applications with Integration Tests | https://oneuptime.com/blog/post/2026-01-26-rust-integration-tests/view | 2026-01-26 |
| S9 | qaskills.sh — Testcontainers Rust Integration Testing Guide 2026 | https://qaskills.sh/blog/testcontainers-rust-integration-testing-guide | 2026-05-07 |
| S10 | developers-heaven.net — Advanced Integration Testing: Docker-based Test Containers for Rust | https://developers-heaven.net/blog/advanced-integration-testing-docker-based-test-containers-for-rust/ | 2026-06-25 |
| S11 | GitHub — testcontainers/testcontainers-rs | https://github.com/testcontainers/testcontainers-rs | — |
| S12 | JetBrains — RustRover 2026.1: Professional Testing With Native cargo-nextest | https://blog.jetbrains.com/rust/2026/04/03/rustrover-2026-1-professional-testing-with-native-cargo-nextest-integration/ | 2026-04-03 |
| S13 | libs.tech — New Rust Testing Libraries 2026 | https://libs.tech/rust/testing-libraries | 2026-08-01 |
| S14 | moldstud.com — Best Practices for Structuring Your Rust Tests | https://moldstud.com/articles/p-best-practices-for-structuring-your-rust-tests-a-comprehensive-guide | 2026-08-02 |
| S15 | corgea.com — Rust Best Practices 2026: Security, Idioms & Error Handling | https://corgea.com/learn/rust-security-best-practices | 2026-08-25 |

---

## What's NEW (687 vs 686)

### 1. Rust 2026.1 Stable Testing Enhancements

**Discovery**: Rust 1.75 stable (2026) includes:
- Enhanced `#[test]` macro with improved diagnostics and async testing support (S1, S2)
- `#[test_case]` macro for parameterized tests — recommended in 2026 docs (S1)
- Test runner now provides **detailed stack traces and performance metrics per test case** (S2)
- `cargo test --test-threads=1` for sequential deterministic runs (S14)

**Source**: S1, S2, S14

### 2. cargo-nextest Native IDE Integration

**Discovery**: RustRover 2026.1 ships native `cargo-nextest` integration (S12). nextest provides:
- Better parallel test execution than stock `cargo test`
- Per-test timeouts and retries
- Snapshot-based test result caching

**Source**: S12

### 3. proptest 1.11 Current Stable (2026)

**Discovery**: proptest 1.11 is the current stable release. Key 2026 facts:
- 140M+ downloads on crates.io (S3)
- `.proptest-regressions` file for regression persistence — **must be committed to git** (S3)
- `property_test` procedural macro for more concise test definitions (S4)
- proptest is "fairly close to being feature-complete" — **passive maintenance mode** (S5)
- proptest generating complex values is **up to 10x slower** than quickcheck (S6, S4)

**Source**: S3, S4, S5, S6

### 4. Testcontainers-rs 0.15 + OnceCell Shared Container Pattern

**Discovery**: testcontainers-rs 0.15 with `testcontainers-modules` crate. Key pattern:
- `OnceCell` for shared container instances across parallel tests (S9)
- Health check polling required before test logic (S8, S10)
- Proper drop-handling to avoid "dangling" containers (S10)
- wiremock 0.6 for HTTP service mocking (S8)

**Source**: S8, S9, S10, S11

### 5. 2026 Testing Best Practices

**Discovery**:
- Exhaustive property-based testing for core logic in CI (S1)
- Modular and reusable test suites (S1)
- Testcontainers for integration tests ensure "reliable and representative of production behavior" (S1)
- `cargo-tarpaulin` for coverage, `cargo-nextest` for speed (S13)
- Flaky tests cause "30% more debugging time" (S14)

**Source**: S1, S13, S14

---

## Defects Found (NEW)

### DEFECT-687-1: proptest Single-Value Edge Case Blindness

**Severity**: HIGH  
**Category**: Testing Gap  

**Description**: proptest cannot find single-value edge cases in large input spaces. The canonical example: `i64::MIN` for `.abs()` will "virtually always pass" because proptest randomly samples and the probability of hitting exactly one failing value in 2^64 space is negligible (S3, S4, S6).

**NeoTrix Impact**: NeoTrix's HyperCube and VSA embeddings operate in high-dimensional vector spaces. Edge cases like zero vectors, NaN propagation in embedding distance calculations, overflow in E8 hexagram state transitions, and boundary values in GWT attention scores are single-point failures that proptest will miss. Our `SelfTest` T1/T2/T3 tiers currently rely on example-based tests for edge cases — but if we adopt proptest for broader coverage, we must NOT abandon targeted unit tests for known singular values.

**Fix**:
- Maintain a dedicated `edge_cases` test module with manually crafted boundary values for all numeric/vector operations
- Use proptest `prop_assume!` to filter out degenerate inputs, but pair with explicit unit tests for i64::MIN-equivalent values
- Add a `SINGULAR_VALUES` constant array per domain type listing known edge cases

**Source**: S3, S4, S6

---

### DEFECT-687-2: proptest 10x Performance Regression on Complex Strategies

**Severity**: MEDIUM  
**Category**: CI Performance  

**Description**: proptest generating complex values (nested structs, constrained strategies, regex-based strings) is **up to an order of magnitude slower** than quickcheck (S6, S4). This is because proptest must hold all intermediate states and relationships for its richer shrinking model.

**NeoTrix Impact**: NeoTrix types are deeply nested — `ConsciousnessState` contains `E8Hexagram` (64-element array), `GWTAttentionWeights`, `VSAEmbedding` (high-dimensional), `EmotionLabel`, `SystemHealthSnapshot`. Property-testing these with `#[derive(Arbitrary)]` will generate slowly. CI pipelines could see test suite times balloon from minutes to tens of minutes.

**Fix**:
- Use quickcheck for simple types (scalars, small enums) where its leaner generation suffices
- Use proptest only for types where shrinking quality matters (complex nested structures)
- Set `PROPTEST_CASES=64` (down from default 256) for local dev, `PROPTEST_CASES=1024` for CI
- Consider `proptest`'s `#[proptest(max_size = N)]` to bound generated structure depth
- Profile test generation times; add a CI gate that fails if property tests exceed time budget

**Source**: S6, S4

---

### DEFECT-687-3: Missing .proptest-regressions Commit Discipline

**Severity**: HIGH  
**Category**: Regression Prevention  

**Description**: proptest persists failing seeds to `.proptest-regressions` file. If this file is not committed to version control, "a fixed bug cannot silently come back" becomes false — regressions reappear (S3).

**NeoTrix Impact**: NeoTrix has no `.proptest-regressions` file in the repo. If proptest is adopted for HyperCube invariants, VSA embedding properties, or SEAL pipeline state machine properties, any discovered bug could silently reappear after a force-push or branch reset.

**Fix**:
- Add `.proptest-regressions` to `.gitignore` exception (ensure it IS tracked)
- Add CI step: `test -f .proptest-regressions && echo "Regressions file present"` as a health check
- Document in `dev-rules.md`: "proptest-regressions file MUST be committed"

**Source**: S3

---

### DEFECT-687-4: Test Container Dangling Resource Leak

**Severity**: MEDIUM  
**Category**: Resource Management  

**Description**: Failed integration tests can leave Docker containers running ("dangling containers") that consume host resources. The standard cleanup via `drop` only works on normal test completion, not on panic/timeout (S10).

**NeoTrix Impact**: NeoTrix's NT-MEMORY domain (SQLite KB) and NT-WORLD (crawler with HTTP endpoints) may need container-based integration tests. If a test panics during KB schema migration or crawl pipeline setup, orphaned containers accumulate. Over time this causes disk space exhaustion and port conflicts in CI.

**Fix**:
- Use `testcontainers` with explicit `AsyncRunner` which handles cleanup on drop
- Add CI cleanup step: `docker container prune -f` between test runs
- Implement `TestContext` with explicit `shutdown()` method called in `#[test]` teardown
- Monitor CI for container count anomalies

**Source**: S10, S8

---

### DEFECT-687-5: Shared Once State Contamination in Parallel Tests

**Severity**: MEDIUM  
**Category**: Test Isolation  

**Description**: The `std::sync::Once` pattern for test initialization (S8) creates shared mutable state across parallel tests. If test A initializes a logger and test B expects a clean state, or if initialization has side effects, cross-test contamination occurs.

**NeoTrix Impact**: NeoTrix's `HeartbeatAggregator` and `ConsciousnessTree` have global singleton-like state. Using `Once` for test setup of these components means parallel test runs may interfere with each other's health snapshots or consciousness cycle state.

**Fix**:
- Replace `Once` with per-test isolation: each test gets its own `ConsciousnessTree` instance
- Use `serial_test` crate (already in S8 dev-dependencies) for tests that truly need global init
- Prefer constructor injection over global state in test fixtures
- Add CI flag `--test-threads=1` for tests that touch global state

**Source**: S8

---

### DEFECT-687-6: proptest Passive Maintenance Mode Risk

**Severity**: LOW  
**Category**: Dependency Risk  

**Description**: proptest is "fairly close to being feature-complete" and "mainly sees passive maintenance" (S5). While stable, this means no new features, slow bug fixes, and potential divergence from Rust nightly developments.

**NeoTrix Impact**: If proptest doesn't keep pace with Rust edition changes (e.g., 2024 edition async improvements, new trait system features), NeoTrix's property tests could break on toolchain upgrades with no upstream fix available.

**Fix**:
- Pin proptest version in `Cargo.toml` with explicit minor version
- Monitor proptest changelog for Rust compatibility notes
- Maintain a thin wrapper module `nt_test::property` that abstracts proptest — if migration to quickcheck or a successor is needed, changes are localized
- Evaluate `strict-proptest` fork (S6) as a contingency

**Source**: S5, S6

---

## Improvements Identified

### IMPROVEMENT-687-1: Adopt cargo-nextest for NeoTrix CI

**Impact**: HIGH  
**Effort**: LOW  

**Description**: cargo-nextest provides parallel test execution with per-test timeouts and retries. RustRover 2026.1 has native integration (S12). NeoTrix's test suite runs `cargo test` which is sequential by default.

**Action**:
- Add `cargo-nextest` to CI pipeline
- Configure `nextest.toml` with per-test timeout (30s default, 120s for integration tests)
- Enable retries for flaky tests (max 2 retries)
- Expected improvement: 40-60% reduction in CI test time

**Source**: S12, S13

---

### IMPROVEMENT-687-2: Parameterized Tests with #[test_case]

**Impact**: MEDIUM  
**Effort**: LOW  

**Description**: Rust 2026.1 recommends `#[test_case]` macro for parameterized tests (S1). NeoTrix's `SelfTest` implementations currently use separate `#[test]` functions for each case.

**Action**:
- Refactor `SelfTest` registration tests to use `#[test_case]` with inline data
- Reduces boilerplate for T1/T2/T3 tier verification
- Example: `#[test_case("nt_core_self" ; "Self module exists")]` instead of 7 separate test functions

**Source**: S1

---

### IMPROVEMENT-687-3: Enhanced Test Diagnostics from Rust 2026.1

**Impact**: MEDIUM  
**Effort**: LOW  

**Description**: Rust 2026.1 test runner provides detailed stack traces and performance metrics per test case (S2). NeoTrix can leverage this for faster debugging of test failures.

**Action**:
- Set `RUST_BACKTRACE=1` in CI test environment
- Enable `cargo test -- --show-output` for failed tests
- Use `#[instrument]` from tracing in test helpers for structured test logs
- Add test duration reporting to CI summaries

**Source**: S2

---

### IMPROVEMENT-687-4: Hybrid proptest + quickcheck Strategy

**Impact**: HIGH  
**Effort**: MEDIUM  

**Description**: proptest excels at shrinking and structured data; quickcheck excels at speed and simplicity (S3, S6, S7). NeoTrix should use both strategically.

**Action**:
- Scalar types (u32, f64, bool, enums): quickcheck (fast, simple)
- Complex nested types (ConsciousnessState, VSAEmbedding, HyperCube): proptest (good shrinking)
- Round-trip properties (serialize→deserialize, encode→decode): proptest (regression persistence)
- Document strategy selection in `dev-rules.md`

**Source**: S3, S6, S7

---

### IMPROVEMENT-687-5: Testcontainers for NT-MEMORY Integration Tests

**Impact**: HIGH  
**Effort**: MEDIUM  

**Description**: NT-MEMORY (SQLite KB) currently has no integration tests against real database instances. testcontainers-rs 0.15 provides isolated database containers (S8, S9, S11).

**Action**:
- Add `testcontainers` + `testcontainers-modules` to dev-dependencies
- Create `tests/common/db.rs` with SQLite/Postgres container setup
- Write integration tests for KB schema migrations, FTS5 search, embedding storage
- Use `OnceCell` for shared container across test file, not across parallel tests

**Source**: S8, S9, S11

---

## Summary

| Category | Count | Items |
|----------|-------|-------|
| New Defects | 6 | Single-value blindness, 10x perf regression, missing regressions file, dangling containers, shared state contamination, passive maintenance risk |
| Improvements | 5 | cargo-nextest, parameterized tests, enhanced diagnostics, hybrid proptest+quickcheck, testcontainers for KB |
| Sources | 15 | S1-S15 |

**Key Insight**: The testing landscape in 2026 has matured significantly — Rust 2026.1 provides better diagnostics, cargo-nextest is IDE-integrated, and proptest is feature-complete but passive. NeoTrix's main risk is adopting proptest blindly without accounting for its single-value edge case blindness and performance characteristics on complex types. The hybrid approach (proptest for shrinking-critical paths, quickcheck for speed-critical paths, targeted unit tests for singular values) is the 2026 optimal strategy.
