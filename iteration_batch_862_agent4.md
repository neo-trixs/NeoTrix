# Agent 4: Performance Profiling (Batch 862)

## Sources

1. https://rs4ts.dev/21-performance/01-flamegraph/ — cargo-flamegraph guide, flamegraph reading, debug=true profiling
2. https://www.rustfaq.org/en/how-to-use-perf-and-flamegraph-with-rust-applications/ — perf+flamegraph pipeline, LTO pitfalls, sampling rates
3. https://oneuptime.com/blog/post/2026-01-07-rust-profiling-perf-flamegraph/view — multi-tool profiling workflow, samply, criterion
4. https://krun.pro/rust-performance-profiling/ — black_box gotchas, monomorphization i-cache pressure, async profiling gaps, PGO
5. https://codehowtoguide.com/how-to-profile-performance-in-rust/ — multi-layer profiling strategy, env stability, criterion CI
6. https://rs4ts.dev/21-performance/00-profiling/ — samply cross-platform, profiling profile convention, frame pointers
7. https://hotpath.rs/blog/sampling_comparison — sampling vs instrumentation for async, wall-clock vs CPU time
8. https://nnethercote.github.io/perf-book/profiling.html — Rust Performance Book profiling chapter, frame pointers, symbol mangling
9. https://github.com/criterion-rs/criterion.rs — criterion.rs benchmarking framework
10. https://bheisler.github.io/criterion.rs/book/user_guide/profiling.html — criterion profiling hooks, --profile-time

## Defects

D-PROF-001: Release profile uses `opt-level = "s"` (size) instead of `opt-level = 3` (speed) — conscious architecture hot paths (E8 reasoning, GWT attention routing, VSA HyperCube ops) are optimized for binary size rather than throughput | `/Users/neo/Downloads/neotrix/Cargo.toml:37` | high | Source: krun.pro (opt-level discussion), codehowtoguide.com (release profile best practices)

D-PROF-002: No `[profile.profiling]` target exists — community convention is `[profile.profiling] inherits = "release" debug = true strip = false` for flamegraph/samply; current setup has `strip = "symbols"` in release making profiler output unreadable without rebuild | `/Users/neo/Downloads/neotrix/Cargo.toml:36-41` | high | Source: rs4ts.dev (profiling profile), rustfaq.org (LTO and profiling), nnethercote.github.io (profiling chapter)

D-PROF-003: `repair_c3.rs` benchmark is a no-op placeholder (`black_box(1)`) — NT-REPAIR self-healing domain has zero C3 benchmark evidence; cannot detect performance regressions in repair workflows | `/Users/neo/Downloads/neotrix/neotrix-core/benches/repair_c3.rs:8-10` | medium | Source: criterion.rs (regression detection), codehowtoguide.com (baseline importance)

D-PROF-004: `seal_core/stats.rs` clones HashMap keys (`record.tier.clone()`) on every cost_log iteration without pre-allocated capacity — RouterStats::from_cost_log runs on every SEAL pipeline tick; allocation pressure scales with routing volume | `/Users/neo/Downloads/neotrix/neotrix-core/src/unified/layers/cognition/nt_mind/seal_core/stats.rs:60-62` | medium | Source: krun.pro (allocation invisible to flamegraph), codehowtoguide.com (Vec::with_capacity)

D-PROF-005: `egress_privacy_guard` scans messages, model, image_data, constraint_json, structured_output, provider_params, and tools on every outbound LLM request — no short-circuit for Trusted tier after secret scrub; full scan runs even when trust=Trusted would skip after line 590 | `/Users/neo/Downloads/neotrix/neotrix-core/src/unified/core/nt_core_llm.rs:584-619` | medium | Source: hotpath.rs (wall-clock vs CPU profiling), krun.pro (sampling profilers miss allocation pressure)

D-PROF-006: Disabled performance benchmark (`benches/_disabled/performance_benchmark.rs`) creates a new `tokio::runtime::Runtime::new()` inside `b.iter()` for every benchmark iteration — runtime creation cost (~1-5ms) dominates measurement, making all FusedArchitecture benchmarks meaningless | `/Users/neo/Downloads/neotrix/benches/_disabled/performance_benchmark.rs:25,51,65,91,112,133,155,169,176,190,204,216,236,252,259,283,286` | high | Source: criterion.rs (setup outside hot loop), hotpath.rs (sampling profiler noise from setup)

D-PROF-007: No criterion benchmarks exist for NT-CORE foundational modules (E8 reasoning engine, HyperCube VSA ops, GWT attention routing, ConsciousnessTree) — these are the most performance-critical components with no regression detection | No bench target | high | Source: criterion.rs (what to benchmark), codehowtoguide.com (CI regression detection)

D-PROF-008: `HeartbeatAggregator::record()` allocates a new `String` for every component name and clones `ComponentHealth` on each heartbeat tick — with 11+ ConsciousnessTree branches reporting, this creates ~11 String allocations per aggregation cycle | `/Users/neo/Downloads/neotrix/neotrix-core/src/unified/core/nt_core_heartbeat.rs:44-53` | low | Source: krun.pro (allocation pressure invisible to flamegraph), hotpath.rs (allocation profiling gap)

D-PROF-009: Criterion version pinned to 0.5 while latest is 0.8 — missing 3 major versions of improvements including async benchmark support, improved statistical analysis, and better regression detection | `/Users/neo/Downloads/neotrix/neotrix-core/Cargo.toml:138` | low | Source: criterion.rs (version history), criterion 0.8 docs (async support, tokio integration)

D-PROF-010: `lto = true` in release profile may break flamegraph stack traces — LTO inlines across crate boundaries collapsing call trees into unreadable single blocks; profiling convention is `lto = false` in profiling profile | `/Users/neo/Downloads/neotrix/Cargo.toml:38` | medium | Source: rustfaq.org (LTO breaks stack traces), nnethercote.github.io (LTO and profiling)

## Key Insights

1. **The `opt-level = "s"` is the single highest-leverage fix.** For an AI-native developer toolkit where E8 reasoning, GWT attention routing, and VSA HyperCube operations are CPU-bound, size optimization actively hurts throughput. Switching to `opt-level = 3` or at minimum `opt-level = 2` could yield 15-40% improvement on compute-bound paths with minimal binary size increase.

2. **No profiling infrastructure exists.** The project has zero `[profile.profiling]` configuration, no frame pointer flags (`-C force-frame-pointers=yes`), and release strips symbols. Any performance investigation requires manual Cargo.toml surgery before profiling can begin — this is the anti-pattern that causes engineers to "optimize by guessing."

3. **The benchmark suite has critical gaps.** NT-REPAIR is a placeholder. The disabled benchmark has a Tokio runtime creation anti-pattern. Most critically, the foundational NT-CORE modules (E8, HyperCube, GWT, ConsciousnessTree) — which are the most CPU-sensitive components — have zero benchmarks. This means regressions in the reasoning engine go undetected.

4. **Allocation pressure is invisible.** The `egress_privacy_guard` scans every field of every outbound request. `RouterStats::from_cost_log` clones keys on every iteration. `HeartbeatAggregator` allocates per-component Strings. These allocation patterns won't appear in CPU flamegraphs but will manifest as tail latency under load. DHAT or `dhat` profiling is needed to expose these.

5. **Async profiling gap.** NeoTrix uses Tokio extensively but has no `tokio-console` integration and no async-aware benchmarking. Sampling profilers (perf/samply) will show executor internals rather than logical async work, making it impossible to identify which async task is the bottleneck without instrumentation-based tools.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| Sources consulted | 10 |
| Critical path gaps identified | 4 (opt-level, profiling profile, E8/HyperCube benchmarks, async profiling) |
