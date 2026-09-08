# Iteration Batch 675 — Profiling, APM, Benchmarking Gap Analysis

**Date**: 2026-09-06
**Context**: Iteration 675 of 10000+. Batch 674 proved (1) no composition root, (2) Service Locator in EventBus, (3) captive dependency in HeartbeatAggregator, (4) EventBus root cause of 6/13 defects, (5) no framework-managed lifecycle. This batch searches 2026 profiling/APM/benchmark state-of-the-art and maps each finding to NEW NeoTrix defects.

---

## Sources Consulted

| # | Source | Date | Domain |
|---|--------|------|--------|
| S1 | oneuptime — Rust Profiling with perf/flamegraph/samply | 2026-01-07 | Profiling |
| S2 | martinuke0 — Deep Dive into Flame Graphs: Profiling Blind Spots, Sampling Bias | 2026-05-22 | Profiling |
| S3 | Augment Code — Application Performance Monitoring: The 2026 Guide | 2026-05-21 | APM |
| S4 | Narwal — APM 2026 Guide: Key Metrics, Tools, Root Cause Analysis | 2026-09-02 | APM |
| S5 | rajpoot.dev — Rust Performance in 2026: Benchmarking, Profiling, Real Wins | 2026-05-04 | Benchmarks |
| S6 | ADHD ecode — Benchmark Rust with cargo bench and Criterion (2026) | 2026-04-16 | Benchmarks |
| S7 | bencher.dev — How to track Rust Criterion benchmarks in CI | 2024-11-09 | CI Benchmarks |
| S8 | ByteLedger — Flame Graph Profiling Guide for 2026 | 2026-07-25 | Profiling |
| S9 | PeerSpot — Best APM and Observability Tools (Aug 2026) | 2026-08 | APM |
| S10 | tech2geek — 30 Best Server Monitoring, APM & Observability Tools for 2026 | 2026-02-14 | APM |

---

## What's NEW vs Batch 674

Batch 674 identified architectural defects (no composition root, Service Locator, captive dependency). This batch shifts lens to **observability/performance infrastructure** and discovers 8 NEW defects that are invisible without profiling — the most critical being that the EventBus performance characteristics are unmeasurable, making the Service Locator defect from 674 impossible to quantify or fix data-driven.

---

## DEFECT-P675-01: No Profiling Profile — Profiling Impossible Without Manual Config (PERSISTED, DEEPER)

**Status**: PERSISTED from P431-01, DEEPER analysis with 2026 data
**Severity**: HIGH
**Location**: `/Users/neo/Downloads/neotrix/Cargo.toml:34-38`

**Evidence**: Workspace defines:
```toml
[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
strip = "symbols"
panic = "abort"

[profile.bench]
inherits = "release"
debug = true
```
There is **no `[profile.profiling]`** profile. The `strip = "symbols"` in release means `perf`/`samply` output is hex gibberish without manual intervention. LTO being enabled breaks stack traces (S1, S2, S8).

**Gap vs 2026 best practice** (S1, S5):
```toml
[profile.profiling]
inherits = "release"
debug = true
lto = false          # LTO breaks stack traces
codegen-units = 16   # parallel codegen for faster builds
strip = false        # preserve symbols for profilers
```

**NEW insight from S2**: Flame graphs have **sampling bias** — periodic sampling aliases with periodic workload patterns. NeoTrix's EventBus processes events in bursts; a naive 99Hz sampler will over/under-represent burst processing depending on alignment. Must use `-j random` jitter (S2). Without a profiling profile, this cannot even be tested.

---

## DEFECT-P675-02: No Allocation Profiling — Memory Leaks Invisible in Daemon Processes (PERSISTED, DEEPER)

**Status**: PERSISTED from P431-06, DEEPER with dhat-rs 2026 data
**Severity**: HIGH
**Location**: All daemon entry points (`run_daemon`, `run_background_daemon`)

**Evidence**: No DHAT, heaptrack, or dhat-rs integration. No `jemalloc`/`mimalloc` allocator configured. S5 confirms: "Allocations are often the hidden cost in Rust. dhat finds them." S6 confirms Criterion's `black_box` prevents optimization but does NOT prevent excessive allocation.

**2026 data** (S5, S6):
```toml
[dev-dependencies]
dhat = "0.3"
```
```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
fn main() {
    let _profiler = dhat::Profiler::new_heap();
    // ... run workload ...
}
// Produces dhat-heap.json — view in dhat viewer
```

**NEW connection to Batch 674**: EventBus dispatches events through heap-allocated `HashMap<String, Vec<Box<dyn Any>>>`. Without allocation profiling, the captive dependency pattern (HeartbeatAggregator creating its own EventBus) cannot be quantified — how many extra allocations does each captive EventBus instance produce? dhat-rs would answer this in one profiling run.

---

## DEFECT-P675-03: No CI Benchmark Regression Detection — Performance Regressions Ship Silently (NEW)

**Severity**: HIGH
**Location**: No `bencher` or CI benchmark tracking configured

**Evidence**: NeoTrix has 7 criterion benchmark files (`memory_c3.rs`, `vector_ops.rs`, `repair_c3.rs`, `real_tasks.rs`, `act_c3.rs`, `affective_c3.rs`, `performance_benchmark.rs` [disabled]). But there is NO CI step that runs `cargo bench` and compares against baseline. S7 (bencher.dev) confirms: "For the same reasons that unit tests are run in CI to prevent feature regressions, benchmarks should be run in CI with Bencher to prevent performance regressions. Performance bugs are bugs!"

**Gap**: Bencher wraps Criterion output, stores results over time, detects regressions via statistical analysis. NeoTrix runs benchmarks locally but never gates PRs on benchmark results. A PR that silently doubles EventBus latency would merge without detection.

**S5 confirms** the #1 mistake: "Optimizing without profiling — you 'know' what's slow. You're wrong. Profile." Without CI benchmark gating, even known-slow code can regress further without anyone noticing.

---

## DEFECT-P675-04: No OpenTelemetry Tracing — Cross-Module Communication Invisible (NEW)

**Severity**: HIGH
**Location**: Entire NeoTrix architecture

**Evidence**: S3 (Augment Code 2026 guide) states: "The modern APM approach in 2026 is a unified observability layer that correlates traces, metrics, logs, real user telemetry, synthetic checks, and continuous profiling against a shared OpenTelemetry resource model." S4 (Narwal 2026) confirms: "A slow checkout page might trace back to a database query three services away."

NeoTrix has 7+ domains (NT-CORE, NT-MEMORY, NT-WORLD, NT-ACT, NT-IO, NT-SHIELD, NT-FEEL) communicating via EventBus. Without distributed tracing, there is NO visibility into:
- How a GWT attention broadcast propagates across modules
- Where EventBus dispatch latency accumulates
- Which module pair has the highest communication cost
- How the SEAL pipeline stages chain across domains

**Connection to Batch 674**: The EventBus Service Locator defect (674) is UNDIAGNOSABLE without tracing. You cannot determine if the Service Locator pattern is actually causing performance problems without measuring cross-module latency. OpenTelemetry traces would make this measurable.

---

## DEFECT-P675-05: Async Profiling Blind Spot — Tokio Work Misrepresented (PERSISTED, DEEPER)

**Status**: PERSISTED from P431-08, DEEPER with S2 data
**Severity**: MEDIUM
**Location**: All tokio-based daemon code

**Evidence**: S2 (martinuke0 2026) confirms: "Periodic sampling can alias with periodic workload patterns." S1 (OneUptime) confirms: samply measures CPU time only, not wall-clock async time. NeoTrix uses tokio extensively (daemon modes, crawl pipeline, KB operations). A sampling profiler on NeoTrix daemon code will show executor mechanics (tokio poll/wake), not logical async work.

**2026 mitigation** (S1): `RUSTFLAGS="-C force-frame-pointers=yes"` for macOS stack traces. `hotpath` crate for wall-clock async measurement. `tokio-console` for runtime diagnostics.

**NEW insight**: S2's "Architecture of a Production-Grade Profiling Pipeline" pattern (Sampling Agent → Kafka → Batch Processor → Dashboard) is the 2026 standard for fleet profiling. NeoTrix runs as a single daemon but the pattern still applies — continuous 30-second sampling windows every hour, stored for trend analysis.

---

## DEFECT-P675-06: Flame Graph Blind Spots — Lock Contention and I/O Wait Invisible (NEW)

**Severity**: MEDIUM
**Location**: EventBus (Mutex contention), KB (SQLite I/O wait), HeartbeatAggregator

**Evidence**: S2 explicitly lists flame graph blind spots:
1. **Lock wait** — "Lock wait, GC pauses, and I/O wait require supplemental tools (eBPF, lock tracing) to surface"
2. **I/O wait** — "Missing kernel stacks hides system-call overhead, I/O wait, and scheduler latency"
3. **Kernel vs user stacks** — Must use `perf record -g -k call-graph` to see both

NeoTrix's EventBus uses `Arc<Mutex<T>>` for event dispatch (batch 674 finding). Under high event throughput, this Mutex will contend. A flame graph will show the Mutex::lock as a wide bar but will NOT distinguish between:
- Normal lock acquisition (fast path)
- Contention-caused waiting (slow path)
- I/O-caused lock hold (SQLite write blocking lock)

**Fix**: Pair flame graphs with lock tracing (`perf lock` or eBPF lock contention tools). For NeoTrix specifically: instrument EventBus dispatch with `tracing::instrument` spans that record lock wait time separately from lock hold time.

---

## DEFECT-P675-07: No Production CPU/Memory Profiling — Daemon Blind in Production (NEW)

**Severity**: HIGH
**Location**: Daemon processes in production

**Evidence**: S2's production profiling patterns:
- **Continuous Profiling**: "Collect 10-second samples every hour, store forever. Large fleets where long-term trends matter."
- **On-Demand Profiling**: "Triggered by an alert (e.g., latency > 95th percentile)."
- **Canary Profiling**: "Run a higher-frequency sampler on a 1% traffic canary."

S3 (Augment Code) confirms: "Production latency propagates across services, and metrics or traces alone often stop short of the responsible function." The APM 2026 standard is **full-stack observability**: metrics + traces + logs + profiling + RUM.

NeoTrix daemon processes run indefinitely. Without production profiling:
- EventBus throughput degradation over time is undetectable
- Memory growth (leak or fragmentation) in KB operations is invisible
- CPU hotspots in E8 reasoning under real workloads are unknown
- The "Alabaster (monitor) rune socket" (CONTEXT.md) has no profiling backend

**2026 tooling** (S1): `samply record --pid $(pgrep myapp) --duration 60` attaches to running process, opens Firefox Profiler UI. Zero config for basic profiling.

---

## DEFECT-P675-08: No Benchmark Memory Profiling — Allocation Hotspots in Hot Paths Unknown (NEW)

**Severity**: MEDIUM
**Location**: All criterion benchmark files

**Evidence**: S5 lists the top 5 Rust slowness causes:
1. Cloning everywhere
2. Allocations in hot paths
3. Vec growth (reallocation)
4. Box in hot paths (dynamic dispatch)
5. Sync vs async overhead

S6 confirms Criterion provides timing but NOT memory profiling. NeoTrix's benchmark files (`vector_ops.rs`, `memory_c3.rs`, `act_c3.rs`) measure throughput but not allocation pressure. A benchmark that shows "500ns per operation" might be doing 10 heap allocations per iteration — the timing looks fine but the allocation pressure under real load (thousands of concurrent operations) would be catastrophic.

**2026 integration** (S5): dhat-rs integrates with benchmarks:
```rust
// In bench setup
let _profiler = dhat::Profiler::new_heap();
// Run benchmark iterations
// dhat-heap.json shows allocation sites with stack traces
```

**Connection to EventBus**: EventBus dispatch clones event data (`Box<dyn Any>`). Without memory profiling in benchmarks, the allocation cost of each EventBus dispatch is unknown. Under high throughput (GWT attention broadcasts), this could be the dominant cost.

---

## DEFECT-P675-09: No Flame Graph CI Regression Detection — Performance Visual Regressions Invisible (NEW)

**Severity**: MEDIUM
**Location**: No CI integration for flame graph diffing

**Evidence**: S2 details a production CI pattern:
```bash
# CI step
perf record -F 99 -g --timeout 30 -- ./myservice --bench
perf script | stackcollapse-perf.pl > new.folded
flamegraph.pl new.folded > new.svg
flamegraph.pl --diff baseline.svg new.svg > diff.svg
if grep -q "diff-color" diff.svg; then
    echo "Performance regression detected"
    exit 1
fi
```

S8 (ByteLedger) confirms: "Automate alerts and CI checks on flame-graph diffs to catch regressions before they hit users." S5 confirms: "Optimizing without profiling — you're wrong. Profile."

NeoTrix has no mechanism to detect when a code change widens a flame graph bar (increases CPU time in a function). A PR that introduces an O(n²) loop in the SEAL pipeline evaluator would merge silently.

---

## DEFECT-P675-10: APM Observability Gap — No Real-User / Synthetic Monitoring for CLI (NEW)

**Severity**: LOW
**Location**: NeoTrix CLI/TUI

**Evidence**: S3 (Augment Code) defines 2026 APM as: "traces + metrics + logs + profiling + RUM + synthetic." S9 (PeerSpot) confirms: "These tools significantly enhance Mean Time to Resolution (MTTR) by offering real-time visibility."

NeoTrix CLI is a developer tool. While RUM (Real User Monitoring) doesn't directly apply, the **equivalent** for a CLI tool is:
- **Command latency tracking**: How long does each CLI command take? (p50/p95/p99)
- **Error rate by command**: Which commands fail most?
- **Usage patterns**: Which features are used together?

NeoTrix has `nt_io` (interface domain) but no telemetry about its own CLI usage. This is not a production APM concern but a product analytics concern — understanding which NeoTrix features are actually used and where they slow down.

---

## Summary Matrix

| ID | Defect | Severity | Status | Root Cause Category |
|----|--------|----------|--------|-------------------|
| P675-01 | No profiling profile | HIGH | PERSISTED+DEEPER | Build infrastructure |
| P675-02 | No allocation profiling | HIGH | PERSISTED+DEEPER | Build infrastructure |
| P675-03 | No CI benchmark regression | HIGH | **NEW** | CI/CD pipeline |
| P675-04 | No OpenTelemetry tracing | HIGH | **NEW** | Observability |
| P675-05 | Async profiling blind spot | MEDIUM | PERSISTED+DEEPER | Profiling methodology |
| P675-06 | Flame graph blind spots (locks/IO) | MEDIUM | **NEW** | Profiling methodology |
| P675-07 | No production CPU/mem profiling | HIGH | **NEW** | Observability |
| P675-08 | No benchmark memory profiling | MEDIUM | **NEW** | Benchmark methodology |
| P675-09 | No flame graph CI regression | MEDIUM | **NEW** | CI/CD pipeline |
| P675-10 | No CLI telemetry/analytics | LOW | **NEW** | Product observability |

**Severity distribution**: HIGH: 5, MEDIUM: 4, LOW: 1

---

## Key Insight: The Observability Void Compounds Architecture Defects

Batch 674 proved architectural defects (no composition root, Service Locator, captive dependency). This batch reveals that **these defects are UNMEASURABLE** without observability infrastructure. The chain is:

1. EventBus has Service Locator pattern (batch 674) → but we can't measure its performance cost
2. HeartbeatAggregator creates captive EventBus (batch 674) → but we can't measure allocation overhead
3. No composition root (batch 674) → but we can't trace cross-module communication costs
4. No framework-managed lifecycle (batch 674) → but we can't profile startup/shutdown sequences

**The observability void means every architectural defect from batch 674 is a HYPOTHESIS, not a MEASURED FACT.** Without profiling, tracing, and benchmarking infrastructure, we are debugging blind.

**Priority**: P675-01 (profiling profile) → P675-03 (CI benchmarks) → P675-04 (OpenTelemetry) → P675-07 (production profiling) → P675-02 (allocation profiling) → P675-06 (lock tracing) → P675-05 (async profiling) → P675-09 (flame graph CI) → P675-08 (bench memory) → P675-10 (CLI telemetry)
