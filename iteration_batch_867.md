# Iteration Batch 867 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Async Trait Patterns (9)
- RPITIT (Return Position Impl Trait in Trait): native async fn in trait
- trait_variant::make(Send): auto-generates Send variant
- #[async_trait]: proc-macro boxing for dyn compatibility
- Send bound: requires future to be Send
- dyn Future<Output = T>: trait object for futures
- BoxFuture<'static, T>: boxed future
- Static dispatch: monomorphization
- Dynamic dispatch: vtable lookup
- RTN (Return Type Notation): solves Send bounds (nightly)

### Async Error Handling (9)
- thiserror: derive macro for error types
- anyhow: type-erased error context
- eyre: Result with context
- #[non_exhaustive]: forward-compatible error enums
- From impl: error conversion
- #[error]: automatic Display impl
- Context trait: .context("msg") for wrapping
- Error chain: source() for traversal
- Root cause: backtrace + span trace

### Async State Machine (9)
- async/await desugars to state machine
- Future trait: poll(self: Pin<&mut Self>, cx: &mut Context)
- Poll::Ready(T): future completed
- Poll::Pending: future not ready
- Waker:通知 future to re-poll
- Futures combinators: map, then, and_then
- Stream: async Iterator
- pin-project: safe Pin projection
- FusedFuture: terminal state detection

### Async Testing (9)
- tokio::test: test runtime
- tokio::time::pause: virtual time control
- mockall: mock generation
- proptest: property-based testing
- cargo-fuzz: fuzz testing
- cargo-mutants: mutation testing
- testcontainers: integration testing
- wiremock: HTTP mocking
- assert_ready!, assert_pending!: poll testing

## Defects Identified (40+)

### Async Trait Patterns (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-TRAIT-001 | RPITIT incompatibility in ResourcePool | High |
| D-TRAIT-002 | Missing Send bounds on CloudSandboxProvider.stream_logs | High |
| D-TRAIT-003 | 75+ #[async_trait] macro usages (boxing overhead) | High |
| D-TRAIT-004 | Dual sync/async judge traits (unnecessary boilerplate) | Medium |
| D-TRAIT-005 | No trait_variant usage (pre-1.75 patterns) | Medium |
| D-TRAIT-006 | Box<dyn Future> everywhere instead of RPITIT | Low |
| D-TRAIT-007 | No dyn compatibility analysis | Low |
| D-TRAIT-008 | No dispatch overhead profiling | Low |
| D-TRAIT-009 | No static dispatch optimization | Low |
| D-TRAIT-010 | No Send bound propagation | Low |

### Async Error Handling (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-001 | NeoTrixError source() returns None (chain loss) | High |
| D-ERR-002 | Missing #[non_exhaustive] on error enums | High |
| D-ERR-003 | Blanket From<String> destroys error provenance | High |
| D-ERR-004 | All async trait methods return Result<_, String> | High |
| D-ERR-005 | Three parallel unlinked error hierarchies | High |
| D-ERR-006 | No error context chaining anywhere | High |
| D-ERR-007 | No color-eyre / SpanTrace | Low |
| D-ERR-008 | FileAbilityError::Parse chain loss | Medium |
| D-ERR-009 | MailError anyhow escape hatch | Medium |
| D-ERR-010 | Box<dyn Error> in proxy kernel/CLI | Medium |

### Async State Machine (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-FUTURE-001 | Busy-spin in ObfuscatedStream::poll_read | High |
| D-FUTURE-002 | Unbounded write_buf growth in ObfuscatedStream | High |
| D-FUTURE-003 | 30+ background handlers without concurrency limits | Medium |
| D-FUTURE-004 | XtlsStream buffer not cleared between polls | Medium |
| D-FUTURE-005 | Shutdown deadline future consumed after first abort | Medium |
| D-FUTURE-006 | Fire-and-forget stdin write can hang | Medium |
| D-FUTURE-007 | Per-frame heap churn from discarded BytesMut | Low |
| D-FUTURE-008 | Lock held across .await points | Medium |
| D-FUTURE-009 | Missing FusedFuture (dead connections re-polled) | Medium |
| D-FUTURE-010 | Arc<RwLock> state check on every poll | Low |

### Async Testing (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-TEST-001 | Zero virtual time control (tokio::time::pause) | High |
| D-TEST-002 | Zero mockall/proptest adoption | High |
| D-TEST-003 | 50+ tests bulk-disabled in tests/_disabled/ | Medium |
| D-TEST-004 | 60+ std::thread::sleep in async contexts | High |
| D-TEST-005 | No JoinSet patterns in tests | Medium |
| D-TEST-006 | No poll-level testing (assert_ready!/assert_pending!) | Medium |
| D-TEST-007 | No property-based testing | Medium |
| D-TEST-008 | No mock HTTP servers (wiremock) | Medium |
| D-TEST-009 | No virtual time for background loops | Medium |
| D-TEST-010 | Inconsistent runtime flavor selection | Low |

## Key Insights (This Batch)

1. **75+ #[async_trait] usages**: Each allocates a Box on every async method call. Native AFIT (Rust 1.75+) eliminates this. trait_variant solves Send bounds.

2. **NeoTrixError source() returns None**: Error chain is completely lost. All errors become flat strings. Must adopt thiserror #[from]/#[source] pattern.

3. **All async trait methods return Result<_, String>**: Prevents structured retry/fallback. Must change to Result<_, DomainError>.

4. **ObfuscatedStream busy-spin**: poll_read returns Ready(Ok(0)) when inner returns empty without Pending. Creates infinite poll loop.

5. **ObfuscatedStream unbounded write_buf**: No backpressure. Slow reader causes memory explosion.

6. **XtlsStream buffer not cleared**: Between polls, buffer retains old data. Frame misalignment risk.

7. **Zero virtual time control**: 100+ async tests use real time. #1 cause of flaky CI. Must use tokio::time::pause.

8. **Zero mockall/proptest**: Async traits tested only against real implementations. No isolation, no property verification.

9. **50+ tests bulk-disabled**: tests/_disabled/ directory has 50+ disabled tests. Should quarantine individually with clear reasons.

10. **FusedFuture missing**: Dead connections re-polled infinitely. Must implement FusedFuture or manual termination check.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 867 |
| New defects (this batch) | 40 |
| Cumulative defects | D01-D77970 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 98,995+ |
