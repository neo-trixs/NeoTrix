# Agent 4: Async Testing Patterns (Batch 867)

## Sources

1. **HelpMeTest** — "Rust Async Test Patterns: Testing Futures, Tokio Tasks, and Concurrent Code" (2026-05-20)
   - https://helpmetest.com/blog/rust-async-test-patterns/
2. **HelpMeTest** — "tokio-test: Testing Async Rust Without a Full Runtime" (2026-05-20)
   - https://helpmetest.com/blog/tokio-test-async-rust-patterns/
3. **HelpMeTest** — "Testing Async Rust with Tokio: Unit Tests, Mocking, and Timeouts" (2026-05-16)
   - https://helpmetest.com/blog/tokio-async-testing-rust
4. **Tokio Official** — "Unit Testing" documentation
   - https://tokio.rs/tokio/topics/testing
5. **Tokio Docs.rs** — `#[tokio::test]` macro reference
   - https://docs.rs/tokio/latest/tokio/attr.test.html
6. **Mergify** — "Flaky tests in Rust: every pattern that breaks CI, and how to kill them" (2026-04-27)
   - https://mergify.com/learn/flaky-tests/rust
7. **Mockall Docs.rs** — "A powerful mock object library for Rust"
   - https://docs.rs/mockall/latest/mockall/
8. **Mockall GitHub** — async trait support documentation
   - https://github.com/asomers/mockall
9. **proptest_async Docs.rs** — "Macros proptest — adjustment for async tests via async_std"
   - https://docs.rs/proptest_async
10. **testkit-async GitHub** — "Practical testing tools for async Rust"
    - https://github.com/ibrahimcesar/testkit-async
11. **Rust FAQ** — "How to Write Async Tests in Rust (with tokio::test)" (2026-04-17)
    - https://www.rustfaq.org/en/how-to-write-async-tests-in-rust-with-tokiotest/
12. **MoldStud** — "Mastering Asynchronous Testing in Rust — Strategies and Frameworks with Tokio" (2026-08-02)
    - https://moldstud.com/articles/p-mastering-asynchronous-testing-in-rust-strategies-and-frameworks-with-tokio
13. **DasRoot** — "Rust Testing Patterns for Reliable Releases" (2026-03-02)
    - https://dasroot.net/posts/2026/03/rust-testing-patterns-reliable-releases/
14. **DasRoot** — "Rust & Async: Practical Patterns for High-Performance Tools" (2026-03-02)
    - https://dasroot.net/posts/2026/02/rust-async-practical-patterns-high-performance-tools/
15. **One Horizon** — "Modern Rust Best Practices in 2026: Beyond the Borrow Checker" (2026-01-08)
    - https://onehorizon.ai/blog/modern-rust-best-practices-in-2026-beyond-the-borrow-checker
16. **ArXiv** — "A Preliminary Study of Fixed Flaky Tests in Rust Projects on GitHub" (2025-02-04)
    - https://arxiv.org/html/2502.02760v1
17. **test-strategy crate** — Async proptest with `#[proptest(async = "tokio")]`
    - https://rustprojectprimer.com/testing/property.html

## Defects

### D-TEST-001: Zero `tokio::time::pause()` / `start_paused` usage across all async tests — timing-sensitive tests are inherently flaky
| Severity | Source |
|----------|--------|
| HIGH | Mergify flaky-tests (pattern #4), Tokio official docs, HelpMeTest |

NeoTrix has 100+ `#[tokio::test]` functions across the codebase (nt_file_ability, nt_shield_stealth_net, nt_core_parallel, nt_world_media_source, etc.) but **zero** calls to `tokio::time::pause()`, `tokio::time::advance()`, or `start_paused = true` in test code. The only "tokio_test" reference is a `todo!("tokio")` placeholder in `nt_core_self_review/tests.rs:250`.

Meanwhile, production code extensively uses `std::thread::sleep` (60+ occurrences) and `tokio::time::sleep` for retry/backoff logic across fetchers, proxy pools, crawl delay, stealth browser, self-heal loops, and the reasoning engine. Without virtual time control, any test that exercises timeout or retry paths will either:
- Wait for real wall-clock time (slow CI)
- Race against scheduler jitter (flaky on CI runners)
- Miss assertion windows due to non-deterministic scheduling

**File affected**: All 100+ `#[tokio::test]` sites across `neotrix-core/src/`
**Fix**: Add `start_paused = true` to `#[tokio::test]` for time-dependent tests; use `tokio::time::advance()` for deterministic time-dependent assertions. Create a shared test helper `nt_test_utils` crate with time-paused test wrappers.

---

### D-TEST-002: No mock framework (mockall/proptest) adopted — async trait testing gaps in NT-CORE, NT-MIND, NT-WORLD
| Severity | Source |
|----------|--------|
| HIGH | Mockall docs, proptest_async docs, DasRoot testing patterns, testkit-async comparison table |

Zero usage of `mockall` or `proptest` anywhere in the codebase. The grep returns no matches. This means:

1. **Async trait mocking**: NeoTrix defines extensive async traits across all domains (PerceptionBridge, CapabilityBridge, SocialSource, the LLM provider interfaces in nt_io_provider, crawl strategies, etc.). None are tested in isolation with mocks — tests either use real implementations or are disabled entirely (`tests/_disabled/` directory).

2. **Property-based testing**: Core logic in HyperCube, E8 hexagram reasoning, VSA embedding math, SEAL pipeline state machines, and EmotionLabel routing have no proptest coverage. These are precisely the kinds of data-transformation logic that benefit from randomized input generation and automatic shrinking.

3. **mockall macro ordering**: Per the mockall docs, `#[automock]` must appear *before* `#[async_trait]` for correct behavior. This ordering constraint is non-obvious and will trap anyone introducing mocks later.

**File affected**: `neotrix-core/src/unified/layers/cognition/nt_core/`, `neotrix-core/src/unified/layers/cognition/nt_mind/`, all trait definitions in `neotrix-core/src/unified/layers/perception/nt_world/`
**Fix**: Adopt mockall for async trait testing; add proptest for data-transformation invariants. Ensure `#[automock]` appears before `#[async_trait]` in all mock definitions.

---

### D-TEST-003: Disabled test suite (tests/_disabled/) — 50+ async tests silently skipped in CI
| Severity | Source |
|----------|--------|
| HIGH | HelpMeTest (test isolation), ArXiv flaky-tests study, Mergify pattern #7 |

The `tests/_disabled/` directory contains three large test files with 50+ `#[tokio::test]` functions:
- `stress_test.rs` (11 tests)
- `e2e_complete_flow.rs` (25+ tests)
- `security_test.rs` (12 tests)

These tests are completely excluded from CI. Per the ArXiv study, 38% of Rust flaky tests stem from "improper wait" — tests that don't properly wait for async operations to complete. Disabling entire test files rather than fixing individual flaky tests creates a blind spot where regressions accumulate silently. The stress_test and security_test files are particularly critical for NT-SHIELD and NT-ACT domain reliability.

**File affected**: `tests/_disabled/stress_test.rs`, `tests/_disabled/e2e_complete_flow.rs`, `tests/_disabled/security_test.rs`
**Fix**: Quarantine individual flaky tests (using `#[ignore]` with reasons) rather than disabling entire files. Add `start_paused = true` and mock external dependencies to stabilize each test before re-enabling.

---

### D-TEST-004: `std::thread::sleep` used in async contexts — blocks Tokio worker threads in production and tests
| Severity | Source |
|----------|--------|
| MEDIUM | One Horizon async best practices ("The Sync in Async Sin"), DasRoot async patterns, Tokio docs |

60+ occurrences of `std::thread::sleep` found in production code, many in async contexts:
- `nt_core_event_bus.rs:464,645` — sleep in async event loop
- `nt_core_forecast.rs:440,486` — blocking sleep in retry loop
- `nt_world_crawl/stealth.rs:264` — blocking sleep for crawl delay
- `nt_world_crawl/fetcher.rs:265,278,291,408` — blocking sleep for backoff
- `nt_world_crawl/unified.rs:355` — blocking sleep for rate limiting
- `nt_world_browse/session.rs:64,115` — blocking sleep in browse session
- `nt_world_scrape.rs:287,348` — blocking sleep in scrape loop
- `nt_mind/infrastructure/code_graph.rs:365` — blocking sleep in code graph
- `nt_mind/evolution/self_evolver.rs:184` — blocking sleep in evolution loop

Per 2026 best practices: "The quickest way to kill your Rust service's performance is to perform a blocking operation inside an async function. This blocks the Tokio worker thread." The `tokio::task::spawn_blocking` pattern should be used instead. These blocking sleeps make it impossible to write deterministic async tests — if the code blocks the worker thread, `tokio::time::pause()` cannot virtualize the sleep.

**File affected**: `nt_core_event_bus.rs`, `nt_core_forecast.rs`, `nt_world_crawl/stealth.rs`, `nt_world_crawl/fetcher.rs`, `nt_world_browse/session.rs`, `nt_world_scrape.rs`, `nt_mind/infrastructure/code_graph.rs`, `nt_mind/evolution/self_evolver.rs`
**Fix**: Replace `std::thread::sleep` with `tokio::time::sleep(...).await` in async contexts, or wrap in `tokio::task::spawn_blocking` for truly blocking operations.

---

### D-TEST-005: No JoinSet/JoinHandle panic propagation in concurrent tests — silent task failures
| Severity | Source |
|----------|--------|
| MEDIUM | HelpMeTest ("Always propagate task panics with .unwrap() on JoinHandle results"), Mergify pattern #3 |

The Mergify flaky-tests analysis identifies "Tokio runtime per-test races" as pattern #3: "A test that `spawn`s a long-running task and returns before the task finishes leaves an orphan task on a runtime that is being dropped." The NeoTrix codebase has 50+ `tokio::spawn` calls in production code but no systematic JoinHandle/JoinSet collection pattern in tests. The `nt_core_parallel/coordinator.rs` tests (lines 635-756) do test parallel execution but without JoinSet-based result collection for spawned tasks.

HelpMeTest's key takeaway: "Always propagate task panics with `.unwrap()` on JoinHandle results — otherwise test failures silently disappear."

**File affected**: `nt_core_parallel/coordinator.rs:635-756`, `nt_core_parallel/tests.rs:34-45`, all test files using `tokio::spawn`
**Fix**: Use `JoinSet` to collect spawned task results in tests; always `.await` and `.unwrap()` JoinHandles to surface panics. Add a test lint that detects spawned-but-not-joined tasks.

---

### D-TEST-006: No `tokio_test` assert_ready!/assert_pending! usage — Future state testing absent
| Severity | Source |
|----------|--------|
| MEDIUM | HelpMeTest tokio-test patterns, Tokio docs, moldstud async testing |

The `tokio-test` crate provides `assert_ready!`, `assert_pending!`, and `task::spawn` for step-by-step Future polling. The NeoTrix codebase has zero usage of these primitives. Only one reference exists — a `todo!("tokio")` in `nt_core_self_review/tests.rs:250`.

This means custom Futures (ConsciousnessTree growth cycles, SEAL pipeline stages, GWT attention routing) are never tested at the poll level. Race conditions between poll states, waker correctness, and poll-after-complete panics are all undetectable without poll-level testing.

**File affected**: `nt_core_self_review/tests.rs:250`, all custom Future implementations across `nt_core/`, `nt_mind/`
**Fix**: Add `tokio-test` as dev-dependency; use `assert_ready!`/`assert_pending!` for custom Future implementations; use `task::spawn` + `poll()` for step-by-step execution testing.

---

### D-TEST-007: No property-based testing for HyperCube/E8/VSA data transformations
| Severity | Source |
|----------|--------|
| MEDIUM | proptest docs, DasRoot testing patterns ("property-based testing for logic"), test-strategy crate |

The E8 hexagram engine, HyperCube knowledge representation, and VSA embedding operations are pure data transformations with mathematical invariants (e.g., associativity, identity elements, dimension consistency). Property-based testing with proptest is the industry-standard approach for such code.

Per the Rust Project Primer: "Property testing generates hundreds of randomized inputs per test, asserts an invariant holds for all of them, and automatically shrinks any failing case to a minimal counterexample." The `test-strategy` crate enables `#[proptest(async = "tokio")]` for async property tests.

**File affected**: `neotrix-core/src/core/` (E8, HyperCube, VSA modules), SEAL pipeline state machines
**Fix**: Add proptest dev-dependencies; write property tests for HyperCube operations (commutativity, associativity), E8 hexagram state transitions, and VSA embedding round-trip invariants. Use `test-strategy` crate for async property tests.

---

### D-TEST-008: Test files in `tests/` directory use real network calls — no mock HTTP server
| Severity | Source |
|----------|--------|
| LOW | MoldStud async testing ("Failing to mock dependencies"), LogRocket mockall patterns, wiremock examples |

Files like `test_stealth_net_e2e.rs`, `mail_integration_test.rs`, `test_doc_parse_gateway.rs`, and `nt_file_ability/integration_tests.rs` appear to make real network calls (based on their names and the project's LLM provider integration). No wiremock or mock HTTP server usage found in the codebase. Per MoldStud: "Real dependencies can lead to flakiness."

**File affected**: `neotrix-core/tests/test_stealth_net_e2e.rs`, `neotrix-core/tests/mail_integration_test.rs`, `neotrix-core/tests/test_doc_parse_gateway.rs`
**Fix**: Use `wiremock::MockServer` for HTTP mocking in integration tests; use `testcontainers` for database-dependent tests. Add `#[ignore]` with reasons for tests requiring real network access.

---

### D-TEST-009: No mock clock for NT-MIND background loop timer testing
| Severity | Source |
|----------|--------|
| LOW | testkit-async design philosophy, Tokio time::pause docs |

The NT-MIND background loop (`nt_mind_background_loop/run.rs:738,894`) spawns timer-based tasks (absorption cycles, experience-tree 60s ticks). These time-dependent loops have no test coverage using virtual time. Without `tokio::time::pause()`, testing timer-triggered behavior would require waiting for real seconds.

**File affected**: `nt_mind_background_loop/run.rs:738,894`, `nt_mind_background_loop/handlers.rs:11`
**Fix**: Write tests with `#[tokio::test(start_paused = true)]` that exercise the background loop's timer-triggered handlers; use `tokio::time::advance(Duration::from_secs(61))` to trigger absorption cycles deterministically.

---

### D-TEST-010: Inconsistent test runtime flavors — no systematic current_thread vs multi_thread selection
| Severity | Source |
|----------|--------|
| LOW | Mergify pattern #3, Rust FAQ ("Pick the flavor that matches your concurrency needs"), Tokio docs |

The codebase uses `#[tokio::test]` (default current_thread) everywhere without explicit `flavor` selection. Per best practices: current_thread should be used for deterministic testing with `RefCell`/`Rc`, while multi_thread should be used when testing true parallelism. The NT-CORE parallel executor tests (`nt_core_parallel/`) test concurrent task scheduling but run on current_thread, meaning they never exercise real thread-pool scheduling.

**File affected**: `nt_core_parallel/tests.rs`, `nt_core_parallel/coordinator.rs`
**Fix**: Use `flavor = "multi_thread"` for concurrency stress tests; keep current_thread for unit tests requiring determinism. Document the convention in dev-rules.md.

## Key Insights

1. **The biggest gap is virtual time control**: 100+ async tests with zero `tokio::time::pause()` usage means every time-dependent test is either slow or flaky. This is the #1 pattern behind flaky Rust CI per Mergify's analysis.

2. **No mocking or property testing is a systemic issue**: NeoTrix's async traits (LLM providers, crawlers, SocialSource, etc.) are tested only against real implementations. This makes it impossible to test error paths, edge cases, and race conditions in isolation.

3. **std::thread::sleep in async contexts is widespread**: 60+ occurrences block Tokio worker threads, defeating the async runtime and making deterministic testing impossible. This is the "sync-in-async sin" that 2026 best practices explicitly warn against.

4. **Disabled test suite is a liability**: 50+ tests in `tests/_disabled/` should be quarantined individually, not bulk-disabled. Each flaky test needs virtual time, mocks, or isolation fixes.

5. **Custom Future implementations have no poll-level testing**: The ConsciousnessTree, SEAL pipeline, and GWT modules implement custom Futures that are never tested at the `poll()` level using `assert_ready!`/`assert_pending!`.

6. **The missing toolchain**: mockall + proptest + tokio-test + wiremock form the standard async testing quartet for 2026 Rust. NeoTrix uses none of them.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources consulted | 17 |
| Files with `#[tokio::test]` | 100+ (across 15+ files) |
| Files with `std::thread::sleep` in async contexts | 20+ |
| Disabled test files | 3 (50+ tests) |
| Mock framework adoption | 0 (mockall, proptest, wiremock) |
| `tokio::time::pause()` usage in tests | 0 |
| `tokio_test` assert_ready/pending usage | 0 |
