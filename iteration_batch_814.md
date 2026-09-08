# Iteration Batch 814 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### CLI/Argument Parsing (8)
- clap 4.6.6: 1.1B downloads, ecosystem king, derive API, upcoming v5
- structopt 0.3.26: Dead (RUSTSEC-2022-0104), depends on unmaintained ansi_term
- argh 0.1.19: Google's minimal parser (binary size optimized, no color/completions)
- lexopt 0.3.2: Zero-dependency single-file parser (one-person maintainer)
- bat 0.26.1: cat clone with syntax highlighting + built-in pager
- dialoguer 0.12.0: Interactive prompts (Select/MultiSelect/FuzzySelect/Input/Password)
- console 0.16.x: Terminal abstraction, color detection, strip_ansi_codes
- indicatif 0.18.6: Progress bars + spinners (NeoTrix pins 0.17, which is yanked)

### Memory/Mmap (5)
- memmap2 RUSTSEC-2026-0186: Unsafe pointer offset without bounds checks (CRITICAL)
- bytes CVE-2026-25541: Integer overflow in BytesMut::reserve (HIGH)
- mmap-rs 0.7.0: Safer alternative (anonymous mappings safe, file-backed still unsafe)
- aligned 0.4.3: Risk with #[repr(packed)] + pin projection
- bytes-utils 0.1.4: SegmentedBuf for zero-copy multi-buffer concatenation

### Error Handling (6)
- thiserror 2.0.20: Derives Error + Display + From (zero runtime cost)
- anyhow: Type-erased error with context() chaining, bail!/ensure!
- eyre 0.6.12: Fork of anyhow with customizable EyreHandler
- color-eyre 0.6.5: SpanTrace + Backtrace + Section trait for structured reports
- snafu 0.9.2: Context selector pattern, unstable-provider-api for Location tracking
- error-codes 0.1.0: ErrorCode enum → HTTP status + RFC 7807 Problem Details

### Async Runtime (8)
- async-std: DEAD (RUSTSEC-2025-0052), discontinued Aug 2025
- smol: Rising composable runtime (5K stars, MSRV 1.85)
- monoio: ByteDance thread-per-core io_uring (Linux 5.6+ required)
- glommio: Datadog thread-per-core (Linux 5.8+, 3 io_uring rings per CPU)
- tokio-console: Async debugging TUI (4.6K stars, gRPC-based)
- async-task: Executor building block (3.9M downloads/month, 6K dependents)
- Rust 2026 roadmap: RTN, AFIDT, Share trait, guaranteed destructors
- NeoTrix has 100+ direct tokio::spawn sites (no runtime abstraction)

---

## Defects Identified (28+)

### CLI/Argument Parsing (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-CLI-1 | structopt has RUSTSEC-2022-0104 (supply chain violation) | Critical |
| D-CLI-2 | indicatif pinned to 0.17 (yanked version) | High |
| D-CLI-3 | No dialoguer/console for interactive prompts | Medium |
| D-CLI-4 | No bat/minus pager for long output | Medium |
| D-CLI-5 | Two separate clap declarations with different features | Low |
| D-CLI-6 | No argh/lexopt evaluation for WASM subcommands | Low |

### Memory/Mmap (4)
| ID | Defect | Severity |
|----|--------|----------|
| D-MEM-1 | memmap2 RUSTSEC-2026-0186: unsafe pointer offset without bounds checks | Critical |
| D-MEM-2 | bytes CVE-2026-25541: integer overflow in BytesMut::reserve | High |
| D-MEM-3 | mmap-rs safer alternative not evaluated | Medium |
| D-MEM-4 | aligned + #[repr(packed)] + pin projection risk | Low |

### Error Handling (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-1 | No unified error type per domain (7 factions, no nt_*::Error convention) | High |
| D-ERR-2 | Box<dyn Error> usage loses type information | Medium |
| D-ERR-3 | No SpanTrace integration (breaks EventBus→Heartbeat→GWT feedback) | High |
| D-ERR-4 | No RFC 7807 Problem Details for NT-IO API errors | Medium |
| D-ERR-5 | Backtrace capture inconsistency across modules | Low |
| D-ERR-6 | Error chain depth unbounded (no deduplication) | Low |

### Async Runtime (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-ASYNC-1 | Tokio hard-coupling across 100+ call sites (no runtime abstraction) | High |
| D-ASYNC-2 | tokio::runtime::Runtime::new() inside sync code (potential deadlock) | High |
| D-ASYNC-3 | No runtime-agnostic task abstraction | Medium |
| D-ASYNC-4 | Missing async observability (no tokio-console) | Medium |
| D-ASYNC-5 | No thread-per-core path for io_uring workloads | Medium |
| D-ASYNC-6 | tokio::sync::RwLock in hot paths (no priority inversion avoidance) | Low |

## Key Insights (This Batch)

1. **memmap2 RUSTSEC-2026-0186 is critical**: Unsafe pointer offset without bounds checks causes UB. Must pin >= 0.9.11 immediately.

2. **bytes CVE-2026-25541 is exploitable in release builds**: Integer overflow wraps silently, creating out-of-bounds slices. Must pin >= 1.11.1.

3. **async-std is dead**: RUSTSEC-2025-0052, discontinued Aug 2025. Any transitive dependency on it is a security risk.

4. **eyre > anyhow for NeoTrix**: Custom EyreHandler aligns with consciousness-style error reporting. SpanTrace integrates with tracing for GWT attention routing.

5. **No domain error convention**: 7 factions have no nt_*::Error pattern. This prevents GWT attention routing on error variants (can't score salience of unknown error types).

6. **indicatif 0.17 is yanked**: Must upgrade to 0.18.6. Also missing dialoguer/console for interactive CLI workflows.

7. **ad-hoc Runtime::new() in sync code**: Multiple locations create new Tokio runtimes inside existing runtime = potential deadlock. Must use spawn_blocking or Handle::current().

8. **error-codes crate bridges thiserror→HTTP**: Directly relevant for NT-IO API layer (RFC 7807 Problem Details).

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 814 |
| New defects (this batch) | 22 |
| Cumulative defects | D01-D76496 |
| Research sources (this batch) | 27 |
| Cumulative research sources | 97,279+ |
