# Iteration Batch 828 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Async Idempotency Patterns (10)
- Typestate (Processing→Completed) with pluggable stores and fencing tokens
- `aklanti/idempotent`: compile-time safety, fingerprint matching
- `axum-idempotent`: Idempotency-Key header mode, hashing mode, replay header
- `bhatti/idempotency-rs`: client-driven keys, atomic lock acquisition
- `mukundakatta/agentidemp`: deterministic content-derived keys (sha256_hex, uuid_v5)
- `scriptkid23/eventbus-rs`: effectively-once delivery, JetStream Nats-Msg-Id
- `nasa-runtime/naidempotency`: 4-state machine (FirstExecution/Replay/ConcurrentInFlight/FingerprintConflict)
- Defects: no standardized idempotency middleware, inconsistent key generation, race conditions, no body fingerprinting, in-memory store unsafe for multi-instance, no in-flight request tracking, no exactly-once for eventbus, decisions not routed through GWT, no standard error responses, no transactional outbox

### Cancellation Propagation (10)
- Parent→child cancellation via Rust ownership
- `tokio::select!` cancels all siblings when one branch completes
- Two-phase pattern (reserve + commit) most effective cancel-safe pattern
- Explicit cooperative cancellation tokens preferred over abort()
- Pin & reuse futures in select! loops — prevents drop/re-creation
- Spawn critical work as dedicated tasks — protect cancel-unsafe ops from parent cancellation
- Avoid holding mutexes across await points — cancellation breaks invariants
- Broadcast channels effective for root-to-leaf shutdown signal propagation
- Structured concurrency: parent scope owns children — abort parent, all children stop consistently
- Default tokens at entry points only, force propagation through internal methods

### Future Combinators (10)
- `then`: salience-update pipeline chaining; requires Result<Item, Error>, not f64 salience
- `and_then`: post-competition processing; same Result-bound issue
- `or_else`: fallback attention path; no "error" concept in GWT competition
- `select`: race cognitive specialists; requires Future + Unpin, conflicts with stateful GWT types
- `select!`: loop-based attention routing; requires FusedFuture, not implemented on GWT types
- `join` / `try_join`: parallel specialist execution; ad-hoc join_all usage, no unified pattern
- `select` output Either; GWT uses bare f64 salience scores, Either creates pattern-matching friction
- Defects: Unpin bound conflict, FusedFuture gap, Either output mismatch
- Recommendations: GWTFuture trait, FusedFuture for GWT types, then_compat/or_else_compat, integrate as SEAL pipeline phases, build nt-future crate

### Async `?` Ergonomics (10)
- `?` in async blocks requires Result or Option return type; compiler cannot infer error type otherwise
- `anyhow` ergonomics: Result<T, anyhow::Error> as return type; `?` propagates any std::error::Error impl
- `thiserror` for library/domain (typed errors, #[from] generated); `anyhow` for application/service layer
- 2026 production standard: `thiserror` for typed domain errors + `anyhow` for application propagation + `?` with `.context()` at boundaries
- Defects: async block `?` inference failure, error type erasure with Box<dyn Error>, missing From implementations at domain boundaries, backtrace overhead in hot SEAL paths, no_std incompatibility
- Recommendations: use anyhow::Result at application layer, thiserror at domain layer, bridge with From impl at boundaries, _lite variants for hot paths, upgrade with backtrace retroactively

---

## Defects Identified (30+)

### Async Idempotency (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-ID-1 | No standardized idempotency middleware for NT-ACT HTTP actions | Critical |
| D-ID-2 | Inconsistent idempotency key generation across modules | High |
| D-ID-3 | Race conditions in concurrent retries without atomic locks | High |
| D-ID-4 | No body fingerprinting for same-key retries | Medium |
| D-ID-5 | In-memory idempotency store unsafe for multi-instance deployments | Critical |
| D-ID-6 | No in-flight request tracking (lease pattern) | High |
| D-ID-7 | No effectively-once delivery for NT-WORLD eventbus | High |
| D-ID-8 | Idempotency decisions not routed through GWT attention | Medium |
| D-ID-9 | No standard error responses for conflict/conflict cases | Medium |
| D-ID-10 | No transactional outbox pattern for DB→event publishing | High |

### Cancellation Propagation (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CAN-1 | select! with biased in shutdown loop creates non-deterministic handler cancellation | Critical |
| D-CAN-2 | Local cancelled flag — no signal propagation through async chain | High |
| D-CAN-3 | cancel() only changes state to Cancelled — no mechanism to stop in-flight tasks | High |
| D-CAN-4 | tc.abort() synchronous flag check — no propagation to in-flight trajectory collection | Medium |
| D-CAN-5 | No cancellation scope for task groups — fire-and-forget tokio::spawn everywhere | Critical |
| D-CAN-6 | Agent loop replace local cancelled flag with propagating token | High |
| D-CAN-7 | Make TaskContract cancellation actually stop in-flight work | High |
| D-CAN-8 | PRM collector abort should interrupt trajectory pipeline | Medium |
| D-CAN-9 | Pin futures in select! loops instead of recreating | Medium |
| D-CAN-10 | Document cancel-safe vs cancel-unsafe with annotations | Low |

### Future Combinators (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-FUT-1 | Unpin bound conflict: select/select! require Future + Unpin, but GWT types carry !Unpin state | Critical |
| D-FUT-2 | FusedFuture gap: select! requires FusedFuture, GWT types don't fuse | Critical |
| D-FUT-3 | Either output mismatch: select returns Either<(A::Out,B),(B::Out,A)>, GWT uses bare f64 salience | Critical |
| D-FUT-4 | Then/and_then require Result-bound, but GWT uses f64 salience | Medium |
| D-FUT-5 | Need GWTFuture trait + FusedFuture for GWT types | High |

### Async `?` Ergonomics (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERG-1 | Async block `?` inference failure without explicit return type | High |
| D-ERG-2 | Error type erasure with Box<dyn Error> erases details, loses From chain | Medium |
| D-ERG-3 | Missing From implementations at domain boundaries causes `?` failures | High |
| D-ERG-4 | Backtrace overhead in hot SEAL paths (~4 us per error construction) | Medium |
| D-ERG-5 | no_std incompatibility with anyhow default features | Low |

## Key Insights (This Batch)

1. **No standardized idempotency middleware**: NT-ACT HTTP actions lack idempotency layer; retries can cause duplicate side effects (double payments, duplicate records). Opportunity/social lead flows lack dedup.

2. **Idempotency key generation inconsistent**: UUIDs, hashed keys, and plain strings used interchangeably across modules → same logical request gets different keys (missed dedup), different requests get same key (false dedup).

3. **Race conditions in concurrent retries**: Two concurrent retries both pass "key absent" check and both execute side effect. No fencing token or lease pattern found.

4. **select! with biased in shutdown loop**: Non-deterministic handler cancellation during shutdown. Shutdown future can be permanently starved by high-volume streams.

5. **Unpin bound conflict**: select/select! require Future + Unpin, but GWT types carry internal state (!Unpin). This blocks direct select! use in attention routing loops.

6. **FusedFuture gap**: select! requires FusedFuture on futures. GWT's cognitive operations don't fuse, causing unsound polling in loop contexts.

7. **Async `?` inference failure**: If NeoTrix uses `async { ... some_await?; ... }` without explicit return type, compiler cannot infer error type and rejects `?` with "cannot use `?` in async block that returns `()`".

8. **Error type erasure with Box<dyn Error>**: Using `Box` erases error details and loses `From` conversion chain, making `?` operator non-composable across async boundaries.

9. **Select output Either mismatch**: select returns Either<(A::Output, B), (B::Output, A)>, but GWT uses bare f64 salience scores. The nested Either creates pattern-matching friction against clean gwt_resonance API.

10. **No transactional outbox pattern**: Business data committed but events not published (or published twice). No outbox pattern observed in NT-ACT.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 828 |
| New defects (this batch) | 28 |
| Cumulative defects | D01-D76786 |
| Research sources (this batch) | 32 |
| Cumulative research sources | 97,655+ |