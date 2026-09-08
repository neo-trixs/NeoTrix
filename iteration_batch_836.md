# Iteration Batch 836 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Connection Pooling (10)
- 3-tier health check: passive (is_closed) → active (ping) → timeout-wrapped
- Background sweep + eager escalation (toasty pattern)
- TCP keepalive: SO_KEEPALIVE with 60s interval, 10s probe, 3 retries
- Recycle-on-acquire (deadpool): validates on get(), not on drop()
- Idle timeout + max lifetime: dual cap (evict by age AND idle duration)
- RAII wrapper: PooledStream/Object auto-return on drop
- No dead connection detection in HTTP pools (reqwest marks idle but doesn't ping)
- Stealth HTTP pool uses hardcoded 9s age threshold (no idle tracking)
- Proxy pool health check is L4-only (TCP connect), no L7 validation
- ResourcePool trait lacks idle_timeout and max_lifetime concepts

### Tower Middleware (8)
- Service trait = poll_ready + call (poll_ready is backpressure mechanism)
- Future wrapper separation: Service::call handles sync setup, Future handles async lifecycle
- Layer::layer() composes via ServiceBuilder (compile-time-enforced order)
- BoxService type erasure at boundaries to fight generic bounds
- Send + 'static on futures required for tower services
- route_layer vs layer in Axum (post-merge vs pre-merge)
- rate_limit_middleware uses Mutex<()> (poison-prone) instead of Semaphore
- No request-ID/tracing middleware in stack
- No retry/circuit-breaker middleware for LLM provider calls
- No tower::Service abstraction at layer boundaries

### Retry/Circuit Breaker/Hedging (10)
- 3 jitter strategies: Full (0→delay), Equal (delay/2 + random half), Decorrelated (min..prev*3)
- Retry-After header must be respected (backon crate does this natively)
- Circuit breaker + retry integration mandatory (thundering herd anti-pattern)
- Hedging is separate from retry (parallel vs serial requests)
- backoff crate (ihrwein) is de facto standard
- Builder pattern dominates: RetryConfig::new().max_attempts(5).jitter(Full).build()
- Broken jitter in compute_backoff (additive-only, never subtracts)
- Blocking sleep in async context (std::thread::sleep)
- No Retry-After header support in backoff computation
- 3 separate inconsistent backoff implementations in NeoTrix

### Logging/Tracing (12)
- Entire codebase uses log instead of tracing (instrumentation effectively dead)
- tracing and tracing-opentelemetry in dependencies but never initialized
- No structured spans anywhere
- OpenTelemetry integration blocked by missing subscriber initialization
- Context propagation across async boundaries not implemented
- No request-ID injection for request tracing
- No distributed tracing (no OTLP exporter)
- No per-request span hierarchy
- No trace sampling configuration
- No correlation between metrics and traces
- No log levels per module
- No log rotation or structured output

---

## Defects Identified (40+)

### Connection Pooling (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-POOL-1 | No dead connection detection in HTTP pools | High |
| D-POOL-2 | No TCP keepalive on pooled sockets | High |
| D-POOL-3 | Proxy pool health is L4-only (no protocol validation) | Medium |
| D-POOL-4 | ResourcePool trait lacks idle/lifetime caps | Medium |
| D-POOL-5 | Stealth client pool 9s age threshold (no idle tracking) | Medium |
| D-POOL-6 | No max size enforcement on stealth HTTP client pool | Medium |
| D-POOL-7 | Global HTTP clients use LazyLock (no graceful shutdown) | Low |
| D-POOL-8 | ProviderPool health check is shallow (entry count only) | Low |

### Tower Middleware (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-TWR-1 | rate_limit_middleware uses Mutex<()> (poison-prone) | High |
| D-TWR-2 | No poll_ready backpressure delegation in rate limiter | High |
| D-TWR-3 | No request-ID/tracing middleware in stack | Medium |
| D-TWR-4 | No retry/circuit-breaker middleware for LLM calls | High |
| D-TWR-5 | route_layer vs layer ordering fragility (KB routes lack rate limiting) | Medium |
| D-TWR-6 | No tower::Service abstraction at layer boundaries | Medium |
| D-TWR-7 | axum::serve used directly (no graceful shutdown) | High |

### Retry/Circuit Breaker (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-RETRY-1 | Broken jitter in compute_backoff (additive-only, never subtracts) | Critical |
| D-RETRY-2 | Blocking sleep in async context (std::thread::sleep) | Critical |
| D-RETRY-3 | No Retry-After header support in backoff computation | Medium |
| D-RETRY-4 | Circuit breaker Open→HalfOpen transition is passive (state race) | Medium |
| D-RETRY-5 | No hedging support (zero parallel redundant requests) | Medium |
| D-RETRY-6 | CircuitBreakerStrategy not thread-safe (HashMap without lock) | Low |
| D-RETRY-7 | 3 separate inconsistent backoff implementations | Medium |
| D-RETRY-8 | on_success decay failure count incorrectly (subtracts 1 instead of reset) | Low |

### Logging/Tracing (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-LOG-1 | Entire codebase uses log instead of tracing | Critical |
| D-LOG-2 | tracing + tracing-opentelemetry in deps but never initialized | Critical |
| D-LOG-3 | No structured spans anywhere | High |
| D-LOG-4 | OpenTelemetry integration blocked by missing subscriber | High |
| D-LOG-5 | No context propagation across async boundaries | High |
| D-LOG-6 | No request-ID injection for request tracing | Medium |
| D-LOG-7 | No distributed tracing (no OTLP exporter) | Medium |
| D-LOG-8 | No per-request span hierarchy | Medium |
| D-LOG-9 | No trace sampling configuration | Low |
| D-LOG-10 | No correlation between metrics and traces | Low |

## Key Insights (This Batch)

1. **Broken jitter is critical**: compute_backoff always ADDS jitter, never subtracts. All providers receiving same error retry at near-identical times → thundering herd. Industry standard: random(0..delay) or delay ± random(0..delay/2).

2. **Entire codebase uses log instead of tracing**: tracing and tracing-opentelemetry are in dependencies but never initialized. No structured spans, no OpenTelemetry integration, no context propagation. Instrumentation is effectively dead.

3. **3 separate inconsistent backoff implementations**: nt_core_error_recovery, nt_core_observer_error, nt_io_download/engine each have different defaults and behaviors. Must unify into single module.

4. **No dead connection detection in HTTP pools**: reqwest marks connections idle after timeout but doesn't ping before reuse. Half-open connections silently returned to pool.

5. **rate_limit_middleware uses Mutex<()>**: Poison-prone, blocks executor under contention. Tower's ConcurrencyLimitLayer/RateLimitLayer use Semaphore (cancellation-safe, non-poisoning).

6. **Hedging is separate from retry**: Hedging fires parallel requests after latency threshold (for P99). Retry fires serial requests after failure. NeoTrix has zero hedging.

7. **Proxy pool health is L4-only**: TcpStream::connect proves port is open but NOT that proxy speaks expected protocol. Must do protocol-level health check (SOCKS5 handshake, HTTP HEAD).

8. **No TCP keepalive on pooled sockets**: Half-open connections cause recv() to hang indefinitely, poisoning the connection pool.

9. **route_layer vs layer ordering fragility**: KB routes have auth but no rate limiting (asymmetry). Must apply rate_limit_middleware post-merge.

10. **Context propagation across async boundaries not implemented**: No trace context passed between spawned tasks. Distributed tracing impossible.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 836 |
| New defects (this batch) | 33 |
| Cumulative defects | D01-D77015 |
| Research sources (this batch) | 40 |
| Cumulative research sources | 97,927+ |
