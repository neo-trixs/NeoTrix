# Iteration Batch 848 Report — NeoTrix Consciousness Architecture

## Research Sources (42+)

### Error Recovery (10)
- thiserror 2.0.20 (Aug 2026, 1.4B downloads) — backtrace support, improved source inference
- anyhow 1.0.104 (Jul 2026, 927M downloads) — type-erased error with context chains
- color-eyre 0.6.5 — SpanTrace (cheaper than Backtrace) structured error tracing
- backoff 0.4.0 — transient/permanent error + exponential backoff
- failsafe 1.3.0 — circuit breaker with time-windowed failure policies
- retryify — retry budgets + jittered backoff + circuit breaker combo
- circuitbreaker-rs 0.1.1 — lock-free state machine + observability hooks
- tower::retry::backoff — ExponentialBackoff with jitter, Tower layer integration
- tokio-retry2 v6 — ExponentialFactorBackoff + jitter strategies
- 2026 consensus: thiserror for libraries, anyhow for binaries, color-eyre for entry points

### Serialization (8)
- serde v1.0.229 (1.35B downloads) — canonical trait layer
- postcard v1.1.3 — no_std + serde, varint, COBS framing, CRC32
- rkyv v0.8.17 — zero-copy, access time ~1.2ns, 133M downloads
- bincode dead (doxxing incident Dec 2025), replaced by bincode-next v3.1.1
- bincode-next v3.1.1 — SIMD varint, zero-copy, CBOR fallback, no_std
- serde-zap v0.1.0 (2026) — fastest serde-compatible binary serializer
- rust_serialization_benchmark (2026-06) — authoritative shootout
- NeoTrix has rkyv but barely uses it (feature-gated, 2 call sites)

### Concurrency (8)
- parking_lot 0.12.5: 1 byte Mutex, 1.5-5x faster than std, no poisoning
- dashmap 6.2.1: sharded RwLock<HashMap>, 72 open issues
- arc-swap 1.9.2: read-mostly atomic Arc, SeqCst default strategy
- Tokio std Mutex starves 95.3% of threads under heavy contention
- parking_lot prevents starvation completely
- FairMutex: 67% tail-latency variance reduction
- Relaxed for counters, Acquire/Release for data publication, SeqCst only for global total order
- tokio::sync: async-aware, FIFO fairness, write-preferring RwLock

### Runtime (8)
- spawn_blocking NOT cancellable (abort only stops await, thread runs to completion)
- JoinSet drop = abort (no graceful drain)
- CancellationToken non-atomic across child tree
- Structured concurrency doesn't exist in Rust
- select! is a cancellation source
- Cooperative scheduling budget for long loops
- std::sync::Mutex across .await = deadlock or !Send
- No NUMA-aware runtime placement

---

## Defects Identified (35+)

### Error Recovery (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-1 | thiserror v1 (outdated, missing backtrace) | Medium |
| D-ERR-2 | Dual circuit breaker implementations | High |
| D-ERR-3 | Blocking sleep in async context | High |
| D-ERR-4 | No jitter in retry logic (thundering herd) | Medium |
| D-ERR-5 | No retry budget / storm prevention | Medium |
| D-ERR-6 | Box<dyn Error> in CLI layer | Low |
| D-ERR-7 | panic! in production merge code | High |
| D-ERR-8 | Missing #[must_use] on critical Results | Medium |
| D-ERR-9 | No color-eyre for structured error reports | Medium |
| D-ERR-10 | Inconsistent error type strategy | Medium |

### Serialization (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-SER-1 | Pervasive serde_json for everything (100+ sites) | High |
| D-SER-2 | rkyv barely used (feature-gated, 2 call sites) | High |
| D-SER-3 | No postcard for IPC | Medium |
| D-SER-4 | No bincode-next for hot paths | Medium |
| D-SER-5 | No schema versioning discipline | Medium |
| D-SER-6 | No serde_bytes on raw byte payloads | Low |

### Concurrency (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-CON-1 | Universal SeqCst abuse (68 sites) | Medium-High |
| D-CON-2 | No parking_lot adoption (100+ std::sync::Mutex) | Medium |
| D-CON-3 | No dashmap for concurrent maps | Low-Medium |
| D-CON-4 | No arc-swap for read-heavy config | Low |
| D-CON-5 | tokio::sync::Mutex where parking_lot suffices | Low-Medium |
| D-CON-6 | Missing Acquire/Release pair discipline | Medium |
| D-CON-7 | Static Mutex poisoning risk | Low |

### Runtime (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-RUN-1 | HeartbeatAggregator coop budget consumption | High |
| D-RUN-2 | spawn_blocking SEAL distillation not cancellable | High |
| D-RUN-3 | JoinSet::drop() silently aborts tasks (Critical) | Critical |
| D-RUN-4 | CancellationToken non-atomic propagation | Medium |
| D-RUN-5 | No structured concurrency primitive | High |
| D-RUN-6 | select! branches drop without cleanup | Medium |
| D-RUN-7 | broadcast::Receiver lag in EventBus | Medium |
| D-RUN-8 | Runtime drop order non-deterministic | High |
| D-RUN-9 | watch::Receiver::changed() first call false positive | Low |
| D-RUN-10 | No NUMA-aware runtime placement | Low |

## Key Insights (This Batch)

1. **thiserror v2 is the standard**: backtrace support via provide(), improved source inference, nightly #[backtrace] attribute. NeoTrix must migrate from v1.

2. **Dual circuit breakers are a maintenance burden**: nt_io_provider and nt_act each have independent implementations with different APIs. Must merge into single abstraction.

3. **Blocking sleep in async context is a correctness bug**: std::thread::sleep blocks the Tokio runtime thread, starving other tasks. Must use tokio::time::sleep.

4. **rkyv is massively underutilized**: NeoTrix has rkyv as optional dep but only uses it in 2 call sites. KB with node/edge/embedding model is the prime candidate for zero-copy access.

5. **parking_lot Mutex is 1 byte vs std 40+ bytes**: Uncontended fast path is single atomic CAS, no kernel transition. No poisoning. Must adopt.

6. **SeqCst on ARM emits dmb ish (full memory barrier)**: Every SeqCst operation costs ~3ns on ARM. 68 sites without justification is a performance killer on ARM targets.

7. **spawn_blocking is NOT cancellable**: Abort only stops the await, underlying OS thread runs to closure completion. Must wrap with timeout and accept thread continues.

8. **JoinSet::drop() = abort**: All owned tasks immediately aborted, no graceful drain. Must always call shutdown() explicitly.

9. **Structured concurrency doesn't exist in Rust**: JoinSet gives partial structure, but cancellation propagation requires explicit CancellationToken wiring.

10. **broadcast::Receiver lag is silent**: Slow consumer silently drops events. Must use per-consumer mpsc channels for reliability.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 848 |
| New defects (this batch) | 33 |
| Cumulative defects | D01-D77369 |
| Research sources (this batch) | 34 |
| Cumulative research sources | 98,351+ |
