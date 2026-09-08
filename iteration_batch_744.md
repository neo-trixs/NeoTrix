# Iteration 744 — Error Handling Ecosystem Audit

## Sources

| # | Source | Date | Topic |
|---|--------|------|-------|
| 1 | pistack.xyz — anyhow vs thiserror vs eyre guide | 2026-06-22 | Library comparison, patterns |
| 2 | rajpoot.dev — Rust Error Handling 2026 | 2026-04-30 | anyhow/thiserror patterns, panic policy |
| 3 | rustify.rs — anyhow glossary | 2026-08-13 | anyhow API, context, bail, downcast |
| 4 | youngju.dev — Rust Ecosystem 2026 Deep-Dive | 2026-05-16 | Full stack: anyhow/thiserror/eyre/snafu/miette |
| 5 | dtolnay/thiserror (GitHub) | current | v2.0.20, `provide()` method, `#[backtrace]` |
| 6 | docs.rs/thiserror 2.0.20 | current | `provide()` forwarding, `#[backtrace]` on source |
| 7 | dtolnay/anyhow (GitHub) | current | Context trait, `.context()` / `.with_context()` |
| 8 | eyre-rs/eyre (GitHub) | current | `EyreHandler`, `stable-eyre`, `color-eyre` |
| 9 | zkat/miette (GitHub) | current | `Diagnostic` derive, source spans, `#[label]` |
| 10 | shepmaster/snafu (GitHub) | current | `Snafu` derive, context selectors |
| 11 | dev.to — Node.js error handling 2026 | 2026-06 | Graceful degradation, circuit breaker |
| 12 | convertatlas.org — graceful error handling | 2026 | Fail fast vs fail safe, error boundaries |
| 13 | namastedev.com — resilient web apps | 2026-03 | User-friendly error patterns |
| 14 | market.lobehub — error-recovery skill | 2026 | Multi-agent error recovery, timeouts |
| 15 | rustfaq.org — miette guide | 2026-04 | `Diagnostic` derive, source snippets |
| 16 | generalistprogrammer.com — miette guide | 2025-11 | Best practices, MSRV |

---

## NEW FINDINGS

### F1. thiserror v2 `provide()` — Error Trait's `provide()` Method (NEW IN 2.0)

**Source**: [5][6] — thiserror 2.0.20

thiserror v2 implements `Error::provide()` to automatically provide `Backtrace` from fields typed `std::backtrace::Backtrace`. Key new capability:

```rust
#[derive(Error, Debug)]
pub enum MyError {
    Io {
        #[from] source: io::Error,
        #[backtrace]  // forwards provide() to source's provide()
    },
}
```

When a field is both `#[source]`/`#[from]` AND `#[backtrace]`, `provide()` is forwarded to the source's `provide()`, sharing the backtrace across error layers. This eliminates the "duplicate backtrace" problem from v1.

**Defect D1**: `provide()` still requires nightly (rustc 1.73+). In 2026 with Rust 1.85+, this should be stabilized. No workaround exists in thiserror — users must use `snafu` with `backtrace-rs` for stable backtrace support.

### F2. eyre `EyreHandler` — Customizable Error Report Format (UNDEREXPLOITED)

**Source**: [8] — eyre-rs/eyre

eyre's `EyreHandler` trait allows swapping the entire error report formatter. Companion crates:
- `stable-eyre`: backtrace-rs for stable Rust
- `color-eyre`: syntax-highlighted source snippets

The `EyreHandler` is the mechanism that enables structured error reports beyond plain text.

**Defect D2**: NeoTrix does not use eyre's `EyreHandler` for custom error formatting. NT-IO (CLI) uses `anyhow` for error propagation but loses the structured diagnostic output that `eyre`/`color-eyre` provides. User-facing CLI errors in NeoTrix are plain anyhow messages without source snippets or suggestions.

### F3. miette `Diagnostic` Derive — Source Span Labels (NOT IN NEOPIX)

**Source**: [9][15][16] — zkat/miette

miette provides `#[derive(Diagnostic)]` which adds:
- Error codes (`#[diagnostic(code(...))]`)
- Source span annotations (`#[label(collection)]`)
- Help URLs (`#[diagnostic(help(...))]`)
- Syntax highlighting via `syntect`
- `url(docsrs)` for auto-linking to docs.rs

miette produces compiler-style diagnostics with pointing arrows:
```
  error[my_app::parse_error]: invalid header
    --> config.toml:3:1
     |
  3   | [broken]
     | ^^^^^^^^ expected `key = value`
     = help: https://docs.rs/my_crate
```

**Defect D3**: NeoTrix's NT-IO CLI layer has no structured diagnostic output. Error messages from `anyhow` are flat strings. For a developer toolkit, users need precise source-location diagnostics (e.g., "parse error at config.toml:3:1") not just "failed to parse config".

### F4. snafu Context Selectors — Compile-Time Error Construction (MISSING IN NEOPIX)

**Source**: [10] — shepmaster/snafu

snafu provides context selectors — named types that carry both the error context AND the additional data. This prevents the "context string proliferation" problem where anyhow's `.context("string")` becomes unsearchable:

```rust
#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Unable to read config from {}", path.display()))]
    ReadConfiguration { source: io::Error, path: PathBuf },
}

// Selector is a named type: ReadConfigurationSnafu
let config = fs::read_to_string(path).context(ReadConfigurationSnafu { path })?;
```

**Defect D4**: NeoTrix modules use `anyhow::context("string literal")` extensively. Error messages are unstructured strings — impossible to match on programmatically, grep in logs, or build error taxonomy from. snafu's named context selectors would enable structured error classification.

### F5. Five-Way Error Handling Taxonomy in 2026 Rust (NEW CONSOLIDATION)

**Source**: [4] — youngju.dev Rust Ecosystem 2026

The 2026 Rust ecosystem has consolidated into exactly 5 error handling approaches:

| Approach | Role | Library |
|----------|------|---------|
| **Library error types** | Typed, matchable errors | `thiserror` (derive), `snafu` (context selectors) |
| **Application aggregation** | Erased, contextual | `anyhow` (opaque), `eyre` (anyhow + custom handler) |
| **User-facing diagnostics** | Pretty, source-annotated | `miette` (Diagnostic derive, spans, labels) |

The three-way split (thiserror:library, anyhow:app, miette:CLI) is now the canonical Rust error handling pattern.

**Defect D5**: NeoTrix uses only 2 of 3 pillars: `thiserror` (for NT-* error types) and `anyhow` (for application propagation). Missing: `miette` for NT-IO CLI diagnostics. The three pillars are incomplete without the diagnostic layer.

### F6. Graceful Degradation Patterns — Circuit Breaker + Fallback Chain (NOT IN RUST CORE)

**Source**: [11][12][13][14]

2026 production error handling patterns include:
1. **Circuit breaker**: Stop hammering failing external services after N failures
2. **Fallback chain**: Try multiple sources in order (Redis → DB → cache → defaults)
3. **Queue-for-later**: Email/notification failures are queued, not dropped
4. **Feature flag kill-switch**: Disable non-critical features on failure
5. **Stale cache fallback**: Serve stale data when fresh fetch fails

**Defect D6**: NeoTrix NT-ACT (action domain) lacks circuit breaker and fallback chain patterns. Tool calls to external services (LLM providers, MCP servers) fail hard with no retry/fallback/degradation strategy. The `ResourceBudgetManager` tracks cost but not failure recovery.

### F7. Error Context Chaining — `.context()` vs `.with_context()` Allocation Cost

**Source**: [1][3][7]

The lazy evaluation pattern `.with_context(|| format!(...))` avoids allocation when there's no error:
```rust
// Allocates ALWAYS (eager)
.context(format!("failed for user {id}"))

// Allocates ONLY on error (lazy)
.with_context(|| format!("failed for user {id}"))
```

**Defect D7**: NeoTrix modules may be using `.context(format!(...))` instead of `.with_context(|| format!(...))` in hot paths. Each unnecessary allocation adds GC pressure in async contexts. This is a micro-optimization but compounds across thousands of error propagation points.

---

## DEFECTS SUMMARY

| ID | Severity | Defect | Fix |
|----|----------|--------|-----|
| D1 | Medium | thiserror `provide()` requires nightly for backtrace | Use `snafu` + `backtrace-rs` for stable backtrace, or gate `#[backtrace]` behind feature flag |
| D2 | High | NT-IO CLI uses plain anyhow — no structured error output | Adopt `eyre` + `color-eyre` or `miette` for CLI-facing error formatting |
| D3 | High | No source-span diagnostics in CLI output | Add `miette::Diagnostic` derive to NT-IO error types with `#[label]` spans |
| D4 | Medium | Unstructured `.context("string")` errors — unmatchable | Adopt `snafu` context selectors or `thiserror` error codes for structured error taxonomy |
| D5 | High | Missing miette pillar — only 2/3 of canonical error pattern | Integrate `miette` as the NT-IO diagnostic layer |
| D6 | High | No circuit breaker / fallback chain in NT-ACT tool calls | Implement circuit breaker pattern for external tool invocations |
| D7 | Low | `.context(format!())` vs `.with_context(|| format!())` allocation waste | Audit hot paths for eager context allocation, switch to lazy `.with_context()` |

---

## KEY INSIGHT

The 2026 Rust error handling ecosystem has settled into a **three-pillar architecture**:

1. **`thiserror`** (library errors) — typed, matchable, derive macros
2. **`anyhow`** (application errors) — opaque, contextual, ergonomic `?`
3. **`miette`** (diagnostics) — source spans, labels, error codes, pretty printing

NeoTrix currently implements pillars 1+2 but is **missing pillar 3**. For a developer-facing CLI toolkit, miette-style diagnostics are not optional — they're the primary user experience for error reporting. This is the highest-priority defect.
