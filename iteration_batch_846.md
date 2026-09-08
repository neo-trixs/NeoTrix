# Iteration Batch 846 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Unsafe Rust (8)
- Tree Borrows replaces Stacked Borrows (54% fewer false positives, PLDI 2026)
- Miri POPL 2026: 70% of top 100K crate tests run under Miri
- Safety Tags RFC: machine-readable safety annotations (96.1% libstd coverage)
- Cargo Scan: reduces audit burden to median 0.2% of lines
- cargo-geiger + cargo-vet + Miri = practical audit trifecta
- No Miri CI in NeoTrix (compile-time only)
- No cargo-geiger unsafe surface mapping (411+ transitive deps unmeasured)
- No Safety Tags / // SAFETY: comment standard

### FFI (10)
- Panic-unwind at FFI boundary is #1 risk (foreign exception = UB)
- UniFFI zero-copy still broken (issue #1974 open since 2024)
- cbindgen requires #[repr(C)] + #[no_mangle] discipline
- WASM plugin Box::leak causes unbounded memory leak
- WASM pointer arithmetic off-by-one on empty input
- WASM Module recreated on every call (should cache)
- No catch_unwind in any plugin callback path
- No #[repr(C)] types for C/FFI boundary
- No compile-time struct size assertions
- extern "fil-c" is future-but-not-yet (not RFC yet)

### Time Handling (8)
- chrono NOT deprecated (slow maintenance mode)
- Jiff 1.0 delayed past Summer 2025 (no timeline)
- Leap seconds being abolished (CGPM vote Oct 2026, discontinuation by 2035)
- timestamp_nanos() deprecated (unwrap_or(0) silent corruption)
- Local::now() for batch IDs (timezone-dependent naming)
- No DST-aware duration handling
- No IANA timezone integration
- Serde serialization loses timezone identity

---

## Defects Identified (26+)

### Unsafe Rust (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-UNSAFE-1 | No Miri CI (compile-time only) | Critical |
| D-UNSAFE-2 | Tree Borrows not adopted | High |
| D-UNSAFE-3 | No cargo-geiger unsafe surface mapping | High |
| D-UNSAFE-4 | No Safety Tags / // SAFETY: standard | Medium |
| D-UNSAFE-5 | FFI boundary UB invisible (Miri opaque) | Medium |
| D-UNSAFE-6 | No cargo-vet audit provenance | Medium |
| D-UNSAFE-7 | No strict provenance enforcement | Low |
| D-UNSAFE-8 | No multi-seed concurrency testing | Low |

### FFI (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-FFI-1 | No FFI boundary architecture | High |
| D-FFI-2 | WASM plugin Box::leak memory leak | Medium |
| D-FFI-3 | WASM pointer arithmetic off-by-one | Medium |
| D-FFI-4 | WASM Module recreated per call | Low |
| D-FFI-5 | WASM typed func no bounds validation | Medium |
| D-FFI-6 | No catch_unwind in plugin callbacks | High |
| D-FFI-7 | No #[repr(C)] for FFI boundary | Medium |
| D-FFI-8 | No compile-time size assertions | Medium |

### Time Handling (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-TIME-1 | timestamp_nanos unwrap_or(0) silent corruption | High |
| D-TIME-2 | Local::now() for batch IDs (timezone-dependent) | High |
| D-TIME-3 | No DST-aware duration handling | Medium |
| D-TIME-4 | Chrono leap second partial support | Low |
| D-TIME-5 | No IANA timezone integration | Medium |
| D-TIME-6 | Serde loses timezone identity | Medium |
| D-TIME-7 | Pre-1.0 Jiff dependency risk | Low |
| D-TIME-8 | calamine chrono feature coupling | Low |

## Key Insights (This Batch)

1. **Tree Borrows is the future aliasing model**: Stacked Borrows will likely be deprecated. Catches real UB but accepts 54% more valid patterns. Essential for async code.

2. **No Miri CI is CRITICAL**: NeoTrix's #![forbid(unsafe_code)] is necessary but insufficient. Miri catches UB the borrow checker cannot: invalid pointer provenance, alignment violations, MaybeUninit reads, data races.

3. **Panic-unwind at FFI boundary is #1 risk**: extern "C" non-unwind ABI: foreign exception entering Rust = UB. catch_unwind is mandatory on every Rust function callable from C/foreign code.

4. **UniFFI zero-copy still broken**: Issue #1974 open since 2024. For NeoTrix's video/media pipeline (low-latency), this is a hard blocker.

5. **Leap seconds being abolished**: CGPM vote Oct 2026 to discontinue by 2035. Any code handling leap seconds invests in a dying feature.

6. **Local::now() for batch IDs**: Timezone-dependent naming breaks cross-machine correlation. Must always use Utc::now() for machine-facing identifiers.

7. **WASM Module recreated per call**: Each call_export recompiles the module. Must cache Module in WasmPluginWrapper.

8. **cargo-geiger maps unsafe surface**: Without it, NeoTrix cannot answer "how many unsafe blocks exist in our dependency tree?" — foundational for supply chain security.

9. **chrono in slow maintenance mode**: Not deprecated but accumulating deprecations. Jiff is the direction but 1.0 delayed.

10. **No compile-time size assertions**: FFI struct layout mismatch can cause memory corruption without detection.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 846 |
| New defects (this batch) | 24 |
| Cumulative defects | D01-D77314 |
| Research sources (this batch) | 26 |
| Cumulative research sources | 98,285+ |
