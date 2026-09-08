# Iteration Batch 888 — Sources 99727-99758

## Testing Patterns (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99727 | proptest v1.10 | github.com/proptest-rs/proptest | 2026 | 140M+ downloads; integrated shrinking; prop_assert! not assert! |
| 99728 | cargo-fuzz | github.com/rust-fuzz/cargo-fuzz | 2026 | fuzz_target! macro; arbitrary structured fuzzing; crash artifacts |
| 99729 | mockall v0.14 | github.com/asomers/mockall | 2026 | #[automock]; .times()/.once()/.never(); Sequence for ordering |
| 99730 | testcontainers-rs | github.com/testcontainers/testcontainers-rs | 2026 | GenericImage; SyncRunner/AsyncRunner; OnceCell reuse |
| 99731 | wiremock v0.6 | github.com/LukeMathWalker/wiremock-rs | 2025 | MockServer; .expect(n); request matchers; response templating |
| 99732 | insta v1.43 | github.com/mitsuhiko/insta | 2025 | assert_snapshot!; cargo insta review; redactions for volatile fields |
| 99733 | rstest v0.26 | github.com/la10736/rstest | 2026 | #[rstest] + #[case]; #[fixture]; #[awt] for async; file-based |
| 99734 | tokio-test | docs.rs/tokio/time/pause | 2026 | start_paused=true; time::advance(); tokio_test::io::Builder |

**Defect categories addressed**: D-TEST-001 to D-TEST-003, D-FUZZ-001 to D-FUZZ-005

## Observability (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99735 | tracing 0.1.44 | docs.rs/tracing | 2025 | #[instrument] + skip/fields/name; % for Display, ? for Debug |
| 99736 | OpenTelemetry Rust | github.com/open-telemetry/opentelemetry-rust | 2026 | Logs+Metrics Stable, Traces Beta; opentelemetry-appender-tracing |
| 99737 | metrics v0.24 | docs.rs/metrics | 2025 | Counter/Gauge/Histogram; facade pattern; atomic global recorder |
| 99738 | Prometheus exporter | docs.rs/metrics-exporter-prometheus | 2025 | PrometheusBuilder::install_recorder(); render() text format |
| 99739 | tracing-subscriber | docs.rs/tracing-subscriber | 2026 | Layer composition; EnvFilter; per-layer filtering |
| 99740 | W3C TraceContext | w3c.github.io/trace-context | 2024 | traceparent header; TraceContextPropagator; tokio::spawn no propagate |
| 99741 | Structured logging | reintech.io/blog/structured-logging-rust | 2026 | key=value not interpolation; lazy eval closures; avoid high-cardinality |
| 99742 | UUIDv7 fast | dev.to/fast-uuid-v7 | 2026-04 | 48-bit ms timestamp; fast-uuid-v7: 8-50ns (165x faster than OsRng) |

**Defect categories addressed**: D-OBS-001 to D-OBS-008, D-LOG-001 to D-LOG-004

## Concurrency Primitives (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99743 | DashMap | github.com/xacrimon/dashmap | 2026 | Sharded RwLock; entry() atomic; iteration locks ALL shards |
| 99744 | ArcSwap | github.com/vorner/arc-swap | 2026 | 120M ops/sec reads; lock-free always; Cache for thread-local |
| 99745 | parking_lot | github.com/Amanieu/parking_lot | 2026 | 1.5-50x faster than std; no poisoning; precise wake-up |
| 99746 | crossbeam-epoch | docs.rs/crossbeam-epoch | 2026 | pin(); guard.unlinked() for deferred free; Collector API |
| 99747 | Atomics ordering | doc.rust-lang.org/nomicon/atomics | 2026 | Relaxed/Acquire/Release/AcqRel/SeqCst; pair Release with Acquire |
| 99748 | Send + Sync | doc.rust-lang.org/nomicon/send-and-sync | 2026 | Send=move across threads; Sync=&T from threads; auto-derived |
| 99749 | Scoped threads | doc.rust-lang.org/std/thread/scope | 2026 | Guarantees join; borrows non-'static; eliminates Arc overhead |
| 99750 | Rayon | github.com/rayon-rs/rayon | 2026 | Work-stealing; par_iter(); 13K★; 266M downloads |

**Defect categories addressed**: D-SYNC-001 to D-SYNC-007, D-RACE-001 to D-RACE-004

## Async Streams (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99751 | async-stream | github.com/tokio-rs/async-stream | 2024 | stream!/try_stream! macros; for await inside blocks |
| 99752 | futures combinators | docs.rs/futures/stream | 2026 | buffer_unordered(n); chunks(n); filter async pitfall |
| 99753 | tokio-stream | docs.rs/tokio-stream | 2026 | ReceiverStream; BroadcastStream; StreamMap; chunks_timeout |
| 99754 | Stream trait impl | tokio.rs/tutorial/streams | 2026 | poll_next; Pin required; Unpin requirement |
| 99755 | BroadcastStream | docs.rs/tokio-stream/BroadcastStream | 2026 | filter_map(\|r\| r.ok()) to strip lag errors |
| 99756 | cancel-safe-futures | docs.rs/cancel-safe-futures | 2025 | for_each_concurrent_then_try; RobustMutex; reserve pattern |
| 99757 | buffer_unordered backpressure | docs.rs/futures/buffer_unordered | 2026 | Caps in-flight; completion order; natural backpressure |
| 99758 | async iteration | async-stream + StreamExt | 2026 | while let Some(v) = stream.next().await; fold for stateful |

**Defect categories addressed**: D-STREAM-001 to D-STREAM-010, D-BACK-001 to D-BACK-006

## New Defects (Batch 888)

### D-TEST-004 HIGH: Zero proptest usage in entire codebase
- **Evidence**: proptest 140M+ downloads; integrated shrinking finds edge cases automatically
- **Impact**: Edge cases (overflow, empty inputs, Unicode splits) found by users not tests
- **Fix**: Add proptest as dev-dependency; add property tests for parsers, serialization round-trips

### D-TEST-005 HIGH: Zero fuzz targets for parsers
- **Evidence**: cargo-fuzz: fuzz_target! macro with arbitrary structured inputs; crash artifacts become regression tests
- **Impact**: Panics in PDF/XML/CSV parsers with malformed input
- **Fix**: Add fuzz targets for PDF, XML, CSV/XLSX, encoding detection, rkyv deserialization

### D-OBS-009 HIGH: Zero OpenTelemetry integration
- **Evidence**: opentelemetry-appender-tracing bridges existing tracing spans→OTel; Logs+Metrics stable
- **Impact**: No distributed tracing; no metrics export; no vendor-neutral instrumentation
- **Fix**: Add opentelemetry-otlp exporter; bridge tracing via opentelemetry-appender-tracing

### D-OBS-010 HIGH: tracing-subscriber not configured with structured layers
- **Evidence**: tracing-subscriber: Layer composition + EnvFilter; per-layer filtering
- **Impact**: No structured logging; log levels not configurable at runtime
- **Fix**: Add tracing_subscriber::registry().with(fmt::layer().with_filter(EnvFilter::from_default_env())).init()

### D-SYNC-008 HIGH: 25+ Arc<RwLock<HashMap>> sites should use DashMap or ArcSwap
- **Evidence**: DashMap: sharded locking, entry() atomic; ArcSwap: 120M ops/sec lock-free reads
- **Impact**: Lock contention under high concurrency; global lock during iteration
- **Fix**: Replace read-mostly with ArcSwap; replace concurrent with DashMap; keep std for low-contention

### D-SYNC-009 MED: No scoped threads for parallel data processing
- **Evidence**: thread::scope guarantees join; borrows non-'static; eliminates Arc overhead
- **Impact**: Arc<RefCell> overhead for short-lived parallel work
- **Fix**: Use thread::scope for batch processing with split_at_mut disjoint slices

### D-STREAM-011 HIGH: Zero async-stream usage for custom stream creation
- **Evidence**: async-stream: stream!/try_stream! macros; canonical way to create custom streams
- **Impact**: Manual poll_next implementations are error-prone and verbose
- **Fix**: Replace manual Stream impls with async-stream macros

### D-LOG-005 HIGH: Zero request ID correlation across services
- **Evidence**: UUIDv7: 48-bit ms timestamp + 74 random; fast-uuid-v7: 8-50ns; attach to root span
- **Impact**: Cross-service debugging impossible without correlated request IDs
- **Fix**: Generate UUIDv7 at edge; attach to root tracing span; propagate in headers

### D-LOG-006 MED: String interpolation in log macros instead of key=value
- **Evidence**: tracing docs: info!(user_id = 123, "login") not info!("user {} logged in", 123)
- **Impact**: Logs not queryable in aggregation systems (ELK, DataDog)
- **Fix**: Replace all string interpolation with structured key=value fields

### D-DEPLOY-011 HIGH: No Clippy unwrap_used/expect_used restriction lints
- **Evidence**: Clippy restriction lints: cherry-pick unwrap_used, expect_used, panic, todo
- **Impact**: unwrap() in production code causes panics on edge cases
- **Fix**: Add #![warn(clippy::unwrap_used)] and #![warn(clippy::expect_used)]
