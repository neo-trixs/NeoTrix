# Agent 4: Error Type Design (Batch 864)

## Sources

1. https://docs.rs/thiserror/latest/thiserror/index.html — thiserror derive macro docs
2. https://github.com/dtolnay/thiserror/ — thiserror GitHub repo
3. https://d34dl0ck.me/rust-bites-designing-error-types-in-rust-libraries/index.html — Designing Error Types in Rust Libraries (2025)
4. https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view — How to Design Error Types with thiserror and anyhow (2026)
5. https://rs4ts.dev/08-error-handling/08-best-practices/ — Rust Error Handling Best Practices (2026)
6. https://docs.rs/anyhow/latest/anyhow/ — anyhow docs
7. https://www.azdanov.dev/articles/2025/rust-error-guidelines — Rust Error Guidelines (2025)
8. https://tristonarmstrong.com/blog/custom-rust-errors — Custom Rust Errors: DomainError per module (2026)
9. https://markaicode.com/rust-error-handling-2025-guide/ — Rust Error Handling Guide 2025
10. https://andrewodendaal.com/rust-error-handling-patterns-production/ — Error Handling Patterns for Production (2026)
11. https://effective-rust.com/errors.html — Effective Rust: Prefer idiomatic Error types
12. https://home.expurple.me/posts/designing-error-types-in-rust-applications/ — Designing Error Types: enum per function (2026)
13. https://alexfedoseev.com/blog/post/thiserror-anyhow-or-how-i-handle-errors-in-rust-apps — thiserror vs anyhow experience (2026)
14. https://krun.pro/rust-anyhow-vs-thiserror/ — anyhow vs thiserror Guide (2026)

## Defects

### D-ERR-001: NeoTrixError is a god-enum — 18 string-wrapped variants collapse all failure modes into opaque strings
**File**: `neotrix-core/src/unified/core/nt_core_error.rs:11-48`
**Severity**: HIGH
**Source**: Source #8 (Custom Rust Errors: DomainError per module), Source #12 (enum per function pattern), Source #3 (Designing Error Types in Rust Libraries)

The core `NeoTrixError` enum has 18 variants (`Config`, `Io`, `Serde`, `Network`, `Mcp`, `Brain`, `Memory`, `Command`, `Path`, `Unimplemented`, `Wasm`, `Crypto`, `Keyring`, `Shield`, `Steer`, `NotFound`, `InvalidInput`, `InvalidState`, `NotImplemented`, `OperationFailed`, `SafetyViolation`) — all carrying `String` payloads except `Command` and `Path`. This is a textbook anti-pattern: a single catch-all error enum with string-wrapped variants provides no typed structure for callers to match on. Best practice (Sources #8, #12) recommends one enum per domain/module, not one enum for the entire system. The `Brain(String)` variant is particularly problematic — it's the default sink for `From<String>` and `From<&str>` conversions (lines 94-103), meaning any string error gets silently absorbed as a "Brain" error with no distinction between an LLM API failure, a parse error, or a logic bug.

### D-ERR-002: NeoTrixError::Io loses the error source chain — `From<io::Error>` discards `.source()`
**File**: `neotrix-core/src/unified/core/nt_core_error.rs:88-92`
**Severity**: MEDIUM
**Source**: Source #4 (thiserror and anyhow design), Source #6 (anyhow docs — context chaining), Source #11 (Effective Rust — `source()` is important)

`impl From<std::io::Error> for NeoTrixError` converts via `err.to_string()`, which discards the original `io::Error` and its `.source()` chain. Best practice (Sources #4, #6, #11) is to preserve the source error using `#[source]` or `#[from]` so the error chain is walkable. When this error propagates to `anyhow` or `eyre`, the underlying OS error code, inner `ErrorKind`, and any chained context are permanently lost. This makes debugging I/O failures in production significantly harder.

### D-ERR-003: Duplicated error hierarchies — NeoTrixError (core), L1Error (L1 layer), FFI NeoTrixError (FFI) are three parallel, unlinked error enums with overlapping variants
**Files**: `neotrix-core/src/unified/core/nt_core_error.rs:11-48`, `neotrix-core/src/unified/layers/action/nt_io/nt_l1_error.rs:7-25`, `neotrix-core/src/neotrix/ffi/types.rs:11-20`
**Severity**: HIGH
**Source**: Source #10 (Error Handling Patterns for Production — layer error conversion), Source #8 (Custom Rust Errors — establish DomainError per domain)

Three separate `NeoTrixError`-like enums exist:
- Core `NeoTrixError` (22 variants, Chinese messages)
- `L1Error` (11 variants, English messages, nearly identical structure)
- FFI `NeoTrixError` (8 variants, unit-struct style, completely different shape)

None of them implement `From` for each other. The core and L1 variants have overlapping semantics (`Config`, `Io`, `Serde`, `Network`, `Brain`) but are distinct types with no conversion path. Best practice (Source #10) requires each layer to define its own error enum and implement `From` for the layer below, forming a clean conversion chain. Instead, NeoTrix has three parallel hierarchies that cannot interoperate, forcing manual `.map_err()` at every boundary.

### D-ERR-004: `From<String>` for NeoTrixError maps to Brain(String) — silent type erasure loses diagnostic information
**File**: `neotrix-core/src/unified/core/nt_core_error.rs:94-103`
**Severity**: MEDIUM
**Source**: Source #3 (Designing Error Types — avoid exposing raw strings), Source #14 (anyhow vs thiserror — type erasure via downcast)

Both `From<String>` and `From<&str>` silently convert any string to `NeoTrixError::Brain(msg)`. This is a trap: callers using `?` on `Result<T, String>` get no signal that the error classification is wrong. A network timeout string becomes "Brain error", a config parse failure becomes "Brain error", a serde error becomes "Brain error" — all indistinguishable. Best practice (Source #3) is to either use `anyhow::Error` for application-level string errors or to map errors explicitly at the point of creation. The `from_string_result` helper (line 108-110) compounds this by making the Brain-sink the canonical conversion path.

### D-ERR-005: No `#[non_exhaustive]` on any error enum — adding variants is a semver-breaking change
**Files**: All error enums across the codebase
**Severity**: HIGH
**Source**: Source #5 (Rust Error Handling Best Practices — keep public error enums non_exhaustive), Source #12 (Designing Error Types — application vs library non_exhaustive), Source #13 (thiserror vs anyhow — non_exhaustive)

Not a single error enum in the codebase uses `#[non_exhaustive]`:
- `NeoTrixError` (core)
- `L1Error`
- `CapabilityError`
- `ConsistencyError`
- `LlmError`
- `CoTError`
- `TaskDispatchError`
- `MediaSourceError` (uses thiserror but no `#[non_exhaustive]`)
- `MailError`
- `FileAbilityError`

Best practice (Source #5) mandates `#[non_exhaustive]` on any error type that may evolve. Since NeoTrix is a rapidly evolving system with C0-C5 constellation maturity, adding variants to these enums will break all downstream `match` arms without a `_` catch-all. For a monorepo this is recoverable, but for the FFI boundary and any external consumers, it's a semver violation.

### D-ERR-006: Inconsistent error derivation — some types use `thiserror`, others hand-implement Display+Error with no unified pattern
**Files**: Multiple — see comparison below
**Severity**: MEDIUM
**Source**: Source #1 (thiserror docs — derive macro), Source #14 (krun.pro — consistency matters more than specific choice)

The codebase uses three different error derivation strategies:
1. **`thiserror::Error` derive** (correct): `FileAbilityError`, `MediaSourceError`, `MailError`, `RegistryError`, `SigningError`, `BackendRouterError`
2. **Manual `Display` + `impl Error`** (verbose, error-prone): `NeoTrixError`, `L1Error`, `CapabilityError`, `LlmError`, `CoTError`, `TaskDispatchError`, `ElementError`, `ConsistencyError`
3. **Manual `Display` + no `impl Error`** (broken): `SandboxError`, `PdfEditError`, `CryptoError`, `AgentError`, `ModelError`

Source #14 emphasizes: "the most important thing is consistency. Agree on a system with your team, write it down, and stick to it." The current codebase has no consistent error strategy — new modules randomly choose between thiserror and manual impls.

### D-ERR-007: MailError has `#[from] anyhow::Error` — library error type depends on application-level error crate
**File**: `neotrix-core/src/unified/layers/action/nt_io/nt_io_mail/error.rs:53-54`
**Severity**: MEDIUM
**Source**: Source #3 (Designing Error Types — avoid leaking internal error types), Source #5 (Best Practices — don't put anyhow in library return types)

`MailError` (a domain-level error type) has `#[error(transparent)] Other(#[from] anyhow::Error)`. Best practice (Sources #3, #5) explicitly warns: "Do not put `anyhow::Error` in a public library return type. It erases the variants, so your library's users can no longer `match` on what went wrong." While `MailError` is internal to NeoTrix, this pattern leaks `anyhow` as a dependency into any module that imports `MailError`. The `#[from] anyhow::Error` also creates a one-way conversion trap: once an error is wrapped in `MailError::Other`, the original typed error is irretrievable without downcasting.

### D-ERR-008: No error context chaining — missing `.context()` / `.with_context()` at trust boundaries
**Files**: Throughout the codebase — grep shows only 6 matches for `anyhow` imports
**Severity**: MEDIUM
**Source**: Source #6 (anyhow docs — context chaining), Source #10 (Production Patterns — every `?` at a boundary gets `.context()`)

The codebase has almost no usage of `anyhow` or `eyre` for error context chaining. Best practice (Source #10): "every `?` at a boundary — network calls, file I/O, parsing — gets a `.context()` or `.with_context()`." Without context, error messages like `"IO error: connection refused"` lack the "what were you doing?" layer that makes production debugging possible. The few `anyhow` imports are in `MailError`'s `Other` variant and two bank knowledge files — not in actual error propagation paths.

### D-ERR-009: CapabilityError and ConsistencyError lack `std::error::Error` trait implementation (CapabilityError has it, ConsistencyError does not)
**File**: `neotrix-core/src/unified/layers/action/traits.rs:164-187`, `neotrix-core/src/unified/layers/action/nt_io/consistency_adapter.rs:137-151`
**Severity**: LOW
**Source**: Source #7 (Rust Error Guidelines — implement Error trait), Source #11 (Effective Rust — implement Error for your error types)

`CapabilityError` (line 187) implements `std::error::Error` but does so manually (empty impl). `ConsistencyError` (lines 137-151) does NOT implement `std::error::Error` at all — it only derives `Debug, Clone, Serialize, Deserialize` and has no `Display` impl. This means `ConsistencyError` cannot be used with `?` in functions returning `Result<T, E> where E: std::error::Error`, cannot be converted to `anyhow::Error`, and cannot participate in any error chain. Best practice (Source #7, #11): every error type should implement `std::error::Error`.

### D-ERR-010: Mixed error message languages (Chinese + English) in same error type
**File**: `neotrix-core/src/unified/core/nt_core_error.rs:50-83`
**Severity**: LOW
**Source**: Source #5 (Best Practices — write error messages like log lines), Source #4 (Error Types Design — lowercase, no period, no "error:")

`NeoTrixError::Display` mixes Chinese and English messages: `"配置错误: {}"` (line 53), `"IO 错误: {}"` (line 54), `"网络错误: {}"` (line 56), `"steer 重定向建议: {}"` (line 75). Meanwhile `L1Error` uses English: `"Config error: {}"` (line 30), `"IO error: {}"` (line 31). The `CapabilityError` uses English: `"Not available: {}"` (line 177). Best practice (Source #5) says error messages should be lowercase, no period, no "error:" prefix — but language consistency is equally important for log parsing, grep-ability, and i18n.

### D-ERR-011: FFI NeoTrixError is unit-only variants with no payload — cannot convey error details across FFI boundary
**File**: `neotrix-core/src/neotrix/ffi/types.rs:10-20`
**Severity**: MEDIUM
**Source**: Source #1 (thiserror docs — errors with data vs unit variants), Source #5 (Best Practices — include relevant data in error variants)

The FFI `NeoTrixError` uses `Copy` unit variants (`InitFailed`, `InvalidInput`, `OperationFailed`, `NotInitialized`, `SerializationError`, `NetworkError`, `PermissionDenied`, `NotFound`) with no associated data. When a native iOS/Android app receives `OperationFailed`, it has no idea what operation failed, why, or what to tell the user. Best practice (Source #5): "Include relevant data in error variants — field names, paths, values." The FFI boundary is precisely where error detail matters most, yet this is where it's most aggressively stripped.

### D-ERR-012: LlmError has `From<String>` that maps to `Unknown(String)` — loses error classification
**File**: `neotrix-core/src/unified/core/nt_core_llm.rs:368-369`
**Severity**: LOW
**Source**: Source #12 (Designing Error Types — enum per function), Source #3 (Designing Error Types in Libraries)

`LlmError` implements `From<String>` mapping to `LlmError::Unknown(String)`. This means any string error from an LLM provider gets classified as "Unknown" even if it's actually a rate limit, auth failure, or network issue. The LLM error type itself is well-structured (6 variants: Network, Authentication, RateLimit, InvalidRequest, Server, Unknown) but the `From<String>` bypasses this classification entirely.

## Key Insights

1. **The codebase has no unified error strategy.** Some modules use `thiserror`, some hand-implement, some don't implement `Error` at all. This inconsistency is the single largest source of error-handling debt.

2. **The "Brain(String)" sink is a silent diagnostic killer.** When `From<String>` maps everything to `Brain(String)`, the error type becomes a black box. This is the Rust equivalent of JavaScript's `catch(e) { throw new Error(String(e)) }` — it destroys the information chain.

3. **No error context chaining means production debugging relies on log messages, not error types.** The codebase should adopt `.with_context()` at every trust boundary (network, I/O, parsing) to build error chains that tell the full story.

4. **The triple error hierarchy (core/L1/FFI) needs unification.** Either use `From` conversions between layers, or collapse to a single error type with domain-specific sub-errors. The current state forces manual `.map_err()` everywhere.

5. **`thiserror` adoption is incomplete but correct where used.** The modules that use `thiserror` (`FileAbilityError`, `MediaSourceError`, `MailError`) are significantly cleaner than the hand-rolled ones. The path forward is to migrate all error types to `thiserror` with `#[non_exhaustive]`.

6. **FFI errors need richer payloads.** The current unit-variant FFI error is unusable for cross-platform error reporting. Consider `#[uniffi(record)]` or at minimum a `detail: String` field.

7. **`eyre` is absent from the codebase.** For application-level error handling (CLI, daemon), `eyre::Report` with `color-eyre` would provide better panic handling and error reporting than the current manual approach.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources consulted | 14 |
| Error types analyzed | 20+ |
| Files with `.unwrap()` in non-test code | 100+ (grep result) |
