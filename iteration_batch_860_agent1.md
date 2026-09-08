# Agent 1: Error Propagation Patterns (Batch 860)

## Sources

1. **Pi Stack — Rust Error Handling Libraries: anyhow vs thiserror vs eyre (2026-06-22)** — Comprehensive comparison of anyhow, thiserror, eyre. Key: thiserror for library typed errors, anyhow for app propagation, eyre+color-eyre for CLI diagnostics. `#[source]` preserves error chains; `.context()`/`.wrap_err()` adds context without losing chain. Type-erased errors (anyhow/eyre) add one heap allocation per error.
2. **thiserror 2.0 Docs.rs** — `#[from]` implies `#[source]`; `#[error(transparent)]` forwards Display/source through. `Backtrace` field auto-detected by name. Private `ErrorRepr` pattern for internal error enums.
3. **eyre-rs/eyre GitHub** — eyre is anyhow fork with customizable `EyreHandler`. `WrapErr::wrap_err()` attaches context. Warning: string-constructed errors encourage brittle string matching.
4. **color-eyre GitHub** — Captures `SpanTrace` + `Backtrace` for rich error reports. `Section` trait for multi-source error aggregation. SpanTrace cheaper than Backtrace; `RUST_SPANTRACE` env controls capture. Debug builds: `backtrace` crate needs `[profile.dev.package.backtrace] opt-level = 3`.
5. **RustWorks/eyre-error-handling** — Comparison to thiserror: "Use thiserror if you need error type that can be handled via match or reported." eyre docs explicitly warn against returning `anyhow::Error` from library public APIs.
6. **Rust Error Crate Comparison (wyattgill9/knowledge-base)** — Four ecosystem layers: typed definition (thiserror/snafu), type-erased reporting (anyhow/eyre), diagnostic rendering (miette/ariadne), observability context (tracing-error/SpanTrace). Structured context (snafu context selectors) > string context for large systems.
7. **Rust Error Handling Guide 2025 (markaicode)** — `thiserror` for libraries, `anyhow` for applications. `#[from]` solves diverse error propagation via `?`.
8. **Rust Crates Error Handling: thiserror vs anyhow (normansoven)** — Practical guide: thiserror for typed enum errors, anyhow for single dynamic error type. Both near-universal in real Rust.
9. **How to Design Error Types with thiserror and anyhow (oneuptime)** — Real-world patterns combining thiserror (library) + anyhow (app boundary).
10. **Rust Error Handling Compared: anyhow vs thiserror vs snafu (dev.to)** — Decision matrix: error type design cost (anyhow=low, thiserror=medium, snafu=high).
11. **Error Handling with anyhow and thiserror (Pratik Dhanave, 2026-05-14)** — Module 23 of Rust from Ground Up. Both build on `Result`/`?` foundation.
12. **Mastering Rust Error Handling Best Practices (ajmani.dev, 2026-05-05)** — Propagation chain: Postgres → thiserror → anyhow context in main(). Pitfalls: don't wrap every line in new error type; only create variants when caller needs to match.
13. **Defensive coding in Rust: error handling patterns that scale (botmonster, 2026-05-16)** — Comprehensive patterns with thiserror, anyhow, miette, color-eyre.
14. **How to Use the color-eyre Crate (rustfaq.org, 2026-04-17)** — `wrap_err` builds numbered error chain. SpanTrace shows request/user ID from tracing spans. `NO_COLOR=1`/`CLICOLOR_FORCE=1` for ANSI control. Don't pair with anyhow.

## Defects

D-ERR-001: **NeoTrixError uses hand-written String-only variants without thiserror, losing all error chain information** | `neotrix-core/src/unified/core/nt_core_error.rs:11-48` | HIGH | Source: Pi Stack guide, thiserror docs. The unified error type has 20+ variants all wrapping `String`. No `#[source]` attributes, no `#[from]` auto-conversions, no backtrace support. Every `.map_err(|e| e.to_string())` across the codebase feeds into this, destroying the original error chain. The `std::error::Error` impl at line 86 is empty — `source()` always returns `None`.

D-ERR-002: **CapabilityError is a hand-written enum not using thiserror, lacks `#[source]` chain** | `neotrix-core/src/unified/layers/action/traits.rs:164-172` | HIGH | Source: thiserror docs, Pi Stack guide. All 6 variants wrap `String` with no `#[source]` or `#[from]`. The `impl std::error::Error` at line 187 is empty. This is the cross-domain error type for NT-ACT capabilities — errors from NT-IO, NT-WORLD, NT-SHIELD all lose their chain when converted via `map_err(|e| e.to_string())` at every call site (100+ occurrences found).

D-ERR-003: **L1Error duplicates NeoTrixError structure without thiserror, creating parallel error universe** | `neotrix-core/src/unified/layers/action/nt_io/nt_l1_error.rs:7-25` | MEDIUM | Source: Pi Stack guide (library/app boundary). L1Error is an almost-identical copy of NeoTrixError (Config/Io/Serde/Network/Brain variants) but for L1 action layer. Both use hand-written String wrappers. This creates a dual error hierarchy that must be manually bridged, and both lose source chains identically.

D-ERR-004: **TaskDispatchError uses String-only variants without `#[source]`, COT/reasoning errors lose root cause** | `neotrix-core/src/unified/core/nt_core_task_dispatcher.rs:1258-1274` | HIGH | Source: color-eyre docs (error chain preservation). All 7 variants (LlmError, ParseError, CotError, ReasoningError, etc.) wrap `String`. At lines 414, 420, 781, 802, 862, 973 — every `.map_err(|e| e.to_string())` discards the original typed error. The TaskDispatchError `From<...>` conversions at these lines flatten everything to strings.

D-ERR-005: **Widespread `.map_err(|e| e.to_string())` pattern across 100+ sites destroys error chains** | `neotrix-core/src/unified/core/nt_core_vector_store/index.rs:231-238`, `nt_core_self/skill_crystal.rs:350-359`, `nt_world_crawl/spider/core.rs:128-137`, `nt_core_forecast.rs:932-962`, `nt_core_second_brain.rs:283-289`, `permission_profiles.rs:193-301`, many more | HIGH | Source: thiserror docs (`#[source]` preserves chain), eyre docs (`.wrap_err()` adds context without losing chain). Every `map_err(|e| e.to_string())` converts a typed error into an opaque string, making it impossible for callers to match on the original error type or access `.source()`. This is the #1 error chain destruction pattern in the codebase.

D-ERR-006: **`From<String> for NeoTrixError` routes all string errors to Brain variant, semantically wrong** | `neotrix-core/src/unified/core/nt_core_error.rs:94-98` | MEDIUM | Source: eyre docs ("string-constructed errors encourage brittle string matching"). Any `Result<T, String>` converted via `?` becomes `NeoTrixError::Brain(msg)`, even if the error is actually an IO, Network, or Config error. Same pattern in `L1Error` (line 64) and `LlmError` (line 368). The `from_string_result()` helper at line 108 institutionalizes this semantic loss.

D-ERR-007: **thiserror 1.0 used instead of 2.0; no eyre/color-eyre in dependency tree** | `neotrix-core/Cargo.toml:30` | LOW | Source: thiserror docs (2.0 adds `#[diagnostic]` support), color-eyre docs (SpanTrace + backtrace for production debugging). Thiserror 2.0 has improved derive macro and better no_std support. color-eyre is absent entirely — no SpanTrace capture, no colored error reports in CLI. For a CLI-focused developer toolkit, this means error output is plain-text only.

D-ERR-008: **FileAbilityError::Parse(String) and Other(String) variants lose parse error chain** | `neotrix-core/src/neotrix/nt_file_ability/types.rs:26-28` | MEDIUM | Source: thiserror docs (`#[source]` on error fields). The `Parse(String)` variant at line 26 receives `.map_err(|e| e.to_string())` calls from 15+ sites (file_adapter.rs:227,281,354; merge_docx.rs:146,177,477,500; doc_parse.rs:19,29,334; structured.rs:31-123). The original error type (calamine::XlsxError, ZipArchive error, etc.) is discarded. Adding `#[source]` to a non-String field would preserve the chain.

D-ERR-009: **MailError::From<anyhow> with `#[error(transparent)]` creates opaque escape hatch** | `neotrix-core/src/unified/layers/action/nt_io/nt_io_mail/error.rs:53-54` | MEDIUM | Source: thiserror docs, Pi Stack guide. The `Other(#[from] anyhow::Error)` variant with `#[error(transparent)]` means any anyhow error can enter the Mail error type and become unmatchable. While `#[from]`+`#[error(transparent)]` is correct thiserror usage, having it as a catch-all in a domain error type defeats the purpose of typed errors — callers cannot distinguish "other" errors programmatically.

D-ERR-010: **`Box<dyn std::error::Error>` used in NT-SHIELD proxy kernel, NT-CORE capability tree CLI, and FFI boundary** | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_proxy_kernel/kernel.rs:152,390,521`, `nt_shield_proxy_kernel/listener/socks5.rs:34,64,73`, `nt_shield_proxy_kernel/listener/http.rs:35,68`, `nt_core_capability_tree/src/cli.rs:231-943` (25+ functions) | MEDIUM | Source: Pi Stack guide ("libraries should expose typed errors"). The proxy kernel uses `Result<(), Box<dyn std::error::Error>>` in 8+ async functions. The capability tree CLI uses it in 25+ methods. These erase error types at domain boundaries, making it impossible for callers to match on specific failure modes. Per best practice: libraries should use thiserror enums, applications should use anyhow/eyre at the boundary.

D-ERR-011: **No `#[source]` chain in CoTError → NeoTrixError conversion; JsonParse error chain lost** | `neotrix-core/src/unified/core/nt_core_cot_generator.rs:153-161` | MEDIUM | Source: color-eyre docs (chain preservation). The `From<CoTError> for NeoTrixError` at line 157 converts `CoTError::JsonParse(serde_json::Error)` to `NeoTrixError::Serde(s.to_string())`. The serde_json::Error's `.source()` (which would show the JSON parsing location) is discarded. Same issue in AnswerEngineError → NeoTrixError at nt_core_answer_engine.rs:331-342.

D-ERR-012: **FFI layer uses `expect("ffi rwlock poisoned")` in 15+ places instead of propagating errors** | `neotrix-core/src/neotrix/ffi/e8_reasoning.rs:46,87,93,112`, `ffi/dual_specialization.rs:61,67,81,93`, `ffi/kb_bridge.rs:32,68,89,97,112,135,163`, `ffi/gwt_attention.rs:44,85,93,98,108,115,125`, `ffi/vsa_hypercube.rs:38,72,80,109,151,160` | HIGH | Source: Pi Stack guide (error handling in production), color-eyre docs. All FFI functions use `expect()` on RwLock locks, which will panic the entire process if a lock is poisoned. In a multi-threaded consciousness architecture, a poisoned lock (from a panic in another thread) should be handled gracefully with error propagation, not process termination. This is especially critical for the FFI boundary where callers (Tauri/iOS) cannot catch Rust panics safely.

## Key Insights

1. **The core problem is NeoTrixError's architecture**: The unified error type at `nt_core_error.rs` is a hand-written enum with all-String variants and an empty `Error::source()` impl. This means every error in the entire system — regardless of origin — loses its chain when converted to NeoTrixError. This is a single-point-of-failure for error diagnostics.

2. **The `.map_err(|e| e.to_string())` anti-pattern is systemic**: Found in 100+ locations across the codebase. This is not occasional misuse — it's the de facto error propagation strategy. Each site destroys a typed error and its chain. The fix requires not just replacing these with `#[from]`/`#[source]` but redesigning the error type hierarchy.

3. **Parallel error universes (NeoTrixError, L1Error, CapabilityError) create conversion debt**: Three nearly-identical hand-written error enums exist in different layers, each requiring manual `From` impls that lose information. The L1Error at `nt_l1_error.rs` was specifically created to avoid "向上引用 core 层的 NeoTrixError (L4+)" but this creates the exact duplication problem the comment acknowledges.

4. **No color-eyre/SpanTrace = no async debugging**: The codebase uses `tokio` extensively (async functions throughout NT-SHIELD, NT-WORLD, NT-ACT). Without `color-eyre` + `tracing-error` SpanTrace, async error reports show only the top-level error with no indication of which async task or span the error occurred in. This makes debugging production async failures extremely difficult.

5. **FFI panic-on-poison is a reliability hazard**: 15+ `expect()` calls in the FFI layer mean a single thread panic can cascade into process-wide failure. For a consciousness architecture with 7+ concurrent domains, this is unacceptable — a panic in NT-MIND's SEAL pipeline could poison the lock that NT-SHIELD's proxy kernel needs, crashing the entire system.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 12 |
| Sources consulted | 14 |
| Defects by severity: HIGH | 5 |
| Defects by severity: MEDIUM | 6 |
| Defects by severity: LOW | 1 |
