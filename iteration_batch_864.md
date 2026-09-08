# Iteration Batch 864 Report — NeoTrix Consciousness Architecture

## Research Sources (36+)

### Async Trait Dispatch (9)
- #[async_trait]: proc-macro boxing for dyn compatibility
- trait_variant::make(Send): auto-generates Send variant
- RPITIT: native async fn in traits (Rust 1.75+)
- BoxFuture<'static, T>: boxed future for dyn dispatch
- dyn Future<Output = T>: trait object for futures
- Send bound: requires future to be Send for tokio::spawn
- Static dispatch: monomorphization, zero overhead
- Dynamic dispatch: vtable lookup, heap allocation
- RTN (Return Type Notation): solves Send bounds (nightly)

### Connection Pool Management (9)
- reqwest::Client: HTTP client with connection pool
- hyper::client::Pool: connection reuse
- deadpool: async connection pool
- bb8: generic connection pool
- pool_max_idle_per_host: idle connection limit
- pool_idle_timeout: idle connection expiration
- tcp_nodelay: disable Nagle's algorithm
- http2_adaptive_window: HTTP/2 flow control
- CONNECT tunnel: proxy connection

### Async Cancellation Safety (9)
- CancellationToken: hierarchical cooperative cancellation
- JoinSet: structured concurrency with abort on drop
- AbortOnDropGuard: manual abort on drop
- select!: branch cancellation on match
- Graceful shutdown: signal → CancellationToken
- Cancel safety: future can be dropped at any await point
- write_all: cancel-unsafe (partial write)
- tokio::spawn: detached, no cancellation
- Watch channel: manual cancellation (no hierarchy)

### Error Type Design (9)
- thiserror: derive macro for error types
- anyhow: type-erased error context
- eyre: Result with context
- #[non_exhaustive]: forward-compatible error enums
- Error trait: source() for chain
- #[error]: automatic Display impl
- From impl: error conversion
- Context trait: .context("msg") for wrapping
- Root cause: backtrace + span trace

## Defects Identified (42+)

### Async Trait Dispatch (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-DISPATCH-001 | 30+ traits use #[async_trait] (heap alloc per call) | High |
| D-DISPATCH-002 | Zero trait_variant usage (pre-1.75 patterns) | High |
| D-DISPATCH-003 | Only 1 trait uses native RPITIT | Medium |
| D-DISPATCH-004 | LlmProvider+GatewayV2: 2 heap allocs + 2 vtable lookups per call | Medium |
| D-DISPATCH-005 | BackendRouter: triple indirection with double boxing | Medium |
| D-DISPATCH-006 | No Send bound mechanism for async trait methods | Medium |
| D-DISPATCH-007 | Box<dyn Future> everywhere instead of RPITIT | Low |
| D-DISPATCH-008 | No dyn compatibility analysis | Low |
| D-DISPATCH-009 | No dispatch overhead profiling | Low |
| D-DISPATCH-010 | No static dispatch optimization | Low |

### Connection Pool Management (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-POOL-001 | fetch_safe_http builds fresh Client per request | High |
| D-POOL-002 | probe_ollama/probe_llamacpp discard pools | Medium |
| D-POOL-003 | model_registry creates Client per provider | Medium |
| D-POOL-004 | Stealth net 9s TTL vs global 90s timeout | High |
| D-POOL-005 | Global client skips TLS verification + missing tcp_nodelay | High |
| D-POOL-006 | No deadpool/bb8 for database pooling | Medium |
| D-POOL-007 | Proxy-path clients not pooled across callers | Medium |
| D-POOL-008 | resolve_redirects_safely creates fresh client per hop | Medium |
| D-POOL-009 | No pool metrics or observability | Medium |
| D-POOL-010 | pool_max_idle_per_host=32 potentially excessive | Low |

### Async Cancellation Safety (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CANCEL-001 | Zero CancellationToken adoption | High |
| D-CANCEL-002 | Zero JoinSet usage (no structured concurrency) | High |
| D-CANCEL-003 | EventBus sync polling not cancellation-safe | Medium |
| D-CANCEL-004 | ~40 fire-and-forget spawns (no abort) | High |
| D-CANCEL-005 | SOCKS5 read_exact not cancel-safe | Medium |
| D-CANCEL-006 | Shutdown deadline race | Medium |
| D-CANCEL-007 | process::exit(0) bypasses Drop | High |
| D-CANCEL-008 | Shared deadline unfairness | Low |
| D-CANCEL-009 | Orphaned stdin write task | Low |
| D-CANCEL-010 | Sequential executor unkillable | Low |

### Error Type Design (12)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-001 | NeoTrixError god-enum with 18 string variants | High |
| D-ERR-002 | Brain(String) error sink loses diagnostics | High |
| D-ERR-003 | Three parallel unlinked error hierarchies | High |
| D-ERR-004 | TaskDispatchError chain loss | High |
| D-ERR-005 | Zero #[non_exhaustive] on error enums | High |
| D-ERR-006 | Inconsistent derivation (thiserror vs manual) | Medium |
| D-ERR-007 | No color-eyre / SpanTrace | Low |
| D-ERR-008 | No error context chaining anywhere | High |
| D-ERR-009 | FileAbilityError::Parse chain loss | Medium |
| D-ERR-010 | MailError anyhow escape hatch | Medium |
| D-ERR-011 | Box<dyn Error> in proxy kernel/CLI | Medium |
| D-ERR-012 | CoT→NeoTrixError chain loss | Medium |

## Key Insights (This Batch)

1. **30+ traits with #[async_trait]**: Each allocates a Box on every async method call. LlmProvider+GatewayV2 hot path has 2 heap allocs + 2 vtable lookups + 1 RwLock per LLM call. Worst case: 16 Box allocs per user request.

2. **Zero CancellationToken/JoinSet**: Entire codebase uses watch::Receiver<bool> and manual Vec<JoinHandle>. No hierarchical cancellation, no structured concurrency. ~40 fire-and-forget spawns.

3. **NeoTrixError god-enum**: 18 string variants with Brain(String) sink. Three parallel unlinked error hierarchies (core/L1/FFI). Zero #[non_exhaustive] for forward compatibility.

4. **Fresh Client per request**: fetch_safe_http builds new reqwest::Client per request. Connection pool is useless. Must centralize to shared Client.

5. **No error context chaining**: Zero .context("msg") calls anywhere. All errors lose diagnostic context. Must adopt thiserror + anyhow/eyre context pattern.

6. **process::exit(0) bypasses Drop**: 12+ locations call process::exit(0) which skips all Drop impls. Must use CancellationToken shutdown.

7. **Stealth net 9s TTL vs global 90s**: Connection pool TTL mismatch causes constant pool churn. Should align timeouts.

8. **Global client skips TLS verification**: danger_accept_invalid_certs(true) on global client. Security risk.

9. **No database connection pooling**: KB uses synchronous rusqlite without pooling. Must adopt deadpool or sqlx::Pool.

10. **RTN (Return Type Notation)**: Will solve Send bounds on async trait futures but blocked until late 2026. Must plan migration path.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 864 |
| New defects (this batch) | 42 |
| Cumulative defects | D01-D77855 |
| Research sources (this batch) | 36 |
| Cumulative research sources | 98,887+ |
