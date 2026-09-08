# Iteration Batch 807 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Zero-Copy Parsing (12)
- rkyv v0.8.17: Active, 32M downloads, but open soundness issue #670 (ArchivedRc validation bypass)
- bincode ecosystem fractured: bincode unmaintained (2025-12), bincode-next v3.1.1 (Apich fork), bincode_reloaded v3.1.15
- postcard v1.1.3: no_std-first, varint encoding, COBS framing, CRC
- FlatBuffers: 29× faster field access than JSON, 8× faster deserialization than Protobuf
- Cap'n Proto: Simpler encoding, 55% larger than Protobuf
- Zero-copy delivers 4-29× speedup depending on workload
- Serde zero-copy: only for &str and &[u8] fields via #[serde(borrow)]
- Yoke crate: lifetime-erased zero-copy by attaching borrowed data to a "cart"
- zerovec: ZeroVec for Vec<T> and HashMap replacements without lifetime pollution
- logos DFA lexer: 21× faster than regex for pattern-heavy parsing
- nosj: SIMD JSON parser, 2× faster than serde_json
- serde_bytes: avoid per-element serde overhead for binary data

### Error Handling (10)
- thiserror for libraries (typed, matchable), anyhow for application boundaries
- Never expose anyhow from library public APIs
- #[derive(Error)] over manual impl Display — zero boilerplate
- clippy::unwrap_used in production code — .unwrap() = attacker-triggerable panic
- Error classification: transient (retry), permanent (fail-fast), content policy (block)
- Circuit breaker + backoff + jitter — layered recovery, not flat retry loops
- error-forge crate: Structured RetryPolicy, CircuitBreaker, ExponentialBackoff
- #[source]/#[from] to preserve error chains
- Downcasting bridges generic handling with specific recovery
- String-smashing anti-pattern defeats typed errors entirely

### Testing Ecosystem (10)
- proptest 1.11.0: Property-based testing, 180M+ downloads, .proptest-regressions file
- cargo-mutants 27.1.0: Mutation testing, --in-diff for per-PR incremental, #[mutants::skip]
- criterion 0.8.2: Statistics-driven benchmarking, linear regression, bootstrap CI
- insta 1.48.0: Snapshot testing with inline/external, cargo-insta review
- cargo-fuzz 0.13.2: libFuzzer wrapper, corpus minimization, crash minimization
- expect-test: Minimalistic snapshot testing from rust-analyzer
- Arbitrary derive for structured fuzzing input generation
- Round-trip invariant oracles: decode(encode(x)) == x
- SelfTest impls lack property verification (111 impls, zero proptest)
- Criterion 0.8 has async benchmarking, improved statistics vs 0.5

### Observability Stack (10)
- OpenTelemetry SDK v0.32.1: Stable Metrics SDK, batch span/log export
- tracing 0.1.44: 731M downloads, structured logging, #[instrument]
- tracing-opentelemetry: Bridge from tracing → OTLP exporter
- opentelemetry-prometheus: Metrics export to Prometheus (pull model)
- Panelist: Type-safe Grafana dashboards in Rust (schema v41, promql!/loki! macros)
- Histogram vs summary PromQL mismatch is common defect
- #[instrument(skip(self), fields(order.id = %order_id))] auto-instrument pattern
- Layered subscriber: registry().with(env_filter).with(otel_layer).with(json_layer)
- OpenTelemetry Collector as sidecar: NeoTrix OTLP → Collector → Jaeger/Prometheus
- NT-SHIELD zero observability: 80+ files with zero tracing::instrument, zero metrics

---

## Defects Identified (37+)

### Zero-Copy Parsing (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-ZC-1 | rkyv #670 soundness gap unpatched (ArchivedRc validation bypass) | High |
| D-ZC-2 | Zero-copy offered but never used (always from_bytes, not access) | High |
| D-ZC-3 | No mmap integration despite memmap2 dependency | High |
| D-ZC-4 | bincode dependency is dead (unmaintained 2025-12) | Medium |
| D-ZC-5 | Serde-only serialization, no binary path for hot data | Medium |
| D-ZC-6 | No Cow<'a, str> in KB schema types (zero-copy impossible) | Medium |
| D-ZC-7 | FlatBuffers/Cap'n Proto not evaluated for IPC | Low |

### Error Handling (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-1 | Duplicate error enums (4 independent types, structurally identical) | High |
| D-ERR-2 | No #[from]/#[source] chains (root cause lost in debugging) | High |
| D-ERR-3 | From<String> → Brain string-smashing anti-pattern | High |
| D-ERR-4 | Zero .context()/.with_context() in production code | Medium |
| D-ERR-5 | Triple error classification overlap (LlmError/ErrorType/ErrorCategory) | Medium |
| D-ERR-6 | ErrorRecoveryError siloed from unified error domain | Medium |
| D-ERR-7 | thiserror = "1" (outdated, should be v2) | Low |
| D-ERR-8 | Production unwrap() in embodied_emotion.rs:115 | Low |
| D-ERR-9 | String-based error classification (locale-dependent, case-sensitive) | Medium |
| D-ERR-10 | 30+ error enums without thiserror derive | High |

### Testing (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-TEST-1 | Zero property-based tests (no proptest in entire codebase) | High |
| D-TEST-2 | Zero mutation testing (no cargo-mutants) | High |
| D-TEST-3 | criterion 2 major versions behind (0.5 vs 0.8.2) | Medium |
| D-TEST-4 | Zero snapshot tests (no insta, no expect-test) | High |
| D-TEST-5 | Zero fuzz targets (no cargo-fuzz, no fuzz/ directory) | High |
| D-TEST-6 | 111 SelfTest impls lack property verification | High |
| D-TEST-7 | No --in-diff incremental mutation in CI | Medium |
| D-TEST-8 | Benchmarks not regression-gated (no critcmp) | Medium |
| D-TEST-9 | No arbitrary crate for structured fuzzing | Medium |
| D-TEST-10 | No invariant-based fuzzing oracles (round-trip checks) | High |

### Observability (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-OBS-1 | OpenTelemetry v0.27 (5 minor versions behind current v0.32.1) | High |
| D-OBS-2 | No metrics export (zero Prometheus integration, no /metrics endpoint) | High |
| D-OBS-3 | Dual logging system (eprintln! alongside tracing) | High |
| D-OBS-4 | NT-SHIELD zero observability (80+ files, zero tracing, zero metrics) | Critical |
| D-OBS-5 | OTel init destroys tracing subscriber (only one can survive) | High |
| D-OBS-6 | No trace-context propagation (no traceparent headers) | Medium |
| D-OBS-7 | No log-trace correlation (logs and traces disconnected) | Medium |
| D-OBS-8 | HeartbeatAggregator not metric-exported (in-process only) | High |
| D-OBS-9 | No dashboard-as-code (no Grafana dashboards provisioned) | High |
| D-OBS-10 | Missing tracing-subscriber features (json, fmt, registry) | Medium |

## Key Insights (This Batch)

1. **rkyv zero-copy is offered but never used**: `from_bytes` always deserializes into owned V. Never uses `rkyv::access` for true zero-copy. Plus memmap2 dependency exists but mmap never implemented.

2. **4 independent error types are structurally identical**: NeoTrixError, L1Error, CapabilityError, LlmError all have the same variants. From<String> → Brain maps every string error into one variant, defeating typed errors.

3. **Zero property-based tests in 9849-test suite**: No proptest, no mutation testing, no fuzzing. SelfTest impls are verified only by example-based tests that could miss correctness gaps.

4. **NT-SHIELD is completely blind**: 80+ files (sandbox, proxy, threat detection, stealth net) with zero tracing::instrument, zero metrics. Security-critical operations are invisible.

5. **Dual logging defeats observability**: Custom log_error!/log_warn! macros bypass tracing entirely, so OTel never sees those events. Only one subscriber can survive OTel init.

6. **Serde JSON is 2-5× slower than binary formats**: Hot-path KB data uses serde_json for everything. bincode-next or postcard would be significantly faster for numeric-heavy types.

7. **OpenTelemetry v0.27 is 5 versions behind**: Missing stable Metrics SDK, improved batch export, thiserror v2 compat.

8. **criterion 0.8.2 has async benchmarking**: NeoTrix uses 0.5 — two major versions behind.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 807 |
| New defects (this batch) | 37 |
| Cumulative defects | D01-D76304 |
| Research sources (this batch) | 42+ |
| Cumulative research sources | 97,038+ |
