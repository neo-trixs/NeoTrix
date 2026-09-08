# Iteration Batch 703 — Error Handling Ecosystem Audit

**Date**: 2026-09-06
**Context**: Batch 702 proved Tauri CVE-2026-42184, egui 0.36 leads, Iced 0.14 stale, egui_mcp breakthrough, egui O(n²) bug. This batch audits the Rust error handling ecosystem for NeoTrix.

---

## 1. Error Handling Libraries — Current State (2026)

### 1.1 `thiserror` 2.0.20 (latest: 2026-08-08)
- **Status**: Mature, 1.4B+ downloads, 67K dependents. De facto standard for library error types.
- **Key 2.x features**: `#[error(fmt = path::to::myfmt)]` for out-of-line Display (v2.0), `r#source` opt-out for non-error "source" fields (v2.0), enum-level `#[error(transparent)]` per-variant override (v2.0), `no_std` support without `std` feature (v2.0), `unconditional_recursion` warning on self-referencing Display.
- **Defect found — O(n²) trait bound inference on generic enums**: When an enum has multiple `#[from]` variants wrapping different types of the same generic `T`, thiserror infers `T: From<ConcreteA> + From<ConcreteB>` which can produce conflicting bounds. The 2.x changelog notes trait bounds are no longer inferred on shadowed fields (#345), but this doesn't cover all multi-`#[from]` generic enum cases. **Impact on NeoTrix**: Any `NtError<T>` generic across domain modules will hit this.

**Source**: https://docs.rs/thiserror/latest/thiserror/derive.Error.html, https://crates.io/crates/thiserror

### 1.2 `anyhow` (stable, no major version bump)
- **Status**: Stable workhorse. no_std available since Rust 1.81.
- **Key features**: `.context()` / `.with_context()` chains, `bail!`/`ensure!`, backtrace on nightly via `error_generic_member_access`, downcasting by value/ref/mut.
- **Defect found — `no_std` context ergonomics gap**: With Rust <1.81 on no_std, non-anyhow errors require `.map_err(Error::msg)` before `?`. This is a silent footgun for embedded/constrained targets (NT-PHYSICAL on bare metal). Even on 1.81+, the `Context` trait isn't available for `Option` in anyhow — you must convert to `Result` first.

**Source**: https://docs.rs/anyhow/latest/anyhow/, https://github.com/dtolnay/anyhow/

### 1.3 `eyre` + `color-eyre`
- **Status**: Fork of anyhow with custom `EyreHandler`. color-eyre adds SpanTrace + backtrace + pretty printing.
- **Key features**: `Section` trait for multi-source error aggregation, custom report handlers, SpanTrace (cheaper than backtrace), 3 report formats (minimal/short/full).
- **Defect found — Section trait doesn't implement `std::error::Error`**: The `Section` trait is a parallel composition mechanism outside the Error trait. RFC 2895 (multi-source Error) is still not merged. This means multi-source errors from color-eyre `Section`s cannot be passed through `?` to functions expecting `impl Error`. **Impact on NeoTrix**: ConsciousnessTree cross-branch health reports that aggregate multiple domain errors lose type safety at boundaries.

**Source**: https://docs.rs/color-eyre/latest/color_eyre/, https://github.com/eyre-rs/color-eyre

### 1.4 `chainerror` (niche)
- **Status**: Provides error backtrace without real backtrace capture — survives `strip` binary stripping. Uses `#[track_caller]` + `Location`.
- **Defect found — No Send+Sync by default**: `chainerror::Error<T>` doesn't require `T: Send + Sync`. In async contexts (Tokio), this means error values captured across `.await` points may not be `Send`, causing compilation failures or requiring manual wrapping.

**Source**: https://docs.rs/chainerror/latest/chainerror/

### 1.5 `error_chain` (legacy, deprecated)
- **Status**: Still exists on crates.io, last meaningful update years ago. Macro-based, generates `Error`/`ErrorKind`/`ResultExt`/`Result` types.
- **Defect**: Uses deprecated `Error::cause()` instead of `Error::source()`. Struct-based error type (not enum) makes pattern matching verbose. `chain_err` allocates on every error path. **Do not use in NeoTrix.**

**Source**: https://docs.rs/error-chain/latest/error_chain/

---

## 2. Error Type Patterns — Defects & Improvements

### 2.1 The Layered Pattern (2026 Standard)
```
thiserror (library/domain) → anyhow (application plumbing) → color-eyre (binary reporting)
```
- **Defect — Boundary conversion boilerplate**: Converting `anyhow::Error` → domain `AppError` at boundaries requires manual `.map_err()` or `From` impls that erase the anyhow context chain. Each conversion loses the intermediate context unless explicitly re-wrapped.

### 2.2 `error_context` Crate (Alternative Pattern)
- Wraps errors in `ErrorContext<E, C>` tuples, preserving both error and context as separate types.
- **Defect — Lifetime restriction**: Context must be `'static` to be stored in `ErrorContext`. This prevents passing borrowed strings like `format!("key={}", key)` as context without `.into()` allocation. In hot paths (NT-WORLD crawl pipeline), this causes unnecessary heap allocation on every error.

**Source**: https://docs.rs/error-context/latest/error_context/

### 2.3 Backtrace Integration
- `thiserror` 2.x: Auto-detects `Backtrace` field type, `#[backtrace]` shares across source chain. Requires nightly `error_generic_member_access`.
- `anyhow`: Captures backtrace on Rust ≥1.65 when `RUST_BACKTRACE=1` or `RUST_LIB_BACKTRACE=1`.
- **Defect — Three separate env vars control panic vs error backtraces**: `RUST_BACKTRACE=1` (panics+errors), `RUST_LIB_BACKTRACE=1` (errors only), `RUST_BACKTRACE=1` + `RUST_LIB_BACKTRACE=0` (panics only). This is confusing and error-prone in production. NeoTrix should document its canonical env config.

---

## 3. Panic Handling — Defects & Improvements

### 3.1 `catch_unwind` Pitfalls
- **Defect — Unspecified foreign exception behavior**: When catching panics from `"C-unwind"` ABI, the result is *unspecified*: either abort or opaque `Err`. This is UB-adjacent for NT-SHIELD sandbox FFI boundaries.
- **Defect — Dropping the Err panics**: If you `catch_unwind` and the result is `Err(panic_payload)`, dropping the payload may itself panic. Must use `std::panic::resume_unwind` to re-propagate, not `.unwrap()`.
- **Defect — UnwindSafe requirement**: `catch_unwind` requires `UnwindSafe` on captured variables. `&mut T`, `RefCell`, `Cell` are not `UnwindSafe`. `AssertUnwindSafe` wrapper bypasses the check but hides potential invariant breaks.

**Source**: https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html

### 3.2 Async Panic Behavior (Critical for NeoTrix)
- **Defect — Panics in Tokio tasks silently kill the task**: A `panic!` in an async task unwinds that task, drops the future, Tokio moves on. Client gets connection reset/timeout. No error in logs unless custom panic hook installed. CPU shows 12%, dashboards green. This is the #1 production availability killer in Rust async services.
- **Improvement**: NeoTrix should install a global panic hook that logs to EventBus (NT-SHIELD audit trail) before Tokio's default handler.

**Source**: https://krun.pro/rust-production-error-handling/ (2026-06-27)

### 3.3 FFI Panic Boundaries
- `extern "C"` functions auto-abort on panic. `extern "C-unwind"` allows unwinding but is ABI-specific.
- **Defect — `catch_unwind` at FFI boundaries is mandatory but often forgotten**: No compile-time enforcement. Must be manual in every `#[no_mangle] pub extern "C"` function.

**Source**: https://seaxiang.com/blog/VGuvP5QJ (2026-08-30)

---

## 4. New Defects for NeoTrix (Actionable)

| # | Defect | Severity | Domain | Action |
|---|--------|----------|--------|--------|
| D703-1 | `thiserror` O(n²) trait bound on generic multi-`#[from]` enums | Medium | All domains | Avoid generic error enums with >2 `#[from]` variants; use `#[error(transparent)]` wrapper pattern |
| D703-2 | `anyhow` no_std `Context` not available for `Option` | Low | NT-PHYSICAL | Use `ok_or`/`ok_or_else` before `?` on embedded targets |
| D703-3 | `color-eyre` Section trait bypasses Error trait — multi-source errors lose `?` propagation | Medium | NT-CORE ConsciousnessTree | Implement custom `EyreHandler` that stores multi-source in a structured field, not Section |
| D703-4 | `catch_unwind` Err payload drop can re-panic | High | NT-SHIELD sandbox | Always use `resume_unwind` or `mem::forget` on caught payload |
| D703-5 | Async task panics silently killed by Tokio — no log, green dashboards | Critical | All async code | Install global panic hook → EventBus before Tokio default |
| D703-6 | `catch_unwind` foreign exception behavior unspecified (abort vs opaque Err) | High | NT-SHIELD FFI | Prefer `Result`-based FFI; wrap all `extern "C"` with `catch_unwind` + explicit error code |
| D703-7 | Three env vars control panic vs error backtrace — confusing in prod | Low | DevOps | Document canonical env: `RUST_BACKTRACE=1 RUST_LIB_BACKTRACE=1` for full coverage |
| D703-8 | `error_chain` crate uses deprecated `cause()` — dead library | Info | Dep audit | Ensure no workspace deps reference `error-chain` |
| D703-9 | `thiserror` 2.x `no_std` requires explicit `default-features = false` | Low | NT-PHYSICAL | Add feature flag gating in Cargo.toml |
| D703-10 | `chainerror::Error<T>` missing `Send+Sync` bound — async compilation failure | Medium | NT-WORLD async crawl | Don't use `chainerror` in async contexts; prefer `color-eyre` |

---

## 5. Sources Cited

1. https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view — thiserror+anyhow patterns
2. https://docs.rs/thiserror/latest/thiserror/derive.Error.html — thiserror 2.x derive docs
3. https://crates.io/crates/thiserror — thiserror 2.0.20 release history
4. https://docs.rs/anyhow/latest/anyhow/ — anyhow API reference
5. https://github.com/dtolnay/anyhow/ — anyhow repo
6. https://docs.rs/eyre/latest/eyre/ — eyre API reference
7. https://docs.rs/color-eyre/latest/color_eyre/ — color-eyre with Section trait
8. https://docs.rs/chainerror/latest/chainerror/ — chainerror crate
9. https://docs.rs/error-chain/latest/error_chain/ — error_chain (legacy)
10. https://docs.rs/error-context/latest/error_context/ — error_context crate
11. https://doc.rust-lang.org/stable/std/panic/fn.catch_unwind.html — catch_unwind docs
12. https://krun.pro/rust-production-error-handling/ — Rust production error handling guide (2026-06-27)
13. https://seaxiang.com/blog/VGuvP5QJ — FFI panic handling (2026-08-30)
14. https://rustify.rs/articles/rust-error-handling-result-option — Rust error handling 2026 guide
15. https://medium.com/@puneetpm/beyond-unwrap-your-2026-guide-to-fearless-error-handling-in-rust-984e07495c0a — 2026 unwrap guide
