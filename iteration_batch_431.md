# Iteration Batch 431 — Performance Profiling, Benchmarking & Optimization

**Date**: 2026-09-06
**Focus**: External research → Design defect identification → Optimization suggestions

---

## 1. Sources Cited

### Performance Profiling (2026)
| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| 1 | [rustfaq.org — perf + flamegraph](https://www.rustfaq.org/en/how-to-use-perf-and-flamegraph-with-rust-applications/) | 2026-04-17 | Profiling workflow: `debug = true` in release profile, `perf record -g --call-graph dwarf`, LTO breaks stack traces — disable for profiling builds |
| 2 | [oneuptime — perf/flamegraph/samply](https://oneuptime.com/blog/post/2026-01-07-rust-profiling-perf-flamegraph/view) | 2026-01-07 | samply as cross-platform profiler using Firefox Profiler UI; DHAT for allocation profiling; `RUSTFLAGS="-C force-frame-pointers=yes"` for macOS stack traces |
| 3 | [hotpath.rs — instrumentation vs sampling](https://hotpath.rs/blog/sampling_comparison) | 2025-12-17 | `hotpath` crate measures logical wall-clock time (including async await); sampling profilers (perf/samply/flamegraph) measure CPU time only; async workloads produce misleading samply results (410% vs 45% discrepancy) |
| 4 | [mstange/samply v0.13.1](https://github.com/mstange/samply) | 2025-02-01 | Cross-platform (macOS/Linux/Windows); off-cpu sampling on macOS/Windows; default 1000Hz; Firefox Profiler integration; `--profile profiling` recommended |
| 5 | [flamegraph-rs v0.6.13](https://github.com/flamegraph-rs/flamegraph) | 2026-06-03 | LTO breaks stack traces; `--no-inline` flag for perf script; recommended to also try samply for interactive UI |

### Benchmarking (2026)
| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| 6 | [iai-callgrind 0.16.1](https://docs.rs/crate/iai-callgrind/latest) | 2026 | Single-shot benchmarking via Valgrind; instruction counts + L1/L2/RAM access counts; integrated DHAT heap profiling since 0.16.0; CI-stable in noisy environments; no Windows support |
| 7 | [Bencher — iai vs criterion](https://bencher.dev/learn/benchmarking/rust/iai/) | 2024-02-13 | Criterion for wall-clock; iai for instruction-count baselines; use both for comprehensive picture |
| 8 | [LambdaClass — criterion + iai](https://blog.lambdaclass.com/benchmarking-and-analyzing-rust-performance-with-criterion-and-iai/) | 2022-04-30 | Criterion not suitable for CI (noise); iai provides single-shot precision; flamegraph as visual complement |

### Performance Optimization (2026)
| # | Source | Date | Key Advance |
|---|--------|------|-------------|
| 9 | [rust-lang/rust-project-goals — scalable-vectors](https://github.com/rust-lang/rust-project-goals/blob/main/src/2026/scalable-vectors.md) | 2026 | RFC #3838 for Scalable Vectors; Sized hierarchy stabilization; SVE/SME for AArch64; extern types unblocked |
| 10 | [elijahpotter — compile times 71%](https://elijahpotter.dev/articles/improving_rust_compile_times_by_71_percent) | 2025-11-05 | Disabling LTO + reverting codegen-units to default → 71% CI compile time reduction; only 3% runtime regression; "compiler creep" analogy |
| 11 | [johal.in — Rust 1.90 SIMD + LLVM 18](https://johal.in/deep-dive-rust-190s-new-simd-instructions-work) | 2026-05-01 | `std::simd` stabilized portable SIMD API; 14 new ARM SVE2 intrinsics; LLVM 18 Graviton6 scheduling model; 4.7x faster vectorized workloads; `target-cpu=neoverse-v2` critical for ARM; alignment metadata reduces load latency 18% |
| 12 | [Krun — Rust optimization guide](https://krun.pro/rust-performance-optimization/) | 2026-05-11 | `RUSTFLAGS="-C target-cpu=native"` for AVX2/SSE4.2; `std::simd` stabilized in 1.78+; `pulp`/`wide`/`macerator` for portable SIMD; trait objects vs generics (vtable overhead); `cargo-asm` for verifying vectorization |
| 13 | [boardor — SIMD complete guide](https://boardor.com/blog/complete-guide-to-rust-simd-programming-from-beginner-to-expert) | 2026-06-26 | Decision tree: auto-vectorization → `wide` (no multiversioning) → `pulp` (multiversioning) → raw intrinsics; function multiversioning for binary distribution; SIMD f32/f64 not auto-vectorized by default |
| 14 | [rust PR #155005 — preserve SIMD element type](https://github.com/rust-lang/rust/pull/155005) | 2026-04-08 | LLVM preserves element type metadata for AArch64; prevents bitcast confusion in pattern matcher; enables portable deinterleaving loads |
| 15 | [Kunal Ganglani — sccache + mold 2026](https://www.kunalganglani.com/blog/reduce-rust-compile-time) | 2026-08-16 | 5-metric baseline (cold/warm/incremental/sccache-hit/link-time); sccache with remote S3 backend; mold linker drops link time 4-8s → 1-3s; `cargo build --timings` for build graph visibility; `-Zmacro-stats` for bloat detection |
| 16 | [DRM HSE — cargo-turbo](https://www.drmhse.com/posts/cutting-a-cold-cargo-check-from-24-seconds-to-half-a-second/) | 2026-08-17 | `cargo turbo` tool: CoW snapshot + checksum freshness → rust-analyzer cold check 23.77s → 0.47s (50.6x); adaptive thread allocation; no patches needed for wrapper-based approach |
| 17 | [VegaLoop — CI compile times](https://vegaloop.com/blog/technical/taming-rust-compile-times-in-ci/) | 2026-06-05 | sccache over target-dir caching; parallel CI jobs (build + test in parallel); separate `ci` profile with `debug=0`; deterministic features across jobs |
| 18 | [rust-lang/rust PR #160005 — memcpy optimization](https://github.com/rust-lang/rust/pull/160005) | 2026 | Excessive memcpy in new solver; fixes for nacl/ijson/nvml-wrapper-sys; -40% wall-time improvement |
| 19 | [rustperfbook — compile times](https://nnethercote.github.io/perf-book/compile-times.html) | 2026 | `cargo llvm-lines` for IR bloat; generic function monomorphization as #1 culprit; extract non-generic inner functions |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-P431-01: No profiling profile — profiling impossible without manual config
**Severity**: HIGH
**File**: `Cargo.toml:34-46`
**Evidence**: Workspace defines `[profile.release]` with `opt-level = "s"` + `lto = true` + `codegen-units = 1` + `strip = "symbols"`. `[profile.bench]` inherits release with `debug = true`. There is **no `[profile.profiling]`** profile.
**Impact**: To profile, developers must manually modify release profile or create a custom one every time. The `strip = "symbols"` in release means `perf`/`samply` output is hex gibberish without manual intervention. LTO being enabled breaks stack traces (sources 1,5).
**Gap vs 2026 best practice**: Community convention is a dedicated `[profile.profiling]` with `debug = true`, `lto = false`, `codegen-units = 16` (source 15, 1). NeoTrix has none.

### DEFECT-P431-02: Benchmark profile lacks sampling-optimized settings
**Severity**: MEDIUM
**File**: `Cargo.toml:44-46`
**Evidence**: `[profile.bench]` inherits release (`lto = true`, `codegen-units = 1`). This makes benchmarks slower to build and LTO can distort flamegraph/callgrind profiles.
**Impact**: `cargo bench` builds with LTO enabled means:
- Benchmark build times are unnecessarily long
- Flamegraph stacks are collapsed/inlined (source 1,5)
- iai-callgrind instruction counts include LTO-induced inlining noise
**Gap**: Should inherit from a profiling-friendly profile with `lto = false` or `lto = "thin"` for benchmark-specific builds.

### DEFECT-P431-03: No SIMD optimization for VSA HyperCube vector operations
**Severity**: HIGH
**File**: `neotrix-core/benches/vector_ops.rs`
**Evidence**: Benchmark uses scalar `f64` iterator chains: `a.iter().zip(b.iter()).map(|(x, y)| a * b).sum()` for dot product and cosine similarity. The `simd-vsa` feature exists but references `dep:holon` (external crate), and no `std::simd` or `pulp` usage is found anywhere in vector hot paths.
**Impact**: VSA HyperCube is the knowledge representation core. Vector similarity (cosine, dot product, L2 norm) operations are on the critical path for every attention routing decision. Scalar f64 iteration leaves 4-8x performance on the table on AVX2/NEON hardware (source 11,12,13).
**Gap vs 2026**: Rust 1.90 stabilized `std::simd` with portable SIMD (source 11). `pulp` crate provides safe multiversioning (source 12,13). NeoTrix has zero SIMD in hot vector paths.

### DEFECT-P431-04: No `target-cpu=native` for release builds
**Severity**: MEDIUM
**File**: `Cargo.toml:34-39`, no `RUSTFLAGS` anywhere
**Evidence**: No `.cargo/config.toml` exists. No `RUSTFLAGS` set in any Cargo.toml. Release profile has no target-cpu setting.
**Impact**: LLVM generates conservative baseline x86-64 (SSE2 only) or generic AArch64 code. Auto-vectorization is suppressed for AVX2/AVX-512/SVE/SVE2. For a consciousness kernel running on developer workstations and ARM servers, this means 30-50% suboptimal SIMD performance (source 11,12).
**Gap**: `RUSTFLAGS="-C target-cpu=native"` for dev builds; explicit target-cpu for production cross-compilation.

### DEFECT-P431-05: No iai-callgrind for CI regression detection
**Severity**: MEDIUM
**Evidence**: Only `criterion` (wall-clock) is used. No `iai-callgrind` dependency anywhere. No CI benchmark regression detection.
**Impact**: Criterion benchmarks are unreliable in CI due to virtualization noise (sources 6,7,8). Performance regressions can land silently. NeoTrix's 7 domain benchmarks (`shield_c3`, `memory_c3`, `act_c3`, `affective_c3`, `repair_c3`, `real_tasks`) are all criterion-based — none can reliably detect regressions in CI.
**Gap**: iai-callgrind 0.16.0+ provides instruction-count-based single-shot benchmarking, immune to CI noise. Should complement (not replace) criterion.

### DEFECT-P431-06: No allocation profiling integration
**Severity**: MEDIUM
**Evidence**: `benches/_disabled/performance_benchmark.rs` is disabled. No DHAT, heaptrack, or dhat-rs integration. No `jemalloc`/`mimalloc` allocator configured.
**Impact**: NeoTrix runs long-lived daemon processes (`run_daemon`, `run_background_daemon`). Without allocation profiling, memory leaks and excessive allocations in hot paths go undetected. The VSA HyperCube knowledge engine, KB pipeline, and crawl pipeline all do heavy allocation.
**Gap**: DHAT (via iai-callgrind 0.16.0) for allocation profiling; jemalloc/mimalloc for production allocation performance.

### DEFECT-P431-07: No build caching infrastructure (sccache/mold)
**Severity**: LOW (dev-experience)
**Evidence**: No `.cargo/config.toml`, no sccache configuration, no mold linker setup, no `cargo build --timings` baseline recorded.
**Impact**: Full rebuilds from scratch are slow. CI builds recompile everything. No measurement baseline exists to track compile time regressions. With 6+ workspace crates, this compounds.
**Gap**: sccache + mold for Linux CI; `cargo build --timings` baseline tracking; `cargo llvm-lines` for IR bloat detection.

### DEFECT-P431-08: Async profiling blind spot — samply measures CPU time only
**Severity**: LOW
**Evidence**: NeoTrix uses tokio extensively (all daemon modes, crawl pipeline, KB operations). No async-aware profiling tooling configured.
**Impact**: Sampling profilers (perf/samply/flamegraph) show misleading results for async code — they see executor mechanics, not logical async work (source 3). Wall-clock async operations appear much shorter than they actually are.
**Gap**: `hotpath` crate or `tokio-console` for async-aware profiling. Instrument critical async paths (crawl, KB write, LLM calls) with wall-clock timing.

### DEFECT-P431-09: Disabled benchmark suggests architectural drift
**Severity**: LOW
**File**: `benches/_disabled/performance_benchmark.rs`
**Evidence**: `FusedArchitecture` benchmarks are in `_disabled/` directory. `FusedArchitecture` is referenced in bench code but `nt_capability_bridge.rs:89` maps `("simd", "NT-CORE", "performance_concurrency")`.
**Impact**: The benchmark-to-feature binding has drifted. Benchmarks that once tracked a feature are now disabled without tracking the feature's current state.

### DEFECT-P431-10: No compile-time performance baseline tracking
**Severity**: LOW
**Evidence**: No `cargo build --timings` output committed. No `-Zmacro-stats` analysis. No `cargo llvm-lines` analysis.
**Impact**: "Compiler creep" (source 10) — compile times grow silently. No baseline to detect regressions. Generic monomorphization bloat (the #1 compile-time culprit per source 19) is unmonitored.

---

## 3. Suggestions

### S431-01: Add `[profile.profiling]` to workspace Cargo.toml
```toml
[profile.profiling]
inherits = "release"
debug = true
lto = false
codegen-units = 16
strip = "none"
```
Then `cargo build --profile profiling` gives perf-ready binaries with symbols, no LTO interference, and parallel codegen.

### S431-02: SIMD-vectorize VSA HyperCube hot paths
Replace scalar f64 iteration with `pulp` crate (safe, multiversioning, stable):
```rust
use pulp::Arch;
fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    Arch::new().dispatch(|| {
        a.chunks_exact(4).zip(b.chunks_exact(4))
            .map(|(x, y)| x[0]*y[0] + x[1]*y[1] + x[2]*y[2] + x[3]*y[3])
            .sum()
    })
}
```
Or `std::simd` on nightly for max performance.

### S431-03: Add iai-callgrind to dev-dependencies for CI benchmarks
Add parallel criterion + iai-callgrind benchmark suites. Criterion for local dev (wall-clock feedback), iai-callgrind for CI regression gates.

### S431-04: Set `RUSTFLAGS="-C target-cpu=native"` in `.cargo/config.toml`
```toml
[build]
rustflags = ["-C", "target-cpu=native"]
```
For cross-compilation targets, override per-target with explicit `target-cpu`.

### S431-05: Adopt sccache + mold for CI and local dev
```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```
Set `RUSTC_WRAPPER=sccache` in environment.

### S431-06: Add allocation profiling baseline
Add `dhat-rs` for development builds or integrate DHAT via iai-callgrind. For production, consider `jemalloc` or `mimalloc` as global allocator for daemon processes.

### S431-07: Add compile-time baseline tracking
Commit `cargo build --timings` output periodically. Add `cargo llvm-lines` to CI for IR bloat detection. Use `-Zmacro-stats` to monitor proc-macro expansion costs.

### S431-08: Add async-aware profiling for daemon modes
Instrument `run_daemon`, crawl pipeline, and KB operations with `hotpath` or `tokio-console` for wall-clock async timing, not just CPU sampling.

---

## 4. Summary

| Metric | Value |
|--------|-------|
| Sources analyzed | 19 |
| Defects identified | 10 |
| HIGH severity | 2 (no profiling profile, no SIMD in hot paths) |
| MEDIUM severity | 4 (bench profile, no target-cpu, no iai-callgrind, no allocation profiling) |
| LOW severity | 4 (no sccache/mold, async blind spot, disabled bench drift, no compile baseline) |
| Concrete suggestions | 8 |

**Priority order**: S431-01 (profiling profile) → S431-02 (SIMD vectors) → S431-04 (target-cpu) → S431-03 (iai-callgrind) → S431-05 (sccache/mold) → S431-06 (alloc profiling) → S431-07 (compile baseline) → S431-08 (async profiling)
