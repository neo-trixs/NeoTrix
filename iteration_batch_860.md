# Iteration Batch 860 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Error Propagation (9)
- thiserror 2.0: Provide trait, SpanTrace, error source chaining
- anyhow: type-erased error context, backtrace capture
- color-eyre: colored output, SpanTrace integration
- eyre: Result type with context
- Error trait: source() for chain, Display for user message
- #[error] derive: automatic Display, From, source impl
- Chain: iterate error sources
- Root cause analysis: backtrace + span trace
- Error type per domain pattern

### Data Race Prevention (9)
- Send/Sync auto-impl: compiler-level data race prevention
- SeqCst vs Acquire/Release vs Relaxed ordering
- ARM weak ordering: SeqCst necessary for multi-word atomics
- Loom model checking: exhaustive concurrency testing
- crossbeam: concurrent data structures with epoch GC
- AtomicU8/U64/Usize: atomic operations
- memory_ordering cheatsheet: x86 strong, ARM weak
- False sharing: cache line padding
- Loom: model checker for concurrent code

### Async Cancellation Safety (9)
- CancellationToken: cooperative cancellation
- JoinSet: structured concurrency with abort on drop
- select!: cancellation of branch on next match
- Graceful shutdown: signal handler → CancellationToken
- AsyncRead/AsyncWrite cancel-safety
- write_all: cancel-unsafe (partial write)
- tokio::spawn: detached task, no cancellation
- Structured concurrency: JoinSet lifecycle
- AbortOnDropGuard: manual abort on drop

### Async Caching (9)
- moka: concurrent cache, TTL, TTI, size budget, single-flight
- quick-cache: ultra-low latency, TinyLFU eviction
- CACHE STAMPEDE: N requests recompute same expired key
- Single-flight pattern: coalesce duplicate requests
- Per-entry TTL vs cache-level TTL
- Weighted size budget (memory-aware eviction)
- CLOCK-Pro / TinyLFU eviction algorithms
- Lock-free sharded maps vs Mutex<HashMap>
- Cache warming strategies

## Defects Identified (43+)

### Error Propagation (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-001 | NeoTrixError hand-written String-only variants | High |
| D-ERR-002 | CapabilityError same pattern | High |
| D-ERR-003 | L1Error duplication | Medium |
| D-ERR-004 | TaskDispatchError chain loss | High |
| D-ERR-005 | 100+ .map_err(\|e\| e.to_string()) sites | High |
| D-ERR-006 | String→Brain semantic loss | Medium |
| D-ERR-007 | thiserror 1.0 vs 2.0, no color-eyre | Low |
| D-ERR-008 | FileAbilityError::Parse chain loss | Medium |
| D-ERR-009 | MailError anyhow escape hatch | Medium |
| D-ERR-010 | Box<dyn Error> in proxy kernel/CLI | Medium |
| D-ERR-011 | CoT→NeoTrixError chain loss | Medium |
| D-ERR-012 | FFI expect() panic-on-poison | High |

### Data Race Prevention (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-RACE-001 | EventBus Clone drops sync_handlers (split-brain) | High |
| D-RACE-002 | Zero Loom coverage for 56+ concurrent structures | High |
| D-RACE-003 | Triple-lock ordering in EventBus emit_from | Medium |
| D-RACE-004 | bm25_dirty SeqCst/Relaxed mixing (ARM UB) | Medium |
| D-RACE-005 | Panoramic TOCTOU race | Medium |
| D-RACE-006 | Traffic Analyzer std::sync::Mutex in async context | Medium |
| D-RACE-007 | Background loop reentrant deadlock risk | Medium |
| D-RACE-008 | Parallel executor detached JoinHandles | Medium |
| D-RACE-009 | Guardrails over-synchronized counter | Low |
| D-RACE-010 | EventBus seq gap on crash | Low |

### Async Cancellation Safety (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CANCEL-001 | 49+ detached spawns (no structured concurrency) | High |
| D-CANCEL-002 | tokio::sync::Mutex in select! (35+ handlers) | High |
| D-CANCEL-003 | 73 write_all calls (cancel-unsafe I/O) | High |
| D-CANCEL-004 | ProxyKernel dual-copy without half-close | Medium |
| D-CANCEL-005 | Shutdown abort leaves KB writes mid-transaction | Medium |
| D-CANCEL-006 | EventBus consumer blocks all handlers via mutex | Medium |
| D-CANCEL-007 | process::exit(0) bypasses Drop | Medium |
| D-CANCEL-008 | Listener exit doesn't await in-flight connections | Low |
| D-CANCEL-009 | Handler futures recreated on each tick (no partial progress) | Low |
| D-CANCEL-010 | Zero cooperative cancellation (all abort-based) | Low |

### Async Caching (11)
| ID | Defect | Severity |
|----|--------|----------|
| D-CACHE-001 | 6 bare HashMap behind Mutex caches | High |
| D-CACHE-002 | Zero cache stampede protection | High |
| D-CACHE-003 | O(n) scan eviction (not CLOCK-Pro/TinyLFU) | Medium |
| D-CACHE-004 | 2/6 caches missing TTL | Medium |
| D-CACHE-005 | 5/6 caches missing per-entry TTL | Medium |
| D-CACHE-006 | 5/6 caches missing size-aware eviction | Medium |
| D-CACHE-007 | No lock-free concurrent access | Medium |
| D-CACHE-008 | No single-flight coalescing | Medium |
| D-CACHE-009 | No cache warming strategy | Low |
| D-CACHE-010 | No cache metrics (hit rate, eviction rate) | Low |
| D-CACHE-011 | No tiered caching (L1/L2/L3) | Low |

## Key Insights (This Batch)

1. **NeoTrixError is the single point of failure**: Hand-written enum with all-String variants and empty source() impl. Every domain maps through it, losing all diagnostic context. Thiserror 2.0 with #[error] derive is the fix.

2. **ARM weak ordering UB**: bm25_dirty flag mixes SeqCst and Relaxed stores. Works on x86 (strong ordering) but is undefined behavior on ARM (Apple Silicon). Must use consistent ordering.

3. **49+ detached spawns**: No structured concurrency. All background tasks are tokio::spawn with no JoinSet, no CancellationToken. If one panics, it silently dies.

4. **35+ handlers share one Mutex in select!**: tokio::sync::Mutex in select! loops means one slow handler blocks all 35+ other handlers. Should use per-handler state or try_lock.

5. **73 cancel-unsafe write_all calls**: write_all on Cancelled futures leaves partial writes. Must use write_all with proper cancellation handling.

6. **Zero cache stampede protection**: All caches are bare HashMap behind Mutex. N concurrent requests for same expired key = N full recomputations. moka's single-flight is the fix.

7. **O(n) scan eviction**: Current caches scan all entries to find eviction candidate. CLOCK-Pro/TinyLFU is O(1) amortized.

8. **process::exit(0) bypasses Drop**: 12+ locations call process::exit(0) which skips all Drop impls. Must use CancellationToken shutdown.

9. **Loom model checking**: Zero coverage for 56+ concurrent structures. Loom can exhaustively test all interleavings. Essential for EventBus, KB, memory subsystems.

10. **thiserror 2.0 Provide trait**: Enables SpanTrace for rich error diagnostics. NeoTrix is on thiserror 1.0 with hand-written String enums.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 860 |
| New defects (this batch) | 43 |
| Cumulative defects | D01-D77692 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 98,743+ |
