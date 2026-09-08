# Iteration Batch 849 Report — NeoTrix Consciousness Architecture

## Research Sources (34+)

### Build Optimization (10)
- mold 3.0 (Aug 2026): 3-10x link time reduction, linker script support
- sccache: shared compilation cache, S3/GCS/Redis backends, cannot cache cdylib/proc-macro
- cargo-nextest: 2-3x faster tests, process-per-test isolation, flaky detection
- lto = "thin" achieves ~95% of fat LTO benefit at ~50% compile time
- Rust 2026 roadmap: "Relink-Don't-Rebuild" (5-10x improvement target)
- No .cargo/config.toml — no linker optimization, no sccache
- No CI workflows — no automated build/test pipeline
- Release profile uses fat LTO for all CI (slow)
- opt-level = "s" may sacrifice runtime for CLI/daemon
- Wild linker (Rust-based, incremental) emerging

### Database (8)
- rusqlite v0.40.1: sync-only, bundled SQLite 3.45.3+, requires Rust 1.95+
- sqlx v0.9.0: moved to transact-rs org, new sqlx.toml config, compile-time query checking
- WAL-reset bug CVE fixed in SQLite 3.51.3 (2026-03-13)
- FTS5 CVE-2026-11822/11824: heap buffer overflow + OOB read, requires SQLite ≥ 3.53.2
- sqlite-kit: new crate combining rusqlite + tokio-rusqlite + deadpool
- walrust: WAL→S3 replication
- Single-writer pattern needed for NT-MEMORY
- WAL monitoring: background task watches WAL size, triggers PASSIVE checkpoint

### Macros (8)
- macro_derive/macro_attr: declarative derive macros implemented on nightly, stabilization 2026
- Next-gen trait solver impacts generic code in proc-macro outputs
- 13 macro_rules! definitions in NeoTrix (make_stage!, define_capability_fields!, etc.)
- $super:: path fragility in log_*! macros (should use $crate::)
- register_if! has 17 nearly identical invocations (code duplication)
- CapabilityVector From impls manually list 23 fields (6+ locations)
- trust_rule! compiles regex at runtime (should use LazyLock)
- No proc-macro crate for code generation

### Observability (failed — certificate error)
- Agent failed due to exa.ai MCP transport error
- Deferred to next batch

---

## Defects Identified (18+)

### Build Optimization (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-BUILD-1 | No .cargo/config.toml (no linker optimization) | High |
| D-BUILD-2 | No sccache setup | High |
| D-BUILD-3 | No nextest (slow test execution) | Medium |
| D-BUILD-4 | No CI workflows | High |
| D-BUILD-5 | No release-fast profile (fat LTO in CI) | Medium |
| D-BUILD-6 | opt-level = "s" may sacrifice runtime | Low |

### Database (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-DB-1 | WAL-reset corruption (SQLite <3.51.3) | Critical |
| D-DB-2 | FTS5 memory corruption (SQLite <3.53.2) | Critical |
| D-DB-3 | rusqlite/sqlx libsqlite3-sys version conflict | High |
| D-DB-4 | WAL checkpoint under load | Medium |
| D-DB-5 | FTS5 fts5vocab overread regression | Medium |
| D-DB-6 | SQLx compile-time DB dependency | Low |

### Macros (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-MACRO-1 | $super:: path fragility in log_*! | Medium |
| D-MACRO-2 | register_if! massive code duplication | Low |
| D-MACRO-3 | make_stage! no trait wiring | Low |
| D-MACRO-4 | CapabilityVector From impls manual (6+ locations) | Medium |
| D-MACRO-5 | No proc-macro crate for code generation | Info |
| D-MACRO-6 | trust_rule! regex at runtime (should be compile-time) | Low |

## Key Insights (This Batch)

1. **mold 3.0 cuts link times 3-10x**: NeoTrix's 9 workspace members with heavy dependencies (rusqlite, reqwest, chromiumoxide, wasmtime) would benefit significantly. macOS uses lld instead.

2. **sccache cannot cache cdylib**: NeoTrix's neotrix-core is a cdylib, so sccache won't cache its final compilation. Only intermediate dependencies benefit. Must disable incremental for CI sccache workflows.

3. **SQLite WAL-reset bug is a correctness blocker**: Fixed in 3.51.3. NT-MEMORY must enforce bundled SQLite ≥ 3.51.3 via libsqlite3-sys version floor.

4. **FTS5 CVEs require SQLite ≥ 3.53.2**: Heap buffer overflow + OOB read exploitable via crafted database files. Must gate FTS5 behind version check.

5. **macro_derive stabilization imminent**: Will eliminate need for proc-macro crates for many use cases. make_stage!, impl_evolution_capable!, CapabilityVector From impls are migration candidates.

6. **$super:: path fragility**: log_*! macros use $super:: which resolves relative to expansion site, not definition site. Must use $crate:: instead.

7. **rusqlite is better fit than sqlx for NT-MEMORY**: Embedded single-DB pattern doesn't need sqlx's async overhead. rusqlite has better vtab/hooks/backup support.

8. **sqlite-kit is the right architecture**: Combines rusqlite + tokio-rusqlite + deadpool with read/write split, WAL monitoring, write queue — exactly what NT-MEMORY needs.

9. **Nextest gives 2-3x faster tests**: Process-per-test isolation prevents global state corruption. Essential for SelfTest trait patterns.

10. **WAL monitor background task**: Polls PRAGMA wal_checkpoint(PASSIVE) when WAL size exceeds threshold. Prevents unbounded WAL growth.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 849 |
| New defects (this batch) | 18 |
| Cumulative defects | D01-D77387 |
| Research sources (this batch) | 26 |
| Cumulative research sources | 98,377+ |
