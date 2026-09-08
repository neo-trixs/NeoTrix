# Iteration Batch 850 Report — NeoTrix Consciousness Architecture

## Research Sources (38+)

### Observability (10)
- tracing 0.1.44: 731M downloads, structured spans + events
- OpenTelemetry Rust 0.32: Metrics/Logs API stable, Traces Beta, OTLP Exporters RC
- tracing-opentelemetry 0.33: bridges tracing spans → OTel traces
- opentelemetry-appender-tracing: routes tracing::info! → OTel Logs pipeline
- metrics crate 0.24: zero-cost facade, Prometheus/OTel/StatsD exporters
- metrics-exporter-otel 0.3.1: bridges metrics → OTel collectors
- W3C Trace Context Level 3: traceparent/tracestate headers
- OTel Context Propagation: FutureExt::with_context for async
- OneUptime structured logs recipe
- NeoTrix has dual logging (100+ log::* callsites bypass tracing subscriber)

### Web Frameworks (8)
- Axum 0.8.9: Tower-native, 401M+ downloads, native async traits
- Actix Web 4.14: 75M+ downloads, 10-15% throughput advantage
- Tower 0.5.3: Service + Layer traits, protocol-agnostic middleware
- Hyper 1.11: Cloudflare found race condition in HTTP/1 dispatch
- Warp 0.4.3: declining adoption
- 2026 comparison: Axum pragmatic default, Actix for max throughput
- NeoTrix already on Axum 0.8 — correct choice
- 10 defects identified: no Tower layer composition, Mutex blocking in async, no graceful shutdown

### WASM (10)
- Wasmtime v47.0.3: GC + Exception-Handling enabled by default
- WASI 0.3.1: native async, stream/future primitives, wasi:io removed
- wasm-bindgen 0.2.127: exnref exception handling default
- Component Model: WIT as stable IDL, semver-dedup of imports
- WASM Threads Phase 5 shipped; shared-everything-threads proposal active
- NeoTrix on wasmtime 42 (5 versions behind)
- wasi-threads removal is breaking change
- No WASI P3 / async component model
- No Component Model integration
- Egress policy not wired to WASM sandbox

### Testing (10)
- proptest 1.11: stronger default for production Rust, Hypothesis-style shrinking
- cargo-fuzz 0.13.1: targets panics, logic errors, unsafe code bugs
- cargo-mutants 27.1.0: ThoughtWorks Radar Trial, injection testing
- quickcheck superseded by proptest
- NeoTrix has zero adoption of any advanced testing tools
- SelfTest quality unverified (no mutation kill-rate)
- No regression persistence (.proptest-regressions or fuzz corpus)
- SEAL pipeline untested for convergence
- No parser robustness testing
- No invariant verification for KB operations

---

## Defects Identified (32+)

### Observability (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-OBS-1 | Dual logging (100+ log::* bypass tracing) | Critical |
| D-OBS-2 | init_tracing() dead code (never invoked) | Critical |
| D-OBS-3 | init_otel() and init_tracing() both call .init() (panic) | Critical |
| D-OBS-4 | OpenTelemetry deps pinned to 0.27 (current 0.32) | High |
| D-OBS-5 | telemetry feature not in default/full (OTel dead) | High |
| D-OBS-6 | No metrics crate integration (zero observability) | High |
| D-OBS-7 | No structured field logging (100+ callsites) | High |
| D-OBS-8 | No W3C TraceContext propagation | Medium |
| D-OBS-9 | No #[instrument] annotations | Medium |
| D-OBS-10 | No log→trace correlation | Medium |
| D-OBS-11 | Custom AtomicU8 level filter (dead weight) | Low |
| D-OBS-12 | log_*! macros use $super:: path | Low |

### Web Frameworks (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-WEB-1 | No Tower Layer composition | High |
| D-WEB-2 | Mutex<> blocking in async AppState | High |
| D-WEB-3 | No graceful shutdown | High |
| D-WEB-4 | Fixed-window rate limiter (burst at boundary) | Medium |
| D-WEB-5 | SSE stream pseudo-streaming | Medium |
| D-WEB-6 | No compression layer | Low |

### WASM (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-WASM-1 | Wasmtime 42 (5 versions behind) | High |
| D-WASM-2 | wasi-threads removal breaking change | High |
| D-WASM-3 | No WASI P3 async component model | High |
| D-WASM-4 | No Component Model integration | High |
| D-WASM-5 | Egress policy not wired to sandbox | Medium |
| D-WASM-6 | No resource limits on sandbox | Medium |
| D-WASM-7 | exec_wasm is a stub | Medium |

### Testing (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-TEST-1 | Zero proptest adoption | High |
| D-TEST-2 | Zero cargo-fuzz adoption | High |
| D-TEST-3 | Zero cargo-mutants adoption | Medium |
| D-TEST-4 | SelfTest quality unverified | High |
| D-TEST-5 | No regression persistence | Medium |
| D-TEST-6 | SEAL pipeline untested for convergence | High |
| D-TEST-7 | No parser robustness testing | High |

## Key Insights (This Batch)

1. **Dual logging is a critical bug**: 100+ log::* callsites bypass tracing subscriber entirely. init_tracing() is dead code. Must unify to tracing.

2. **OpenTelemetry 0.27 → 0.32 is breaking**: TracerProvider builder pattern changed, OTLP exporter builder changed. Must upgrade.

3. **metrics crate is zero-cost facade**: Counter/Gauge/Histogram with Prometheus/OTel exporters. Essential for production observability.

4. **Axum is correct choice for NT-IO**: Tower-native, forbid(unsafe_code), native async traits. Fix middleware/state issues, not framework.

5. **wasmtime 42 → 47 is mandatory**: GC + EH on by default, Spectre mitigations, wasi-threads removal. Must upgrade.

6. **WASI P3 is the future**: Native async, stream/future primitives. NT-SHIELD sandbox must adopt for long-running agent tasks.

7. **Component Model with WIT**: WIT is the stable IDL for WASM. Must integrate for sandbox interfaces.

8. **proptest is stronger than quickcheck**: Hypothesis-style shrinking, state-machine module, regression persistence. Must adopt.

9. **cargo-mutants verifies test quality**: Coverage tells what code is reached; mutation testing tells what code is checked. Essential for SelfTest.

10. **No graceful shutdown in Axum**: Hard kill loses in-flight requests. Must add with_graceful_shutdown.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 850 |
| New defects (this batch) | 32 |
| Cumulative defects | D01-D77419 |
| Research sources (this batch) | 38 |
| Cumulative research sources | 98,415+ |
