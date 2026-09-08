# Iteration Batch 755 — Tokio/Async Runtime/Task Scheduling Research

**Date**: 2026-09-07
**Prior**: Batch 754 (dual logging system, no structured JSON logs, zero `#[instrument]`, OTel dead code, tracing-subscriber dead code)
**Sources**: Tokio GitHub releases, dev.to, rustify.rs, reintech.io, toolsku.com, matterai.so, lucaberton.com, tokio-rs/console, johal.in, arxiv.org

---

## Source Inventory

| # | Source | URL | Date |
|---|--------|-----|------|
| S1 | Tokio releases page | github.com/tokio-rs/tokio/releases | 2026-07-20 (v1.53.1) |
| S2 | Tokio 2.0 "dominates" article (fictional) | dev.to/myroslavmokhammadabd | 2026-03-04 |
| S3 | Rust Async Runtimes 2026 comparison | rustify.rs/articles/rust-async-runtimes | 2026 |
| S4 | Tokio vs async-std vs smol 2026 | reintech.io/blog/tokio-vs-async-std | 2026-02-09 |
| S5 | Tokio runtime deep dive | lucaberton.com/blog/rust-async-runtimes | 2026-05-04 |
| S6 | Tokio LIFO slot regression #8065 | github.com/tokio-rs/tokio/issues/8065 | 2026-04-18 |
| S7 | Tokio v1.51.1/v1.51.2 release discussion | github.com/tokio-rs/tokio/discussions/8116 | 2026-05-04 |
| S8 | tokio-console debugger | github.com/tokio-rs/console | 2026 |
| S9 | Tokio task dump docs | docs.rs/tokio/latest/tokio/runtime/dump | 2026 |
| S10 | Tokio v1.52.2 release (LIFO revert) | github.com/tokio-rs/tokio/releases | 2026-05-04 |
| S11 | Runtime comparison benchmarks | toolsku.com/en/blog/rust-async-runtime-comparison | 2026-04-30 |
| S12 | smol performance data | johal.in/perf-test-2026 | 2026-04-28 |
| S13 | async-std discontinued announcement | matterai.so/guides/rust-networking | 2026-03-02 |
| S14 | Work-stealing research paper | arxiv.org/html/2603.05766v1 | 2026-03-05 |
| S15 | Tokio v1.52.4 (before_park fix) | github.com/tokio-rs/tokio/releases | 2026-07-16 |
| S16 | Tokio blog (dial9 flight recorder) | tokio.rs/blog | 2026-03-18 |
| S17 | Tokio v1.53.0/v1.53.1 release | github.com/tokio-rs/tokio/releases | 2026-07-17 |

---

## NEW Findings (Not in Batch 754)

### F755-01: Tokio LIFO Slot Stealing Regression — Silent CPU Tax
**Defect Class**: Scheduler / Hidden Performance Regression
**Severity**: HIGH

Tokio 1.51.0 introduced "steal tasks from the LIFO slot" (PR #7431). The change removed the `should_notify` fast-path that suppressed spurious worker wakeups when pushing to an empty LIFO slot. Result:
- +8.5% aggregate CPU on high-QPS microsecond-handler workloads
- `worker_park_count` 2.6× increase
- `worker_noop_count / worker_park_count` 4× increase (1/2 of unparks find no work)
- `worker_steal_count` 18× increase

The regression was **reverted** in v1.51.2 and v1.52.2 (May 4, 2026). But the LIFO slot remains non-stealable (issue #4941 still open). `disable_lifo_slot()` does NOT recover performance — the `!lifo_enabled` branch hits the same unconditional `notify_parked_local()`.

**NeoTrix Impact**: NeoTrix's async subsystems (NT-IO, NT-ACT) may be pinned to Tokio 1.51.0-era code. If `tokio::runtime::Builder` is configured with default LIFO slot enabled and high-frequency task spawning, NeoTrix could be silently paying the CPU tax even after the revert, because the `before_park` driver skip bug (fixed in 1.52.4/1.51.4) compounds the issue.

**Action Required**: Audit NeoTrix's `Cargo.toml` for exact Tokio version pin. If pinned to 1.51.0–1.52.1, upgrade to ≥1.52.4 (or ≥1.51.4 for LTS 1.51.x line). Verify `tokio::runtime::Builder` does not call `disable_lifo_slot()` (it's unstable and doesn't help).

**Sources**: S6, S7, S10, S15

---

### F755-02: Tokio `before_park` Driver Skip Bug — Lost I/O Events
**Defect Class**: Runtime / Driver starvation
**Severity**: HIGH

Fixed in Tokio v1.52.4 and v1.51.4 (July 16, 2026): "runtime: don't skip the driver when `before_park` schedules work" (PR #8222). When `before_park` schedules new work, the runtime previously skipped the I/O driver poll, causing I/O events to be delayed or lost.

**NeoTrix Impact**: If NeoTrix runs on Tokio <1.52.4 (or <1.51.4 for LTS), any code path where `before_park` hook schedules tasks could silently starve I/O — e.g., EventBus wakeups, KB write flushes, or crawl pipeline network I/O.

**Action Required**: Upgrade to Tokio ≥1.52.4 (or ≥1.51.4). Add a CI check: `cargo tree -p tokio | grep tokio` must show version ≥1.52.4 or ≥1.51.4.

**Sources**: S15, S17

---

### F755-03: Tokio Worker Thread Misconfiguration — 358% Throughput Gap
**Defect Class**: Runtime Configuration / Silent Under-provisioning
**Severity**: MEDIUM

Benchmark data from johal.in (2026-04-28): Tokio's default `#[tokio::main]` spawns worker threads equal to CPU cores. But explicit tuning matters:
- 16 worker threads on 16 vCPUs: 142k req/s
- Default single worker: 31k req/s (**358% gap**)
- Over-provisioning (32 threads on 16 vCPUs): 12% memory overhead, 7% throughput drop

**NeoTrix Impact**: NeoTrix's CLI binary (`neotrix`) and desktop app (`neotrix-tauri`) both use Tokio. If `#[tokio::main]` is used without explicit `worker_threads`, the system defaults to num_cpus — which is usually correct on desktop/server but wrong on embedded/edge or containerized environments where `num_cpus` reports host cores not cgroup limits.

**Action Required**: Audit all `#[tokio::main]` and `Builder::new_multi_thread()` calls. For containerized deployments, use `std::thread::available_parallelism()` instead of `num_cpus`. Document recommended `worker_threads` in deployment guides.

**Sources**: S11, S12

---

### F755-04: `tokio-console` / `tokio::runtime::dump` — Zero Integration in NeoTrix
**Defect Class**: Observability / No Runtime Introspection
**Severity**: MEDIUM

Tokio provides two mature debugging/inspection tools:
1. **tokio-console** (console-subscriber): Real-time task spawn rates, poll durations, idle/busy time, waker ops, resource utilization. Requires `--cfg tokio_unstable`.
2. **`tokio::runtime::dump`**: Snapshot of runtime state (task backtraces, traces). Requires `tokio_unstable` + Linux + `taskdump` feature. Cross-platform backtrace not available.

Both are **gated behind `tokio_unstable`** cfg flag. NeoTrix has zero integration with either.

**NeoTrix Impact**: When NeoTrix tasks hang (e.g., NT-WORLD crawl stalls, NT-MEMORY KB write hangs), there is no way to introspect which tasks are parked, which are blocked, or what their poll history is. Batch 754 found zero `#[instrument]` spans — combining this with zero tokio-console means NeoTrix is flying blind on async task lifecycle.

**Action Required**:
1. Add `console-subscriber` as dev-dependency, gated behind `tokio_unstable` cfg
2. Add `#[tokio::main]` with `enable_all()` and `on_thread_start`/`on_thread_stop` hooks for lifecycle logging
3. For production: expose a `neotrix runtime-dump` CLI command using `Handle::dump()` (Linux-only, feature-gated)
4. For cross-platform: integrate `tokio-metrics` crate for task poll latency histograms

**Sources**: S8, S9, S16

---

### F755-05: `dial9` — Tokio Flight Recorder (New 2026 Tool)
**Defect Class**: Observability / Missing Production Diagnostics
**Severity**: LOW (new tool, early adoption)

TokioConf 2026 (March 18) introduced **dial9**: a flight recorder for Tokio. Like JVM flight recorders, it captures low-overhead production traces for post-mortem analysis. This is distinct from tokio-console (real-time) and taskdump (snapshot).

**NeoTrix Impact**: dial9 could fill the gap between Batch 754's dead OTel code and the need for production async diagnostics. If dial9 matures, it could replace the custom tracing infrastructure that Batch 754 found dead.

**Action Required**: Monitor dial9 releases. When stable, evaluate as replacement for the dead opentelemetry-stack + tracing-subscriber pipeline.

**Sources**: S16

---

### F755-06: async-std Officially Discontinued (2025)
**Defect Class**: Dependency Risk / Ecosystem Shift
**Severity**: LOW

async-std was officially discontinued in 2025 (per matterai.so, S13). The recommendation is to migrate to smol (similar API philosophy) or Tokio.

**NeoTrix Impact**: If NeoTrix has any async-std dependencies or references, they must be eliminated. Also relevant for skill/domain documentation that may reference async-std as an alternative.

**Action Required**: Grep for `async-std` in Cargo.toml files and documentation. Remove any references.

**Sources**: S13

---

### F755-07: smol Memory Efficiency — 38% Less Than Tokio at Scale
**Defect Class**: Resource Optimization / Opportunity
**Severity**: LOW (design consideration)

At 50k idle concurrent tasks: smol uses 88MB vs Tokio's 142MB (38% less). smol's TCP echo: 1.8KB/connection vs Tokio's 2.4KB. Timer precision: smol ±30μs vs Tokio ±50μs at 1ms.

**NeoTrix Impact**: For edge/embedded scenarios (NT-PHYSICAL physical embodiment domain), smol could be a better fit than Tokio for low-memory targets. However, NeoTrix's ecosystem dependency on tokio-based crates (axum, sqlx, tonic) makes full migration impractical. Consider smol for isolated subsystems only.

**Action Required**: Document smol as an option for NT-PHYSICAL embedded targets. No immediate migration needed for main runtime.

**Sources**: S3, S4, S11, S12

---

### F755-08: Tokio mpsc Channel Bugs — Underflow + Missing Notify
**Defect Class**: Synchronization / Channel Correctness
**Severity**: MEDIUM

Tokio v1.52.3 (May 8, 2026) fixed three sync bugs:
1. **mpsc `len()` underflow** (PR #8062): `len()` could underflow, returning incorrect count
2. **`OwnedPermit::release()` missing notify** (PR #8075): Releasing a permit didn't notify receivers, causing lost wakeups
3. **`RwLock` zero `max_readers`** (PR #8076): `RwLock` with `max_readers=0` was not rejected

**NeoTrix Impact**: If NeoTrix uses `tokio::sync::mpsc` channels (likely in EventBus, crawl pipeline coordination, or cross-domain messaging), any version <1.52.3 is susceptible to:
- Incorrect channel length reporting (affects backpressure logic)
- Lost wakeups on permit release (tasks waiting on channel may hang indefinitely)
- Panics on zero-reader RwLock

**Action Required**: Upgrade to Tokio ≥1.52.3. Audit all `mpsc::channel` and `OwnedPermit` usage for correctness under load.

**Sources**: S1, S17

---

### F755-09: Tokio 2.0 Does NOT Exist — Fictitious Claims
**Defect Class**: Misinformation / Research Integrity
**Severity**: INFO

The dev.to article "Unveiled: Tokio 2.0 Dominates Rust Async Runtimes in 2026" (S2, 2026-03-04) is **fictional/SEO-generated**. Tokio latest stable is v1.53.1 (July 2026). No Tokio 2.0 has been announced. The article's claims about "zero-cost abstractions" redesign, unified channel API, and `spawn` accepting `&mut self` closures are fabricated.

**NeoTrix Impact**: Research agents must not cite this article as evidence of Tokio 2.0 features. All Tokio-related architectural decisions should reference actual releases (v1.x LTS lines).

**Action Required**: Add S2 to a "disinformation" list in research agent configuration. Verify all Tokio claims against github.com/tokio-rs/tokio/releases.

**Sources**: S2, S1

---

### F755-10: Work-Stealing Bulk Operations — Adaptive Chunk Sizing
**Defect Class**: Scheduler Design / Research Insight
**Severity**: LOW (architecture consideration)

arxiv:2603.05766 (March 2026) presents a lock-free work-stealing algorithm for bulk operations. Key insights:
- **Steal-half policy** (used by Go, Tokio): reduces steal frequency but risks over-stealing
- **Adaptive chunk sizing**: dynamically adjust steal count based on victim's load — avoids both under-stealing and over-stealing
- **Block-based Work Stealing (BWoS)**: segment deques into fixed-size blocks, synchronize only at block level — reduces contention
- **NUMA-aware stealing**: indiscriminate stealing causes costly remote memory accesses on multi-socket systems

**NeoTrix Impact**: NeoTrix's custom task scheduling (if any exists beyond Tokio's default) should consider adaptive chunk sizing for bulk crawl operations (NT-WORLD) or parallel KB writes (NT-MEMORY). The current work-stealing is Tokio's built-in; no custom scheduling was found in Batch 754.

**Action Required**: When NeoTrix needs custom scheduling (e.g., prioritized task lanes for GWT attention routing), implement adaptive chunk stealing using BWoS-inspired block deques rather than Tokio's default flat queues.

**Sources**: S14

---

## Summary of New Defects

| ID | Defect | Severity | Component |
|----|--------|----------|-----------|
| F755-01 | LIFO slot CPU regression (1.51.0) | HIGH | Tokio scheduler |
| F755-02 | `before_park` driver skip bug | HIGH | Tokio runtime |
| F755-03 | Worker thread misconfiguration | MEDIUM | NeoTrix runtime config |
| F755-04 | Zero tokio-console/dump integration | MEDIUM | NeoTrix observability |
| F755-05 | Missing dial9 flight recorder | LOW | NeoTrix diagnostics |
| F755-06 | async-std discontinued | LOW | Dependency risk |
| F755-07 | smol memory advantage (design option) | LOW | Architecture |
| F755-08 | mpsc channel bugs (underflow/notify) | MEDIUM | Sync primitives |
| F755-09 | Tokio 2.0 fictitious claims | INFO | Research integrity |
| F755-10 | Adaptive chunk work-stealing research | LOW | Scheduler design |

## Sources Cited

- S1: github.com/tokio-rs/tokio/releases (v1.53.1, v1.52.4, v1.52.3, v1.52.2, v1.51.2)
- S2: dev.to/myroslavmokhammadabd (fictional Tokio 2.0 article)
- S3: rustify.rs/articles/rust-async-runtimes-tokio-vs-async-std-2026
- S4: reintech.io/blog/tokio-vs-async-std-vs-smol-rust-async-runtime-comparison-2026
- S5: lucaberton.com/blog/rust-async-runtimes-tokio-2026
- S6: github.com/tokio-rs/tokio/issues/8065 (LIFO slot regression)
- S7: github.com/tokio-rs/tokio/discussions/8116 (v1.51.1 release)
- S8: github.com/tokio-rs/console
- S9: docs.rs/tokio/latest/tokio/runtime/dump
- S10: github.com/tokio-rs/tokio/releases (v1.52.2 LIFO revert)
- S11: toolsku.com/en/blog/rust-async-runtime-comparison-2026
- S12: johal.in/perf-test-2026-rust-async-runtime-speed-tokio
- S13: matterai.so/guides/rust-networking (async-std discontinued)
- S14: arxiv.org/html/2405.08187v1 + arxiv.org/pdf/2603.05766 (work-stealing research)
- S15: github.com/tokio-rs/tokio/releases (v1.52.4 before_park fix)
- S16: tokio.rs/blog (dial9, TokioConf 2026)
- S17: github.com/tokio-rs/tokio/releases (v1.53.0/v1.53.1)
