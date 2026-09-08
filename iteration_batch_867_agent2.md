# Agent 2: Async Error Handling (Batch 867)

## Sources

| # | Source | Key Topics |
|---|--------|------------|
| 1 | [drunkleen.com — Error Handling in Async Rust](https://drunkleen.com/posts/rust-async-error-handling) (2025-11) | Stack traces unreliable in async, orphan tasks, panic in async is a bug, thiserror/anyhow patterns |
| 2 | [sharpskill.dev — Rust Error Handling 2026](https://sharpskill.dev/en/blog/rust/rust-error-handling-result-option-thiserror-anyhow) (2026-07) | Result/Option, ? operator, thiserror vs anyhow, async error patterns identical to sync |
| 3 | [andrewodendaal.com — Production Error Patterns](https://andrewodendaal.com/rust-error-handling-patterns-production/) (2026-03) | ? is composable conversion pipeline, JoinHandle double-Result, unwrap is a bug in production |
| 4 | [medium.com — Async Error Best Practices](https://medium.com/@adamszpilewicz/error-handling-in-async-rust-best-practices-for-real-projects-46a2cce1cecc) (2025-09) | Context-rich errors, task panics, timeout double-Result, channel errors, graceful shutdowns |
| 5 | [botmonster.com — Defensive Error Patterns](https://botmonster.com/coding/defensive-coding-rust-error-handling-patterns/) (2026-04) | thiserror for libs, anyhow for apps, JoinHandle nested Result, async closures |
| 6 | [microsoft.github.io — Rust Patterns ch10](https://microsoft.github.io/RustTraining/rust-patterns-book/ch10-error-handling-patterns.html) | #[from] auto-generates From, .context() wraps, ? desugars to From::from + early return |
| 7 | [rs4ts.dev — Error Handling](https://rs4ts.dev/08-error-handling/) (2026-06) | Result as failure value not jump, unwrap only for provable invariants, error chain via source() |
| 8 | [rust-lang — RFC 3668 Async Closures](https://rust-lang.github.io/rfcs/3668-async-closures.html) | Async closures self-borrow, || async {} vs async || {}, error propagation boundaries |
| 9 | [anyhow crate docs](https://docs.rs/anyhow/latest/anyhow/) | .context()/.with_context() chains, bail!/ensure!, downcast_ref for recovery |
| 10 | [thiserror crate docs](https://docs.rs/thiserror/latest/thiserror/) | #[from] implies #[source], #[non_exhaustive] for future-proofing, error(transparent) |

## Defects

### D-ERR-001: NeoTrixError discarded error chain — `source()` always returns None

**File**: `neotrix-core/src/unified/core/nt_core_error.rs:86`

The unified error type implements `std::error::Error` with an empty body. This means `source()` always returns `None`, so the entire cause chain is lost when errors propagate. The `From<std::io::Error>` impl at line 89-91 calls `.to_string()` — converting the structured `io::Error` to a flat string, destroying its `ErrorKind`, raw OS error, and nested cause. When this error reaches the top-level reporter, debugging output is `"IO 错误: Connection refused (os error 111)"` instead of a chainable `source()` with `kind()` metadata. Production error diagnostics lose the original error type entirely.

**Severity**: HIGH — Silent information loss in every error propagation path through the unified type.

---

### D-ERR-002: NeoTrixError missing `#[non_exhaustive]` — every new variant is breaking change

**File**: `neotrix-core/src/unified/core/nt_core_error.rs:11`

The `NeoTrixError` enum has 17+ variants and is public across the entire codebase. Without `#[non_exhaustive]`, adding any new variant (e.g., for a new domain) is a semver-breaking change because downstream `match` statements will become non-exhaustive. The enum is already bloated — adding variants will cascade breakage. The `#[non_exhaustive]` attribute is the standard fix (as documented in [botmonster.com](https://botmonster.com/coding/defensive-coding-rust-error-handling-patterns/)), requiring callers to add a wildcard `_ =>` arm.

**Severity**: MEDIUM — Architectural debt; every error type addition creates churn.

---

### D-ERR-003: `From<String>` maps ALL string errors to Brain variant — error provenance destroyed

**File**: `neotrix-core/src/unified/core/nt_core_error.rs:94-103`

Both `From<String>` and `From<&str>` map to `NeoTrixError::Brain(msg)`. This means any `?` operator on a `Result<_, String>` anywhere in the codebase — regardless of whether the error originated from IO, network, shield, memory, or any other domain — gets silently classified as a "Brain error." The domain-specific discrimination that the enum was designed to provide is bypassed by this blanket conversion. Code using `Result<_, String>` loses all ability to distinguish error origins at compile time.

**Severity**: HIGH — Structural defect in error classification; defeats the purpose of a typed error enum.

---

### D-ERR-004: All async trait methods return `Result<_, String>` — no structured error matching

**File**: `neotrix-core/src/unified/core/l7_capability/traits.rs:13,21,22,25,26,34`

The core `CapabilityPlugin`, `EnergyCore`, and `WisdomBridge` async traits all return `Result<_, String>`. This means callers cannot pattern-match on error variants for retry logic, fallback behavior, or metrics labeling. The `execute()`, `receive_wisdom()`, `emit_action()`, `emerge()`, `process_energy()`, and `accumulate()` methods all erase error structure to a string. This propagates across every plugin implementation (74 async_trait usages found), making structured error recovery impossible at the capability layer.

**Severity**: HIGH — Prevents any retry/fallback/circuit-breaker logic based on error kind at the capability boundary.

---

### D-ERR-005: Spawned task silently drops errors in LLM stream relay

**File**: `neotrix-core/src/unified/core/nt_core_llm.rs:60-73`

In `stream_complete()`, a `tokio::spawn` relays streaming chunks from the provider to the consumer. When `tx.send(item).await.is_err()` (consumer dropped), the loop `break`s silently. But when `rx.recv()` returns `None` (provider finished), the spawned task also exits silently. The critical issue: if the provider's stream yields an error, it's forwarded via `tx.send(item).await`, but if the send itself fails, the error is silently dropped. More importantly, there's no logging or error reporting for the spawned relay task — if it panics, the `JoinError` is never awaited (orphan task). Best practice per [drunkleen.com](https://drunkleen.com/posts/rust-async-error-handling) and [medium.com](https://medium.com/@adamszpilewicz/error-handling-in-async-rust-best-practices-for-real-projects-46a2cce1cecc): "Always .await the JoinHandle and check for panic."

**Severity**: MEDIUM — Orphan spawned task; errors from stream relay vanish without trace.

---

### D-ERR-006: `ErrorType` is non-exhaustive enum without `#[non_exhaustive]` — same breaking change risk

**File**: `neotrix-core/src/unified/core/nt_core_error_recovery.rs:4-15`

The `ErrorType` enum (used for error recovery classification) has 9 variants and is not marked `#[non_exhaustive]`. The `ErrorRecoveryStrategy::can_handle()` method pattern-matches on it. Adding a new error type (inevitable as the system grows) will break all `can_handle()` implementations across the codebase. Same architectural debt as D-ERR-002 but in the recovery subsystem.

**Severity**: MEDIUM — Recovery strategy extensibility blocked by enum exhaustiveness.

---

### D-ERR-007: `std::thread::sleep` in async error recovery blocks the tokio runtime

**File**: `neotrix-core/src/unified/core/nt_core_observer_error.rs:59`

The `ObserverErrorRecovery::execute()` method uses `std::thread::sleep(Duration::from_millis(delay))` inside what should be an async context. Per [rustfaq.org](https://www.rustfaq.org/en/how-to-avoid-common-async-pitfalls-in-rust/): "Using `std::thread::sleep` inside async code puts the entire thread to sleep. The executor can't run other tasks on that thread." While `execute()` is currently sync, the surrounding system is async — if this is ever called from an async context (or if the `F: FnMut` closure calls async code), it will block the entire runtime. The method should be `async` with `tokio::time::sleep`.

**Severity**: MEDIUM — Latent blocking hazard; will deadlock if called from async context.

---

### D-ERR-008: `NeoTrixError` uses string-wrapped variants instead of structured inner errors

**File**: `neotrix-core/src/unified/core/nt_core_error.rs:12-48`

Every variant wraps a `String` (e.g., `Config(String)`, `Io(String)`, `Network(String)`). Per [sharpskill.dev](https://sharpskill.dev/en/blog/rust/rust-error-handling-result-option-thiserror-anyhow) and [botmonster.com](https://botmonster.com/coding/defensive-coding-rust-error-handling-patterns/), production error types should wrap the original error to preserve `source()` chains and enable structured recovery. The current design forces all error context into flat strings. A proper design would use `#[from]` or `#[source]` fields wrapping the original error types (e.g., `Io(#[from] std::io::Error)` instead of `Io(String)`).

**Severity**: HIGH — Fundamental design flaw; prevents error chaining, source walking, and type-safe recovery.

---

### D-ERR-009: Async closure `|| async {}` pattern causes silent borrow lifetime failures

**File**: `neotrix-core/src/unified/core/nt_core_llm.rs:60` (spawned async block)

The pattern `tokio::spawn(async move { ... })` is used throughout the codebase for async task spawning. Per [RFC 3668](https://rust-lang.github.io/rfcs/3668-async-closures.html), `|| async {}` (closure returning async block) differs from `async || {}` (async closure) in borrow semantics. The closure+async-block form cannot self-borrow captured data, requiring `clone()` or `move` on every capture. This is a known pitfall — the async block captures by `move`, which can silently cause data races if the same data is shared across spawned tasks without `Arc`. The codebase has 74 `#[async_trait]` usages and many `tokio::spawn` sites, any of which may silently lose borrow safety.

**Severity**: LOW — Structural awareness issue; individual sites need case-by-case audit.

---

### D-ERR-010: `panic!()` in async test code — if test harness panics, runtime unrecoverable

**File**: `neotrix-core/src/unified/layers/action/nt_io/nt_io_provider/gateway/mod.rs:1284`

In the streaming E2E test, a channel receive error triggers `panic!("stream error: {:?}", e)`. While this is in a `#[cfg(test)]` block, the panic propagates through the tokio runtime. If this panic were ever accidentally left in production code, the entire tokio runtime would enter an inconsistent state. Per [drunkleen.com](https://drunkleen.com/posts/rust-async-error-handling): "Panic in async is almost always a bug, not a normal error." The `nt_core_self_review` scanner (line 567-575) already detects `panic!()` in production code, but test code is excluded from this check — and this specific pattern demonstrates how easily panic-based error handling leaks into production.

**Severity**: LOW — Test code, but establishes dangerous pattern.

---

## Key Insights

1. **The unified error type (`NeoTrixError`) is the root cause of most async error handling defects.** Its string-wrapped variants, missing `source()` chain, and blanket `From<String>` → Brain conversion create a structural bottleneck that propagates poor error handling to every module.

2. **The capability layer is blind to error types.** All async trait methods return `Result<_, String>`, making it impossible to implement retry, fallback, or circuit-breaker logic based on error kind. This is the single most impactful defect for production reliability.

3. **The error recovery system uses sync primitives.** `std::thread::sleep` in error recovery is a latent deadlock if ever called from async context. The entire recovery stack should be async-aware.

4. **Spawned tasks are fire-and-forget.** The LLM stream relay and other `tokio::spawn` calls never await their JoinHandle, violating the "always .await spawned tasks" best practice.

5. **The codebase correctly uses `thiserror` in leaf modules** (file_ability, eval_harness, task_dispatcher, etc.) but the core error type and capability traits bypass it. The fix is to make `NeoTrixError` use `thiserror` with `#[from]`/`#[source]` and add `#[non_exhaustive]`.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources consulted | 10 |
| Severity: HIGH | 4 |
| Severity: MEDIUM | 4 |
| Severity: LOW | 2 |
