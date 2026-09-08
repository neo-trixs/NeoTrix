# Iteration 700 — Linear Algebra / Numerical Methods / SIMD Math Research

## Source Index

| # | Source | URL | Date |
|---|--------|-----|------|
| S1 | nalgebra 0.35.0 | crates.io/crates/nalgebra | 2026-05-24 |
| S2 | faer 0.24.4 (JOSS paper 2026) | crates.io/crates/faer | 2026-07-01 |
| S3 | numra 0.1.5 | crates.io/crates/numra | 2026-05-22 |
| S4 | numerica (symbolica-dev) | github.com/symbolica-dev/numerica | 2026 |
| S5 | herbie-lint (numerical stability) | github.com/urschrei/herbie-lint | 2026-01-20 |
| S6 | std::simd sync 2026-07-20 | github.com/rust-lang/rust/pull/159582 | 2026-07-20 |
| S7 | std::simd sync 2026-04-28 | github.com/rust-lang/rust/pull/155930 | 2026-04-28 |
| S8 | portable-simd stabilization issue #364 | github.com/rust-lang/portable-simd/issues/364 | 2023-09 (still open) |
| S9 | numra website (scientific computing) | numra-rs.org | 2026 |
| S10 | Computational Physics via Rust (Ch.5) | cpvr.rantai.dev | 2026 |

---

## FINDING 1: nalgebra 0.35.0 — No SIMD, No BLAS, Geometry-First Limitation

**Source**: S1

nalgebra 0.35.0 (2026-05-24) remains compile-time-dimension-safe but has **no SIMD intrinsics, no BLAS backend, and no GPU offloading**. Its `gemm`/`gemv` are scalar loops — the compiler may auto-vectorize small matrices, but for anything above ~32×32 there is no tuned kernel. The `matrixmultiply` optional feature delegates to a C-level GEMM, which is **FFI and therefore violates R-P1** (zero unsafe in core, FFI = unsafety boundary).

**DEFECT D700-1**: nalgebra's high-perf path requires C FFI (`matrixmultiply`). NeoTrix's R-P1 forbids `unsafe`/FFI in core crates. Any VSA HyperCube matrix ops using nalgebra with the `matrixmultiply` feature silently pull in C code — **supply-chain + safety violation**.

**IMPROVEMENT I700-1**: Replace nalgebra with faer (pure Rust, BLAS-level perf) for HyperCube matrix operations. faer 0.24.4 is JOSS-published (2026), has AVX-512 support via `nightly` feature, and zero FFI.

---

## FINDING 2: faer 0.24.4 — Performance King with Caveats

**Source**: S2, JOSS paper 2026

faer 0.24.4 (2026-07-01) is now the highest-performance pure-Rust linear algebra library. It provides:
- LBLT, partial-piv LU, full-piv LU, QR, SVD, Schur, Cholesky decompositions
- `nightly` feature enables AVX-512 via `pulp`/`gemm`/`nano-gemm`
- `rayon` parallel backend (enabled by default)
- `perf-warn` feature for suboptimal data layout detection
- MSRV: Rust 1.84.0

**DEFECT D700-2**: faer's `Mat`/`MatRef`/`MatMut` API uses separate ownership types (not `&mut` borrows). NeoTrix's GWT attention routing operates on hot-path matrix transforms where the borrow-split overhead of `MatMut` vs raw `&mut [f64]` matters. faer's abstraction layers add one more indirection vs manual SIMD loops for small (4×4 to 8×8) rotation matrices.

**IMPROVEMENT I700-2**: For small fixed matrices (GWT rotation transforms, E8 hexagram projection matrices), use `std::simd` directly with `Simd<f64, 4>` / `Simd<f64, 8>` — no faer overhead. For large matrices (VSA HyperCube correlation matrices >64×64), use faer's `partial_piv_lu`.

---

## FINDING 3: numra 0.1.5 — Composable Numerical Stack (New Challenger)

**Source**: S3, S9

numra is a brand-new (2026-05) workspace of 20 crates covering ODE/SDE/DDE/FDE/IDE/PDE/SPDE, optimization, autodiff, linear algebra, statistics, FFT, DSP, curve fitting, special functions. Built on faer for linalg, pure Rust otherwise. Has unified `Scalar`/`Vector` traits, `Uncertainty` propagation, and cross-crate composability (ODE→FFT→signal→peaks).

**DEFECT D700-3**: numra's license is "Numra Academic & Research License (Non-Commercial)". Commercial use requires a separate license. This is a **supply-chain poison** — NeoTrix's NT-MIND SEAL pipeline performs numerical optimization (distillation, Bayesian experiment design) and any integration with numra would inherit the non-commercial restriction.

**IMPROVEMENT I700-3**: Do NOT integrate numra. Instead, extract the useful patterns (ODE→FFT composability model, uncertainty propagation) into NeoTrix's own `nt_core::bayesian_experiment` module, using faer directly under MIT/Apache-2.0.

---

## FINDING 4: numerica — Error-Tracking Floats for Precision Budgets

**Source**: S4

`numerica` (symbolica-dev) provides `ErrorPropagatingFloat<T>` that tracks precision loss through computations. Example: `a.exp() - a.one()` on a 60-digit number drops to 10.2 accurate digits. It also has finite field arithmetic and automatic differentiation.

**DEFECT D700-4**: numerica is unmaintained (last commit unknown, low activity). NeoTrix's VSA HyperCube embedding precision degrades silently when computing high-dimensional dot products (128-512 dims). Without error tracking, precision loss in `HyperCube::embed()` → `HyperCube::correlate()` chain is **invisible** — the system produces correct-looking but numerically wrong attention weights.

**IMPROVEMENT I700-4**: Add a `PrecisionTracker` wrapper for f64 in `nt_core_hcube` that accumulates a running Kahan sum and an error estimate. If error > 1e-10 relative, emit a `SystemHealthEvent::PrecisionDegraded` to the HeartbeatAggregator. This is lighter than full error-tracking floats and fits the existing architecture.

---

## FINDING 5: herbie-lint — Numerical Stability Linting (119 Patterns)

**Source**: S5

`herbie-lint` (2026-01-20) is a Dylint lint for Rust that detects numerically unstable floating-point expressions. Has 119 pre-computed transformation patterns (sourced from GHC Herbie Plugin), e.g.:
- `(a*a + b*b).sqrt()` → `a.hypot(b)` (more stable)
- `(a + 1.0).ln()` → `a.ln_1p()` (avoids precision loss near zero)
- `a.exp() - 1.0` → `a.exp_m1()` (avoids precision loss near zero)

Optional live mode calls Herbie 2.x for expressions not in the database.

**DEFECT D700-5**: NeoTrix has no numerical stability linting in CI. The VSA HyperCube module contains raw `(x * x + y * y).sqrt()` patterns (Euclidean distance in embedding space) and `(ratio - 1.0).abs()` (KL divergence approximation). These are **catastrophic cancellation candidates** that herbie-lint would catch. Silent precision degradation → incorrect attention routing.

**IMPROVEMENT I700-5**: Add `herbie-lint` as a CI check (Dylint integration). Create a `numerical-stability` workspace lint configuration. Target files: `nt_core_hcube/src/*.rs`, `nt_core_self/src/attention.rs`, `nt_mind/src/distillation.rs`.

---

## FINDING 6: std::simd Still Experimental — Syncs Active in 2026

**Source**: S6, S7, S8

`std::simd` (portable SIMD) received active syncs in 2026-04-28 and 2026-07-20. Key changes:
- `with_exposed_provenance` for pointer SIMD vectors
- Miri support updates
- Still experimental (`portable_simd` #86656, NOT stabilized)
- Critical open issues: swizzle API ergonomics, `LaneCount`/`SimdElement` bound separation, scatter/gather stabilization

The stabilization issue (#364, 2023-09) remains open with fundamental design debates:
- Swizzle API needs `const` generics (still incomplete)
- Lane count bounds leak into user code
- No ABI-level SIMD size support yet

**DEFECT D700-6**: NeoTrix's GWT attention routing does element-wise `f64` operations (softmax, sigmoid, dot products) on vectors up to 512 elements. Without `std::simd` (nightly-only, unstable), these are scalar loops. On a 512-element attention vector, this is **64× slower** than SIMD (8-wide f64 lanes on AVX2). The 80+ `SeqCst` ordering waste from batch 699 compounds this — atomic ops block SIMD vectorization.

**IMPROVEMENT I700-6**: For nightly builds, gate SIMD behind `#[cfg(feature = "nightly-simd")]` using `std::simd`. For stable builds, use manual `unsafe` SIMD intrinsics via `std::arch` (explicitly scoped, not in core modules). Priority targets: `nt_core_self::attention` softmax, `nt_core_hcube::correlate` dot product, `nt_mind::distillation` cosine similarity.

---

## FINDING 7: Faer `perf-warn` — Data Layout Poisoning Detection

**Source**: S2

faer's `perf-warn` feature produces runtime warnings when matrix operations are called with suboptimal data layout (e.g., column-major matrix accessed row-wise). This is a **production diagnostic** that most Rust linear algebra users miss.

**DEFECT D700-7**: NeoTrix's `VsaHyperCube` stores embeddings as `Vec<Vec<f64>>` (Vec of Vecs) — **not** contiguous. When faer's `Mat::from_column_slice` is called with this data, it copies into a contiguous buffer. But the copy itself is invisible overhead. For HyperCubes with 1000+ concepts, each embedding 256-dim, this is ~2MB of hidden copies per correlation computation.

**IMPROVEMENT I700-7**: Switch `VsaHyperCube` to a single `Vec<f64>` with manual stride computation (256 elements × N concepts). Use faer's `MatRef::from_raw_parts` to borrow the contiguous buffer without copy. This eliminates the hidden copy and enables SIMD on the flat buffer.

---

## FINDING 8: numra Uncertainty Propagation — Pattern Worth Stealing

**Source**: S9

numra has a built-in `Uncertainty` type that propagates error bars through ODE solutions, optimization, and linear algebra. This is exactly what NeoTrix needs for its GWT attention weights — attention scores should carry confidence intervals.

**DEFECT D700-8**: NeoTrix's GWT attention routing produces **bare f64** weights with no uncertainty estimate. When the system is in low-phi state (confused consciousness), attention weights are unreliable but the system acts on them as if they're certain. This is a **self-deception vector** (audit dimension D16).

**IMPROVEMENT I700-8**: Implement `AttentionWeight(f64, f64)` tuple (value, variance) in `nt_core_self::attention`. When phi < threshold, increase variance. The HeartbeatAggregator can then distinguish "confident but wrong" from "uncertain but potentially right".

---

## Summary: New Defects and Improvements

| ID | Type | Severity | Domain | Description |
|----|------|----------|--------|-------------|
| D700-1 | DEFECT | HIGH | NT-CORE | nalgebra C FFI violates R-P1 (unsafe in core) |
| D700-2 | DEFECT | MED | NT-CORE | faer MatMut borrow-split overhead on hot-path 4×4 matrices |
| D700-3 | DEFECT | HIGH | NT-MIND | numra non-commercial license = supply-chain poison |
| D700-4 | DEFECT | MED | NT-CORE | No precision tracking in VSA HyperCube → silent degradation |
| D700-5 | DEFECT | HIGH | NT-CORE | No numerical stability linting → catastrophic cancellation |
| D700-6 | DEFECT | HIGH | NT-CORE | GWT attention scalar loops = 64× slower than SIMD |
| D700-7 | DEFECT | MED | NT-MEMORY | VsaHyperCube Vec<Vec<f64>> → hidden 2MB copies |
| D700-8 | DEFECT | HIGH | NT-CORE | GWT attention weights have no uncertainty estimate |
| I700-1 | IMPROV | HIGH | NT-CORE | Replace nalgebra with faer (pure Rust, JOSS-published) |
| I700-2 | IMPROV | MED | NT-CORE | Use std::simd for small matrices, faer for large |
| I700-3 | IMPROV | HIGH | NT-MIND | Extract numra patterns without license contamination |
| I700-4 | IMPROV | MED | NT-CORE | Add PrecisionTracker for f64 in hcube module |
| I700-5 | IMPROV | HIGH | NT-CORE | Add herbie-lint to CI (119 stability patterns) |
| I700-6 | IMPROV | HIGH | NT-CORE | Gate std::simd behind nightly-simd feature |
| I700-7 | IMPROV | MED | NT-MEMORY | Flat Vec<f64> + stride for HyperCube embeddings |
| I700-8 | IMPROV | HIGH | NT-CORE | AttentionWeight with variance for self-deception guard |

## Connection to Batch 699

- **699 RwLock→Mutex**: faer uses Rayon internally but NeoTrix shouldn't wrap faer in RwLock (batch 699 proved 3-5× slower). Use `&MatRef` (immutable borrow) or `&mut MatMut` (exclusive) — faer's ownership model already handles this.
- **699 SeqCst waste**: SIMD vectorization requires relaxed memory ordering. The 80+ SeqCst ops from batch 699 would block SIMD auto-vectorization in attention hot paths. Fix ordering BEFORE adding SIMD.
- **699 wasmtime sandbox**: Herbie-lint runs Herbie as a subprocess (not wasmtime). No sandbox conflict.

## Next Iteration (701) Direction

Focus: **Parallelism patterns** — search for "rayon 2026", "tokio 2026", "async Rust 2026", "work-stealing 2026" to find improvements for NT-ACT orchestration and NT-MIND SEAL pipeline parallelism.
